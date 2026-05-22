use clap::Subcommand;

#[derive(Subcommand)]
pub enum AuditCmd {
    /// Show recent action audit entries
    Log {
        #[arg(long)]
        limit: Option<usize>,
        #[arg(long)]
        action_type: Option<String>,
        #[arg(long)]
        status: Option<String>,
    },
    /// Clear in-memory audit entries
    Clear,
}
