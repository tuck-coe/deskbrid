//! deskbrid daemon entry point.

use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    let config = deskbrid::config::Config::from_env();

    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(&config.log_level)
        .init();

    tracing::info!("Starting deskbrid v{}", deskbrid::VERSION);

    deskbrid::run(config).await
}
