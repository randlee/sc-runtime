//! Process-wide `sc-observability` logger for the generated daemon.
//!
//! `main` constructs [`Observability`] after config load and keeps it alive for
//! the whole process. Library crates (`sc-runtime`, `sc-cron`, …) will receive
//! an injected handle from this type once the workspace exposes a
//! `sc-observability-types` emit trait (sc-runtime observability-injection doc).

use std::path::PathBuf;

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
    logger: sc_observability::v2::Logger,
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
        Ok(Self { logger })
    }

    /// Handle for library injection (`Daemon::builder().logger(...)`, cron scheduler).
    #[must_use]
    pub fn logger(&self) -> &sc_observability::v2::Logger {
        &self.logger
    }
}

fn observe(err: impl std::fmt::Display) -> ObservabilityStart {
    ObservabilityStart {
        cause: err.to_string(),
    }
}
