//! Development-only bridge to the published 1.5.0 logger.
//!
//! The application owns construction, flushing, and shutdown. This adapter
//! retains a shared logger and only forwards the canonical interface.

#![forbid(unsafe_code)]

use std::sync::Arc;

use sc_observability::v2::Logger;
use sc_observability_types::{CanonicalLog, LogEvent, ServiceName, v2::EventError};

/// Forwards canonical events to the released logger's non-blocking admission.
pub struct LoggerAdapter {
    logger: Arc<Logger>,
}

impl LoggerAdapter {
    /// Wraps an application-owned logger without changing its lifecycle.
    #[must_use]
    pub fn new(logger: Arc<Logger>) -> Self {
        Self { logger }
    }
}

impl CanonicalLog for LoggerAdapter {
    fn service_name(&self) -> &ServiceName {
        self.logger.service_name()
    }

    fn emit(&self, event: LogEvent) -> Result<(), EventError> {
        self.logger.try_log(event)
    }
}
