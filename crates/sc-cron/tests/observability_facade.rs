//! Compile-time identity checks against the original public contracts.
use sc_cron::observability::{CanonicalLog, EventError, LogEvent, ServiceName, SharedCanonicalLog};

#[test]
fn facade_preserves_nominal_identity_in_both_directions() {
    fn original_to_facade(
        log: sc_command::observability::SharedCanonicalLog,
    ) -> SharedCanonicalLog {
        log
    }
    fn facade_to_original(
        log: SharedCanonicalLog,
    ) -> sc_command::observability::SharedCanonicalLog {
        log
    }
    fn receiver(log: &dyn CanonicalLog) -> &dyn sc_command::observability::CanonicalLog {
        log
    }
    fn event(event: LogEvent) -> sc_observability_types::LogEvent {
        event
    }
    fn service(service: ServiceName) -> sc_observability_types::ServiceName {
        service
    }
    fn error(error: EventError) -> sc_observability_types::v2::EventError {
        error
    }
    let original: sc_command::observability::SharedCanonicalLog = std::sync::Arc::new(
        sc_command::observability::NoopCanonicalLog::new(ServiceName::new("facade").unwrap()),
    );
    let round_trip = facade_to_original(original_to_facade(original.clone()));
    assert!(std::sync::Arc::ptr_eq(&original, &round_trip));
    assert_eq!(
        receiver(round_trip.as_ref()).service_name().as_str(),
        "facade"
    );
    let _ = (event, service, error);
}
