use crate::protocol::Action;
use serde_json::Value;

pub(super) fn parse_macro(raw: &Value, _id: &str, type_str: &str) -> anyhow::Result<Action> {
    match type_str {
        "macro.record.start" => {
            let name = raw["name"]
                .as_str()
                .ok_or_else(|| anyhow::anyhow!("macro.record.start requires 'name'"))?
                .to_string();
            if name.trim().is_empty() {
                anyhow::bail!("macro.record.start 'name' must not be empty");
            }
            Ok(Action::MacroRecordStart {
                name,
                description: raw["description"].as_str().map(String::from),
            })
        }
        "macro.record.stop" => Ok(Action::MacroRecordStop),
        "macro.replay" => {
            let name = raw["name"]
                .as_str()
                .ok_or_else(|| anyhow::anyhow!("macro.replay requires 'name'"))?
                .to_string();
            if name.trim().is_empty() {
                anyhow::bail!("macro.replay 'name' must not be empty");
            }
            let mode = raw["mode"].as_str().map(String::from);
            if let Some(ref m) = mode
                && !["fast", "timed", "stepped"].contains(&m.as_str())
            {
                anyhow::bail!(
                    "macro.replay 'mode' must be 'fast', 'timed', or 'stepped', got '{}'",
                    m
                );
            }
            Ok(Action::MacroReplay {
                name,
                mode,
                loop_count: raw["loop_count"].as_u64().map(|n| n as u32),
                stop_on_error: raw["stop_on_error"].as_bool(),
            })
        }
        "macro.list" => Ok(Action::MacroList),
        "macro.get" => {
            let name = raw["name"]
                .as_str()
                .ok_or_else(|| anyhow::anyhow!("macro.get requires 'name'"))?
                .to_string();
            Ok(Action::MacroGet { name })
        }
        "macro.delete" => {
            let name = raw["name"]
                .as_str()
                .ok_or_else(|| anyhow::anyhow!("macro.delete requires 'name'"))?
                .to_string();
            Ok(Action::MacroDelete { name })
        }
        "macro.export" => {
            let name = raw["name"]
                .as_str()
                .ok_or_else(|| anyhow::anyhow!("macro.export requires 'name'"))?
                .to_string();
            Ok(Action::MacroExport { name })
        }
        "macro.import" => {
            let name = raw["name"]
                .as_str()
                .ok_or_else(|| anyhow::anyhow!("macro.import requires 'name'"))?
                .to_string();
            let data = raw["data"]
                .as_str()
                .ok_or_else(|| anyhow::anyhow!("macro.import requires 'data'"))?
                .to_string();
            Ok(Action::MacroImport { name, data })
        }
        _ => anyhow::bail!("unknown macro action: {}", type_str),
    }
}
