//! A Monty row with the `monty` feature off: it stores, and launch records
//! `HookFailed` without starting an interpreter.

#![cfg(not(feature = "monty"))]
#![expect(clippy::expect_used, reason = "integration test harness")]

mod common;

use common::{Fixture, idle_job, shutdown};
use sc_cron::{HookTarget, RunOutcome, ScheduleSpec};
use serde_json::json;
use std::time::Duration;

#[tokio::test]
async fn a_monty_row_without_the_feature_records_hook_failed() {
    let fixture = Fixture::new();
    let spec = ScheduleSpec::with_hook(
        "later",
        "0 * * * * *",
        Duration::from_secs(20),
        json!({}),
        HookTarget::monty("def cron_stop(hook):\n    return {'action': 'continue'}\n")
            .expect("source"),
    )
    .expect("spec");
    let schedule = fixture.start(spec);
    assert!(matches!(schedule.spec.hook, HookTarget::Monty { .. }));
    let worker = fixture.worker(idle_job()).await;
    fixture.step(60).await;
    fixture
        .store
        .wait_write("hook failed", |write| {
            matches!(write, common::Write::Run { outcome: RunOutcome::Failed(why), changed: true, .. }
                if why.contains("HOOK_FAILED"))
        })
        .await;
    shutdown(worker).await;
    let row = fixture.row(&schedule);
    assert!(
        matches!(row.last_outcome, Some(RunOutcome::Failed(ref why)) if why.contains("HOOK_FAILED")),
        "{:?}",
        row.last_outcome
    );
}
