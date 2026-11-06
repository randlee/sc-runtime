//! Canonical logging contracts for application-owned receivers.
//!
//! These are the same nominal types accepted by the existing logger-aware
//! constructors. Receivers own admission and sinks; scheduler emission remains
//! best effort. No logging runtime is installed by this module.

pub use sc_command::observability::{CanonicalLog, NoopCanonicalLog, SharedCanonicalLog};
pub use sc_observability_types as types;
pub use sc_observability_types::{LogEvent, ServiceName, v2::EventError};
