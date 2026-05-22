use super::*;
use crate::protocol;

pub(super) async fn bluetooth_list(
    _backend: &X11Backend,
) -> anyhow::Result<Vec<protocol::BluetoothDeviceInfo>> {
    Ok(Vec::new())
}

pub(super) async fn bluetooth_scan(
    _backend: &X11Backend,
    _duration: Option<u32>,
) -> anyhow::Result<()> {
    anyhow::bail!("not implemented on x11 backend")
}

pub(super) async fn bluetooth_stop_scan(_backend: &X11Backend) -> anyhow::Result<()> {
    anyhow::bail!("not implemented on x11 backend")
}

pub(super) async fn bluetooth_connect(_backend: &X11Backend, _address: &str) -> anyhow::Result<()> {
    anyhow::bail!("not implemented on x11 backend")
}

pub(super) async fn bluetooth_disconnect(
    _backend: &X11Backend,
    _address: &str,
) -> anyhow::Result<()> {
    anyhow::bail!("not implemented on x11 backend")
}
