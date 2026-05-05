//! deskbrid daemon entry point.

use anyhow::Result;
use tracing::{info, warn};

#[tokio::main]
async fn main() -> Result<()> {
    let config = deskbrid::config::Config::from_env();
    let socket_path = config
        .socket_path
        .clone()
        .unwrap_or_else(deskbrid::default_socket_path);

    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(&config.log_level)
        .init();

    tracing::info!("Starting deskbrid v{}", deskbrid::VERSION);

    let (shutdown_tx, shutdown_rx) = tokio::sync::watch::channel(false);
    let mut daemon = tokio::spawn(deskbrid::run(config, shutdown_rx));

    tokio::select! {
        result = &mut daemon => {
            match result {
                Ok(result) => result?,
                Err(error) => return Err(error.into()),
            }
        }
        signal = tokio::signal::ctrl_c() => {
            signal?;
            info!("shutting down");
            if shutdown_tx.send(true).is_err() {
                warn!("shutdown receiver dropped before signal delivery");
            }

            match daemon.await {
                Ok(result) => result?,
                Err(error) => return Err(error.into()),
            }
        }
    }

    match tokio::fs::remove_file(&socket_path).await {
        Ok(()) => {}
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => warn!("failed to remove socket {}: {error}", socket_path.display()),
    }

    Ok(())
}
