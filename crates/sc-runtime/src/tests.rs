//! Crate tests that need the private shutdown trigger or a known instance root.

use std::sync::{Arc, Mutex};

use rmcp::ServerHandler;
use rmcp::transport::streamable_http_server::session::local::LocalSessionManager;
use rmcp::transport::streamable_http_server::{StreamableHttpServerConfig, StreamableHttpService};
use tokio::sync::oneshot;
use utoipa_axum::router::OpenApiRouter;

use crate::shutdown::RunControl;
use crate::{Daemon, DaemonConfig, RuntimeError};

fn app() -> sc_transport::AppName {
    match sc_transport::AppName::new("sc-runtime-test") {
        Ok(app) => app,
        Err(_) => panic!("sc-runtime-test is a valid application name"),
    }
}

fn rooted_config(root: &std::path::Path) -> DaemonConfig {
    let mut config = DaemonConfig::default();
    config.transport.instance_root = Some(root.to_path_buf());
    #[cfg(windows)]
    {
        let addr = std::net::SocketAddr::from((std::net::Ipv4Addr::LOCALHOST, 0));
        config.endpoint_flag = Some(format!("tcp:{addr}"));
    }
    config
}

#[derive(Clone)]
struct Demo;

impl ServerHandler for Demo {
    fn get_info(&self) -> rmcp::model::ServerConfig {
        rmcp::model::ServerConfig::new(rmcp::model::ServerCapabilities::builder().build())
    }
}

fn demo_service(_stores: ()) -> StreamableHttpService<Demo, LocalSessionManager> {
    StreamableHttpService::new(
        || Ok(Demo),
        std::sync::Arc::new(LocalSessionManager::default()),
        StreamableHttpServerConfig::default()
            .with_legacy_session_mode(false)
            .with_json_response(true),
    )
}

#[tokio::test]
async fn lock_precedes_stores_then_routes_and_mcp() {
    let dir = tempfile::tempdir().expect("temp dir");
    let config = rooted_config(dir.path());
    let order = Arc::new(Mutex::new(Vec::new()));
    let lock_path = dir.path().join("daemon.lock");
    let (ready_tx, ready_rx) = oneshot::channel();
    let (stop_tx, stop_rx) = oneshot::channel();
    let recorded = Arc::clone(&order);
    let order_for_task = Arc::clone(&order);
    let task = tokio::spawn(async move {
        Daemon::builder(&app(), &config)
            .stores(move || {
                let recorded = Arc::clone(&recorded);
                let lock_path = lock_path.clone();
                async move {
                    let file = std::fs::OpenOptions::new()
                        .read(true)
                        .write(true)
                        .open(&lock_path)
                        .expect("lock file");
                    let mut probe = fd_lock::RwLock::new(file);
                    assert!(
                        probe.try_write().is_err(),
                        "daemon.lock must already be held"
                    );
                    recorded.lock().expect("order").push("stores");
                    Ok(())
                }
            })
            .routes({
                let recorded = Arc::clone(&order_for_task);
                move |()| {
                    recorded.lock().expect("order").push("routes");
                    OpenApiRouter::new()
                }
            })
            .mcp({
                let recorded = Arc::clone(&order_for_task);
                move |stores| {
                    recorded.lock().expect("order").push("mcp");
                    demo_service(stores)
                }
            })
            .with_control(RunControl {
                ready: Some(ready_tx),
                shutdown: Some(stop_rx),
                listen_for_signals: false,
            })
            .run()
            .await
    });
    let _endpoint = ready_rx.await.expect("daemon ready");
    let seen = order.lock().expect("order").clone();
    assert_eq!(seen.first().copied(), Some("stores"));
    assert_eq!(seen.len(), 3);
    assert!(seen.contains(&"routes"));
    assert!(seen.contains(&"mcp"));
    let _ = stop_tx.send(());
    task.await.expect("join").expect("run");
}

#[tokio::test]
async fn lock_contention_skips_stores() {
    let dir = tempfile::tempdir().expect("temp dir");
    let config = rooted_config(dir.path());
    let (ready_tx, ready_rx) = oneshot::channel();
    let (stop_tx, stop_rx) = oneshot::channel();
    let held = tokio::spawn({
        let config = config.clone();
        async move {
            Daemon::builder(&app(), &config)
                .stores(|| async { Ok(()) })
                .routes(|()| OpenApiRouter::new())
                .with_control(RunControl {
                    ready: Some(ready_tx),
                    shutdown: Some(stop_rx),
                    listen_for_signals: false,
                })
                .run()
                .await
        }
    });
    let _endpoint = ready_rx.await.expect("holder ready");
    let called = Arc::new(Mutex::new(false));
    let flag = Arc::clone(&called);
    let err = Daemon::builder(&app(), &config)
        .stores(move || {
            let flag = Arc::clone(&flag);
            async move {
                *flag.lock().expect("flag") = true;
                Ok(())
            }
        })
        .routes(|()| OpenApiRouter::new())
        .run()
        .await
        .expect_err("second daemon");
    assert!(matches!(err, RuntimeError::LockHeld { .. }));
    assert_eq!(err.code(), "RUNTIME.LOCK_HELD");
    assert!(!*called.lock().expect("flag"));
    let _ = stop_tx.send(());
    held.await.expect("join").expect("holder");
    assert!(
        dir.path().join("daemon.lock").is_file(),
        "shutdown must not delete daemon.lock"
    );
}

