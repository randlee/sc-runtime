//! Temporary development-only injection contract over unchanged registry value types.
//!
//! This facade supplies no production Logger adapter, queue, sink, or lifecycle.
//! Remove its local path override before release and verify the matching upstream pair.

#![forbid(unsafe_code)]

// Preserve all released error and value APIs, including feature-gated v1 paths.
// This deliberate facade re-export preserves the original package's type identity.
pub use released_types::*;

use std::sync::Arc;

/// A trusted synchronous canonical event receiver.
///
/// Implementations must return promptly, perform no sink I/O or capacity wait,
/// and never panic. Libraries treat rejection as best effort.
pub trait CanonicalLog: Send + Sync {
    /// Returns the receiver's service identity.
    fn service_name(&self) -> &ServiceName;

    /// Attempts immediate event admission.
    ///
    /// # Errors
    /// Returns an event error when the receiver rejects the event.
    fn emit(&self, event: LogEvent) -> Result<(), v2::EventError>;
}

/// Shared ownership of an object-safe canonical event receiver.
pub type SharedCanonicalLog = Arc<dyn CanonicalLog>;

/// A receiver that retains a service name and discards all events.
#[derive(Clone, Debug)]
pub struct NoopCanonicalLog {
    service: ServiceName,
}

impl NoopCanonicalLog {
    /// Creates a discarding receiver with the supplied validated identity.
    #[must_use]
    pub fn new(service_name: ServiceName) -> Self {
        Self {
            service: service_name,
        }
    }

    /// Uses a valid static service name, falling back to `noop` for invalid names.
    #[must_use]
    pub fn named_or_default(service: &'static str) -> Self {
        let service = ServiceName::new(service).unwrap_or_else(|_| {
            // The exact pinned 1.5.0 validator accepts this fixed ASCII literal.
            // Consumer input never reaches an unwrap or expect.
            ServiceName::new("noop")
                .expect("fixed noop identity is valid in pinned sc-observability-types 1.5.0")
        });
        Self::new(service)
    }
}

impl CanonicalLog for NoopCanonicalLog {
    fn service_name(&self) -> &ServiceName {
        &self.service
    }

    fn emit(&self, _event: LogEvent) -> Result<(), v2::EventError> {
        Ok(())
    }
}

#[cfg(feature = "test-util")]
mod test_util;

#[cfg(feature = "test-util")]
pub use test_util::{RecordingCanonicalLog, RejectingCanonicalLog};

#[cfg(test)]
mod tests;
