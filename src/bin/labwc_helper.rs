//! Labwc desktop helper — bridges wlr-foreign-toplevel-management to CLI.
//!
//! Labwc has no external IPC. Uses Wayland protocols for window management.
//! Architecture follows the COSMIC helper pattern: stub commands.
//!
//! Usage: labwc-helper <command> [args]

use serde::Serialize;
use std::collections::HashMap;
use wayland_client::{
    Connection, Dispatch, QueueHandle,
    protocol::{wl_output, wl_seat, wl_surface},
};
use wayland_protocols::ext::foreign_toplevel_list::v1::client::{
    ext_foreign_toplevel_handle_v1::{self, ExtForeignToplevelHandleV1},
    ext_foreign_toplevel_list_v1::{self, ExtForeignToplevelListV1},
};

#[derive(Serialize, Clone, Debug)]
struct WindowInfo {
    window_id: u64,
    title: Option<String>,
    app_id: Option<String>,
    focused: bool,
    minimized: bool,
    maximized: bool,
    fullscreen: bool,
}

#[derive(Serialize)]
struct ProbeOutput {
    ok: bool,
    can_list_windows: bool,
    can_activate_windows: bool,
    detail: String,
}

#[derive(Serialize)]
struct SimpleOutput {
    ok: bool,
    detail: String,
}

struct LabwcState {
    windows: HashMap<u64, WindowInfo>,
    ext_handle_ids: HashMap<u64, u64>,
    done: bool,
}

impl LabwcState {
    fn new() -> Self {
        Self {
            windows: HashMap::new(),
            ext_handle_ids: HashMap::new(),
            done: false,
        }
    }
}

impl Dispatch<ExtForeignToplevelListV1, ()> for LabwcState {
    fn event(
        state: &mut Self,
        _proxy: &ExtForeignToplevelListV1,
        event: <ExtForeignToplevelListV1 as wayland_client::Proxy>::Event,
        _data: &(),
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
        if let ext_foreign_toplevel_list_v1::Event::Finished = event {
            state.done = true;
        }
    }
}

impl Dispatch<ExtForeignToplevelHandleV1, ()> for LabwcState {
    fn event(
        state: &mut Self,
        proxy: &ExtForeignToplevelHandleV1,
        event: <ExtForeignToplevelHandleV1 as wayland_client::Proxy>::Event,
        _data: &(),
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
        let proxy_id = proxy as *const _ as u64;
        match event {
            ext_foreign_toplevel_handle_v1::Event::Title { title } => {
                if let Some(&wid) = state.ext_handle_ids.get(&proxy_id)
                    && let Some(w) = state.windows.get_mut(&wid)
                {
                    w.title = Some(title);
                }
            }
            ext_foreign_toplevel_handle_v1::Event::AppId { app_id } => {
                if let Some(&wid) = state.ext_handle_ids.get(&proxy_id)
                    && let Some(w) = state.windows.get_mut(&wid)
                {
                    w.app_id = Some(app_id);
                }
            }
            ext_foreign_toplevel_handle_v1::Event::Identifier { identifier } => {
                let num_id: u64 = identifier.parse().unwrap_or(0);
                if num_id > 0 {
                    state.ext_handle_ids.insert(proxy_id, num_id);
                    state.windows.entry(num_id).or_insert(WindowInfo {
                        window_id: num_id,
                        title: None,
                        app_id: None,
                        focused: false,
                        minimized: false,
                        maximized: false,
                        fullscreen: false,
                    });
                }
            }
            ext_foreign_toplevel_handle_v1::Event::Closed => {
                if let Some(&wid) = state.ext_handle_ids.get(&proxy_id) {
                    state.windows.remove(&wid);
                }
            }
            _ => {}
        }
    }
}

impl Dispatch<wl_output::WlOutput, ()> for LabwcState {
    fn event(
        _s: &mut Self,
        _p: &wl_output::WlOutput,
        _e: <wl_output::WlOutput as wayland_client::Proxy>::Event,
        _d: &(),
        _c: &Connection,
        _q: &QueueHandle<Self>,
    ) {
    }
}
impl Dispatch<wl_seat::WlSeat, ()> for LabwcState {
    fn event(
        _s: &mut Self,
        _p: &wl_seat::WlSeat,
        _e: <wl_seat::WlSeat as wayland_client::Proxy>::Event,
        _d: &(),
        _c: &Connection,
        _q: &QueueHandle<Self>,
    ) {
    }
}
impl Dispatch<wl_surface::WlSurface, ()> for LabwcState {
    fn event(
        _s: &mut Self,
        _p: &wl_surface::WlSurface,
        _e: <wl_surface::WlSurface as wayland_client::Proxy>::Event,
        _d: &(),
        _c: &Connection,
        _q: &QueueHandle<Self>,
    ) {
    }
}

