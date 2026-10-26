//! Listener binding behind the `server` feature.
//!
//! `bind` acquires an endpoint lock for a UDS path, replaces a stale socket,
//! and returns a [`Listener`] that `axum::serve` accepts. TCP binding does not
//! take that lock.

use std::fs::File;
use std::io;
use std::path::{Path, PathBuf};
use std::pin::Pin;
use std::task::{Context, Poll};

use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};
use tokio::net::{TcpListener, TcpStream};

use crate::endpoint::Endpoint;
use crate::error::TransportError;

/// Binds one listener for `endpoint`.
///
/// v0.1 configuration names one endpoint. This function binds that one
/// listener. A caller may invoke it twice for two different endpoints.
///
/// For a UDS path `S`, the endpoint lock is the sibling path formed by
/// appending the literal suffix `.lock` to the full socket path (`S.lock`,
/// not a replaced extension). `bind` opens that file and takes a nonblocking
/// exclusive lock before it inspects or removes `S`. The returned [`Listener`]
/// retains the lock until it is dropped. A competing bind that cannot acquire
/// the lock returns [`TransportError::EndpointBusy`] and does not unlink,
/// replace, or chmod `S`.
///
/// The lock, metadata check, stale-socket removal, `UnixListener` bind, and
/// permission change run inside [`tokio::task::spawn_blocking`] so the async
/// caller does not block the runtime on filesystem or flock I/O.
///
/// Once the lock is held, an existing socket file is removed and a new socket
/// is bound at `S`. Liveness is not inferred with a connect probe or a process
/// check. A path that exists and is not a socket is left in place and returned
/// as [`TransportError::NotASocket`]. The socket file mode is `0o600`
/// (`mode & 0o077 == 0`). The parent directory is not created; a missing
/// directory is [`TransportError::Bind`].
///
/// Dropping the listener removes `S` only while that same listener still holds
/// `S.lock`. The lock file is never deleted. Releasing the OS lock is enough.
/// Drop itself cannot await, so the final unlink stays on the drop path while
/// the lock is still held.
///
/// A TCP endpoint binds the given address. Port `0` lets the caller read the
/// actual address from [`Listener::local_endpoint`].
///
/// # Errors
///
/// Returns [`TransportError::Bind`] when the address is in use, the parent
/// directory is missing, or lock I/O fails. Returns
/// [`TransportError::EndpointBusy`] when the endpoint lock is held. Returns
/// [`TransportError::NotASocket`] when the UDS path is not a socket. On
/// non-Unix targets a UDS endpoint returns [`TransportError::UnsupportedEndpoint`]
/// and the recovery text names a `tcp:` endpoint.
///
/// # Examples
///
/// ```
/// # #[tokio::main]
/// # async fn main() -> Result<(), sc_transport::TransportError> {
/// let endpoint = "tcp:127.0.0.1:0".parse()?;
/// let listener = sc_transport::bind(&endpoint).await?;
/// let bound = listener.local_endpoint()?;
/// drop(listener);
/// # let _ = bound;
/// # Ok(())
/// # }
/// ```
pub async fn bind(endpoint: &Endpoint) -> Result<Listener, TransportError> {
    match endpoint {
        Endpoint::Tcp(addr) => bind_tcp(*addr).await,
        Endpoint::Uds(path) => bind_uds(path).await,
    }
}

/// A bound listener accepted by `axum::serve`.
///
/// The router does not change between UDS and TCP. Read the bound address with
/// [`Listener::local_endpoint`]. The [`axum::serve::Listener`] associated types
/// are opaque and are not part of the crate-root facade.
///
/// For a UDS listener, this value retains the endpoint lock described on
/// [`bind`] for its whole lifetime. Drop removes the socket file and then
/// releases the lock. The lock file stays on disk.
///
/// [`Listener::accept`](axum::serve::Listener::accept) retries a connection
/// reset, abort, refusal, interrupt, or `WouldBlock`. Any other accept error
/// is fatal: the inner socket is closed and `accept` does not return, so the
/// caller must drop the serve task to release this listener. Fatal errors are
/// not retried.
#[derive(Debug)]
pub struct Listener {
    inner: ListenerInner,
    endpoint: Endpoint,
    socket_path: Option<PathBuf>,
    endpoint_lock: Option<File>,
    fatal_accept: Option<String>,
}

