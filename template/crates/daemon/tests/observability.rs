//! Production JSONL composition proof without delivering process signals.

#[cfg(feature = "cron")]
#[tokio::test]
async fn production_http_and_cron_share_jsonl_and_release_handles() {
    use std::sync::Arc;
    use std::time::Duration;

    use daemon::{cron, observability::Observability};
    use sc_observability_types::LogEvent;
    use sc_runtime::{DaemonConfig, testing::DaemonFixture};
    use utoipa_axum::router::OpenApiRouter;

    let root = tempfile::tempdir().expect("log root");
    let logging = serde_json::from_value(serde_json::json!({
        "enableFileSink": true, "enableConsoleSink": false,
    }))
    .expect("logging settings");
    let observability = Observability::init(daemon::APP_NAME, &logging, root.path().to_path_buf())
        .unwrap_or_else(|error| panic!("logger init: {error}"));
    let shared = observability.logger();
    let weak = Arc::downgrade(&shared);
    let config = cron::CronConfig::default();
    let scheduler = daemon::composition::CronServices::default()
        .open_scheduler(&config, root.path(), Arc::clone(&shared))
        .expect("production scheduler composition");
    let worker = scheduler.spawn_worker(Arc::new(cron::ApplicationJob));
    let mut config = DaemonConfig::default();
    config.endpoint_flag = Some("tcp:127.0.0.1:0".to_owned());
    let daemon = DaemonFixture::start_with_logger(
        &sc_transport::AppName::new(daemon::APP_NAME).expect("app"),
        config,
        || async { Ok(()) },
        |()| OpenApiRouter::new(),
        Arc::clone(&shared),
    )
    .await
    .expect("serving handshake within fixture deadline");
    let response: serde_json::Value = daemon.client().get("/health").await.expect("real health");
    assert_eq!(response["ok"], true);
    daemon.shutdown().await.expect("runtime drained and joined");
    assert!(
        worker
            .shutdown(tokio::time::Instant::now() + Duration::from_secs(5))
            .await
    );
    drop(scheduler);
    drop(shared);
    assert_eq!(
        weak.strong_count(),
        1,
        "only the observability owner retains the shared handle"
    );
    observability.shutdown().expect("logger flushed and joined");
    assert!(
        weak.upgrade().is_none(),
        "owner released its last shared handle"
    );
    let log = root
        .path()
        .join("logs")
        .join(format!("{}.log.jsonl", daemon::APP_NAME));
    let events: Vec<LogEvent> = std::fs::read_to_string(log)
        .expect("production JSONL")
        .lines()
        .map(|line| serde_json::from_str(line).expect("canonical LogEvent"))
        .collect();
    assert!(!events.is_empty());
    assert!(
        events
            .iter()
            .all(|event| event.service.as_str() == daemon::APP_NAME)
    );
    assert!(
        events
            .iter()
            .any(|event| event.target.as_str() == "cron.scheduler"
                && event.action.as_str() == "logger.attached")
    );
    let start = events
        .iter()
        .find(|event| {
            event.target.as_str() == "http.server" && event.action.as_str() == "request.start"
        })
        .expect("HTTP start");
    assert!(start.request_id.is_some());
    assert!(
        events
            .iter()
            .any(|event| event.target.as_str() == "http.server"
                && event.action.as_str() == "request.finish"
                && event.request_id == start.request_id)
    );
}
