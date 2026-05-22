use crate::protocol;

pub(super) fn parse_wmctrl_windows(
    raw: &str,
    active_window_id: Option<&str>,
) -> Vec<protocol::WindowInfo> {
    raw.lines()
        .filter_map(|line| parse_wmctrl_line(line, active_window_id))
        .collect()
}

pub(super) fn parse_wmctrl_line(
    line: &str,
    active_window_id: Option<&str>,
) -> Option<protocol::WindowInfo> {
    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts.len() < 9 {
        return None;
    }

    let id = normalize_window_id(parts[0]);
    let workspace_id = parts[1]
        .parse::<i32>()
        .ok()
        .map(|d| if d < 0 { 0 } else { d as u32 })
        .unwrap_or(0);
    let pid = parts[2].parse::<u32>().ok();
    let x = parts[3].parse::<i32>().ok()?;
    let y = parts[4].parse::<i32>().ok()?;
    let width = parts[5].parse::<u32>().ok()?;
    let height = parts[6].parse::<u32>().ok()?;
    let app_id = normalize_wm_class(parts[7]);
    let title = parts.get(9..).map(|tail| tail.join(" ")).unwrap_or_default();

    Some(protocol::WindowInfo {
        is_focused: active_window_id
            .map(normalize_window_id)
            .is_some_and(|active| active == id),
        id,
        title,
        app_id,
        workspace_id,
        is_minimized: false,
        geometry: Some(protocol::Geometry {
            x,
            y,
            width,
            height,
        }),
        pid,
    })
}

pub(super) fn normalize_window_id(raw: &str) -> String {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return String::new();
    }
    if trimmed.starts_with("0x") || trimmed.starts_with("0X") {
        return trimmed.to_ascii_lowercase();
    }
    trimmed
        .parse::<u64>()
        .map(|id| format!("0x{:08x}", id))
        .unwrap_or_else(|_| trimmed.to_ascii_lowercase())
}

fn normalize_wm_class(raw: &str) -> String {
    let class = raw.trim();
    if class.is_empty() || class.eq_ignore_ascii_case("n/a") {
        return String::new();
    }
    class
        .rsplit('.')
        .find(|part| !part.trim().is_empty())
        .unwrap_or(class)
        .to_ascii_lowercase()
}

pub(super) fn shell_escape(s: &str) -> String {
    format!("'{}'", s.replace('\'', "'\\''"))
}

pub(super) fn parse_xrandr_query(raw: &str) -> Vec<protocol::MonitorInfo> {
    let mut monitors = Vec::new();
    let mut current: Option<protocol::MonitorInfo> = None;

    for line in raw.lines() {
        if !line.starts_with(' ') && line.contains(" connected") {
            if let Some(monitor) = current.take() {
                monitors.push(monitor);
            }
            let parts: Vec<&str> = line.split_whitespace().collect();
            let geometry = parts
                .iter()
                .find(|part| part.contains('+') && part.contains('x'));
            let mut monitor = protocol::MonitorInfo {
                id: monitors.len() as u32,
                name: parts.first().copied().unwrap_or("").to_string(),
                width: 0,
                height: 0,
                scale: 1.0,
                primary: parts.contains(&"primary"),
                enabled: geometry.is_some(),
                x: 0,
                y: 0,
                refresh_rate: None,
                rotation: parse_xrandr_rotation(line),
            };
            if let Some(geometry) = geometry {
                parse_xrandr_geometry(geometry, &mut monitor);
            }
            current = Some(monitor);
            continue;
        }

        let Some(ref mut monitor) = current else {
            continue;
        };
        let trimmed = line.trim();
        if trimmed.contains('*') {
            let parts: Vec<&str> = trimmed.split_whitespace().collect();
            if let Some(refresh) = parts.iter().find(|part| part.contains('*')) {
                monitor.refresh_rate = refresh.trim_end_matches(['*', '+']).parse().ok();
            }
        }
    }

    if let Some(monitor) = current.take() {
        monitors.push(monitor);
    }
    monitors
}

pub(super) fn parse_xrandr_geometry(value: &str, monitor: &mut protocol::MonitorInfo) {
    if let Some(x_pos) = value.find('x') {
        monitor.width = value[..x_pos].parse().unwrap_or(0);
        let rest = &value[x_pos + 1..];
        let offset_start = rest.find(['+', '-']).unwrap_or(rest.len());
        monitor.height = rest[..offset_start].parse().unwrap_or(0);
        let offset_str = &rest[offset_start..];
        let mut parts = offset_str.split('+');
        let x_part = parts.next().unwrap_or("");
        monitor.x = x_part.parse().unwrap_or(0);
        for (i, part) in parts.enumerate() {
            if i == 0 {
                monitor.y = part.parse().unwrap_or(0);
            }
        }
    }
}

pub(super) fn parse_xrandr_rotation(line: &str) -> String {
    let rotations = ["normal", "left", "right", "inverted"];
    let active_segment = line.split('(').next().unwrap_or(line);
    let tokens: Vec<&str> = active_segment.split_whitespace().collect();

    if let Some(geometry_idx) = tokens
        .iter()
        .position(|part| part.contains('+') && part.contains('x'))
        && let Some(candidate) = tokens.get(geometry_idx + 1)
        && rotations.contains(candidate)
    {
        return (*candidate).to_string();
    }

    for token in tokens {
        if rotations.contains(&token) {
            return token.to_string();
        }
    }
    "normal".into()
}

pub(super) fn xrandr_rotation(rotation: &str) -> anyhow::Result<&'static str> {
    match rotation {
        "normal" => Ok("normal"),
        "left" => Ok("left"),
        "right" => Ok("right"),
        "inverted" => Ok("inverted"),
        _ => anyhow::bail!("unsupported monitor rotation: {}", rotation),
    }
}

pub(super) fn format_monitor_float(value: f64) -> String {
    let mut out = format!("{:.3}", value);
    while out.contains('.') && out.ends_with('0') {
        out.pop();
    }
    if out.ends_with('.') {
        out.pop();
    }
    out
}
