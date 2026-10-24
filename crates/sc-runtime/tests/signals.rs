//! Process-signal proofs. They start a daemon only when the isolated-CI marker is set.

mod common;

use std::path::PathBuf;
use std::time::Duration;

use sc_runtime::DaemonConfig;
use sc_runtime::testing::DaemonFixture;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

const ISOLATED: &str = "SC_RUNTIME_ISOLATED_CI";
const CHILD: &str = "SC_RUNTIME_SIGNAL_CHILD";

fn refuse_without_isolation() -> bool {
    if std::env::var_os(ISOLATED).is_some() {
        return false;
    }
    eprintln!("{ISOLATED} is unset; signal test not run");
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

#[tokio::test]
#[cfg(unix)]
async fn signal_child() {
    if std::env::var_os(CHILD).is_none() {
        eprintln!("{CHILD} is unset; signal child not run");
        return;
    }
    let mut config = DaemonConfig::default();
    let root = PathBuf::from(std::env::var("SC_RUNTIME_SIGNAL_DIR").expect("dir"));
    config.transport.instance_root = Some(root.clone());
    let endpoint = std::env::var("SC_RUNTIME_SIGNAL_ENDPOINT").expect("endpoint");
    config.endpoint_flag = Some(endpoint);
    Daemon::run_child(&config).await;
}

async fn drive_signal(signal: &str) {
    if refuse_without_isolation() {
        return;
    }
    let dir = tempfile::tempdir().expect("dir");
    let sock = dir.path().join("daemon.sock");
    let endpoint = format!("uds:{}", sock.display());
    let mut child = std::process::Command::new(std::env::current_exe().expect("exe"))
        .arg("signal_child")
        .arg("--exact")
        .env(ISOLATED, "1")
        .env(CHILD, "1")
        .env("SC_RUNTIME_SIGNAL_DIR", dir.path())
        .env("SC_RUNTIME_SIGNAL_ENDPOINT", &endpoint)
        .spawn()
        .expect("spawn");
    let inflight = dir.path().join("inflight");
    let client = wait_for_listener(&endpoint).await;
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
    let status = std::process::Command::new("kill")
        .args([format!("-{signal}"), child.id().to_string()])
        .status()
        .expect("kill");
    assert!(status.success(), "kill -{signal}");
    std::fs::write(dir.path().join("release"), b"1").expect("release");
    let body = request.await.expect("request task");
    assert_eq!(body, "done");
    let finished = child.wait().expect("wait");
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

async fn wait_for_listener(endpoint: &str) -> reqwest::Client {
    let parsed: sc_transport::Endpoint = endpoint.parse().expect("endpoint");
    let client = common::http(&parsed);
    let health = common::url(&parsed, "/health");
    for _ in 0..100 {
        if client.get(&health).send().await.is_ok() {
            return client;
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
    panic!("child daemon did not answer /health");
}

fn request_url(endpoint: &str) -> String {
    let parsed: sc_transport::Endpoint = endpoint.parse().expect("endpoint");
    common::url(&parsed, "/slow")
}

struct Daemon;

impl Daemon {
    async fn run_child(config: &DaemonConfig) {
        let result = sc_runtime::Daemon::builder(&common::app(), config)
            .stores(|| async { Ok(()) })
            .routes(slow_router)
            .run()
            .await;
        if let Err(err) = result {
            panic!("child run failed: {err}");
        }
    }
}
