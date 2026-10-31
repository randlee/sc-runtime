//! Public error, scheduled panic, and special-file failure boundaries.
use super::*;

const PANIC_VALUE: &str = "qa1-resolved-panic-value-canary";
const LOCATOR: &str = "provider://qa1-private-locator-canary";

#[test]
fn public_deserialization_errors_omit_rejected_values_and_unknown_fields() {
    for document in [
        r#"{"dotenv":"provider://qa1-private-locator-canary","variables":{}}"#,
        r#"{"dotenv":[{"path":true,"required":true}],"variables":{}}"#,
        r#"{"dotenv":[{"path":"/tmp/file","required":"qa1-resolved-panic-value-canary"}],"variables":{}}"#,
        r#"{"dotenv":[],"variables":{},"provider://qa1-private-locator-canary":"qa1-resolved-panic-value-canary"}"#,
        r#"{"dotenv":[{"path":"/tmp/file","required":true,"provider://qa1-private-locator-canary":true}],"variables":{}}"#,
        r#"{"dotenv":[],"variables":{"KEY":{"reference":false}}}"#,
        r#"{"dotenv":[],"variables":{"KEY":{"set":"qa1-resolved-panic-value-canary","reference":"provider://qa1-private-locator-canary"}}}"#,
        r#"{"dotenv":[],"variables":{"KEY":{"clear":false}}}"#,
        r#"{"dotenv":[],"variables":{"KEY":{"set":"qa1-resolved-panic-value-canary"},"KEY":{"clear":true}}}"#,
    ] {
        let error = serde_json::from_str::<HookEnvironment>(document).expect_err("reject wire");
        for detail in [error.to_string(), format!("{error:?}")] {
            assert!(detail.contains("invalid hook environment"), "{detail}");
            for rejected in [PANIC_VALUE, LOCATOR, "unknown field", "invalid type"] {
                assert!(!detail.contains(rejected), "{detail}");
            }
        }
    }
}

struct PanickingResolver;
#[async_trait::async_trait]
impl EnvironmentReferenceResolver for PanickingResolver {
    async fn resolve(
        &self,
        request: EnvironmentReferenceRequest<'_>,
    ) -> Result<Option<String>, EnvironmentReferenceError> {
        panic!("provider panic: {} {PANIC_VALUE}", request.reference());
    }
}

fn panic_fixture(
    hook: HookTarget,
    host: HookEnvironmentHost,
) -> (
    common::Fixture,
    Arc<sc_observability_types::RecordingCanonicalLog>,
    Schedule,
) {
    let mut fixture = common::Fixture::new();
    let recorder = Arc::new(sc_observability_types::RecordingCanonicalLog::new(
        sc_observability_types::ServiceName::new("panic-canary").expect("service"),
    ));
    fixture.scheduler = Scheduler::new_with_logger(
        fixture.store.clone(),
        fixture.clock.clone(),
        Arc::new(SeqJitter::default()),
        recorder.clone(),
    )
    .with_hook_environment(host);
    let row = fixture.start(spec(
        "panic-canary",
        hook,
        HookEnvironment::new()
            .reference("KEY", LOCATOR)
            .expect("reference"),
    ));
    (fixture, recorder, row)
}

async fn assert_panic_boundary(
    fixture: &common::Fixture,
    recorder: &sc_observability_types::RecordingCanonicalLog,
    row: &Schedule,
    worker: bool,
    code: &str,
    action: &str,
) {
    let outcome = if worker {
        let handle = tokio::time::timeout(common::WAIT, fixture.worker(common::idle_job()))
            .await
            .expect("worker startup");
        tokio::time::timeout(common::WAIT, fixture.step(60))
            .await
            .expect("worker step");
        fixture
            .store
            .wait_write("panic recorded", |write| {
                matches!(write, common::Write::Run { id, outcome, .. }
                if *id == row.id && outcome.failure_parts().and_then(|p| p.code) == Some(code))
            })
            .await;
        common::shutdown(handle).await;
        fixture.row(row).last_outcome.expect("worker persisted")
    } else {
        due(fixture).await
    };
    assert_code(&outcome, code);
    assert_eq!(fixture.row(row).last_outcome, Some(outcome.clone()));
    let events = recorder.events();
    let origin = events
        .iter()
        .filter(|event| event.action.as_str() == action)
        .collect::<Vec<_>>();
    assert_eq!(origin.len(), 1, "{events:?}");
    assert_eq!(
        origin[0].diagnostic.as_ref().map(|d| d.code.as_str()),
        Some(code)
    );
    assert!(origin[0].correlation_id.is_some());
    assert_eq!(
        events
            .iter()
            .filter(|event| matches!(event.action.as_str(), "run.finished" | "due.finished"))
            .count(),
        1
    );
    for detail in [
        outcome.to_string(),
        format!("{events:?}"),
        format!("{:?}", fixture.row(row).last_outcome),
    ] {
        for rejected in [PANIC_VALUE, LOCATOR, "provider panic"] {
            assert!(!detail.contains(rejected), "{detail}");
        }
    }
}

