//! MCP adapters. Present only when the mcp option is on.
//!
//! `LocalSessionManager` is the manager type rmcp's stateless HTTP service
//! requires. It is not an application session store. Session mode is off and
//! responses are JSON, so a client can call a tool without initializing a
//! session.

use std::sync::Arc;

use rmcp::ServerHandler;
use rmcp::handler::server::router::tool::ToolRouter;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::transport::streamable_http_server::session::local::LocalSessionManager;
use rmcp::transport::streamable_http_server::{StreamableHttpServerConfig, StreamableHttpService};
use rmcp::{tool, tool_handler, tool_router};
use sc_command::IntoMcp;
use service::Stores;

/// Builds the stateless MCP service mounted at `/mcp`.
pub fn service(stores: Stores) -> StreamableHttpService<WidgetServer, LocalSessionManager> {
    StreamableHttpService::new(
        move || Ok(WidgetServer::new(stores.clone())),
        Arc::new(LocalSessionManager::default()),
        StreamableHttpServerConfig::default()
            .with_legacy_session_mode(false)
            .with_json_response(true),
    )
}

/// Tool host holding the shared stores value.
#[derive(Clone)]
pub struct WidgetServer {
    stores: Stores,
    tool_router: ToolRouter<Self>,
}

impl WidgetServer {
    fn new(stores: Stores) -> Self {
        Self {
            stores,
            tool_router: Self::tool_router(),
        }
    }
}

#[cfg(not(feature = "general-db"))]
#[tool_router(router = tool_router)]
impl WidgetServer {
    /// Creates one widget.
    #[tool(name = "widget_create", description = "Create one widget.")]
    async fn widget_create(
        &self,
        Parameters(body): Parameters<api_types::CreateWidget>,
    ) -> Result<rmcp::model::CallToolResult, sc_command::McpError> {
        service::create_widget(&self.stores, body).await.into_mcp()
    }

    /// Gets one widget by id.
    #[tool(name = "widget_get", description = "Get one widget by id.")]
    async fn widget_get(
        &self,
        Parameters(body): Parameters<api_types::GetWidget>,
    ) -> Result<rmcp::model::CallToolResult, sc_command::McpError> {
        service::get_widget(&self.stores, body).await.into_mcp()
    }
}

#[cfg(feature = "general-db")]
#[tool_router(router = tool_router)]
impl WidgetServer {
    /// Creates one widget.
    #[tool(name = "widget_create", description = "Create one widget.")]
    async fn widget_create(
        &self,
        Parameters(body): Parameters<api_types::CreateWidget>,
    ) -> Result<rmcp::model::CallToolResult, sc_command::McpError> {
        service::create_widget(&self.stores, body).await.into_mcp()
    }

    /// Gets one widget by id.
    #[tool(name = "widget_get", description = "Get one widget by id.")]
    async fn widget_get(
        &self,
        Parameters(body): Parameters<api_types::GetWidget>,
    ) -> Result<rmcp::model::CallToolResult, sc_command::McpError> {
        service::get_widget(&self.stores, body).await.into_mcp()
    }

    /// Deletes one widget. Same service call as `delete_row` on `widgets`.
    #[tool(name = "widget_delete", description = "Delete one widget.")]
    async fn widget_delete(
        &self,
        Parameters(body): Parameters<api_types::DeleteWidget>,
    ) -> Result<rmcp::model::CallToolResult, sc_command::McpError> {
        service::delete_widget(&self.stores, body).await.into_mcp()
    }

    /// Deletes one row.
    #[tool(name = "delete_row", description = "Delete one row.")]
    async fn delete_row(
        &self,
        Parameters(body): Parameters<api_types::DeleteRow>,
    ) -> Result<rmcp::model::CallToolResult, sc_command::McpError> {
        service::delete_row(&self.stores, &body.table, &body.id)
            .await
            .into_mcp()
    }
}

#[tool_handler(router = self.tool_router)]
impl ServerHandler for WidgetServer {}
