//! DBus hub — watches GNOME Shell, Notifications, IdleMonitor, and more.

use crate::events::EventBus;
use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use tokio::sync::watch;
use tokio::time::{self, Duration};
use tracing::{debug, warn};
use zbus::zvariant::OwnedValue;

const SHELL_DEST: &str = "org.gnome.Shell";
const SHELL_PATH: &str = "/org/gnome/Shell";
const SHELL_IFACE: &str = "org.gnome.Shell";
const DISPLAY_CONFIG_DEST: &str = "org.gnome.Mutter.DisplayConfig";
const DISPLAY_CONFIG_PATH: &str = "/org/gnome/Mutter/DisplayConfig";
const DISPLAY_CONFIG_IFACE: &str = "org.gnome.Mutter.DisplayConfig";

#[derive(Clone)]
pub struct Hub {
    conn: zbus::Connection,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Hash)]
pub struct WindowInfo {
    pub title: String,
    pub app_id: String,
    pub pid: i64,
    #[serde(default)]
    pub workspace: i64,
    #[serde(default)]
    pub focused: bool,
    #[serde(default)]
    pub geometry: [i64; 4],
    #[serde(default)]
    pub wm_class: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct MonitorInfo {
    pub id: u32,
    pub width: i32,
    pub height: i32,
    pub scale: f64,
    pub refresh: u32,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
struct FocusWindow {
    title: String,
    app_id: String,
    pid: i64,
    workspace: i64,
    geometry: [i64; 4],
    wm_class: String,
}

type Mode = (
    String,
    i32,
    i32,
    f64,
    f64,
    Vec<f64>,
    HashMap<String, OwnedValue>,
);
type PhysicalMonitor = (
    (String, String, String, String),
    Vec<Mode>,
    HashMap<String, OwnedValue>,
);
type LogicalMonitor = (
    i32,
    i32,
    f64,
    u32,
    bool,
    Vec<(String, String, String, String)>,
    HashMap<String, OwnedValue>,
);

impl Hub {
    pub async fn new(_event_bus: EventBus) -> Result<Self> {
        let conn = zbus::Connection::session()
            .await
            .context("connecting to session bus")?;
        Ok(Self { conn })
    }

    pub async fn list_windows(&self) -> Result<Vec<WindowInfo>> {
        let script = r#"
            JSON.stringify(
              global.get_window_actors().map(w => {
                const m = w.meta_window;
                const rect = m.get_frame_rect();
                return {
                  title: m.get_title() || "",
                  app_id: m.get_wm_class() || "",
                  pid: m.get_pid() || 0,
                  workspace: m.get_workspace() ? m.get_workspace().index() : 0,
                  focused: global.display.focus_window === m,
                  geometry: [rect.x, rect.y, rect.width, rect.height],
                  wm_class: m.get_wm_class() || ""
                };
              })
            )
        "#;
        self.eval_json(script).await
    }

    pub async fn list_monitors(&self) -> Result<Vec<MonitorInfo>> {
        let proxy = zbus::Proxy::new(
            &self.conn,
            DISPLAY_CONFIG_DEST,
            DISPLAY_CONFIG_PATH,
            DISPLAY_CONFIG_IFACE,
        )
        .await
        .context("creating mutter display config proxy")?;

        let (_serial, monitors, logical_monitors, _properties): (
            u32,
            Vec<PhysicalMonitor>,
            Vec<LogicalMonitor>,
            HashMap<String, OwnedValue>,
        ) = proxy
            .call("GetCurrentState", &())
            .await
            .context("calling org.gnome.Mutter.DisplayConfig.GetCurrentState")?;

        let mut scales_by_connector = HashMap::new();
        for (_, _, scale, _, _, connector_refs, _) in &logical_monitors {
            for (connector, _, _, _) in connector_refs {
                scales_by_connector.insert(connector.clone(), *scale);
            }
        }

        let mut result = Vec::with_capacity(monitors.len());
        for (index, ((connector, _, _, _), modes, _)) in monitors.iter().enumerate() {
            let current_mode = modes
                .iter()
                .find(|(_, _, _, _, _, _, properties)| property_bool(properties, "is-current"))
                .or_else(|| modes.first())
                .ok_or_else(|| anyhow!("monitor {connector} reported no modes"))?;

            result.push(MonitorInfo {
                id: index as u32,
                width: current_mode.1,
                height: current_mode.2,
                scale: scales_by_connector
                    .get(connector)
                    .copied()
                    .unwrap_or(current_mode.4),
                refresh: current_mode.3.round().max(0.0) as u32,
            });
        }

        Ok(result)
    }

    pub async fn focus_window(
        &self,
        app_id: Option<&str>,
        title: Option<&str>,
        exact: bool,
    ) -> Result<()> {
        if app_id.is_none() && title.is_none() {
            return Err(anyhow!("window:focus requires app_id or title"));
        }

        let app_id =
            serde_json::to_string(&app_id.unwrap_or_default()).context("encoding app_id filter")?;
        let title =
            serde_json::to_string(&title.unwrap_or_default()).context("encoding title filter")?;
        let script = format!(
            r#"
            (() => {{
              const appId = {app_id};
              const title = {title};
              const exact = {exact};
              const windows = global.get_window_actors().map(w => w.meta_window);
              const matches = value => exact ? value === title || value === appId : value.toLowerCase().includes(title.toLowerCase()) || value.toLowerCase().includes(appId.toLowerCase());
              const found = windows.find(w => {{
                const wmClass = w.get_wm_class() || "";
                const windowTitle = w.get_title() || "";
                if (appId && matches(wmClass)) return true;
                if (title && matches(windowTitle)) return true;
                return false;
              }});
              if (!found) return JSON.stringify({{ ok: false }});
              found.activate(global.get_current_time());
              return JSON.stringify({{ ok: true }});
            }})()
            "#
        );

        let result: serde_json::Value = self.eval_json(&script).await?;
        if result
            .get("ok")
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(false)
        {
            Ok(())
        } else {
            Err(anyhow!("no matching window found"))
        }
    }

    pub async fn send_notification(&self, summary: &str, body: &str, urgency: &str) -> Result<u32> {
        let proxy = zbus::Proxy::new(
            &self.conn,
            "org.freedesktop.Notifications",
            "/org/freedesktop/Notifications",
            "org.freedesktop.Notifications",
        )
        .await
        .context("creating notifications proxy")?;

        let urgency_byte = match urgency {
            "low" => 0_u8,
            "critical" => 2_u8,
            _ => 1_u8,
        };
        let hints = std::collections::HashMap::<&str, zbus::zvariant::Value<'_>>::from([(
            "urgency",
            zbus::zvariant::Value::from(urgency_byte),
        )]);
        let actions: Vec<&str> = Vec::new();

