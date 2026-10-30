//! Public scheduler tests for the immutable application registry and CLI adapters.
#![expect(clippy::expect_used, reason = "integration assertions")]

mod common;
use common::{Fixture, idle_job};
use sc_cron::{
    AtmHostConfig, CronError, HookEvent, HookTarget, MontyHostCall, MontyHostRegistry, RunOutcome,
    ScComposeHostConfig, ScheduleSpec,
};
use serde_json::{Value, json};
use std::sync::Arc;
use std::time::{Duration, Instant};

async fn run(registry: MontyHostRegistry, tick: &str, timeout: Duration) -> RunOutcome {
    let mut fixture = Fixture::new();
    fixture.scheduler = fixture.scheduler.with_monty_hosts(registry);
    let source = format!(
        "def cron_stop(hook):\n    return {{'action': 'continue'}}\n\ndef cron_tick(hook):\n{tick}\n"
    );
    fixture.start(
        ScheduleSpec::with_hook(
            "host-test",
            "* * * * *",
            timeout,
            json!({}),
            HookTarget::monty(&source).expect("source"),
        )
        .expect("spec"),
    );
    fixture.clock.advance(chrono::Duration::seconds(60));
    let runs = tokio::time::timeout(common::WAIT, fixture.scheduler.run_due(idle_job()))
        .await
        .expect("bounded feed")
        .expect("run due");
    assert_eq!(runs.len(), 1);
    runs[0].outcome.clone()
}
fn code(outcome: &RunOutcome, expected: &str) {
    match outcome {
        RunOutcome::Failed(detail) => assert!(detail.starts_with(expected), "{detail}"),
        other => panic!("expected {expected}, got {other:?}"),
    }
}
const BUDGET: Duration = Duration::from_secs(10);

#[tokio::test]
async fn custom_host_without_cli() {
    let host = |call: &MontyHostCall| {
        assert_eq!(call.name, "jev_call");
        assert_eq!(call.schedule.as_str(), "host-test");
        assert_eq!(call.phase, HookEvent::Tick);
        assert!(call.deadline > Instant::now());
        assert_eq!(call.args, [json!({"x": [1, true, null]})]);
        assert_eq!(call.kwargs.get("label"), Some(&json!("ok")));
        Ok(json!({"answer": 42}))
    };
    let registry = MontyHostRegistry::new()
        .register("jev_call", host)
        .expect("register");
    assert_eq!(run(registry, "    a = jev_call({'x': [1, True, None]}, label='ok')\n    b = jev_call({'x': [1, True, None]}, label='ok')\n    if a['answer'] != 42 or b != a:\n        raise RuntimeError('result')", BUDGET).await, RunOutcome::Ok);
}
#[tokio::test]
async fn handler_error_is_typed_even_when_invalid_spec() {
    let registry = MontyHostRegistry::new()
        .register("fail", |_: &MontyHostCall| {
            Err(CronError::InvalidSpec(
                "HOST_DENIED: arbitrary prose".into(),
            ))
        })
        .expect("register");
    code(
        &run(registry, "    fail()", BUDGET).await,
        "HOST_CALL_FAILED",
    );
}
#[test]
fn invalid_duplicate_and_reserved_registration() {
    for name in ["", "a-b", "1bad", "__hook", "cron_stop", "cron_tick"] {
        assert!(
            matches!(
                MontyHostRegistry::new().register(name, |_: &MontyHostCall| Ok(Value::Null)),
                Err(CronError::InvalidSpec(_))
            ),
            "{name}"
        );
    }
    let registry = MontyHostRegistry::new()
        .register("a", |_: &MontyHostCall| Ok(Value::Null))
        .expect("a");
    assert!(
        registry
            .register("a", |_: &MontyHostCall| Ok(Value::Null))
            .is_err()
    );
    assert!(ScComposeHostConfig::new("").is_err());
    assert!(AtmHostConfig::new("").is_err());
}
#[tokio::test]
async fn render_helper_not_registered() {
    let registry = MontyHostRegistry::new();
    assert_eq!(registry.names().count(), 0);
    code(
        &run(registry, "    render_template('x', {})", BUDGET).await,
        "HOST_UNDEFINED",
    );
    let registry = MontyHostRegistry::new()
        .with_atm(AtmHostConfig::default())
        .expect("atm");
    assert_eq!(registry.names().collect::<Vec<_>>(), ["atm_run"]);
}
#[tokio::test]
async fn source_cannot_install_or_replace_a_handler() {
    code(
        &run(MontyHostRegistry::new(), "    register('evil', 1)", BUDGET).await,
        "HOST_UNDEFINED",
    );
    let registry = MontyHostRegistry::new()
        .register("custom", |_: &MontyHostCall| Ok(json!(9)))
        .expect("custom");
    assert_eq!(
        run(
            registry,
            "    custom = 4\n    if custom != 4:\n        raise RuntimeError('local variable')",
            BUDGET
        )
        .await,
        RunOutcome::Ok
    );
}

