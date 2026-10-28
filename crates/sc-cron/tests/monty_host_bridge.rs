//! Monty `render_template` through [`sc_cron::Scheduler::run_due`].
//!
//! These tests call the public scheduler. They do not import the host module.
//! A missing program name is a comment in the hook source, read only when
//! feature `test-utils` is on; the tests do not change `PATH` or the environment.

#![allow(clippy::expect_used)]

mod common;

use std::time::Duration;

use common::{Fixture, idle_job};
use sc_cron::{HookTarget, RunOutcome, ScheduleSpec};
use serde_json::json;

fn spec(name: &str, source: &str, payload: serde_json::Value) -> ScheduleSpec {
    ScheduleSpec::with_hook(
        name,
        "0 * * * * *",
        Duration::from_secs(20),
        payload,
        HookTarget::monty(source).expect("source"),
    )
    .expect("spec")
}

async fn due(fixture: &Fixture) -> Vec<sc_cron::DueRun> {
    fixture.clock.advance(chrono::Duration::seconds(60));
    fixture
        .scheduler
        .run_due(idle_job())
        .await
        .expect("run_due")
}

fn one(runs: &[sc_cron::DueRun]) -> &RunOutcome {
    assert_eq!(runs.len(), 1, "{runs:?}");
    &runs[0].outcome
}

fn failed(runs: &[sc_cron::DueRun]) -> String {
    match one(runs) {
        RunOutcome::Failed(detail) => detail.clone(),
        other => panic!("expected Failed, got {other:?}"),
    }
}

#[tokio::test]
async fn render_template_returns_sc_compose_stdout() {
    let source = r#"
def cron_stop(hook):
    return {"action": "continue", "reason": "ok"}

def cron_tick(hook):
    first = render_template(hook["payload"]["template"], hook["payload"]["context"])
    second = render_template("Hi {{ name }}", {"name": "two"})
    if first != "Hello cron" and first != "Hello cron\n":
        raise RuntimeError("first:" + first)
    if second != "Hi two" and second != "Hi two\n":
        raise RuntimeError("second:" + second)
    return None
"#;
    let fixture = Fixture::new();
    fixture.start(spec(
        "sync",
        source,
        json!({
            "template": "Hello {{ name }}",
            "context": {"name": "cron"},
        }),
    ));
    let runs = due(&fixture).await;
    assert_eq!(one(&runs), &RunOutcome::Ok, "{runs:?}");
}

#[tokio::test]
async fn another_host_function_fails_the_feed() {
    let source = r#"
# sc-cron-test-host-fn: compose_file
def cron_stop(hook):
    return {"action": "continue", "reason": "ok"}

def cron_tick(hook):
    compose_file("t.j2", {})
    return None
"#;
    let fixture = Fixture::new();
    fixture.start(spec("sync", source, json!({})));
    let detail = failed(&due(&fixture).await);
    assert!(detail.contains("compose_file"), "{detail}");
    assert!(detail.contains("allowlisted"), "{detail}");
    assert!(!detail.contains("HOOK_FAILED"), "{detail}");
}

#[tokio::test]
async fn an_os_call_fails_the_feed() {
    let source = r#"
def cron_stop(hook):
    return {"action": "continue", "reason": "ok"}

def cron_tick(hook):
    open("t.j2")
    return None
"#;
    let fixture = Fixture::new();
    fixture.start(spec("sync", source, json!({})));
    let detail = failed(&due(&fixture).await);
    assert!(detail.contains("os call"), "{detail}");
    assert!(detail.contains("allowlisted"), "{detail}");
}

#[tokio::test]
async fn a_non_zero_compose_exit_is_failed() {
    let source = r#"
def cron_stop(hook):
    return {"action": "continue", "reason": "ok"}

def cron_tick(hook):
    render_template("Hello {{ name", {"name": "cron"})
    return None
"#;
    let fixture = Fixture::new();
    fixture.start(spec("sync", source, json!({})));
    let detail = failed(&due(&fixture).await);
    assert!(detail.contains("compose exited"), "{detail}");
    assert!(!detail.contains("HOOK_FAILED"), "{detail}");
}

#[tokio::test]
async fn a_missing_compose_program_is_failed_and_drops_the_repl() {
    let source = r#"
# sc-cron-test-compose-program: sc-cron-missing-compose-b10
n = 0

def cron_stop(hook):
    return {"action": "continue", "reason": "ok"}

def cron_tick(hook):
    global n
    n = n + 1
    if n > 1:
        raise RuntimeError("repl kept")
    render_template("Hello {{ name }}", {"name": "cron"})
    return None
"#;
    let fixture = Fixture::new();
    fixture.start(spec("sync", source, json!({})));
    let first = failed(&due(&fixture).await);
    assert!(first.contains("sc-cron-missing-compose-b10"), "{first}");
    assert!(!first.contains("HOOK_FAILED"), "{first}");
    assert!(!first.contains("repl kept"), "{first}");
    let second = failed(&due(&fixture).await);
    assert!(second.contains("sc-cron-missing-compose-b10"), "{second}");
    assert!(!second.contains("repl kept"), "{second}");
}

#[tokio::test]
async fn a_tick_that_does_not_render_runs_without_the_compose_program() {
    let source = r#"
# sc-cron-test-compose-program: sc-cron-missing-compose-b10
def cron_stop(hook):
    return {"action": "continue", "reason": "ok"}

def cron_tick(hook):
    return None
"#;
    let fixture = Fixture::new();
    fixture.start(spec("sync", source, json!({})));
    let runs = due(&fixture).await;
    assert_eq!(one(&runs), &RunOutcome::Ok, "{runs:?}");
}
