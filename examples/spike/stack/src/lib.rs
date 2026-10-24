//! Disposable library for the Phase A compatibility spike.
//!
//! The default build is the CLI side of the feature split. The `server`
//! feature is the only build that links Axum, rmcp, and sqlx.

/// Marker linked by the spike CLI on the default feature set.
#[must_use]
pub fn client_marker() -> &'static str {
    "spike-stack-default"
}

#[cfg(feature = "server")]
mod serve;

#[cfg(feature = "server")]
pub use serve::{observability_pin, put_note, serve, Note};
