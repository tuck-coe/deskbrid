use crate::protocol::Action;
use serde_json::Value;

pub(super) fn parse_hotkeys(raw: &Value, _id: &str, type_str: &str) -> anyhow::Result<Action> {
    Ok(match type_str {
        // Hotkeys
        "hotkeys.register" => Action::HotkeysRegister {
            hotkey_id: raw["hotkey_id"].as_str().unwrap_or("").into(),
            keys: raw["keys"]
                .as_array()
                .map(|a| {
                    a.iter()
                        .filter_map(|v| v.as_str().map(String::from))
                        .collect()
                })
                .unwrap_or_default(),
        },
        "hotkeys.unregister" => Action::HotkeysUnregister {
            hotkey_id: raw["hotkey_id"].as_str().unwrap_or("").into(),
        },
        _ => anyhow::bail!("unknown hotkeys type: {type_str}"),
    })
}
