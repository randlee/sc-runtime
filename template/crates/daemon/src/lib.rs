//! HTTP adapters for the widget service.
//!
//! `/health` is mounted by sc-runtime and reports process liveness. It does
//! not check the widget database. This template has no store-backed readiness
//! route.

{% if mcp %}
pub mod mcp;
{% endif %}
pub mod routes;

use std::path::PathBuf;

use sc_runtime::DaemonConfig;
use sc_transport::{AppName, TransportError};
use serde::Deserialize;

/// Generated application name. cargo-generate fills this from `--name`.
pub const APP_NAME: &str = "{{project-name}}";

/// Application configuration loaded before [`sc_runtime::Daemon::builder`].
#[derive(Debug, Deserialize)]
pub struct AppConfig {
    /// SQLite file. A relative path is joined to the instance root.
    pub database: PathBuf,
    /// Plain logging values. Example choice: console on, file off.
    #[serde(default)]
    pub logging: sc_observability::LogSettings,
    /// Endpoint fields passed through to the daemon builder.
    #[serde(flatten)]
    pub daemon: DaemonConfig,
}

/// Resolves the instance root from configuration or the transport default.
///
/// This does not invent an endpoint or a TCP port.
///
/// # Errors
///
/// Returns the transport error when the instance root cannot be resolved.
pub fn instance_root(config: &AppConfig, app: &AppName) -> Result<PathBuf, TransportError> {
    if let Some(root) = &config.daemon.transport.instance_root {
        return Ok(root.clone());
    }
    sc_transport::instance_root(app)
}

/// Resolves the database path from configuration or the transport instance root.
///
/// This does not invent an endpoint or a TCP port.
///
/// # Errors
///
/// Returns the transport error when the instance root cannot be resolved.
pub fn database_path(config: &AppConfig, app: &AppName) -> Result<PathBuf, TransportError> {
    if config.database.is_absolute() {
        return Ok(config.database.clone());
    }
    Ok(instance_root(config, app)?.join(&config.database))
}

/// Maps a service store-open error into [`sc_runtime::RuntimeError::StoresFailed`].
///
/// `service` does not depend on `sc-runtime`. The daemon stores closure is the
/// one place that performs this mapping.
#[must_use]
pub fn map_store_open(err: service::StoreOpenError) -> sc_runtime::RuntimeError {
    sc_runtime::RuntimeError::stores_failed(err.code, err.cause, err.recovery, Some(err.detail))
}
