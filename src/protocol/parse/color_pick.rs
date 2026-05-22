use super::helpers::*;
use crate::protocol::Action;
use serde_json::Value;

pub(super) fn parse_color_pick(raw: &Value, _id: &str, type_str: &str) -> anyhow::Result<Action> {
    Ok(match type_str {
        // Color picker
        "color.pick" => Action::ColorPick {
            x: raw["x"]
                .as_u64()
                .ok_or_else(|| anyhow::anyhow!("missing or invalid 'x' field"))?
                as u32,
            y: raw["y"]
                .as_u64()
                .ok_or_else(|| anyhow::anyhow!("missing or invalid 'y' field"))?
                as u32,
            path: optional_non_empty_string(raw, "path")?,
        },
        _ => anyhow::bail!("unknown color_pick type: {type_str}"),
    })
}
