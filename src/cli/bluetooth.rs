use clap::Subcommand;

#[derive(Subcommand)]
pub enum BluetoothCmd {
    /// List known devices
    List,
    /// Scan for devices
    Scan,
    /// Connect to device
    Connect { address: String },
    /// Disconnect device
    Disconnect { address: String },
}
