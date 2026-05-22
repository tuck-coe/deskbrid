use crate::protocol::Action;
use serde_json::Value;

pub(super) fn parse_audio(raw: &Value, _id: &str, type_str: &str) -> anyhow::Result<Action> {
    Ok(match type_str {
        // Audio
        "audio.list_sinks" => Action::AudioListSinks,
        "audio.set_sink_volume" => Action::AudioSetSinkVolume {
            sink_id: raw["sink_id"].as_u64().unwrap_or(0) as u32,
            volume: raw["volume"].as_f64().unwrap_or(1.0),
        },
        _ => anyhow::bail!("unknown audio type: {type_str}"),
    })
}
