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

    #[cfg(feature = "server")]
    pub(crate) fn endpoint_busy(endpoint: Endpoint, cause: impl Into<String>) -> Self {
        Self::EndpointBusy {
            code: CODE_ENDPOINT_BUSY,
            endpoint,
            cause: TransportCause::new(cause),
        }
    }

    #[cfg(feature = "server")]
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
            | Self::NotASocket { cause, .. } => Some(cause),
            Self::InvalidName { .. }
            | Self::InvalidEndpoint { .. }
            | Self::MissingTcpPort { .. }
            | Self::InstanceRootUnavailable { .. } => None,
        }
    }
}

impl fmt::Display for TransportError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidName { code, name, detail } => write!(f, "{code}: {name}: {detail}"),
            Self::InvalidEndpoint {
                code,
                source,
                detail,
            } => {
                write!(f, "{code}: {source}: {detail}")
            }
            Self::MissingTcpPort { code, detail } => write!(f, "{code}: {detail}"),
            Self::InstanceRootUnavailable { code, detail } => write!(f, "{code}: {detail}"),
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
            } => {
                write!(f, "{code}: {endpoint}: {cause}")
            }
            Self::ResponseDecode {
                code,
                status,
                cause,
                body_preview,
            } => {
                if body_preview.is_empty() {
                    write!(f, "{code}: http {status}: {cause}")
                } else {
                    write!(f, "{code}: http {status}: {cause}: {body_preview}")
                }
            }
        }
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
        assert!(err.to_string().contains("tcp_port"), "{err}");
        assert!(Error::source(&err).is_none());
    }
}
