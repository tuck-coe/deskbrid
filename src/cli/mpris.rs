use clap::Subcommand;

#[derive(Subcommand)]
pub enum MprisCmd {
    /// List MPRIS media players
    List,
    /// Show one player, or the first active player
    Get { player: Option<String> },
    /// Send a playback command: play_pause, play, pause, stop, next, previous
    Control {
        action: String,
        #[arg(long)]
        player: Option<String>,
    },
}
