use super::*;
use crate::protocol;

pub(super) async fn audio_list_sinks(
    _backend: &X11Backend,
) -> anyhow::Result<Vec<protocol::AudioSinkInfo>> {
    Ok(Vec::new())
}

pub(super) async fn audio_set_sink_volume(
    _backend: &X11Backend,
    _sink_id: u32,
    _volume: f64,
) -> anyhow::Result<()> {
    anyhow::bail!("not implemented on x11 backend")
}
