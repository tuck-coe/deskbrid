use crate::protocol::Action;
use serde_json::Value;

pub(super) fn parse_location(raw: &Value, _id: &str, type_str: &str) -> anyhow::Result<Action> {
    Ok(match type_str {
        // Location
        "location.get" => Action::LocationGet,
        "ui.tree.get" => Action::UiTreeGet,
        "ui.element.click" => Action::UiElementClick {
            selector: raw["selector"].as_str().unwrap_or("").into(),
        },
        "ui.element.set_text" => Action::UiElementSetText {
            selector: raw["selector"].as_str().unwrap_or("").into(),
            text: raw["text"].as_str().unwrap_or("").into(),
        },
        _ => anyhow::bail!("unknown location type: {type_str}"),
    })
}
