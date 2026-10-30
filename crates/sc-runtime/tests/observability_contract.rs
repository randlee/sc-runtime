use std::sync::Arc;

use sc_observability_types::{
    CanonicalLog, NoopCanonicalLog, RecordingCanonicalLog, RejectingCanonicalLog, ServiceName,
    SharedCanonicalLog,
};
use sc_runtime::{Daemon, DaemonBuilder, DaemonConfig, RuntimeError};
use sc_transport::AppName;
use utoipa_axum::router::OpenApiRouter;

fn same_phase<P, S, F, R>(
    builder: DaemonBuilder<P, S, F, R>,
    logger: SharedCanonicalLog,
) -> DaemonBuilder<P, S, F, R> {
    builder.logger(logger)
}

#[test]
fn logger_preserves_all_phases_and_releases_replaced_handles() {
    let app = AppName::new("application-name").unwrap();
    let original: SharedCanonicalLog = Arc::new(RecordingCanonicalLog::new(
        ServiceName::new("first").unwrap(),
    ));
    let weak = Arc::downgrade(&original);
    let builder = same_phase(Daemon::builder(&app, &DaemonConfig::default()), original);
    let recorder: SharedCanonicalLog = Arc::new(RecordingCanonicalLog::new(
        ServiceName::new("injected").unwrap(),
    ));
    let builder = builder.logger(Arc::clone(&recorder));
    assert!(weak.upgrade().is_none());
    let builder = same_phase(builder.stores(|| async { Ok(()) }), Arc::clone(&recorder));
    let ready = same_phase(
        builder.routes(|()| OpenApiRouter::new()),
        Arc::clone(&recorder),
    );
    assert_eq!(Arc::strong_count(&recorder), 2);
    assert!(format!("{ready:?}").contains("injected"));
    drop(ready);
    assert_eq!(Arc::strong_count(&recorder), 1);
}

#[test]
fn legacy_builder_identity_is_fixed_and_trait_is_object_safe() {
    let app = AppName::new("other-application").unwrap();
    let builder = Daemon::builder(&app, &DaemonConfig::default());
    let debug = format!("{builder:?}");
    assert!(debug.contains("sc-runtime"));
    assert!(!debug.contains("other-application"));
    let log: Arc<dyn CanonicalLog> = Arc::new(NoopCanonicalLog::named_or_default("sc-runtime"));
    assert_eq!(log.service_name().as_str(), "sc-runtime");
}

#[tokio::test]
async fn rejecting_receiver_preserves_runtime_failure() {
    let app = AppName::new("contract-rejection").unwrap();
    let receiver = Arc::new(RejectingCanonicalLog::new(
        ServiceName::new("reject").unwrap(),
    ));
    let mut config = DaemonConfig::default();
    // Endpoint validation fails before any signal, lock or application store side effect.
    config.endpoint_flag = Some("not-an-endpoint".into());
    let outcome = Daemon::builder(&app, &config)
        .logger(receiver)
        .stores(|| async {
            Err::<(), _>(RuntimeError::stores_failed(
                "SENTINEL", "sentinel", "retry", None,
            ))
        })
        .routes(|()| OpenApiRouter::new())
        .run()
        .await;
    assert!(outcome.is_err());
    assert!(!outcome.unwrap_err().to_string().contains("sentinel"));
}

#[test]
fn library_dependency_and_blocking_adapter_edges_are_scoped() {
    let manifest = include_str!("../Cargo.toml");
    assert!(manifest.contains("sc-observability-types.workspace = true"));
    assert!(
        !manifest
            .lines()
            .any(|line| line.starts_with("sc-observability =")
                || line.starts_with("sc-observability-otlp"))
    );
    let builder = include_str!("../src/builder.rs");
    assert!(!builder.contains("enqueue_blocking"));
    assert!(!builder.contains(".log("));
    assert!(!builder.contains("NeedLogger"));
}
