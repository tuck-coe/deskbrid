use super::helpers::*;
use crate::protocol::Action;
use serde_json::Value;

pub(super) fn parse_a11y(raw: &Value, _id: &str, type_str: &str) -> anyhow::Result<Action> {
    Ok(match type_str {
        // Accessibility
        "a11y.tree" => Action::A11yTree {
            depth: raw["depth"].as_u64().map(|v| v as u32),
        },
        "a11y.get_element" => Action::A11yGetElement {
            role: raw["role"].as_str().map(String::from),
            name: raw["name"].as_str().map(String::from),
            index: raw["index"].as_u64().map(|v| v as u32),
        },
        "a11y.click_element" => Action::A11yClickElement {
            role: raw["role"].as_str().map(String::from),
            name: raw["name"].as_str().map(String::from),
            index: raw["index"].as_u64().map(|v| v as u32),
        },
        "a11y.get_text" => Action::A11yGetText {
            role: raw["role"].as_str().map(String::from),
            name: raw["name"].as_str().map(String::from),
            index: raw["index"].as_u64().map(|v| v as u32),
        },
        "a11y.snapshot_tree" => Action::A11ySnapshotTree {
            app_name: raw["app_name"].as_str().map(String::from),
            pid: raw["pid"].as_u64().map(|v| v as u32),
            max_nodes: raw["max_nodes"].as_u64().map(|v| v as usize),
            max_depth: raw["max_depth"].as_u64().map(|v| v as u32),
        },
        "a11y.perform_action" => Action::A11yPerformAction {
            object_ref: required_non_empty_string(raw, "object_ref")?,
            action_name: raw["action_name"].as_str().map(String::from),
        },
        "a11y.set_value" => Action::A11ySetValue {
            object_ref: required_non_empty_string(raw, "object_ref")?,
            value: raw["value"].as_str().unwrap_or("").to_string(),
        },
        "a11y.get_element_text" => Action::A11yGetElementText {
            object_ref: required_non_empty_string(raw, "object_ref")?,
            max_chars: raw["max_chars"].as_i64().map(|v| v as i32),
        },
        "a11y.list_apps" => Action::A11yListApps {
            limit: raw["limit"].as_u64().map(|v| v as usize),
        },
        "a11y.doctor" => Action::A11yDoctor,
        "a11y.setup_accessibility" => Action::A11ySetupAccessibility,
        "a11y.click_element_by_ref" => Action::A11yClickElementByRef {
            object_ref: required_non_empty_string(raw, "object_ref")?,
        },
        _ => anyhow::bail!("unknown a11y type: {type_str}"),
    })
}
