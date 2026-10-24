//! Typed failures from [`crate::DaemonBuilder::run`].

use std::fmt;

/// Stable code when another daemon holds `<instance-root>/daemon.lock`.
pub(crate) const CODE_LOCK_HELD: &str = "RUNTIME.LOCK_HELD";

/// Stable code when the instance root cannot be prepared or the lock file cannot be opened.
pub(crate) const CODE_INSTANCE_FAILED: &str = "RUNTIME.INSTANCE_FAILED";

/// Stable code when endpoint resolution fails.
pub(crate) const CODE_ENDPOINT_FAILED: &str = "RUNTIME.ENDPOINT_FAILED";

/// Stable code when the listener cannot be bound.
pub(crate) const CODE_BIND_FAILED: &str = "RUNTIME.BIND_FAILED";

/// Stable code when serving the router fails.
pub(crate) const CODE_SERVE_FAILED: &str = "RUNTIME.SERVE_FAILED";

/// Failure returned by [`crate::DaemonBuilder::run`].
///
/// `StoresFailed` carries plain text the application's stores closure built.
/// This crate does not name a store error type. `code` is `&'static str`.
/// `message`, `remediation`, and `cause` are owned strings. There is no
/// `AppName` variant. The type is not generic.
///
/// # Examples
///
/// ```
/// use sc_runtime::RuntimeError;
///
/// let err = RuntimeError::stores_failed(
///     "STORE.OPEN",
///     "the store did not open",
///     "check the database path",
///     None,
/// );
/// assert_eq!(err.code(), "STORE.OPEN");
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum RuntimeError {
    /// Another daemon holds the instance lock. The stores closure did not run.
    #[non_exhaustive]
    LockHeld {
        /// Stable code [`CODE_LOCK_HELD`](self::CODE_LOCK_HELD).
        code: &'static str,
        /// What happened.
        message: String,
        /// How to recover.
        remediation: String,
        /// Extra cause text, when the lock call returned one.
        cause: Option<String>,
    },
    /// The stores closure returned this failure. The application fills every field.
    #[non_exhaustive]
    StoresFailed {
        /// Caller-supplied stable code.
        code: &'static str,
        /// Caller-supplied message.
        message: String,
        /// Caller-supplied recovery text.
        remediation: String,
        /// Caller-supplied cause text.
        cause: Option<String>,
    },
    /// The instance root or lock file could not be prepared. This is not [`LockHeld`](Self::LockHeld).
    #[non_exhaustive]
    InstanceFailed {
        /// Stable code [`CODE_INSTANCE_FAILED`](self::CODE_INSTANCE_FAILED).
        code: &'static str,
        /// What happened.
        message: String,
        /// How to recover.
        remediation: String,
        /// Resolver or I/O text.
        cause: Option<String>,
    },
    /// Endpoint resolution failed. No listener was bound.
    #[non_exhaustive]
    EndpointFailed {
        /// Stable code [`CODE_ENDPOINT_FAILED`](self::CODE_ENDPOINT_FAILED).
        code: &'static str,
        /// What happened.
        message: String,
        /// How to recover.
        remediation: String,
        /// Resolver text.
        cause: Option<String>,
    },
    /// `sc-transport` could not bind the listener.
    #[non_exhaustive]
    BindFailed {
        /// Stable code [`CODE_BIND_FAILED`](self::CODE_BIND_FAILED).
        code: &'static str,
        /// What happened.
        message: String,
        /// How to recover.
        remediation: String,
        /// Bind text.
        cause: Option<String>,
    },
    /// `axum::serve` returned an error.
    #[non_exhaustive]
    ServeFailed {
        /// Stable code [`CODE_SERVE_FAILED`](self::CODE_SERVE_FAILED).
        code: &'static str,
        /// What happened.
        message: String,
        /// How to recover.
        remediation: String,
        /// Serve text.
        cause: Option<String>,
    },
}

impl RuntimeError {
    /// Builds [`RuntimeError::StoresFailed`] from text the application owns.
    ///
    /// The stores closure maps its store error into this value. This crate
    /// does not downcast that error.
    #[must_use]
    pub fn stores_failed(
        code: &'static str,
        message: impl Into<String>,
        remediation: impl Into<String>,
        cause: Option<String>,
    ) -> Self {
        Self::StoresFailed {
            code,
            message: message.into(),
            remediation: remediation.into(),
            cause,
        }
    }

