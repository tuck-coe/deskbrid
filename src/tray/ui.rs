//! Tray UI — DeskbridTray struct, ksni::Tray implementation, and run() entry point.

use ksni::{Handle, ToolTip, TrayMethods};
use std::sync::{Arc, Mutex};
use tracing::{error, info, warn};

use super::state::{UpdateState, check_and_update, event_loop};

pub(crate) struct DeskbridTray {
    pub(crate) state: Arc<Mutex<UpdateState>>,
    pub(crate) handle: Arc<Mutex<Option<Handle<Self>>>>,
    pub(crate) shutdown: Arc<tokio::sync::Notify>,
}

impl DeskbridTray {
    fn new(shutdown: Arc<tokio::sync::Notify>) -> Self {
        Self {
            state: Arc::new(Mutex::new(UpdateState::default())),
            handle: Arc::new(Mutex::new(None)),
            shutdown,
        }
    }
}

impl ksni::Tray for DeskbridTray {
    fn id(&self) -> String {
        env!("CARGO_PKG_NAME").into()
    }

    fn title(&self) -> String {
        "Deskbrid".into()
    }

    fn icon_name(&self) -> String {
        let state = self.state.lock().unwrap();
        if !state.daemon_running {
            "process-stop".into()
        } else if state.update_available {
            "software-update-available".into()
        } else {
            "deskbrid".into()
        }
    }

    fn status(&self) -> ksni::Status {
        let state = self.state.lock().unwrap();
        if state.update_available {
            ksni::Status::NeedsAttention
        } else {
            ksni::Status::Active
        }
    }

    fn tool_tip(&self) -> ToolTip {
        let state = self.state.lock().unwrap();
        let title = if !state.daemon_running {
            "Deskbrid — daemon not running".into()
        } else if state.update_available {
            format!(
                "Deskbrid — Update available: v{} → v{}",
                state.current_version, state.latest_version
            )
        } else if state.checked {
            format!("Deskbrid v{} — up to date", state.current_version)
        } else {
            "Deskbrid — checking for updates...".into()
        };

        ToolTip {
            title,
            description: "Click for menu".into(),
            ..Default::default()
        }
    }

    // Left-click opens the menu
    const MENU_ON_ACTIVATE: bool = true;

    fn menu(&self) -> Vec<ksni::MenuItem<Self>> {
        use ksni::menu::*;
        let state = self.state.lock().unwrap();
        let mut items: Vec<ksni::MenuItem<Self>> = Vec::new();

        // ─── Daemon control ────────────────────────────
        if state.daemon_running {
            items.push(
                StandardItem {
                    label: "Stop Daemon".into(),
                    icon_name: "process-stop".into(),
                    activate: Box::new(|_: &mut Self| {
                        tokio::spawn(async {
                            let result = tokio::process::Command::new("systemctl")
                                .args(["--user", "stop", "deskbrid.service"])
                                .stdout(std::process::Stdio::null())
                                .stderr(std::process::Stdio::null())
                                .status()
                                .await;
                            match result {
                                Ok(s) if s.success() => info!("Daemon stopped"),
                                _ => {
                                    let _ = tokio::process::Command::new("pkill")
                                        .arg("deskbrid")
                                        .stdout(std::process::Stdio::null())
                                        .stderr(std::process::Stdio::null())
                                        .status()
                                        .await;
                                }
                            }
                        });
                    }),
                    ..Default::default()
                }
                .into(),
            );
        } else {
            items.push(
                StandardItem {
                    label: "Start Daemon".into(),
                    icon_name: "media-playback-start".into(),
                    activate: Box::new(|_: &mut Self| {
                        tokio::spawn(async {
                            let _ = tokio::process::Command::new("systemctl")
                                .args(["--user", "start", "deskbrid.service"])
                                .stdout(std::process::Stdio::null())
                                .stderr(std::process::Stdio::null())
                                .status()
                                .await;
                        });
                    }),
                    ..Default::default()
                }
                .into(),
            );
        }

        items.push(MenuItem::Separator);

        // ─── Version info ──────────────────────────────
        if state.update_available {
            items.push(
                StandardItem {
                    label: format!(
                        "Update: v{} → v{}",
                        state.current_version, state.latest_version
                    ),
                    enabled: false,
                    ..Default::default()
                }
                .into(),
            );
            items.push(
                StandardItem {
                    label: "Update Now".into(),
                    icon_name: "system-software-update".into(),
                    activate: Box::new(|_: &mut Self| {
                        tokio::spawn(async {
                            match tokio::process::Command::new("deskbrid")
                                .arg("update")
                                .status()
                                .await
                            {
                                Ok(status) if status.success() => {
                                    info!("Self-update completed");
                                }
                                Ok(status) => {
                                    warn!("Self-update exited: {}", status);
                                }
                                Err(e) => {
                                    error!("Self-update failed: {e}");
                                }
                            }
                        });
                    }),
                    ..Default::default()
                }
                .into(),
            );
        } else if state.checked {
            items.push(
                StandardItem {
                    label: format!("v{} — up to date", state.current_version),
                    enabled: false,
                    ..Default::default()
                }
                .into(),
            );
        } else {
            items.push(
                StandardItem {
                    label: "Checking...".into(),
                    enabled: false,
                    ..Default::default()
                }
                .into(),
            );
        }

        items.push(MenuItem::Separator);

        // ─── Actions ────────────────────────────────────
        items.push(
            StandardItem {
                label: "Check Now".into(),
                icon_name: "view-refresh".into(),
                activate: Box::new(|this: &mut Self| {
                    let state = this.state.clone();
                    let handle_opt = {
                        let h = this.handle.lock().unwrap();
                        h.clone()
                    };
                    tokio::spawn(async move {
                        if let Err(e) = check_and_update(&state, handle_opt.as_ref()).await {
                            error!("Manual check failed: {e}");
                        }
                    });
                }),
                ..Default::default()
            }
            .into(),
        );

        items.push(
            StandardItem {
                label: "Open Dashboard".into(),
                icon_name: "applications-internet".into(),
                activate: Box::new(|_: &mut Self| {
                    tokio::spawn(async {
                        let _ = tokio::process::Command::new("xdg-open")
                            .arg("http://localhost:20129")
                            .status()
                            .await;
                    });
                }),
                ..Default::default()
            }
            .into(),
        );

        items.push(MenuItem::Separator);

        items.push(
            StandardItem {
                label: "Quit".into(),
                icon_name: "application-exit".into(),
                activate: Box::new(|this: &mut Self| {
                    this.shutdown.notify_one();
                }),
                ..Default::default()
            }
            .into(),
        );

        items
    }
}

/// Run the tray icon. Blocks until Quit is selected or Ctrl+C received.
pub async fn run() -> anyhow::Result<()> {
    let shutdown = Arc::new(tokio::sync::Notify::new());
    let tray = DeskbridTray::new(shutdown.clone());
    let state = tray.state.clone();
    let handle_ref = tray.handle.clone();

    let tray_handle = tray.spawn().await?;
    *handle_ref.lock().unwrap() = Some(tray_handle);

    info!("Deskbrid tray icon started");

    // Background loop
    let bg_state = state.clone();
    let bg_handle = handle_ref.clone();
    let bg_shutdown = shutdown.clone();
    tokio::spawn(async move {
        event_loop(bg_state, bg_handle, bg_shutdown).await;
    });

    // Wait for shutdown
    shutdown.notified().await;
    info!("Tray shutting down");

    Ok(())
}
