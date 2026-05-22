use super::*;
use crate::protocol;
use crate::protocol::DeskbridEvent;
use std::collections::HashMap;
use std::process::Stdio;
use std::sync::{Arc, Mutex};
use tokio::process::Command;
use tokio::sync::broadcast;

impl GnomeBackend {
    pub async fn new(event_tx: broadcast::Sender<DeskbridEvent>) -> anyhow::Result<Self> {
        let conn = zbus::Connection::session().await?;
        let mut backend = Self {
            conn,
            event_tx,
            watchers: Arc::new(Mutex::new(HashMap::new())),
            rd_session_path: String::new(),
            sc_stream_path: String::new(),
            last_mouse: std::sync::Mutex::new((960.0, 540.0)),
        };
        backend.init_remote_desktop().await?;
        if let Err(e) = backend.init_screen_cast().await {
            tracing::warn!(
                "ScreenCast unavailable (absolute mouse positioning disabled): {}",
                e
            );
        }
        Ok(backend)
    }

    pub(super) async fn sh(&self, cmd: &str, args: &[&str]) -> anyhow::Result<String> {
        let output = Command::new(cmd)
            .args(args)
            .stdin(Stdio::null())
            .stderr(Stdio::piped())
            .output()
            .await?;
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("{} failed: {}", cmd, stderr.trim());
        }
        Ok(String::from_utf8(output.stdout)?.trim().to_string())
    }

    pub(super) async fn sh_ok(&self, cmd: &str, args: &[&str]) -> bool {
        Command::new(cmd)
            .args(args)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .await
            .map(|s| s.success())
            .unwrap_or(false)
    }

    pub(super) async fn sh_owned(&self, cmd: &str, args: Vec<String>) -> anyhow::Result<String> {
        let refs: Vec<&str> = args.iter().map(String::as_str).collect();
        self.sh(cmd, &refs).await
    }

    const EXT_BUS: &'static str = "org.deskbrid.WindowManager";
    const EXT_PATH: &'static str = "/org/deskbrid/WindowManager";
    const EXT_IFACE: &'static str = "org.deskbrid.WindowManager";

    pub(super) async fn ext_call_parsed(
        &self,
        method: &str,
        extra_args: &[&str],
    ) -> anyhow::Result<String> {
        let method_full = format!("{}.{}", Self::EXT_IFACE, method);
        let mut args = vec![
            "call",
            "--session",
            "--dest",
            Self::EXT_BUS,
            "--object-path",
            Self::EXT_PATH,
            "--method",
            &method_full,
        ];
        args.extend(extra_args);
        self.sh("gdbus", &args).await
    }

    pub(super) async fn ext_call_bool(
        &self,
        method: &str,
        extra_args: &[&str],
    ) -> anyhow::Result<()> {
        let raw = self.ext_call_parsed(method, extra_args).await?;
        if raw.contains("true") {
            Ok(())
        } else {
            anyhow::bail!("GNOME extension method {} returned false", method)
        }
    }

    pub(super) async fn resolve_window(&self, id: &str) -> anyhow::Result<protocol::WindowInfo> {
        if id.trim().is_empty() {
            anyhow::bail!("window id must not be empty");
        }
        let raw = self.ext_call_parsed("ListWindows", &[]).await?;
        let windows = super::parsers::parse_extension_json_windows(&raw)?;
        let id_l = id.to_lowercase();
        windows
            .iter()
            .find(|w| w.id.eq_ignore_ascii_case(id))
            .cloned()
            .or_else(|| {
                windows
                    .iter()
                    .find(|w| w.app_id.eq_ignore_ascii_case(id))
                    .cloned()
            })
            .or_else(|| {
                windows
                    .iter()
                    .find(|w| w.title.eq_ignore_ascii_case(id))
                    .cloned()
            })
            .or_else(|| {
                windows
                    .iter()
                    .find(|w| {
                        w.app_id.to_lowercase().contains(&id_l)
                            || w.title.to_lowercase().contains(&id_l)
                    })
                    .cloned()
            })
            .ok_or_else(|| anyhow::anyhow!("window not found: {}", id))
    }

    async fn init_remote_desktop(&mut self) -> anyhow::Result<()> {
        let reply = self
            .conn
            .call_method(
                Some("org.gnome.Mutter.RemoteDesktop"),
                "/org/gnome/Mutter/RemoteDesktop",
                Some("org.gnome.Mutter.RemoteDesktop"),
                "CreateSession",
                &(),
            )
            .await?;
        let path: zbus::zvariant::OwnedObjectPath = reply.body().deserialize()?;
        self.conn
            .call_method(
                Some("org.gnome.Mutter.RemoteDesktop"),
                path.as_str(),
                Some("org.gnome.Mutter.RemoteDesktop.Session"),
                "Start",
                &(),
            )
            .await?;
        self.rd_session_path = path.to_string();
        tracing::info!("RemoteDesktop session started: {}", self.rd_session_path);
        Ok(())
    }

    async fn init_screen_cast(&mut self) -> anyhow::Result<()> {
        use std::collections::HashMap;
        let props: HashMap<&str, zbus::zvariant::Value<'_>> = HashMap::new();
        let reply = self
            .conn
            .call_method(
                Some("org.gnome.Mutter.ScreenCast"),
                "/org/gnome/Mutter/ScreenCast",
                Some("org.gnome.Mutter.ScreenCast"),
                "CreateSession",
                &(props,),
            )
            .await?;
        let session_path: zbus::zvariant::OwnedObjectPath = reply.body().deserialize()?;
        self.conn
            .call_method(
                Some("org.gnome.Mutter.ScreenCast"),
                session_path.as_str(),
                Some("org.gnome.Mutter.ScreenCast.Session"),
                "Start",
                &(),
            )
            .await?;

        let mut monitor_candidates = Vec::new();
        if let Ok(monitors) = self.get_monitors().await {
            if let Some(primary) = monitors
                .iter()
                .find(|m| m.primary)
                .or_else(|| monitors.first())
            {
                monitor_candidates.push(primary.name.clone());
            }
            for m in monitors {
                if !monitor_candidates.iter().any(|n| n == &m.name) {
                    monitor_candidates.push(m.name);
                }
            }
        }
        if !monitor_candidates.iter().any(|n| n == "DP-1") {
            monitor_candidates.push("DP-1".to_string());
        }

        let stream_props: HashMap<&str, zbus::zvariant::Value<'_>> = HashMap::new();
        let mut last_err: Option<anyhow::Error> = None;
        for connector in monitor_candidates {
            tracing::info!("Trying ScreenCast monitor: {}", connector);
            match self
                .conn
                .call_method(
                    Some("org.gnome.Mutter.ScreenCast"),
                    session_path.as_str(),
                    Some("org.gnome.Mutter.ScreenCast.Session"),
                    "RecordMonitor",
                    &(connector.as_str(), stream_props.clone()),
                )
                .await
            {
                Ok(reply) => {
                    let sp: zbus::zvariant::OwnedObjectPath = reply.body().deserialize()?;
                    self.sc_stream_path = sp.to_string();
                    tracing::info!("ScreenCast stream created: {}", self.sc_stream_path);
                    return Ok(());
                }
                Err(e) => {
                    tracing::warn!("RecordMonitor failed for {}: {}", connector, e);
                    last_err = Some(e.into());
                }
            }
        }
        Err(last_err.unwrap_or_else(|| anyhow::anyhow!("failed to record any monitor")))
    }

    pub(super) async fn rd_call<B>(&self, method: &str, body: &B) -> anyhow::Result<()>
    where
        B: serde::Serialize + zbus::zvariant::Type,
    {
        self.conn
            .call_method(
                Some("org.gnome.Mutter.RemoteDesktop"),
                self.rd_session_path.as_str(),
                Some("org.gnome.Mutter.RemoteDesktop.Session"),
                method,
                body,
            )
            .await?;
        Ok(())
    }

    pub(super) async fn rd_keysym(&self, keysym: u32, pressed: bool) -> anyhow::Result<()> {
        self.rd_call("NotifyKeyboardKeysym", &(keysym, pressed))
            .await
    }

    pub(super) async fn rd_button(&self, button: i32, pressed: bool) -> anyhow::Result<()> {
        self.rd_call("NotifyPointerButton", &(button, pressed))
            .await
    }
}