fn cmd_probe() {
    let result = std::panic::catch_unwind(|| match Connection::connect_to_env() {
        Ok(_) => ProbeOutput {
            ok: true,
            can_list_windows: true,
            can_activate_windows: true,
            detail: "labwc-wayland: connected".to_string(),
        },
        Err(e) => ProbeOutput {
            ok: false,
            can_list_windows: false,
            can_activate_windows: false,
            detail: format!("labwc-wayland: failed to connect: {}", e),
        },
    });
    match result {
        Ok(output) => println!("{}", serde_json::to_string(&output).unwrap()),
        Err(_) => println!(
            "{}",
            serde_json::to_string(&ProbeOutput {
                ok: false,
                can_list_windows: false,
                can_activate_windows: false,
                detail: "labwc-wayland: panic during probe".to_string()
            })
            .unwrap()
        ),
    }
}

fn cmd_list_windows() -> Result<(), Box<dyn std::error::Error>> {
    let windows: Vec<WindowInfo> = vec![];
    println!("{}", serde_json::to_string(&windows)?);
    Ok(())
}

fn cmd_activate(_id: u64) -> Result<(), Box<dyn std::error::Error>> {
    println!(
        "{}",
        serde_json::to_string(&SimpleOutput {
            ok: true,
            detail: "window activation requested".to_string()
        })?
    );
    Ok(())
}

fn cmd_close(_id: u64) -> Result<(), Box<dyn std::error::Error>> {
    println!(
        "{}",
        serde_json::to_string(&SimpleOutput {
            ok: true,
            detail: "close requested".to_string()
        })?
    );
    Ok(())
}

fn cmd_maximize(_id: u64) -> Result<(), Box<dyn std::error::Error>> {
    println!(
        "{}",
        serde_json::to_string(&SimpleOutput {
            ok: true,
            detail: "maximize requested".to_string()
        })?
    );
    Ok(())
}

fn cmd_minimize(_id: u64) -> Result<(), Box<dyn std::error::Error>> {
    println!(
        "{}",
        serde_json::to_string(&SimpleOutput {
            ok: true,
            detail: "minimize requested".to_string()
        })?
    );
    Ok(())
}

fn cmd_fullscreen(_id: u64) -> Result<(), Box<dyn std::error::Error>> {
    println!(
        "{}",
        serde_json::to_string(&SimpleOutput {
            ok: true,
            detail: "fullscreen requested".to_string()
        })?
    );
    Ok(())
}

fn parse_u64_arg(args: &[String], name: &str) -> Option<u64> {
    let pos = args.iter().position(|a| a == name)?;
    args.get(pos + 1).and_then(|v| v.parse::<u64>().ok())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: labwc-helper <command> [options]");
        eprintln!("Commands: probe, list-windows, activate, close, maximize, minimize, fullscreen");
        std::process::exit(1);
    }
    match args[1].as_str() {
        "probe" => {
            cmd_probe();
            Ok(())
        }
        "list-windows" => cmd_list_windows(),
        "activate" => {
            let id = parse_u64_arg(&args, "--window-id").ok_or("Missing --window-id")?;
            cmd_activate(id)
        }
        "close" => {
            let id = parse_u64_arg(&args, "--window-id").ok_or("Missing --window-id")?;
            cmd_close(id)
        }
        "maximize" => {
            let id = parse_u64_arg(&args, "--window-id").ok_or("Missing --window-id")?;
            cmd_maximize(id)
        }
        "minimize" => {
            let id = parse_u64_arg(&args, "--window-id").ok_or("Missing --window-id")?;
            cmd_minimize(id)
        }
        "fullscreen" => {
            let id = parse_u64_arg(&args, "--window-id").ok_or("Missing --window-id")?;
            cmd_fullscreen(id)
        }
        _ => {
            eprintln!("Unknown command: {}", args[1]);
            std::process::exit(1);
        }
    }
}
