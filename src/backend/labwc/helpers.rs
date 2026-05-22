use crate::protocol;
use serde_json::Value;

/// Parse `labwc-helper list-windows` JSON output (when labwc-helper is available).
pub(super) fn parse_labwc_windows_json(raw: &Value) -> Vec<protocol::WindowInfo> {
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

/// Parse `wlrctl toplevel list` output (fallback when labwc-helper is missing).
///
/// Output format (one window per line):
///   0: Firefox (firefox)
///   1: Alacritty (Alacritty)
pub(super) fn parse_wlrctl_windows(
    raw: &str,
    focused_id: Option<&str>,
) -> Vec<protocol::WindowInfo> {
    raw.lines()
        .filter_map(|line| {
            let (id_part, rest) = line.split_once(':')?;
            let id = id_part.trim().to_string();
            let rest = rest.trim();
            let (title, app_id) = if let Some((t, a)) = rest.rsplit_once(" (") {
                (t.to_string(), a.strip_suffix(')').unwrap_or(a).to_string())
            } else {
                (rest.to_string(), String::new())
            };
            let is_focused = focused_id.is_some_and(|fid| fid == id);
            Some(protocol::WindowInfo {
                id,
                title,
                app_id: app_id.to_ascii_lowercase(),
                workspace_id: 0,
                is_focused,
                is_minimized: false,
                geometry: None,
                pid: None,
            })
        })
        .collect()
}
