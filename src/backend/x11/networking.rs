use super::*;
use crate::protocol;

pub(super) async fn network_status(
    _backend: &X11Backend,
) -> anyhow::Result<protocol::NetworkStatusInfo> {
    Ok(protocol::NetworkStatusInfo {
        online: false,
        net_type: "unknown".into(),
    })
}

pub(super) async fn network_interfaces(
    _backend: &X11Backend,
) -> anyhow::Result<Vec<protocol::NetworkInterfaceInfo>> {
    Ok(Vec::new())
}

pub(super) async fn wifi_scan(
    _backend: &X11Backend,
) -> anyhow::Result<Vec<protocol::WifiNetworkInfo>> {
    Ok(Vec::new())
}

pub(super) async fn wifi_connect(
    _backend: &X11Backend,
    _ssid: &str,
    _password: Option<&str>,
) -> anyhow::Result<()> {
    anyhow::bail!("not implemented on x11 backend")
}
