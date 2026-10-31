//! Deterministic test receivers; production builds leave this module disabled.

use std::sync::Mutex;
use std::sync::atomic::{AtomicU64, Ordering};

use crate::{CanonicalLog, ErrorContext, LogEvent, Remediation, ServiceName, error_codes, v2};

/// An ordered event buffer shared safely through an `Arc`.
#[derive(Debug)]
pub struct RecordingCanonicalLog {
    service: ServiceName,
    // emit takes &self and must serialize writes for ordered snapshot semantics.
    events: Mutex<Vec<LogEvent>>,
}

impl RecordingCanonicalLog {
    /// Creates an empty recording receiver.
    #[must_use]
    pub fn new(service: ServiceName) -> Self {
        Self {
            service,
            events: Mutex::new(Vec::new()),
        }
    }

    /// Returns a detached snapshot in emission order, recovering lock poison.
    #[must_use]
    pub fn events(&self) -> Vec<LogEvent> {
        self.lock().clone()
    }

    /// Removes all recorded events, recovering lock poison.
    pub fn clear(&self) {
        self.lock().clear();
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, Vec<LogEvent>> {
        // A poisoned Vec remains a valid collection; these are test-only events.
        self.events
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }
}

impl CanonicalLog for RecordingCanonicalLog {
    fn service_name(&self) -> &ServiceName {
        &self.service
    }

    fn emit(&self, event: LogEvent) -> Result<(), v2::EventError> {
        self.lock().push(event);
        Ok(())
    }
}

/// A receiver that counts attempts and constructs a fresh routing error each time.
#[derive(Debug)]
pub struct RejectingCanonicalLog {
    service: ServiceName,
    // Only the count is shared; it publishes no other data or ordering contract.
    attempts: AtomicU64,
}

impl RejectingCanonicalLog {
    /// Creates a rejecting receiver with no recorded attempts.
    #[must_use]
    pub fn new(service: ServiceName) -> Self {
        Self {
            service,
            attempts: AtomicU64::new(0),
        }
    }

    /// Returns the number of attempted emissions.
    #[must_use]
    pub fn attempts(&self) -> u64 {
        self.attempts.load(Ordering::Relaxed)
    }
}

impl CanonicalLog for RejectingCanonicalLog {
    fn service_name(&self) -> &ServiceName {
        &self.service
    }

    fn emit(&self, _event: LogEvent) -> Result<(), v2::EventError> {
        self.attempts.fetch_add(1, Ordering::Relaxed);
        Err(v2::EventError::Routing {
            context: Box::new(ErrorContext::new(
                error_codes::DIAGNOSTIC_INVALID,
                "test receiver rejected the event",
                Remediation::recoverable(
                    "Use a recording receiver to accept test events.",
                    [] as [&str; 0],
                ),
            )),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn poisoned_recording_lock_recovers_for_emit_snapshot_and_clear() {
        let receiver = RecordingCanonicalLog::new(ServiceName::new("test").unwrap());
        let _ = std::panic::catch_unwind(|| {
            let _guard = receiver.events.lock().unwrap();
            panic!("poison the test-only event buffer");
        });
        assert!(receiver.events.is_poisoned());
        receiver.emit(crate::tests::event("first")).unwrap();
        assert_eq!(receiver.events().len(), 1);
        receiver.clear();
        assert!(receiver.events().is_empty());
    }
}
