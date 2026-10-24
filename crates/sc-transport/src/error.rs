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

/// Failure returned by every fallible public function in this crate.
///
/// `DaemonNotRunning` carries `code`, the endpoint, and the cause. It has no
/// suggested action, no application name, and no observability error type.
/// A later CLI maps `DAEMON.NOT_RUNNING` and chooses the action text.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum TransportError {
    /// `name` failed the application-name predicate.
    InvalidName {
        /// Always [`CODE_INVALID_NAME`].
        code: &'static str,
        /// The rejected name.
        name: String,
    },
    /// `source` is not valid endpoint text.
    InvalidEndpoint {
        /// Always [`CODE_INVALID_ENDPOINT`].
        code: &'static str,
        /// The text that failed to parse.
        source: String,
        /// Why parsing failed.
        detail: String,
    },
    /// The Windows default needs `TransportConfig::tcp_port` or a `tcp:` override.
    MissingTcpPort {
        /// Always [`CODE_MISSING_TCP_PORT`].
        code: &'static str,
    },
    /// No per-user base directory was available.
    InstanceRootUnavailable {
        /// Always [`CODE_INSTANCE_ROOT`].
        code: &'static str,
        /// Which location was missing.
        detail: String,
    },
    /// The endpoint refused the connection or the UDS path was missing.
    DaemonNotRunning {
        /// Always [`CODE_DAEMON_NOT_RUNNING`].
        code: &'static str,
        /// The endpoint the client tried to open.
        endpoint: Endpoint,
        /// The I/O failure from the connection attempt.
        cause: String,
    },
    /// The response body could not be deserialized as `T`.
    ResponseDecode {
        /// Always [`CODE_RESPONSE_DECODE`].
        code: &'static str,
        /// HTTP status of the response that failed to decode.
        status: u16,
        /// The decode failure.
        cause: String,
    },
    /// The connect or request timeout elapsed.
    TimedOut {
        /// Always [`CODE_TIMED_OUT`].
        code: &'static str,
        /// The endpoint the client was calling.
        endpoint: Endpoint,
        /// The timeout failure.
        cause: String,
    },
    /// A client failure other than a down daemon, a timeout, or a decode.
    Request {
        /// Always [`CODE_REQUEST`].
        code: &'static str,
        /// The endpoint the client was calling.
        endpoint: Endpoint,
        /// The underlying failure.
        cause: String,
    },
    /// Binding the listener failed.
    Bind {
        /// Always [`CODE_BIND`].
        code: &'static str,
        /// The endpoint that could not be bound.
        endpoint: Endpoint,
        /// The underlying failure.
        cause: String,
    },
    /// The UDS endpoint lock is already held.
    EndpointBusy {
        /// Always [`CODE_ENDPOINT_BUSY`].
        code: &'static str,
        /// The endpoint whose lock is held.
        endpoint: Endpoint,
        /// Why the lock was not acquired.
        cause: String,
    },
    /// The UDS path exists and is not a socket file.
    NotASocket {
        /// Always [`CODE_NOT_A_SOCKET`].
        code: &'static str,
        /// The path that was left unchanged.
        endpoint: Endpoint,
        /// Why the path was rejected.
        cause: String,
    },
}

impl TransportError {
    /// Stable code for this failure.
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

    pub(crate) fn invalid_name(name: impl Into<String>) -> Self {
        Self::InvalidName {
            code: CODE_INVALID_NAME,
            name: name.into(),
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
            cause: cause.into(),
        }
    }

    pub(crate) fn response_decode(status: u16, cause: impl Into<String>) -> Self {
        Self::ResponseDecode {
            code: CODE_RESPONSE_DECODE,
            status,
            cause: cause.into(),
        }
    }

    pub(crate) fn timed_out(endpoint: Endpoint, cause: impl Into<String>) -> Self {
        Self::TimedOut {
            code: CODE_TIMED_OUT,
            endpoint,
            cause: cause.into(),
        }
    }

    pub(crate) fn request(endpoint: Endpoint, cause: impl Into<String>) -> Self {
        Self::Request {
            code: CODE_REQUEST,
            endpoint,
            cause: cause.into(),
        }
    }

    #[cfg(feature = "server")]
    pub(crate) fn bind(endpoint: Endpoint, cause: impl Into<String>) -> Self {
        Self::Bind {
            code: CODE_BIND,
            endpoint,
            cause: cause.into(),
        }
    }

    #[cfg(feature = "server")]
    pub(crate) fn endpoint_busy(endpoint: Endpoint, cause: impl Into<String>) -> Self {
        Self::EndpointBusy {
            code: CODE_ENDPOINT_BUSY,
            endpoint,
            cause: cause.into(),
        }
    }

    #[cfg(feature = "server")]
    pub(crate) fn not_a_socket(endpoint: Endpoint, cause: impl Into<String>) -> Self {
        Self::NotASocket {
            code: CODE_NOT_A_SOCKET,
            endpoint,
            cause: cause.into(),
        }
    }
}

impl fmt::Display for TransportError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidName { code, name } => write!(f, "{code}: {name}"),
            Self::InvalidEndpoint {
                code,
                source,
                detail,
            } => {
                write!(f, "{code}: {source}: {detail}")
            }
            Self::MissingTcpPort { code } => write!(f, "{code}"),
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
            } => {
                write!(f, "{code}: http {status}: {cause}")
            }
        }
    }
}

impl std::error::Error for TransportError {}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;

    #[test]
    fn daemon_not_running_has_code_endpoint_and_cause_only() {
        let endpoint = Endpoint::Uds(PathBuf::from("daemon.sock"));
        let err = TransportError::daemon_not_running(endpoint.clone(), "connection refused");
        match err {
            TransportError::DaemonNotRunning {
                code,
                endpoint: got,
                cause,
            } => {
                assert_eq!(code, CODE_DAEMON_NOT_RUNNING);
                assert_eq!(got, endpoint);
                assert_eq!(cause, "connection refused");
            }
            other => panic!("unexpected {other:?}"),
        }
    }
}
