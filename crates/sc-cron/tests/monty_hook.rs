//! Monty hooks through the worker and `run_due`. Time is a manual clock.
//! The interpreter runs on the blocking pool in real time. A test waits on a
//! store write or on the scheduler's Monty feed watch, bounded by
//! [`common::WAIT`].

#![expect(clippy::expect_used, reason = "integration test harness")]

mod common;

use std::sync::Arc;
use std::time::{Duration, Instant};

use async_trait::async_trait;
use common::{Fixture, Write, idle_job, shutdown};
use sc_cron::{
    Clock, HookTarget, Job, RunOutcome, Schedule, ScheduleSpec, ScheduleStore, StopDecision,
};
use serde_json::{Value, json};

const FAST: &str = r#"
def cron_stop(hook):
    return {"action": "continue", "reason": "ok"}

def cron_tick(hook):
    return None
"#;

const STATE: &str = r#"
n = 0

def cron_stop(hook):
    return {"action": "continue", "reason": "ok"}

def cron_tick(hook):
    global n
    n = n + 1
    if n == 2:
        raise RuntimeError("seen 2")
    return None
"#;

const JSON_HOOK: &str = r#"
def cron_stop(hook):
    check(hook, "cron.stop")
    return {"action": "continue", "reason": "ok"}

def cron_tick(hook):
    check(hook, "cron.tick")
    return None

def check(hook, event):
    if hook["protocol"] != "sc-cron.hook.v1":
        raise ValueError("protocol")
    if hook["event"] != event:
        raise ValueError("event")
    if hook["schedule"] != "sync":
        raise ValueError("schedule")
    payload = hook["payload"]
    if payload["scope"] != "a":
        raise ValueError("scope")
    if payload["n"][0] != 1:
        raise ValueError("n")
    if hook["deadline"] == "":
        raise ValueError("deadline")
    if "beads" in hook or "team" in hook or "phase" in hook or "atm" in hook:
        raise ValueError("extra")
"#;

/// A stop check that stays inside the interpreter long enough for a reload.
const HOLD_STOP: &str = r#"
def cron_stop(hook):
    i = 0
    while i < 400000:
        i = i + 1
    return {"action": "continue", "reason": "ok"}

def cron_tick(hook):
    raise RuntimeError("old-tick")
"#;

const NEW_SOURCE: &str = r#"
def cron_stop(hook):
    return {"action": "continue", "reason": "ok"}

def cron_tick(hook):
    raise RuntimeError("new-source")
"#;

const DISABLE: &str = r#"
n = 0

def cron_stop(hook):
    global n
    n = n + 1
    if n != 1:
        raise RuntimeError("repl survived")
    return {"action": "disable", "reason": "bye"}

def cron_tick(hook):
    raise RuntimeError("tick")
"#;

const BAD_ACTION: &str = r#"
def cron_stop(hook):
    return {"action": "nope", "reason": "x"}

def cron_tick(hook):
    raise RuntimeError("tick-ran")
"#;

const TIME_LIMIT: &str = r#"
n = 0

def cron_stop(hook):
    global n
    n = n + 1
    if n > 1:
        raise RuntimeError("repl kept")
    while True:
        pass
    return {"action": "continue", "reason": "ok"}

def cron_tick(hook):
    return None
"#;

const RAISES: &str = r#"
n = 0

def cron_stop(hook):
    global n
    n = n + 1
    if n > 1:
        raise RuntimeError("repl kept")
    raise RuntimeError("boom")

def cron_tick(hook):
    return None
"#;

const SYNTAX: &str = "def (\n";

const MISSING_FN: &str = r#"
def cron_tick(hook):
    return None
"#;

fn spec(name: &str, source: &str, timeout: Duration) -> ScheduleSpec {
    ScheduleSpec::with_hook(
        name,
        "0 * * * * *",
        timeout,
        json!({"scope": "a", "n": [1, 2]}),
        HookTarget::monty(source).expect("source"),
    )
    .expect("spec")
}

fn count_runs(writes: &[Write], schedule: &Schedule, needle: Option<&str>) -> usize {
    writes
        .iter()
        .filter(|write| match write {
            Write::Run {
                id,
                outcome: RunOutcome::Failed(why),
                changed: true,
            } if *id == schedule.id => needle.is_none_or(|needle| why.contains(needle)),
            Write::Run {
                id, changed: true, ..
            } if *id == schedule.id && needle.is_none() => true,
            Write::Run {
                id,
                outcome: RunOutcome::TimedOut,
                changed: true,
            } if *id == schedule.id && needle == Some("timed_out") => true,
            _ => false,
        })
        .count()
}

