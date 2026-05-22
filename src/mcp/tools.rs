//! MCP tool implementations — bridges MCP tools to Deskbrid's backend and a11y modules.

use serde_json::json;
use std::sync::Arc;

use crate::DaemonState;

/// Backend-agnostic action execution helper.
async fn execute_action_str(
    state: &DaemonState,
    action_type: &str,
) -> anyhow::Result<serde_json::Value> {
    let action =
        crate::protocol::Action::from_json(&format!(r#"{{"type":"{}","id":"mcp"}}"#, action_type))
            .map(|(_, a)| a)?;
    let backend = state.backend.read().await;
    let backend = backend
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("no backend loaded"))?;
    crate::daemon::execute::execute_action(action, backend.as_ref(), state).await
}

pub async fn do_focus_window(
    state: &DaemonState,
    window_id: &str,
) -> anyhow::Result<serde_json::Value> {
    let action = crate::protocol::Action::WindowsFocus(window_id.to_string());
    let backend = state.backend.read().await;
    let backend = backend
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("no backend loaded"))?;
    crate::daemon::execute::execute_action(action, backend.as_ref(), state).await
}

pub async fn do_type_text(state: &DaemonState, text: &str) -> anyhow::Result<serde_json::Value> {
    let action = crate::protocol::Action::InputKeyboardType {
        text: text.to_string(),
    };
    let backend = state.backend.read().await;
    let backend = backend
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("no backend loaded"))?;
    crate::daemon::execute::execute_action(action, backend.as_ref(), state).await
}

pub async fn do_press_keys(
    state: &DaemonState,
    keys: &[String],
) -> anyhow::Result<serde_json::Value> {
    let action = crate::protocol::Action::InputKeyboardCombo {
        keys: keys.to_vec(),
    };
    let backend = state.backend.read().await;
    let backend = backend
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("no backend loaded"))?;
    crate::daemon::execute::execute_action(action, backend.as_ref(), state).await
}

pub async fn do_mouse_move(
    state: &DaemonState,
    x: f64,
    y: f64,
) -> anyhow::Result<serde_json::Value> {
    let action = crate::protocol::Action::InputMouse {
        action: "move".into(),
        x: Some(x),
        y: Some(y),
        button: None,
        dx: None,
        dy: None,
    };
    let backend = state.backend.read().await;
    let backend = backend
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("no backend loaded"))?;
    crate::daemon::execute::execute_action(action, backend.as_ref(), state).await
}

pub async fn do_mouse_click(
    state: &DaemonState,
    button: &str,
) -> anyhow::Result<serde_json::Value> {
    let action = crate::protocol::Action::InputMouse {
        action: "click".into(),
        x: None,
        y: None,
        button: Some(button.to_string()),
        dx: None,
        dy: None,
    };
    let backend = state.backend.read().await;
    let backend = backend
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("no backend loaded"))?;
    crate::daemon::execute::execute_action(action, backend.as_ref(), state).await
}

pub async fn do_screenshot(state: &DaemonState) -> anyhow::Result<serde_json::Value> {
    execute_action_str(state, "screenshot").await
}

pub async fn do_clipboard_read(state: &DaemonState) -> anyhow::Result<serde_json::Value> {
    execute_action_str(state, "clipboard.read").await
}

pub async fn do_clipboard_write(
    state: &DaemonState,
    text: &str,
) -> anyhow::Result<serde_json::Value> {
    let action = crate::protocol::Action::ClipboardWrite {
        text: text.to_string(),
    };
    let backend = state.backend.read().await;
    let backend = backend
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("no backend loaded"))?;
    crate::daemon::execute::execute_action(action, backend.as_ref(), state).await
}

// --- AT-SPI tools ---

pub async fn do_list_apps(state: &DaemonState) -> anyhow::Result<serde_json::Value> {
    let action = crate::protocol::Action::A11yListApps { limit: Some(50) };
    crate::daemon::execute::execute_action(
        action,
        &*state
            .backend
            .read()
            .await
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("no backend loaded"))?,
        state,
    )
    .await
}

pub async fn do_get_accessibility_tree(
    state: &DaemonState,
    app_name: Option<&str>,
    pid: Option<u32>,
    max_nodes: Option<usize>,
    max_depth: Option<u32>,
) -> anyhow::Result<serde_json::Value> {
    crate::a11y::tree::snapshot_tree(app_name, pid, max_nodes, max_depth).await
}

pub async fn do_perform_action(
    state: &DaemonState,
    object_ref: &str,
    action_name: Option<&str>,
) -> anyhow::Result<serde_json::Value> {
    let _ = state;
    crate::a11y::actions::perform_action(object_ref, action_name).await
}

pub async fn do_set_element_value(
    state: &DaemonState,
    object_ref: &str,
    value: &str,
) -> anyhow::Result<serde_json::Value> {
    let _ = state;
    crate::a11y::value::set_element_value(object_ref, value).await
}

pub async fn do_get_element_text(
    state: &DaemonState,
    object_ref: &str,
    max_chars: Option<i32>,
) -> anyhow::Result<serde_json::Value> {
    let _ = state;
    crate::a11y::value::get_element_text(object_ref, max_chars).await
}

pub async fn do_click_element(
    state: &DaemonState,
    object_ref: &str,
) -> anyhow::Result<serde_json::Value> {
    let _ = state;
    crate::a11y::actions::click_element(object_ref).await
}

pub async fn do_doctor(state: &DaemonState) -> anyhow::Result<serde_json::Value> {
    let _ = state;
    Ok(crate::a11y::setup::doctor_report().await)
}

pub async fn do_setup_accessibility(state: &DaemonState) -> anyhow::Result<serde_json::Value> {
    let _ = state;
    let enabled = crate::a11y::setup::enable_accessibility().await?;
    Ok(json!({"enabled": enabled}))
}

pub async fn do_capabilities(state: &DaemonState) -> anyhow::Result<serde_json::Value> {
    let backend = state.backend.read().await;
    let backend_name = backend.as_ref().map(|b| b.name()).unwrap_or("none");
    Ok(json!({
        "backend": backend_name,
        "tools": crate::protocol::Action::public_action_types(),
        "mcp_enabled": true,
        "atspi_enabled": crate::a11y::setup::check_accessibility_enabled().await,
    }))
}
