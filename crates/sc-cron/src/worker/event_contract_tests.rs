use super::*;
use sc_command::observability::{RecordingCanonicalLog, SharedCanonicalLog};
use sc_observability_types::{Level, ServiceName};

#[test]
fn fixed_origin_rows_use_exact_taxonomy_and_best_effort_envelope() {
    let recorder = Arc::new(RecordingCanonicalLog::new(
        ServiceName::new("worker-test").unwrap(),
    ));
    let logger: SharedCanonicalLog = recorder.clone();
    let rows = [
        (
            Event::TargetResolveFailed,
            "cron.worker",
            "target.resolve_failed",
            Level::Warn,
            Some("failed"),
            Some("TARGET_RESOLVE_FAILED"),
        ),
        (
            Event::ScheduleReloadFailed,
            "cron.worker",
            "schedule.reload_failed",
            Level::Error,
            Some("failed"),
            Some("SCHEDULE_RELOAD_FAILED"),
        ),
        (
            Event::ScheduleCancelled,
            "cron.worker",
            "schedule.cancelled",
            Level::Info,
            Some("stopped"),
            None,
        ),
        (
            Event::RunTimedOut,
            "cron.worker",
            "run.timed_out",
            Level::Warn,
            Some("timeout"),
            Some("RUN_TIMEOUT"),
        ),
        (
            Event::RunSkippedInFlight,
            "cron.worker",
            "run.skipped_in_flight",
            Level::Info,
            None,
            None,
        ),
        (
            Event::RunMissedSkipped,
            "cron.worker",
            "run.missed_skipped",
            Level::Info,
            None,
            None,
        ),
        (
            Event::RunMissedReplayed,
            "cron.worker",
            "run.missed_replayed",
            Level::Info,
            None,
            None,
        ),
        (
            Event::RunLaunched,
            "cron.worker",
            "run.launched",
            Level::Info,
            None,
            None,
        ),
        (
            Event::RunRecordFailed,
            "cron.worker",
            "run.record_failed",
            Level::Error,
            Some("failed"),
            Some("RUN_RECORD_FAILED"),
        ),
        (
            Event::ScheduleNextFailed,
            "cron.worker",
            "schedule.next_failed",
            Level::Error,
            Some("failed"),
            Some("SCHEDULE_NEXT_FAILED"),
        ),
        (
            Event::ScheduleStopped,
            "cron.worker",
            "schedule.stopped",
            Level::Info,
            Some("stopped"),
            None,
        ),
        (
            Event::ScheduleStopFailed,
            "cron.worker",
            "schedule.stop_failed",
            Level::Error,
            Some("failed"),
            Some("SCHEDULE_STOP_FAILED"),
        ),
        (
            Event::ScheduleDisabled,
            "cron.worker",
            "schedule.disabled",
            Level::Info,
            Some("stopped"),
            None,
        ),
        (
            Event::ScheduleDisableFailed,
            "cron.worker",
            "schedule.disable_failed",
            Level::Error,
            Some("failed"),
            Some("SCHEDULE_DISABLE_FAILED"),
        ),
        (
            Event::RunFenced,
            "cron.worker",
            "run.fenced",
            Level::Warn,
            Some("fenced"),
            Some("SCHEDULE_FENCED"),
        ),
        (
            Event::DueLaunched,
            "cron.scheduler",
            "due.launched",
            Level::Info,
            None,
            None,
        ),
        (
            Event::DueFenced,
            "cron.scheduler",
            "due.fenced",
            Level::Warn,
            Some("fenced"),
            Some("SCHEDULE_FENCED"),
        ),
        (
            Event::DuePartialWrite,
            "cron.scheduler",
            "due.partial_write",
            Level::Warn,
            Some("partial"),
            Some("FOLLOW_UP_FENCED"),
        ),
        (
            Event::DueFollowUpFailed,
            "cron.scheduler",
            "due.follow_up_failed",
            Level::Error,
            Some("partial"),
            Some("FOLLOW_UP_WRITE_FAILED"),
        ),
        (
            Event::DueRecordFailed,
            "cron.scheduler",
            "due.record_failed",
            Level::Error,
            Some("failed"),
            Some("RUN_RECORD_FAILED"),
        ),
        (
            Event::DueStopped,
            "cron.scheduler",
            "due.stopped",
            Level::Info,
            Some("stopped"),
            None,
        ),
        (
            Event::DueStopFailed,
            "cron.scheduler",
            "due.stop_failed",
            Level::Error,
            Some("failed"),
            Some("SCHEDULE_STOP_FAILED"),
        ),
        (
            Event::DueNextFailed,
            "cron.scheduler",
            "due.next_failed",
            Level::Error,
            Some("failed"),
            Some("SCHEDULE_NEXT_FAILED"),
        ),
        (
            Event::JobJoinFailed,
            "cron.worker",
            "job.join_failed",
            Level::Error,
            Some("failed"),
            Some("JOB_JOIN_FAILED"),
        ),
        #[cfg(feature = "monty")]
        (
            Event::DueJoinFailed,
            "cron.scheduler",
            "due.join_failed",
            Level::Error,
            Some("failed"),
            Some("DUE_JOIN_FAILED"),
        ),
        (
            Event::DueTaskFailed,
            "cron.scheduler",
            "due.task_failed",
            Level::Error,
            Some("failed"),
            Some("RUN_TASK_FAILED"),
        ),
        #[cfg(feature = "monty")]
        (
            Event::MontyResolveFailed,
            "cron.scheduler",
            "monty.resolve_failed",
            Level::Error,
            Some("failed"),
            Some("MONTY_RESOLVE"),
        ),
        #[cfg(feature = "monty")]
        (
            Event::MontyTickFailed,
            "cron.scheduler",
            "monty.tick_failed",
            Level::Error,
            Some("failed"),
            Some("MONTY_TICK"),
        ),
        #[cfg(feature = "monty")]
        (
            Event::FeedJoinFailed,
            "cron.monty",
            "feed.join_failed",
            Level::Error,
            Some("failed"),
            Some("FEED_JOIN_FAILED"),
        ),
    ];
    for (event, target, action, level, outcome, code) in rows {
        recorder.clear();
        if let Some(event) = event_record(
            logger.service_name().clone(),
            event,
            None,
            serde_json::json!({"error": "opaque"}),
        ) {
            let _ = logger.emit(event);
        };
        let events = recorder.events();
        assert_eq!(events.len(), 1);
        let actual = &events[0];
        assert_eq!(
            (actual.target.as_str(), actual.action.as_str(), actual.level),
            (target, action, level)
        );
        assert_eq!(actual.outcome.as_ref().map(|v| v.as_str()), outcome);
        assert_eq!(actual.diagnostic.as_ref().map(|v| v.code.as_str()), code);
        assert_eq!(actual.service.as_str(), "worker-test");
        assert!(actual.trace.is_none() && actual.state_transition.is_none());
        assert!(!actual.fields.contains_key("code"));
        if let Some(diagnostic) = &actual.diagnostic {
            assert_eq!(diagnostic.timestamp, actual.timestamp);
        }
    }
}

