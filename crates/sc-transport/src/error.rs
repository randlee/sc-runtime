//! Typed failures for endpoint resolution, the client, and listener binding.

use std::fmt;

use crate::endpoint::Endpoint;

/// Stable code for an application name that fails [`crate::AppName`] construction.
pub const CODE_INVALID_NAME: &str = "APP.INVALID_NAME";

/// Stable code for an endpoint string that is not `uds:` or `tcp:`.
pub const CODE_INVALID_ENDPOINT: &str = "TRANSPORT.INVALID_ENDPOINT";

/// Stable code when the Windows platform default has no configured port.
pub const CODE_MISSING_TCP_PORT: &str = "TRANSPORT.MISSING_TCP_PORT";

/// Stable code when the per-user base directory cannot be determined.
pub const CODE_INSTANCE_ROOT: &str = "TRANSPORT.INSTANCE_ROOT_UNAVAILABLE";

/// Stable code when the daemon is not accepting connections.
pub const CODE_DAEMON_NOT_RUNNING: &str = "DAEMON.NOT_RUNNING";

/// Stable code when a response body is not the requested type.
pub const CODE_RESPONSE_DECODE: &str = "TRANSPORT.RESPONSE_DECODE";

/// Stable code when a connect or request timeout elapses.
pub const CODE_TIMED_OUT: &str = "TRANSPORT.TIMED_OUT";

/// Stable code for a client failure that is not a down daemon or a decode.
pub const CODE_REQUEST: &str = "TRANSPORT.REQUEST";

/// Stable code when binding a listener fails.
pub const CODE_BIND: &str = "TRANSPORT.BIND";

/// Stable code when another listener holds the endpoint lock.
pub const CODE_ENDPOINT_BUSY: &str = "TRANSPORT.ENDPOINT_BUSY";

/// Stable code when a UDS path exists and is not a socket.
pub const CODE_NOT_A_SOCKET: &str = "TRANSPORT.NOT_A_SOCKET";

/// Stable code when the platform cannot bind a UDS endpoint.
pub const CODE_UNSUPPORTED_ENDPOINT: &str = "TRANSPORT.UNSUPPORTED_ENDPOINT";

const MISSING_TCP_PORT_DETAIL: &str = "set TransportConfig::tcp_port or pass a tcp: endpoint override; this crate has no default port";

/// Cause text chained from [`TransportError::source`].
///
/// Public functions return [`TransportError`]. This type is the `cause`
/// payload on those variants, not a second `Result` error.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct TransportCause {
    message: String,
}

impl TransportCause {
    pub(crate) fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }

    /// Returns the cause text.
    pub fn as_str(&self) -> &str {
        &self.message
    }

    /// Returns whether the cause text is empty.
    pub fn is_empty(&self) -> bool {
        self.message.is_empty()
    }
}

