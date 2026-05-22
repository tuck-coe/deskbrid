use clap::Subcommand;

#[derive(Subcommand)]
pub enum AudioCmd {
    /// List audio sinks
    Sinks,
    /// Set sink volume
    Volume { sink_id: u32, volume: f64 },
}
