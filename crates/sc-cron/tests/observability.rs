//! Canonical event contracts exercised through scheduler and worker behavior.
mod common;

use std::collections::{BTreeMap, VecDeque};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use async_trait::async_trait;
use sc_cron::{
    Clock, CronError, Generation, HookTarget, Job, ManualClock, MissedTickPolicy, NewSchedule,
    RunOutcome, Schedule, ScheduleId, ScheduleLimits, ScheduleName, ScheduleSpec, ScheduleStore,
    Scheduler, SeqJitter, StopDecision, StoreError, StoreErrorKind, Timestamp,
};
use sc_observability_types::{
    Level, LogEvent, RecordingCanonicalLog, RejectingCanonicalLog, ServiceName, SharedCanonicalLog,
};
use serde_json::{Value, json};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Operation {
    List,
    Record,
    Next,
    Stop,
    Disable,
}
#[derive(Clone, Copy)]
enum Answer {
    Pass,
    Fence,
    Fail,
}

struct FaultStore {
    inner: common::RecordingStore,
    // Test cases configure individual synchronous store calls. No guard crosses await.
    answers: Mutex<BTreeMap<Operation, VecDeque<Answer>>>,
}
impl FaultStore {
    fn new() -> Self {
        Self {
            inner: common::RecordingStore::new(),
            answers: Mutex::new(BTreeMap::new()),
        }
    }
    fn plant(&self, operation: Operation, answers: impl IntoIterator<Item = Answer>) {
        self.answers
            .lock()
            .unwrap()
            .insert(operation, answers.into_iter().collect());
    }
    fn answer(&self, operation: Operation) -> Result<Option<bool>, CronError> {
        let answer = self
            .answers
            .lock()
            .unwrap()
            .get_mut(&operation)
            .and_then(VecDeque::pop_front);
        match answer {
            Some(Answer::Fail) => Err(CronError::Store(StoreError::new(
                StoreErrorKind::Other,
                "planted store failure",
            ))),
            Some(Answer::Fence) => Ok(Some(false)),
            _ => Ok(None),
        }
    }
}
impl ScheduleStore for FaultStore {
    fn limits(&self) -> ScheduleLimits {
        self.inner.limits()
    }
    fn create(&self, row: &NewSchedule, at: Timestamp) -> Result<Schedule, CronError> {
        self.inner.create(row, at)
    }
    fn get(&self, id: ScheduleId) -> Result<Option<Schedule>, CronError> {
        self.inner.get(id)
    }
    fn list_active(&self) -> Result<Vec<Schedule>, CronError> {
        self.answer(Operation::List)?;
        self.inner.list_active()
    }
    fn record_run(
        &self,
        id: ScheduleId,
        generation: Generation,
        at: Timestamp,
        outcome: &RunOutcome,
    ) -> Result<bool, CronError> {
        match self.answer(Operation::Record)? {
            Some(answer) => Ok(answer),
            None => self.inner.record_run(id, generation, at, outcome),
        }
    }
    fn set_next(
        &self,
        id: ScheduleId,
        generation: Generation,
        at: Timestamp,
    ) -> Result<bool, CronError> {
        match self.answer(Operation::Next)? {
            Some(answer) => Ok(answer),
            None => self.inner.set_next(id, generation, at),
        }
    }
    fn stop(
        &self,
        id: ScheduleId,
        generation: Generation,
        at: Timestamp,
        reason: &str,
    ) -> Result<bool, CronError> {
        match self.answer(Operation::Stop)? {
            Some(answer) => Ok(answer),
            None => self.inner.stop(id, generation, at, reason),
        }
    }
    fn disable(
        &self,
        id: ScheduleId,
        generation: Generation,
        at: Timestamp,
        reason: &str,
    ) -> Result<bool, CronError> {
        match self.answer(Operation::Disable)? {
            Some(answer) => Ok(answer),
            None => self.inner.disable(id, generation, at, reason),
        }
    }
    fn set_enabled(&self, name: &ScheduleName, enabled: bool) -> Result<bool, CronError> {
        self.inner.set_enabled(name, enabled)
    }
    fn set_enabled_many(&self, flags: &[(ScheduleName, bool)]) -> Result<bool, CronError> {
        self.inner.set_enabled_many(flags)
    }
    fn replace_and_set_enabled(
        &self,
        row: &NewSchedule,
        enabled: bool,
        at: Timestamp,
        flags: &[(ScheduleName, bool)],
    ) -> Result<Option<Schedule>, CronError> {
        self.inner.replace_and_set_enabled(row, enabled, at, flags)
    }
}

