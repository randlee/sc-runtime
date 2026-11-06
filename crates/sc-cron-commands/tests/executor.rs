#![cfg(feature = "scheduler")]
#[path = "support/operation_double.rs"]
mod operations;

use operations::{OperationCall, RecordingOperations};
use sc_command::{ErrorKind, OpError};
use sc_cron::{
    CommitAdmission, CronError, EndedSchedule, Generation, HookTarget, PersistenceOutcome,
    Schedule, ScheduleId, SchedulePage, ScheduleSelection, ScheduleSpec, ShutdownBudget,
    StoreError, StoreErrorKind, TransactionOutcome, WorkerStopControl,
};
use sc_cron_commands::*;
use serde_json::{Value, json};
use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};
use std::time::{Duration, Instant};

#[derive(Default)]
struct Bindings {
    calls: AtomicUsize,
    deny: bool,
}
impl ApplicationTargetBindings for Bindings {
    fn resolve(&self, target: &ExecutionTarget, payload: Value) -> Result<ResolvedTarget, OpError> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        if self.deny {
            return Err(OpError::from_code(
                ErrorKind::Unavailable,
                "APP.TARGET_DENIED",
                "target denied",
            ));
        }
        let hook = match target {
            ExecutionTarget::Rust => HookTarget::Rust,
            ExecutionTarget::Python { module } => HookTarget::python(module).unwrap(),
            ExecutionTarget::Monty { source } => HookTarget::monty(source.clone()).unwrap(),
        };
        Ok(ResolvedTarget { hook, payload })
    }
}

fn request(command: &str, body: Value) -> CronRequest {
    serde_json::from_value(json!({"version":"1","command":command,"request":body})).unwrap()
}
fn start() -> CronRequest {
    request(
        "cron.start",
        json!({"name":"nightly","cron":"*/5 * * * *","timeout_ms":25000,
        "target":{"kind":"rust"},"payload":{"job":"sync"},"environment":{"dotenv":[],"variables":{}}}),
    )
}
fn row(id: u64, ended: bool) -> Schedule {
    let at = "2026-10-26T10:01:00Z".parse().unwrap();
    Schedule {
        id: ScheduleId::new(id),
        generation: Generation::FIRST,
        spec: ScheduleSpec::new(
            "nightly",
            "*/5 * * * *",
            Duration::from_secs(25),
            json!({"secret":"CANARY"}),
        )
        .unwrap(),
        jitter: Duration::ZERO,
        created_at: at,
        next_run_at: (!ended).then_some(at),
        last_run_at: None,
        last_outcome: None,
        ended_at: ended.then_some(at),
        stop_reason: None,
        enabled: !ended,
    }
}
fn fixture() -> (
    Arc<RecordingOperations>,
    Arc<Bindings>,
    WorkerStopControl,
    StandardCronExecutor<RecordingOperations, Bindings>,
) {
    let operations = Arc::new(RecordingOperations::default());
    let bindings = Arc::new(Bindings::default());
    let control = WorkerStopControl::new().unwrap();
    let executor = StandardCronExecutor::new(
        Arc::clone(&operations),
        Arc::clone(&bindings),
        control.operation_context(),
    );
    (operations, bindings, control, executor)
}
fn deadline() -> Instant {
    Instant::now() + Duration::from_secs(3600)
}

