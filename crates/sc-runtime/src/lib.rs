//! Assembles one daemon: instance lock, project stores, one router, one listener.
//!
//! The application loads configuration before [`Daemon::builder`]. This crate
//! does not read a config file and does not depend on `sc-config`.
//! [`Daemon::builder`] borrows [`sc_transport::AppName`]. There is no
//! `sc_runtime::AppName`.
//!
//! `run` exists only after `stores` and `routes`. `mcp` is optional on that
//! ready state. Endpoint text is resolved before the lock so an invalid
//! endpoint does not open stores. Signal listeners are registered, and on
//! Windows Ctrl-C is polled, before the lock and before bind. The order after
//! a valid endpoint is signals, lock, stores, routes and optional MCP, then
//! bind. The assembled router limits handler time. Shutdown drains in-flight
//! requests up to a bound.
//!
//! # Examples
//!
//! ```no_run
//! use sc_runtime::{Daemon, DaemonConfig};
//! use sc_transport::AppName;
//! use utoipa_axum::router::OpenApiRouter;
//!
//! # #[tokio::main(flavor = "current_thread")]
//! # async fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let app = AppName::new("my-app")?;
//! let config = DaemonConfig::default();
//! Daemon::builder(&app, &config)
//!     .stores(|| async { Ok(()) })
//!     .routes(|()| OpenApiRouter::new())
//!     .run()
//!     .await?;
//! # Ok(())
//! # }
//! ```

#![deny(unsafe_code)]

mod builder;
mod config;
mod error;
mod lock;
mod router;
mod shutdown;

#[doc(inline)]
pub use builder::{Daemon, DaemonBuilder};
#[doc(inline)]
pub use config::DaemonConfig;
#[doc(inline)]
pub use error::RuntimeError;

#[cfg(feature = "test-util")]
pub mod testing;

#[doc(hidden)]
pub use builder::{NeedRoutes, NeedStores, Ready};

#[cfg(all(test, feature = "test-util"))]
mod tests;
