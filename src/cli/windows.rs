use clap::Subcommand;

#[derive(Subcommand)]
pub enum WindowCmd {
    /// List all windows
    List,
    /// Focus a window
    Focus { window_id: String },
    /// Get window details
    Get { window_id: String },
    /// Close a window
    Close { window_id: String },
    /// Minimize a window
    Minimize { window_id: String },
    /// Maximize a window
    Maximize { window_id: String },
    /// Move and resize a window
    MoveResize {
        window_id: String,
        x: i32,
        y: i32,
        width: u32,
        height: u32,
    },
    /// Tile a window to a preset: left, right, top, bottom, top_left, top_right, bottom_left, bottom_right, center, fill
    Tile {
        window_id: String,
        preset: String,
        #[arg(long)]
        monitor: Option<u32>,
        #[arg(long)]
        padding: Option<u32>,
    },
    /// Focus an app if open, launch it if not
    ActivateOrLaunch {
        app_id: String,
        /// Command to launch when no matching window exists. Defaults to app_id.
        #[arg(last = true)]
        command: Vec<String>,
    },
}