#[tokio::test(flavor = "current_thread")]
async fn all_verbs_forward_exact_native_arguments_and_safe_results() {
    let (operations, bindings, _, executor) = fixture();
    operations
        .starts
        .lock()
        .unwrap()
        .push_back(Ok(row(7, false)));
    let cutoff = deadline();
    let response = executor
        .execute(start(), cutoff)
        .await
        .into_result()
        .unwrap();
    assert!(matches!(
        response,
        CronResponse::Started(ScheduleView {
            id: 7,
            generation: 1,
            ..
        })
    ));
    {
        let calls = operations.calls.lock().unwrap();
        let OperationCall::Start(context, spec) = &calls[0] else {
            panic!("start")
        };
        assert_eq!(context.deadline(), Some(cutoff));
        assert_eq!(spec.name.as_str(), "nightly");
        assert_eq!(spec.expr.as_str(), "*/5 * * * *");
        assert_eq!(spec.timeout.get(), Duration::from_secs(25));
        assert_eq!(spec.payload, json!({"job":"sync"}));
        assert!(matches!(spec.hook, HookTarget::Rust));
    }
    operations.ends.lock().unwrap().push_back(Ok(EndedSchedule {
        id: ScheduleId::new(7),
        generation: Generation::FIRST,
    }));
    let response = executor
        .execute(
            request(
                "cron.end",
                json!({"name":"nightly","generation":1,"reason":"operator"}),
            ),
            cutoff,
        )
        .await;
    assert!(matches!(
        response.data(),
        Some(CronResponse::Ended(EndResult {
            id: 7,
            generation: 1
        }))
    ));
    operations
        .queries
        .lock()
        .unwrap()
        .push_back(Ok(SchedulePage {
            items: vec![row(7, false), row(9, true)],
            next_after_id: Some(ScheduleId::new(9)),
        }));
    let response = executor
        .execute(
            request("cron.list", json!({"state":"all","limit":2,"cursor":3})),
            cutoff,
        )
        .await;
    let serialized = serde_json::to_string(&response).unwrap();
    assert!(!serialized.contains("CANARY"));
    assert!(!serialized.contains("payload"));
    let calls = operations.calls.lock().unwrap();
    let OperationCall::End(context, end) = &calls[1] else {
        panic!("end")
    };
    assert_eq!(context.deadline(), Some(cutoff));
    assert_eq!(end.name.as_str(), "nightly");
    assert_eq!(end.generation.get(), 1);
    assert_eq!(end.reason, "operator");
    let OperationCall::Query(context, query) = &calls[2] else {
        panic!("query")
    };
    assert_eq!(context.deadline(), Some(cutoff));
    assert_eq!(query.selection, ScheduleSelection::All);
    assert_eq!(query.limit, 2);
    assert_eq!(query.after_id, Some(ScheduleId::new(3)));
    let Some(CronResponse::Listed(page)) = response.data() else {
        panic!("list")
    };
    assert_eq!(page.next_cursor, Some(9));
    assert_eq!(page.items[1].state, ScheduleState::Ended);
    assert_eq!(bindings.calls.load(Ordering::SeqCst), 1);
}

#[tokio::test(flavor = "current_thread")]
async fn invalid_requests_never_call_bindings_or_operations() {
    let (operations, bindings, _, executor) = fixture();
    let mut cases = vec![];
    for (key, value) in [
        ("name", json!("")),
        ("cron", json!("bad")),
        ("timeout_ms", json!(0)),
        ("timeout_ms", json!(u64::MAX)),
        ("payload", json!("x".repeat(sc_cron::MAX_PAYLOAD_BYTES + 1))),
    ] {
        let mut bad = start();
        let CronOperation::Start(body) = &mut bad.operation else {
            unreachable!()
        };
        match key {
            "name" => body.name = value.as_str().unwrap().into(),
            "cron" => body.cron = value.as_str().unwrap().into(),
            "timeout_ms" => body.timeout_ms = value.as_u64().unwrap(),
            _ => body.payload = value,
        }
        cases.push(bad);
    }
    cases.push(request(
        "cron.end",
        json!({"name":"nightly","generation":0,"reason":"operator"}),
    ));
    cases.push(request(
        "cron.end",
        json!({"name":"nightly","generation":1,"reason":""}),
    ));
    for (limit, cursor) in [(0, None), (1001, None), (100, Some(0))] {
        cases.push(request(
            "cron.list",
            json!({"state":"all","limit":limit,"cursor":cursor}),
        ));
    }
    let mut bad = start();
    bad.version = "99".into();
    cases.push(bad);
    let mut huge = start();
    let CronOperation::Start(body) = &mut huge.operation else {
        unreachable!()
    };
    body.payload = json!("x".repeat(sc_cron_commands::contract::MAX_REQUEST_BYTES));
    cases.push(huge);
    for request in cases {
        let response = executor.execute(request, deadline()).await;
        assert_eq!(response.error().unwrap().kind, ErrorKind::InvalidInput);
    }
    assert!(operations.calls.lock().unwrap().is_empty());
    assert_eq!(bindings.calls.load(Ordering::SeqCst), 0);
}

