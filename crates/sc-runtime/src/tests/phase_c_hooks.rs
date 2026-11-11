//! Production bound receipt and portable cleanup fixture ownership.
use super::*;
use crate::testing::DaemonFixture;
use std::sync::atomic::AtomicUsize;

const WAIT: Duration = Duration::from_secs(5);

fn port_zero_config() -> DaemonConfig {
    DaemonConfig {
        endpoint_flag: Some("tcp:127.0.0.1:0".into()),
        ..DaemonConfig::default()
    }
}

#[tokio::test]
async fn bound_callback_reports_continuously_owned_listener_before_serving() {
    let root = tempfile::tempdir().expect("root");
    let mut config = port_zero_config();
    config.transport.instance_root = Some(root.path().into());
    let (receipt, endpoint) = oneshot::channel();
    let (stop, stopped) = oneshot::channel();
    let calls = Arc::new(AtomicUsize::new(0));
    let cleanups = calls.clone();
    let run = tokio::spawn(async move {
        Daemon::builder(&app(), &config)
            .stores(|| async { Ok(()) })
            .routes(|()| OpenApiRouter::new())
            .with_control(RunControl::detached(None, Some(stopped)))
            .run_with_cleanup_and_bound(
                move |endpoint| {
                    let text = endpoint.to_string();
                    let address = text.strip_prefix("tcp:").expect("tcp endpoint");
                    let address: std::net::SocketAddr = address.parse().expect("actual address");
                    assert_ne!(address.port(), 0);
                    // The callback runs before serve is polled: a competing binder
                    // must fail while the original listener remains continuously owned.
                    let error = std::net::TcpListener::bind(address).expect_err("owned listener");
                    assert_eq!(error.kind(), std::io::ErrorKind::AddrInUse);
                    let _ = receipt.send(endpoint);
                },
                move || async move {
                    cleanups.fetch_add(1, Ordering::SeqCst);
                },
            )
            .await
    });
    let endpoint = tokio::time::timeout(WAIT, endpoint)
        .await
        .expect("bound receipt")
        .expect("endpoint");
    let client = sc_transport::Client::new(endpoint).expect("client");
    let health: serde_json::Value = tokio::time::timeout(WAIT, client.get("/health"))
        .await
        .expect("health bound")
        .expect("health");
    assert_eq!(health["data"]["status"], "ok");
    stop.send(()).expect("stop");
    tokio::time::timeout(WAIT, run)
        .await
        .expect("run bound")
        .expect("join")
        .expect("run");
    assert_eq!(calls.load(Ordering::SeqCst), 1);
}

#[tokio::test]
async fn prebind_failure_drops_unused_stores_before_once_only_cleanup() {
    let root = tempfile::tempdir().expect("root");
    let mut config = rooted_config(root.path());
    config.endpoint_flag = Some("invalid:endpoint".into());
    let (unused, mut dropped) = oneshot::channel::<()>();
    let bound = Arc::new(AtomicBool::new(false));
    let bound_observer = bound.clone();
    let cleanup = Arc::new(AtomicUsize::new(0));
    let observed = cleanup.clone();
    let error = Daemon::builder(&app(), &config)
        .stores(move || async move {
            let _ = unused.send(());
            Ok(())
        })
        .routes(|()| OpenApiRouter::new())
        .with_control(RunControl::detached(None, None))
        .run_with_cleanup_and_bound(
            move |_| {
                bound_observer.store(true, Ordering::SeqCst);
            },
            move || async move {
                assert!(
                    matches!(
                        dropped.try_recv(),
                        Err(oneshot::error::TryRecvError::Closed)
                    ),
                    "unused closure dropped before cleanup"
                );
                observed.fetch_add(1, Ordering::SeqCst);
            },
        )
        .await
        .expect_err("invalid endpoint");
    assert!(matches!(error, RuntimeError::EndpointFailed { .. }));
    assert_eq!(cleanup.load(Ordering::SeqCst), 1);
    assert!(!bound.load(Ordering::SeqCst));
}

