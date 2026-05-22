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