#[tokio::test(flavor = "current_thread")]
async fn environment_and_target_capabilities_are_checked_before_writes() {
    let (operations, _, _, executor) = fixture();
    for target in [
        ExecutionTarget::Python {
            module: "-c".into(),
        },
        ExecutionTarget::Monty {
            source: "x".repeat(sc_cron::MAX_HOOK_SOURCE_BYTES + 1),
        },
    ] {
        let mut bad = start();
        let CronOperation::Start(body) = &mut bad.operation else {
            unreachable!()
        };
        body.target = target;
        assert!(!executor.execute(bad, deadline()).await.is_ok());
    }
    let mut bad = start();
    let CronOperation::Start(body) = &mut bad.operation else {
        unreachable!()
    };
    body.environment
        .variables
        .insert("SECRET".into(), EnvironmentOperation::Set("CANARY".into()));
    assert!(!executor.execute(bad, deadline()).await.is_ok());
    assert!(operations.calls.lock().unwrap().is_empty());
    let mut python = start();
    let CronOperation::Start(body) = &mut python.operation else {
        unreachable!()
    };
    body.target = ExecutionTarget::Python {
        module: "job.py".into(),
    };
    body.environment.variables.insert(
        "TOKEN".into(),
        EnvironmentOperation::Reference("secret/key".into()),
    );
    body.environment
        .variables
        .insert("CLEAR".into(), EnvironmentOperation::Clear(true));
    body.environment.dotenv.push(DotenvSource {
        path: std::env::temp_dir()
            .join("allowed.env")
            .to_string_lossy()
            .into(),
        required: false,
    });
    operations
        .starts
        .lock()
        .unwrap()
        .push_back(Ok(row(1, false)));
    assert!(executor.execute(python, deadline()).await.is_ok());
    let calls = operations.calls.lock().unwrap();
    let OperationCall::Start(_, spec) = &calls[0] else {
        panic!("start")
    };
    assert!(matches!(spec.hook, HookTarget::Python { .. }));
    assert_eq!(spec.environment.len(), 2);
    assert_eq!(spec.environment.dotenv_sources().len(), 1);
}

#[tokio::test(flavor = "current_thread")]
async fn child_deadline_and_shared_cancellation_use_normal_public_control() {
    let operations = Arc::new(RecordingOperations::default());
    let bindings = Arc::new(Bindings::default());
    let control = WorkerStopControl::new().unwrap();
    let earlier = deadline();
    let executor = StandardCronExecutor::new(
        Arc::clone(&operations),
        bindings,
        control.operation_context().with_deadline(earlier),
    );
    operations
        .starts
        .lock()
        .unwrap()
        .push_back(Ok(row(1, false)));
    assert!(
        executor
            .execute(start(), earlier + Duration::from_secs(1))
            .await
            .is_ok()
    );
    {
        let calls = operations.calls.lock().unwrap();
        let OperationCall::Start(context, _) = &calls[0] else {
            panic!("start")
        };
        assert_eq!(context.deadline(), Some(earlier));
    }
    control
        .request_stop(ShutdownBudget::new(earlier, earlier).unwrap())
        .unwrap();
    control.cancel_now();
    let error = executor
        .execute(start(), deadline())
        .await
        .into_result()
        .unwrap_err();
    assert!(matches!(
        error.code.as_str(),
        "CRON.CANCELLED" | "CRON.DEADLINE_EXCEEDED"
    ));
    assert_eq!(operations.calls.lock().unwrap().len(), 1);
    control.finish_application_shutdown().unwrap();
    let (operations, _, _, executor) = fixture();
    assert!(
        !executor
            .execute(start(), Instant::now() - Duration::from_secs(1))
            .await
            .is_ok()
    );
    assert!(operations.calls.lock().unwrap().is_empty());
}

#[tokio::test(flavor = "current_thread")]
async fn errors_are_typed_value_free_and_keep_true_commit_evidence() {
    let (operations, _, _, executor) = fixture();
    let cases = [
        (
            CronError::AlreadyActive(sc_cron::ScheduleName::new("CANARY").unwrap()),
            ErrorKind::Conflict,
        ),
        (
            CronError::NotFound(sc_cron::ScheduleName::new("CANARY").unwrap()),
            ErrorKind::NotFound,
        ),
        (
            CronError::Fenced {
                id: ScheduleId::new(7),
                generation: Generation::FIRST,
            },
            ErrorKind::Conflict,
        ),
        (
            CronError::Store(StoreError::new(StoreErrorKind::Busy, "CANARY")),
            ErrorKind::Unavailable,
        ),
    ];
    for (error, kind) in cases {
        operations.starts.lock().unwrap().push_back(Err(error));
        let response = executor.execute(start(), deadline()).await;
        assert_eq!(response.error().unwrap().kind, kind);
        assert!(!serde_json::to_string(&response).unwrap().contains("CANARY"));
    }
    operations
        .starts
        .lock()
        .unwrap()
        .push_back(Err(StoreError::new(
            StoreErrorKind::DeadlineExceeded,
            "CANARY",
        )
        .with_source(std::io::Error::other("CANARY"))
        .with_transaction_outcome(TransactionOutcome {
            admission: CommitAdmission::Admitted,
            persistence: PersistenceOutcome::Committed,
            completed_after_cutoff: true,
        })
        .into()));
    let error = executor
        .execute(start(), deadline())
        .await
        .into_result()
        .unwrap_err();
    assert_eq!(error.code.as_str(), "CRON.DEADLINE_EXCEEDED");
    assert_eq!(
        error.details,
        Some(
            json!({"admission":"admitted","persistence":"committed","completed_after_cutoff":true})
        )
    );
    assert!(error.suggested_action().unwrap().contains("reconcile"));
    assert!(!format!("{error:?}").contains("CANARY"));
}

