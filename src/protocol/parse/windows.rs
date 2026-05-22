use super::helpers::*;
use crate::protocol::Action;
use serde_json::Value;

pub(super) fn parse_windows(raw: &Value, _id: &str, type_str: &str) -> anyhow::Result<Action> {
    Ok(match type_str {
        // Windows
        "windows.list" => Action::WindowsList,
        "windows.focus" => Action::WindowsFocus(required_non_empty_string(raw, "window_id")?),
        "windows.get" => Action::WindowsGet(required_non_empty_string(raw, "window_id")?),
        "windows.close" => Action::WindowsClose(required_non_empty_string(raw, "window_id")?),
        "windows.minimize" => Action::WindowsMinimize(required_non_empty_string(raw, "window_id")?),
        "windows.maximize" => Action::WindowsMaximize(required_non_empty_string(raw, "window_id")?),
        "windows.move_resize" => {
            let x = raw["x"]
                .as_i64()
                .ok_or_else(|| anyhow::anyhow!("missing or invalid 'x' field"))?
                as i32;
            let y = raw["y"]
                .as_i64()
                .ok_or_else(|| anyhow::anyhow!("missing or invalid 'y' field"))?
                as i32;
            let width = raw["width"]
                .as_u64()
                .ok_or_else(|| anyhow::anyhow!("missing or invalid 'width' field"))?
                as u32;
            let height = raw["height"]
                .as_u64()
                .ok_or_else(|| anyhow::anyhow!("missing or invalid 'height' field"))?
                as u32;
            if width == 0 || height == 0 {
                anyhow::bail!("'width' and 'height' must be positive");
            }
            Action::WindowsMoveResize {
                window_id: required_non_empty_string(raw, "window_id")?,
                x,
                y,
                width,
                height,
            }
        }
        "windows.tile" => Action::WindowsTile {
            window_id: required_non_empty_string(raw, "window_id")?,
            preset: required_non_empty_string(raw, "preset")?,
            monitor: raw["monitor"].as_u64().map(|value| value as u32),
            padding: raw["padding"].as_u64().map(|value| value as u32),
        },
        "windows.activate_or_launch" => Action::WindowsActivateOrLaunch {
            app_id: required_non_empty_string(raw, "app_id")?,
            command: optional_string_array(raw, "command")?,
            workdir: raw["workdir"].as_str().map(String::from),
            env: raw["env"].as_object().map(|o| {
                o.iter()
                    .map(|(k, v)| (k.clone(), v.as_str().unwrap_or("").to_string()))
                    .collect()
            }),
        },
        _ => anyhow::bail!("unknown windows type: {type_str}"),
    })
}
