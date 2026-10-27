//! A hook target stored on each schedule row, and the one-shot run of what is
//! due (`Scheduler::run_due`), through the public API on a manual clock over
//! the recording store.
//!
//! The Python rows use the stub module of `common`, run by `python3`, so the
//! tests that need one run on Unix.

#![expect(
    clippy::clone_on_ref_ptr,
    reason = "test code: the Arcs are cloned for each closure"
)]

mod common;

use std::path::PathBuf;
use std::sync::Arc;

use common::{Fixture, ScriptedJob, shutdown};
use sc_cron::{HookTarget, RunOutcome};

/// A stored Python path that cannot be started: the run is recorded `Failed`
/// and the row stays active and enabled.
#[tokio::test]
async fn an_unresolvable_row_is_recorded_failed_and_stays_active_and_enabled() {
    let fixture = Fixture::new();
    let mut spec = fixture.spec("unresolvable");
    spec.hook = HookTarget::Python {
        module: PathBuf::from("-not-a-path"),
    };
    let schedule = fixture.start(spec);
    let parks = fixture.clock.parks();
    let worker = fixture.scheduler.spawn_worker(Arc::new(ScriptedJob::new()));
    fixture.clock.wait_parks_above(parks).await;
    fixture.step(60).await;
    let id = schedule.id;
    fixture
        .store
        .wait_write("a failed resolve recorded", |write| {
            matches!(write, common::Write::Run { id: ran, outcome: RunOutcome::Failed(why), changed: true }
                if *ran == id && why.starts_with("resolve: "))
        })
        .await;
    shutdown(worker).await;
    let row = fixture.row(&schedule);
    assert!(row.is_active() && row.enabled, "{row:?}");
    assert!(
        matches!(row.last_outcome, Some(RunOutcome::Failed(_))),
        "{row:?}"
    );
}

/// A Rust row runs the worker's job.
#[tokio::test]
async fn a_resolved_row_runs_its_target() {
    let fixture = Fixture::new();
    let job = Arc::new(ScriptedJob::new());
    let schedule = fixture.start(fixture.spec("resolved"));
    let parks = fixture.clock.parks();
    let worker = fixture.scheduler.spawn_worker(job.clone());
    fixture.clock.wait_parks_above(parks).await;
    fixture.step(60).await;
    fixture.store.wait_recorded(&schedule, RunOutcome::Ok).await;
    shutdown(worker).await;
    assert_eq!(job.runs(), 1);
}

/// `run_due` runs a due row and leaves a row that is not due alone.
#[tokio::test]
async fn run_due_runs_what_is_due_and_skips_what_is_not() {
    let fixture = Fixture::new();
    let job = Arc::new(ScriptedJob::new());
    let due = fixture.start(fixture.spec("due"));
    let later = fixture.start(
        sc_cron::ScheduleSpec::new(
            "later",
            "0 0 * * * *",
            std::time::Duration::from_secs(20),
            serde_json::json!({}),
        )
        .expect("spec"),
    );
    // 00:01:00: `due` (every minute) is due, `later` (hourly) is not.
    fixture.clock.advance(chrono::Duration::seconds(60));
    let runs = fixture
        .scheduler
        .run_due(job.clone())
        .await
        .expect("run due");
    assert_eq!(runs.len(), 1, "{runs:?}");
    assert_eq!(runs[0].name, due.spec.name);
    assert_eq!(runs[0].outcome, RunOutcome::Ok);
    assert_eq!(job.runs(), 1);
    let ran = fixture.row(&due);
    assert_eq!(ran.last_outcome, Some(RunOutcome::Ok));
    assert_eq!(ran.next_run_at, Some(common::ts("2026-10-19T00:02:00Z")));
    let untouched = fixture.row(&later);
    assert_eq!(untouched.last_outcome, None);
    assert_eq!(untouched.next_run_at, later.next_run_at);
}

#[cfg(unix)]
mod python {
    use super::*;
    use common::Stub;

    /// One table holds a Rust row and a Python row; each runs its own target.
    #[tokio::test]
    async fn one_table_runs_a_rust_row_and_a_python_row_each_with_its_own_target() {
        let fixture = Fixture::new();
        let stub = Stub::new();
        stub.reply("stop", r#"{"action": "continue"}"#);
        let job = Arc::new(ScriptedJob::new());
        let rust_row = fixture.start(fixture.spec("rust-row"));
        let python_row = fixture.start(fixture.python_spec("python-row", stub.module()));
        let parks = fixture.clock.parks();
        let worker = fixture.scheduler.spawn_worker(job.clone());
        fixture.clock.wait_parks_above(parks).await;
        fixture.step(60).await;
        fixture.store.wait_recorded(&rust_row, RunOutcome::Ok).await;
        fixture
            .store
            .wait_recorded(&python_row, RunOutcome::Ok)
            .await;
        shutdown(worker).await;
        assert_eq!(job.runs(), 1, "the Rust row ran its job once");
        let events: Vec<String> = stub
            .calls()
            .iter()
            .map(|c| {
                let v: serde_json::Value = serde_json::from_str(&c.stdin).expect("stdin");
                v["event"].as_str().unwrap_or_default().to_owned()
            })
            .collect();
        assert_eq!(
            events,
            ["cron.stop", "cron.tick"],
            "the Python row ran its module"
        );
    }
}
