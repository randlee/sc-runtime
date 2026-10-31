use std::any::TypeId;
use std::sync::Arc;

use super::*;

pub(crate) fn event(action: &str) -> LogEvent {
    LogEvent {
        version: (*OBSERVATION_SCHEMA_VERSION).clone(),
        timestamp: Timestamp::now_utc(),
        level: Level::Info,
        service: ServiceName::new("test").unwrap(),
        target: TargetCategory::new("test.receiver").unwrap(),
        action: ActionName::new(action).unwrap(),
        message: None,
        identity: ProcessIdentity::default(),
        trace: None,
        request_id: None,
        correlation_id: None,
        outcome: None,
        diagnostic: None,
        state_transition: None,
        fields: Default::default(),
    }
}

#[test]
fn canonical_receiver_is_object_safe_send_sync_and_noop_accepts() {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<SharedCanonicalLog>();
    let receiver: SharedCanonicalLog = Arc::new(NoopCanonicalLog::named_or_default("sc-runtime"));
    assert_eq!(receiver.service_name().as_str(), "sc-runtime");
    receiver.emit(event("discarded")).unwrap();
    let same = Arc::clone(&receiver);
    assert!(Arc::ptr_eq(&same, &receiver));
}

#[test]
fn noop_keeps_valid_names_and_falls_back_for_invalid_names() {
    for name in ["sc-runtime", "sc-cron", "app.production"] {
        assert_eq!(
            NoopCanonicalLog::named_or_default(name)
                .service_name()
                .as_str(),
            name
        );
    }
    for invalid in ["", "contains space", "slash/name", "服务"] {
        let receiver = NoopCanonicalLog::named_or_default(invalid);
        assert_eq!(receiver.service_name().as_str(), "noop");
        receiver.emit(event("discarded")).unwrap();
    }
    let identity = ServiceName::new("chosen").unwrap();
    assert_eq!(
        NoopCanonicalLog::new(identity.clone()).service_name(),
        &identity
    );
}

#[test]
fn facade_retains_released_value_and_error_type_identity() {
    assert_eq!(
        TypeId::of::<ServiceName>(),
        TypeId::of::<released_types::ServiceName>()
    );
    assert_eq!(
        TypeId::of::<LogEvent>(),
        TypeId::of::<released_types::LogEvent>()
    );
    assert_eq!(
        TypeId::of::<v2::EventError>(),
        TypeId::of::<released_types::v2::EventError>()
    );
    assert_eq!(
        TypeId::of::<ErrorContext>(),
        TypeId::of::<released_types::ErrorContext>()
    );
    let original: released_types::LogEvent = event("original");
    let facade: LogEvent = original;
    let restored: released_types::LogEvent = facade;
    assert_eq!(restored.action.as_str(), "original");
    #[cfg(feature = "v1")]
    {
        #[allow(
            deprecated,
            reason = "the development facade must preserve the released v1 error API"
        )]
        let identity = (
            TypeId::of::<EventError>(),
            TypeId::of::<released_types::EventError>(),
        );
        assert_eq!(identity.0, identity.1);
    }
}

#[test]
fn command_accepts_facade_codes_and_remediation_without_conversion() {
    let code = ErrorCode::new_static("SHIM.IDENTITY");
    let remediation = Remediation::recoverable("Retry the test.", [] as [&str; 0]);
    let error = sc_command::OpError::new(sc_command::ErrorKind::Internal, code.clone(), "test")
        .with_remediation(remediation.clone());
    let returned_code: ErrorCode = error.code;
    let returned_remediation: Option<Remediation> = error.remediation;
    assert_eq!(returned_code, code);
    assert_eq!(returned_remediation, Some(remediation));
}

#[cfg(feature = "test-util")]
#[test]
fn recording_preserves_shared_order_and_detached_snapshots() {
    let recorder = Arc::new(RecordingCanonicalLog::new(
        ServiceName::new("test").unwrap(),
    ));
    let shared: SharedCanonicalLog = recorder.clone();
    shared.emit(event("first")).unwrap();
    let second = Arc::clone(&shared);
    std::thread::spawn(move || second.emit(event("second")).unwrap())
        .join()
        .unwrap();
    let snapshot = recorder.events();
    assert_eq!(
        snapshot
            .iter()
            .map(|e| e.action.as_str())
            .collect::<Vec<_>>(),
        ["first", "second"]
    );
    recorder.clear();
    assert!(recorder.events().is_empty());
    assert_eq!(snapshot.len(), 2);
    shared.emit(event("after-clear")).unwrap();
    assert_eq!(recorder.events()[0].action.as_str(), "after-clear");
}

#[cfg(feature = "test-util")]
#[test]
fn rejecting_counts_shared_attempts_and_returns_fresh_routing_contexts() {
    let receiver = Arc::new(RejectingCanonicalLog::new(
        ServiceName::new("test").unwrap(),
    ));
    let shared: SharedCanonicalLog = receiver.clone();
    assert_eq!(receiver.attempts(), 0);
    let first = shared.emit(event("one")).unwrap_err();
    let second = shared.emit(event("two")).unwrap_err();
    assert_eq!(receiver.attempts(), 2);
    assert!(matches!(first, v2::EventError::Routing { .. }));
    assert!(matches!(second, v2::EventError::Routing { .. }));
    assert!(!std::ptr::eq(first.context(), second.context()));
    assert_eq!(first.diagnostic().code, error_codes::DIAGNOSTIC_INVALID);
    assert_eq!(second.diagnostic().code, error_codes::DIAGNOSTIC_INVALID);
    assert_eq!(receiver.service_name().as_str(), "test");
    // Errors remain the released error type, with its original context API.
    let _: released_types::v2::EventError = first;
}
