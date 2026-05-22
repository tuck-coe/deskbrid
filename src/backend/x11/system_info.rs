use super::*;
use crate::protocol;

pub(super) async fn system_info(backend: &X11Backend) -> anyhow::Result<protocol::SystemInfo> {
    Ok(protocol::SystemInfo {
        desktop: backend.detected_de.clone(),
        desktop_version: "unknown".into(),
        compositor: "x11".into(),
        session_type: "x11".into(),
        monitors: backend.xrandr_monitors().await.unwrap_or_else(|_| {
            vec![protocol::MonitorInfo {
                id: 0,
                name: "X11".into(),
                width: 1920,
                height: 1080,
                scale: 1.0,
                primary: true,
                enabled: true,
                x: 0,
                y: 0,
                refresh_rate: None,
                rotation: "normal".into(),
            }]
        }),
        workspace_count: 1,
        current_workspace: 0,
        idle_seconds: 0,
    })
}

pub(super) async fn idle_seconds(_backend: &X11Backend) -> anyhow::Result<u64> {
    Ok(0)
}

pub(super) async fn power_action(_backend: &X11Backend, _action: &str) -> anyhow::Result<()> {
    anyhow::bail!("not implemented on x11 backend")
}

pub(super) async fn battery_status(
    _backend: &X11Backend,
) -> anyhow::Result<Vec<protocol::BatteryInfo>> {
    Ok(Vec::new())
}
