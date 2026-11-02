#![cfg(feature = "test-util")]

use axum::{Router, extract::Path, http::StatusCode, routing::get};
use sc_command::observability::{RecordingCanonicalLog, RejectingCanonicalLog};
use sc_observability_types::ServiceName;

// Shared transport helpers also contain process-signal helpers used by other tests.
#[allow(dead_code)]
mod common;
use sc_runtime::testing::instrument_router;
use std::sync::Arc;
use std::time::Duration;

async fn start(router: Router) -> (String, tokio::task::JoinHandle<()>) {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let task = tokio::spawn(async move {
        axum::serve(listener, router).await.unwrap();
    });
    (format!("http://{addr}"), task)
}

fn recorder() -> Arc<RecordingCanonicalLog> {
    Arc::new(RecordingCanonicalLog::new(
        ServiceName::new("test-service").unwrap(),
    ))
}

async fn status(Path(code): Path<u16>) -> StatusCode {
    StatusCode::from_u16(code).unwrap()
}

#[tokio::test]
async fn records_statuses_templates_ids_and_concurrency() {
    let recorder = recorder();
    let (_stop, rx) = tokio::sync::watch::channel(false);
    let router = instrument_router(
        Router::new().route("/status/{code}", get(status)),
        recorder.clone(),
        Duration::from_secs(5),
        rx,
        Duration::from_secs(1),
    );
    let (url, task) = start(router).await;
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(3))
        .build()
        .unwrap();
    for (code, outcome, action) in [
        (101, "success", "request.finish"),
        (200, "success", "request.finish"),
        (302, "success", "request.finish"),
        (404, "client_error", "request.finish"),
        (500, "server_error", "request.error"),
    ] {
        recorder.clear();
        let response = client
            .get(format!("{url}/status/{code}?secret=hidden"))
            .header("x-request-id", "supplied-id")
            .header("authorization", "Bearer private-authorization")
            .header("cookie", "private-cookie")
            .body("private-request-body")
            .send()
            .await
            .unwrap();
        assert_eq!(response.status().as_u16(), code);
        assert_eq!(response.headers()["x-request-id"], "supplied-id");
        let events = recorder.events();
        assert_eq!(events.len(), 2);
        assert_eq!(events[0].action.as_str(), "request.start");
        assert_eq!(events[1].action.as_str(), action);
        assert_eq!(events[1].outcome.as_ref().unwrap().as_str(), outcome);
        assert_eq!(
            events[0].request_id.as_ref().unwrap().as_str(),
            "supplied-id"
        );
        assert_eq!(events[1].request_id, events[0].request_id);
        assert_eq!(events[1].fields["path"], "/status/{code}");
        assert_eq!(events[1].fields["status"], code);
        assert_eq!(events[1].service.as_str(), "test-service");
        assert!(events[1].diagnostic.is_none());
        assert!(events[1].fields["elapsed_ms"].is_u64());
        for event in &events {
            assert_eq!(event.target.as_str(), "http.server");
            assert_eq!(event.fields["method"], "GET");
            assert_eq!(event.fields["path"], "/status/{code}");
            let encoded = serde_json::to_string(event).unwrap();
            for secret in [
                "hidden",
                "private-authorization",
                "private-cookie",
                "private-request-body",
            ] {
                assert!(!encoded.contains(secret), "event leaked {secret}");
            }
        }
    }
    recorder.clear();
    let response = client
        .get(format!("{url}/unknown/secret?x=secret"))
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    let events = recorder.events();
    assert_eq!(events.len(), 2);
    for event in &events {
        assert_eq!(event.fields["path"], "<unmatched>");
        assert!(!serde_json::to_string(event).unwrap().contains("secret"));
    }
    for invalid in ["bad/request-id", "", &"x".repeat(129)] {
        recorder.clear();
        let response = client
            .get(format!("{url}/status/200"))
            .header("x-request-id", invalid)
            .send()
            .await
            .unwrap();
        let id = response.headers()["x-request-id"].to_str().unwrap();
        assert!(id.starts_with("req-"));
        assert_ne!(id, invalid);
        assert!(
            recorder
                .events()
                .iter()
                .all(|event| event.request_id.as_ref().unwrap().as_str() == id)
        );
    }
    recorder.clear();
    let (a, b) = tokio::join!(
        client.get(format!("{url}/status/200")).send(),
        client.get(format!("{url}/status/200")).send()
    );
    let a = a.unwrap().headers()["x-request-id"]
        .to_str()
        .unwrap()
        .to_owned();
    let b = b.unwrap().headers()["x-request-id"]
        .to_str()
        .unwrap()
        .to_owned();
    assert_ne!(a, b);
    let events = recorder.events();
    for id in [a, b] {
        let matching: Vec<_> = events
            .iter()
            .filter(|e| e.request_id.as_ref().unwrap().as_str() == id)
            .collect();
        assert_eq!(matching.len(), 2);
        assert_eq!(matching[0].action.as_str(), "request.start");
        assert_eq!(matching[1].action.as_str(), "request.finish");
    }
    task.abort();
}

