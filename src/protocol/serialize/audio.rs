use super::Action;
use serde_json::json;

pub(super) fn serialize_audio(action: &Action, id: &str) -> serde_json::Value {
    match action {
        // Audio
        Action::AudioListSinks => json!({"type": "audio.list_sinks", "id": id}),
        Action::AudioSetSinkVolume { sink_id, volume } => {
            json!({"type": "audio.set_sink_volume", "id": id, "sink_id": sink_id, "volume": volume})
        }

        // Monitor
        Action::MonitorList => json!({"type": "monitor.list", "id": id}),
        Action::MonitorSetPrimary { output } => {
            json!({"type": "monitor.set_primary", "id": id, "output": output})
        }
        Action::MonitorSetResolution {
            output,
            width,
            height,
            refresh_rate,
        } => {
            let mut obj = json!({"type": "monitor.set_resolution", "id": id, "output": output, "width": width, "height": height});
            if let Some(refresh) = refresh_rate {
                obj["refresh_rate"] = json!(refresh);
            }
            obj
        }
        Action::MonitorSetScale { output, scale } => {
            json!({"type": "monitor.set_scale", "id": id, "output": output, "scale": scale})
        }
        Action::MonitorSetRotation { output, rotation } => {
            json!({"type": "monitor.set_rotation", "id": id, "output": output, "rotation": rotation})
        }
        Action::MonitorEnable { output } => {
            json!({"type": "monitor.enable", "id": id, "output": output})
        }
        Action::MonitorDisable { output } => {
            json!({"type": "monitor.disable", "id": id, "output": output})
        }
        _ => unreachable!("not a audio action"),
    }
}