    pub(crate) fn lock_held(cause: Option<String>) -> Self {
        Self::LockHeld {
            code: CODE_LOCK_HELD,
            message: "a daemon already holds the instance lock".to_owned(),
            remediation:
                "stop the other daemon for this instance root, or choose a different instance root"
                    .to_owned(),
            cause,
        }
    }

    pub(crate) fn instance_failed(cause: impl Into<String>) -> Self {
        Self::InstanceFailed {
            code: CODE_INSTANCE_FAILED,
            message: "the instance root could not be prepared".to_owned(),
            remediation:
                "pass a directory the process can create as TransportConfig::instance_root"
                    .to_owned(),
            cause: Some(cause.into()),
        }
    }

    pub(crate) fn endpoint_failed(cause: impl Into<String>) -> Self {
        Self::EndpointFailed {
            code: CODE_ENDPOINT_FAILED,
            message: "the endpoint could not be resolved".to_owned(),
            remediation: "use uds:<path> or tcp:<socket-addr>, or set TransportConfig::tcp_port when the platform default needs a port".to_owned(),
            cause: Some(cause.into()),
        }
    }

    pub(crate) fn bind_failed(cause: impl Into<String>) -> Self {
        Self::BindFailed {
            code: CODE_BIND_FAILED,
            message: "the listener could not be bound".to_owned(),
            remediation: "leave a live endpoint in place, or choose an address that is free"
                .to_owned(),
            cause: Some(cause.into()),
        }
    }

    pub(crate) fn serve_failed(cause: impl Into<String>) -> Self {
        Self::ServeFailed {
            code: CODE_SERVE_FAILED,
            message: "serving the router failed".to_owned(),
            remediation: "read the cause and start the daemon again".to_owned(),
            cause: Some(cause.into()),
        }
    }

    /// Stable code for this failure.
    #[must_use]
    pub fn code(&self) -> &'static str {
        match self {
            Self::LockHeld { code, .. }
            | Self::StoresFailed { code, .. }
            | Self::InstanceFailed { code, .. }
            | Self::EndpointFailed { code, .. }
            | Self::BindFailed { code, .. }
            | Self::ServeFailed { code, .. } => code,
        }
    }

    /// Message text.
    #[must_use]
    pub fn message(&self) -> &str {
        match self {
            Self::LockHeld { message, .. }
            | Self::StoresFailed { message, .. }
            | Self::InstanceFailed { message, .. }
            | Self::EndpointFailed { message, .. }
            | Self::BindFailed { message, .. }
            | Self::ServeFailed { message, .. } => message,
        }
    }

    /// Recovery text.
    #[must_use]
    pub fn remediation(&self) -> &str {
        match self {
            Self::LockHeld { remediation, .. }
            | Self::StoresFailed { remediation, .. }
            | Self::InstanceFailed { remediation, .. }
            | Self::EndpointFailed { remediation, .. }
            | Self::BindFailed { remediation, .. }
            | Self::ServeFailed { remediation, .. } => remediation,
        }
    }

    /// Optional cause text.
    #[must_use]
    pub fn cause(&self) -> Option<&str> {
        match self {
            Self::LockHeld { cause, .. }
            | Self::StoresFailed { cause, .. }
            | Self::InstanceFailed { cause, .. }
            | Self::EndpointFailed { cause, .. }
            | Self::BindFailed { cause, .. }
            | Self::ServeFailed { cause, .. } => cause.as_deref(),
        }
    }
}

impl fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}: {}. {}",
            self.code(),
            self.message(),
            self.remediation()
        )?;
        if let Some(cause) = self.cause() {
            write!(f, " ({cause})")?;
        }
        Ok(())
    }
}

impl std::error::Error for RuntimeError {}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_error_bounds<T: std::error::Error + Send + Sync + 'static>() {}

    #[test]
    fn stores_failed_fields_are_plain_text() {
        assert_error_bounds::<RuntimeError>();
        let err = RuntimeError::stores_failed(
            "STORE.OPEN",
            "the store did not open",
            "check the database path",
            Some("disk full".to_owned()),
        );
        let RuntimeError::StoresFailed {
            code,
            message,
            remediation,
            cause,
        } = err
        else {
            return assert!(matches!(err, RuntimeError::StoresFailed { .. }));
        };
        let _code: &'static str = code;
        let _message: String = message;
        let _remediation: String = remediation;
        let _cause: Option<String> = cause;
        assert_eq!(_code, "STORE.OPEN");
        assert_eq!(_message, "the store did not open");
        assert_eq!(_remediation, "check the database path");
        assert_eq!(_cause.as_deref(), Some("disk full"));
    }
}
