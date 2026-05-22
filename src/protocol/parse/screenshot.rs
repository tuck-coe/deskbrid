use super::helpers::*;
use crate::protocol::Action;
use serde_json::Value;
use crate::protocol::types::Region;

pub(super) fn parse_screenshot(raw: &Value, _id: &str, type_str: &str) -> anyhow::Result<Action> {
    Ok(match type_str {
        // Screenshot
        "screenshot" => Action::Screenshot {
            monitor: raw["monitor"].as_u64().map(|v| v as u32),
            region: raw.get("region").and_then(|r| {
                Some(Region {
                    x: r["x"].as_u64()? as u32,
                    y: r["y"].as_u64()? as u32,
                    width: r["width"].as_u64()? as u32,
                    height: r["height"].as_u64()? as u32,
                })
            }),
            window_id: raw["window_id"].as_str().map(String::from),
        },
        "screenshot.ocr" => Action::ScreenshotOcr {
            path: optional_non_empty_string(raw, "path")?,
            language: optional_non_empty_string(raw, "language")?,
            psm: optional_u32(raw, "psm")?,
            bounding_boxes: raw["bounding_boxes"].as_bool().unwrap_or(false),
            monitor: raw["monitor"].as_u64().map(|v| v as u32),
            region: raw.get("region").and_then(|r| {
                Some(Region {
                    x: r["x"].as_u64()? as u32,
                    y: r["y"].as_u64()? as u32,
                    width: r["width"].as_u64()? as u32,
                    height: r["height"].as_u64()? as u32,
                })
            }),
            window_id: raw["window_id"].as_str().map(String::from),
        },
        "screenshot.diff" => Action::ScreenshotDiff {
            before_path: required_non_empty_string_alias(raw, "before_path", "before")?,
            after_path: optional_non_empty_string_alias(raw, "after_path", "after")?,
            tolerance: optional_u8(raw, "tolerance")?,
            diff_path: optional_non_empty_string(raw, "diff_path")?,
            save_diff: raw["save_diff"].as_bool().unwrap_or(false),
            monitor: raw["monitor"].as_u64().map(|v| v as u32),
            region: raw.get("region").and_then(|r| {
                Some(Region {
                    x: r["x"].as_u64()? as u32,
                    y: r["y"].as_u64()? as u32,
                    width: r["width"].as_u64()? as u32,
                    height: r["height"].as_u64()? as u32,
                })
            }),
            window_id: raw["window_id"].as_str().map(String::from),
        },
        _ => anyhow::bail!("unknown screenshot type: {type_str}"),
    })
}
