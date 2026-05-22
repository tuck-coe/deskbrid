use crate::protocol;
use serde_json::Value;

pub(super) fn parse_labwc_windows(raw: &Value) -> Vec<protocol::WindowInfo> {
    raw.as_array()
        .map(|arr| {
            arr.iter()
                .filter_map(|w| {
                    let id = w["window_id"].as_u64().map(|n| n.to_string())?;
                    let title = w["title"].as_str().unwrap_or("").to_string();
                    let app_id = w["app_id"].as_str().unwrap_or("").to_string();
                    let focused = w["focused"].as_bool().unwrap_or(false);
                    let minimized = w["minimized"].as_bool().unwrap_or(false);
                    Some(protocol::WindowInfo {
                        is_focused: focused,
                        id,
                        title,
                        app_id: app_id.to_ascii_lowercase(),
                        workspace_id: 0,
                        is_minimized: minimized,
                        geometry: None,
                        pid: None,
                    })
                })
                .collect()
        })
        .unwrap_or_default()
}