impl Listener {
    /// Returns the bound endpoint.
    ///
    /// For TCP, this is the address after binding, including an ephemeral port
    /// chosen for port `0`. For UDS, this is the path that was bound.
    ///
    /// # Errors
    ///
    /// Returns [`TransportError::Bind`] when the TCP listener cannot report
    /// its address, or when a fatal accept error has stopped the listener.
    pub fn local_endpoint(&self) -> Result<Endpoint, TransportError> {
        if let Some(detail) = &self.fatal_accept {
            return Err(TransportError::bind(self.endpoint.clone(), detail.clone()));
        }
        match &self.inner {
            ListenerInner::Closed => Err(TransportError::bind(
                self.endpoint.clone(),
                "listener is closed",
            )),
            ListenerInner::Tcp(listener) => {
                let addr = TcpListener::local_addr(listener)
                    .map_err(|err| TransportError::bind(self.endpoint.clone(), err.to_string()))?;
                Ok(Endpoint::Tcp(addr))
            }
            #[cfg(unix)]
            ListenerInner::Unix(_) => Ok(self.endpoint.clone()),
        }
    }
}

impl Drop for Listener {
    fn drop(&mut self) {
        // Remove the socket while the endpoint lock is still held. The lock
        // file itself is never deleted; dropping the file releases the lock.
        // Drop cannot await, so this unlink stays synchronous.
        if let Some(path) = self.socket_path.take() {
            let _ = std::fs::remove_file(path);
        }
        drop(self.endpoint_lock.take());
    }
}

/// Opaque byte stream accepted from a [`Listener`].
///
/// Required as `axum::serve::Listener::Io`. Not a crate-root facade item.
/// Callers use [`Listener::local_endpoint`] and `axum::serve`.
#[doc(hidden)]
#[non_exhaustive]
pub struct ListenerIo {
    inner: IoInner,
}

/// Opaque address accepted from a [`Listener`].
///
/// Required as `axum::serve::Listener::Addr`. Prefer
/// [`Listener::local_endpoint`] for the bound [`Endpoint`].
#[doc(hidden)]
#[non_exhaustive]
pub struct ListenerAddr {
    inner: AddrInner,
}

impl std::fmt::Debug for ListenerIo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("ListenerIo")
    }
}

impl std::fmt::Debug for ListenerAddr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.inner {
            AddrInner::Tcp(addr) => f.debug_tuple("ListenerAddr").field(addr).finish(),
            #[cfg(unix)]
            AddrInner::Unix(addr) => f.debug_tuple("ListenerAddr").field(addr).finish(),
        }
    }
}

impl axum::serve::Listener for Listener {
    type Io = ListenerIo;
    type Addr = ListenerAddr;

    async fn accept(&mut self) -> (Self::Io, Self::Addr) {
        loop {
            if matches!(self.inner, ListenerInner::Closed) {
                std::future::pending::<(Self::Io, Self::Addr)>().await;
            }
            match self.accept_once().await {
                Ok(pair) => return pair,
                Err(err) => match accept_class(&err) {
                    AcceptClass::Transient => {}
                    AcceptClass::Yield => tokio::task::yield_now().await,
                    AcceptClass::Fatal => {
                        self.fatal_accept = Some(err.to_string());
                        self.inner = ListenerInner::Closed;
                        std::future::pending::<(Self::Io, Self::Addr)>().await;
                    }
                },
            }
        }
    }

    fn local_addr(&self) -> io::Result<Self::Addr> {
        match &self.inner {
            ListenerInner::Closed => Err(io::Error::new(
                io::ErrorKind::NotConnected,
                "listener stopped after a fatal accept error",
            )),
            ListenerInner::Tcp(listener) => Ok(ListenerAddr {
                inner: AddrInner::Tcp(listener.local_addr()?),
            }),
            #[cfg(unix)]
            ListenerInner::Unix(listener) => Ok(ListenerAddr {
                inner: AddrInner::Unix(listener.local_addr()?),
            }),
        }
    }
}

