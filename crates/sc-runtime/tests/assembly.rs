mod common;

use std::ffi::OsStr;
use std::process::{Child, Command, ExitStatus};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use rmcp::ServerHandler;
use rmcp::transport::streamable_http_server::session::local::LocalSessionManager;
use rmcp::transport::streamable_http_server::{StreamableHttpServerConfig, StreamableHttpService};
use sc_runtime::testing::DaemonFixture;
use sc_runtime::{Daemon, DaemonConfig, RuntimeError};
use serde::Deserialize;
use utoipa_axum::router::OpenApiRouter;
use utoipa_axum::routes;

#[derive(Clone)]
struct Demo;

impl ServerHandler for Demo {
    fn get_info(&self) -> rmcp::model::ServerConfig {
        rmcp::model::ServerConfig::new(rmcp::model::ServerCapabilities::builder().build())
    }
}

fn service(_stores: ()) -> StreamableHttpService<Demo, LocalSessionManager> {
    StreamableHttpService::new(
        || Ok(Demo),
        Arc::new(LocalSessionManager::default()),
        StreamableHttpServerConfig::default()
            .with_legacy_session_mode(false)
            .with_json_response(true),
    )
}

#[utoipa::path(get, path = "/project", responses((status = 200, description = "project route")))]
async fn project() -> axum::Json<&'static str> {
    axum::Json("project")
}

fn project_router(_stores: ()) -> OpenApiRouter {
    OpenApiRouter::new().routes(routes!(project))
}

#[derive(Debug, Deserialize)]
struct Envelope {
    version: String,
    ok: bool,
    data: HealthData,
}

#[derive(Debug, Deserialize)]
struct HealthData {
    status: String,
}

#[tokio::test]
async fn one_listener_serves_project_openapi_health_and_mcp() {
    let fixture = DaemonFixture::start_with_mcp(
        &common::app(),
        common::config(),
        || async { Ok(()) },
        project_router,
        service,
    )
    .await
    .expect("start");

    let body: String = fixture.client().get("/project").await.expect("project");
    assert_eq!(body, "project");

    let spec: serde_json::Value = fixture
        .client()
        .get("/openapi.json")
        .await
        .expect("openapi");
    assert!(spec.get("openapi").is_some(), "{spec}");
    let paths = spec
        .get("paths")
        .and_then(|value| value.as_object())
        .expect("paths");
    assert!(paths.contains_key("/project"), "{paths:?}");

    let health: Envelope = fixture.client().get("/health").await.expect("health");
    assert_eq!(health.version, "1");
    assert!(health.ok);
    assert_eq!(health.data.status, "ok");

    let http = common::http(fixture.endpoint());
    let response = http
        .post(common::url(fixture.endpoint(), "/mcp"))
        .header("accept", "application/json, text/event-stream")
        .header("content-type", "application/json")
        .json(&serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {
                "protocolVersion": "2025-03-26",
                "capabilities": {},
                "clientInfo": { "name": "sc-runtime-test", "version": "0.1.0" }
            }
        }))
        .send()
        .await
        .expect("initialize");
    assert!(response.status().is_success(), "{}", response.status());
    let mcp: serde_json::Value = response.json().await.expect("mcp json");
    assert!(mcp.get("result").is_some(), "{mcp}");
    assert!(mcp.get("error").is_none(), "{mcp}");

    fixture.shutdown().await.expect("shutdown");
}

#[tokio::test]
async fn missing_mcp_is_not_mounted() {
    let fixture = DaemonFixture::start(
        &common::app(),
        common::config(),
        || async { Ok(()) },
        project_router,
    )
    .await
    .expect("start");
    let response = common::http(fixture.endpoint())
        .get(common::url(fixture.endpoint(), "/mcp"))
        .send()
        .await
        .expect("mcp");
    assert_eq!(response.status(), reqwest::StatusCode::NOT_FOUND);
    let health: Envelope = fixture.client().get("/health").await.expect("health");
    assert!(health.ok);
    fixture.shutdown().await.expect("shutdown");
}

#[tokio::test]
async fn two_fixtures_are_independent() {
    let first = DaemonFixture::start(
        &common::app(),
        common::config(),
        || async { Ok(()) },
        project_router,
    )
    .await
    .expect("first");
    let second = DaemonFixture::start(
        &common::app(),
        common::config(),
        || async { Ok(()) },
        project_router,
    )
    .await
    .expect("second");
    assert_ne!(first.instance_root(), second.instance_root());
    assert_ne!(first.endpoint_str(), second.endpoint_str());
    let _: String = first.client().get("/project").await.expect("first project");
    let _: String = second
        .client()
        .get("/project")
        .await
        .expect("second project");
    first.shutdown().await.expect("stop first");
    let _: String = second
        .client()
        .get("/project")
        .await
        .expect("second still up");
    second.shutdown().await.expect("stop second");
}

