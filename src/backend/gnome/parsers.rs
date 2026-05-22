use crate::protocol;
use crate::protocol::Geometry;

// ─── Free helper functions ─────────────────────────────

pub(super) fn parse_extension_json_windows(raw: &str) -> anyhow::Result<Vec<protocol::WindowInfo>> {
    let inner = raw.trim().trim_start_matches('(').trim_end_matches(')');
    let json_str = inner
        .trim()
        .trim_start_matches('\'')
        .trim_end_matches(',')
        .trim()
        .trim_end_matches('\'');
    let parsed: Vec<serde_json::Value> = serde_json::from_str(json_str)?;
    Ok(parsed
        .into_iter()
        .map(|w| protocol::WindowInfo {
            id: w["id"]
                .as_u64()
                .map(|n| n.to_string())
                .unwrap_or_else(|| w["id"].as_str().unwrap_or("").to_string()),
            title: w["title"].as_str().unwrap_or("").to_string(),
            app_id: w["app_id"].as_str().unwrap_or("").to_string(),
            workspace_id: w["workspace_index"].as_u64().unwrap_or(0) as u32,
            is_focused: w["focused"].as_bool().unwrap_or(false),
            is_minimized: w["minimized"].as_bool().unwrap_or(false),
            geometry: w["geometry"].as_array().and_then(|arr| {
                Some(Geometry {
                    x: arr.first()?.as_i64()? as i32,
                    y: arr.get(1)?.as_i64()? as i32,
                    width: arr.get(2)?.as_u64()? as u32,
                    height: arr.get(3)?.as_u64()? as u32,
                })
            }),
            pid: w["pid"].as_u64().map(|p| p as u32),
        })
        .collect())
}

pub(super) fn parse_gnome_randr(out: &str, monitors: &mut Vec<protocol::MonitorInfo>) {
    let mut name = String::new();
    let mut w = 1920u32;
    let mut h = 1080u32;
    let mut scale = 1.0f64;
    let mut idx = 0u32;
    for line in out.lines() {
        if line.starts_with("  ") || line.trim().is_empty() {
            if line.contains("x")
                && line.contains('@')
                && let Some(res) = line.split_whitespace().next()
            {
                let d: Vec<&str> = res.split('x').collect();
                if d.len() == 2 {
                    w = d[0].parse().unwrap_or(1920);
                    h = d[1]
                        .split('@')
                        .next()
                        .unwrap_or("1080")
                        .parse()
                        .unwrap_or(1080);
                }
            }
            if line.to_lowercase().contains("scale") {
                scale = line
                    .split(':')
                    .nth(1)
                    .unwrap_or("1.0")
                    .trim()
                    .parse()
                    .unwrap_or(1.0);
            }
            continue;
        }
        if !name.is_empty() {
            monitors.push(protocol::MonitorInfo {
                id: idx,
                name: name.clone(),
                width: w,
                height: h,
                scale,
                primary: idx == 0,
                enabled: true,
                x: 0,
                y: 0,
                refresh_rate: None,
                rotation: "normal".into(),
            });
            idx += 1;
        }
        name = line.split_whitespace().next().unwrap_or("").to_string();
    }
    if !name.is_empty() {
        monitors.push(protocol::MonitorInfo {
            id: idx,
            name,
            width: w,
            height: h,
            scale,
            primary: idx == 0,
            enabled: true,
            x: 0,
            y: 0,
            refresh_rate: None,
            rotation: "normal".into(),
        });
    }
}

pub(super) fn parse_wlr_randr(out: &str, monitors: &mut Vec<protocol::MonitorInfo>) {
    let mut name = String::new();
    let mut w = 1920u32;
    let mut h = 1080u32;
    let mut scale = 1.0f64;
    let mut idx = 0u32;
    for line in out.lines() {
        if !line.starts_with(' ') && !line.is_empty() {
            if !name.is_empty() {
                monitors.push(protocol::MonitorInfo {
                    id: idx,
                    name: name.clone(),
                    width: w,
                    height: h,
                    scale,
                    primary: idx == 0,
                    enabled: true,
                    x: 0,
                    y: 0,
                    refresh_rate: None,
                    rotation: "normal".into(),
                });
                idx += 1;
            }
            name = line.split(' ').next().unwrap_or("").to_string();
        }
        if line.contains("current")
            && let Some(res) = line.split_whitespace().next()
        {
            let d: Vec<&str> = res.split('x').collect();
            if d.len() == 2 {
                w = d[0].parse().unwrap_or(1920);
                h = d[1]
                    .split('@')
                    .next()
                    .unwrap_or("1080")
                    .parse()
                    .unwrap_or(1080);
            }
        }
        if line.contains("Scale:") {
            scale = line
                .split("Scale:")
                .nth(1)
                .unwrap_or("1.0")
                .trim()
                .parse()
                .unwrap_or(1.0);
        }
    }
    if !name.is_empty() {
        monitors.push(protocol::MonitorInfo {
            id: idx,
            name,
            width: w,
            height: h,
            scale,
            primary: idx == 0,
            enabled: true,
            x: 0,
            y: 0,
            refresh_rate: None,
            rotation: "normal".into(),
        });
    }
}
