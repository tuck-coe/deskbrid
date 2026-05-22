//! MCP type definitions and transport layer.

use anyhow::Context;
use rmcp::{
    ErrorData,
    handler::server::ServerHandler,
    model::{
        CallToolRequestParam, CallToolResult, Content, Implementation, ListToolsResult,
        ServerCapabilities, ServerInfo, Tool,
    },
    role::RoleServer,
    service::{RequestContext, RoleServer as RoleServerTrait},
    transport::stdio,
};
use std::sync::Arc;

use crate::DaemonState;

pub struct DeskbridMcp {
    pub state: Arc<DaemonState>,
}

impl DeskbridMcp {
    pub fn new(state: Arc<DaemonState>) -> Self {
        Self { state }
    }
}

pub async fn serve_stdio(state: Arc<DaemonState>) -> anyhow::Result<()> {
    let server = DeskbridMcp::new(state);
    let transport = stdio::stdio_server()
        .await
        .context("failed to start MCP stdio server")?;
    transport.serve(server).await.context("MCP server error")?;
    Ok(())
}

#[rmcp::tool_router]
impl DeskbridMcp {
    #[rmcp(tool)]
    async fn list_windows(
        &self,
        _ctx: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, ErrorData> {
        let backend = self.state.backend.read().await;
        let backend = backend
            .as_ref()
            .ok_or_else(|| ErrorData::new(500, "no backend".into()))?;

        let output = crate::daemon::execute::execute_public(
            "windows.list",
            "",
            backend.as_ref(),
            &self.state,
        )
        .await
        .map_err(|e| ErrorData::new(500, e.to_string()))?;

        Ok(CallToolResult::success(vec![Content::text(
            output.to_string(),
        )]))
    }

    #[rmcp(tool)]
    async fn focus_window(
        &self,
        ctx: RequestContext<RoleServer>,
        window_id: String,
    ) -> Result<CallToolResult, ErrorData> {
        let _ = ctx;
        let output = tools::do_focus_window(&self.state, &window_id)
            .await
            .map_err(|e| ErrorData::new(500, e.to_string()))?;
        Ok(CallToolResult::success(vec![Content::text(
            output.to_string(),
        )]))
    }

    #[rmcp(tool)]
    async fn type_text(
        &self,
        ctx: RequestContext<RoleServer>,
        text: String,
    ) -> Result<CallToolResult, ErrorData> {
        let _ = ctx;
        let output = tools::do_type_text(&self.state, &text)
            .await
            .map_err(|e| ErrorData::new(500, e.to_string()))?;
        Ok(CallToolResult::success(vec![Content::text(
            output.to_string(),
        )]))
    }

    #[rmcp(tool)]
    async fn press_keys(
        &self,
        ctx: RequestContext<RoleServer>,
        keys: Vec<String>,
    ) -> Result<CallToolResult, ErrorData> {
        let _ = ctx;
        let output = tools::do_press_keys(&self.state, &keys)
            .await
            .map_err(|e| ErrorData::new(500, e.to_string()))?;
        Ok(CallToolResult::success(vec![Content::text(
            output.to_string(),
        )]))
    }

    #[rmcp(tool)]
    async fn mouse_move(
        &self,
        ctx: RequestContext<RoleServer>,
        x: f64,
        y: f64,
    ) -> Result<CallToolResult, ErrorData> {
        let _ = ctx;
        let output = tools::do_mouse_move(&self.state, x, y)
            .await
            .map_err(|e| ErrorData::new(500, e.to_string()))?;
        Ok(CallToolResult::success(vec![Content::text(
            output.to_string(),
        )]))
    }

    #[rmcp(tool)]
    async fn mouse_click(
        &self,
        ctx: RequestContext<RoleServer>,
        button: String,
    ) -> Result<CallToolResult, ErrorData> {
        let _ = ctx;
        let output = tools::do_mouse_click(&self.state, &button)
            .await
            .map_err(|e| ErrorData::new(500, e.to_string()))?;
        Ok(CallToolResult::success(vec![Content::text(
            output.to_string(),
        )]))
    }

    #[rmcp(tool)]
    async fn screenshot(
        &self,
        ctx: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, ErrorData> {
        let _ = ctx;
        let output = tools::do_screenshot(&self.state)
            .await
            .map_err(|e| ErrorData::new(500, e.to_string()))?;
        Ok(CallToolResult::success(vec![Content::text(
            output.to_string(),
        )]))
    }

    #[rmcp(tool)]
    async fn clipboard_read(
        &self,
        ctx: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, ErrorData> {
        let _ = ctx;
        let output = tools::do_clipboard_read(&self.state)
            .await
            .map_err(|e| ErrorData::new(500, e.to_string()))?;
        Ok(CallToolResult::success(vec![Content::text(
            output.to_string(),
        )]))
    }

    #[rmcp(tool)]
    async fn clipboard_write(
        &self,
        ctx: RequestContext<RoleServer>,
        text: String,
    ) -> Result<CallToolResult, ErrorData> {
        let _ = ctx;
        let output = tools::do_clipboard_write(&self.state, &text)
            .await
            .map_err(|e| ErrorData::new(500, e.to_string()))?;
        Ok(CallToolResult::success(vec![Content::text(
            output.to_string(),
        )]))
    }

    // --- AT-SPI tools ---

    #[rmcp(tool)]
    async fn list_apps(
        &self,
        ctx: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, ErrorData> {
        let _ = ctx;
        let output = tools::do_list_apps(&self.state)
            .await
            .map_err(|e| ErrorData::new(500, e.to_string()))?;
        Ok(CallToolResult::success(vec![Content::text(
            output.to_string(),
        )]))
    }

    #[rmcp(tool)]
    async fn get_accessibility_tree(
        &self,
        ctx: RequestContext<RoleServer>,
        app_name: Option<String>,
        pid: Option<u32>,
        max_nodes: Option<usize>,
        max_depth: Option<u32>,
    ) -> Result<CallToolResult, ErrorData> {
        let _ = ctx;
        let output = tools::do_get_accessibility_tree(
            &self.state,
            app_name.as_deref(),
            pid,
            max_nodes,
            max_depth,
        )
        .await
        .map_err(|e| ErrorData::new(500, e.to_string()))?;
        Ok(CallToolResult::success(vec![Content::text(
            output.to_string(),
        )]))
    }

    #[rmcp(tool)]
    async fn perform_action(
        &self,
        ctx: RequestContext<RoleServer>,
        object_ref: String,
        action_name: Option<String>,
    ) -> Result<CallToolResult, ErrorData> {
        let _ = ctx;
        let output = tools::do_perform_action(&self.state, &object_ref, action_name.as_deref())
            .await
            .map_err(|e| ErrorData::new(500, e.to_string()))?;
        Ok(CallToolResult::success(vec![Content::text(
            output.to_string(),
        )]))
    }

    #[rmcp(tool)]
    async fn set_element_value(
        &self,
        ctx: RequestContext<RoleServer>,
        object_ref: String,
        value: String,
    ) -> Result<CallToolResult, ErrorData> {
        let _ = ctx;
        let output = tools::do_set_element_value(&self.state, &object_ref, &value)
            .await
            .map_err(|e| ErrorData::new(500, e.to_string()))?;
        Ok(CallToolResult::success(vec![Content::text(
            output.to_string(),
        )]))
    }

    #[rmcp(tool)]
    async fn get_element_text(
        &self,
        ctx: RequestContext<RoleServer>,
        object_ref: String,
        max_chars: Option<i32>,
    ) -> Result<CallToolResult, ErrorData> {
        let _ = ctx;
        let output = tools::do_get_element_text(&self.state, &object_ref, max_chars)
            .await
            .map_err(|e| ErrorData::new(500, e.to_string()))?;
        Ok(CallToolResult::success(vec![Content::text(
            output.to_string(),
        )]))
    }

    #[rmcp(tool)]
    async fn click_element(
        &self,
        ctx: RequestContext<RoleServer>,
        object_ref: String,
    ) -> Result<CallToolResult, ErrorData> {
        let _ = ctx;
        let output = tools::do_click_element(&self.state, &object_ref)
            .await
            .map_err(|e| ErrorData::new(500, e.to_string()))?;
        Ok(CallToolResult::success(vec![Content::text(
            output.to_string(),
        )]))
    }

    #[rmcp(tool)]
    async fn doctor(&self, ctx: RequestContext<RoleServer>) -> Result<CallToolResult, ErrorData> {
        let _ = ctx;
        let output = tools::do_doctor(&self.state)
            .await
            .map_err(|e| ErrorData::new(500, e.to_string()))?;
        Ok(CallToolResult::success(vec![Content::text(
            output.to_string(),
        )]))
    }

    #[rmcp(tool)]
    async fn setup_accessibility(
        &self,
        ctx: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, ErrorData> {
        let _ = ctx;
        let output = tools::do_setup_accessibility(&self.state)
            .await
            .map_err(|e| ErrorData::new(500, e.to_string()))?;
        Ok(CallToolResult::success(vec![Content::text(
            output.to_string(),
        )]))
    }

    #[rmcp(tool)]
    async fn capabilities(
        &self,
        ctx: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, ErrorData> {
        let _ = ctx;
        let output = tools::do_capabilities(&self.state)
            .await
            .map_err(|e| ErrorData::new(500, e.to_string()))?;
        Ok(CallToolResult::success(vec![Content::text(
            output.to_string(),
        )]))
    }
}

#[rmcp::tool_router]
impl ServerHandler for DeskbridMcp {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: Default::default(),
            capabilities: ServerCapabilities::builder()
                .enable_tools()
                .build(),
            server_info: Implementation {
                name: "deskbrid".into(),
                version: env!("CARGO_PKG_VERSION").into(),
            },
            instructions: Some("Deskbrid MCP server — Linux desktop automation for AI agents. Control windows, inject keystrokes, take screenshots, read accessibility trees.".into()),
        }
    }
}
