use crate::DaemonState;
use crate::protocol::Action;
use tracing::warn;

use super::execute::execute_action;
use super::helpers::{not_supported_response, permission_denied_response};
use super::system::{execute_system_control_action, is_system_control_action};
use super::terminal::{execute_terminal_action, is_terminal_action};
use super::wait_for_condition;
use super::{AuditRecord, execute_audit_action, is_audit_action, record_audit_entry};

pub async fn dispatch_action(
    action: Action,
    state: &DaemonState,
    peer_uid: u32,
    seq: u64,
) -> serde_json::Value {
    let started = std::time::Instant::now();

    // Check permissions first
    if !state.permissions.check(peer_uid, &action) {
        let response = permission_denied_response(seq);
        audit_response(state, &action, peer_uid, seq, &response, started, None).await;
        return response;
    }
    for implied_action in implied_permission_actions(&action) {
        if !state.permissions.check(peer_uid, &implied_action) {
            let response = permission_denied_response(seq);
            audit_response(state, &action, peer_uid, seq, &response, started, None).await;
            return response;
        }
    }
    if let Action::WindowsActivateOrLaunch {
        command,
        workdir,
        env,
        ..
    } = &action
    {
        let process_start = Action::ProcessStart {
            command: command.clone(),
            workdir: workdir.clone(),
            env: env.clone(),
        };
        if !state.permissions.check(peer_uid, &process_start) {
            let response = permission_denied_response(seq);
            audit_response(state, &action, peer_uid, seq, &response, started, None).await;
            return response;
        }
    }

    if is_audit_action(&action) {
        let result = execute_audit_action(action.clone(), state).await;
        return action_response(state, &action, peer_uid, seq, result, started, None).await;
    }
    if is_system_control_action(&action) {
        let result = execute_system_control_action(action.clone(), state).await;
        return action_response(state, &action, peer_uid, seq, result, started, None).await;
    }
    if is_terminal_action(&action) {
        let result = execute_terminal_action(action.clone(), state).await;
        return action_response(state, &action, peer_uid, seq, result, started, None).await;
    }

    let backend = state.backend.read().await;
    let backend = match backend.as_ref() {
        Some(b) => b,
        None => {
            let response = not_supported_response(
                "no desktop backend loaded (start daemon inside a supported Linux session)",
                seq,
            );
            audit_response(state, &action, peer_uid, seq, &response, started, None).await;
            return response;
        }
    };

    let result = if let Action::WaitFor {
        condition,
        params,
        timeout_ms,
        interval_ms,
    } = &action
    {
        wait_for_condition(
            state,
            backend.as_ref(),
            condition,
            params.clone(),
            *timeout_ms,
            *interval_ms,
        )
        .await
    } else {
        execute_action(action.clone(), backend.as_ref()).await
    };
    action_response(state, &action, peer_uid, seq, result, started, None).await
}

async fn action_response(
    state: &DaemonState,
    action: &Action,
    peer_uid: u32,
    seq: u64,
    result: anyhow::Result<serde_json::Value>,
    started: std::time::Instant,
    dry_run: Option<bool>,
) -> serde_json::Value {
    let response = match result {
        Ok(data) => {
            emit_action_event(state, action, &data);
            serde_json::json!({
                "type": "response", "id": "action", "seq": seq, "status": "ok", "data": data
            })
        }
        Err(e) => {
            warn!("Action failed: {}", e);
            serde_json::json!({
                "type": "response", "id": "action", "seq": seq, "status": "error",
                "error": { "code": "INTERNAL_ERROR", "message": format!("{}", e) }
            })
        }
    };
    audit_response(state, action, peer_uid, seq, &response, started, dry_run).await;
    response
}

async fn audit_response(
    state: &DaemonState,
    action: &Action,
    peer_uid: u32,
    seq: u64,
    response: &serde_json::Value,
    started: std::time::Instant,
    dry_run: Option<bool>,
) {
    let status = response["status"].as_str().unwrap_or("unknown").to_string();
    let error = response
        .get("error")
        .and_then(|error| error.get("message"))
        .and_then(|message| message.as_str())
        .map(String::from);
    record_audit_entry(
        state,
        AuditRecord {
            seq,
            peer_uid,
            action_type: action.action_type().to_string(),
            status,
            duration_ms: started.elapsed().as_millis().try_into().unwrap_or(u64::MAX),
            error,
            dry_run,
        },
    )
    .await;
}

pub fn implied_permission_actions(action: &Action) -> Vec<Action> {
    match action {
        Action::LayoutProfileSave { .. } => {
            vec![
                Action::WindowsList,
                Action::WorkspacesList,
                Action::SystemInfo,
            ]
        }
        Action::LayoutProfileRestore { .. } => vec![
            Action::WindowsMoveResize {
                window_id: "profile".into(),
                x: 0,
                y: 0,
                width: 1,
                height: 1,
            },
            Action::WindowsMinimize("profile".into()),
            Action::WorkspaceSwitch(0),
            Action::WorkspaceMoveWindow {
                window_id: "profile".into(),
                workspace_id: 0,
                follow: false,
            },
        ],
        _ => Vec::new(),
    }
}

pub fn emit_action_event(state: &DaemonState, action: &Action, data: &serde_json::Value) {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let event = match action {
        // Use the resolved window ID from the response data when available,
        // so subscribers get the canonical ID, not the caller-provided selector.
        Action::WindowsFocus(_) => {
            let window_id = data
                .get("focused")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown")
                .to_string();
            Some(crate::protocol::DeskbridEvent::WindowFocused {
                window_id,
                timestamp: now,
            })
        }
        Action::WorkspaceSwitch(id) => Some(crate::protocol::DeskbridEvent::WorkspaceChanged {
            workspace_id: *id,
            timestamp: now,
        }),
        Action::WorkspaceMoveWindow {
            window_id,
            workspace_id,
            ..
        } => Some(crate::protocol::DeskbridEvent::WorkspaceWindowMoved {
            window_id: window_id.clone(),
            workspace_id: *workspace_id,
            timestamp: now,
        }),
        _ => None,
    };
    if let Some(evt) = event {
        let _ = state.event_tx.send(evt);
    }
    let _ = data;
}
