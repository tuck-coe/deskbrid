use clap::Subcommand;

#[derive(Subcommand)]
pub enum FilesCmd {
    /// Search for files
    Search {
        pattern: String,
        #[arg(long)]
        root: Option<String>,
        #[arg(long, default_value = "50")]
        max_results: u32,
    },
    /// Watch a path for changes
    Watch { path: String },
    /// Stop watching a path
    Unwatch { path: String },
}
