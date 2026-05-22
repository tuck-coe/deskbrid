//! MCP (Model Context Protocol) server mode for Deskbrid.
//!
//! Exposes Deskbrid's desktop automation tools to MCP-compatible AI clients
//! (Claude Code, Cursor, etc.) via stdio transport.

pub mod tools;
pub mod types;

use anyhow::Context;
use std::sync::Arc;

use crate::DaemonState;

/// Run the MCP stdio server. Called from `deskbrid mcp`.
pub async fn run_mcp_server() -> anyhow::Result<()> {
    let event_tx = tokio::sync::broadcast::channel(256).0;
    let state = Arc::new(DaemonState::new());

    // Load the desktop backend
    let backend = crate::backend::detect_backend()
        .await
        .context("no desktop backend detected — are you running in a supported session?")?;
    *state.backend.write().await = Some(backend);

    tracing::info!("Deskbrid MCP server starting on stdio");
    types::serve_stdio(state).await
}

/// Run the MCP server on a TCP port (for `deskbrid daemon --mcp-port <PORT>`).
pub async fn run_mcp_tcp(port: u16) -> anyhow::Result<()> {
    use tokio::net::TcpListener;
    let event_tx = tokio::sync::broadcast::channel(256).0;
    let state = std::sync::Arc::new(crate::DaemonState::new());

    let backend = crate::backend::detect_backend()
        .await
        .context("no desktop backend detected")?;
    *state.backend.write().await = Some(backend);

    let addr = format!("127.0.0.1:{port}");
    let listener = TcpListener::bind(&addr)
        .await
        .context(format!("failed to bind MCP TCP listener on {addr}"))?;
    tracing::info!("Deskbrid MCP TCP server listening on {addr}");

    loop {
        let (stream, peer) = listener.accept().await?;
        let state = state.clone();
        tokio::spawn(async move {
            let transport = rmcp::transport::tokio::TokioStream::new(stream);
            if let Err(e) = types::serve_on_transport(state, transport).await {
                tracing::error!("MCP connection error from {peer}: {e}");
            }
        });
    }
}
