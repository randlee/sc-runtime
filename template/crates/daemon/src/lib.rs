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

include!("open/{{db}}.rs");

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

/// Maps a service store-open error into [`sc_runtime::RuntimeError::StoresFailed`].
///
/// `service` does not depend on `sc-runtime`. The daemon stores closure is the
/// one place that performs this mapping.
#[must_use]
pub fn map_store_open(err: service::StoreOpenError) -> sc_runtime::RuntimeError {
    sc_runtime::RuntimeError::stores_failed(err.code, err.cause, err.recovery, Some(err.detail))
}