impl Listener {
    async fn accept_once(&mut self) -> io::Result<(ListenerIo, ListenerAddr)> {
        match &mut self.inner {
            ListenerInner::Closed => Err(io::Error::new(
                io::ErrorKind::NotConnected,
                "listener stopped after a fatal accept error",
            )),
            ListenerInner::Tcp(listener) => {
                let (stream, addr) = TcpListener::accept(listener).await?;
                Ok((
                    ListenerIo {
                        inner: IoInner::Tcp(stream),
                    },
                    ListenerAddr {
                        inner: AddrInner::Tcp(addr),
                    },
                ))
            }
            #[cfg(unix)]
            ListenerInner::Unix(listener) => {
                let (stream, addr) = tokio::net::UnixListener::accept(listener).await?;
                Ok((
                    ListenerIo {
                        inner: IoInner::Unix(stream),
                    },
                    ListenerAddr {
                        inner: AddrInner::Unix(addr),
                    },
                ))
            }
        }
    }
}

impl AsyncRead for ListenerIo {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        match &mut self.get_mut().inner {
            IoInner::Tcp(stream) => Pin::new(stream).poll_read(cx, buf),
            #[cfg(unix)]
            IoInner::Unix(stream) => Pin::new(stream).poll_read(cx, buf),
        }
    }
}

impl AsyncWrite for ListenerIo {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        match &mut self.get_mut().inner {
            IoInner::Tcp(stream) => Pin::new(stream).poll_write(cx, buf),
            #[cfg(unix)]
            IoInner::Unix(stream) => Pin::new(stream).poll_write(cx, buf),
        }
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        match &mut self.get_mut().inner {
            IoInner::Tcp(stream) => Pin::new(stream).poll_flush(cx),
            #[cfg(unix)]
            IoInner::Unix(stream) => Pin::new(stream).poll_flush(cx),
        }
    }

    fn poll_shutdown(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        match &mut self.get_mut().inner {
            IoInner::Tcp(stream) => Pin::new(stream).poll_shutdown(cx),
            #[cfg(unix)]
            IoInner::Unix(stream) => Pin::new(stream).poll_shutdown(cx),
        }
    }
}

#[derive(Debug)]
enum IoInner {
    Tcp(TcpStream),
    #[cfg(unix)]
    Unix(tokio::net::UnixStream),
}

enum AddrInner {
    Tcp(std::net::SocketAddr),
    #[cfg(unix)]
    Unix(tokio::net::unix::SocketAddr),
}

#[derive(Debug)]
enum ListenerInner {
    Tcp(TcpListener),
    #[cfg(unix)]
    Unix(tokio::net::UnixListener),
    Closed,
}

enum AcceptClass {
    Transient,
    Yield,
    Fatal,
}

fn accept_class(err: &io::Error) -> AcceptClass {
    match err.kind() {
        io::ErrorKind::ConnectionRefused
        | io::ErrorKind::ConnectionAborted
        | io::ErrorKind::ConnectionReset
        | io::ErrorKind::Interrupted => AcceptClass::Transient,
        io::ErrorKind::WouldBlock => AcceptClass::Yield,
        _ => AcceptClass::Fatal,
    }
}

async fn bind_tcp(addr: std::net::SocketAddr) -> Result<Listener, TransportError> {
    let endpoint = Endpoint::Tcp(addr);
    let listener = TcpListener::bind(addr)
        .await
        .map_err(|err| TransportError::bind(endpoint.clone(), err.to_string()))?;
    Ok(Listener {
        inner: ListenerInner::Tcp(listener),
        endpoint,
        socket_path: None,
        endpoint_lock: None,
        fatal_accept: None,
    })
}

async fn bind_uds(path: &Path) -> Result<Listener, TransportError> {
    #[cfg(unix)]
    {
        let path = path.to_path_buf();
        let endpoint = Endpoint::Uds(path.clone());
        match tokio::task::spawn_blocking(move || bind_uds_blocking(path)).await {
            Ok(result) => result,
            Err(err) => Err(TransportError::bind(
                endpoint,
                format!("blocking bind task failed: {err}"),
            )),
        }
    }
    #[cfg(windows)]
    {
        bind_uds_blocking(path.to_path_buf())
    }
    #[cfg(not(any(unix, windows)))]
    {
        Err(TransportError::unsupported_endpoint(
            Endpoint::Uds(path.to_path_buf()),
            "UDS endpoints are not supported on this platform",
        ))
    }
}