type TestScheduler = Scheduler<FaultStore, ManualClock, SeqJitter>;
struct Fixture {
    store: Arc<FaultStore>,
    clock: Arc<ManualClock>,
    scheduler: TestScheduler,
}
impl Fixture {
    fn new(logger: SharedCanonicalLog) -> Self {
        let store = Arc::new(FaultStore::new());
        let clock = Arc::new(ManualClock::new(common::at("2026-10-19T00:00:00Z")));
        let scheduler = Scheduler::new_with_logger(
            store.clone(),
            clock.clone(),
            Arc::new(SeqJitter::default()),
            logger,
        );
        Self {
            store,
            clock,
            scheduler,
        }
    }
    fn start(&self, name: &str) -> Schedule {
        self.scheduler.start(spec(name)).unwrap()
    }
    async fn worker(&self, job: Arc<dyn Job>) -> sc_cron::WorkerHandle {
        let parks = self.clock.parks();
        let worker = self.scheduler.spawn_worker(job);
        self.clock.wait_parks_above(parks).await;
        worker
    }
    async fn advance(&self, seconds: i64) {
        let parks = self.clock.parks();
        self.clock.advance(chrono::Duration::seconds(seconds));
        self.clock.wait_parks_above(parks).await;
    }
    fn row(&self, row: &Schedule) -> Schedule {
        self.store.get(row.id).unwrap().unwrap()
    }
}
fn spec(name: &str) -> ScheduleSpec {
    ScheduleSpec::new(
        name,
        "0 * * * * *",
        Duration::from_secs(20),
        json!({"body":"must-not-be-logged"}),
    )
    .unwrap()
}
fn recorder() -> Arc<RecordingCanonicalLog> {
    Arc::new(RecordingCanonicalLog::new(
        ServiceName::new("cron-tests").unwrap(),
    ))
}

struct TestJob {
    outcome: RunOutcome,
    stop: StopDecision,
    panic: bool,
}
impl TestJob {
    fn returning(outcome: RunOutcome) -> Arc<dyn Job> {
        Arc::new(Self {
            outcome,
            stop: StopDecision::Continue,
            panic: false,
        })
    }
    fn stopping(stop: StopDecision) -> Arc<dyn Job> {
        Arc::new(Self {
            outcome: RunOutcome::Ok,
            stop,
            panic: false,
        })
    }
}
#[async_trait]
impl Job for TestJob {
    async fn run(&self, _: &Value, _: Instant) -> RunOutcome {
        assert!(!self.panic, "job panic");
        self.outcome.clone()
    }
    async fn should_stop(&self, _: &Value) -> StopDecision {
        self.stop.clone()
    }
}

async fn wait_event(log: &RecordingCanonicalLog, action: &str, count: usize) {
    tokio::time::timeout(common::WAIT, async {
        loop {
            if log
                .events()
                .iter()
                .filter(|e| e.action.as_str() == action)
                .count()
                >= count
            {
                return;
            }
            tokio::task::yield_now().await;
        }
    })
    .await
    .unwrap_or_else(|_| panic!("missing {action}: {:?}", log.events()));
}
fn events(log: &RecordingCanonicalLog, action: &str) -> Vec<LogEvent> {
    log.events()
        .into_iter()
        .filter(|e| e.action.as_str() == action)
        .collect()
}
fn only(log: &RecordingCanonicalLog, action: &str) -> LogEvent {
    let events = events(log, action);
    assert_eq!(events.len(), 1, "{action}: {events:?}");
    assert_fixed_row(&events[0]);
    events[0].clone()
}

fn assert_terminal(event: &LogEvent, outcome: &RunOutcome) {
    let (level, label, code) = match outcome {
        RunOutcome::Ok => (Level::Info, "success", None),
        RunOutcome::Refused(_) => (Level::Info, "refused", None),
        RunOutcome::Failed(_) => (Level::Error, "failed", Some("JOB_FAILED")),
        RunOutcome::TimedOut => (Level::Warn, "timeout", Some("RUN_TIMEOUT")),
    };
    assert_eq!(event.level, level);
    assert_eq!(event.outcome.as_ref().unwrap().as_str(), label);
    assert_eq!(event.diagnostic.as_ref().map(|d| d.code.as_str()), code);
    match outcome {
        RunOutcome::Ok => {
            assert!(!event.fields.contains_key("error") && !event.fields.contains_key("detail"))
        }
        RunOutcome::Refused(reason) => assert_eq!(event.fields["reason"], *reason),
        RunOutcome::Failed(detail) => assert_eq!(event.fields["error"], *detail),
        RunOutcome::TimedOut => assert_eq!(event.fields["deadline_remaining_ms"].as_u64(), Some(0)),
    }
}
fn assert_envelope(log: &RecordingCanonicalLog) {
    for event in log.events() {
        assert_eq!(event.service.as_str(), "cron-tests");
        assert!(!event.fields.contains_key("code"));
        assert!(event.trace.is_none() && event.state_transition.is_none());
        assert!(
            !event.fields.contains_key("correlation_id") && !event.fields.contains_key("run_id")
        );
        match event.action.as_str() {
            "run.launched"
            | "due.launched"
            | "run.finished"
            | "due.finished"
            | "target.resolve_failed"
            | "job.join_failed"
            | "run.timed_out"
            | "run.record_failed"
            | "due.record_failed"
            | "due.partial_write"
            | "due.follow_up_failed" => assert!(
                event.correlation_id.is_some(),
                "{} has run context",
                event.action.as_str()
            ),
            "logger.attached"
            | "schedule.reload_failed"
            | "run.skipped_in_flight"
            | "run.missed_skipped"
            | "run.missed_replayed" => assert!(event.correlation_id.is_none()),
            _ => {}
        }
        if let Some(diagnostic) = &event.diagnostic {
            assert_eq!(diagnostic.timestamp, event.timestamp);
        }
        assert!(
            !serde_json::to_string(&event.fields)
                .unwrap()
                .contains("must-not-be-logged")
        );
    }
}

