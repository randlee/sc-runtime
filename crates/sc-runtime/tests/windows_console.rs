//! Native console child, invoked only by the isolated Windows CI driver.
#![cfg(windows)]

use std::io::Write;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::time::Duration;

use axum::Json;
use sc_command::observability::{NoopCanonicalLog, SharedCanonicalLog};
use sc_runtime::{Daemon, DaemonConfig, RuntimeError};
use sc_transport::AppName;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt};
use utoipa_axum::router::OpenApiRouter;

const CHILD: &str = "SC_RUNTIME_CONSOLE_CHILD";
const CONTROL: &str = "SC_RUNTIME_CONSOLE_CONTROL";
const WAIT: Duration = Duration::from_secs(15);

async fn exchange(event: &str) {
    let mut stream =
        tokio::net::TcpStream::connect(std::env::var(CONTROL).expect("control endpoint"))
            .await
            .expect("control connection");
    stream
        .write_all(format!("{{\"event\":\"{event}\"}}\n").as_bytes())
        .await
        .expect("notice");
    let mut line = String::new();
    let count = tokio::time::timeout(WAIT, tokio::io::BufReader::new(stream).read_line(&mut line))
        .await
        .expect("control deadline")
        .expect("control release");
    assert!(count > 0 && line.trim() == "release");
}

#[utoipa::path(get, path = "/slow", responses((status = 200, body = String)))]
async fn slow() -> Json<&'static str> {
    exchange("inflight").await;
    Json("done")
}

async fn child() -> Result<(), RuntimeError> {
    let app = AppName::new("sc-runtime-console-test").expect("name");
    let mut config = DaemonConfig::default();
    config.transport.instance_root = Some(
        std::env::var_os("SC_RUNTIME_CONSOLE_ROOT")
            .expect("root")
            .into(),
    );
    config.endpoint_flag = Some("tcp:127.0.0.1:0".into());
    if std::env::var_os("SC_RUNTIME_CONSOLE_LOCK_PROBE").is_some() {
        let opened = Arc::new(AtomicBool::new(false));
        let attempted = opened.clone();
        let error = Daemon::builder(&app, &config)
            .stores(move || async move {
                attempted.store(true, Ordering::SeqCst);
                Ok(())
            })
            .routes(|()| OpenApiRouter::new())
            .run()
            .await
            .expect_err("old owner still holds lock during cleanup");
        assert!(matches!(error, RuntimeError::LockHeld { .. }));
        assert!(!opened.load(Ordering::SeqCst));
        return Ok(());
    }
    let logger: SharedCanonicalLog = Arc::new(NoopCanonicalLog::named_or_default("console-test"));
    let logger_lifetime = Arc::downgrade(&logger);
    let store = Arc::new(());
    let store_lifetime = Arc::downgrade(&store);
    let cleanup_count = Arc::new(AtomicUsize::new(0));
    let cleanup_counter = cleanup_count.clone();
    let early = std::env::var_os("SC_RUNTIME_CONSOLE_EARLY").is_some();
    let result = Daemon::builder(&app, &config)
        .logger(logger)
        .stores(move || async move {
            if early {
                exchange("stores_entered").await;
            }
            Ok(store)
        })
        .routes(|_| OpenApiRouter::new().routes(utoipa_axum::routes!(slow)))
        .run_with_cleanup_and_bound(
            |endpoint| {
                let address = std::env::var(CONTROL).expect("control endpoint");
                let mut stream = std::net::TcpStream::connect_timeout(
                    &address.parse().expect("control address"),
                    WAIT,
                )
                .expect("bound connection");
                stream.set_write_timeout(Some(WAIT)).expect("write timeout");
                let body = serde_json::json!({"event":"bound", "endpoint":endpoint.to_string()});
                writeln!(stream, "{body}").expect("bound receipt");
            },
            move || async move {
                assert_eq!(cleanup_counter.fetch_add(1, Ordering::SeqCst), 0);
                assert!(
                    store_lifetime.upgrade().is_some(),
                    "stores retained during cleanup"
                );
                assert!(
                    logger_lifetime.upgrade().is_some(),
                    "same logger retained during cleanup"
                );
                exchange("cleanup").await;
            },
        )
        .await;
    assert_eq!(cleanup_count.load(Ordering::SeqCst), 1);
    result
}

#[test]
fn console_child() {
    if std::env::var_os(CHILD).is_none() {
        return;
    }
    for (name, expected) in [
        ("SC_RUNTIME_EPHEMERAL_CI", "1"),
        ("SC_RUNTIME_GHA_EPHEMERAL_SIGNAL_TEST", "1"),
        ("GITHUB_ACTIONS", "true"),
        ("RUNNER_ENVIRONMENT", "github-hosted"),
    ] {
        assert_eq!(
            std::env::var(name).as_deref(),
            Ok(expected),
            "console children require the complete isolated-host guard: {name}"
        );
    }
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("child runtime");
    let result = runtime.block_on(child()); // allow: isolated native Windows test process entry
    if let Err(error) = result {
        panic!("console child: {error}");
    }
}