#[tokio::test]
async fn deadline_codes_and_rejection_are_best_effort() {
    for (drain, expected) in [
        (false, "RUNTIME.REQUEST_TIMEOUT"),
        (true, "RUNTIME.DRAIN_TIMEOUT"),
    ] {
        let recorder = recorder();
        let (stop, rx) = tokio::sync::watch::channel(drain);
        let router = instrument_router(
            Router::new().route("/hang", get(std::future::pending::<&'static str>)),
            recorder.clone(),
            if drain {
                Duration::from_secs(2)
            } else {
                Duration::from_millis(30)
            },
            rx,
            Duration::from_millis(30),
        );
        let (url, task) = start(router).await;
        // This bound is independent of the middleware deadlines under test and
        // covers body collection too. Clean up before reporting either failure.
        let exchange = tokio::time::timeout(Duration::from_secs(3), async {
            let response = reqwest::get(format!("{url}/hang")).await?;
            let status = response.status();
            let headers = response.headers().clone();
            let body = response.json::<serde_json::Value>().await?;
            Ok::<_, reqwest::Error>((status, headers, body))
        })
        .await;
        task.abort();
        let _ = task.await;
        drop(stop);
        let (status, headers, body) = exchange
            .expect("HTTP deadline exchange exceeded independent 3-second bound")
            .expect("HTTP deadline exchange failed");
        assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE);
        let id = headers["x-request-id"].to_str().unwrap().to_owned();
        assert_eq!(body["error"]["code"], expected);
        let events = recorder.events();
        assert_eq!(events.len(), 2);
        assert_eq!(events[1].request_id.as_ref().unwrap().as_str(), id);
        assert_eq!(events[1].outcome.as_ref().unwrap().as_str(), "timeout");
        assert_eq!(events[1].fields["deadline_remaining_ms"], 0);
        assert_eq!(
            events[1].diagnostic.as_ref().unwrap().code.as_str(),
            expected
        );
    }
    let rejecting = Arc::new(RejectingCanonicalLog::new(
        ServiceName::new("reject").unwrap(),
    ));
    let (_stop, rx) = tokio::sync::watch::channel(false);
    let router = instrument_router(
        Router::new().route("/ok", get(|| async { "body" })),
        rejecting.clone(),
        Duration::from_secs(1),
        rx,
        Duration::from_secs(1),
    );
    let (url, task) = start(router).await;
    let response = tokio::time::timeout(Duration::from_secs(1), reqwest::get(format!("{url}/ok")))
        .await
        .unwrap()
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    assert!(response.headers().contains_key("x-request-id"));
    assert_eq!(response.text().await.unwrap(), "body");
    assert_eq!(rejecting.attempts(), 2);
    task.abort();
}

#[tokio::test]
async fn real_daemon_preserves_injected_handle_and_records_http_events() {
    use sc_command::observability::SharedCanonicalLog;
    use sc_runtime::testing::DaemonFixture;
    use utoipa_axum::router::OpenApiRouter;

    let recorder = recorder();
    let shared: SharedCanonicalLog = recorder.clone();
    let expected = shared.clone();
    let fixture = DaemonFixture::start_with_logger(
        &common::app(),
        common::config(),
        || async { Ok(()) },
        move |()| {
            OpenApiRouter::new().route(
                "/identity",
                get(
                    move |axum::Extension(actual): axum::Extension<SharedCanonicalLog>| {
                        let expected = expected.clone();
                        async move { axum::Json(Arc::ptr_eq(&actual, &expected)) }
                    },
                ),
            )
        },
        shared,
    )
    .await
    .unwrap();
    assert!(fixture.instance_root().join("daemon.lock").exists());
    recorder.clear();
    let response = common::http(fixture.endpoint())
        .get(common::url(fixture.endpoint(), "/identity"))
        .header("x-request-id", "identity-regression")
        .send()
        .await
        .unwrap();
    assert_eq!(response.headers()["x-request-id"], "identity-regression");
    assert!(response.json::<bool>().await.unwrap());
    let events = recorder.events();
    assert_eq!(events.len(), 2);
    assert_eq!(events[0].action.as_str(), "request.start");
    assert_eq!(events[1].action.as_str(), "request.finish");
    for event in events {
        assert_eq!(event.service.as_str(), "test-service");
        assert_eq!(event.request_id.unwrap().as_str(), "identity-regression");
        assert_eq!(event.fields["path"], "/identity");
    }
    fixture.shutdown().await.unwrap();
}

/// Models real bounded admission with try_send; neither failure can wait for capacity.
struct RejectedQueue {
    service: ServiceName,
    sender: std::sync::mpsc::SyncSender<()>,
    _receiver: Option<std::sync::Mutex<std::sync::mpsc::Receiver<()>>>,
    attempts: std::sync::atomic::AtomicU64,
}

impl RejectedQueue {
    fn new(disconnected: bool) -> Self {
        let (sender, receiver) = std::sync::mpsc::sync_channel(1);
        sender.try_send(()).unwrap();
        Self {
            service: ServiceName::new("reject-queue").unwrap(),
            sender,
            _receiver: (!disconnected).then_some(std::sync::Mutex::new(receiver)),
            attempts: std::sync::atomic::AtomicU64::new(0),
        }
    }
}

impl sc_command::observability::CanonicalLog for RejectedQueue {
    fn service_name(&self) -> &ServiceName {
        &self.service
    }

    fn emit(
        &self,
        _: sc_observability_types::LogEvent,
    ) -> Result<(), sc_observability_types::v2::EventError> {
        use sc_observability_types::{
            ErrorContext, FailureClassification, Remediation, error_codes,
        };
        self.attempts
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let classification = match self.sender.try_send(()) {
            Err(std::sync::mpsc::TrySendError::Full(())) => FailureClassification::QueueFull,
            Err(std::sync::mpsc::TrySendError::Disconnected(())) => FailureClassification::Closed,
            Ok(()) => panic!("fixture must reject admission"),
        };
        Err(sc_observability_types::v2::EventError::classified_routing(
            Box::new(ErrorContext::new(
                error_codes::DIAGNOSTIC_INVALID,
                "test queue rejected admission",
                Remediation::recoverable("Restore queue admission.", [] as [&str; 0]),
            )),
            classification,
        ))
    }
}

#[tokio::test]
async fn queue_full_and_disconnected_preserve_responses_headers_and_metrics() {
    use sc_runtime::testing::DaemonFixture;
    use utoipa_axum::router::OpenApiRouter;

    for disconnected in [false, true] {
        let logger = Arc::new(RejectedQueue::new(disconnected));
        let fixture = DaemonFixture::start_with_logger(
            &common::app(),
            common::config(),
            || async { Ok(()) },
            |()| {
                OpenApiRouter::new()
                    .route(
                        "/ok",
                        get(|| async { ([("x-handler", "retained")], "body") }),
                    )
                    .route(
                        "/error",
                        get(|| async { (StatusCode::BAD_GATEWAY, "private-error-body") }),
                    )
            },
            logger.clone(),
        )
        .await
        .unwrap();
        let client = common::http(fixture.endpoint());
        for (path, status, body) in [
            ("/ok", StatusCode::OK, "body"),
            ("/error", StatusCode::BAD_GATEWAY, "private-error-body"),
        ] {
            let before = logger.attempts.load(std::sync::atomic::Ordering::Relaxed);
            let response = tokio::time::timeout(
                Duration::from_secs(2),
                client
                    .get(common::url(fixture.endpoint(), path))
                    .header("x-request-id", "rejected-request")
                    .send(),
            )
            .await
            .unwrap()
            .unwrap();
            assert_eq!(response.status(), status);
            assert_eq!(response.headers()["x-request-id"], "rejected-request");
            if path == "/ok" {
                assert_eq!(response.headers()["x-handler"], "retained");
            }
            assert_eq!(response.text().await.unwrap(), body);
            assert_eq!(
                logger.attempts.load(std::sync::atomic::Ordering::Relaxed) - before,
                2
            );
        }
        // The fixture readiness probe plus the two requests above are already counted.
        let metrics: serde_json::Value = client
            .get(common::url(fixture.endpoint(), "/metrics"))
            .send()
            .await
            .unwrap()
            .json()
            .await
            .unwrap();
        assert_eq!(metrics["requests"], 3);
        assert_eq!(metrics["server_errors"], 1);
        fixture.shutdown().await.unwrap();
    }
}

#[tokio::test]
async fn queue_rejection_preserves_deadlines_and_handler_cancellation() {
    use std::sync::atomic::{AtomicBool, Ordering};
    struct Cancelled(Arc<AtomicBool>);
    impl Drop for Cancelled {
        fn drop(&mut self) {
            self.0.store(true, Ordering::SeqCst);
        }
    }
    for disconnected in [false, true] {
        for drain in [false, true] {
            let logger = Arc::new(RejectedQueue::new(disconnected));
            let cancelled = Arc::new(AtomicBool::new(false));
            let flag = cancelled.clone();
            let (_stop, rx) = tokio::sync::watch::channel(drain);
            let router = instrument_router(
                Router::new().route(
                    "/hang",
                    get(move || {
                        let flag = flag.clone();
                        async move {
                            let _guard = Cancelled(flag);
                            std::future::pending::<&'static str>().await
                        }
                    }),
                ),
                logger.clone(),
                if drain {
                    Duration::from_secs(5)
                } else {
                    Duration::from_millis(30)
                },
                rx,
                Duration::from_millis(30),
            );
            let (url, task) = start(router).await;
            let response =
                tokio::time::timeout(Duration::from_secs(2), reqwest::get(format!("{url}/hang")))
                    .await
                    .unwrap()
                    .unwrap();
            assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
            assert!(response.headers().contains_key("x-request-id"));
            let body: serde_json::Value = response.json().await.unwrap();
            assert_eq!(
                body["error"]["code"],
                if drain {
                    "RUNTIME.DRAIN_TIMEOUT"
                } else {
                    "RUNTIME.REQUEST_TIMEOUT"
                }
            );
            assert!(cancelled.load(Ordering::SeqCst));
            assert_eq!(logger.attempts.load(Ordering::Relaxed), 2);
            task.abort();
        }
    }
}