#[tokio::test]
async fn parallel_cleanup_fixtures_report_distinct_actual_port_zero_endpoints() {
    let count = Arc::new(AtomicUsize::new(0));
    // AppName must outlive both constructor futures.
    let name = app();
    let first_count = count.clone();
    let first = DaemonFixture::start_with_cleanup(
        &name,
        port_zero_config(),
        || async { Ok(()) },
        |()| OpenApiRouter::new(),
        move || async move {
            first_count.fetch_add(1, Ordering::SeqCst);
        },
    );
    let second_count = count.clone();
    let second = DaemonFixture::start_with_cleanup(
        &name,
        port_zero_config(),
        || async { Ok(()) },
        |()| OpenApiRouter::new(),
        move || async move {
            second_count.fetch_add(1, Ordering::SeqCst);
        },
    );
    let (first, second) = tokio::join!(first, second);
    let first = first.expect("first");
    let second = second.expect("second");
    assert_ne!(first.endpoint(), second.endpoint());
    assert!(!first.endpoint_str().ends_with(":0"));
    assert!(!second.endpoint_str().ends_with(":0"));
    for fixture in [&first, &second] {
        let client =
            sc_transport::Client::new(fixture.endpoint().clone()).expect("actual endpoint client");
        let health: serde_json::Value = client
            .get("/health")
            .await
            .expect("health at accessor endpoint");
        assert_eq!(health["data"]["status"], "ok");
    }
    let (a, b) = tokio::join!(first.shutdown(), second.shutdown());
    a.expect("first shutdown");
    b.expect("second shutdown");
    assert_eq!(count.load(Ordering::SeqCst), 2);
}

#[tokio::test]
async fn cleanup_fixture_preserves_primary_startup_failure_and_runs_once() {
    let cleaned = Arc::new(AtomicUsize::new(0));
    let observed = cleaned.clone();
    let error =
        RuntimeError::stores_failed("STORE.CAUSAL", "primary failure", "fix application", None);
    let expected = error.clone();
    let result = DaemonFixture::start_with_cleanup(
        &app(),
        port_zero_config(),
        move || async move { Err::<(), _>(error) },
        |()| OpenApiRouter::new(),
        move || async move {
            observed.fetch_add(1, Ordering::SeqCst);
        },
    )
    .await;
    assert_eq!(result.err(), Some(expected));
    assert_eq!(cleaned.load(Ordering::SeqCst), 1);
}

#[tokio::test]
async fn dropping_cleanup_fixture_joins_after_cleanup_and_retains_stores() {
    let store = Arc::new(());
    let lifetime = Arc::downgrade(&store);
    let (entered, cleaning) = oneshot::channel();
    let (release, released) = oneshot::channel();
    let events = Arc::new(Mutex::new(Vec::new()));
    let cleanup_events = events.clone();
    let fixture = DaemonFixture::start_with_cleanup(
        &app(),
        port_zero_config(),
        move || async move { Ok(store) },
        |_| OpenApiRouter::new(),
        move || async move {
            let _ = entered.send(());
            let _ = released.await;
            cleanup_events
                .lock()
                .expect("events")
                .push("cleanup_finished");
        },
    )
    .await
    .expect("start cleanup fixture");
    let dropped_events = events.clone();
    let dropping = tokio::task::spawn_blocking(move || {
        drop(fixture);
        dropped_events
            .lock()
            .expect("events")
            .push("fixture_drop_returned");
    });
    tokio::time::timeout(WAIT, cleaning)
        .await
        .expect("cleanup entered")
        .expect("notice");
    assert!(
        lifetime.upgrade().is_some(),
        "stores retained during cleanup"
    );
    assert!(
        events.lock().expect("events").is_empty(),
        "drop cannot return during held cleanup"
    );
    release.send(()).expect("release cleanup");
    tokio::time::timeout(WAIT, dropping)
        .await
        .expect("physical join bound")
        .expect("drop worker");
    assert!(lifetime.upgrade().is_none());
    assert_eq!(
        *events.lock().expect("events"),
        ["cleanup_finished", "fixture_drop_returned"]
    );
}