#[tokio::test]
async fn native_worker_and_due_terminal_table_copies_consumer_prose_without_parsing() {
    let outcomes = [
        RunOutcome::Ok,
        RunOutcome::Refused("DUE_JOIN_FAILED: arbitrary; trusted".into()),
        RunOutcome::Failed("TARGET_RESOLVE_FAILED: arbitrary; trusted".into()),
        RunOutcome::TimedOut,
    ];
    for outcome in outcomes {
        for worker_path in [false, true] {
            let log = recorder();
            let fixture = Fixture::new(log.clone());
            let row = fixture.start("terminal");
            if worker_path {
                let worker = fixture.worker(TestJob::returning(outcome.clone())).await;
                fixture.advance(60).await;
                wait_event(&log, "run.finished", 1).await;
                common::shutdown(worker).await;
                assert_terminal(&only(&log, "run.finished"), &outcome);
                assert_eq!(
                    only(&log, "run.launched").fields["schedule"],
                    row.id.to_string()
                );
            } else {
                fixture.clock.advance(chrono::Duration::seconds(60));
                let runs = fixture
                    .scheduler
                    .run_due(TestJob::returning(outcome.clone()))
                    .await
                    .unwrap();
                assert_eq!(runs[0].outcome, outcome);
                assert_terminal(&only(&log, "due.finished"), &outcome);
                assert_eq!(
                    only(&log, "due.launched").fields["schedule"],
                    row.id.to_string()
                );
            }
            assert_eq!(fixture.row(&row).last_outcome, Some(outcome.clone()));
            assert_envelope(&log);
            assert!(log.events().iter().all(|event| {
                event
                    .diagnostic
                    .as_ref()
                    .is_none_or(|d| matches!(d.code.as_str(), "JOB_FAILED" | "RUN_TIMEOUT"))
            }));
        }
    }
}

#[tokio::test]
async fn resolution_and_worker_panic_emit_fixed_origins_before_normalized_terminal() {
    for panic in [false, true] {
        let log = recorder();
        let fixture = Fixture::new(log.clone());
        let mut target = spec("origin");
        if !panic {
            target.hook = HookTarget::Python {
                module: "-invalid-path".into(),
            };
        }
        let row = fixture.scheduler.start(target).unwrap();
        let job: Arc<dyn Job> = Arc::new(TestJob {
            outcome: RunOutcome::Ok,
            stop: StopDecision::Continue,
            panic,
        });
        let worker = fixture.worker(job).await;
        fixture.advance(60).await;
        wait_event(&log, "run.finished", 1).await;
        common::shutdown(worker).await;
        let outcome = fixture.row(&row).last_outcome.unwrap();
        let parts = outcome.failure_parts().unwrap();
        let (action, code, recovery) = if panic {
            (
                "job.join_failed",
                "JOB_JOIN_FAILED",
                "fix the job panic and retry",
            )
        } else {
            (
                "target.resolve_failed",
                "TARGET_RESOLVE_FAILED",
                "fix hook target configuration and retry",
            )
        };
        assert_eq!(parts.code, Some(code));
        assert_eq!(parts.recovery, Some(recovery));
        let origin = only(&log, action);
        assert_eq!(origin.diagnostic.unwrap().code.as_str(), code);
        assert_terminal(&only(&log, "run.finished"), &outcome);
        let all = log.events();
        assert!(
            all.iter()
                .position(|e| e.action.as_str() == action)
                .unwrap()
                < all
                    .iter()
                    .position(|e| e.action.as_str() == "run.finished")
                    .unwrap()
        );
        assert_envelope(&log);
    }
}

#[tokio::test]
async fn due_store_failure_and_fencing_table_preserves_committed_outcome() {
    let cases = [
        (
            Operation::Record,
            vec![Answer::Fail],
            "due.record_failed",
            false,
        ),
        (Operation::Record, vec![Answer::Fence], "due.fenced", false),
        (Operation::Next, vec![Answer::Fail], "due.next_failed", true),
        (Operation::Next, vec![Answer::Fence], "due.fenced", false),
        (
            Operation::Next,
            vec![Answer::Pass, Answer::Fence],
            "due.partial_write",
            true,
        ),
        (
            Operation::Next,
            vec![Answer::Pass, Answer::Fail],
            "due.follow_up_failed",
            true,
        ),
    ];
    for (operation, answers, action, recorded) in cases {
        let log = recorder();
        let fixture = Fixture::new(log.clone());
        let row = fixture.start("due-fault");
        fixture.store.plant(operation, answers);
        fixture.clock.advance(chrono::Duration::seconds(60));
        let runs = fixture
            .scheduler
            .run_due(TestJob::returning(RunOutcome::Ok))
            .await
            .unwrap();
        only(&log, action);
        assert_eq!(fixture.row(&row).last_outcome.is_some(), recorded);
        assert_eq!(events(&log, "due.finished").len(), usize::from(recorded));
        if !runs.is_empty() {
            assert_eq!(runs[0].outcome, RunOutcome::Ok);
        }
        assert_envelope(&log);
    }
}