#[tokio::test]
async fn python_resolver_panics_are_value_free_in_due_and_worker_records_and_events() {
    for worker in [false, true] {
        let (fixture, recorder, row) = panic_fixture(
            HookTarget::python("never-executed.py").expect("python"),
            HookEnvironmentHost::new().with_reference_resolver(Arc::new(PanickingResolver)),
        );
        let (code, action) = if worker {
            ("JOB_JOIN_FAILED", "job.join_failed")
        } else {
            ("RUN_TASK_FAILED", "due.task_failed")
        };
        assert_panic_boundary(&fixture, &recorder, &row, worker, code, action).await;
    }
}

#[cfg(feature = "monty")]
struct ValueResolver;
#[cfg(feature = "monty")]
#[async_trait::async_trait]
impl EnvironmentReferenceResolver for ValueResolver {
    async fn resolve(
        &self,
        request: EnvironmentReferenceRequest<'_>,
    ) -> Result<Option<String>, EnvironmentReferenceError> {
        assert_eq!(request.reference(), LOCATOR);
        Ok(Some(PANIC_VALUE.to_owned()))
    }
}

#[cfg(feature = "monty")]
#[tokio::test]
async fn monty_host_panics_are_value_free_in_due_and_worker_records_and_events() {
    for worker in [false, true] {
        for phase in ["stop", "tick"] {
            let source = format!(
                "import os\ndef cron_stop(h):\n    {}\n    return {{'action':'continue'}}\ndef cron_tick(h):\n    {}\n",
                if phase == "stop" {
                    "panic_host(os.getenv('KEY'))"
                } else {
                    "pass"
                },
                if phase == "tick" {
                    "panic_host(os.getenv('KEY'))"
                } else {
                    "pass"
                }
            );
            let (mut fixture, recorder, row) = panic_fixture(
                HookTarget::monty(source).expect("monty"),
                HookEnvironmentHost::new().with_reference_resolver(Arc::new(ValueResolver)),
            );
            fixture.scheduler = fixture.scheduler.with_monty_hosts(
                MontyHostRegistry::new()
                    .register("panic_host", |call: &MontyHostCall| {
                        panic!("handler panic: {} {LOCATOR}", call.args[0]);
                    })
                    .expect("registry"),
            );
            let (code, action) = if worker {
                ("FEED_JOIN_FAILED", "feed.join_failed")
            } else {
                ("DUE_JOIN_FAILED", "due.join_failed")
            };
            assert_panic_boundary(&fixture, &recorder, &row, worker, code, action).await;
        }
    }
}

#[cfg(feature = "monty")]
#[tokio::test]
async fn monty_resolver_panics_are_value_free_in_due_and_worker_records_and_events() {
    for worker in [false, true] {
        let (fixture, recorder, row) = panic_fixture(
            HookTarget::monty(
                "def cron_stop(h): return {'action':'continue'}\ndef cron_tick(h): pass",
            )
            .expect("monty"),
            HookEnvironmentHost::new().with_reference_resolver(Arc::new(PanickingResolver)),
        );
        let (code, action) = if worker {
            ("FEED_JOIN_FAILED", "feed.join_failed")
        } else {
            ("DUE_JOIN_FAILED", "due.join_failed")
        };
        assert_panic_boundary(&fixture, &recorder, &row, worker, code, action).await;
    }
}

