use clap::Subcommand;

#[derive(Subcommand)]
pub enum ClipboardCmd {
    /// Read clipboard contents
    Read,
    /// Write to clipboard
    Write { text: String },
    /// List clipboard entries observed through Deskbrid
    History {
        #[arg(long)]
        limit: Option<usize>,
        #[arg(long)]
        query: Option<String>,
    },
    /// Clear Deskbrid clipboard history
    ClearHistory,
}
