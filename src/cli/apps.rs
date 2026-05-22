use clap::Subcommand;

#[derive(Subcommand)]
pub enum AppCmd {
    /// List installed launchable applications
    List {
        #[arg(long = "category")]
        categories: Vec<String>,
        #[arg(long = "mime-type")]
        mime_types: Vec<String>,
        #[arg(long)]
        include_hidden: bool,
        #[arg(long)]
        limit: Option<usize>,
    },
    /// Search installed applications
    Search {
        query: String,
        #[arg(long)]
        limit: Option<usize>,
    },
    /// Show one application by desktop ID
    Get { app_id: String },
}