        proxy
            .call(
                "Notify",
                &("deskbrid", 0_u32, "", summary, body, actions, hints, -1_i32),
            )
            .await
            .context("sending notification")
    }

    pub async fn watch_windows(
        self,
        event_bus: EventBus,
        mut shutdown: watch::Receiver<bool>,
    ) -> Result<()> {
        let mut ticker = time::interval(Duration::from_millis(500));
        let mut last_focus: Option<FocusWindow> = None;
        let mut previous_windows: HashSet<WindowInfo> = HashSet::new();

        loop {
            tokio::select! {
                _ = shutdown.changed() => break,
                _ = ticker.tick() => {
                    match self.poll_focus().await {
                        Ok(Some(focus)) if last_focus.as_ref() != Some(&focus) => {
                            event_bus.emit(
                                "window:focus",
                                serde_json::json!({
                                    "title": focus.title,
                                    "app_id": focus.app_id,
                                    "pid": focus.pid,
                                    "workspace": focus.workspace,
                                    "geometry": focus.geometry,
                                    "wm_class": focus.wm_class,
                                }),
                            );
                            last_focus = Some(focus);
                        }
                        Ok(_) => {}
                        Err(error) => warn!("window focus poll failed: {error:#}"),
                    }

                    match self.list_windows().await {
                        Ok(current) => {
                            let current_set: HashSet<_> = current.iter().cloned().collect();

                            for opened in current_set.difference(&previous_windows) {
                                event_bus.emit(
                                    "window:open",
                                    serde_json::json!({
                                        "title": opened.title,
                                        "app_id": opened.app_id,
                                        "pid": opened.pid,
                                        "workspace": opened.workspace,
                                        "geometry": opened.geometry,
                                    }),
                                );
                            }

                            for closed in previous_windows.difference(&current_set) {
                                event_bus.emit(
                                    "window:close",
                                    serde_json::json!({
                                        "app_id": closed.app_id,
                                        "pid": closed.pid,
                                    }),
                                );
                            }

                            previous_windows = current_set;
                        }
                        Err(error) => warn!("window list poll failed: {error:#}"),
                    }
                }
            }
        }

        Ok(())
    }

    pub async fn watch_notifications(
        self,
        event_bus: EventBus,
        mut shutdown: watch::Receiver<bool>,
    ) -> Result<()> {
        let mut child = Command::new("dbus-monitor")
            .args([
                "--session",
                "interface='org.freedesktop.Notifications',member='Notify'",
            ])
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::null())
            .spawn()
            .context("spawning dbus-monitor for notifications")?;

        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| anyhow!("dbus-monitor stdout unavailable"))?;
        let mut lines = BufReader::new(stdout).lines();
        let mut block: Vec<String> = Vec::new();

        loop {
            tokio::select! {
                _ = shutdown.changed() => {
                    if let Err(error) = child.start_kill() {
                        warn!("failed to stop notification monitor: {error:#}");
                    }
                    let _ = child.wait().await;
                    break;
                }
                line = lines.next_line() => {
                    match line.context("reading dbus-monitor output")? {
                        Some(line) => {
                            if line.trim().is_empty() {
                                if let Some(event) = parse_notification_block(&block) {
                                    event_bus.emit("notifications", event);
                                }
                                block.clear();
                                continue;
                            }
                            block.push(line);
                        }
                        None => {
                            warn!("notification monitor exited");
                            break;
                        }
                    }
                }
            }
        }

        Ok(())
    }

    pub async fn watch_idle(
        self,
        _event_bus: EventBus,
        mut shutdown: watch::Receiver<bool>,
    ) -> Result<()> {
        debug!("idle monitor not implemented in phase 1");
        let _ = shutdown.changed().await;
        Ok(())
    }

    async fn poll_focus(&self) -> Result<Option<FocusWindow>> {
        let script = r#"
            (() => {
              const m = global.display.focus_window;
              if (!m) return "null";
              const rect = m.get_frame_rect();
              return JSON.stringify({
                title: m.get_title() || "",
                app_id: m.get_wm_class() || "",
                pid: m.get_pid() || 0,
                workspace: m.get_workspace() ? m.get_workspace().index() : 0,
                geometry: [rect.x, rect.y, rect.width, rect.height],
                wm_class: m.get_wm_class() || ""
              });
            })()
        "#;
        self.eval_json(script).await
    }

    async fn eval_json<T>(&self, script: &str) -> Result<T>
    where
        T: for<'de> Deserialize<'de>,
    {
        let proxy = zbus::Proxy::new(&self.conn, SHELL_DEST, SHELL_PATH, SHELL_IFACE)
            .await
            .context("creating gnome shell proxy")?;
        let (success, result): (bool, String) = proxy
            .call("Eval", &(script))
            .await
            .context("calling org.gnome.Shell.Eval")?;

        if !success {
            return Err(anyhow!("shell eval returned unsuccessful result"));
        }

        serde_json::from_str(&result).with_context(|| format!("parsing shell eval json: {result}"))
    }
}

fn property_bool(properties: &HashMap<String, OwnedValue>, key: &str) -> bool {
    properties
        .get(key)
        .and_then(|value| bool::try_from(value.clone()).ok())
        .unwrap_or(false)
}

fn parse_notification_block(block: &[String]) -> Option<serde_json::Value> {
    if !block.iter().any(|line| line.contains("member=Notify")) {
        return None;
    }

    let strings: Vec<String> = block
        .iter()
        .filter_map(|line| {
            let trimmed = line.trim();
            trimmed
                .strip_prefix("string ")
                .map(|value| value.trim_matches('"').to_string())
        })
        .collect();
    let uints: Vec<u64> = block
        .iter()
        .filter_map(|line| line.trim().strip_prefix("uint32 ")?.parse::<u64>().ok())
        .collect();

    if strings.len() < 4 {
        return None;
    }

    Some(serde_json::json!({
        "app": strings.first().cloned().unwrap_or_default(),
        "app_icon": strings.get(1).cloned().unwrap_or_default(),
        "summary": strings.get(2).cloned().unwrap_or_default(),
        "body": strings.get(3).cloned().unwrap_or_default(),
        "urgency": "normal",
        "id": uints.first().copied().unwrap_or(0),
    }))
}
