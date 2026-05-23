//! MCP server — rmcp-based server replacing the bare JSON-RPC handler.
//! Uses rmcp 1.7.0 `#[tool_router(server_handler)]` pattern.

use crate::DaemonState;
use rmcp::{
    handler::server::wrapper::{Json, Parameters},
    schemars, tool, tool_router,
};
use serde::Deserialize;
use serde_json::Value;
use std::sync::Arc;
use tokio::runtime::Handle;

#[derive(Clone)]
pub struct McpServer {
    state: Arc<DaemonState>,
    rt: Handle,
}

impl McpServer {
    pub fn new(state: Arc<DaemonState>) -> Self {
        Self {
            state,
            rt: Handle::current(),
        }
    }

    fn windows_list(&self) -> Result<Value, String> {
        let backend = self.state.backend.clone();
        self.rt.block_on(async {
            let be = backend.read().await;
            let be = be.as_ref().ok_or("no backend")?;
            let wins = be.windows_list().await.map_err(|e| e.to_string())?;
            Ok(serde_json::to_value(&wins).unwrap_or_default())
        })
    }
}

// ── Tool input/output types ────────────────────────────

#[derive(Deserialize, schemars::JsonSchema, Default)]
struct WindowIdParam {
    #[schemars(description = "Window ID from list_windows")]
    window_id: String,
}

// ─── Tool implementations ───────────────────────────────

#[tool_router(server_handler)]
impl McpServer {
    #[tool(
        name = "list_windows",
        description = "List all open windows on the Linux desktop with IDs, titles, classes, workspace, and geometry."
    )]
    fn list_windows(&self) -> Json<Value> {
        Json(self.windows_list().unwrap_or_default())
    }

    #[tool(
        name = "focus_window",
        description = "Focus (activate) a window by its ID."
    )]
    fn focus_window(
        &self,
        Parameters(WindowIdParam { window_id }): Parameters<WindowIdParam>,
    ) -> Json<Value> {
        let rt = self.rt.clone();
        let backend = self.state.backend.clone();
        let result: Value = rt.block_on(async {
            let be = backend.read().await;
            match be.as_ref() {
                Some(be) => {
                    be.window_focus(&window_id).await.ok();
                    Value::Null
                }
                None => serde_json::json!({"error": "no backend"}),
            }
        });
        Json(result)
    }

    #[tool(name = "close_window", description = "Close a window by its ID.")]
    fn close_window(
        &self,
        Parameters(WindowIdParam { window_id }): Parameters<WindowIdParam>,
    ) -> Json<Value> {
        let rt = self.rt.clone();
        let backend = self.state.backend.clone();
        let result: Value = rt.block_on(async {
            let be = backend.read().await;
            match be.as_ref() {
                Some(be) => {
                    be.window_close(&window_id).await.ok();
                    Value::Null
                }
                None => serde_json::json!({"error": "no backend"}),
            }
        });
        Json(result)
    }

    #[tool(
        name = "system_info",
        description = "Get system information — hostname, OS, kernel, uptime, memory, CPU."
    )]
    fn system_info(&self) -> Json<Value> {
        let rt = self.rt.clone();
        let backend = self.state.backend.clone();
        let result: Value = rt.block_on(async {
            let be = backend.read().await;
            match be.as_ref() {
                Some(be) => match be.system_info().await {
                    Ok(info) => serde_json::to_value(&info).unwrap_or_default(),
                    Err(e) => serde_json::json!({"error": e.to_string()}),
                },
                None => serde_json::json!({"error": "no backend"}),
            }
        });
        Json(result)
    }
}

/// Run the MCP server over stdio transport.
pub async fn run_mcp(state: Arc<DaemonState>) -> anyhow::Result<()> {
    use rmcp::{service::serve_server, transport::stdio};

    let server = McpServer::new(state);
    serve_server(server, stdio()).await?.waiting().await?;
    Ok(())
}
