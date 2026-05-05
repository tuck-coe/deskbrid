//! deskbrid — The HAL your Linux desktop agents are missing.
//!
//! A standalone daemon that wraps Wayland protocols, DBus APIs, and PipeWire
//! into a JSON-over-Unix-socket protocol for agent-native desktop control.

pub mod capture;
pub mod clipboard;
pub mod config;
pub mod dbus;
pub mod events;
pub mod input;
pub mod protocol;

use anyhow::{anyhow, Context, Result};
use std::path::PathBuf;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{UnixListener, UnixStream};
use tracing::{debug, info, warn};

/// Default socket path: $XDG_RUNTIME_DIR/deskbrid/socket
pub fn default_socket_path() -> PathBuf {
    let runtime =
        std::env::var("XDG_RUNTIME_DIR").unwrap_or_else(|_| "/run/user/1000".to_string());
    PathBuf::from(runtime).join("deskbrid").join("socket")
}

/// Version of the running daemon
pub const VERSION: &str = "0.1.0";

/// Main daemon entry point.
pub async fn run(config: config::Config) -> Result<()> {
    let socket_path = config.socket_path.unwrap_or_else(default_socket_path);

    if let Some(parent) = socket_path.parent() {
        tokio::fs::create_dir_all(parent)
            .await
            .with_context(|| format!("creating socket dir {}", parent.display()))?;
    }

    let _ = tokio::fs::remove_file(&socket_path).await;

    let listener = UnixListener::bind(&socket_path)
        .with_context(|| format!("binding socket {}", socket_path.display()))?;
    info!("listening on {}", socket_path.display());

    let event_bus = events::EventBus::new();
    let input_session = input::InputSession::new().await?;
    let clipboard_monitor = clipboard::Monitor::new(event_bus.clone());
    let dbus_hub = dbus::Hub::new(event_bus.clone()).await?;

    tokio::spawn(dbus_hub.clone().watch_windows(event_bus.clone()));
    tokio::spawn(dbus_hub.clone().watch_notifications(event_bus.clone()));
    tokio::spawn(dbus_hub.clone().watch_idle(event_bus.clone()));

    loop {
        let (stream, addr) = listener.accept().await.context("accepting client")?;
        let eb = event_bus.clone();
        let inp = input_session.clone();
        let cb = clipboard_monitor.clone();
        let dh = dbus_hub.clone();

        tokio::spawn(async move {
            debug!("client connected: {:?}", addr);
            if let Err(error) = handle_client(stream, eb, inp, cb, dh).await {
                warn!("client error: {error:#}");
            }
        });
    }
}

async fn handle_client(
    stream: UnixStream,
    event_bus: events::EventBus,
    input_session: input::InputSession,
    clipboard_monitor: clipboard::Monitor,
    dbus_hub: dbus::Hub,
) -> Result<()> {
    let (reader, mut writer) = stream.into_split();
    let mut reader = BufReader::new(reader);
    let mut line = String::new();
    let mut subscriptions = protocol::Session::new();
    let mut events_rx = event_bus.subscribe();

    write_server_message(
        &mut writer,
        &protocol::ServerMessage::Hello {
            version: VERSION,
            pid: std::process::id(),
        },
    )
    .await?;

    loop {
        tokio::select! {
            read_result = reader.read_line(&mut line) => {
                let bytes = read_result.context("reading client message")?;
                if bytes == 0 {
                    break;
                }

                if bytes > 1024 * 1024 {
                    return Err(anyhow!("client message exceeds 1 MiB"));
                }

                let parsed = serde_json::from_str::<protocol::ClientMessage>(&line).map_err(|error| {
                    anyhow!("invalid client message: {error}")
                });
                line.clear();

                match parsed {
                    Ok(protocol::ClientMessage::Subscribe { events }) => {
                        for event in events {
                            subscriptions.subscribe(&event);
                        }
                        let response = protocol::ServerMessage::Result {
                            id: uuid::Uuid::new_v4().to_string(),
                            ok: true,
                            data: Some(serde_json::json!({})),
                            error: None,
                            message: None,
                        };
                        write_server_message(&mut writer, &response).await?;
                    }
                    Ok(protocol::ClientMessage::Unsubscribe { events }) => {
                        for event in events {
                            subscriptions.unsubscribe(&event);
                        }
                        let response = protocol::ServerMessage::Result {
                            id: uuid::Uuid::new_v4().to_string(),
                            ok: true,
                            data: Some(serde_json::json!({})),
                            error: None,
                            message: None,
                        };
                        write_server_message(&mut writer, &response).await?;
                    }
                    Ok(protocol::ClientMessage::Action { id, action, params }) => {
                        let response = match dispatch_action(&action, params, &input_session, &clipboard_monitor, &dbus_hub).await {
                            Ok(data) => protocol::ServerMessage::Result {
                                id,
                                ok: true,
                                data: Some(data),
                                error: None,
                                message: None,
                            },
                            Err(error) => protocol::ServerMessage::Result {
                                id,
                                ok: false,
                                data: None,
                                error: Some("internal_error".to_string()),
                                message: Some(error.to_string()),
                            },
                        };
                        write_server_message(&mut writer, &response).await?;
                    }
                    Err(error) => {
                        let response = protocol::ServerMessage::Result {
                            id: uuid::Uuid::new_v4().to_string(),
                            ok: false,
                            data: None,
                            error: Some("invalid_params".to_string()),
                            message: Some(error.to_string()),
                        };
                        write_server_message(&mut writer, &response).await?;
                    }
                }
            }
            event = events_rx.recv() => {
                match event {
                    Ok(event) if subscriptions.is_subscribed(&event.event_type) => {
                        let message = protocol::ServerMessage::Event {
                            event: event.event_type,
                            data: event.data,
                        };
                        write_server_message(&mut writer, &message).await?;
                    }
                    Ok(_) => {}
                    Err(tokio::sync::broadcast::error::RecvError::Lagged(skipped)) => {
                        warn!("client event stream lagged by {skipped} messages");
                    }
                    Err(tokio::sync::broadcast::error::RecvError::Closed) => break,
                }
            }
        }
    }

    Ok(())
}

