use super::helpers::*;
use crate::protocol::Action;
use serde_json::Value;

pub(super) fn parse_clipboard(raw: &Value, _id: &str, type_str: &str) -> anyhow::Result<Action> {
    Ok(match type_str {
        // Clipboard
        "clipboard.read" => Action::ClipboardRead,
        "clipboard.write" => Action::ClipboardWrite {
            text: raw["text"].as_str().unwrap_or("").into(),
        },
        "clipboard.history" => Action::ClipboardHistoryList {
            limit: raw["limit"].as_u64().map(|value| value as usize),
            query: optional_non_empty_string(raw, "query")?,
        },
        "clipboard.history.clear" => Action::ClipboardHistoryClear,
        _ => anyhow::bail!("unknown clipboard type: {type_str}"),
    })
}
