use super::*;
use crate::protocol;

pub(super) async fn audio_list_sinks(
    backend: &HyprBackend,
) -> anyhow::Result<Vec<protocol::AudioSinkInfo>> {
    let output = backend
        .sh("pactl", &["list", "sinks"])
        .await
        .unwrap_or_default();
    let mut sinks = Vec::new();
    let mut current_id = 0u32;
    let mut current_name = String::new();
    let mut current_desc = String::new();
    let mut current_volume = 1.0_f64;
    let mut current_muted = false;

    for line in output.lines() {
        let t = line.trim();
        if let Some(rest) = t.strip_prefix("Sink #") {
            if current_id > 0 {
                sinks.push(protocol::AudioSinkInfo {
                    id: current_id,
                    name: std::mem::take(&mut current_name),
                    description: std::mem::take(&mut current_desc),
                    volume: current_volume,
                    muted: current_muted,
                });
            }
            current_id = rest.parse().unwrap_or(0);
            current_name.clear();
            current_desc.clear();
            current_volume = 1.0;
            current_muted = false;
        } else if let Some(v) = t.strip_prefix("Description: ") {
            current_desc = v.to_string();
            if current_name.is_empty() {
                current_name = v.to_string();
            }
        } else if let Some(v) = t.strip_prefix("Name: ") {
            current_name = v.to_string();
        } else if let Some(v) = t.strip_prefix("Volume: ") {
            // Format: "front-left: 62271 /  95% / -1.33 dB,   front-right: ..."
            current_volume = v
                .split('%')
                .next()
                .and_then(|s| s.rsplit('/').next())
                .and_then(|s| s.trim().parse::<u32>().ok())
                .map(|pct| pct as f64 / 100.0)
                .unwrap_or(1.0);
        } else if let Some(v) = t.strip_prefix("Mute: ") {
            current_muted = v.trim() == "yes";
        }
    }
    if current_id > 0 {
        sinks.push(protocol::AudioSinkInfo {
            id: current_id,
            name: current_name,
            description: current_desc,
            volume: current_volume,
            muted: current_muted,
        });
    }
    Ok(sinks)
}

pub(super) async fn audio_set_sink_volume(
    backend: &HyprBackend,
    sink_id: u32,
    volume: f64,
) -> anyhow::Result<()> {
    let vol_pct = (volume * 100.0) as u32;
    backend
        .sh(
            "pactl",
            &[
                "set-sink-volume",
                &sink_id.to_string(),
                &format!("{}%", vol_pct),
            ],
        )
        .await?;
    Ok(())
}
