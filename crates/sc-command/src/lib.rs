//! Lossless sc-ai-cli response envelope and the two server conversions.
//!
//! Default features export [`Envelope`], [`OpError`], and [`ErrorKind`].
//! Feature `server` adds the HTTP response conversion and the sealed MCP
//! conversion. That feature is off by default.
//!
//! The envelope contract version for values this crate creates is
//! [`ENVELOPE_VERSION`] (`"1"`). A value read from JSON keeps the `version`
//! string it arrived with.

#![doc = include_str!("../README.md")]

mod envelope;
mod error;

#[cfg(feature = "server")]
mod server;

#[doc(inline)]
pub use envelope::Envelope;
#[doc(inline)]
pub use error::{ErrorKind, OpError};

#[cfg(feature = "server")]
#[doc(inline)]
pub use server::{IntoMcp, McpError};

/// Envelope contract version written by constructors in this crate.
///
/// Deserialization keeps the version string from the input instead of
/// replacing it with this constant.
pub const ENVELOPE_VERSION: &str = "1";

/// Stable code for an envelope whose `ok` flag disagrees with `data` or
/// `error`, or whose object is missing a required key.
pub const DISCRIMINANT_CODE: &str = "ENVELOPE.DISCRIMINANT";

/// Stable code for a payload whose `Serialize` implementation fails.
pub const SERIALIZATION_CODE: &str = "ENVELOPE.SERIALIZATION";
