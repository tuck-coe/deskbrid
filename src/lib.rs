//! deskbrid — The HAL your Linux desktop agents are missing.
//!
//! A standalone daemon that wraps Wayland protocols, DBus APIs, and PipeWire
//! into a JSON-over-Unix-socket protocol for agent-native desktop control.

pub mod protocol;
pub mod dbus;
pub mod input;
pub mod clipboard;
pub mod capture;
pub mod events;
pub mod config;

use anyhow::Result;
use std::path::PathBuf;
use tokio::net::UnixListener;

/// Default socket path: $XDG_RUNTIME_DIR/deskbrid/socket
pub fn default_socket_path() -> PathBuf {
    let runtime = std::env::var("XDG_RUNTIME_DIR")
        .unwrap_or_else(|_| "/run/user/1000".to_string());
    PathBuf::from(runtime).join("deskbrid").join("socket")
}

/// Version of the running daemon
pub const VERSION: &str = "0.1.0";

/// Main daemon entry point.
pub async fn run(config: config::Config) -> Result<()> {
    let socket_path = config.socket_path.unwrap_or_else(default_socket_path);

    // Ensure parent directory exists
    if let Some(parent) = socket_path.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }

    // Remove stale socket
    let _ = tokio::fs::remove_file(&socket_path).await;

    let listener = UnixListener::bind(&socket_path)?;
    tracing::info!("Listening on {}", socket_path.display());

    // Initialize subsystems
    let event_bus = events::EventBus::new();
    let input_session = input::InputSession::new().await?;
    let clipboard_monitor = clipboard::Monitor::new();
    let dbus_hub = dbus::Hub::new(event_bus.clone());

    // Spawn DBus event watchers
    tokio::spawn(dbus_hub.watch_windows(event_bus.clone()));
    tokio::spawn(dbus_hub.watch_notifications(event_bus.clone()));
    tokio::spawn(dbus_hub.watch_idle(event_bus.clone()));

    // Accept connections
    loop {
        let (stream, addr) = listener.accept().await?;
        let eb = event_bus.clone();
        let inp = input_session.clone();
        let cb = clipboard_monitor.clone();
        let dh = dbus_hub.clone();

        tokio::spawn(async move {
            tracing::debug!("Client connected: {:?}", addr);
            if let Err(e) = handle_client(stream, eb, inp, cb, dh).await {
                tracing::warn!("Client error: {e}");
            }
        });
    }
}

async fn handle_client(
    stream: tokio::net::UnixStream,
    event_bus: events::EventBus,
    input_session: input::InputSession,
    clipboard_monitor: clipboard::Monitor,
    dbus_hub: dbus::Hub,
) -> Result<()> {
    use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

    let (reader, mut writer) = stream.into_split();
    let mut reader = BufReader::new(reader);
    let mut line = String::new();

    // Send hello
    let hello = serde_json::json!({
        "type": "hello",
        "version": VERSION,
        "pid": std::process::id()
    });
    writer.write_all(format!("{hello}\n").as_bytes()).await?;

    let session = protocol::Session::new();

    loop {
        line.clear();
        let n = reader.read_line(&mut line).await?;
        if n == 0 {
            break; // connection closed
        }

        let msg: protocol::ClientMessage = match serde_json::from_str(&line) {
            Ok(m) => m,
            Err(e) => {
                let err = serde_json::json!({
                    "type": "result", "id": null, "ok": false,
                    "error": "invalid_params", "message": e.to_string()
                });
                writer.write_all(format!("{err}\n").as_bytes()).await?;
                continue;
            }
        };

        match msg {
            protocol::ClientMessage::Subscribe { events } => {
                for ev in events {
                    session.subscribe(&ev);
                }
                let ok = serde_json::json!({
                    "type": "result", "id": uuid::Uuid::new_v4(), "ok": true
                });
                writer.write_all(format!("{ok}\n").as_bytes()).await?;
            }
            protocol::ClientMessage::Unsubscribe { events } => {
                for ev in events {
                    session.unsubscribe(&ev);
                }
                let ok = serde_json::json!({
                    "type": "result", "id": uuid::Uuid::new_v4(), "ok": true
                });
                writer.write_all(format!("{ok}\n").as_bytes()).await?;
            }
            protocol::ClientMessage::Action { id, action, params } => {
                let result = dispatch_action(&action, params, &input_session, &clipboard_monitor, &dbus_hub).await;
                let resp = match result {
                    Ok(data) => serde_json::json!({
                        "type": "result", "id": id, "ok": true, "data": data
                    }),
                    Err(e) => serde_json::json!({
                        "type": "result", "id": id, "ok": false,
                        "error": "internal_error", "message": e.to_string()
                    }),
                };
                writer.write_all(format!("{resp}\n").as_bytes()).await?;
            }
        }
    }

    Ok(())
}

async fn dispatch_action(
    action: &str,
    params: serde_json::Value,
    input_session: &input::InputSession,
    _clipboard: &clipboard::Monitor,
    _dbus: &dbus::Hub,
) -> Result<serde_json::Value> {
    match action {
        "inject:type" => {
            let text = params["text"].as_str()
                .ok_or_else(|| anyhow::anyhow!("missing 'text' param"))?;
            input_session.type_text(text).await?;
            Ok(serde_json::json!({}))
        }
        "inject:key" => {
            let keys: Vec<String> = serde_json::from_value(params["keys"].clone())
                .map_err(|_| anyhow::anyhow!("missing or invalid 'keys' param"))?;
            input_session.send_keys(&keys).await?;
            Ok(serde_json::json!({}))
        }
        "inject:mouse" => {
            let kind = params["type"].as_str()
                .ok_or_else(|| anyhow::anyhow!("missing 'type' param"))?;
            let x = params["x"].as_f64().unwrap_or(0.0);
            let y = params["y"].as_f64().unwrap_or(0.0);
            input_session.mouse_action(kind, x, y).await?;
            Ok(serde_json::json!({}))
        }
        "info" => {
            Ok(serde_json::json!({
                "deskbrid_version": VERSION,
                "desktop": "GNOME",
                "session_type": "wayland",
                "capabilities": ["window", "inject", "clipboard", "screenshot",
                                 "screencast", "notifications", "display", "idle", "audio"]
            }))
        }
        other => {
            Err(anyhow::anyhow!("unknown action: {other}"))
        }
    }
}
