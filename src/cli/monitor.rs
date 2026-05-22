use clap::Subcommand;

#[derive(Subcommand)]
pub enum MonitorCmd {
    /// List monitors and outputs
    List,
    /// Set the primary monitor/output
    Primary { output: String },
    /// Set output resolution
    Resolution {
        output: String,
        width: u32,
        height: u32,
        #[arg(long)]
        refresh: Option<f64>,
    },
    /// Set output scale
    Scale { output: String, scale: f64 },
    /// Set output rotation: normal, left, right, inverted
    Rotate { output: String, rotation: String },
    /// Enable an output
    Enable { output: String },
    /// Disable an output
    Disable { output: String },
}
