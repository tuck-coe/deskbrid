//! Configuration for the deskbrid daemon.

use std::path::PathBuf;

/// Daemon configuration.
#[derive(Debug, Clone)]
pub struct Config {
    /// Socket path override (defaults to $XDG_RUNTIME_DIR/deskbrid/socket)
    pub socket_path: Option<PathBuf>,

    /// Log level (trace, debug, info, warn, error)
    pub log_level: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            socket_path: None,
            log_level: "info".to_string(),
        }
    }
}

impl Config {
    /// Load from CLI args / env / config file.
    pub fn from_env() -> Self {
        let mut config = Self::default();

        if let Ok(level) = std::env::var("DESKBRID_LOG") {
            config.log_level = level;
        }

        config
    }
}