#[cfg(unix)]
fn executable(body: &str) -> (tempfile::TempDir, std::path::PathBuf) {
    use std::os::unix::fs::PermissionsExt;
    let dir = tempfile::tempdir().expect("fixture");
    let path = dir.path().join("cli");
    std::fs::write(&path, format!("#!/bin/sh\n{body}\n")).expect("write fixture");
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o700)).expect("chmod");
    (dir, path)
}
#[cfg(unix)]
fn compose(program: &std::path::Path) -> MontyHostRegistry {
    MontyHostRegistry::new()
        .with_sc_compose(ScComposeHostConfig::new(program).expect("config"))
        .expect("compose")
}
#[cfg(unix)]
#[tokio::test]
async fn compose_non_render_argv() {
    let (_dir, program) = executable("printf '%s\\n' \"$@\"");
    let source = "    result = sc_compose_run('extract', ['--root', '/unchanged path', '$(echo bad)', '--flag=x'])\n    if result['stdout'] != 'extract\\n--root\\n/unchanged path\\n$(echo bad)\\n--flag=x\\n':\n        raise RuntimeError(result['stdout'])";
    assert_eq!(run(compose(&program), source, BUDGET).await, RunOutcome::Ok);
}
#[cfg(unix)]
#[tokio::test]
async fn atm_argv() {
    let (_dir, program) = executable("printf '%s\\n' \"$@\"");
    let registry = MontyHostRegistry::new()
        .with_atm(AtmHostConfig::new(program).expect("config"))
        .expect("atm");
    assert_eq!(run(registry, "    result = atm_run('agent', ['list'])\n    if result['stdout'] != 'agent\\nlist\\n':\n        raise RuntimeError('argv')", BUDGET).await, RunOutcome::Ok);
}
#[cfg(unix)]
#[tokio::test]
async fn cli_nonzero_result() {
    let (_dir, program) = executable("printf 'out'; printf 'err' >&2; exit 7");
    assert_eq!(run(compose(&program), "    result = sc_compose_run('any')\n    if result != {'stdout': 'out', 'stderr': 'err', 'exit_code': 7}:\n        raise RuntimeError('result')", BUDGET).await, RunOutcome::Ok);
}
#[cfg(unix)]
#[tokio::test]
async fn cli_cwd_and_empty_environment() {
    let (dir, program) = executable("printf '%s\\n' \"${HOME-absent}\"; pwd");
    let registry = MontyHostRegistry::new()
        .with_sc_compose(
            ScComposeHostConfig::new(&program)
                .expect("config")
                .with_working_directory(dir.path())
                .expect("cwd"),
        )
        .expect("registry");
    let cwd = dir.path().canonicalize().expect("canonical cwd");
    let source = format!(
        "    result = sc_compose_run('anything')\n    if result['stdout'] != {}:\n        raise RuntimeError(result['stdout'])",
        serde_json::to_string(&format!("absent\n{}\n", cwd.display())).expect("literal")
    );
    assert_eq!(run(registry, &source, BUDGET).await, RunOutcome::Ok);
}
#[cfg(unix)]
#[tokio::test]
async fn invalid_adapter_shape_is_denied_before_spawn() {
    for invocation in [
        "sc_compose_run('')",
        "sc_compose_run(1)",
        "sc_compose_run('x', [1])",
        "sc_compose_run('x', args=[])",
        "sc_compose_run('x', [], 3)",
    ] {
        let registry = compose(std::path::Path::new("/absent/b16-cli"));
        code(
            &run(registry, &format!("    {invocation}"), BUDGET).await,
            "HOST_DENIED",
        );
    }
}
#[cfg(unix)]
#[tokio::test]
async fn cli_spawn_failure() {
    code(
        &run(
            compose(std::path::Path::new("/absent/b16-cli")),
            "    sc_compose_run('anything')",
            BUDGET,
        )
        .await,
        "HOST_SPAWN",
    );
}
#[cfg(unix)]
#[tokio::test]
async fn cli_deadline() {
    let (dir, program) = executable(
        "printf 'started' > \"$0.started\"; /bin/sleep 20; printf 'finished' > \"$0.finished\"",
    );
    assert_eq!(
        run(
            compose(&program),
            "    sc_compose_run('anything')",
            Duration::from_secs(2)
        )
        .await,
        RunOutcome::TimedOut
    );
    // A row can expire before the executable reaches its bootstrap marker.
    // The started-child driver test separately proves active group cleanup.
    if dir.path().join("cli.started").exists() {
        assert_eq!(
            std::fs::read_to_string(dir.path().join("cli.started")).expect("start marker"),
            "started"
        );
    }
    assert!(!dir.path().join("cli.finished").exists());
}
#[cfg(unix)]
#[tokio::test]
async fn cli_output_cap() {
    for stream in ["", " >&2"] {
        let body =
            format!("i=0; while [ \"$i\" -lt 70000 ]; do printf x{stream}; i=$((i+1)); done");
        let (_dir, program) = executable(&body);
        code(
            &run(compose(&program), "    sc_compose_run('anything')", BUDGET).await,
            "HOST_OUTPUT",
        );
    }
}
#[tokio::test]
async fn callback_deadline_checked_after_return() {
    let registry = MontyHostRegistry::new()
        .register("late", |call: &MontyHostCall| {
            std::thread::park_timeout(
                call.deadline.saturating_duration_since(Instant::now()) + Duration::from_millis(10),
            );
            Ok(Value::Null)
        })
        .expect("late");
    assert_eq!(
        run(registry, "    late()", Duration::from_millis(100)).await,
        RunOutcome::TimedOut
    );
}

#[test]
fn registry_order_is_irrelevant_and_handler_trait_is_object_safe() {
    let _: Arc<dyn sc_cron::MontyHostHandler> = Arc::new(|_: &MontyHostCall| Ok(Value::Null));
    let a = MontyHostRegistry::new()
        .register("z", |_: &MontyHostCall| Ok(Value::Null))
        .expect("z")
        .register("a", |_: &MontyHostCall| Ok(Value::Null))
        .expect("a");
    let b = MontyHostRegistry::new()
        .register("a", |_: &MontyHostCall| Ok(Value::Null))
        .expect("a")
        .register("z", |_: &MontyHostCall| Ok(Value::Null))
        .expect("z");
    assert_eq!(a.names().collect::<Vec<_>>(), b.names().collect::<Vec<_>>());
}
