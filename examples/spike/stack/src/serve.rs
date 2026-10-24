//! Server feature: one Axum router, stateless rmcp, and one SQLite function.

use std::sync::Arc;

use axum::extract::State;
use axum::Json;
use rmcp::handler::server::router::tool::ToolRouter;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::transport::streamable_http_server::session::local::LocalSessionManager;
use rmcp::transport::streamable_http_server::{StreamableHttpServerConfig, StreamableHttpService};
use rmcp::{tool, tool_handler, tool_router, ServerHandler};
use sc_command::Envelope;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use sqlx::sqlite::{SqliteConnectOptions, SqlitePool, SqlitePoolOptions};
use utoipa::ToSchema;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

/// One note stored by the spike operation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema, ToSchema)]
pub struct Note {
    /// Stable key for the note.
    pub key: String,
    /// Note body stored in SQLite.
    pub body: String,
}

/// Stores `note` and reads the same row back.
///
/// This is the only function in the spike that contains SQL. The REST handler
/// and the MCP tool both call it.
///
/// # Errors
///
/// Returns the sqlx error when the statement fails.
pub async fn put_note(pool: &SqlitePool, note: Note) -> Result<Note, sqlx::Error> {
    sqlx::query("CREATE TABLE IF NOT EXISTS notes (key TEXT PRIMARY KEY, body TEXT NOT NULL)")
        .execute(pool)
        .await?;
    sqlx::query(
        "INSERT INTO notes (key, body) VALUES (?1, ?2)
         ON CONFLICT(key) DO UPDATE SET body = excluded.body",
    )
    .bind(&note.key)
    .bind(&note.body)
    .execute(pool)
    .await?;
    let (key, body) =
        sqlx::query_as::<_, (String, String)>("SELECT key, body FROM notes WHERE key = ?1")
            .bind(&note.key)
            .fetch_one(pool)
            .await?;
    Ok(Note { key, body })
}

/// Names the observability crates linked into the server feature.
///
/// The strings are the crate paths the spike resolves, not a version pin.
/// Versions come from the lockfile.
#[must_use]
pub fn observability_pin() -> String {
    let code = sc_observability_types::ErrorCode::new_static("SPIKE.PIN");
    format!(
        "{code} {}",
        sc_observability::constants::DEFAULT_LOG_FILE_SUFFIX
    )
}

/// Configuration loaded by `sc-config` before `Daemon::builder`.
#[derive(Debug, serde::Deserialize)]
pub struct SpikeConfig {
    /// Database file opened inside the stores closure.
    pub database: std::path::PathBuf,
    /// Daemon lock directory and endpoint settings.
    #[serde(flatten)]
    pub daemon: sc_runtime::DaemonConfig,
}

/// Opens the spike database. The application maps sqlx errors into `RuntimeError`.
///
/// # Errors
///
/// Returns [`sc_runtime::RuntimeError::StoresFailed`] when the file cannot be
/// created or the pool cannot open.
pub async fn stores(database: std::path::PathBuf) -> Result<SqlitePool, sc_runtime::RuntimeError> {
    let _pin = observability_pin();
    if let Some(parent) = database.parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent).map_err(|err| {
                sc_runtime::RuntimeError::stores_failed(
                    "SPIKE.STORE",
                    "the note database could not be created",
                    "check the database path",
                    Some(err.to_string()),
                )
            })?;
        }
    }
    SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(
            SqliteConnectOptions::new()
                .filename(database)
                .create_if_missing(true),
        )
        .await
        .map_err(|err| {
            sc_runtime::RuntimeError::stores_failed(
                "SPIKE.STORE",
                "the note database did not open",
                "check the database path",
                Some(err.to_string()),
            )
        })
}

/// Project router. OpenAPI, health, and `/mcp` are mounted by `sc-runtime`.
pub fn routes(pool: SqlitePool) -> OpenApiRouter {
    OpenApiRouter::new()
        .routes(routes!(put_note_route))
        .with_state(pool)
}

/// rmcp service mounted at `/mcp` as given.
pub fn mcp(pool: SqlitePool) -> StreamableHttpService<NoteServer, LocalSessionManager> {
    StreamableHttpService::new(
        move || Ok(NoteServer::new(pool.clone())),
        Arc::new(LocalSessionManager::default()),
        StreamableHttpServerConfig::default()
            .with_legacy_session_mode(false)
            .with_json_response(true),
    )
}

#[derive(Clone)]
pub struct NoteServer {
    pool: SqlitePool,
    tool_router: ToolRouter<Self>,
}

impl NoteServer {
    fn new(pool: SqlitePool) -> Self {
        Self {
            pool,
            tool_router: Self::tool_router(),
        }
    }
}

#[tool_router(router = tool_router)]
impl NoteServer {
    /// Stores one note.
    #[tool(name = "note_put", description = "Store one note.")]
    async fn note_put(&self, Parameters(note): Parameters<Note>) -> Result<String, String> {
        let saved = put_note(&self.pool, note)
            .await
            .map_err(|error| error.to_string())?;
        serde_json::to_string(&saved).map_err(|error| error.to_string())
    }
}

#[tool_handler(router = self.tool_router)]
impl ServerHandler for NoteServer {}

#[utoipa::path(
    post,
    path = "/notes",
    request_body = Note,
    responses((status = 200, description = "Stored note", body = Note))
)]
async fn put_note_route(State(pool): State<SqlitePool>, Json(note): Json<Note>) -> Envelope<Note> {
    match put_note(&pool, note).await {
        Ok(saved) => Envelope::success(saved),
        Err(error) => Envelope::failure(sc_command::OpError::new(
            sc_command::ErrorKind::Internal,
            sc_observability_types::ErrorCode::new_static("SPIKE.STORE"),
            error.to_string(),
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn put_note_is_stable_for_the_same_key() {
        let dir = tempfile::tempdir().expect("temp dir");
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect_with(
                SqliteConnectOptions::new()
                    .filename(dir.path().join("notes.db"))
                    .create_if_missing(true),
            )
            .await
            .expect("pool");
        let note = Note {
            key: "spike".to_owned(),
            body: "ok".to_owned(),
        };
        let saved = put_note(&pool, note.clone()).await.expect("put");
        assert_eq!(saved, note);
        pool.close().await;
    }
}