async fn wait_run_count(
    fixture: &Fixture,
    schedule: &Schedule,
    needle: Option<&str>,
    count: usize,
) {
    fixture
        .store
        .wait_writes(&format!("{count} runs ({needle:?})"), |writes| {
            count_runs(writes, schedule, needle) >= count
        })
        .await;
}

async fn wait_feeds_settled(feeds: &mut tokio::sync::watch::Receiver<usize>) {
    let settled = tokio::time::timeout(common::WAIT, feeds.wait_for(|level| *level == 0))
        .await
        .is_ok_and(|ready| ready.is_ok());
    let level = *feeds.borrow();
    assert!(
        settled,
        "timed out waiting for monty feeds to finish; still {level}"
    );
}

async fn wait_launch_stored(fixture: &Fixture, schedule: &Schedule, before: usize) {
    let id = schedule.id;
    fixture
        .store
        .wait_writes("next run stored for launch", move |writes| {
            writes.iter().skip(before).any(|write| {
                matches!(write, Write::Next { id: written, changed: true } if *written == id)
            })
        })
        .await;
}

fn runs_containing(fixture: &Fixture, schedule: &Schedule, needle: &str) -> usize {
    fixture
        .store
        .writes()
        .iter()
        .filter(|write| {
            matches!(write, Write::Run { id, outcome: RunOutcome::Failed(why), changed: true }
                if *id == schedule.id && why.contains(needle))
        })
        .count()
}

async fn wait_failed(fixture: &Fixture, schedule: &Schedule, needle: &str) {
    let id = schedule.id;
    let needle = needle.to_owned();
    fixture
        .store
        .wait_write("failed run", move |write| {
            matches!(write, Write::Run { id: written, outcome: RunOutcome::Failed(why), changed: true }
                if *written == id && why.contains(&needle))
        })
        .await;
}

struct Hang;

#[async_trait]
impl Job for Hang {
    async fn run(&self, _payload: &Value, _deadline: Instant) -> RunOutcome {
        std::future::pending().await
    }

    async fn should_stop(&self, _payload: &Value) -> StopDecision {
        StopDecision::Continue
    }
}

#[tokio::test]
async fn state_survives_ticks_of_the_same_generation() {
    let fixture = Fixture::new();
    let schedule = fixture.start(spec("sync", STATE, Duration::from_secs(5)));
    let worker = fixture.worker(idle_job()).await;
    fixture.step(60).await;
    fixture.store.wait_recorded(&schedule, RunOutcome::Ok).await;
    fixture.step(60).await;
    wait_failed(&fixture, &schedule, "seen 2").await;
    shutdown(worker).await;
}

#[tokio::test]
async fn the_hook_object_matches_the_protocol() {
    let fixture = Fixture::new();
    let schedule = fixture.start(spec("sync", JSON_HOOK, Duration::from_secs(5)));
    let worker = fixture.worker(idle_job()).await;
    fixture.step(60).await;
    fixture.store.wait_recorded(&schedule, RunOutcome::Ok).await;
    shutdown(worker).await;
}

#[tokio::test]
async fn a_replace_during_stop_compiles_the_new_source_on_the_next_run() {
    let fixture = Fixture::new();
    let schedule = fixture.start(spec("sync", HOLD_STOP, Duration::from_secs(15)));
    let worker = fixture.worker(idle_job()).await;
    fixture.step(60).await;
    let replaced = fixture
        .scheduler
        .replace_and_set_enabled_async(
            spec("sync", NEW_SOURCE, Duration::from_secs(5)),
            true,
            Vec::new(),
        )
        .await
        .expect("replace");
    fixture.step(60).await;
    wait_failed(&fixture, &replaced, "new-source").await;
    shutdown(worker).await;
    assert_eq!(runs_containing(&fixture, &schedule, "old-tick"), 0);
}

