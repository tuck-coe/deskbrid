use clap::Subcommand;

#[derive(Subcommand)]
pub enum NotifyCmd {
    /// Send a notification
    Send {
        #[arg(long)]
        title: String,
        #[arg(long)]
        body: String,
        #[arg(long, default_value = "normal")]
        urgency: String,
    },
    /// Close a notification
    Close { notification_id: u32 },
}
