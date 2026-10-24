use sc_runtime::DaemonConfig;
use sc_transport::{AppName, Endpoint};

#[allow(dead_code)]
pub fn app() -> AppName {
    AppName::new("sc-runtime-test").expect("app name")
}

#[allow(dead_code)]
pub fn config() -> DaemonConfig {
    #[cfg(windows)]
    {
        let mut config = DaemonConfig::default();
        config.endpoint_flag = Some(format!("tcp:{}", ephemeral_tcp()));
        config
    }
    #[cfg(not(windows))]
    {
        DaemonConfig::default()
    }
}

#[cfg(windows)]
fn ephemeral_tcp() -> std::net::SocketAddr {
    std::net::SocketAddr::from((std::net::Ipv4Addr::LOCALHOST, 0))
}

#[allow(dead_code)]
pub fn http(endpoint: &Endpoint) -> reqwest::Client {
    let builder = reqwest::Client::builder();
    #[cfg(unix)]
    let builder = if let Endpoint::Uds(path) = endpoint {
        builder.unix_socket(path.clone())
    } else {
        builder
    };
    #[cfg(not(unix))]
    let builder = {
        let _ = endpoint;
        builder
    };
    builder.build().expect("client")
}

#[allow(dead_code)]
pub fn url(endpoint: &Endpoint, path: &str) -> String {
    match endpoint {
        Endpoint::Tcp(addr) => format!("http://{addr}{path}"),
        Endpoint::Uds(_) => format!("http://localhost{path}"),
        _ => format!("http://localhost{path}"),
    }
}

/// Environment variable that carries the child-to-parent sync socket path.
#[cfg(unix)]
pub const SYNC_SOCK_ENV: &str = "SC_RUNTIME_SYNC_SOCK";

/// How long a parent waits for one child sync message.
///
/// This covers process start plus the child's own serving probe.
#[cfg(unix)]
pub const CHILD_SYNC_BUDGET: std::time::Duration = std::time::Duration::from_secs(30);

/// How long the child probes `GET /health` before reporting failure.
#[cfg(unix)]
const SERVING_PROBE: std::time::Duration = std::time::Duration::from_secs(15);

/// Parent end of a Unix-socket handshake with a re-exec child.
///
/// The child connects and writes one line after it is serving. Parents block
/// on [`ChildSync::expect_line`] instead of polling HTTP.
#[cfg(unix)]
pub struct ChildSync {
    listener: tokio::net::UnixListener,
    path: std::path::PathBuf,
}

#[cfg(unix)]
impl ChildSync {
    /// Binds a fresh socket under `dir`.
    ///
    /// # Panics
    ///
    /// Panics when the socket cannot be bound or switched to nonblocking mode.
    pub fn bind(dir: &std::path::Path) -> Self {
        let path = dir.join("child-sync.sock");
        let listener = std::os::unix::net::UnixListener::bind(&path)
            .unwrap_or_else(|err| panic!("bind child sync socket: {err}"));
        listener
            .set_nonblocking(true)
            .expect("child sync socket nonblocking");
        let listener =
            tokio::net::UnixListener::from_std(listener).expect("tokio child sync listener");
        Self { listener, path }
    }

    /// Path passed to the child through [`SYNC_SOCK_ENV`].
    #[must_use]
    pub fn path(&self) -> &std::path::Path {
        &self.path
    }

    /// Waits until the child writes `expected` as a single line.
    ///
    /// # Errors
    ///
    /// Returns a message when the deadline expires or the socket fails. The
    /// returned stream is the child's connection, so the caller can write a
    /// reply on it.
    pub async fn expect_line(
        &mut self,
        expected: &str,
        budget: std::time::Duration,
    ) -> Result<tokio::net::UnixStream, String> {
        let wait = async {
            let (stream, _) = self
                .listener
                .accept()
                .await
                .map_err(|err| format!("accept child sync: {err}"))?;
            let mut reader = tokio::io::BufReader::new(stream);
            let mut line = String::new();
            tokio::io::AsyncBufReadExt::read_line(&mut reader, &mut line)
                .await
                .map_err(|err| format!("read child sync: {err}"))?;
            Ok::<_, String>((reader.into_inner(), line))
        };
        let (stream, line) = tokio::time::timeout(budget, wait)
            .await
            .map_err(|_| format!("child did not report {expected} within {budget:?}"))??;
        let got = line.trim_end();
        if got != expected {
            return Err(format!("child reported {got:?}, expected {expected}"));
        }
        Ok(stream)
    }
}

/// Probes `GET /health` on the child runtime, then writes `ready` to `sync_sock`.
///
/// # Panics
///
/// Panics when the probe misses its deadline or the sync socket cannot be written.
#[cfg(unix)]
pub async fn announce_serving(endpoint: &str, sync_sock: &std::path::Path) {
    let parsed: Endpoint = endpoint.parse().expect("endpoint");
    let client = http(&parsed);
    let health = url(&parsed, "/health");
    let deadline = tokio::time::Instant::now() + SERVING_PROBE;
    loop {
        if let Ok(response) = client.get(&health).send().await
            && response.status().is_success()
        {
            write_sync_line(sync_sock, "ready").await;
            return;
        }
        if tokio::time::Instant::now() >= deadline {
            write_sync_line(sync_sock, "failed").await;
            panic!("child daemon did not answer /health within {SERVING_PROBE:?} at {endpoint}");
        }
        tokio::task::yield_now().await;
    }
}

#[cfg(unix)]
async fn write_sync_line(path: &std::path::Path, line: &str) {
    let mut stream = tokio::net::UnixStream::connect(path)
        .await
        .unwrap_or_else(|err| panic!("connect child sync socket {}: {err}", path.display()));
    use tokio::io::AsyncWriteExt;
    stream
        .write_all(format!("{line}\n").as_bytes())
        .await
        .unwrap_or_else(|err| panic!("write child sync: {err}"));
    stream
        .flush()
        .await
        .unwrap_or_else(|err| panic!("flush child sync: {err}"));
}
