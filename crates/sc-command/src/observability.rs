//! Canonical event injection without linking a logging runtime.
//!
//! Applications may implement this open receiver port and own their sinks and
//! lifecycle. This temporary 1.5.0 contract is removed after upstream capability
//! proof: <https://github.com/randlee/sc-runtime/issues/97> (ADR-RUN-0017).

use sc_observability_types::{LogEvent, ServiceName, v2};

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
    ///
    /// The fixed fallback is validated by the exact types 1.5.0 dependency.
    /// ADR-RUN-0017 permits only this literal invariant; caller input is never
    /// unwrapped. Upstream capability cleanup removes this temporary exception.
    #[must_use]
    pub fn named_or_default(service: &'static str) -> Self {
        let service = ServiceName::new(service).unwrap_or_else(|_| {
            // ADR-RUN-0017 fixed-literal invariant, bounded by exact types =1.5.0.
            // Never unwrap caller/environment data. Remove with upstream cleanup:
            // https://github.com/randlee/sc-runtime/issues/97
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
