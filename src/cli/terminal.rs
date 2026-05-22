use clap::Subcommand;

#[derive(Subcommand)]
pub enum TerminalCmd {
    /// Create an interactive PTY session
    Create {
        #[arg(long)]
        shell: Option<String>,
        #[arg(long)]
        cwd: Option<String>,
        #[arg(long)]
        rows: Option<u16>,
        #[arg(long)]
        cols: Option<u16>,
    },
    /// Write text to a terminal session
    Write { terminal_id: String, input: String },
    /// Read buffered terminal output
    Read {
        terminal_id: String,
        #[arg(long)]
        max_bytes: Option<u64>,
        #[arg(long, default_value_t = true)]
        flush: bool,
    },
    /// Resize a terminal session
    Resize {
        terminal_id: String,
        rows: u16,
        cols: u16,
    },
    /// List active terminal sessions
    List,
    /// Kill a terminal session
    Kill {
        terminal_id: String,
        #[arg(long)]
        signal: Option<String>,
    },
}