#[tokio::test]
async fn an_external_delete_during_stop_skips_the_tick_and_writes_no_outcome() {
    let fixture = Fixture::new();
    let schedule = fixture.start(spec("sync", HOLD_STOP, Duration::from_secs(15)));
    let mut feeds = fixture.scheduler.subscribe_monty_feeds();
    let worker = fixture.worker(idle_job()).await;
    fixture.step(60).await;
    fixture
        .scheduler
        .stop_async(schedule.id, schedule.generation, "gone")
        .await
        .expect("stop");
    assert!(
        *feeds.borrow() >= 1,
        "the stop check finished before the external stop"
    );
    wait_feeds_settled(&mut feeds).await;
    shutdown(worker).await;
    let row = fixture.row(&schedule);
    assert!(row.last_outcome.is_none(), "{:?}", row.last_outcome);
    assert!(!row.is_active());
    assert_eq!(runs_containing(&fixture, &schedule, "old-tick"), 0);
}

#[tokio::test]
async fn an_external_disable_during_stop_skips_the_tick() {
    let fixture = Fixture::new();
    let schedule = fixture.start(spec("sync", HOLD_STOP, Duration::from_secs(15)));
    let mut feeds = fixture.scheduler.subscribe_monty_feeds();
    let worker = fixture.worker(idle_job()).await;
    fixture.step(60).await;
    fixture
        .scheduler
        .set_enabled_async(&schedule.spec.name, false)
        .await
        .expect("disable");
    assert!(
        *feeds.borrow() >= 1,
        "the stop check finished before the external disable"
    );
    wait_feeds_settled(&mut feeds).await;
    shutdown(worker).await;
    let row = fixture.row(&schedule);
    assert!(row.last_outcome.is_none(), "{:?}", row.last_outcome);
    assert!(!row.enabled);
    assert_eq!(runs_containing(&fixture, &schedule, "old-tick"), 0);
}

#[tokio::test]
async fn shutdown_during_continue_leaves_the_outcome_unchanged() {
    let fixture = Fixture::new();
    let schedule = fixture.start(spec("sync", HOLD_STOP, Duration::from_secs(15)));
    let worker = fixture.worker(idle_job()).await;
    fixture.step(60).await;
    let stopped = worker
        .shutdown(tokio::time::Instant::now() + Duration::from_secs(10))
        .await;
    assert!(stopped, "the hold finished before the deadline");
    let row = fixture.row(&schedule);
    assert!(row.last_outcome.is_none(), "{:?}", row.last_outcome);
    assert_eq!(runs_containing(&fixture, &schedule, "old-tick"), 0);
}

#[tokio::test]
async fn a_handle_deadline_aborts_rust_siblings_and_waits_for_monty() {
    let fixture = Fixture::new();
    let quick = fixture.start(spec("quick", FAST, Duration::from_secs(5)));
    let slow = fixture.start(spec("slow", TIME_LIMIT, Duration::from_secs(2)));
    let rust = fixture.start(fixture.spec("rust"));
    let worker = fixture.worker(Arc::new(Hang)).await;
    fixture.step(60).await;
    fixture.store.wait_recorded(&quick, RunOutcome::Ok).await;
    let handle = tokio::spawn(async move {
        worker
            .shutdown(tokio::time::Instant::now() + Duration::from_millis(300))
            .await
    });
    fixture
        .store
        .wait_recorded(&rust, RunOutcome::TimedOut)
        .await;
    assert!(
        fixture.row(&slow).last_outcome.is_none(),
        "monty is not timed out at the handle deadline: {:?}",
        fixture.row(&slow).last_outcome
    );
    assert_eq!(
        fixture.row(&rust).last_outcome,
        Some(RunOutcome::TimedOut),
        "the rust sibling is aborted"
    );
    let stopped = handle.await.expect("shutdown task");
    assert!(!stopped, "shutdown waits out the monty feed");
    assert_eq!(fixture.row(&slow).last_outcome, Some(RunOutcome::TimedOut));
    assert_eq!(runs_containing(&fixture, &slow, "repl kept"), 0);
}