#[tokio::test(flavor = "current_thread")]
async fn capacity_refusal_and_explicit_monty_capability() {
    let (operations, _, _, executor) = fixture();
    operations
        .starts
        .lock()
        .unwrap()
        .push_back(Err(StoreError::new(
            StoreErrorKind::Unavailable,
            "command_capacity_exhausted",
        )
        .into()));
    let error = executor
        .execute(start(), deadline())
        .await
        .into_result()
        .unwrap_err();
    assert_eq!(error.code.as_str(), "CRON.COMMAND_CAPACITY_EXHAUSTED");
    assert!(
        error
            .suggested_action()
            .unwrap()
            .contains("wait for owned work")
    );
    let mut monty = start();
    let CronOperation::Start(body) = &mut monty.operation else {
        unreachable!()
    };
    body.target = ExecutionTarget::Monty {
        source: "def cron_tick(hook): pass".into(),
    };
    #[cfg(feature = "monty")]
    operations
        .starts
        .lock()
        .unwrap()
        .push_back(Ok(row(1, false)));
    let response = executor.execute(monty, deadline()).await;
    assert_eq!(response.is_ok(), cfg!(feature = "monty"));
    assert_eq!(
        operations.calls.lock().unwrap().len(),
        1 + usize::from(cfg!(feature = "monty"))
    );
}

#[tokio::test(flavor = "current_thread")]
async fn application_authorization_refuses_before_any_native_operation() {
    let operations = Arc::new(RecordingOperations::default());
    let bindings = Arc::new(Bindings {
        calls: AtomicUsize::new(0),
        deny: true,
    });
    let control = WorkerStopControl::new().unwrap();
    let executor = StandardCronExecutor::new(
        Arc::clone(&operations),
        Arc::clone(&bindings),
        control.operation_context(),
    );
    let error = executor
        .execute(start(), deadline())
        .await
        .into_result()
        .unwrap_err();
    assert_eq!(error.code.as_str(), "APP.TARGET_DENIED");
    assert_eq!(bindings.calls.load(Ordering::SeqCst), 1);
    assert!(operations.calls.lock().unwrap().is_empty());
}

#[tokio::test(flavor = "current_thread")]
async fn invalid_environment_policy_refuses_before_authorization_and_native_calls() {
    let (operations, bindings, _, executor) = fixture();
    for (name, operation) in [
        ("2INVALID", EnvironmentOperation::Set("value".into())),
        (
            "TOKEN",
            EnvironmentOperation::Set("v".repeat(sc_cron::MAX_ENV_VALUE_BYTES + 1)),
        ),
        ("TOKEN", EnvironmentOperation::Reference(String::new())),
        ("TOKEN", EnvironmentOperation::Clear(false)),
    ] {
        let mut request = start();
        let CronOperation::Start(body) = &mut request.operation else {
            unreachable!()
        };
        body.target = ExecutionTarget::Python {
            module: "job.py".into(),
        };
        body.environment.variables.insert(name.into(), operation);
        assert_eq!(
            executor
                .execute(request, deadline())
                .await
                .error()
                .unwrap()
                .kind,
            ErrorKind::InvalidInput
        );
    }
    assert!(operations.calls.lock().unwrap().is_empty());
    assert_eq!(bindings.calls.load(Ordering::SeqCst), 0);
}

