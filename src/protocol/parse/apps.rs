use super::helpers::*;
use crate::protocol::Action;
use serde_json::Value;

pub(super) fn parse_apps(raw: &Value, _id: &str, type_str: &str) -> anyhow::Result<Action> {
    Ok(match type_str {
        // Apps
        "apps.list" => Action::AppList {
            categories: optional_string_array(raw, "categories")?,
            mime_types: optional_string_array(raw, "mime_types")?,
            include_hidden: raw["include_hidden"].as_bool().unwrap_or(false),
            limit: raw["limit"].as_u64().map(|value| value as usize),
        },
        "apps.search" => Action::AppSearch {
            query: required_non_empty_string(raw, "query")?,
            limit: raw["limit"].as_u64().map(|value| value as usize),
        },
        "apps.get" => Action::AppGet {
            app_id: required_non_empty_string(raw, "app_id")?,
        },
        _ => anyhow::bail!("unknown apps type: {type_str}"),
    })
}