#[tokio::test]
async fn shutdown_closes_the_listener() {
    let fixture = DaemonFixture::start(
        &common::app(),
        common::config(),
        || async { Ok(()) },
        project_router,
    )
    .await
    .expect("start");
    let endpoint = fixture.endpoint().clone();
    let root = fixture.instance_root().to_path_buf();
    fixture.shutdown().await.expect("shutdown");
    let err = sc_transport::Client::new(endpoint.clone())
        .expect("client")
        .get::<String>("/project")
        .await
        .expect_err("down");
    assert!(matches!(
        err,
        sc_transport::TransportError::DaemonNotRunning { .. }
    ));
    if let sc_transport::Endpoint::Uds(path) = endpoint {
        assert!(!path.exists(), "{}", path.display());
    }
    let _ = root;
}

#[cfg(unix)]
#[tokio::test]
async fn second_root_cannot_replace_a_live_socket() {
    let sock_dir = tempfile::tempdir().expect("socket dir");
    let sock = sock_dir.path().join("live.sock");
    let mut config = common::config();
    config.endpoint_flag = Some(format!("uds:{}", sock.display()));
    let fixture = DaemonFixture::start(&common::app(), config, || async { Ok(()) }, project_router)
        .await
        .expect("first");
    let other = tempfile::tempdir().expect("other root");
    let mut second = DaemonConfig::default();
    second.transport.instance_root = Some(other.path().to_path_buf());
    second.endpoint_flag = Some(format!("uds:{}", sock.display()));
    let called = Arc::new(Mutex::new(false));
    let flag = Arc::clone(&called);
    let err = Daemon::builder(&common::app(), &second)
        .stores(move || {
            let flag = Arc::clone(&flag);
            async move {
                *flag.lock().expect("flag") = true;
                Ok(())
            }
        })
        .routes(project_router)
        .run()
        .await
        .expect_err("second");
    assert!(matches!(err, RuntimeError::BindFailed { .. }));
    assert!(sock.exists(), "failed startup removed the live socket");
    let body: String = fixture.client().get("/project").await.expect("still up");
    assert_eq!(body, "project");
    let _ = called;
    fixture.shutdown().await.expect("shutdown");
}

#[cfg(unix)]
#[tokio::test]
async fn explicit_root_defaults_to_daemon_sock() {
    let fixture = DaemonFixture::start(
        &common::app(),
        common::config(),
        || async { Ok(()) },
        project_router,
    )
    .await
    .expect("start");
    let lock = fixture.instance_root().join("daemon.lock");
    let sock = fixture.instance_root().join("daemon.sock");
    assert!(lock.is_file(), "{}", lock.display());
    assert!(sock.exists(), "{}", sock.display());
    let body: String = fixture.client().get("/project").await.expect("project");
    assert_eq!(body, "project");
    let mut second = DaemonConfig::default();
    second.transport.instance_root = Some(fixture.instance_root().to_path_buf());
    let err = Daemon::builder(&common::app(), &second)
        .stores(|| async { Ok(()) })
        .routes(project_router)
        .run()
        .await
        .expect_err("lock");
    assert!(matches!(err, RuntimeError::LockHeld { .. }));
    fixture.shutdown().await.expect("shutdown");
}

#[cfg(unix)]
#[tokio::test]
async fn endpoint_override_skips_the_default_socket() {
    let sock_dir = tempfile::tempdir().expect("socket dir");
    let sock = sock_dir.path().join("override.sock");
    let mut config = DaemonConfig::default();
    config.endpoint_flag = Some(format!("uds:{}", sock.display()));
    let fixture = DaemonFixture::start(&common::app(), config, || async { Ok(()) }, project_router)
        .await
        .expect("start");
    let default_sock = fixture.instance_root().join("daemon.sock");
    assert!(
        !default_sock.exists(),
        "default socket appeared at {}",
        default_sock.display()
    );
    let mut transport = sc_transport::TransportConfig::default();
    transport.instance_root = Some(fixture.instance_root().to_path_buf());
    let flag = format!("uds:{}", sock.display());
    let endpoint =
        sc_transport::resolve_endpoint(&common::app(), Some(OsStr::new(&flag)), None, &transport)
            .expect("resolve");
    let client = sc_transport::Client::new(endpoint).expect("client");
    let body: String = client.get("/project").await.expect("project");
    assert_eq!(body, "project");
    fixture.shutdown().await.expect("shutdown");
}