async fn write_server_message(
    writer: &mut tokio::net::unix::OwnedWriteHalf,
    message: &protocol::ServerMessage,
) -> Result<()> {
    let encoded = serde_json::to_string(message).context("serializing server message")?;
    writer
        .write_all(encoded.as_bytes())
        .await
        .context("writing server message")?;
    writer.write_all(b"\n").await.context("writing newline")?;
    Ok(())
}

async fn dispatch_action(
    action: &str,
    params: serde_json::Value,
    input_session: &input::InputSession,
    clipboard: &clipboard::Monitor,
    dbus: &dbus::Hub,
) -> Result<serde_json::Value> {
    match action {
        "inject:type" => {
            let text = params
                .get("text")
                .and_then(serde_json::Value::as_str)
                .ok_or_else(|| anyhow!("missing 'text' param"))?;
            input_session.type_text(text).await?;
            Ok(serde_json::json!({}))
        }
        "inject:key" => {
            let keys: Vec<String> = serde_json::from_value(
                params
                    .get("keys")
                    .cloned()
                    .ok_or_else(|| anyhow!("missing 'keys' param"))?,
            )
            .context("invalid 'keys' param")?;
            input_session.send_keys(&keys).await?;
            Ok(serde_json::json!({}))
        }
        "inject:mouse" => {
            input_session.mouse_action(&params).await?;
            Ok(serde_json::json!({}))
        }
        "window:list" => {
            let windows = dbus.list_windows().await?;
            Ok(serde_json::json!({ "windows": windows }))
        }
        "window:focus" => {
            let app_id = params.get("app_id").and_then(serde_json::Value::as_str);
            let title = params.get("title").and_then(serde_json::Value::as_str);
            let exact = params
                .get("exact")
                .and_then(serde_json::Value::as_bool)
                .unwrap_or(false);
            dbus.focus_window(app_id, title, exact).await?;
            Ok(serde_json::json!({}))
        }
        "clipboard:read" => {
            let text = clipboard.read().await?;
            Ok(serde_json::json!({
                "text": text,
                "mime_types": ["text/plain"],
            }))
        }
        "clipboard:write" => {
            let text = params
                .get("text")
                .and_then(serde_json::Value::as_str)
                .ok_or_else(|| anyhow!("missing 'text' param"))?;
            clipboard.write(text).await?;
            Ok(serde_json::json!({}))
        }
        "notification:send" => {
            let summary = params
                .get("summary")
                .and_then(serde_json::Value::as_str)
                .ok_or_else(|| anyhow!("missing 'summary' param"))?;
            let body = params
                .get("body")
                .and_then(serde_json::Value::as_str)
                .unwrap_or_default();
            let urgency = params
                .get("urgency")
                .and_then(serde_json::Value::as_str)
                .unwrap_or("normal");
            let id = dbus.send_notification(summary, body, urgency).await?;
            Ok(serde_json::json!({ "id": id }))
        }
        "screenshot" => {
            let monitor = params
                .get("monitor")
                .and_then(serde_json::Value::as_u64)
                .map(|value| value as u32);
            let path = capture::screenshot(monitor).await?;
            Ok(serde_json::json!({ "path": path }))
        }
        "screencast:start" => {
            let monitor = params
                .get("monitor")
                .and_then(serde_json::Value::as_u64)
                .unwrap_or(0) as u32;
            let framerate = params
                .get("framerate")
                .and_then(serde_json::Value::as_u64)
                .unwrap_or(15) as u32;
            let node_id = capture::start_screencast(monitor, framerate).await?;
            Ok(serde_json::json!({ "node_id": node_id }))
        }
        "screencast:stop" => {
            let node_id = params
                .get("node_id")
                .and_then(serde_json::Value::as_u64)
                .ok_or_else(|| anyhow!("missing 'node_id' param"))? as u32;
            capture::stop_screencast(node_id).await?;
            Ok(serde_json::json!({}))
        }
        "info" => Ok(serde_json::json!({
            "deskbrid_version": VERSION,
            "desktop": "GNOME",
            "session_type": "wayland",
            "capabilities": [
                "window",
                "inject",
                "clipboard",
                "screenshot",
                "screencast",
                "notifications",
                "display",
                "idle",
                "audio"
            ]
        })),
        other => Err(anyhow!("unknown action: {other}")),
    }
}
