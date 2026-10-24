//! Disposable library for the Phase A compatibility spike.
//!
//! The default build is the CLI side of the feature split. The `server`
//! feature is the only build that links Axum, rmcp, and sqlx. That feature
//! binds a Unix domain socket and compiles only on Unix targets.

/// Marker linked by the spike CLI on the default feature set.
#[must_use]
pub fn client_marker() -> &'static str {
    "spike-stack-default"
}

#[cfg(all(feature = "server", not(unix)))]
compile_error!(
    "spike-stack feature `server` binds tokio::net::UnixListener and only builds on Unix targets"
);

#[cfg(all(feature = "server", unix))]
mod serve;

#[cfg(all(feature = "server", unix))]
pub use serve::{observability_pin, put_note, serve, Note};