impl fmt::Display for TransportCause {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for TransportCause {}

impl AsRef<str> for TransportCause {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

/// Failure returned by every fallible public function in this crate.
///
/// `DaemonNotRunning` carries `code`, the endpoint, and the cause. It has no
/// suggested action, no application name, and no observability error type.
/// A later CLI maps `DAEMON.NOT_RUNNING` and chooses the action text.
/// [`TransportError::recovery`] is the next step for every variant. It is
/// not a remediation type and it does not name a daemon start command.
///
/// Each struct variant is [`non_exhaustive`]. The `code` field is
/// informational. [`TransportError::code`] is the source of truth.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum TransportError {
    /// `name` failed the application-name predicate.
    #[non_exhaustive]
    InvalidName {
        /// Informational copy of [`CODE_INVALID_NAME`]. Prefer [`TransportError::code`].
        code: &'static str,
        /// The rejected name.
        name: String,
        /// Why the name was rejected, including the allowed set.
        detail: String,
    },
    /// `source` is not valid endpoint text.
    #[non_exhaustive]
    InvalidEndpoint {
        /// Informational copy of [`CODE_INVALID_ENDPOINT`]. Prefer [`TransportError::code`].
        code: &'static str,
        /// The text that failed to parse.
        source: String,
        /// Why parsing failed.
        detail: String,
    },
    /// The Windows default needs `TransportConfig::tcp_port` or a `tcp:` override.
    #[non_exhaustive]
    MissingTcpPort {
        /// Informational copy of [`CODE_MISSING_TCP_PORT`]. Prefer [`TransportError::code`].
        code: &'static str,
        /// How to supply the missing port.
        detail: &'static str,
    },
    /// No per-user base directory was available.
    #[non_exhaustive]
    InstanceRootUnavailable {
        /// Informational copy of [`CODE_INSTANCE_ROOT`]. Prefer [`TransportError::code`].
        code: &'static str,
        /// Which location was missing.
        detail: String,
    },
    /// The endpoint refused the connection or the UDS path was missing.
    #[non_exhaustive]
    DaemonNotRunning {
        /// Informational copy of [`CODE_DAEMON_NOT_RUNNING`]. Prefer [`TransportError::code`].
        code: &'static str,
        /// The endpoint the client tried to open.
        endpoint: Endpoint,
        /// The I/O failure from the connection attempt.
        cause: TransportCause,
    },
    /// The response body could not be deserialized as `T`.
    #[non_exhaustive]
    ResponseDecode {
        /// Informational copy of [`CODE_RESPONSE_DECODE`]. Prefer [`TransportError::code`].
        code: &'static str,
        /// HTTP status of the response that failed to decode.
        status: u16,
        /// The decode failure.
        cause: TransportCause,
        /// Bounded, lossy UTF-8 prefix of the body that failed to decode.
        body_preview: String,
    },
    /// The connect or request timeout elapsed.
    #[non_exhaustive]
    TimedOut {
        /// Informational copy of [`CODE_TIMED_OUT`]. Prefer [`TransportError::code`].
        code: &'static str,
        /// The endpoint the client was calling.
        endpoint: Endpoint,
        /// The timeout failure.
        cause: TransportCause,
    },
    /// A client failure other than a down daemon, a timeout, or a decode.
    #[non_exhaustive]
    Request {
        /// Informational copy of [`CODE_REQUEST`]. Prefer [`TransportError::code`].
        code: &'static str,
        /// The endpoint the client was calling.
        endpoint: Endpoint,
        /// The underlying failure.
        cause: TransportCause,
    },
    /// Binding the listener failed.
    #[non_exhaustive]
    Bind {
        /// Informational copy of [`CODE_BIND`]. Prefer [`TransportError::code`].
        code: &'static str,
        /// The endpoint that could not be bound.
        endpoint: Endpoint,
        /// The underlying failure.
        cause: TransportCause,
    },
    /// The UDS endpoint lock is already held.
    #[non_exhaustive]
    EndpointBusy {
        /// Informational copy of [`CODE_ENDPOINT_BUSY`]. Prefer [`TransportError::code`].
        code: &'static str,
        /// The endpoint whose lock is held.
        endpoint: Endpoint,
        /// Why the lock was not acquired.
        cause: TransportCause,
    },
    /// The UDS path exists and is not a socket file.
    #[non_exhaustive]
    NotASocket {
        /// Informational copy of [`CODE_NOT_A_SOCKET`]. Prefer [`TransportError::code`].
        code: &'static str,
        /// The path that was left unchanged.
        endpoint: Endpoint,
        /// Why the path was rejected.
        cause: TransportCause,
    },
    /// This platform cannot bind the endpoint. UDS on non-Unix is this case.
    #[non_exhaustive]
    UnsupportedEndpoint {
        /// Informational copy of [`CODE_UNSUPPORTED_ENDPOINT`]. Prefer [`TransportError::code`].
        code: &'static str,
        /// The endpoint that was rejected.
        endpoint: Endpoint,
        /// Why the endpoint is unsupported.
        cause: TransportCause,
    },
}

impl TransportError {
    /// Stable code for this failure.
    ///
    /// This is the source of truth. The `code` field on each variant is
    /// informational and is not read here.
    pub fn code(&self) -> &'static str {
        match self {
            Self::InvalidName { .. } => CODE_INVALID_NAME,
            Self::InvalidEndpoint { .. } => CODE_INVALID_ENDPOINT,
            Self::MissingTcpPort { .. } => CODE_MISSING_TCP_PORT,
            Self::InstanceRootUnavailable { .. } => CODE_INSTANCE_ROOT,
            Self::DaemonNotRunning { .. } => CODE_DAEMON_NOT_RUNNING,
            Self::ResponseDecode { .. } => CODE_RESPONSE_DECODE,
            Self::TimedOut { .. } => CODE_TIMED_OUT,
            Self::Request { .. } => CODE_REQUEST,
            Self::Bind { .. } => CODE_BIND,
            Self::EndpointBusy { .. } => CODE_ENDPOINT_BUSY,
            Self::NotASocket { .. } => CODE_NOT_A_SOCKET,
            Self::UnsupportedEndpoint { .. } => CODE_UNSUPPORTED_ENDPOINT,
        }
    }

