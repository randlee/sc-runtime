use std::sync::{Arc, Mutex, mpsc};
use std::time::Duration;

use sc_observability::v2::{LogSink, Logger, LoggerConfig, SinkRegistration};
use sc_observability_dev_adapter::LoggerAdapter;
use sc_observability_types::{
    ActionName, CanonicalLog, CorrelationId, Level, LogEvent, ProcessIdentity, SchemaVersion,
    ServiceName, SinkHealth, SinkHealthState, SinkName, TargetCategory, Timestamp,
    constants::OBSERVATION_ENVELOPE_VERSION, v2::LogSinkError,
};

const WAIT: Duration = Duration::from_secs(10);

fn event(service: ServiceName) -> LogEvent {
    LogEvent {
        version: SchemaVersion::new(OBSERVATION_ENVELOPE_VERSION).expect("version"),
        timestamp: Timestamp::UNIX_EPOCH,
        level: Level::Info,
        service,
        target: TargetCategory::new("http.server").expect("target"),
        action: ActionName::new("request.start").expect("action"),
        message: None,
        identity: ProcessIdentity::default(),
        trace: None,
        request_id: Some(CorrelationId::new("request-123").expect("id")),
        correlation_id: None,
        outcome: None,
        diagnostic: None,
        state_transition: None,
        fields: serde_json::Map::new(),
    }
}

fn health() -> SinkHealth {
    SinkHealth {
        name: SinkName::new("adapter-test").expect("sink name"),
        state: SinkHealthState::Healthy,
        last_error: None,
    }
}

struct RecordingSink(Mutex<Vec<LogEvent>>);
impl LogSink for RecordingSink {
    fn write(&self, event: &LogEvent) -> Result<(), LogSinkError> {
        self.0.lock().expect("recording lock").push(event.clone());
        Ok(())
    }
    fn health(&self) -> SinkHealth {
        health()
    }
}

fn logger(root: &std::path::Path, sink: Arc<dyn LogSink>, capacity: usize) -> Arc<Logger> {
    let name = ServiceName::new("adapter-service").expect("service");
    let mut config = LoggerConfig::default_for(name, root.to_path_buf());
    config.enable_file_sink = false;
    config.enable_console_sink = false;
    config.queue_capacity = capacity;
    let mut builder = Logger::builder(config).expect("builder");
    builder.register_sink(SinkRegistration::typed(sink));
    Arc::new(builder.build().expect("logger"))
}

#[test]
#[allow(
    deprecated,
    reason = "the pinned 1.5.0 shutdown diagnostic is the contract under test"
)]
fn forwards_real_events_and_native_shutdown_error() {
    let root = tempfile::tempdir().expect("temp root");
    let sink = Arc::new(RecordingSink(Mutex::new(Vec::new())));
    let logger = logger(root.path(), sink.clone(), 8);
    let adapter = LoggerAdapter::new(Arc::clone(&logger));
    assert_eq!(adapter.service_name(), logger.service_name());
    let original = event(adapter.service_name().clone());
    adapter.emit(original.clone()).expect("admitted");
    logger.flush().expect("owner flush");
    let events = sink.0.lock().expect("recording lock");
    assert_eq!(events.len(), 1);
    assert_eq!(
        serde_json::to_value(&events[0]).expect("wire"),
        serde_json::to_value(original).expect("wire")
    );
    drop(events);
    logger.shutdown().expect("owner shutdown");
    let emitted = adapter
        .emit(event(adapter.service_name().clone()))
        .expect_err("closed");
    let direct = logger
        .try_log(event(logger.service_name().clone()))
        .expect_err("closed");
    assert_eq!(emitted.diagnostic().code, direct.diagnostic().code);
    assert_eq!(
        emitted.diagnostic().code,
        sc_observability::error_codes::LOGGER_SHUTDOWN
    );
}

struct BlockingSink {
    entered: mpsc::Sender<()>,
    release: Mutex<mpsc::Receiver<()>>,
}
impl LogSink for BlockingSink {
    fn write(&self, _event: &LogEvent) -> Result<(), LogSinkError> {
        let _ = self.entered.send(());
        let _ = self.release.lock().expect("release lock").recv();
        Ok(())
    }
    fn health(&self) -> SinkHealth {
        health()
    }
}

#[test]
fn saturated_real_queue_rejects_without_waiting_for_sink() {
    let root = tempfile::tempdir().expect("temp root");
    let (entered_tx, entered_rx) = mpsc::channel();
    let (release_tx, release_rx) = mpsc::channel();
    let sink = Arc::new(BlockingSink {
        entered: entered_tx,
        release: Mutex::new(release_rx),
    });
    let logger = logger(root.path(), sink, 1);
    let adapter = Arc::new(LoggerAdapter::new(Arc::clone(&logger)));
    adapter
        .emit(event(adapter.service_name().clone()))
        .expect("initial");
    entered_rx.recv_timeout(WAIT).expect("writer entered sink");
    adapter
        .emit(event(adapter.service_name().clone()))
        .expect("queue slot");
    let worker_adapter = Arc::clone(&adapter);
    let (result_tx, result_rx) = mpsc::channel();
    let worker = std::thread::spawn(move || {
        let result = worker_adapter.emit(event(worker_adapter.service_name().clone()));
        let _ = result_tx.send(result);
    });
    let result = result_rx.recv_timeout(WAIT);
    drop(release_tx);
    worker.join().expect("worker");
    let error = result
        .expect("admission must return while sink is blocked")
        .expect_err("queue full");
    assert_eq!(
        error.diagnostic().code,
        sc_observability::error_codes::LOGGER_QUEUE_FULL
    );
    logger.shutdown().expect("owner shutdown");
}