#[tokio::test]
async fn two_monty_feeds_and_one_rust_run_at_handle_deadline() {
    let fixture = Fixture::new();
    let monty_short = fixture.start(spec("monty-short", TIME_LIMIT, Duration::from_millis(400)));
    let monty_long = fixture.start(spec("monty-long", TIME_LIMIT, Duration::from_secs(5)));
    let rust = fixture.start(fixture.spec("rust"));
    let worker = fixture.worker(Arc::new(Hang)).await;
    fixture.step(60).await;
    fixture
        .store
        .wait_recorded(&monty_short, RunOutcome::TimedOut)
        .await;
    assert!(
        fixture.row(&monty_long).last_outcome.is_none(),
        "second monty feed still running: {:?}",
        fixture.row(&monty_long).last_outcome
    );
    let handle = tokio::spawn(async move {
        worker
            .shutdown(tokio::time::Instant::now() + Duration::from_millis(300))
            .await
    });
    fixture
        .store
        .wait_recorded(&rust, RunOutcome::TimedOut)
        .await;
    assert!(
        fixture.row(&monty_long).last_outcome.is_none(),
        "handle deadline must not time out the second monty feed early: {:?}",
        fixture.row(&monty_long).last_outcome
    );
    assert_eq!(
        fixture.row(&rust).last_outcome,
        Some(RunOutcome::TimedOut),
        "rust sibling aborted at handle deadline"
    );
    let stopped = handle.await.expect("shutdown task");
    assert!(!stopped, "worker waits out remaining monty feeds");
    assert_eq!(
        fixture.row(&monty_long).last_outcome,
        Some(RunOutcome::TimedOut)
    );
}

#[tokio::test]
async fn a_monty_only_shutdown_does_not_time_out_until_the_feed_returns() {
    let fixture = Fixture::new();
    let schedule = fixture.start(spec("slow", TIME_LIMIT, Duration::from_secs(2)));
    let feeds = fixture.scheduler.subscribe_monty_feeds();
    let worker = fixture.worker(idle_job()).await;
    fixture.step(60).await;
    assert!(
        *feeds.borrow() >= 1,
        "monty feed did not start before shutdown"
    );
    let handle = tokio::spawn(async move {
        worker
            .shutdown(tokio::time::Instant::now() + Duration::from_millis(250))
            .await
    });
    let heard = tokio::time::timeout(
        common::WAIT,
        fixture.scheduler.wait_monty_shutdown_deadlines(0),
    )
    .await;
    assert!(
        matches!(heard, Ok(true)),
        "shutdown deadline was not observed"
    );
    assert!(fixture.row(&schedule).last_outcome.is_none());
    assert!(
        *feeds.borrow() >= 1,
        "monty feed ended at the shutdown deadline"
    );
    let stopped = handle.await.expect("shutdown task");
    assert!(!stopped);
    assert_eq!(
        fixture.row(&schedule).last_outcome,
        Some(RunOutcome::TimedOut)
    );
}

#[tokio::test]
async fn dropping_run_due_does_not_abort_the_feed_or_start_a_second_one() {
    let fixture = Fixture::new();
    let schedule = fixture.start(spec("sync", HOLD_STOP, Duration::from_secs(15)));
    fixture.clock.advance(chrono::Duration::seconds(60));
    let before = fixture.store.writes().len();
    let scheduler = fixture.scheduler.share();
    let running = tokio::spawn(async move { scheduler.run_due(idle_job()).await });
    wait_launch_stored(&fixture, &schedule, before).await;
    let due = fixture.clock.now();
    fixture
        .store
        .set_next(
            schedule.id,
            schedule.generation,
            sc_cron::Timestamp::from_datetime(due),
        )
        .expect("put the row back to due");
    let second = fixture.scheduler.run_due(idle_job()).await.expect("second");
    assert!(
        second.is_empty(),
        "the name is already in a feed: {second:?}"
    );
    // Do not step the manual clock: that moves the monotonic deadline the
    // in-flight feed already captured. The worker still sees the row as due.
    let worker = fixture.worker(idle_job()).await;
    running.abort();
    wait_failed(&fixture, &schedule, "old-tick").await;
    shutdown(worker).await;
    assert_eq!(runs_containing(&fixture, &schedule, "old-tick"), 1);
}

#[tokio::test]
async fn an_awaited_run_due_records_timed_out_only_after_the_feed_returns() {
    let fixture = Fixture::new();
    let schedule = fixture.start(spec("slow", TIME_LIMIT, Duration::from_secs(1)));
    fixture.clock.advance(chrono::Duration::seconds(60));
    let before = fixture.store.writes().len();
    let scheduler = fixture.scheduler.share();
    let id = schedule.id;
    let running = tokio::spawn(async move { scheduler.run_due(idle_job()).await });
    wait_launch_stored(&fixture, &schedule, before).await;
    assert!(fixture.row(&schedule).last_outcome.is_none());
    let done = running.await.expect("join").expect("run_due");
    assert_eq!(done.len(), 1);
    assert_eq!(done[0].outcome, RunOutcome::TimedOut);
    assert_eq!(
        fixture.row(&schedule).last_outcome,
        Some(RunOutcome::TimedOut)
    );
    let _ = id;
}