    /// Returns the recovery step for this failure.
    ///
    /// The text is the next step for the caller. It is static, it is not a
    /// `suggested_action`, and [`Self::DaemonNotRunning`] does not name a
    /// start command.
    ///
    /// # Examples
    ///
    /// ```
    /// use sc_transport::AppName;
    ///
    /// let error = AppName::new("").expect_err("empty name");
    /// assert!(error.recovery().contains("ASCII"));
    /// assert!(error.to_string().contains(error.recovery()));
    /// ```
    pub fn recovery(&self) -> &'static str {
        match self {
            Self::InvalidName { .. } => "Use 1 to 64 bytes of ASCII letters, digits, '-', and '_'.",
            Self::InvalidEndpoint { .. } => "Use uds:<path> or tcp:<socket-addr>.",
            Self::MissingTcpPort { .. } => MISSING_TCP_PORT_DETAIL,
            Self::InstanceRootUnavailable { .. } => {
                "Set XDG_RUNTIME_DIR or HOME on Unix, LOCALAPPDATA on Windows, or pass TransportConfig::instance_root."
            }
            Self::DaemonNotRunning { .. } => {
                "Check that a listener is accepting connections on the endpoint. This crate only reports the failure."
            }
            Self::ResponseDecode { .. } => {
                "Compare the HTTP status, the decode cause, and body_preview with the requested type."
            }
            Self::TimedOut { .. } => {
                "Retry the call. Connect and request timeouts are fixed at 2 seconds in v0.1."
            }
            Self::Request { .. } => "Correct the endpoint. This failure is not a down daemon.",
            Self::Bind { .. } => {
                "Free the address or create the parent directory. This crate does not create it."
            }
            Self::EndpointBusy { .. } => {
                "Leave the existing socket in place. Another listener holds the endpoint lock."
            }
            Self::NotASocket { .. } => {
                "Choose a socket path. The existing path was left unchanged."
            }
            Self::UnsupportedEndpoint { .. } => {
                "Use a tcp: endpoint. Unix domain sockets are not available on this platform."
            }
        }
    }

    pub(crate) fn invalid_name(name: impl Into<String>, detail: impl Into<String>) -> Self {
        Self::InvalidName {
            code: CODE_INVALID_NAME,
            name: name.into(),
            detail: detail.into(),
        }
    }

    pub(crate) fn invalid_endpoint(source: impl Into<String>, detail: impl Into<String>) -> Self {
        Self::InvalidEndpoint {
            code: CODE_INVALID_ENDPOINT,
            source: source.into(),
            detail: detail.into(),
        }
    }

    #[cfg_attr(all(unix, not(test)), allow(dead_code))]
    pub(crate) fn missing_tcp_port() -> Self {
        Self::MissingTcpPort {
            code: CODE_MISSING_TCP_PORT,
            detail: MISSING_TCP_PORT_DETAIL,
        }
    }

    pub(crate) fn instance_root(detail: impl Into<String>) -> Self {
        Self::InstanceRootUnavailable {
            code: CODE_INSTANCE_ROOT,
            detail: detail.into(),
        }
    }

    pub(crate) fn daemon_not_running(endpoint: Endpoint, cause: impl Into<String>) -> Self {
        Self::DaemonNotRunning {
            code: CODE_DAEMON_NOT_RUNNING,
            endpoint,
            cause: TransportCause::new(cause),
        }
    }

    pub(crate) fn response_decode(
        status: u16,
        cause: impl Into<String>,
        body_preview: impl Into<String>,
    ) -> Self {
        Self::ResponseDecode {
            code: CODE_RESPONSE_DECODE,
            status,
            cause: TransportCause::new(cause),
            body_preview: body_preview.into(),
        }
    }

    pub(crate) fn timed_out(endpoint: Endpoint, cause: impl Into<String>) -> Self {
        Self::TimedOut {
            code: CODE_TIMED_OUT,
            endpoint,
            cause: TransportCause::new(cause),
        }
    }

    pub(crate) fn request(endpoint: Endpoint, cause: impl Into<String>) -> Self {
        Self::Request {
            code: CODE_REQUEST,
            endpoint,
            cause: TransportCause::new(cause),
        }
    }

    #[cfg(feature = "server")]
    pub(crate) fn bind(endpoint: Endpoint, cause: impl Into<String>) -> Self {
        Self::Bind {
            code: CODE_BIND,
            endpoint,
            cause: TransportCause::new(cause),
        }
    }

    /// Rejects an endpoint this platform cannot bind.
    ///
    /// Recovery tells the caller to use a `tcp:` endpoint.
    #[cfg(all(feature = "server", not(unix)))]
    pub(crate) fn unsupported_endpoint(endpoint: Endpoint, cause: impl Into<String>) -> Self {
        Self::UnsupportedEndpoint {
            code: CODE_UNSUPPORTED_ENDPOINT,
            endpoint,
            cause: TransportCause::new(cause),
        }
    }

    #[cfg(feature = "server")]
    #[cfg_attr(
        windows,
        expect(
            dead_code,
            reason = "Unix endpoint-lock error. Windows bind returns UnsupportedEndpoint instead"
        )
    )]
    pub(crate) fn endpoint_busy(endpoint: Endpoint, cause: impl Into<String>) -> Self {
        Self::EndpointBusy {
            code: CODE_ENDPOINT_BUSY,
            endpoint,
            cause: TransportCause::new(cause),
        }
    }

    #[cfg(feature = "server")]
    #[cfg_attr(
        windows,
        expect(
            dead_code,
            reason = "Unix socket-path error. Windows bind returns UnsupportedEndpoint instead"
        )
    )]
    pub(crate) fn not_a_socket(endpoint: Endpoint, cause: impl Into<String>) -> Self {
        Self::NotASocket {
            code: CODE_NOT_A_SOCKET,
            endpoint,
            cause: TransportCause::new(cause),
        }
    }

    fn chained_cause(&self) -> Option<&TransportCause> {
        match self {
            Self::DaemonNotRunning { cause, .. }
            | Self::ResponseDecode { cause, .. }
            | Self::TimedOut { cause, .. }
            | Self::Request { cause, .. }
            | Self::Bind { cause, .. }
            | Self::EndpointBusy { cause, .. }
            | Self::NotASocket { cause, .. }
            | Self::UnsupportedEndpoint { cause, .. } => Some(cause),
            Self::InvalidName { .. }
            | Self::InvalidEndpoint { .. }
            | Self::MissingTcpPort { .. }
            | Self::InstanceRootUnavailable { .. } => None,
        }
    }
}

