use crate::DaemonState;
use std::sync::Arc;
use tracing::{error, info, warn};

const DASHBOARD_PORT: u16 = 20129;

const HTML_PAGE: &str = r##"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width, initial-scale=1.0">
<title>Deskbrid Dashboard</title>
<style>
  @import url('https://fonts.googleapis.com/css2?family=Inter:wght@400;600;700&family=JetBrains+Mono:wght@400;600&display=swap');
  * { margin: 0; padding: 0; box-sizing: border-box; }
  body {
    font-family: 'Inter', -apple-system, BlinkMacSystemFont, sans-serif;
    background: #0a0a0f; color: #e2e8f0; min-height: 100vh; padding: 1.5rem;
  }
  .container { max-width: 1400px; margin: 0 auto; }
  header {
    display: flex; align-items: center; justify-content: space-between;
    padding-bottom: 1rem; border-bottom: 1px solid #1e293b; margin-bottom: 1.5rem;
    flex-wrap: wrap; gap: 0.5rem;
  }
  header h1 {
    font-size: 1.5rem; font-weight: 700;
    background: linear-gradient(135deg, #06b6d4, #22d3ee);
    -webkit-background-clip: text; -webkit-text-fill-color: transparent; background-clip: text;
  }
  .header-right { display: flex; align-items: center; gap: 0.75rem; flex-wrap: wrap; }
  .version-badge {
    background: #0f172a; border: 1px solid #334155;
    color: #94a3b8; padding: 0.2rem 0.6rem; border-radius: 9999px;
    font-size: 0.75rem; font-family: 'JetBrains Mono', monospace;
  }
  .live-dot {
    display: inline-block; width: 8px; height: 8px; border-radius: 50%;
    background: #22c55e; box-shadow: 0 0 8px #22c55e66;
    animation: pulse 2s infinite;
  }
  @keyframes pulse { 0%,100%{opacity:1} 50%{opacity:0.4} }
  .live-text { font-size: 0.75rem; color: #22c55e; font-weight: 600; }
  .status-dot {
    display: inline-block; width: 10px; height: 10px; border-radius: 50%; margin-right: 0.4rem;
  }
  .status-dot.online { background: #22c55e; box-shadow: 0 0 8px #22c55e66; }
  .status-dot.offline { background: #ef4444; box-shadow: 0 0 8px #ef444466; }
  .grid { display: grid; grid-template-columns: repeat(3, 1fr); gap: 1rem; }
  @media (max-width: 1100px) { .grid { grid-template-columns: repeat(2, 1fr); } }
  @media (max-width: 700px) { .grid { grid-template-columns: 1fr; } body { padding: 0.75rem; } }
  .card {
    background: #0f172a; border: 1px solid #1e293b; border-radius: 10px; padding: 1rem;
    transition: border-color 0.3s;
  }
  .card.updated { border-color: #06b6d4; }
  .card.wide { grid-column: span 2; }
  @media (max-width: 1100px) { .card.wide { grid-column: span 1; } }
  .card h2 {
    font-size: 0.75rem; font-weight: 600; text-transform: uppercase;
    letter-spacing: 0.05em; color: #64748b; margin-bottom: 0.6rem;
    display: flex; align-items: center; gap: 0.4rem;
  }
  .card h2 .count {
    font-size: 0.65rem; background: #1e293b; color: #94a3b8;
    padding: 0.1rem 0.4rem; border-radius: 4px; font-weight: 400; margin-left: auto;
  }
  .kv { display: flex; justify-content: space-between; padding: 0.3rem 0; border-bottom: 1px solid #1e293b; font-size: 0.8rem; }
  .kv:last-child { border-bottom: none; }
  .kv .key { color: #94a3b8; }
  .kv .val { color: #e2e8f0; font-weight: 500; text-align: right; max-width: 60%; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .kv .val.accent { color: #06b6d4; font-family: 'JetBrains Mono', monospace; font-size: 0.75rem; }
  .window-row {
    display: flex; align-items: center; gap: 0.4rem; padding: 0.25rem 0;
    border-bottom: 1px solid #1e293b; font-size: 0.78rem;
  }
  .window-row:last-child { border-bottom: none; }
  .window-icon { font-size: 0.85rem; flex-shrink: 0; }
  .window-title { flex: 1; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .window-ws {
    background: #1e293b; color: #64748b; padding: 0.1rem 0.4rem;
    border-radius: 4px; font-size: 0.65rem; font-family: 'JetBrains Mono', monospace; flex-shrink: 0;
  }
  .window-focused { color: #06b6d4; font-weight: 600; }
  .audit-row {
    display: flex; gap: 0.4rem; padding: 0.2rem 0; border-bottom: 1px solid #1e293b;
    font-size: 0.72rem; align-items: center;
  }
  .audit-row:last-child { border-bottom: none; }
  .audit-status {
    padding: 0.1rem 0.3rem; border-radius: 3px; font-weight: 600;
    font-size: 0.62rem; text-transform: uppercase; flex-shrink: 0;
  }
  .audit-status.ok { background: #064e3b40; color: #22c55e; }
  .audit-status.error { background: #7f1d1d40; color: #ef4444; }
  .audit-action { color: #06b6d4; font-family: 'JetBrains Mono', monospace; font-size: 0.68rem; flex: 1; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .audit-ts { color: #64748b; font-size: 0.65rem; flex-shrink: 0; }
  .clip-entry {
    padding: 0.25rem 0; border-bottom: 1px solid #1e293b; font-size: 0.78rem;
    display: flex; gap: 0.5rem; align-items: baseline;
  }
  .clip-entry:last-child { border-bottom: none; }
  .clip-text { flex: 1; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; color: #e2e8f0; }
  .clip-src { color: #64748b; font-size: 0.65rem; flex-shrink: 0; }
  .clip-ts { color: #475569; font-size: 0.62rem; flex-shrink: 0; }
  .rule-row {
    display: flex; align-items: center; gap: 0.4rem; padding: 0.2rem 0;
    border-bottom: 1px solid #1e293b; font-size: 0.75rem;
  }
  .rule-row:last-child { border-bottom: none; }
  .rule-name { flex: 1; font-weight: 500; }
  .rule-trigger { color: #64748b; font-size: 0.65rem; font-family: 'JetBrains Mono', monospace; }
  .rule-enabled { color: #22c55e; font-size: 0.65rem; font-weight: 600; }
  .rule-disabled { color: #ef4444; font-size: 0.65rem; }
  .btn {
    display: inline-flex; align-items: center; gap: 0.4rem;
    background: #06b6d4; color: #0a0a0f; border: none; padding: 0.5rem 1rem;
    border-radius: 7px; font-weight: 600; font-size: 0.82rem; cursor: pointer;
    text-decoration: none; font-family: 'Inter', sans-serif; transition: background 0.2s;
  }
  .btn:hover { background: #22d3ee; }
  .btn-sm { padding: 0.3rem 0.6rem; font-size: 0.7rem; border-radius: 5px; }
  .btn-bar { display: flex; gap: 0.5rem; align-items: center; margin-bottom: 1.5rem; flex-wrap: wrap; }
  .screenshot-wrap { margin-bottom: 1rem; }
  .screenshot-wrap img { max-width: 100%; border-radius: 8px; border: 1px solid #1e293b; }
  .empty { color: #64748b; font-style: italic; font-size: 0.8rem; padding: 0.25rem 0; }
  .error-box { background: #7f1d1d20; border: 1px solid #7f1d1d40; border-radius: 8px; padding: 0.6rem 0.8rem; color: #fca5a5; font-size: 0.8rem; margin-bottom: 1rem; }
</style>
</head>
<body>
<div class="container">
<header>
  <h1>⚡ Deskbrid</h1>
  <div class="header-right">
    <span class="live-dot" id="live-dot"></span>
    <span class="live-text">SSE</span>
    <span class="version-badge">__VERSION__</span>
  </div>
</header>

<div class="btn-bar">
  <a href="?screenshot=1" class="btn">📸 Screenshot</a>
  <a href="/screenshot" class="btn" style="background:#1e293b;color:#e2e8f0">📸 Raw PNG</a>
  <span class="status-badge" style="font-size:0.8rem;color:#94a3b8;margin-left:auto">
    <span class="status-dot __STATUS_CLASS__"></span>__STATUS_TEXT__
  </span>
</div>

__ERROR_BOX__
__SCREENSHOT_HTML__

<div class="grid">
  <div class="card" id="card-system">    <h2>🖥 System</h2>    <div class="card-body">__SYSTEM__</div></div>
  <div class="card" id="card-monitors"> <h2>🖼 Monitors</h2>  <div class="card-body">__MONITORS__</div></div>
  <div class="card" id="card-network">  <h2>📡 Network</h2>   <div class="card-body">__NETWORK__</div></div>
  <div class="card" id="card-audio">    <h2>🔊 Audio</h2>     <div class="card-body">__AUDIO__</div></div>
  <div class="card wide" id="card-windows"><h2>🪟 Windows <span class="count" id="win-count"></span></h2><div class="card-body">__WINDOWS__</div></div>
  <div class="card wide" id="card-clipboard"><h2>📋 Clipboard <span class="count" id="clip-count"></span></h2><div class="card-body">__CLIPBOARD__</div></div>
  <div class="card wide" id="card-audit">   <h2>📋 Audit Log <span class="count" id="audit-count"></span></h2><div class="card-body">__AUDIT__</div></div>
  <div class="card" id="card-sessions"> <h2>👤 Sessions <span class="count" id="sess-count"></span></h2><div class="card-body">__SESSIONS__</div></div>
  <div class="card" id="card-rules">    <h2>⚙ Rules <span class="count" id="rule-count"></span></h2><div class="card-body">__RULES__</div></div>
  <div class="card" id="card-notifications"><h2>🔔 Notifications <span class="count" id="notif-count"></span></h2><div class="card-body">__NOTIFICATIONS__</div></div>
  <div class="card" id="card-macros">   <h2>🎬 Macros <span class="count" id="macro-count"></span></h2><div class="card-body">__MACROS__</div></div>
</div>
</div>

<script>
const ES = new EventSource('/events');
ES.onmessage = function(e) {
  try {
    const data = JSON.parse(e.data);
    if (data.card && data.html) {
      const card = document.getElementById('card-' + data.card);
      if (card) {
        card.querySelector('.card-body').innerHTML = data.html;
        card.classList.add('updated');
        setTimeout(() => card.classList.remove('updated'), 600);
      }
    }
    if (data.counts) {
      for (const [k, v] of Object.entries(data.counts)) {
        const el = document.getElementById(k + '-count');
        if (el) el.textContent = v;
      }
    }
  } catch(_) {}
};
ES.onerror = function() {
  document.getElementById('live-dot').style.background = '#ef4444';
};
</script>
</body>
</html>"##;

async fn build_page(state: &DaemonState, show_screenshot: bool) -> String {
    let mut page = HTML_PAGE
        .replace("__VERSION__", env!("CARGO_PKG_VERSION"))
        .to_string();

    let backend_guard = state.backend.read().await;
    let backend_available = backend_guard.is_some();

    let (status_class, status_text) = if backend_available {
        ("online", "daemon running")
    } else {
        ("offline", "no backend loaded")
    };
    page = page.replace("__STATUS_CLASS__", status_class);
    page = page.replace("__STATUS_TEXT__", status_text);

    let mut error_box = String::new();
    let mut screenshot_html = String::new();

    let system_info = if let Some(ref backend) = *backend_guard {
        match backend.system_info().await {
            Ok(info) => Some(info),
            Err(e) => {
                error_box = error_box_html(&format!("System info failed: {}", e));
                None
            }
        }
    } else {
        None
    };

    page = page.replace("__SYSTEM__", &render_system(&system_info));
    page = page.replace("__MONITORS__", &render_monitors(&system_info));
    page = page.replace("__NETWORK__", &render_network().await);
    page = page.replace("__AUDIO__", &render_audio().await);
    page = page.replace("__WINDOWS__", &render_windows(&backend_guard).await);
    page = page.replace("__CLIPBOARD__", &render_clipboard(state).await);
    page = page.replace("__AUDIT__", &render_audit(state).await);
    page = page.replace("__SESSIONS__", &render_sessions(state).await);
    page = page.replace("__RULES__", &render_rules(state).await);
    page = page.replace("__NOTIFICATIONS__", &render_notifications(state).await);
    page = page.replace("__MACROS__", &render_macros().await);

    if show_screenshot && let Some(ref backend) = *backend_guard {
        match backend.screenshot(None, None, None).await {
            Ok(result) => match std::fs::read(&result.path) {
                Ok(bytes) => {
                    let b64 = base64_encode(&bytes);
                    screenshot_html = format!(
                        r#"<div class="screenshot-wrap card"><h2>📸 Screenshot ({w}x{h})</h2><img src="data:image/png;base64,{b64}" alt="Screenshot"></div>"#,
                        w = result.width,
                        h = result.height,
                        b64 = b64,
                    );
                }
                Err(e) => {
                    screenshot_html = error_box_html(&format!("Failed to read screenshot: {}", e))
                }
            },
            Err(e) => screenshot_html = error_box_html(&format!("Screenshot failed: {}", e)),
        }
    }
    page = page.replace("__SCREENSHOT_HTML__", &screenshot_html);
    page = page.replace("__ERROR_BOX__", &error_box);
    page
}

// ── Card renderers (all public for SSE reuse) ────────────

fn render_system(info: &Option<crate::protocol::SystemInfo>) -> String {
    let Some(info) = info else {
        return r#"<div class="empty">No backend</div>"#.into();
    };
    let mut rows = String::new();
    rows.push_str(&kv("Desktop", &info.desktop));
    rows.push_str(&kv("Version", &info.desktop_version));
    rows.push_str(&kv("Compositor", &info.compositor));
    rows.push_str(&kv("Session", &info.session_type));
    rows.push_str(&kv(
        "Workspace",
        &format!("{}/{}", info.current_workspace, info.workspace_count),
    ));
    rows.push_str(&kv("Idle", &format!("{}s", info.idle_seconds)));
    rows
}

fn render_monitors(info: &Option<crate::protocol::SystemInfo>) -> String {
    let Some(info) = info else {
        return r#"<div class="empty">No backend</div>"#.into();
    };
    let mut rows = String::new();
    for m in &info.monitors {
        let star = if m.primary { " ⭐" } else { "" };
        let scale = if m.scale != 1.0 {
            format!(" ({}x)", m.scale)
        } else {
            String::new()
        };
        let hz = m
            .refresh_rate
            .map(|r| format!("{:.0}Hz", r))
            .unwrap_or_else(|| "?".into());
        rows.push_str(&kv(
            &format!("Monitor {}", m.id),
            &format!("{}x{} @ {}{}{}", m.width, m.height, hz, scale, star),
        ));
    }
    if rows.is_empty() {
        r#"<div class="empty">No monitors</div>"#.into()
    } else {
        rows
    }
}

async fn render_network() -> String {
    use tokio::process::Command;
    let status = Command::new("nmcli")
        .args(["-t", "-f", "STATE", "general", "status"])
        .output()
        .await;
    let mut rows = match status {
        Ok(o) if o.status.success() => {
            let state = String::from_utf8_lossy(&o.stdout).trim().to_string();
            kv("State", &state)
        }
        _ => return r#"<div class="empty">nmcli unavailable</div>"#.into(),
    };
    if let Ok(o) = Command::new("nmcli")
        .args([
            "-t",
            "-f",
            "DEVICE,TYPE,STATE,CONNECTION",
            "device",
            "status",
        ])
        .output()
        .await
        && o.status.success()
    {
        for line in String::from_utf8_lossy(&o.stdout).lines().take(4) {
            let parts: Vec<&str> = line.split(':').collect();
            if parts.len() >= 4 {
                rows.push_str(&kv(
                    parts[0],
                    &format!("{} — {} ({})", parts[1], parts[2], parts[3]),
                ));
            }
        }
    }
    rows
}

async fn render_audio() -> String {
    use tokio::process::Command;
    let out = Command::new("pactl")
        .args(["get-default-sink"])
        .output()
        .await;
    let sink = match out {
        Ok(o) if o.status.success() => String::from_utf8_lossy(&o.stdout).trim().to_string(),
        _ => return r#"<div class="empty">PipeWire/PulseAudio unavailable</div>"#.into(),
    };
    let vol_out = Command::new("pactl")
        .args(["get-sink-volume", &sink])
        .output()
        .await;
    let mut rows = kv("Sink", &sink);
    if let Ok(o) = vol_out
        && o.status.success()
    {
        let txt = String::from_utf8_lossy(&o.stdout);
        if let Some(vol) = txt.split('/').nth(1) {
            let pct: i32 = vol.trim().parse().unwrap_or(0);
            rows.push_str(&kv(
                "Volume",
                &format!("{} {}%", volume_bar(pct as u8), pct),
            ));
        }
    }
    let mute_out = Command::new("pactl")
        .args(["get-sink-mute", &sink])
        .output()
        .await;
    if let Ok(o) = mute_out
        && o.status.success()
    {
        let txt = String::from_utf8_lossy(&o.stdout);
        let muted = txt.contains("yes");
        rows.push_str(&kv("Muted", if muted { "🔇 Yes" } else { "🔊 No" }));
    }
    rows
}

async fn render_windows(backend: &Option<Box<dyn crate::backend::DesktopBackend>>) -> String {
    let Some(backend) = backend else {
        return r#"<div class="empty">No backend</div>"#.into();
    };
    match backend.windows_list().await {
        Ok(windows) => {
            if windows.is_empty() {
                return r#"<div class="empty">No windows</div>"#.into();
            }
            let mut rows = String::new();
            for w in windows.iter().take(30) {
                let fc = if w.is_focused { "window-focused" } else { "" };
                let min = if w.is_minimized { " 🗕" } else { "" };
                let title = if w.title.is_empty() {
                    &w.app_id
                } else {
                    &w.title
                };
                rows.push_str(&format!(
                    r#"<div class="window-row"><span class="window-icon">🪟</span><span class="window-title {fc}">{t}{min}</span><span class="window-ws">WS{ws}</span></div>"#,
                    fc = fc,
                    t = html_escape(title),
                    min = min,
                    ws = w.workspace_id,
                ));
            }
            rows
        }
        Err(e) => format!(
            r#"<div class="empty">Error: {}</div>"#,
            html_escape(&e.to_string())
        ),
    }
}

async fn render_clipboard(state: &DaemonState) -> String {
    let db = state.database.lock().await;
    match db.get_clipboard_history(10, None) {
        Ok(entries) => {
            if entries.is_empty() {
                return r#"<div class="empty">No clipboard history</div>"#.into();
            }
            let mut rows = String::new();
            for e in entries.iter().rev() {
                let text = if e.text.len() > 80 {
                    format!("{}…", &e.text[..80])
                } else {
                    e.text.clone()
                };
                rows.push_str(&format!(
                    r#"<div class="clip-entry"><span class="clip-text">{}</span><span class="clip-src">{}</span><span class="clip-ts">#{}</span></div>"#,
                    html_escape(&text),
                    html_escape(&e.source),
                    e.id,
                ));
            }
            rows
        }
        Err(_) => r#"<div class="empty">DB unavailable</div>"#.into(),
    }
}

async fn render_audit(state: &DaemonState) -> String {
    let entries = state.audit_log.lock().await;
    if entries.is_empty() {
        return r#"<div class="empty">No audit entries yet</div>"#.into();
    }
    let count = entries.len();
    let show = count.min(30);
    let start = count - show;
    let mut rows = String::new();
    for (i, entry) in entries.iter().enumerate() {
        if i < start {
            continue;
        }
        let sc = if entry.status == "ok" { "ok" } else { "error" };
        let dur = if entry.duration_ms >= 1000 {
            format!("{:.1}s", entry.duration_ms as f64 / 1000.0)
        } else {
            format!("{}ms", entry.duration_ms)
        };
        let mut ad = entry.action_type.clone();
        if let Some(ref err) = entry.error {
            ad.push_str(&format!(" — {}", err));
        }
        rows.push_str(&format!(
            r#"<div class="audit-row"><span class="audit-status {sc}">{st}</span><span class="audit-action">{ac}</span><span style="color:#64748b;font-size:0.65rem">uid:{uid} {dur}</span><span class="audit-ts">#{id}</span></div>"#,
            sc = sc,
            st = entry.status,
            ac = html_escape(&ad),
            uid = entry.peer_uid,
            dur = dur,
            id = entry.id,
        ));
    }
    rows
}

async fn render_sessions(state: &DaemonState) -> String {
    let sessions = state.sessions.lock().await;
    if sessions.is_empty() {
        return r#"<div class="empty">No sessions</div>"#.into();
    }
    let mut rows = String::new();
    for s in sessions.values() {
        rows.push_str(&kv(&s.name, &format!("{} vars", s.vars.len())));
    }
    rows
}

async fn render_rules(state: &DaemonState) -> String {
    let rules = state.rules.lock().await;
    let list = rules.list();
    if list.is_empty() {
        return r#"<div class="empty">No rules configured</div>"#.into();
    }
    let mut rows = String::new();
    for r in list.iter().take(10) {
        let enabled = if r.enabled {
            r#"<span class="rule-enabled">ON</span>"#
        } else {
            r#"<span class="rule-disabled">OFF</span>"#
        };
        rows.push_str(&format!(
            r#"<div class="rule-row"><span class="rule-name">{}</span><span class="rule-trigger">{:?}</span>{}</div>"#,
            html_escape(&r.name),
            r.trigger,
            enabled,
        ));
    }
    rows
}

async fn render_notifications(state: &DaemonState) -> String {
    let db = state.database.lock().await;
    match db.get_notifications(8, None, None) {
        Ok(entries) => {
            if entries.is_empty() {
                return r#"<div class="empty">No notifications</div>"#.into();
            }
            let mut rows = String::new();
            for n in entries.iter().rev() {
                let app = n["app_name"].as_str().unwrap_or("?");
                let title = n["title"].as_str().unwrap_or("(no title)");
                let id = n["id"].as_u64().unwrap_or(0);
                rows.push_str(&format!(
                    r#"<div class="audit-row"><span style="color:#64748b;font-size:0.65rem">{}</span><span style="flex:1;overflow:hidden;text-overflow:ellipsis;white-space:nowrap;font-size:0.75rem">{}</span><span class="audit-ts">#{}</span></div>"#,
                    html_escape(app),
                    html_escape(title),
                    id,
                ));
            }
            rows
        }
        Err(_) => r#"<div class="empty">DB unavailable</div>"#.into(),
    }
}

async fn render_macros() -> String {
    match crate::daemon::macro_engine::list_macros() {
        Ok(list) => {
            if list.is_empty() {
                return r#"<div class="empty">No macros recorded</div>"#.into();
            }
            let mut rows = String::new();
            for m in list.iter().take(8) {
                rows.push_str(&kv(&m.name, &format!("{} actions", m.action_count)));
            }
            rows
        }
        Err(_) => r#"<div class="empty">Macro engine unavailable</div>"#.into(),
    }
}

// ── SSE update helpers ───────────────────────────────────

async fn sse_card_html(card: &str, state: &DaemonState) -> String {
    match card {
        "system" => {
            let backend = state.backend.read().await;
            let info = if let Some(ref b) = *backend {
                b.system_info().await.ok()
            } else {
                None
            };
            render_system(&info)
        }
        "monitors" => {
            let backend = state.backend.read().await;
            let info = if let Some(ref b) = *backend {
                b.system_info().await.ok()
            } else {
                None
            };
            render_monitors(&info)
        }
        "windows" => {
            let backend = state.backend.read().await;
            render_windows(&backend).await
        }
        "clipboard" => render_clipboard(state).await,
        "audit" => render_audit(state).await,
        "network" => render_network().await,
        "audio" => render_audio().await,
        "sessions" => render_sessions(state).await,
        "rules" => render_rules(state).await,
        "notifications" => render_notifications(state).await,
        "macros" => render_macros().await,
        _ => r#"<div class="empty">Unknown card</div>"#.into(),
    }
}

// ── Helpers ──────────────────────────────────────────────

fn kv(key: &str, value: &str) -> String {
    format!(
        r#"<div class="kv"><span class="key">{}</span><span class="val">{}</span></div>"#,
        html_escape(key),
        html_escape(value)
    )
}

fn volume_bar(vol: u8) -> String {
    let n = (vol.min(100) / 10) as usize;
    let filled = "█".repeat(n);
    let empty = "░".repeat(10 - n);
    format!("{}{}", filled, empty)
}

fn error_box_html(msg: &str) -> String {
    format!(r#"<div class="error-box">⚠ {}</div>"#, html_escape(msg))
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

fn base64_encode(data: &[u8]) -> String {
    use base64::Engine;
    base64::engine::general_purpose::STANDARD.encode(data)
}

// ── HTTP ─────────────────────────────────────────────────

fn parse_request_line(line: &str) -> Option<(&str, &str)> {
    let mut parts = line.split_whitespace();
    let method = parts.next()?;
    let path = parts.next()?;
    Some((method, path))
}

fn http_response(status: u16, content_type: &str, body: &[u8]) -> Vec<u8> {
    let status_text = match status {
        200 => "OK",
        404 => "Not Found",
        _ => "Internal Server Error",
    };
    let header = format!(
        "HTTP/1.1 {} {}\r\nContent-Type: {}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        status,
        status_text,
        content_type,
        body.len()
    );
    let mut response = header.into_bytes();
    response.extend_from_slice(body);
    response
}

async fn handle_request(
    mut stream: tokio::net::TcpStream,
    state: Arc<DaemonState>,
) -> anyhow::Result<()> {
    use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

    let (read_half, mut write_half) = stream.split();
    let mut reader = BufReader::new(read_half);
    let mut request_line = String::new();
    reader.read_line(&mut request_line).await?;

    let (method, path) = parse_request_line(request_line.trim()).unwrap_or(("GET", "/"));

    loop {
        let mut line = String::new();
        reader.read_line(&mut line).await?;
        if line.trim().is_empty() {
            break;
        }
    }

    // SSE event stream — polls cards every 3 seconds
    if method == "GET" && path == "/events" {
        let header = "HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\nCache-Control: no-cache\r\nConnection: keep-alive\r\n\r\n";
        write_half.write_all(header.as_bytes()).await?;
        write_half.flush().await?;

        let connected = "data: {\"type\":\"connected\"}\n\n";
        write_half.write_all(connected.as_bytes()).await?;
        write_half.flush().await?;

        // Poll loop
        let volatile_cards = [
            "windows",
            "clipboard",
            "audit",
            "network",
            "audio",
            "sessions",
            "rules",
            "notifications",
            "macros",
        ];
        loop {
            tokio::time::sleep(std::time::Duration::from_secs(3)).await;
            for card in &volatile_cards {
                let html = sse_card_html(card, &state).await;
                let json = serde_json::json!({"card": card, "html": html});
                if let Ok(line) = serde_json::to_string(&json) {
                    let sse = format!("data: {}\n\n", line);
                    if write_half.write_all(sse.as_bytes()).await.is_err() {
                        return Ok(()); // client disconnected
                    }
                }
            }
            if write_half.flush().await.is_err() {
                return Ok(());
            }
        }
    }

    if method == "GET" && (path == "/screenshot" || path.starts_with("/screenshot?")) {
        let backend_guard = state.backend.read().await;
        if let Some(ref backend) = *backend_guard {
            match backend.screenshot(None, None, None).await {
                Ok(result) => match std::fs::read(&result.path) {
                    Ok(bytes) => {
                        drop(backend_guard);
                        write_half
                            .write_all(&http_response(200, "image/png", &bytes))
                            .await?;
                        return Ok(());
                    }
                    Err(e) => {
                        drop(backend_guard);
                        let body = format!("Failed to read screenshot: {}", e);
                        write_half
                            .write_all(&http_response(500, "text/plain", body.as_bytes()))
                            .await?;
                        return Ok(());
                    }
                },
                Err(e) => {
                    drop(backend_guard);
                    let body = format!("Screenshot failed: {}", e);
                    write_half
                        .write_all(&http_response(500, "text/plain", body.as_bytes()))
                        .await?;
                    return Ok(());
                }
            }
        } else {
            drop(backend_guard);
            write_half
                .write_all(&http_response(503, "text/plain", b"No backend loaded"))
                .await?;
            return Ok(());
        }
    }

    if method == "GET" && (path == "/" || path.starts_with("/?")) {
        let show_screenshot = path.contains("screenshot=1");
        let html = build_page(&state, show_screenshot).await;
        write_half
            .write_all(&http_response(
                200,
                "text/html; charset=utf-8",
                html.as_bytes(),
            ))
            .await?;
    } else {
        write_half
            .write_all(&http_response(404, "text/plain", b"Not Found"))
            .await?;
    }

    Ok(())
}

pub async fn start(state: Arc<DaemonState>) {
    let addr = format!("0.0.0.0:{}", DASHBOARD_PORT);
    let listener = match tokio::net::TcpListener::bind(&addr).await {
        Ok(l) => l,
        Err(e) => {
            error!("Dashboard bind {}: {}", addr, e);
            return;
        }
    };
    info!("Dashboard: http://{}", addr);
    loop {
        match listener.accept().await {
            Ok((stream, _)) => {
                let state = Arc::clone(&state);
                tokio::spawn(async move {
                    if let Err(e) = handle_request(stream, state).await {
                        warn!("Dashboard: {}", e);
                    }
                });
            }
            Err(e) => error!("Dashboard accept: {}", e),
        }
    }
}