#[cfg(unix)]
fn bind_uds_blocking(path: PathBuf) -> Result<Listener, TransportError> {
    use std::os::unix::fs::{FileTypeExt, PermissionsExt};

    let endpoint = Endpoint::Uds(path.clone());
    let lock_path = endpoint_lock_path(&path);
    let lock = acquire_endpoint_lock(&endpoint, &lock_path)?;
    if path.exists() {
        let metadata = std::fs::metadata(&path)
            .map_err(|err| TransportError::bind(endpoint.clone(), err.to_string()))?;
        if !metadata.file_type().is_socket() {
            return Err(TransportError::not_a_socket(
                endpoint,
                "path exists and is not a socket",
            ));
        }
        std::fs::remove_file(&path)
            .map_err(|err| TransportError::bind(endpoint.clone(), err.to_string()))?;
    }
    let listener = match tokio::net::UnixListener::bind(&path) {
        Ok(listener) => listener,
        Err(err) => return Err(TransportError::bind(endpoint, err.to_string())),
    };
    let mut permissions = std::fs::metadata(&path)
        .map_err(|err| TransportError::bind(endpoint.clone(), err.to_string()))?
        .permissions();
    permissions.set_mode(0o600);
    std::fs::set_permissions(&path, permissions)
        .map_err(|err| TransportError::bind(endpoint.clone(), err.to_string()))?;
    Ok(Listener {
        inner: ListenerInner::Unix(listener),
        endpoint,
        socket_path: Some(path),
        endpoint_lock: Some(lock),
        fatal_accept: None,
    })
}

fn endpoint_lock_path(socket: &Path) -> PathBuf {
    let mut name = socket.as_os_str().to_owned();
    name.push(".lock");
    PathBuf::from(name)
}

#[cfg(windows)]
fn bind_uds_blocking(path: PathBuf) -> Result<Listener, TransportError> {
    let endpoint = Endpoint::Uds(path.clone());
    Err(acquire_endpoint_lock(&endpoint, &endpoint_lock_path(&path)))
}

#[cfg(windows)]
fn acquire_endpoint_lock(endpoint: &Endpoint, lock_path: &Path) -> TransportError {
    TransportError::unsupported_endpoint(
        endpoint.clone(),
        format!(
            "UDS endpoints are not supported on this platform ({})",
            lock_path.display()
        ),
    )
}

#[cfg(unix)]
fn acquire_endpoint_lock(endpoint: &Endpoint, lock_path: &Path) -> Result<File, TransportError> {
    use std::fs::TryLockError;
    use std::os::unix::fs::PermissionsExt;

    let file = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(lock_path)
        .map_err(|err| TransportError::bind(endpoint.clone(), err.to_string()))?;
    let mut permissions = file
        .metadata()
        .map_err(|err| TransportError::bind(endpoint.clone(), err.to_string()))?
        .permissions();
    permissions.set_mode(0o600);
    std::fs::set_permissions(lock_path, permissions)
        .map_err(|err| TransportError::bind(endpoint.clone(), err.to_string()))?;
    match file.try_lock() {
        Ok(()) => Ok(file),
        Err(TryLockError::WouldBlock) => Err(TransportError::endpoint_busy(
            endpoint.clone(),
            format!("endpoint lock held at {}", lock_path.display()),
        )),
        Err(TryLockError::Error(err)) => {
            Err(TransportError::bind(endpoint.clone(), err.to_string()))
        }
    }
}

#[cfg(test)]
mod tests {
    use std::io;

    use super::{AcceptClass, accept_class};

    #[test]
    fn connection_reset_is_transient() {
        let err = io::Error::new(io::ErrorKind::ConnectionReset, "reset");
        assert!(matches!(accept_class(&err), AcceptClass::Transient));
    }

    #[test]
    fn would_block_yields_instead_of_stopping() {
        let err = io::Error::new(io::ErrorKind::WouldBlock, "again");
        assert!(matches!(accept_class(&err), AcceptClass::Yield));
    }

    #[test]
    fn permission_denied_is_fatal() {
        let err = io::Error::new(io::ErrorKind::PermissionDenied, "denied");
        assert!(matches!(accept_class(&err), AcceptClass::Fatal));
    }

    #[cfg(unix)]
    #[test]
    fn too_many_open_files_is_fatal() {
        let err = io::Error::from_raw_os_error(24);
        assert!(
            matches!(accept_class(&err), AcceptClass::Fatal),
            "{err:?} kind {:?}",
            err.kind()
        );
    }
}
