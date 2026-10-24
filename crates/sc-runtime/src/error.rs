//! Typed failures from [`crate::DaemonBuilder::run`].

use std::fmt;

use sc_transport::TransportError;

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

/// Stable code when the project router already defines a reserved runtime path.
pub(crate) const CODE_ROUTE_CONFLICT: &str = "RUNTIME.ROUTE_CONFLICT";

/// Which assembly step produced a [`TransportError`].
pub(crate) enum TransportStep {
    /// Instance-root resolution.
    Instance,
    /// Endpoint resolution.
    Endpoint,
    /// Listener bind.
    Bind,
}

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
        /// Structured transport failure when this step came from `sc-transport`.
        transport: Option<Box<TransportError>>,
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
        /// Structured transport failure when this step came from `sc-transport`.
        transport: Option<Box<TransportError>>,
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
        /// Structured transport failure when this step came from `sc-transport`.
        transport: Option<Box<TransportError>>,
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
    /// The project's router already defines a reserved runtime path.
    #[non_exhaustive]
    RouteConflict {
        /// Stable code [`CODE_ROUTE_CONFLICT`](self::CODE_ROUTE_CONFLICT).
        code: &'static str,
        /// What happened.
        message: String,
        /// How to recover.
        remediation: String,
        /// The reserved path that collided.
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
            transport: None,
        }
    }

    /// Maps a [`TransportError`] onto the assembly step that produced it.
    ///
    /// The runtime code stays the step code. `remediation` is
    /// [`TransportError::recovery`]. `cause` records the transport code and
    /// fields. The original error is kept for [`Self::transport`].
    pub(crate) fn from_transport(step: TransportStep, err: TransportError) -> Self {
        let remediation = err.recovery().to_owned();
        let cause = Some(transport_facts(&err));
        let transport = Some(Box::new(err));
        match step {
            TransportStep::Instance => Self::InstanceFailed {
                code: CODE_INSTANCE_FAILED,
                message: "the instance root could not be prepared".to_owned(),
                remediation,
                cause,
                transport,
            },
            TransportStep::Endpoint => Self::EndpointFailed {
                code: CODE_ENDPOINT_FAILED,
                message: "the endpoint could not be resolved".to_owned(),
                remediation,
                cause,
                transport,
            },
            TransportStep::Bind => Self::BindFailed {
                code: CODE_BIND_FAILED,
                message: "the listener could not be bound".to_owned(),
                remediation,
                cause,
                transport,
            },
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

    pub(crate) fn route_conflict(path: impl Into<String>) -> Self {
        let path = path.into();
        Self::RouteConflict {
            code: CODE_ROUTE_CONFLICT,
            message: format!("the project router already defines reserved path {path}"),
            remediation: "remove GET /health, GET /openapi.json, or /mcp from the project router"
                .to_owned(),
            cause: Some(path),
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
            | Self::ServeFailed { code, .. }
            | Self::RouteConflict { code, .. } => code,
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
            | Self::ServeFailed { message, .. }
            | Self::RouteConflict { message, .. } => message,
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
            | Self::ServeFailed { remediation, .. }
            | Self::RouteConflict { remediation, .. } => remediation,
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
            | Self::ServeFailed { cause, .. }
            | Self::RouteConflict { cause, .. } => cause.as_deref(),
        }
    }

    /// Structured transport failure when this step came from `sc-transport`.
    ///
    /// [`Self::code`] still names the assembly step. The returned error keeps
    /// the transport code and fields. `None` when the cause is not a
    /// transport error.
    ///
    /// # Examples
    ///
    /// ```
    /// use sc_runtime::RuntimeError;
    ///
    /// fn reads_transport_code(err: &RuntimeError) -> Option<&'static str> {
    ///     err.transport().map(sc_transport::TransportError::code)
    /// }
    ///
    /// let err = RuntimeError::stores_failed("STORE.OPEN", "closed", "check the path", None);
    /// assert!(reads_transport_code(&err).is_none());
    /// ```
    #[must_use]
    pub fn transport(&self) -> Option<&TransportError> {
        match self {
            Self::InstanceFailed { transport, .. }
            | Self::EndpointFailed { transport, .. }
            | Self::BindFailed { transport, .. } => transport.as_deref(),
            Self::LockHeld { .. }
            | Self::StoresFailed { .. }
            | Self::ServeFailed { .. }
            | Self::RouteConflict { .. } => None,
        }
    }
}

/// Records the transport code and variant fields without using [`Display`](TransportError).
fn transport_facts(err: &TransportError) -> String {
    match err {
        TransportError::InvalidName { name, detail, .. } => {
            format!("{}: name={name}: {detail}", err.code())
        }
        TransportError::InvalidEndpoint { source, detail, .. } => {
            format!("{}: source={source}: {detail}", err.code())
        }
        TransportError::MissingTcpPort { detail, .. } => {
            format!("{}: {detail}", err.code())
        }
        TransportError::InstanceRootUnavailable { detail, .. } => {
            format!("{}: {detail}", err.code())
        }
        TransportError::ResponseDecode {
            status,
            cause,
            body_preview,
            ..
        } => format!("{}: status={status}: {cause}: {body_preview}", err.code()),
        TransportError::DaemonNotRunning {
            endpoint, cause, ..
        }
        | TransportError::TimedOut {
            endpoint, cause, ..
        }
        | TransportError::Request {
            endpoint, cause, ..
        }
        | TransportError::Bind {
            endpoint, cause, ..
        }
        | TransportError::EndpointBusy {
            endpoint, cause, ..
        }
        | TransportError::NotASocket {
            endpoint, cause, ..
        } => format!("{}: endpoint={endpoint}: {cause}", err.code()),
        // `TransportError` is `non_exhaustive`. Keep the code if a new variant appears.
        _ => format!("{}: {err}", err.code()),
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

impl std::error::Error for RuntimeError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.transport()
            .map(|err| err as &(dyn std::error::Error + 'static))
    }
}

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

    #[test]
    fn transport_mapping_keeps_code_and_fields() {
        let missing = "not-an-endpoint"
            .parse::<sc_transport::Endpoint>()
            .expect_err("endpoint");
        for step in [
            TransportStep::Instance,
            TransportStep::Endpoint,
            TransportStep::Bind,
        ] {
            let err = RuntimeError::from_transport(step, missing.clone());
            let transport = err.transport().expect("structured transport error");
            assert_eq!(transport.code(), missing.code());
            assert_eq!(err.remediation(), missing.recovery());
            let cause = err.cause().expect("cause");
            assert!(cause.contains(missing.code()), "{cause}");
            assert!(cause.contains("source=not-an-endpoint"), "{cause}");
            assert!(
                cause.contains("endpoint must start with uds: or tcp:"),
                "{cause}"
            );
            assert_ne!(cause, missing.to_string());
            match transport {
                TransportError::InvalidEndpoint { source, detail, .. } => {
                    assert_eq!(source, "not-an-endpoint");
                    assert_eq!(detail, "endpoint must start with uds: or tcp:");
                }
                other => panic!("expected InvalidEndpoint, got {other:?}"),
            }
            let source = std::error::Error::source(&err).expect("source");
            let downcast = source
                .downcast_ref::<TransportError>()
                .expect("downcast to TransportError");
            assert_eq!(downcast, &missing);
        }
        let RuntimeError::EndpointFailed { transport, .. } =
            RuntimeError::from_transport(TransportStep::Endpoint, missing)
        else {
            panic!("endpoint step");
        };
        assert!(transport.is_some());
    }
}