#[tokio::test(flavor = "current_thread")]
async fn all_persistence_states_are_retained_without_retry_or_rollback_claims() {
    let (operations, _, _, executor) = fixture();
    for persistence in [
        PersistenceOutcome::NoMutationAttempted,
        PersistenceOutcome::Committed,
        PersistenceOutcome::RolledBack,
        PersistenceOutcome::Indeterminate,
    ] {
        let outcome = TransactionOutcome {
            admission: CommitAdmission::Admitted,
            persistence,
            completed_after_cutoff: true,
        };
        operations
            .starts
            .lock()
            .unwrap()
            .push_back(Err(StoreError::new(
                StoreErrorKind::Indeterminate,
                "provider-private",
            )
            .with_transaction_outcome(outcome)
            .into()));
        let error = executor
            .execute(start(), deadline())
            .await
            .into_result()
            .unwrap_err();
        assert_eq!(error.code.as_str(), "CRON.INDETERMINATE");
        assert_eq!(error.details.as_ref().unwrap()["admission"], "admitted");
        assert_eq!(
            error.details.as_ref().unwrap()["completed_after_cutoff"],
            true
        );
        assert!(
            error
                .suggested_action()
                .unwrap()
                .contains("separately authorized")
        );
        assert!(
            !serde_json::to_string(&error)
                .unwrap()
                .contains("provider-private")
        );
    }
    assert_eq!(operations.calls.lock().unwrap().len(), 4);
}

#[tokio::test(flavor = "current_thread")]
async fn invalid_causes_are_static_actionable_and_redacted() {
    let (operations, bindings, _, executor) = fixture();
    for (error, reason, recovery) in [
        (
            CronError::BadExpression("PRIVATE_PROVIDER_CANARY".into()),
            "invalid_expression",
            "cron expression",
        ),
        (
            CronError::InvalidSpec("PRIVATE_PROVIDER_CANARY".into()),
            "invalid_specification",
            "schedule limits",
        ),
    ] {
        operations.starts.lock().unwrap().push_back(Err(error));
        let response = executor.execute(start(), deadline()).await;
        let error = response.error().unwrap();
        assert_eq!(error.kind, ErrorKind::InvalidInput);
        assert_eq!(error.code.as_str(), "CRON.INVALID_INPUT");
        assert_eq!(error.details, Some(json!({"reason": reason})));
        assert!(error.suggested_action().unwrap().contains(recovery));
        let wire = serde_json::to_value(&response).unwrap();
        assert_eq!(wire["version"], "1");
        assert_eq!(wire["ok"], false);
        assert!(wire["data"].is_null());
        assert!(!wire.to_string().contains("PRIVATE_PROVIDER_CANARY"));
    }
    assert_eq!(operations.calls.lock().unwrap().len(), 2);
    assert_eq!(bindings.calls.load(Ordering::SeqCst), 2);

    let (operations, bindings, _, executor) = fixture();
    let mut version = start();
    version.version = "PRIVATE_REQUEST_CANARY".into();
    let mut huge = start();
    let CronOperation::Start(body) = &mut huge.operation else {
        unreachable!()
    };
    body.payload = json!("PRIVATE_REQUEST_CANARY".repeat(contract::MAX_REQUEST_BYTES / 10));
    for (request, reason, recovery) in [
        (version, "unsupported_version", "version to 1"),
        (huge, "request_size", "2 MiB"),
        (
            request(
                "cron.end",
                json!({"name":"PRIVATE_REQUEST_CANARY", "generation":0,"reason":"private"}),
            ),
            "generation",
            "generation from list",
        ),
        (
            request("cron.list", json!({"state":"all","limit":0,"cursor":null})),
            "limit",
            "1 through 1000",
        ),
        (
            request("cron.list", json!({"state":"all","limit":100,"cursor":0})),
            "cursor",
            "next_cursor",
        ),
    ] {
        let response = executor.execute(request, deadline()).await;
        let error = response.error().unwrap();
        assert_eq!(error.kind, ErrorKind::InvalidInput);
        assert_eq!(error.code.as_str(), "CRON.INVALID_INPUT");
        assert_eq!(error.details, Some(json!({"reason": reason})));
        assert!(error.suggested_action().unwrap().contains(recovery));
        assert!(
            !serde_json::to_string(&response)
                .unwrap()
                .contains("PRIVATE_REQUEST_CANARY")
        );
    }
    assert!(operations.calls.lock().unwrap().is_empty());
    assert_eq!(bindings.calls.load(Ordering::SeqCst), 0);
}