#[tokio::test]
async fn stores_error_skips_routes_and_bind() {
    let dir = tempfile::tempdir().expect("temp dir");
    let mut config = rooted_config(dir.path());
    let endpoint = dir.path().join("daemon.sock");
    config.endpoint_flag = Some(format!("uds:{}", endpoint.display()));
    let routes_called = Arc::new(Mutex::new(false));
    let flag = Arc::clone(&routes_called);
    let err = Daemon::builder(&app(), &config)
        .stores(|| async {
            Err(RuntimeError::stores_failed(
                "STORE.OPEN",
                "the store did not open",
                "check the database path",
                Some("disk full".to_owned()),
            ))
        })
        .routes(move |()| {
            *flag.lock().expect("flag") = true;
            OpenApiRouter::new()
        })
        .run()
        .await
        .expect_err("stores");
    match err {
        RuntimeError::StoresFailed {
            code,
            message,
            remediation,
            cause,
        } => {
            let _code: &'static str = code;
            let _message: String = message;
            let _remediation: String = remediation;
            let _cause: Option<String> = cause;
            assert_eq!(_code, "STORE.OPEN");
            assert_eq!(_message, "the store did not open");
            assert_eq!(_remediation, "check the database path");
            assert_eq!(_cause.as_deref(), Some("disk full"));
        }
        other => panic!("expected StoresFailed, got {other}"),
    }
    assert!(!*routes_called.lock().expect("flag"));
    assert!(!endpoint.exists());
}

#[tokio::test]
async fn file_instance_root_is_not_lock_held() {
    let dir = tempfile::tempdir().expect("temp dir");
    let file = dir.path().join("not-a-directory");
    std::fs::write(&file, b"x").expect("file");
    let mut config = DaemonConfig::default();
    config.transport.instance_root = Some(file);
    let called = Arc::new(Mutex::new(false));
    let flag = Arc::clone(&called);
    let err = Daemon::builder(&app(), &config)
        .stores(move || {
            let flag = Arc::clone(&flag);
            async move {
                *flag.lock().expect("flag") = true;
                Ok(())
            }
        })
        .routes(|()| OpenApiRouter::new())
        .run()
        .await
        .expect_err("file root");
    assert!(matches!(err, RuntimeError::InstanceFailed { .. }));
    assert!(!matches!(err, RuntimeError::LockHeld { .. }));
    assert!(!*called.lock().expect("flag"));
}

#[tokio::test]
async fn bad_endpoint_does_not_bind() {
    let dir = tempfile::tempdir().expect("temp dir");
    let mut config = rooted_config(dir.path());
    config.endpoint_flag = Some("not-an-endpoint".to_owned());
    let called = Arc::new(Mutex::new(false));
    let flag = Arc::clone(&called);
    let err = Daemon::builder(&app(), &config)
        .stores(move || {
            let flag = Arc::clone(&flag);
            async move {
                *flag.lock().expect("flag") = true;
                Ok(())
            }
        })
        .routes(|()| OpenApiRouter::new())
        .run()
        .await
        .expect_err("endpoint");
    assert!(matches!(err, RuntimeError::EndpointFailed { .. }));
    assert_eq!(err.code(), "RUNTIME.ENDPOINT_FAILED");
    assert!(!*called.lock().expect("flag"));
}

#[tokio::test]
async fn reserved_health_route_is_route_conflict() {
    let dir = tempfile::tempdir().expect("temp dir");
    let config = rooted_config(dir.path());
    let err = Daemon::builder(&app(), &config)
        .stores(|| async { Ok(()) })
        .routes(|()| OpenApiRouter::new().route("/health", axum::routing::get(|| async { "nope" })))
        .run()
        .await
        .expect_err("overlap");
    assert!(matches!(err, RuntimeError::RouteConflict { .. }), "{err:?}");
    assert_eq!(err.code(), "RUNTIME.ROUTE_CONFLICT");
}

#[tokio::test]
async fn busy_tcp_port_is_bind_failed() {
    let listener = std::net::TcpListener::bind((std::net::Ipv4Addr::LOCALHOST, 0)).expect("bind");
    let addr = listener.local_addr().expect("addr");
    let dir = tempfile::tempdir().expect("temp dir");
    let mut config = rooted_config(dir.path());
    config.endpoint_flag = Some(format!("tcp:{addr}"));
    let err = Daemon::builder(&app(), &config)
        .stores(|| async { Ok(()) })
        .routes(|()| OpenApiRouter::new())
        .run()
        .await
        .expect_err("bind");
    assert!(matches!(err, RuntimeError::BindFailed { .. }));
    assert_eq!(err.code(), "RUNTIME.BIND_FAILED");
    drop(listener);
}

#[test]
fn daemon_config_deserializes() {
    let config: DaemonConfig = serde_json::from_str(
        r#"{"endpoint":"tcp:127.0.0.1:9","instance_root":"instance","tcp_port":9,"endpoint_flag":"uds:sock"}"#,
    )
    .expect("json");
    let _ = Daemon::builder(&app(), &config);
}

#[test]
fn error_is_a_std_error() {
    fn assert_bounds<T: std::error::Error + Send + Sync + 'static>() {}
    assert_bounds::<RuntimeError>();
}
