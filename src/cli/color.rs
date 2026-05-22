use clap::Subcommand;

#[derive(Subcommand)]
pub enum ColorCmd {
    /// Pick a pixel color from the screen or an image path
    Pick {
        x: u32,
        y: u32,
        #[arg(long)]
        path: Option<String>,
    },
}
