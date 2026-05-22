use clap::Subcommand;

#[derive(Subcommand)]
pub enum WorkspaceCmd {
    /// List workspaces
    List,
    /// Switch to a workspace
    Switch { workspace_id: u32 },
    /// Move window to workspace
    Move {
        window_id: String,
        workspace_id: u32,
        #[arg(long)]
        follow: bool,
    },
}

#[derive(Subcommand)]
pub enum ProfileCmd {
    /// List saved layout profiles
    List,
    /// Save the current window/workspace layout
    Save {
        name: String,
        #[arg(long)]
        overwrite: bool,
    },
    /// Show one saved layout profile
    Get { name: String },
    /// Delete a saved layout profile
    Delete { name: String },
    /// Restore a saved layout profile
    Restore { name: String },
}
