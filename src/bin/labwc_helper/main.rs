//! Labwc desktop helper — bridges ext_foreign_toplevel_list_v1 to JSON-over-stdin/stdout.
//!
//! Usage: labwc-helper <command> [options]
//!
//! Implements ext_foreign_toplevel_list_v1 for window listing and control
//! via the labwc Wayland compositor.

mod commands;
mod helpers;
mod types;
mod wayland;

use helpers::parse_u64_arg;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: labwc-helper <command> [options]");
        eprintln!("Commands: probe, list-windows, activate, close,");
        eprintln!("          maximize, unmaximize, minimize, unminimize, fullscreen, unfullscreen");
        std::process::exit(1);
    }

    match args[1].as_str() {
        "probe" => commands::probe(),
        "list-windows" => commands::list_windows(),
        "activate" => {
            let wid = parse_u64_arg(&args, "--window-id").unwrap_or(0);
            commands::activate(wid);
        }
        "close" => {
            let wid = parse_u64_arg(&args, "--window-id").unwrap_or(0);
            commands::close(wid);
        }
        "maximize" => {
            let wid = parse_u64_arg(&args, "--window-id").unwrap_or(0);
            commands::set_maximized(wid, true);
        }
        "unmaximize" => {
            let wid = parse_u64_arg(&args, "--window-id").unwrap_or(0);
            commands::set_maximized(wid, false);
        }
        "minimize" => {
            let wid = parse_u64_arg(&args, "--window-id").unwrap_or(0);
            commands::set_minimized(wid, true);
        }
        "unminimize" => {
            let wid = parse_u64_arg(&args, "--window-id").unwrap_or(0);
            commands::set_minimized(wid, false);
        }
        "fullscreen" => {
            let wid = parse_u64_arg(&args, "--window-id").unwrap_or(0);
            commands::set_fullscreen(wid, true);
        }
        "unfullscreen" => {
            let wid = parse_u64_arg(&args, "--window-id").unwrap_or(0);
            commands::set_fullscreen(wid, false);
        }
        _ => {
            eprintln!("Unknown command: {}", args[1]);
            std::process::exit(1);
        }
    }
}