#[tokio::test]
async fn worker_write_failure_rows_keep_stop_disable_and_retry_behavior() {
    for (operation, action) in [
        (Operation::Record, "run.record_failed"),
        (Operation::Next, "schedule.next_failed"),
        (Operation::Stop, "schedule.stop_failed"),
        (Operation::Disable, "schedule.disable_failed"),
    ] {
        let log = recorder();
        let fixture = Fixture::new(log.clone());
        let row = fixture.start("worker-fault");
        fixture.store.plant(operation, [Answer::Fail]);
        let job = match operation {
            Operation::Stop => TestJob::stopping(StopDecision::Delete("done".into())),
            Operation::Disable => TestJob::stopping(StopDecision::Disable("quiet".into())),
            _ => TestJob::returning(RunOutcome::Ok),
        };
        let worker = fixture.worker(job).await;
        fixture.advance(60).await;
        wait_event(&log, action, 1).await;
        assert!(fixture.row(&row).is_active());
        common::shutdown(worker).await;
        only(&log, action);
        assert_envelope(&log);
    }
    let log = recorder();
    let fixture = Fixture::new(log.clone());
    fixture.start("reload");
    fixture.store.plant(Operation::List, [Answer::Fail]);
    let worker = fixture.worker(TestJob::returning(RunOutcome::Ok)).await;
    wait_event(&log, "schedule.reload_failed", 1).await;
    fixture.advance(1).await;
    common::shutdown(worker).await;
    only(&log, "schedule.reload_failed");
}

#[tokio::test]
async fn successful_worker_stop_disable_and_generation_fence_rows_are_exact() {
    for (decision, action) in [
        (StopDecision::Delete("done".into()), "schedule.stopped"),
        (StopDecision::Disable("quiet".into()), "schedule.disabled"),
    ] {
        let log = recorder();
        let fixture = Fixture::new(log.clone());
        let row = fixture.start("stop");
        let worker = fixture.worker(TestJob::stopping(decision)).await;
        fixture.advance(60).await;
        wait_event(&log, action, 1).await;
        common::shutdown(worker).await;
        assert_eq!(only(&log, action).outcome.unwrap().as_str(), "stopped");
        assert!(fixture.row(&row).last_outcome.is_some() || !fixture.row(&row).is_active());
        if action == "schedule.disabled" {
            assert_terminal(
                &only(&log, "run.finished"),
                &RunOutcome::Refused("quiet".into()),
            );
        }
    }
    for operation in [
        Operation::Record,
        Operation::Next,
        Operation::Stop,
        Operation::Disable,
    ] {
        let log = recorder();
        let fixture = Fixture::new(log.clone());
        fixture.start("fenced");
        fixture.store.plant(operation, [Answer::Fence]);
        let job = match operation {
            Operation::Stop => TestJob::stopping(StopDecision::Delete("done".into())),
            Operation::Disable => TestJob::stopping(StopDecision::Disable("quiet".into())),
            _ => TestJob::returning(RunOutcome::Ok),
        };
        let worker = fixture.worker(job).await;
        fixture.advance(60).await;
        wait_event(&log, "run.fenced", 1).await;
        common::shutdown(worker).await;
        assert_eq!(
            only(&log, "run.fenced").diagnostic.unwrap().code.as_str(),
            "SCHEDULE_FENCED"
        );
    }
}

struct HangingJob {
    entered: tokio::sync::watch::Sender<Option<Instant>>,
}
#[async_trait]
impl Job for HangingJob {
    async fn should_stop(&self, _: &Value) -> StopDecision {
        StopDecision::Continue
    }
    async fn run(&self, _: &Value, deadline: Instant) -> RunOutcome {
        self.entered.send_replace(Some(deadline));
        std::future::pending().await
    }
}
async fn entered(job: &HangingJob) -> Instant {
    let mut entered = job.entered.subscribe();
    tokio::time::timeout(common::WAIT, entered.wait_for(Option::is_some))
        .await
        .unwrap()
        .unwrap()
        .unwrap()
}