#[cfg(feature = "monty")]
#[tokio::test]
async fn registered_closure_reads_the_same_set_reference_clear_snapshot_in_both_phases() {
    let root = tempfile::tempdir().expect("root");
    let file = root.path().join("host.env");
    std::fs::write(&file, "CLEAR_ME=removed-canary\nFILE=file-canary\n").expect("file");
    let resolver = Arc::new(RecordingResolver::default());
    let observations = Arc::new(std::sync::Mutex::new(Vec::new()));
    let observed = observations.clone();
    let changed_file = file.clone();
    let registry = MontyHostRegistry::new()
        .register("observe", move |call: &MontyHostCall| {
            assert_eq!(call.environment_value("SET"), Some("set-canary"));
            assert_eq!(
                call.environment_value("REFERENCE"),
                Some("reference-canary")
            );
            assert_eq!(call.environment_value("FILE"), Some("file-canary"));
            assert_eq!(call.environment_value("CLEAR_ME"), None);
            assert_eq!(call.environment_value("ABSENT"), None);
            let entries = call
                .environment_variables()
                .collect::<std::collections::BTreeMap<_, _>>();
            assert_eq!(entries.len(), 3);
            assert_eq!(entries["REFERENCE"], "reference-canary");
            for value in [
                "set-canary",
                "reference-canary",
                "file-canary",
                "removed-canary",
                LOCATOR,
            ] {
                assert!(!format!("{call:?}").contains(value));
            }
            observed.lock().expect("observations").push(call.phase);
            if call.phase == HookEvent::Stop {
                std::fs::write(&changed_file, "FILE=changed-after-stop\n").expect("change file");
            }
            Ok(json!(call.environment_value("REFERENCE")))
        })
        .expect("registry");
    let mut fixture = common::Fixture::new();
    fixture.scheduler = fixture
        .scheduler
        .with_monty_hosts(registry)
        .with_hook_environment(
            HookEnvironmentHost::new()
                .allow_dotenv_file(&file)
                .expect("allow")
                .with_reference_resolver(resolver.clone()),
        );
    fixture.start(spec("custom-snapshot", HookTarget::monty("import os\ndef cron_stop(h):\n    assert observe()==os.getenv('REFERENCE')\n    return {'action':'continue'}\ndef cron_tick(h):\n    assert observe()==os.getenv('REFERENCE')\n").expect("monty"),
        HookEnvironment::new().dotenv(&file, true).expect("file")
            .set("SET", "set-canary").expect("set")
            .reference("REFERENCE", "value:reference-canary").expect("reference")
            .clear("CLEAR_ME").expect("clear")));
    assert_eq!(due(&fixture).await, RunOutcome::Ok);
    assert_eq!(
        *observations.lock().expect("observations"),
        [HookEvent::Stop, HookEvent::Tick]
    );
    assert_eq!(resolver.calls.lock().expect("calls").len(), 1);
}

