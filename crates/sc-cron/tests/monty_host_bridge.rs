//! Monty `render_template` and `sc_compose_run` through [`sc_cron::Scheduler::run_due`].
//!
//! These tests call the public scheduler. They do not import the host module,
//! they do not change `PATH` or the environment, and they do not call
//! `Scheduler::with_monty_host`. They therefore spawn the default basename
//! `sc-compose` on `PATH` and register `render_template` and `sc_compose_run`.
//! An embedder sets another program with `MontyHostConfig`. Hook source cannot
//! choose the program, the temp root, or another host function. Crate-private
//! injection of those values lives in `monty_host`'s unit tests
//! (`execute_feed_with_compose_program`).

#![expect(clippy::expect_used, reason = "integration test harness")]

mod common;

use std::time::Duration;

use common::{Fixture, idle_job};
use sc_cron::{HookTarget, RunOutcome, ScheduleSpec};
use serde_json::json;

fn spec(name: &str, source: &str, payload: serde_json::Value) -> ScheduleSpec {
    ScheduleSpec::with_hook(
        name,
        "* * * * *",
        Duration::from_secs(20),
        payload,
        HookTarget::monty(source).expect("source"),
    )
    .expect("spec")
}

async fn due(fixture: &Fixture) -> Vec<sc_cron::DueRun> {
    fixture.clock.advance(chrono::Duration::seconds(60));
    match tokio::time::timeout(common::WAIT, fixture.scheduler.run_due(idle_job())).await {
        Ok(result) => result.expect("run_due"),
        Err(_elapsed) => panic!(
            "run_due did not finish within {:?}; sc-compose may be hung",
            common::WAIT
        ),
    }
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
    assert!(detail.contains("HOST_DENIED"), "{detail}");
    assert!(detail.contains("host function"), "{detail}");
    assert!(detail.contains("allowlisted"), "{detail}");
    assert!(!detail.contains("HOOK_FAILED"), "{detail}");
}

#[tokio::test]
async fn an_undefined_name_names_itself() {
    let source = r#"
def cron_stop(hook):
    return {"action": "continue", "reason": "ok"}

def cron_tick(hook):
    x = not_defined
    return x
"#;
    let fixture = Fixture::new();
    fixture.start(spec("sync", source, json!({})));
    let detail = failed(&due(&fixture).await);
    assert!(detail.contains("not_defined"), "{detail}");
    assert!(detail.contains("HOST_UNDEFINED"), "{detail}");
    assert!(!detail.contains("allowlisted"), "{detail}");
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
    assert!(detail.contains("open"), "{detail}");
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
    assert!(detail.contains("COMPOSE_EXIT"), "{detail}");
    assert!(detail.contains("compose exited"), "{detail}");
    assert!(!detail.contains("HOOK_FAILED"), "{detail}");
}

#[tokio::test]
async fn a_tick_that_does_not_render_is_ok() {
    let source = r#"
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