#[tokio::test]
async fn skipped_missed_cancelled_and_timeout_rows_follow_manual_clock_decisions() {
    for (policy, action) in [
        (MissedTickPolicy::Coalesce, "run.missed_replayed"),
        (MissedTickPolicy::Skip, "run.missed_skipped"),
    ] {
        let log = recorder();
        let mut fixture = Fixture::new(log.clone());
        fixture.scheduler = fixture.scheduler.with_missed_tick_policy(policy);
        fixture.start("missed");
        let worker = fixture.worker(TestJob::returning(RunOutcome::Ok)).await;
        fixture.advance(180).await;
        wait_event(&log, action, 1).await;
        common::shutdown(worker).await;
        only(&log, action);
    }
    let log = recorder();
    let fixture = Fixture::new(log.clone());
    let mut target = spec("hanging");
    target.timeout = sc_cron::RunTimeout::new(Duration::from_secs(180)).unwrap();
    let row = fixture.scheduler.start(target).unwrap();
    let job = Arc::new(HangingJob {
        entered: tokio::sync::watch::channel(None).0,
    });
    let worker = fixture.worker(job.clone()).await;
    fixture.advance(60).await;
    entered(&job).await;
    fixture.advance(60).await;
    wait_event(&log, "run.skipped_in_flight", 1).await;
    only(&log, "run.skipped_in_flight");
    fixture
        .scheduler
        .set_enabled(&row.spec.name, false)
        .unwrap();
    wait_event(&log, "schedule.cancelled", 1).await;
    common::shutdown(worker).await;
    only(&log, "schedule.cancelled");
    let log = recorder();
    let fixture = Fixture::new(log.clone());
    let row = fixture.start("timeout");
    let job = Arc::new(HangingJob {
        entered: tokio::sync::watch::channel(None).0,
    });
    let worker = fixture.worker(job.clone()).await;
    fixture.advance(60).await;
    let deadline = entered(&job).await;
    fixture.advance(20).await;
    wait_event(&log, "run.finished", 1).await;
    common::shutdown(worker).await;
    assert_eq!(deadline, fixture.clock.monotonic_now());
    assert_eq!(fixture.row(&row).last_outcome, Some(RunOutcome::TimedOut));
    assert_eq!(
        only(&log, "run.timed_out").fields["deadline_remaining_ms"].as_u64(),
        Some(0)
    );
    assert_terminal(&only(&log, "run.finished"), &RunOutcome::TimedOut);
}

#[tokio::test]
async fn due_final_stop_recovery_distinguishes_failure_fence_and_success() {
    for answer in [Answer::Fail, Answer::Fence, Answer::Pass] {
        let log = recorder();
        let fixture = Fixture::new(log.clone());
        let row = fixture
            .scheduler
            .start(
                ScheduleSpec::new(
                    "final",
                    "0 1 0 19 10 * 2026",
                    Duration::from_secs(20),
                    json!({}),
                )
                .unwrap(),
            )
            .unwrap();
        fixture.store.plant(Operation::Stop, [Answer::Fail, answer]);
        fixture.clock.advance(chrono::Duration::seconds(60));
        let runs = fixture
            .scheduler
            .run_due(TestJob::returning(RunOutcome::Ok))
            .await
            .unwrap();
        assert_eq!(runs.len(), 1);
        only(&log, "due.finished");
        only(&log, "due.follow_up_failed");
        assert!(fixture.row(&row).is_active());
        let runs = fixture
            .scheduler
            .run_due(TestJob::returning(RunOutcome::Failed(
                "must not rerun".into(),
            )))
            .await
            .unwrap();
        assert!(runs.is_empty());
        let action = match answer {
            Answer::Fail => "due.stop_failed",
            Answer::Fence => "due.fenced",
            Answer::Pass => "due.stopped",
        };
        only(&log, action);
        assert_eq!(fixture.row(&row).last_outcome, Some(RunOutcome::Ok));
    }
}

#[tokio::test]
async fn rejecting_every_event_preserves_due_results_and_worker_stop_retry_and_deadline() {
    let accepted = recorder();
    let rejected = Arc::new(RejectingCanonicalLog::new(
        ServiceName::new("reject").unwrap(),
    ));
    let mut snapshots = Vec::new();
    for logger in [
        accepted.clone() as SharedCanonicalLog,
        rejected.clone() as SharedCanonicalLog,
    ] {
        for decision in [
            StopDecision::Continue,
            StopDecision::Disable("quiet".into()),
            StopDecision::Delete("done".into()),
        ] {
            let fixture = Fixture::new(logger.clone());
            let row = fixture.start("decision");
            fixture.clock.advance(chrono::Duration::seconds(60));
            let runs = fixture
                .scheduler
                .run_due(TestJob::stopping(decision))
                .await
                .unwrap();
            let state = fixture.row(&row);
            snapshots.push((
                runs[0].outcome.clone(),
                state.is_active(),
                state.enabled,
                state.last_outcome.clone(),
            ));
        }
        let fixture = Fixture::new(logger.clone());
        let row = fixture.start("retry");
        fixture.store.inner.fail_next_stop();
        let worker = fixture
            .worker(TestJob::stopping(StopDecision::Delete("done".into())))
            .await;
        fixture.advance(60).await;
        fixture.store.inner.wait_failures(1).await;
        fixture.advance(1).await;
        fixture.advance(59).await;
        fixture.store.inner.wait_stopped(&row, "done").await;
        common::shutdown(worker).await;
        let state = fixture.row(&row);
        snapshots.push((
            RunOutcome::Ok,
            state.is_active(),
            state.enabled,
            state.last_outcome.clone(),
        ));
        let fixture = Fixture::new(logger);
        let row = fixture.start("deadline");
        let job = Arc::new(HangingJob {
            entered: tokio::sync::watch::channel(None).0,
        });
        let worker = fixture.worker(job.clone()).await;
        fixture.advance(60).await;
        let deadline = entered(&job).await;
        fixture.advance(20).await;
        fixture
            .store
            .inner
            .wait_recorded(&row, RunOutcome::TimedOut)
            .await;
        common::shutdown(worker).await;
        assert_eq!(deadline, fixture.clock.monotonic_now());
        let state = fixture.row(&row);
        snapshots.push((
            RunOutcome::TimedOut,
            state.is_active(),
            state.enabled,
            state.last_outcome,
        ));
    }
    assert_eq!(&snapshots[..5], &snapshots[5..]);
    assert!(rejected.attempts() > 10);
}

