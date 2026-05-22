use crate::protocol::Action;
use serde_json::Value;

pub(super) fn parse_browser(raw: &Value, _id: &str, type_str: &str) -> anyhow::Result<Action> {
    Ok(match type_str {
        // Browser
        "browser.list_tabs" => Action::BrowserListTabs,
        "browser.navigate" => Action::BrowserNavigate {
            tab_index: raw["tab_index"].as_u64().map(|v| v as u32),
            url: raw["url"].as_str().unwrap_or("").into(),
        },
        "browser.evaluate" => Action::BrowserEvaluate {
            tab_index: raw["tab_index"].as_u64().map(|v| v as u32),
            expression: raw["expression"].as_str().unwrap_or("").into(),
            await_promise: raw["await_promise"].as_bool().unwrap_or(true),
        },
        "browser.screenshot_tab" => Action::BrowserScreenshotTab {
            tab_index: raw["tab_index"].as_u64().map(|v| v as u32),
        },
        "browser.click" => Action::BrowserClick {
            tab_index: raw["tab_index"].as_u64().map(|v| v as u32),
            selector: raw["selector"].as_str().unwrap_or("").into(),
        },
        _ => anyhow::bail!("unknown browser type: {type_str}"),
    })
}
