//! HTTP adapters for the widget service.

{% if mcp %}pub mod mcp;{% endif %}
pub mod routes;

use std::path::PathBuf;

use sc_runtime::DaemonConfig;
use sc_transport::AppName;
use serde::Deserialize;

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

/// Resolves the database path from configuration or the transport instance root.
///
/// This does not invent an endpoint or a TCP port.
///
/// # Errors
///
/// Returns the transport error text when the instance root cannot be resolved.
pub fn database_path(config: &AppConfig, app: &AppName) -> Result<PathBuf, String> {
    if config.database.is_absolute() {
        return Ok(config.database.clone());
    }
    let root = if let Some(root) = &config.daemon.transport.instance_root {
        root.clone()
    } else {
        sc_transport::instance_root(app).map_err(|err| err.to_string())?
    };
    Ok(root.join(&config.database))
}