#[cfg(feature = "monty")]
#[tokio::test]
async fn monty_recording_and_rejection_preserve_success_failure_refusal_and_timeout() {
    // The infinite witness runs only in the owned subprocess below. Its parent
    // deadline also covers destruction of Tokio's non-abortable blocking pool.
    let child_root = std::env::var_os("SC_CRON_TIMEOUT_WITNESS_ROOT").map(std::path::PathBuf::from);
    if let Some(root) = &child_root {
        std::fs::write(root.join("ready"), "ready").unwrap();
    }
    let sources = [
        "def cron_stop(hook):\n return {'action': 'continue', 'reason': 'ok'}\ndef cron_tick(hook):\n return None\n",
        "def cron_stop(hook):\n return {'action': 'continue', 'reason': 'ok'}\ndef cron_tick(hook):\n raise ValueError('consumer failure')\n",
        "def cron_stop(hook):\n return {'action': 'disable', 'reason': 'quiet'}\ndef cron_tick(hook):\n return None\n",
        "def cron_stop(hook):\n return {'action': 'continue', 'reason': 'ok'}\ndef cron_tick(hook):\n while True:\n  pass\n",
    ];
    for (index, source) in sources.into_iter().enumerate() {
        if (index == 3) != child_root.is_some() {
            continue;
        }
        for worker_path in [false, true] {
            let log = recorder();
            let reject = Arc::new(RejectingCanonicalLog::new(
                ServiceName::new("reject-monty").unwrap(),
            ));
            let mut states = Vec::new();
            for logger in [
                log.clone() as SharedCanonicalLog,
                reject.clone() as SharedCanonicalLog,
            ] {
                let fixture = Fixture::new(logger);
                let mut target = spec("monty");
                target.hook = HookTarget::monty(source).unwrap();
                if index == 3 {
                    target.timeout = sc_cron::RunTimeout::new(Duration::from_millis(50)).unwrap();
                }
                let row = fixture.scheduler.start(target).unwrap();
                if worker_path {
                    let worker = fixture
                        .worker(TestJob::returning(RunOutcome::Failed(
                            "wrong target".into(),
                        )))
                        .await;
                    fixture.advance(60).await;
                    if index == 2 {
                        fixture.store.inner.wait_disabled(&row, "quiet").await;
                    } else {
                        tokio::time::timeout(common::WAIT, async {
                            loop {
                                if fixture.row(&row).last_outcome.is_some() {
                                    break;
                                }
                                tokio::task::yield_now().await;
                            }
                        })
                        .await
                        .unwrap();
                    }
                    common::shutdown(worker).await;
                } else {
                    fixture.clock.advance(chrono::Duration::seconds(60));
                    let runs = fixture
                        .scheduler
                        .run_due(TestJob::returning(RunOutcome::Failed(
                            "wrong target".into(),
                        )))
                        .await
                        .unwrap();
                    assert_eq!(runs.len(), 1);
                }
                let state = fixture.row(&row);
                states.push((
                    state.last_outcome.clone().unwrap(),
                    state.is_active(),
                    state.enabled,
                ));
            }
            assert_eq!(states[0], states[1]);
            assert!(reject.attempts() > 1);
            let action = if worker_path {
                "run.finished"
            } else {
                "due.finished"
            };
            if worker_path && index == 2 {
                only(&log, "schedule.disabled");
            }
            assert_terminal(&only(&log, action), &states[0].0);
            let launched = only(
                &log,
                if worker_path {
                    "run.launched"
                } else {
                    "due.launched"
                },
            );
            assert_eq!(launched.correlation_id, only(&log, action).correlation_id);
            assert_envelope(&log);
        }
    }
    if let Some(root) = child_root {
        std::fs::write(root.join("result"), "passed").unwrap();
    }
}

