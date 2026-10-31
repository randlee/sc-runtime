//! Process-wide `sc-observability` logger for the generated daemon.
//!
//! `main` constructs [`Observability`] after config load and keeps it alive for
//! the whole process. Runtime and cron share one canonical handle. The caller
//! drains and drops both consumers before consuming the owner with `shutdown`.

use std::path::PathBuf;
use std::sync::Arc;

use sc_observability_types::SharedCanonicalLog;

use sc_observability::{LogSettings, LoggerConfig, ServiceName};

/// Observability failed to start. The cause is the builder error text.
pub struct ObservabilityStart {
    cause: String,
}

impl std::fmt::Display for ObservabilityStart {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.cause)
    }
}

/// Running logger: file + console sinks under the resolved log root.
pub struct Observability {
    logger: Arc<sc_observability::v2::Logger>,
    shared: SharedCanonicalLog,
}

impl Observability {
    /// Builds the service logger for `app_name` using config and `log_root`.
    ///
    /// `log_root` is usually the transport instance root when
    /// `logging.log_root` is unset. File events land in
    /// `<log_root>/logs/<service>.log.jsonl`.
    ///
    /// # Errors
    ///
    /// Returns [`ObservabilityStart`] when service name or logger build fails.
    pub fn init(
        app_name: &str,
        logging: &LogSettings,
        log_root: PathBuf,
    ) -> Result<Self, ObservabilityStart> {
        let service = ServiceName::new(app_name).map_err(observe)?;
        let mut settings = LoggerConfig::default_for(service, log_root);
        if let Some(level) = logging.level {
            settings.level = level;
        }
        if let Some(enabled) = logging.enable_file_sink {
            settings.enable_file_sink = enabled;
        }
        if let Some(enabled) = logging.enable_console_sink {
            settings.enable_console_sink = enabled;
        }
        let logger = sc_observability::v2::LoggerBuilder::new(settings)
            .map_err(observe)?
            .build()
            .map_err(observe)?;
        let logger = Arc::new(logger);
        let shared: SharedCanonicalLog = logger.clone();
        Ok(Self { logger, shared })
    }

    /// Handle for library injection (`Daemon::builder().logger(...)`, cron scheduler).
    #[must_use]
    pub fn logger(&self) -> SharedCanonicalLog {
        Arc::clone(&self.shared)
    }

    /// Flushes and joins the logger after all injected consumers have dropped.
    ///
    /// # Errors
    /// Returns the production logger's sink or drain failure.
    pub fn shutdown(self) -> Result<(), sc_observability::v2::ShutdownError> {
        let Self { logger, shared } = self;
        drop(shared);
        logger.shutdown()
    }
}

fn observe(err: impl std::fmt::Display) -> ObservabilityStart {
    ObservabilityStart {
        cause: err.to_string(),
    }
}