impl fmt::Display for TransportError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let recovery = self.recovery();
        match self {
            Self::InvalidName { code, name, detail } => {
                write_with_recovery(f, &format!("{code}: {name}: {detail}"), recovery)
            }
            Self::InvalidEndpoint {
                code,
                source,
                detail,
            } => write_with_recovery(f, &format!("{code}: {source}: {detail}"), recovery),
            Self::MissingTcpPort { code, detail } => {
                write_with_recovery(f, &format!("{code}: {detail}"), recovery)
            }
            Self::InstanceRootUnavailable { code, detail } => {
                write_with_recovery(f, &format!("{code}: {detail}"), recovery)
            }
            Self::DaemonNotRunning {
                code,
                endpoint,
                cause,
            }
            | Self::TimedOut {
                code,
                endpoint,
                cause,
            }
            | Self::Request {
                code,
                endpoint,
                cause,
            }
            | Self::Bind {
                code,
                endpoint,
                cause,
            }
            | Self::EndpointBusy {
                code,
                endpoint,
                cause,
            }
            | Self::NotASocket {
                code,
                endpoint,
                cause,
            }
            | Self::UnsupportedEndpoint {
                code,
                endpoint,
                cause,
            } => write_with_recovery(f, &format!("{code}: {endpoint}: {cause}"), recovery),
            Self::ResponseDecode {
                code,
                status,
                cause,
                body_preview,
            } => {
                let fact = if body_preview.is_empty() {
                    format!("{code}: http {status}: {cause}")
                } else {
                    format!("{code}: http {status}: {cause}: {body_preview}")
                };
                write_with_recovery(f, &fact, recovery)
            }
        }
    }
}

/// Appends `recovery` when the diagnostic does not already contain it.
fn write_with_recovery(f: &mut fmt::Formatter<'_>, fact: &str, recovery: &str) -> fmt::Result {
    if fact.contains(recovery) {
        f.write_str(fact)
    } else {
        write!(f, "{fact}; {recovery}")
    }
}

impl std::error::Error for TransportError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.chained_cause()
            .map(|cause| cause as &(dyn std::error::Error + 'static))
    }
}

#[cfg(test)]
mod tests {
    use std::error::Error;
    use std::path::PathBuf;

    use super::*;