#[cfg(feature = "monty")]
#[test]
fn monty_nonterminating_feeds_have_external_deadline() {
    struct OwnedChild(std::process::Child);
    impl Drop for OwnedChild {
        fn drop(&mut self) {
            let _ = self.0.kill();
            let _ = self.0.wait();
        }
    }
    let root = tempfile::tempdir().unwrap();
    let until = Instant::now() + common::WAIT;
    let mut child = OwnedChild(
        std::process::Command::new(std::env::current_exe().unwrap())
            .args([
                "--exact",
                "monty_recording_and_rejection_preserve_success_failure_refusal_and_timeout",
                "--nocapture",
            ])
            .env("SC_CRON_TIMEOUT_WITNESS_ROOT", root.path())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::inherit())
            .spawn()
            .unwrap(),
    );
    // Readiness, completed assertions, and process exit must all happen before
    // this single absolute bound. Drop always kills and reaps on any failure.
    let status = loop {
        if let Some(status) = child.0.try_wait().unwrap() {
            break status;
        }
        assert!(
            Instant::now() < until,
            "Monty deadline witness exceeded independent process bound"
        );
        std::thread::sleep(Duration::from_millis(5)); // allow: test polls its owned child
    };
    assert!(
        root.path().join("ready").is_file(),
        "child never became ready"
    );
    assert_eq!(
        std::fs::read_to_string(root.path().join("result")).unwrap(),
        "passed"
    );
    assert!(status.success(), "deadline subprocess failed: {status}");
}

fn assert_fixed_row(event: &LogEvent) {
    let (target, level, outcome, code, fields): (&str, Level, Option<&str>, Option<&str>, &[&str]) =
        match event.action.as_str() {
            "target.resolve_failed" => (
                "cron.worker",
                Level::Warn,
                Some("failed"),
                Some("TARGET_RESOLVE_FAILED"),
                &["schedule", "target", "error"],
            ),
            "schedule.reload_failed" => (
                "cron.worker",
                Level::Error,
                Some("failed"),
                Some("SCHEDULE_RELOAD_FAILED"),
                &["error"],
            ),
            "schedule.cancelled" => (
                "cron.worker",
                Level::Info,
                Some("stopped"),
                None,
                &["schedule", "reason"],
            ),
            "run.timed_out" => (
                "cron.worker",
                Level::Warn,
                Some("timeout"),
                Some("RUN_TIMEOUT"),
                &["schedule", "deadline_remaining_ms"],
            ),
            "run.skipped_in_flight" => (
                "cron.worker",
                Level::Info,
                None,
                None,
                &["schedule", "reason"],
            ),
            "run.missed_skipped" => ("cron.worker", Level::Info, None, None, &["schedule"]),
            "run.missed_replayed" => ("cron.worker", Level::Info, None, None, &["schedule"]),
            "run.launched" => (
                "cron.worker",
                Level::Info,
                None,
                None,
                &["schedule", "deadline_remaining_ms"],
            ),
            "run.record_failed" => (
                "cron.worker",
                Level::Error,
                Some("failed"),
                Some("RUN_RECORD_FAILED"),
                &["schedule", "error"],
            ),
            "schedule.next_failed" => (
                "cron.worker",
                Level::Error,
                Some("failed"),
                Some("SCHEDULE_NEXT_FAILED"),
                &["schedule", "error"],
            ),
            "schedule.stopped" => (
                "cron.worker",
                Level::Info,
                Some("stopped"),
                None,
                &["schedule", "reason"],
            ),
            "schedule.stop_failed" => (
                "cron.worker",
                Level::Error,
                Some("failed"),
                Some("SCHEDULE_STOP_FAILED"),
                &["schedule", "error"],
            ),
            "schedule.disabled" => (
                "cron.worker",
                Level::Info,
                Some("stopped"),
                None,
                &["schedule", "reason"],
            ),
            "schedule.disable_failed" => (
                "cron.worker",
                Level::Error,
                Some("failed"),
                Some("SCHEDULE_DISABLE_FAILED"),
                &["schedule", "error"],
            ),
            "run.fenced" => (
                "cron.worker",
                Level::Warn,
                Some("fenced"),
                Some("SCHEDULE_FENCED"),
                &["schedule", "operation"],
            ),
            "due.launched" => (
                "cron.scheduler",
                Level::Info,
                None,
                None,
                &["schedule", "deadline_remaining_ms"],
            ),
            "due.fenced" => (
                "cron.scheduler",
                Level::Warn,
                Some("fenced"),
                Some("SCHEDULE_FENCED"),
                &["schedule", "operation"],
            ),
            "due.partial_write" => (
                "cron.scheduler",
                Level::Warn,
                Some("partial"),
                Some("FOLLOW_UP_FENCED"),
                &["schedule", "outcome"],
            ),
            "due.follow_up_failed" => (
                "cron.scheduler",
                Level::Error,
                Some("partial"),
                Some("FOLLOW_UP_WRITE_FAILED"),
                &["schedule", "error", "outcome"],
            ),
            "due.record_failed" => (
                "cron.scheduler",
                Level::Error,
                Some("failed"),
                Some("RUN_RECORD_FAILED"),
                &["schedule", "error"],
            ),
            "due.stopped" => (
                "cron.scheduler",
                Level::Info,
                Some("stopped"),
                None,
                &["schedule"],
            ),
            "due.stop_failed" => (
                "cron.scheduler",
                Level::Error,
                Some("failed"),
                Some("SCHEDULE_STOP_FAILED"),
                &["schedule", "error"],
            ),
            "due.next_failed" => (
                "cron.scheduler",
                Level::Error,
                Some("failed"),
                Some("SCHEDULE_NEXT_FAILED"),
                &["schedule", "error"],
            ),
            "job.join_failed" => (
                "cron.worker",
                Level::Error,
                Some("failed"),
                Some("JOB_JOIN_FAILED"),
                &["schedule", "error"],
            ),
            "due.join_failed" => (
                "cron.scheduler",
                Level::Error,
                Some("failed"),
                Some("DUE_JOIN_FAILED"),
                &["schedule", "error"],
            ),
            "due.task_failed" => (
                "cron.scheduler",
                Level::Error,
                Some("failed"),
                Some("RUN_TASK_FAILED"),
                &["schedule", "error"],
            ),
            "monty.resolve_failed" => (
                "cron.scheduler",
                Level::Error,
                Some("failed"),
                Some("MONTY_RESOLVE"),
                &["schedule", "error"],
            ),
            "monty.tick_failed" => (
                "cron.scheduler",
                Level::Error,
                Some("failed"),
                Some("MONTY_TICK"),
                &["schedule", "error"],
            ),
            "feed.join_failed" => (
                "cron.monty",
                Level::Error,
                Some("failed"),
                Some("FEED_JOIN_FAILED"),
                &["schedule", "error"],
            ),
            "run.finished" | "due.finished" => {
                assert!(event.fields["schedule"].is_string());
                return;
            }
            other => panic!("unexpected event action: {other}"),
        };
    assert_eq!(event.target.as_str(), target);
    assert_eq!(event.level, level);
    assert_eq!(event.outcome.as_ref().map(|v| v.as_str()), outcome);
    assert_eq!(event.diagnostic.as_ref().map(|v| v.code.as_str()), code);
    for field in fields {
        if *field == "deadline_remaining_ms" {
            assert!(event.fields[*field].as_u64().is_some());
        } else {
            assert!(
                event.fields[*field].is_string(),
                "missing {field}: {event:?}"
            );
        }
    }
}

