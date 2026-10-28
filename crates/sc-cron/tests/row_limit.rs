//! Embedder schedule row limit (ADR-RUN-0014).
//!
//! The clock is manual. Nothing waits on a timer.

#![expect(
    clippy::expect_used,
    reason = "test code: a failed expectation is the test failure"
)]

use std::num::NonZeroUsize;
use std::sync::Arc;
use std::time::Duration;

use sc_cron::{
    CronError, ManualClock, ScheduleLimits, ScheduleSpec, ScheduleStore, Scheduler, SeqJitter,
    SqliteScheduleStore,
};

fn limits(rows: usize) -> ScheduleLimits {
    ScheduleLimits {
        max_schedule_rows: NonZeroUsize::new(rows),
    }
}

fn open(
    path: &std::path::Path,
    rows: ScheduleLimits,
) -> Scheduler<SqliteScheduleStore, ManualClock, SeqJitter> {
    Scheduler::open(
        path,
        rows,
        Arc::new(ManualClock::new(
            "2026-10-19T00:00:00Z".parse().expect("timestamp"),
        )),
        Arc::new(SeqJitter::default()),
    )
    .expect("open")
}

fn spec(name: &str) -> ScheduleSpec {
    ScheduleSpec::new(
        name,
        "0 * * * * *",
        Duration::from_secs(20),
        serde_json::json!({}),
    )
    .expect("spec")
}

fn invalid_spec(error: CronError) -> String {
    match error {
        CronError::InvalidSpec(detail) => detail,
        other => panic!("expected InvalidSpec, got {other}"),
    }
}

#[test]
fn at_the_limit_a_new_name_is_invalid_and_replace_is_not() {
    let root = tempfile::tempdir().expect("tempdir");
    let path = root.path().join("cron.db");
    let scheduler = open(&path, limits(1));
    assert_eq!(scheduler.limits(), limits(1));
    assert_eq!(scheduler.store().limits(), limits(1));

    let first = scheduler.start(spec("alpha")).expect("first");
    scheduler
        .set_enabled(&first.spec.name, false)
        .expect("disable");
    let error = scheduler.start(spec("beta")).expect_err("over limit");
    assert_eq!(
        error.recovery(),
        "stop or delete a schedule to free a row, or raise the schedule row limit (0 or null means unlimited)"
    );
    let detail = invalid_spec(error);
    assert_eq!(scheduler.store().list_active().expect("list").len(), 1);
    assert!(detail.contains("schedule row limit is 1"), "{detail}");
    assert!(detail.contains("current row count is 1"), "{detail}");
    assert_eq!(
        scheduler
            .start(spec("beta"))
            .expect_err("still over")
            .code(),
        "INVALID_SPEC"
    );

    let replaced = scheduler
        .replace_and_set_enabled(spec("alpha"), true, &[])
        .expect("replace");
    assert_eq!(replaced.spec.name.as_str(), "alpha");
    assert_eq!(scheduler.store().list_active().expect("list").len(), 1);
    let still = invalid_spec(
        scheduler
            .start(spec("beta"))
            .expect_err("replace kept the count"),
    );
    assert!(still.contains("current row count is 1"), "{still}");
}

#[test]
fn delete_frees_a_slot_for_a_new_name() {
    let root = tempfile::tempdir().expect("tempdir");
    let scheduler = open(&root.path().join("cron.db"), limits(1));
    let first = scheduler.start(spec("alpha")).expect("first");
    scheduler
        .stop(first.id, first.generation, "deleted")
        .expect("delete");
    assert!(scheduler.store().list_active().expect("list").is_empty());
    scheduler.start(spec("beta")).expect("slot freed");
    assert_eq!(scheduler.store().list_active().expect("list").len(), 1);
}

#[test]
fn restarting_an_ended_name_claims_a_slot() {
    let root = tempfile::tempdir().expect("tempdir");
    let path = root.path().join("cron.db");
    let scheduler = open(&path, limits(1));
    let first = scheduler.start(spec("alpha")).expect("first");
    scheduler
        .stop(first.id, first.generation, "deleted")
        .expect("delete");
    scheduler.start(spec("beta")).expect("other name");
    let detail = invalid_spec(
        scheduler
            .replace_and_set_enabled(spec("alpha"), true, &[])
            .expect_err("ended name at the limit"),
    );
    assert!(detail.contains("current row count is 1"), "{detail}");
    assert_eq!(scheduler.store().list_active().expect("list").len(), 1);
}

#[test]
fn unlimited_accepts_more_rows_than_the_template_default() {
    let root = tempfile::tempdir().expect("tempdir");
    let scheduler = open(&root.path().join("cron.db"), ScheduleLimits::default());
    assert!(scheduler.limits().max_schedule_rows.is_none());
    for index in 0..12 {
        scheduler
            .start(spec(&format!("n{index}")))
            .expect("unlimited");
    }
    assert_eq!(scheduler.store().list_active().expect("list").len(), 12);

    let capped = open(&root.path().join("capped.db"), limits(10));
    for index in 0..10 {
        capped.start(spec(&format!("c{index}"))).expect("under ten");
    }
    let detail = invalid_spec(capped.start(spec("overflow")).expect_err("eleventh"));
    assert!(detail.contains("schedule row limit is 10"), "{detail}");
    assert!(detail.contains("current row count is 10"), "{detail}");
}

#[test]
fn lowering_the_limit_keeps_existing_rows() {
    let root = tempfile::tempdir().expect("tempdir");
    let path = root.path().join("cron.db");
    {
        let scheduler = open(&path, ScheduleLimits::default());
        scheduler.start(spec("alpha")).expect("alpha");
        scheduler.start(spec("beta")).expect("beta");
        scheduler.start(spec("gamma")).expect("gamma");
    }
    let scheduler = open(&path, limits(1));
    assert_eq!(scheduler.store().list_active().expect("kept").len(), 3);
    let detail = invalid_spec(scheduler.start(spec("delta")).expect_err("new name"));
    assert!(detail.contains("schedule row limit is 1"), "{detail}");
    assert!(detail.contains("current row count is 3"), "{detail}");
    scheduler
        .replace_and_set_enabled(spec("alpha"), false, &[])
        .expect("replace existing");
    assert_eq!(
        scheduler.store().list_active().expect("still kept").len(),
        3
    );
}