#[tokio::test]
async fn a_disable_answer_skips_the_tick_and_drops_the_repl() {
    let fixture = Fixture::new();
    let schedule = fixture.start(spec("sync", DISABLE, Duration::from_secs(5)));
    let worker = fixture.worker(idle_job()).await;
    fixture.step(60).await;
    fixture.store.wait_disabled(&schedule, "bye").await;
    fixture
        .scheduler
        .set_enabled_async(&schedule.spec.name, true)
        .await
        .expect("enable");
    fixture.step(60).await;
    let id = schedule.id;
    fixture
        .store
        .wait_writes("second disable", move |writes| {
            writes
                .iter()
                .filter(|write| {
                    matches!(write, Write::Disable { id: written, reason, changed: true }
                        if *written == id && reason == "bye")
                })
                .count()
                >= 2
        })
        .await;
    shutdown(worker).await;
    assert_eq!(runs_containing(&fixture, &schedule, "tick"), 0);
    assert_eq!(runs_containing(&fixture, &schedule, "repl survived"), 0);
}

#[tokio::test]
async fn a_stop_answer_other_than_continue_disable_or_delete_fails_without_a_tick() {
    let fixture = Fixture::new();
    let schedule = fixture.start(spec("sync", BAD_ACTION, Duration::from_secs(5)));
    let worker = fixture.worker(idle_job()).await;
    fixture.step(60).await;
    wait_failed(&fixture, &schedule, "nope").await;
    shutdown(worker).await;
    assert_eq!(runs_containing(&fixture, &schedule, "tick-ran"), 0);
}

#[tokio::test]
async fn a_feed_time_limit_drops_the_repl_so_the_next_tick_compiles_again() {
    let fixture = Fixture::new();
    let schedule = fixture.start(spec("sync", TIME_LIMIT, Duration::from_millis(400)));
    let worker = fixture.worker(idle_job()).await;
    fixture.step(60).await;
    fixture
        .store
        .wait_recorded(&schedule, RunOutcome::TimedOut)
        .await;
    fixture.step(60).await;
    wait_run_count(&fixture, &schedule, Some("timed_out"), 2).await;
    shutdown(worker).await;
    assert_eq!(runs_containing(&fixture, &schedule, "repl kept"), 0);
}

#[tokio::test]
async fn a_raising_hook_drops_the_repl_so_the_next_tick_compiles_again() {
    let fixture = Fixture::new();
    let schedule = fixture.start(spec("sync", RAISES, Duration::from_secs(5)));
    let worker = fixture.worker(idle_job()).await;
    fixture.step(60).await;
    wait_failed(&fixture, &schedule, "boom").await;
    fixture.step(60).await;
    wait_run_count(&fixture, &schedule, Some("boom"), 2).await;
    shutdown(worker).await;
    assert_eq!(runs_containing(&fixture, &schedule, "repl kept"), 0);
}

#[tokio::test]
async fn source_that_does_not_compile_is_hook_failed() {
    let fixture = Fixture::new();
    let schedule = fixture.start(spec("sync", SYNTAX, Duration::from_secs(5)));
    let worker = fixture.worker(idle_job()).await;
    fixture.step(60).await;
    wait_failed(&fixture, &schedule, "HOOK_FAILED").await;
    shutdown(worker).await;
}

#[tokio::test]
async fn source_missing_a_hook_function_is_hook_failed() {
    let fixture = Fixture::new();
    let schedule = fixture.start(spec("sync", MISSING_FN, Duration::from_secs(5)));
    let worker = fixture.worker(idle_job()).await;
    fixture.step(60).await;
    wait_failed(&fixture, &schedule, "HOOK_FAILED").await;
    shutdown(worker).await;
}

#[tokio::test]
async fn a_failing_feed_still_lets_shutdown_finish() {
    let fixture = Fixture::new();
    let schedule = fixture.start(spec("sync", RAISES, Duration::from_secs(5)));
    let worker = fixture.worker(idle_job()).await;
    fixture.step(60).await;
    wait_failed(&fixture, &schedule, "boom").await;
    let stopped = worker
        .shutdown(tokio::time::Instant::now() + Duration::from_secs(2))
        .await;
    assert!(stopped);
}