#[tokio::test]
async fn logical_occurrences_and_restarted_generations_have_distinct_correlations() {
    let sources: &[Option<&str>] = &[
        None,
        #[cfg(feature = "monty")]
        Some(
            "def cron_stop(hook):\n return {'action': 'continue', 'reason': 'ok'}\ndef cron_tick(hook):\n return None\n",
        ),
    ];
    for &source in sources {
        for worker_path in [false, true] {
            let log = recorder();
            let fixture = Fixture::new(log.clone());
            let mut target = spec("occurrences");
            if let Some(source) = source {
                target.hook = HookTarget::monty(source).unwrap();
            }
            let mut row = fixture.scheduler.start(target.clone()).unwrap();
            let first_generation = row.generation;
            let mut correlations = Vec::new();
            for occurrence in 0..3 {
                if occurrence == 2 {
                    fixture
                        .scheduler
                        .stop(row.id, row.generation, "restart")
                        .unwrap();
                    let restarted = fixture.scheduler.start(target.clone()).unwrap();
                    assert_eq!(restarted.id, row.id);
                    assert_ne!(restarted.generation, first_generation);
                    row = restarted;
                }
                let selected = fixture.row(&row).next_run_at.unwrap().as_datetime();
                let expected = format!(
                    "cron-s{}-g{}-t{}-n{}",
                    row.id,
                    row.generation,
                    selected.timestamp(),
                    selected.timestamp_subsec_nanos()
                );
                if worker_path {
                    let worker = fixture.worker(TestJob::returning(RunOutcome::Ok)).await;
                    fixture.advance(60).await;
                    tokio::time::timeout(common::WAIT, async {
                        while log
                            .events()
                            .iter()
                            .filter(|event| event.action.as_str() == "run.finished")
                            .count()
                            != occurrence + 1
                        {
                            tokio::task::yield_now().await;
                        }
                    })
                    .await
                    .unwrap();
                    common::shutdown(worker).await;
                } else {
                    fixture.clock.advance(chrono::Duration::seconds(60));
                    assert_eq!(
                        fixture
                            .scheduler
                            .run_due(TestJob::returning(RunOutcome::Ok))
                            .await
                            .unwrap()[0]
                            .outcome,
                        RunOutcome::Ok
                    );
                }
                let launch_action = if worker_path {
                    "run.launched"
                } else {
                    "due.launched"
                };
                let finish_action = if worker_path {
                    "run.finished"
                } else {
                    "due.finished"
                };
                let events = log.events();
                let launch = events
                    .iter()
                    .filter(|event| event.action.as_str() == launch_action)
                    .nth(occurrence)
                    .unwrap();
                let finish = events
                    .iter()
                    .filter(|event| event.action.as_str() == finish_action)
                    .nth(occurrence)
                    .unwrap();
                assert_eq!(launch.correlation_id.as_ref().unwrap().as_str(), expected);
                assert_eq!(launch.correlation_id, finish.correlation_id);
                correlations.push(launch.correlation_id.clone().unwrap());
            }
            assert_ne!(correlations[0], correlations[1]);
            assert_ne!(correlations[1], correlations[2]);
            assert_envelope(&log);
        }
    }
}
