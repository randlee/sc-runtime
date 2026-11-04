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
{% if cron_commands %}
use rmcp::handler::server::tool::Extension;
use sc_cron_commands::{CronExecutor, CronRequest, CronResponse};
use sc_command::{Envelope, ErrorKind, OpError};
{% endif %}
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

{% if cron_commands %}
/// Builds the existing stateless MCP service with injected generic cron tools.
/// The application owns the actual executor and selected production registration.
pub fn service_with_cron_executor(stores: Stores, executor: Arc<dyn CronExecutor>)
    -> StreamableHttpService<WidgetServer, LocalSessionManager> {
    StreamableHttpService::new(
        move || Ok(WidgetServer::with_cron_executor(stores.clone(), executor.clone())),
        Arc::new(LocalSessionManager::default()),
        StreamableHttpServerConfig::default().with_legacy_session_mode(false).with_json_response(true),
    )
}
{% endif %}

/// Tool host holding the shared stores value.
#[derive(Clone)]
pub struct WidgetServer {
    stores: Stores,
    {% if cron_commands %}
    cron_executor: Option<Arc<dyn CronExecutor>>,
    {% endif %}
    tool_router: ToolRouter<Self>,
}

impl WidgetServer {
    fn new(stores: Stores) -> Self {
        // `#[tool_router]` records every `#[tool]` fn in that impl. It does not
        // honor `#[cfg]` on one fn, so delete tools are a second router. Create
        // and get are written once.
        #[cfg(feature = "general-db")]
        let tool_router = {
            let mut tool_router = Self::tool_router();
            tool_router.merge(Self::delete_tools());
            tool_router
        };
        #[cfg(not(feature = "general-db"))]
        let tool_router = Self::tool_router();
        Self {
            stores,
            {% if cron_commands %}
            cron_executor: None,
            {% endif %}
            tool_router,
        }
    }
}

{% if cron_commands %}
impl WidgetServer {
    /// Adds all three standard verbs using the same application-owned executor.
    pub fn with_cron_executor(stores: Stores, executor: Arc<dyn CronExecutor>) -> Self {
        let mut server = Self::new(stores);
        server.cron_executor = Some(executor);
        server.tool_router.merge(Self::cron_tools());
        server
    }

    async fn execute_cron(&self, verb: &str, request: CronRequest, parts: axum::http::request::Parts)
        -> Envelope<CronResponse> {
        let Some(deadline) = parts.extensions.get::<std::time::Instant>().copied() else {
            return Envelope::failure(OpError::from_code(ErrorKind::Unavailable,
                "CRON.COMMAND.UNAVAILABLE", "request deadline context is unavailable")
                .with_suggestion("mount the service on the sc-runtime request router"));
        };
        let Some(executor) = self.cron_executor.as_deref() else {
            return Envelope::failure(OpError::from_code(ErrorKind::Unavailable,
                "CRON.COMMAND.UNAVAILABLE", "application cron executor is unavailable")
                .with_suggestion("install the application-owned cron executor before registering commands"));
        };
        service::cron_commands::execute(executor, verb, request, deadline).await
    }
}

#[tool_router(router = cron_tools)]
impl WidgetServer {
    /// Starts an application-authorized Rust, Python or Monty schedule.
    #[tool(name = "cron.start", description = "Start an application-authorized schedule.")]
    async fn cron_start(&self, Parameters(request): Parameters<CronRequest>,
        Extension(parts): Extension<axum::http::request::Parts>) -> Result<rmcp::model::CallToolResult, sc_command::McpError> {
        self.execute_cron("cron.start", request, parts).await.into_result().into_mcp()
    }

    /// Ends the specified schedule generation.
    #[tool(name = "cron.end", description = "End the specified schedule generation.")]
    async fn cron_end(&self, Parameters(request): Parameters<CronRequest>,
        Extension(parts): Extension<axum::http::request::Parts>) -> Result<rmcp::model::CallToolResult, sc_command::McpError> {
        self.execute_cron("cron.end", request, parts).await.into_result().into_mcp()
    }

    /// Reads a bounded safe page of schedule views.
    #[tool(name = "cron.list", description = "Read a bounded page of schedule views.")]
    async fn cron_list(&self, Parameters(request): Parameters<CronRequest>,
        Extension(parts): Extension<axum::http::request::Parts>) -> Result<rmcp::model::CallToolResult, sc_command::McpError> {
        self.execute_cron("cron.list", request, parts).await.into_result().into_mcp()
    }
}
{% endif %}

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

/// Delete tools. Merged into the shared router only when `general-db` is on.
#[cfg(feature = "general-db")]
#[tool_router(router = delete_tools)]
impl WidgetServer {
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
