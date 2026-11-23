#![cfg(feature = "test-util")]

use std::future;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use axum::Router;
use axum::body::{Body, to_bytes};
use axum::http::{Request, StatusCode};
use axum::routing::get;
use sc_command::observability::{RecordingCanonicalLog, SharedCanonicalLog};
use sc_observability_types::ServiceName;
use sc_runtime::testing::instrument_router;
use tokio::sync::oneshot;
use tower::ServiceExt;

struct CancelNotice(Option<oneshot::Sender<()>>);

impl Drop for CancelNotice {
    fn drop(&mut self) {
        if let Some(notice) = self.0.take() {
            let _ = notice.send(());
        }
    }
}

#[tokio::test(start_paused = true)]
async fn public_helper_applies_timeout_tracking_logging_and_cancellation() {
    let logger = Arc::new(RecordingCanonicalLog::new(
        ServiceName::new("test-service").expect("valid service name"),
    ));
    let shared_logger: SharedCanonicalLog = logger.clone();
    let (started_tx, started_rx) = oneshot::channel();
    let (cancelled_tx, cancelled_rx) = oneshot::channel();
    let started = Arc::new(Mutex::new(Some(started_tx)));
    let cancelled = Arc::new(Mutex::new(Some(cancelled_tx)));

    let handler_started = Arc::clone(&started);
    let handler_cancelled = Arc::clone(&cancelled);
    let router = Router::new().route(
        "/hang",
        get(move || {
            let started = handler_started
                .lock()
                .expect("started sender lock")
                .take()
                .expect("handler called once");
            let cancelled = handler_cancelled
                .lock()
                .expect("cancel sender lock")
                .take()
                .expect("handler called once");
            async move {
                let _cancel_notice = CancelNotice(Some(cancelled));
                let _ = started.send(());
                future::pending::<StatusCode>().await
            }
        }),
    );
    let (_stop, shutdown) = tokio::sync::watch::channel(false);
    let limit = Duration::from_millis(50);
    let router = instrument_router(
        router,
        shared_logger,
        limit,
        shutdown,
        Duration::from_secs(1),
    );

    let request = router.oneshot(
        Request::builder()
            .uri("/hang")
            .body(Body::empty())
            .expect("valid request"),
    );
    tokio::pin!(request);
    tokio::select! {
        biased;
        response = &mut request => panic!("request completed before timeout: {response:?}"),
        started = started_rx => started.expect("handler started"),
    }

    tokio::time::advance(limit).await;
    let response = tokio::time::timeout(Duration::from_secs(1), &mut request)
        .await
        .expect("timeout response is bounded")
        .expect("router service succeeds");
    assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
    let request_id = response
        .headers()
        .get("x-request-id")
        .expect("request tracking adds an ID")
        .to_str()
        .expect("request ID is a string")
        .to_owned();
    let body = to_bytes(response.into_body(), 16 * 1024)
        .await
        .expect("bounded response body");
    let envelope: serde_json::Value = serde_json::from_slice(&body).expect("JSON envelope");
    assert_eq!(envelope["error"]["code"], "RUNTIME.REQUEST_TIMEOUT");
    tokio::time::timeout(Duration::from_secs(1), cancelled_rx)
        .await
        .expect("handler cancellation is bounded")
        .expect("handler future was dropped");

    let events = logger.events();
    assert_eq!(events.len(), 2);
    assert_eq!(events[0].action.as_str(), "request.start");
    assert_eq!(events[1].request_id.as_ref().unwrap().as_str(), request_id);
}
