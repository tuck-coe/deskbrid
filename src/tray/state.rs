//! Tray update state and monitoring — version checks, daemon status, event loop.

use super::ui::DeskbridTray;
use ksni::Handle;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::time::sleep;
use tracing::{debug, info};

#[derive(Debug, Clone, Default)]
pub(crate) struct UpdateState {
    pub(crate) current_version: String,
    pub(crate) latest_version: String,
    pub(crate) update_available: bool,
    pub(crate) checked: bool,
    pub(crate) daemon_running: bool,
}

/// Check for updates from the daemon, update tray state.
pub(crate) async fn check_and_update(
    state: &Arc<Mutex<UpdateState>>,
    handle: Option<&Handle<DeskbridTray>>,
) -> anyhow::Result<()> {
    let runtime = std::env::var("XDG_RUNTIME_DIR")
        .unwrap_or_else(|_| format!("/run/user/{}", unsafe { libc::getuid() }));
    let sock = format!("{}/deskbrid.sock", runtime);

    let mut stream = tokio::net::UnixStream::connect(&sock).await?;

    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    let request = serde_json::json!({
        "type": "system.update",
        "id": "tray-check",
        "check": true,
        "force": false
    });
    stream
        .write_all(format!("{}\n", serde_json::to_string(&request)?).as_bytes())
        .await?;

    let mut buf = vec![0u8; 4096];
    let n = stream.read(&mut buf).await?;
    let response: serde_json::Value = serde_json::from_slice(&buf[..n])?;

    let update_available = response["data"]["update_available"]
        .as_bool()
        .unwrap_or(false);
    let current = response["data"]["current_version"]
        .as_str()
        .unwrap_or("?")
        .to_string();
    let latest = response["data"]["latest_version"]
        .as_str()
        .unwrap_or("?")
        .to_string();

    {
        let mut s = state.lock().unwrap();
        s.current_version = current;
        s.latest_version = latest;
        s.update_available = update_available;
        s.checked = true;
        s.daemon_running = true;
    }

    if let Some(h) = handle {
        h.update(|_| {}).await;
    }

    Ok(())
}

/// Check if daemon is running and update state.
pub(crate) async fn check_daemon_status(state: &Arc<Mutex<UpdateState>>) {
    let running = tokio::process::Command::new("systemctl")
        .args(["--user", "is-active", "--quiet", "deskbrid.service"])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .await
        .map(|s| s.success())
        .unwrap_or(false);

    let mut s = state.lock().unwrap();
    s.daemon_running = running;
}

/// Background loop: monitor daemon status and updates.
pub(crate) async fn event_loop(
    state: Arc<Mutex<UpdateState>>,
    handle: Arc<Mutex<Option<Handle<DeskbridTray>>>>,
    shutdown: Arc<tokio::sync::Notify>,
) {
    // Initial daemon check
    check_daemon_status(&state).await;

    // Initial update check if daemon is running
    {
        let running = state.lock().unwrap().daemon_running;
        if running {
            let h = { handle.lock().unwrap().clone() };
            if let Err(e) = check_and_update(&state, h.as_ref()).await {
                debug!("Initial update check failed: {e}");
            }
        }
    }

    // Refresh tray after initial checks
    {
        let h = handle.lock().unwrap().clone();
        if let Some(ref h) = h {
            h.update(|_| {}).await;
        }
    }

    loop {
        tokio::select! {
            _ = shutdown.notified() => {
                info!("Tray shutdown requested");
                break;
            }
            _ = sleep(Duration::from_secs(30)) => {
                // Periodic daemon status check
                check_daemon_status(&state).await;

                let running = state.lock().unwrap().daemon_running;
                if running {
                    let h = { handle.lock().unwrap().clone() };
                    if let Err(e) = check_and_update(&state, h.as_ref()).await {
                        debug!("Periodic check failed: {e}");
                    }
                }
            }
        }
    }
}