    #[test]
    fn daemon_not_running_has_code_endpoint_and_cause_only() {
        let endpoint = Endpoint::Uds(PathBuf::from("daemon.sock"));
        let err = TransportError::daemon_not_running(endpoint.clone(), "connection refused");
        match &err {
            TransportError::DaemonNotRunning {
                code,
                endpoint: got,
                cause,
            } => {
                assert_eq!(*code, CODE_DAEMON_NOT_RUNNING);
                assert_eq!(got, &endpoint);
                assert_eq!(cause.as_str(), "connection refused");
            }
            other => panic!("unexpected {other:?}"),
        }
        let rendered = format!("{err:?}");
        assert!(!rendered.contains("suggested_action"));
        assert_eq!(
            Error::source(&err).map(ToString::to_string).as_deref(),
            Some("connection refused")
        );
    }

    #[test]
    fn missing_tcp_port_explains_recovery() {
        let err = TransportError::missing_tcp_port();
        assert_eq!(err.code(), CODE_MISSING_TCP_PORT);
        assert!(err.recovery().contains("tcp_port"), "{}", err.recovery());
        assert!(err.to_string().contains(err.recovery()), "{err}");
        assert!(Error::source(&err).is_none());
    }

    #[test]
    fn recovery_text_is_displayed_for_key_variants() {
        let endpoint = Endpoint::Tcp("127.0.0.1:9".parse().unwrap());
        let cases = [
            TransportError::invalid_name("my app", "name contains ' '"),
            TransportError::invalid_endpoint("http://x", "endpoint must start with uds: or tcp:"),
            TransportError::missing_tcp_port(),
            TransportError::instance_root("HOME is unset"),
            TransportError::daemon_not_running(endpoint.clone(), "connection refused"),
            TransportError::response_decode(500, "expected object", "{\"err\":true}"),
            TransportError::timed_out(endpoint.clone(), "timed out"),
            TransportError::request(endpoint.clone(), "not a socket directory"),
            TransportError::Bind {
                code: CODE_BIND,
                endpoint: endpoint.clone(),
                cause: TransportCause::new("address already in use"),
            },
            TransportError::EndpointBusy {
                code: CODE_ENDPOINT_BUSY,
                endpoint: endpoint.clone(),
                cause: TransportCause::new("lock held"),
            },
            TransportError::NotASocket {
                code: CODE_NOT_A_SOCKET,
                endpoint: endpoint.clone(),
                cause: TransportCause::new("regular file"),
            },
            TransportError::UnsupportedEndpoint {
                code: CODE_UNSUPPORTED_ENDPOINT,
                endpoint,
                cause: TransportCause::new("UDS endpoints are not supported on this platform"),
            },
        ];
        for err in cases {
            let recovery = err.recovery();
            assert!(!recovery.is_empty(), "{err:?}");
            assert!(
                err.to_string().contains(recovery),
                "display missing recovery for {err:?}: {err}"
            );
            assert!(
                !recovery.contains("daemon start"),
                "recovery must not name a start command: {recovery}"
            );
            assert!(
                !recovery.contains("suggested_action"),
                "recovery is not a suggested_action field: {recovery}"
            );
        }

        let invalid = TransportError::invalid_name("", "name is empty");
        assert!(
            invalid.recovery().contains("ASCII"),
            "{}",
            invalid.recovery()
        );
        let endpoint_err = TransportError::invalid_endpoint("", "endpoint text is empty");
        assert!(
            endpoint_err.recovery().contains("uds:"),
            "{}",
            endpoint_err.recovery()
        );
        let down = TransportError::daemon_not_running(
            Endpoint::Uds(PathBuf::from("daemon.sock")),
            "connection refused",
        );
        assert!(down.recovery().contains("listener"), "{}", down.recovery());
        let root = TransportError::instance_root("missing");
        assert!(
            root.recovery().contains("XDG_RUNTIME_DIR"),
            "{}",
            root.recovery()
        );
        assert!(
            root.recovery().contains("LOCALAPPDATA"),
            "{}",
            root.recovery()
        );
        let unsupported = TransportError::UnsupportedEndpoint {
            code: CODE_UNSUPPORTED_ENDPOINT,
            endpoint: Endpoint::Uds(PathBuf::from("daemon.sock")),
            cause: TransportCause::new("UDS endpoints are not supported on this platform"),
        };
        assert_eq!(unsupported.code(), CODE_UNSUPPORTED_ENDPOINT);
        assert!(
            unsupported.recovery().contains("tcp:"),
            "{}",
            unsupported.recovery()
        );
    }
}
