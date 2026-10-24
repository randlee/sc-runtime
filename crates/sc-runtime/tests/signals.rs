//! Process-signal proofs. They start a daemon only when the isolated-CI marker is set.

mod common;

#[cfg(unix)]
use std::path::PathBuf;
#[cfg(unix)]
use std::process::{Child, Command, ExitStatus};
#[cfg(unix)]
use std::time::Duration;

#[cfg(unix)]
use sc_runtime::DaemonConfig;
#[cfg(unix)]
use sc_runtime::testing::DaemonFixture;
#[cfg(unix)]
use utoipa_axum::router::OpenApiRouter;
#[cfg(unix)]
use utoipa_axum::routes;

const CHILD: &str = "SC_RUNTIME_SIGNAL_CHILD";

#[cfg(unix)]
const EPHEMERAL: &str = "SC_RUNTIME_EPHEMERAL_CI";

#[cfg(unix)]
fn refuse_without_isolation() -> bool {
    if std::env::var_os(EPHEMERAL).is_some() {
        false
    } else {
        eprintln!("{EPHEMERAL} is unset; signal test not run");
        true
    }
}

#[cfg(unix)]
#[utoipa::path(get, path = "/slow", responses((status = 200, description = "slow")))]
async fn slow() -> &'static str {
    #[cfg(unix)]
    {
        hold_until_release().await;
    }
    "done"
}

/// Tells the parent the request is in flight, then waits for `release`.
///
/// The wait is one read on the sync socket, so the handler stays inside the
/// request until the parent has delivered the signal.
#[cfg(unix)]
async fn hold_until_release() {
    let path = std::env::var(common::SYNC_SOCK_ENV).expect("sync sock");
    let mut stream = tokio::net::UnixStream::connect(&path)
        .await
        .expect("inflight connect");
    tokio::io::AsyncWriteExt::write_all(&mut stream, b"inflight\n")
        .await
        .expect("inflight");
    tokio::io::AsyncWriteExt::flush(&mut stream)
        .await
        .expect("flush inflight");
    let mut reader = tokio::io::BufReader::new(stream);
    let mut line = String::new();
    let read = tokio::time::timeout(
        Duration::from_secs(10),
        tokio::io::AsyncBufReadExt::read_line(&mut reader, &mut line),
    )
    .await;
    let n = read.expect("release deadline").expect("read release");
    assert!(
        n > 0 && line.starts_with("release"),
        "release line {line:?}"
    );
}

#[cfg(unix)]
fn slow_router(_stores: ()) -> OpenApiRouter {
    OpenApiRouter::new().routes(routes!(slow))
}

/// Non-async child entry. Re-exec must not nest a Tokio runtime inside `#[tokio::test]`.
#[test]
fn child_daemon_main() {
    if std::env::var_os(CHILD).is_none() {
        return;
    }
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("child runtime");
    let ok = runtime.block_on(serve_child());
    std::process::exit(if ok { 0 } else { 1 });
}

#[tokio::test]
#[cfg(unix)]
async fn sigterm_drains_and_releases_the_lock() {
    drive_signal("TERM").await;
}

#[tokio::test]
#[cfg(unix)]
async fn sigint_drains_and_releases_the_lock() {
    drive_signal("INT").await;
}

async fn serve_child() -> bool {
    #[cfg(not(unix))]
    {
        false
    }
    #[cfg(unix)]
    {
        let root = PathBuf::from(std::env::var("SC_RUNTIME_SIGNAL_DIR").expect("dir"));
        let endpoint = std::env::var("SC_RUNTIME_SIGNAL_ENDPOINT").expect("endpoint");
        let sync_sock = PathBuf::from(std::env::var(common::SYNC_SOCK_ENV).expect("sync sock"));
        let endpoint_for_run = endpoint.clone();
        let serve = tokio::spawn(async move {
            let mut config = DaemonConfig::default();
            config.transport.instance_root = Some(root);
            config.endpoint_flag = Some(endpoint_for_run);
            sc_runtime::Daemon::builder(&common::app(), &config)
                .stores(|| async { Ok(()) })
                .routes(slow_router)
                .run()
                .await
        });
        common::announce_serving(&endpoint, &sync_sock).await;
        matches!(serve.await, Ok(Ok(())))
    }
}

#[cfg(unix)]
struct ChildGuard {
    child: Option<Child>,
}

#[cfg(unix)]
impl ChildGuard {
    fn spawn(dir: &std::path::Path, endpoint: &str, sync_sock: &std::path::Path) -> Self {
        let mut command = Command::new(std::env::current_exe().expect("exe"));
        command
            .arg("--exact")
            .arg("child_daemon_main")
            .arg("--nocapture")
            .env(EPHEMERAL, "1")
            .env(CHILD, "1")
            .env("SC_RUNTIME_SIGNAL_DIR", dir)
            .env("SC_RUNTIME_SIGNAL_ENDPOINT", endpoint)
            .env(common::SYNC_SOCK_ENV, sync_sock);
        Self {
            child: Some(command.spawn().expect("spawn")),
        }
    }

