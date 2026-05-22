use clap::Subcommand;

#[derive(Subcommand)]
pub enum NetworkCmd {
    /// Connection status
    Status,
    /// List interfaces
    Interfaces,
}

#[derive(Subcommand)]
pub enum WifiCmd {
    /// Scan for networks
    Scan,
    /// Connect to a network
    Connect { ssid: String },
}
