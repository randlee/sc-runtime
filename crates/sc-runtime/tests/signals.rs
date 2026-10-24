//! Process-signal proofs. They start a daemon only when the isolated-CI marker is set.

mod common;

use std::path::PathBuf;
use std::process::{Child, Command, ExitStatus};
use std::time::Duration;

use sc_runtime::DaemonConfig;
use sc_runtime::testing::DaemonFixture;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

const EPHEMERAL: &str = "SC_RUNTIME_EPHEMERAL_CI";
const CHILD: &str = "SC_RUNTIME_SIGNAL_CHILD";

fn refuse_without_isolation() -> bool {
    if std::env::var_os(EPHEMERAL).is_some() {
        return false;
    }
    eprintln!("{EPHEMERAL} is unset; signal test not run");
    true
}

#[utoipa::path(get, path = "/slow", responses((status = 200, description = "slow")))]
async fn slow() -> &'static str {
    let root = std::env::var("SC_RUNTIME_SIGNAL_DIR").expect("signal dir");
    let dir = PathBuf::from(root);
    std::fs::write(dir.join("inflight"), b"1").expect("inflight");
    let release = dir.join("release");
    for _ in 0..100 {
        if release.exists() {
            return "done";
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
    "done"
}

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
    let mut config = DaemonConfig::default();
    let root = PathBuf::from(std::env::var("SC_RUNTIME_SIGNAL_DIR").expect("dir"));
    config.transport.instance_root = Some(root);
    let endpoint = std::env::var("SC_RUNTIME_SIGNAL_ENDPOINT").expect("endpoint");
    config.endpoint_flag = Some(endpoint);
    sc_runtime::Daemon::builder(&common::app(), &config)
        .stores(|| async { Ok(()) })
        .routes(slow_router)
        .run()
        .await
        .is_ok()
}

struct ChildGuard {
    child: Option<Child>,
}

impl ChildGuard {
    fn spawn(dir: &std::path::Path, endpoint: &str) -> Self {
        let mut command = Command::new(std::env::current_exe().expect("exe"));
        command
            .arg("--exact")
            .arg("child_daemon_main")
            .arg("--nocapture")
            .env(EPHEMERAL, "1")
            .env(CHILD, "1")
            .env("SC_RUNTIME_SIGNAL_DIR", dir)
            .env("SC_RUNTIME_SIGNAL_ENDPOINT", endpoint);
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
        let deadline = tokio::time::Instant::now() + timeout;
        loop {
            if let Some(status) = self.try_wait() {
                return status;
            }
            if tokio::time::Instant::now() >= deadline {
                self.kill_and_reap();
                panic!("child did not exit within {timeout:?}");
            }
            tokio::time::sleep(Duration::from_millis(50)).await;
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

impl Drop for ChildGuard {
    fn drop(&mut self) {
        self.kill_and_reap();
    }
}

async fn drive_signal(signal: &str) {
    if refuse_without_isolation() {
        return;
    }
    let dir = tempfile::tempdir().expect("dir");
    let sock = dir.path().join("daemon.sock");
    let endpoint = format!("uds:{}", sock.display());
    let mut child = ChildGuard::spawn(dir.path(), &endpoint);
    let inflight = dir.path().join("inflight");
    let client = wait_for_listener(&endpoint, &mut child).await;
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
    for _ in 0..100 {
        if inflight.exists() {
            break;
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
    assert!(inflight.exists(), "request never reached the handler");
    child.signal(signal);
    std::fs::write(dir.path().join("release"), b"1").expect("release");
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

async fn wait_for_listener(endpoint: &str, child: &mut ChildGuard) -> reqwest::Client {
    let parsed: sc_transport::Endpoint = endpoint.parse().expect("endpoint");
    let client = common::http(&parsed);
    let health = common::url(&parsed, "/health");
    for _ in 0..100 {
        if child.try_wait().is_some() {
            panic!("daemon exited before it became healthy at {endpoint}");
        }
        if client.get(&health).send().await.is_ok() {
            return client;
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
    child.kill_and_reap();
    panic!("child daemon did not answer /health");
}

fn request_url(endpoint: &str) -> String {
    let parsed: sc_transport::Endpoint = endpoint.parse().expect("endpoint");
    common::url(&parsed, "/slow")
}