    fn signal(&self, signal: &str) {
        let child = self.child.as_ref().expect("child still owned");
        let status = Command::new("kill")
            .args([format!("-{signal}"), child.id().to_string()])
            .status()
            .expect("kill");
        assert!(status.success(), "kill -{signal}");
    }

    async fn wait_or_kill(&mut self, timeout: Duration) -> ExitStatus {
        let mut child = self.child.take().expect("child still owned");
        let pid = child.id();
        let (tx, rx) = std::sync::mpsc::channel();
        let waiter = std::thread::spawn(move || {
            let waited = child.wait();
            let _ = tx.send(waited);
        });
        match rx.recv_timeout(timeout) {
            Ok(Ok(status)) => {
                let _ = waiter.join();
                status
            }
            Ok(Err(err)) => panic!("wait for child: {err}"),
            Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
                panic!("child wait disconnected");
            }
            Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                let _ = Command::new("kill")
                    .args(["-KILL", &pid.to_string()])
                    .status();
                match rx.recv_timeout(Duration::from_secs(2)) {
                    Ok(Ok(status)) => {
                        let _ = waiter.join();
                        panic!("child {pid} did not exit within {timeout:?}; killed ({status})");
                    }
                    other => panic!(
                        "child {pid} did not exit within {timeout:?}; kill follow-up {other:?}"
                    ),
                }
            }
        }
    }

    fn try_wait(&mut self) -> Option<ExitStatus> {
        let child = self.child.as_mut()?;
        match child.try_wait() {
            Ok(Some(status)) => {
                self.child.take();
                Some(status)
            }
            Ok(None) => None,
            Err(err) => panic!("wait for child: {err}"),
        }
    }

    fn kill_and_reap(&mut self) {
        if let Some(mut child) = self.child.take() {
            let _ = child.kill();
            let _ = child.wait();
        }
    }
}

#[cfg(unix)]
impl Drop for ChildGuard {
    fn drop(&mut self) {
        self.kill_and_reap();
    }
}

#[cfg(unix)]
async fn drive_signal(signal: &str) {
    if refuse_without_isolation() {
        return;
    }
    let dir = tempfile::tempdir().expect("dir");
    let sock = dir.path().join("daemon.sock");
    let endpoint = format!("uds:{}", sock.display());
    let mut sync = common::ChildSync::bind(dir.path());
    let mut child = ChildGuard::spawn(dir.path(), &endpoint, sync.path());
    expect_sync(
        &mut sync,
        &mut child,
        "ready",
        &endpoint,
        "child daemon did not answer /health",
    )
    .await;
    let client = http_client(&endpoint);
    let request = tokio::spawn({
        let endpoint = endpoint.clone();
        async move {
            client
                .get(request_url(&endpoint))
                .send()
                .await
                .expect("slow request")
                .text()
                .await
                .expect("body")
        }
    });
    let mut inflight = expect_sync(
        &mut sync,
        &mut child,
        "inflight",
        &endpoint,
        "request never reached the handler",
    )
    .await;
    child.signal(signal);
    tokio::io::AsyncWriteExt::write_all(&mut inflight, b"release\n")
        .await
        .expect("release");
    tokio::io::AsyncWriteExt::flush(&mut inflight)
        .await
        .expect("flush release");
    let body = request.await.expect("request task");
    assert_eq!(body, "done");
    let finished = child.wait_or_kill(Duration::from_secs(10)).await;
    assert!(finished.success(), "child status {finished}");
    assert!(!sock.exists(), "socket file remains");
    let later = sc_transport::Client::new(endpoint.parse().expect("endpoint"))
        .expect("client")
        .get::<String>("/slow")
        .await;
    assert!(later.is_err(), "listener still accepts");
    let mut restarted = DaemonConfig::default();
    restarted.transport.instance_root = Some(dir.path().to_path_buf());
    restarted.endpoint_flag = Some(format!("uds:{}", sock.display()));
    let fixture = DaemonFixture::start(&common::app(), restarted, || async { Ok(()) }, slow_router)
        .await
        .expect("restart");
    fixture.shutdown().await.expect("restart shutdown");
}

#[cfg(unix)]
fn http_client(endpoint: &str) -> reqwest::Client {
    let parsed: sc_transport::Endpoint = endpoint.parse().expect("endpoint");
    common::http(&parsed)
}

#[cfg(unix)]
async fn expect_sync(
    sync: &mut common::ChildSync,
    child: &mut ChildGuard,
    expected: &str,
    endpoint: &str,
    failure: &str,
) -> tokio::net::UnixStream {
    match sync.expect_line(expected, common::CHILD_SYNC_BUDGET).await {
        Ok(stream) => stream,
        Err(err) => {
            if let Some(status) = child.try_wait() {
                panic!("{failure} at {endpoint}: child exited {status} ({err})");
            }
            child.kill_and_reap();
            panic!("{failure} at {endpoint} ({err})");
        }
    }
}

#[cfg(unix)]
fn request_url(endpoint: &str) -> String {
    let parsed: sc_transport::Endpoint = endpoint.parse().expect("endpoint");
    common::url(&parsed, "/slow")
}