#[cfg(unix)]
const EPHEMERAL: &str = "SC_RUNTIME_EPHEMERAL_CI";
#[cfg(unix)]
const SC8_CHILD: &str = "SC_RUNTIME_SC8_CHILD";

/// Non-async child entry. Re-exec must not nest a Tokio runtime inside `#[tokio::test]`.
#[cfg(unix)]
#[test]
fn child_honours_sc_endpoint() {
    if std::env::var_os(SC8_CHILD).is_none() {
        return;
    }
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("child runtime");
    let ok = runtime.block_on(async {
        let mut config = DaemonConfig::default();
        let root = std::path::PathBuf::from(std::env::var("SC_RUNTIME_SC8_DIR").expect("dir"));
        config.transport.instance_root = Some(root);
        sc_runtime::Daemon::builder(&common::app(), &config)
            .stores(|| async { Ok(()) })
            .routes(project_router)
            .run()
            .await
            .is_ok()
    });
    std::process::exit(if ok { 0 } else { 1 });
}

#[cfg(unix)]
struct ChildGuard {
    child: Option<Child>,
}

#[cfg(unix)]
impl ChildGuard {
    fn spawn(dir: &std::path::Path, endpoint: &str) -> Self {
        let mut command = Command::new(std::env::current_exe().expect("exe"));
        command
            .arg("child_honours_sc_endpoint")
            .arg("--exact")
            .arg("--nocapture")
            .env(EPHEMERAL, "1")
            .env(SC8_CHILD, "1")
            .env("SC_RUNTIME_SC8_DIR", dir)
            .env("SC_ENDPOINT", endpoint);
        Self {
            child: Some(command.spawn().expect("spawn")),
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
#[tokio::test]
async fn child_daemon_uses_sc_endpoint() {
    if std::env::var_os(EPHEMERAL).is_none() {
        eprintln!("{EPHEMERAL} is unset; REQ-RT-0008 SC4 not run");
        return;
    }
    let root = tempfile::tempdir().expect("root");
    let sock_dir = tempfile::tempdir().expect("socket dir");
    let sock = sock_dir.path().join("override.sock");
    let endpoint = format!("uds:{}", sock.display());
    let mut child = ChildGuard::spawn(root.path(), &endpoint);
    let mut transport = sc_transport::TransportConfig::default();
    transport.instance_root = Some(root.path().to_path_buf());
    let resolved = sc_transport::resolve_endpoint(
        &common::app(),
        None,
        Some(OsStr::new(&endpoint)),
        &transport,
    )
    .expect("resolve");
    let client = sc_transport::Client::new(resolved).expect("client");
    let mut reached = None;
    for _ in 0..100 {
        if child.try_wait().is_some() {
            panic!("daemon exited before it became reachable at {endpoint}");
        }
        if let Ok(body) = client.get::<String>("/project").await {
            reached = Some(body);
            break;
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
    let body = reached.unwrap_or_else(|| {
        child.kill_and_reap();
        panic!("child daemon did not answer /project");
    });
    assert_eq!(body, "project");
    assert!(
        root.path().join("daemon.lock").is_file(),
        "daemon.lock missing under {}",
        root.path().display()
    );
    child.kill_and_reap();
}

#[tokio::test]
async fn child_uses_the_endpoint_string() {
    if std::env::var_os("SC_RUNTIME_ENDPOINT_CHILD").is_some() {
        let app = common::app();
        let config = DaemonConfig::default();
        let endpoint = sc_transport::resolve_endpoint_from_env(&app, None, &config.transport)
            .expect("resolve");
        let client = sc_transport::Client::new(endpoint).expect("client");
        let body: String = client.get("/project").await.expect("project");
        assert_eq!(body, "project");
        return;
    }
    let fixture = DaemonFixture::start(
        &common::app(),
        common::config(),
        || async { Ok(()) },
        project_router,
    )
    .await
    .expect("start");
    let output = std::process::Command::new(std::env::current_exe().expect("exe"))
        .arg("child_uses_the_endpoint_string")
        .arg("--exact")
        .env("SC_RUNTIME_ENDPOINT_CHILD", "1")
        .env("SC_ENDPOINT", fixture.endpoint_str())
        .output()
        .expect("child");
    assert!(
        output.status.success(),
        "child failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    fixture.shutdown().await.expect("shutdown");
}

#[test]
fn daemon_config_json_round_trip_shape() {
    let config: DaemonConfig =
        serde_json::from_str(r#"{"tcp_port":9,"endpoint_flag":null}"#).expect("json");
    assert_eq!(config.transport.tcp_port, Some(9));
    assert!(config.endpoint_flag.is_none());
}