#[cfg(unix)]
#[test]
fn authorized_fifo_without_writer_is_rejected_before_open_and_runtime_shutdown_is_bounded() {
    let output = super::owned_child::output(
        std::process::Command::new(std::env::current_exe().expect("self"))
            .args([
                "--exact",
                "failure_boundaries::fifo_without_writer_child",
                "--nocapture",
            ])
            .env("SC_CRON_FIFO_CHILD", "1"),
        Duration::from_secs(5),
    );
    assert!(
        output.status.success(),
        "{} {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[cfg(unix)]
#[test]
fn fifo_without_writer_child() {
    if std::env::var("SC_CRON_FIFO_CHILD").as_deref() != Ok("1") {
        return;
    }
    let root = tempfile::tempdir().expect("root");
    let file = root.path().join("no-writer.env");
    let output = super::owned_child::output(
        std::process::Command::new("mkfifo").arg(&file),
        Duration::from_secs(1),
    );
    assert!(output.status.success(), "mkfifo");
    let start = Instant::now();
    let runtime = tokio::runtime::Runtime::new().expect("runtime");
    runtime.block_on(async {
        for required in [false, true] {
            let mut fixture = common::Fixture::new();
            fixture.scheduler = fixture.scheduler.with_hook_environment(
                HookEnvironmentHost::new()
                    .allow_dotenv_file(&file)
                    .expect("host"),
            );
            fixture.start(spec(
                "fifo",
                HookTarget::python("never-executed.py").expect("hook"),
                HookEnvironment::new()
                    .dotenv(&file, required)
                    .expect("policy"),
            ));
            let outcome = tokio::time::timeout(Duration::from_secs(1), due(&fixture))
                .await
                .expect("prompt invalid file");
            assert_code(&outcome, "ENV_FILE_INVALID");
        }
    });
    drop(runtime);
    assert!(
        start.elapsed() < Duration::from_secs(2),
        "runtime teardown is bounded"
    );
}

const UNKNOWN_ACTION: &str = "unknown action; expected continue, disable or delete";
const REPLY_RECOVERY: &str = "Return an object with string \"action\" (continue, disable, or delete) and a non-empty \"reason\" when disabling or deleting.";
const ACTION_SECRETS: [&str; 4] = [
    "prod-secret\ncredential",
    "prod-secret\tcredential",
    "prod-secret\\credential",
    "prod-secret\"credential",
];

struct StopActionResolver(String);
#[async_trait::async_trait]
impl EnvironmentReferenceResolver for StopActionResolver {
    async fn resolve(
        &self,
        request: EnvironmentReferenceRequest<'_>,
    ) -> Result<Option<String>, EnvironmentReferenceError> {
        assert_eq!(request.reference(), "external-stop-action");
        Ok(Some(self.0.clone()))
    }
}

async fn assert_unknown_action_boundary(hook: HookTarget, secret: &str, worker: bool) {
    let mut fixture = common::Fixture::new();
    let recorder = Arc::new(sc_observability_types::RecordingCanonicalLog::new(
        sc_observability_types::ServiceName::new("stop-reply-canary").expect("service"),
    ));
    fixture.scheduler = Scheduler::new_with_logger(
        fixture.store.clone(),
        fixture.clock.clone(),
        Arc::new(SeqJitter::default()),
        recorder.clone(),
    )
    .with_hook_environment(
        HookEnvironmentHost::new()
            .with_reference_resolver(Arc::new(StopActionResolver(secret.to_owned()))),
    );
    let row = fixture.start(spec(
        "invalid-action",
        hook,
        HookEnvironment::new()
            .reference("KEY", "external-stop-action")
            .expect("reference"),
    ));
    let expected_detail = format!("stop check: HOST_REPLY: {UNKNOWN_ACTION}; {REPLY_RECOVERY}");
    let expected = RunOutcome::Failed(expected_detail.clone());
    let outcome = if worker {
        let handle = tokio::time::timeout(common::WAIT, fixture.worker(common::idle_job()))
            .await
            .expect("worker startup");
        tokio::time::timeout(common::WAIT, fixture.step(60))
            .await
            .expect("worker step");
        fixture.store.wait_recorded(&row, expected.clone()).await;
        common::shutdown(handle).await;
        fixture.row(&row).last_outcome.expect("worker recorded")
    } else {
        due(&fixture).await
    };
    assert_eq!(outcome, expected);
    let parts = outcome.failure_parts().expect("typed failure");
    assert_eq!(parts.code, Some("HOST_REPLY"));
    assert_eq!(parts.message, UNKNOWN_ACTION);
    assert_eq!(parts.recovery, Some(REPLY_RECOVERY));
    let stored = fixture.row(&row).last_outcome.expect("persisted outcome");
    assert_eq!(stored, expected);
    let events = recorder.events();
    let terminal = events
        .iter()
        .filter(|event| {
            event.action.as_str()
                == if worker {
                    "run.finished"
                } else {
                    "due.finished"
                }
        })
        .collect::<Vec<_>>();
    assert_eq!(terminal.len(), 1, "{events:?}");
    assert_eq!(terminal[0].fields["error"], expected_detail);
    assert_eq!(
        terminal[0].diagnostic.as_ref().map(|d| d.code.as_str()),
        Some("JOB_FAILED")
    );
    assert!(terminal[0].correlation_id.is_some());
    let escaped = serde_json::to_string(secret).expect("escaped canary");
    let escaped = &escaped[1..escaped.len() - 1];
    for detail in [
        format!("{stored:?}"),
        serde_json::to_string(&events).expect("events"),
    ] {
        for forbidden in [secret, escaped, "prod-secret"] {
            assert!(
                !detail.contains(forbidden),
                "resolved action leaked: {detail}"
            );
        }
    }
}

#[tokio::test]
async fn python_unknown_stop_actions_are_value_free_in_records_and_terminal_events() {
    let root = tempfile::tempdir().expect("hook root");
    let module = python_file(
        root.path(),
        "import os,json,sys\njson.load(sys.stdin)\nprint(json.dumps({'action':os.getenv('KEY')}))\n",
    );
    for secret in ACTION_SECRETS {
        for worker in [false, true] {
            assert_unknown_action_boundary(
                HookTarget::python(&module).expect("python"),
                secret,
                worker,
            )
            .await;
        }
    }
}

#[cfg(feature = "monty")]
#[tokio::test]
async fn monty_unknown_stop_actions_are_value_free_in_records_and_terminal_events() {
    let source =
        "import os\ndef cron_stop(h): return {'action':os.getenv('KEY')}\ndef cron_tick(h): pass";
    for secret in ACTION_SECRETS {
        for worker in [false, true] {
            assert_unknown_action_boundary(
                HookTarget::monty(source).expect("monty"),
                secret,
                worker,
            )
            .await;
        }
    }
}
