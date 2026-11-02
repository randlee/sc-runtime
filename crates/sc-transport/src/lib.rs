//! Local HTTP between a daemon and its CLI.
//!
//! The default build resolves an [`Endpoint`], computes an instance root, and
//! sends JSON `GET` and `POST` requests with [`Client`]. The `server` feature
//! adds [`bind`], which returns a [`Listener`] that `axum::serve` accepts.
//!
//! This crate reports transport facts. It does not start a daemon, retry for
//! readiness, or know application envelopes.
//!
//! # Examples
//!
//! ```
//! use sc_transport::{resolve_endpoint, AppName, Endpoint, TransportConfig};
//!
//! # fn main() -> Result<(), sc_transport::TransportError> {
//! let app = AppName::new("my-app")?;
//! let mut config = TransportConfig::default();
//! config.instance_root = Some(std::env::temp_dir());
//! let endpoint = resolve_endpoint(
//!     &app,
//!     Some(std::ffi::OsStr::new("tcp:127.0.0.1:9")),
//!     None,
//!     &config,
//! )?;
//! assert_eq!(endpoint, "tcp:127.0.0.1:9".parse::<Endpoint>()?);
//! # Ok(())
//! # }
//! ```

#![deny(unsafe_code)]

mod app_name;
mod client;
mod config;
mod endpoint;
mod error;
mod resolve;

#[cfg(feature = "server")]
mod server;

#[doc(inline)]
pub use app_name::AppName;
#[doc(inline)]
pub use client::Client;
#[doc(inline)]
pub use config::TransportConfig;
#[doc(inline)]
pub use endpoint::Endpoint;
#[doc(inline)]
pub use error::{
    CODE_BIND, CODE_DAEMON_NOT_RUNNING, CODE_ENDPOINT_BUSY, CODE_INSTANCE_ROOT,
    CODE_INVALID_ENDPOINT, CODE_INVALID_NAME, CODE_MISSING_TCP_PORT, CODE_NOT_A_SOCKET,
    CODE_REQUEST, CODE_RESPONSE_DECODE, CODE_TIMED_OUT, CODE_UNSUPPORTED_ENDPOINT, TransportCause,
    TransportError,
};
#[doc(inline)]
pub use resolve::{instance_root, resolve_endpoint, resolve_endpoint_from_env};

#[cfg(feature = "server")]
#[doc(inline)]
pub use server::{Listener, bind};
#[cfg(feature = "server")]
#[doc(hidden)]
pub use server::{ListenerAddr, ListenerIo};
