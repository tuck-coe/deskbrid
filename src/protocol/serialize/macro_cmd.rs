use crate::protocol::Action;
use serde_json::json;

pub(super) fn serialize_macro(action: &Action, id: &str) -> serde_json::Value {
    match action {
        Action::MacroRecordStart { name, description } => {
            let mut envelope = json!({
                "type": "macro.record.start",
                "id": id,
                "name": name,
            });
            if let Some(desc) = description {
                envelope["description"] = json!(desc);
            }
            envelope
        }
        Action::MacroRecordStop => json!({"type": "macro.record.stop", "id": id}),
        Action::MacroReplay {
            name,
            mode,
            loop_count,
            stop_on_error,
        } => {
            let mut envelope = json!({
                "type": "macro.replay",
                "id": id,
                "name": name,
            });
            if let Some(m) = mode {
                envelope["mode"] = json!(m);
            }
            if let Some(lc) = loop_count {
                envelope["loop_count"] = json!(lc);
            }
            if let Some(soe) = stop_on_error {
                envelope["stop_on_error"] = json!(soe);
            }
            envelope
        }
        Action::MacroList => json!({"type": "macro.list", "id": id}),
        Action::MacroGet { name } => json!({"type": "macro.get", "id": id, "name": name}),
        Action::MacroDelete { name } => {
            json!({"type": "macro.delete", "id": id, "name": name})
        }
        Action::MacroExport { name } => {
            json!({"type": "macro.export", "id": id, "name": name})
        }
        Action::MacroImport { name, data } => {
            json!({"type": "macro.import", "id": id, "name": name, "data": data})
        }
        _ => json!({"type": "unknown", "id": id}),
    }
}
