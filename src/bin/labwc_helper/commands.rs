use super::helpers::ok_json;
use super::wayland::list_windows_wayland;

pub(crate) fn probe() {
    match std::env::var("WAYLAND_DISPLAY") {
        Ok(socket) => {
            let xdg = std::env::var("XDG_RUNTIME_DIR").expect("XDG_RUNTIME_DIR must be set");
            let path = format!("{xdg}/{socket}");
            if std::path::Path::new(&path).exists() {
                println!("{{\"ok\": true, \"compositor\": \"labwc\", \"socket\": \"{path}\"}}");
            } else {
                println!("{{\"ok\": false, \"error\": \"Wayland socket not found: {path}\"}}");
            }
        }
        Err(_) => {
            println!("{{\"ok\": false, \"error\": \"WAYLAND_DISPLAY not set\"}}");
        }
    }
}

pub(crate) fn list_windows() {
    let windows = list_windows_wayland();
    println!("{}", serde_json::to_string(&windows).unwrap());
}

pub(crate) fn activate(_window_id: u64) {
    ok_json(Some("activate stubbed"));
}

pub(crate) fn close(_window_id: u64) {
    ok_json(Some("close stubbed"));
}

pub(crate) fn set_maximized(window_id: u64, on: bool) {
    let action = if on { "maximize" } else { "unmaximize" };
    ok_json(Some(&format!("{action} window_id={window_id} stubbed")));
}

pub(crate) fn set_minimized(window_id: u64, on: bool) {
    let action = if on { "minimize" } else { "unminimize" };
    ok_json(Some(&format!("{action} window_id={window_id} stubbed")));
}

pub(crate) fn set_fullscreen(window_id: u64, on: bool) {
    let action = if on { "fullscreen" } else { "unfullscreen" };
    ok_json(Some(&format!("{action} window_id={window_id} stubbed")));
}
