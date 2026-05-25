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
<meta http-equiv="refresh" content="5">
<style>
  @import url('https://fonts.googleapis.com/css2?family=Inter:wght@400;600;700&family=JetBrains+Mono:wght@400;600&display=swap');
  * { margin: 0; padding: 0; box-sizing: border-box; }
  body {
    font-family: 'Inter', -apple-system, BlinkMacSystemFont, sans-serif;
    background: #0a0a0f;
    color: #e2e8f0;
    min-height: 100vh;
    padding: 2rem;
  }
  .container { max-width: 960px; margin: 0 auto; }
  header {
    display: flex; align-items: center; justify-content: space-between;
    padding-bottom: 1.5rem; border-bottom: 1px solid #1e293b; margin-bottom: 2rem;
  }
  header h1 {
    font-size: 1.75rem; font-weight: 700;
    background: linear-gradient(135deg, #06b6d4, #22d3ee);
    -webkit-background-clip: text; -webkit-text-fill-color: transparent;
    background-clip: text;
  }
  .version-badge {
    background: #0f172a; border: 1px solid #334155;
    color: #94a3b8; padding: 0.25rem 0.75rem; border-radius: 9999px;
    font-size: 0.8rem; font-family: 'JetBrains Mono', monospace;
  }
  .status-dot {
    display: inline-block; width: 10px; height: 10px; border-radius: 50%;
    margin-right: 0.5rem;
  }
  .status-dot.online { background: #22c55e; box-shadow: 0 0 8px #22c55e66; }
  .status-dot.offline { background: #ef4444; box-shadow: 0 0 8px #ef444466; }
  .status-badge {
    display: inline-flex; align-items: center; gap: 0.4rem;
    font-size: 0.85rem; font-weight: 600;
  }
  .grid { display: grid; grid-template-columns: 1fr 1fr; gap: 1.25rem; }
  .card {
    background: #0f172a; border: 1px solid #1e293b; border-radius: 12px;
    padding: 1.25rem;
  }
  .card.full { grid-column: 1 / -1; }
  .card h2 {
    font-size: 0.85rem; font-weight: 600; text-transform: uppercase;
    letter-spacing: 0.05em; color: #64748b; margin-bottom: 0.75rem;
  }
  .kv { display: flex; justify-content: space-between; padding: 0.35rem 0; border-bottom: 1px solid #1e293b; }
  .kv:last-child { border-bottom: none; }
  .kv .key { color: #94a3b8; font-size: 0.85rem; }
  .kv .val { color: #e2e8f0; font-weight: 500; font-size: 0.85rem; text-align: right; }
  .kv .val.accent { color: #06b6d4; font-family: 'JetBrains Mono', monospace; }
  .window-row {
    display: flex; align-items: center; gap: 0.5rem; padding: 0.35rem 0;
    border-bottom: 1px solid #1e293b; font-size: 0.82rem;
  }
  .window-row:last-child { border-bottom: none; }
  .window-icon { font-size: 1rem; }
  .window-title { flex: 1; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .window-ws {
    background: #1e293b; color: #64748b; padding: 0.1rem 0.5rem;
    border-radius: 4px; font-size: 0.7rem; font-family: 'JetBrains Mono', monospace;
  }
  .window-focused { color: #06b6d4; font-weight: 600; }
  .audit-row {
    display: flex; gap: 0.5rem; padding: 0.3rem 0; border-bottom: 1px solid #1e293b;
    font-size: 0.78rem; align-items: center;
  }
  .audit-row:last-child { border-bottom: none; }
  .audit-status {
    padding: 0.1rem 0.4rem; border-radius: 4px; font-weight: 600;
    font-size: 0.7rem; text-transform: uppercase;
  }
  .audit-status.ok { background: #064e3b40; color: #22c55e; }
  .audit-status.error { background: #7f1d1d40; color: #ef4444; }
  .audit-action { color: #06b6d4; font-family: 'JetBrains Mono', monospace; font-size: 0.72rem; }
  .audit-ts { color: #64748b; font-size: 0.7rem; margin-left: auto; }
  .btn {
    display: inline-flex; align-items: center; gap: 0.5rem;
    background: #06b6d4; color: #0a0a0f; border: none; padding: 0.6rem 1.25rem;
    border-radius: 8px; font-weight: 600; font-size: 0.9rem; cursor: pointer;
    text-decoration: none; font-family: 'Inter', sans-serif; transition: background 0.2s;
  }
  .btn:hover { background: #22d3ee; }
  .btn:active { background: #0891b2; }
  .btn-bar { display: flex; gap: 0.75rem; align-items: center; margin-bottom: 1.5rem; }
  .screenshot-wrap { margin-top: 1.5rem; }
  .screenshot-wrap img { max-width: 100%; border-radius: 8px; border: 1px solid #1e293b; }
  .empty { color: #64748b; font-style: italic; font-size: 0.85rem; padding: 0.5rem 0; }
  .error-box { background: #7f1d1d20; border: 1px solid #7f1d1d40; border-radius: 8px; padding: 0.75rem 1rem; color: #fca5a5; font-size: 0.85rem; margin-bottom: 1rem; }
  .warn-box { background: #78350f20; border: 1px solid #78350f40; border-radius: 8px; padding: 0.75rem 1rem; color: #fcd34d; font-size: 0.85rem; margin-bottom: 1rem; }
  @media (max-width: 640px) {
    body { padding: 1rem; }
    .grid { grid-template-columns: 1fr; }
    header { flex-direction: column; align-items: flex-start; gap: 0.5rem; }
  }
</style>
</head>
<body>
<div class="container">
<header>
  <h1>⚡ Deskbrid</h1>
  <span class="version-badge">__VERSION__</span>
</header>

<div class="btn-bar">
  <span class="status-badge"><span class="status-dot __STATUS_CLASS__"></span>__STATUS_TEXT__</span>
  <span style="color:#64748b">·</span>
  <span class="status-badge" style="font-weight:400;font-size:0.85rem;color:#94a3b8">Auto-refresh: 5s</span>
  <a href="?screenshot=1" class="btn" style="margin-left:auto">📸 Take Screenshot</a>
</div>

__ERROR_BOX__
__WARN_BOX__
__SCREENSHOT_HTML__

<div class="grid">
  <div class="card">
    <h2>🖥 System</h2>
    __SYSTEM_INFO__
  </div>
  <div class="card">
    <h2>🖼 Monitors</h2>
    __MONITORS__
  </div>
  <div class="card full">
    <h2>🪟 Windows</h2>
    __WINDOWS__
  </div>
  <div class="card full">
    <h2>📋 Audit Log (recent)</h2>
    __AUDIT__
  </div>
</div>
</div>
</body>
</html>"##;

/// Build the HTML page with live data from the daemon state.
async fn build_page(state: &DaemonState, show_screenshot: bool) -> (String, Vec<u8>) {
    let mut page = HTML_PAGE
        .replace("__VERSION__", env!("CARGO_PKG_VERSION"))
        .to_string();

    // Status and backend check
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
    let mut warn_box = String::new();
    let mut screenshot_html = String::new();

    // Fetch system info once (used for both system card and monitors card)
    let system_info = if let Some(ref backend) = *backend_guard {
        match backend.system_info().await {
            Ok(info) => Some(info),
            Err(e) => {
                error_box = error_box_html(&format!("Failed to get system info: {}", e));
                None
            }
        }
    } else {
        warn_box =
            warn_box_html("No desktop backend loaded — window/screenshot features unavailable");
        None
    };

    // System card
    if let Some(ref info) = system_info {
        let mut rows = String::new();
        rows.push_str(&kv("Desktop", &info.desktop));
        rows.push_str(&kv("Version", &info.desktop_version));
        rows.push_str(&kv("Compositor", &info.compositor));
        rows.push_str(&kv("Session Type", &info.session_type));
        rows.push_str(&kv(
            "Workspace",
            &format!("{}/{}", info.current_workspace, info.workspace_count),
        ));
        rows.push_str(&kv("Idle", &format!("{}s", info.idle_seconds)));
        page = page.replace("__SYSTEM_INFO__", &rows);
    } else {
        page = page.replace("__SYSTEM_INFO__", "");
    }

    // Monitors card
    if let Some(ref info) = system_info {
        let mut rows = String::new();
        for m in &info.monitors {
            let primary_marker = if m.primary { " ⭐" } else { "" };
            rows.push_str(&kv(
                &format!("Monitor {}", m.id),
                &format!(
                    "{}x{} @ {}Hz{}{}",
                    m.width,
                    m.height,
                    m.refresh_rate
                        .map(|r| format!("{:.0}", r))
                        .unwrap_or_else(|| "?".into()),
                    if m.scale != 1.0 {
                        format!(" ({}x)", m.scale)
                    } else {
                        String::new()
                    },
                    primary_marker
                ),
            ));
        }
        if rows.is_empty() {
            page = page.replace(
                "__MONITORS__",
                "<div class=\"empty\">No monitors detected</div>",
            );
        } else {
            page = page.replace("__MONITORS__", &rows);
        }
    } else {
        page = page.replace("__MONITORS__", "<div class=\"empty\">No backend</div>");
    }

    // Windows
    let windows_html = if let Some(ref backend) = *backend_guard {
        match backend.windows_list().await {
            Ok(windows) => {
                if windows.is_empty() {
                    "<div class=\"empty\">No windows found</div>".to_string()
                } else {
                    let mut rows = String::new();
                    for w in &windows {
                        let focus_class = if w.is_focused { "window-focused" } else { "" };
                        let minimized = if w.is_minimized { " 🗕" } else { "" };
                        let title = if w.title.is_empty() {
                            &w.app_id
                        } else {
                            &w.title
                        };
                        rows.push_str(&format!(
                            r#"<div class="window-row"><span class="window-icon">🪟</span><span class="window-title {focus_class}">{title}{minimized}</span><span class="window-ws">WS{w}</span></div>"#,
                            focus_class = focus_class,
                            title = html_escape(title),
                            w = w.workspace_id,
                            minimized = minimized,
                        ));
                    }
                    rows
                }
            }
            Err(e) => {
                format!(
                    "<div class=\"empty\">Error listing windows: {}</div>",
                    html_escape(&e.to_string())
                )
            }
        }
    } else {
        "<div class=\"empty\">No backend loaded</div>".to_string()
    };
    page = page.replace("__WINDOWS__", &windows_html);

    // Audit log
    let audit_html = {
        let entries = state.audit_log.lock().await;
        if entries.is_empty() {
            "<div class=\"empty\">No audit entries yet</div>".to_string()
        } else {
            let mut rows = String::new();
            let count = entries.len();
            let show_count = count.min(50);
            let start = count - show_count;
            for (i, entry) in entries.iter().enumerate() {
                if i >= start {
                    let status_class = if entry.status == "ok" { "ok" } else { "error" };
                    let duration = if entry.duration_ms >= 1000 {
                        format!("{:.1}s", entry.duration_ms as f64 / 1000.0)
                    } else {
                        format!("{}ms", entry.duration_ms)
                    };
                    let mut action_display = entry.action_type.clone();
                    if let Some(ref err) = entry.error {
                        action_display.push_str(&format!(" — {}", err));
                    }
                    rows.push_str(&format!(
                        r#"<div class="audit-row"><span class="audit-status {status_class}">{status}</span><span class="audit-action">{action}</span><span style="color:#64748b;font-size:0.7rem">uid:{uid} {dur}</span><span class="audit-ts">#{id}</span></div>"#,
                        status_class = status_class,
                        status = entry.status,
                        action = html_escape(&action_display),
                        uid = entry.peer_uid,
                        dur = duration,
                        id = entry.id,
                    ));
                }
            }
            rows
        }
    };
    page = page.replace("__AUDIT__", &audit_html);

    // Screenshot
    if show_screenshot {
        if let Some(ref backend) = *backend_guard {
            match backend.screenshot(None, None, None).await {
                Ok(result) => match std::fs::read(&result.path) {
                    Ok(bytes) => {
                        let b64 = base64_encode(&bytes);
                        screenshot_html = format!(
                            r#"<div class="screenshot-wrap card" style="margin-bottom:1.5rem"><h2>📸 Screenshot ({w}x{h})</h2><img src="data:image/png;base64,{b64}" alt="Screenshot"></div>"#,
                            w = result.width,
                            h = result.height,
                            b64 = b64,
                        );
                    }
                    Err(e) => {
                        screenshot_html = error_box_html(&format!(
                            "Screenshot captured but failed to read file: {}",
                            e
                        ));
                    }
                },
                Err(e) => {
                    screenshot_html = error_box_html(&format!("Screenshot failed: {}", e));
                }
            }
        } else {
            screenshot_html = warn_box_html("No backend loaded — cannot capture screenshot");
        }
    }
    page = page.replace("__SCREENSHOT_HTML__", &screenshot_html);

    // Drop the backend guard
    drop(backend_guard);

    page = page.replace("__ERROR_BOX__", &error_box);
    page = page.replace("__WARN_BOX__", &warn_box);

    (page, screenshot_html.into_bytes())
}

fn kv(key: &str, value: &str) -> String {
    format!(
        r#"<div class="kv"><span class="key">{}</span><span class="val">{}</span></div>"#,
        html_escape(key),
        html_escape(value)
    )
}

fn error_box_html(msg: &str) -> String {
    format!(r#"<div class="error-box">{}</div>"#, html_escape(msg))
}

fn warn_box_html(msg: &str) -> String {
    format!(r#"<div class="warn-box">⚠ {}</div>"#, html_escape(msg))
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

/// Minimal HTTP request parser — returns (method, path).
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

/// Handle a single HTTP request.
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

    // Drain headers
    loop {
        let mut line = String::new();
        reader.read_line(&mut line).await?;
        if line.trim().is_empty() {
            break;
        }
    }

    let show_screenshot = path.contains("screenshot=1");

    // Route: /screenshot (raw PNG) → screenshot capture
    // Route: / or /?... → HTML dashboard page
    if method == "GET" && (path == "/screenshot" || path.starts_with("/screenshot?")) {
        // Capture and serve raw PNG
        let backend_guard = state.backend.read().await;
        if let Some(ref backend) = *backend_guard {
            match backend.screenshot(None, None, None).await {
                Ok(result) => match std::fs::read(&result.path) {
                    Ok(bytes) => {
                        drop(backend_guard);
                        let response = http_response(200, "image/png", &bytes);
                        write_half.write_all(&response).await?;
                        return Ok(());
                    }
                    Err(e) => {
                        drop(backend_guard);
                        let body = format!("Failed to read screenshot: {}", e);
                        let response = http_response(500, "text/plain", body.as_bytes());
                        write_half.write_all(&response).await?;
                        return Ok(());
                    }
                },
                Err(e) => {
                    drop(backend_guard);
                    let body = format!("Screenshot failed: {}", e);
                    let response = http_response(500, "text/plain", body.as_bytes());
                    write_half.write_all(&response).await?;
                    return Ok(());
                }
            }
        } else {
            drop(backend_guard);
            let body = "No backend loaded";
            let response = http_response(503, "text/plain", body.as_bytes());
            write_half.write_all(&response).await?;
        }
    } else if method == "GET" && (path == "/" || path.starts_with("/?")) {
        // Serve the HTML dashboard page
        let (html, _screenshot_data) = build_page(&state, show_screenshot).await;
        let response = http_response(200, "text/html; charset=utf-8", html.as_bytes());
        write_half.write_all(&response).await?;
    } else {
        let body = "Not Found";
        let response = http_response(404, "text/plain", body.as_bytes());
        write_half.write_all(&response).await?;
    }

    Ok(())
}

/// Start the dashboard HTTP server on port 20129.
pub async fn start(state: Arc<DaemonState>) {
    let addr = format!("0.0.0.0:{}", DASHBOARD_PORT);
    let listener = match tokio::net::TcpListener::bind(&addr).await {
        Ok(l) => l,
        Err(e) => {
            error!("Dashboard: failed to bind {}: {}", addr, e);
            return;
        }
    };

    info!("Dashboard listening on http://{}", addr);

    loop {
        match listener.accept().await {
            Ok((stream, _)) => {
                let state = Arc::clone(&state);
                tokio::spawn(async move {
                    if let Err(e) = handle_request(stream, state).await {
                        warn!("Dashboard request error: {}", e);
                    }
                });
            }
            Err(e) => {
                error!("Dashboard accept error: {}", e);
            }
        }
    }
}