#[test]
fn python_resolved_target_carries_the_same_arc_through_run_target() {
    let recorder = Arc::new(RecordingCanonicalLog::new(
        ServiceName::new("python").unwrap(),
    ));
    let logger: SharedCanonicalLog = recorder;
    let store = Arc::new(
        crate::SqliteScheduleStore::open_in_memory(crate::ScheduleLimits::default()).unwrap(),
    );
    let clock = Arc::new(crate::SystemClock);
    let scheduler = crate::Scheduler::new_with_logger(
        store,
        clock.clone(),
        Arc::new(crate::SystemJitter),
        logger.clone(),
    );
    let mut spec = crate::ScheduleSpec::new(
        "python",
        "* * * * *",
        std::time::Duration::from_secs(1),
        serde_json::json!({}),
    )
    .unwrap();
    spec.hook = crate::HookTarget::python("/tmp/b15-hook.py").unwrap();
    let row = scheduler.start(spec).unwrap();
    let job: Arc<dyn crate::Job> = Arc::new(Idle);
    let before = Arc::strong_count(&logger);
    let target = resolve(&job, &row, &logger, None).unwrap();
    assert_eq!(Arc::strong_count(&logger), before + 1);
    let future = run_target(target, clock, &row, std::time::Instant::now());
    assert_eq!(Arc::strong_count(&logger), before + 1);
    drop(future);
    assert_eq!(Arc::strong_count(&logger), before);
}
#[cfg(feature = "monty")]
#[tokio::test]
async fn feed_join_panic_emits_origin_then_one_recorded_terminal_and_drops_guard() {
    let recorder = Arc::new(RecordingCanonicalLog::new(
        ServiceName::new("feed").unwrap(),
    ));
    let store = Arc::new(
        crate::SqliteScheduleStore::open_in_memory(crate::ScheduleLimits::default()).unwrap(),
    );
    let clock = Arc::new(crate::SystemClock);
    let scheduler = crate::Scheduler::new_with_logger(
        store.clone(),
        clock.clone(),
        Arc::new(crate::SystemJitter),
        recorder.clone(),
    );
    let row = scheduler
        .start(
            crate::ScheduleSpec::new(
                "feed-panic",
                "* * * * *",
                std::time::Duration::from_secs(1),
                serde_json::json!({}),
            )
            .unwrap(),
        )
        .unwrap();
    let coord = Arc::new(RunCoord::with_logger(recorder.clone()));
    let (_, shutdown) = tokio::sync::oneshot::channel();
    let mut worker = Worker::new(
        store.clone(),
        clock,
        Arc::new(Idle),
        MissedTickPolicy::Coalesce,
        coord.clone(),
        std::time::Duration::from_secs(30),
        shutdown,
        Arc::new(crate::MontyHostRegistry::new()),
    );
    worker.slots.push(Slot {
        schedule: row.clone(),
        next: None,
        run: None,
        queued: Queued::None,
        last_skipped: None,
    });
    let guard = MontyFeedGuard::enter(&coord);
    let result = tokio::spawn(async {
        panic!("feed panic");
        #[allow(unreachable_code)]
        crate::monty_run::FeedDone {
            session: None,
            result: crate::monty_run::FeedResult::Continue,
        }
    })
    .await;
    recorder.clear();
    assert!(
        worker
            .monty_feed_from_join(0, result, Some(guard), None)
            .await
            .is_none()
    );
    assert_eq!(coord.feeds.load(std::sync::atomic::Ordering::Acquire), 0);
    let events = recorder.events();
    assert_eq!(
        events.iter().map(|e| e.action.as_str()).collect::<Vec<_>>(),
        ["feed.join_failed", "run.finished"]
    );
    assert_eq!(
        events[0].diagnostic.as_ref().unwrap().code.as_str(),
        "FEED_JOIN_FAILED"
    );
    assert_eq!(
        events[1].diagnostic.as_ref().unwrap().code.as_str(),
        "JOB_FAILED"
    );
    let persisted = store.get(row.id).unwrap().unwrap().last_outcome.unwrap();
    assert_eq!(
        persisted.failure_parts().unwrap().code,
        Some("FEED_JOIN_FAILED")
    );
}
struct Idle;
#[async_trait::async_trait]
impl crate::Job for Idle {
    async fn should_stop(&self, _: &serde_json::Value) -> StopDecision {
        StopDecision::Continue
    }
    async fn run(&self, _: &serde_json::Value, _: std::time::Instant) -> RunOutcome {
        RunOutcome::Ok
    }
}
