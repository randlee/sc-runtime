//! Causal tests use held physical locks and commit-hook barriers, with watchdogs only for failure.
use super::*;
use crate::{
    CommitAdmission, PersistenceOutcome, ScheduleQuery, ScheduleSelection, Scheduler,
    ShutdownBudget, ShutdownStatus, StoreOperationContext, SystemClock, SystemJitter,
};
use std::sync::{Arc, mpsc};
use std::time::Instant;

type TestScheduler = Scheduler<SqliteScheduleStore, SystemClock, SystemJitter>;
fn make_scheduler(capacity: u32) -> TestScheduler {
    Scheduler::new(
        Arc::new(SqliteScheduleStore::open_in_memory(ScheduleLimits::default()).unwrap()),
        Arc::new(SystemClock),
        Arc::new(SystemJitter),
    )
    .with_command_capacity(capacity)
    .unwrap()
}
fn spec(name: &str) -> ScheduleSpec {
    ScheduleSpec::new(
        name,
        "* * * * *",
        Duration::from_secs(1),
        serde_json::json!({}),
    )
    .unwrap()
}
struct Noop;
#[async_trait::async_trait]
impl crate::Job for Noop {
    async fn run(&self, _: &serde_json::Value, _: Instant) -> RunOutcome {
        RunOutcome::Ok
    }
    async fn should_stop(&self, _: &serde_json::Value) -> crate::StopDecision {
        crate::StopDecision::Continue
    }
}
fn hold(store: Arc<SqliteScheduleStore>) -> (mpsc::Sender<()>, std::thread::JoinHandle<()>) {
    let (held, entered) = mpsc::channel();
    let (release, wait) = mpsc::channel();
    let thread = std::thread::spawn(move || {
        let _connection = store.connection.lock().unwrap();
        held.send(()).unwrap();
        let _ = wait.recv();
    });
    entered.recv().unwrap();
    (release, thread)
}
// Release and physically join even if an assertion/watchdog fails.
struct HeldConnection(
    Option<mpsc::Sender<()>>,
    Option<std::thread::JoinHandle<()>>,
);
impl Drop for HeldConnection {
    fn drop(&mut self) {
        self.0.take();
        if let Some(holder) = self.1.take() {
            let _ = holder.join();
        }
    }
}

async fn commands(context: &StoreOperationContext, count: u32) {
    tokio::time::timeout(Duration::from_secs(5), async {
        while context.state.counts().1 != count {
            tokio::task::yield_now().await;
        }
    })
    .await
    .expect("admission handshake");
}
fn assert_reason(error: CronError, reason: &str) {
    let CronError::Store(error) = error else {
        panic!("expected store error");
    };
    assert_eq!(error.context(), reason);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn held_initial_load_and_cancelled_waiters_retain_capacity_until_physical_completion() {
    let scheduler = Arc::new(make_scheduler(1));
    let (release, holder) = hold(Arc::clone(scheduler.store()));
    let worker = scheduler.spawn_worker_cooperative(Arc::new(Noop)).unwrap();
    let control = worker.stop_control();
    let context = control.operation_context();
    let admitted = {
        let scheduler = Arc::clone(&scheduler);
        let context = context.clone();
        tokio::spawn(async move { scheduler.start_with_context(spec("held"), context).await })
    };
    commands(&context, 1).await;
    assert_reason(
        scheduler
            .query_with_context(ScheduleQuery::default(), context.clone())
            .await
            .unwrap_err(),
        "command_capacity_exhausted",
    );
    admitted.abort();
    let _ = admitted.await;
    assert_eq!(
        context.state.counts().1,
        1,
        "dropping transport must retain the root permit"
    );
    let now = Instant::now();
    control
        .request_stop(
            ShutdownBudget::new(
                now + Duration::from_secs(60),
                now + Duration::from_secs(120),
            )
            .unwrap(),
        )
        .unwrap();
    control
        .request_stop(
            ShutdownBudget::new(
                now + Duration::from_secs(100),
                now + Duration::from_secs(200),
            )
            .unwrap(),
        )
        .unwrap();
    let budget = context.state.budget().unwrap();
    assert!(budget.grace() <= Instant::now());
    assert_eq!(budget.cleanup(), now + Duration::from_secs(120));
    // Physical mutex remains held: cancellation must reach waiting store work independently.
    tokio::time::timeout(Duration::from_secs(5), context.state.join())
        .await
        .unwrap();
    assert_eq!(context.state.counts(), (0, 0));
    let report = tokio::time::timeout(Duration::from_secs(5), worker.join_owned())
        .await
        .unwrap()
        .unwrap();
    assert!(report.grace_exceeded);
    release.send(()).unwrap();
    holder.join().unwrap();
    assert!(scheduler.store().list_active().unwrap().is_empty());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn full_default64_is_shared_by_handles_and_configuration_latches() {
    let scheduler = Arc::new(Scheduler::new(
        Arc::new(SqliteScheduleStore::open_in_memory(ScheduleLimits::default()).unwrap()),
        Arc::new(SystemClock),
        Arc::new(SystemJitter),
    ));
    let (release, holder) = hold(Arc::clone(scheduler.store()));
    let worker = scheduler.spawn_worker_cooperative(Arc::new(Noop)).unwrap();
    let control = worker.stop_control();
    let context = control.operation_context();
    assert!(scheduler.share().with_command_capacity(2).is_err());
    let mut waiters = Vec::new();
    for n in 0..64 {
        let shared = scheduler.share();
        let context = context.clone();
        waiters.push(tokio::spawn(async move {
            shared
                .start_with_context(spec(&format!("queued-{n}")), context)
                .await
        }));
    }
    commands(&context, 64).await;
    assert_reason(
        scheduler
            .share()
            .query_with_context(ScheduleQuery::default(), context.clone())
            .await
            .unwrap_err(),
        "command_capacity_exhausted",
    );
    control.cancel_now();
    for waiter in waiters {
        assert!(waiter.await.unwrap().is_err());
    }
    tokio::time::timeout(Duration::from_secs(5), worker.join_owned())
        .await
        .unwrap()
        .unwrap();
    release.send(()).unwrap();
    holder.join().unwrap();
    assert!(scheduler.store().list_active().unwrap().is_empty());
    for capacity in [0, 65537] {
        assert!(make_scheduler(1).with_command_capacity(capacity).is_err());
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn real_grace_expiry_is_published_without_join_or_releasing_connection() {
    let scheduler = make_scheduler(2);
    let (release, holder) = hold(Arc::clone(scheduler.store()));
    let _held = HeldConnection(Some(release), Some(holder));
    let worker = scheduler.spawn_worker_cooperative(Arc::new(Noop)).unwrap();
    let control = worker.stop_control();
    let mut status = control.subscribe_status();
    let context = control.operation_context();
    tokio::time::timeout(Duration::from_secs(5), async {
        while context.state.counts().0 == 0 {
            tokio::task::yield_now().await;
        }
    })
    .await
    .expect("initial load is physically admitted before stop");
    let now = Instant::now();
    control
        .request_stop(
            ShutdownBudget::new(
                now + Duration::from_millis(25),
                now + Duration::from_millis(50),
            )
            .unwrap(),
        )
        .unwrap();
    tokio::time::timeout(
        Duration::from_secs(2),
        status.wait_for(|status| *status == ShutdownStatus::DeadlineExceeded),
    )
    .await
    .unwrap()
    .unwrap();
    assert!(Instant::now() >= now + Duration::from_millis(25));
    assert!(control.operation_context().check().is_err());
    tokio::time::timeout(Duration::from_secs(2), worker.join_owned())
        .await
        .unwrap()
        .unwrap();
}

#[tokio::test]
async fn completed_work_retains_missed_deadline_status_before_timer_is_polled() {
    for cleanup in [false, true] {
        let controller = Arc::new(crate::operation::Controller::new());
        // No spawned timer can run until the first await. The physical operation
        // completes after the chosen cutoff, then the worker finishes before stop.
        let cutoff = Instant::now();
        let guard = controller.register(cleanup, None, false).unwrap();
        drop(guard);
        controller.worker_finished();
        let mut status = controller.subscribe();
        controller
            .stop(ShutdownBudget::new(cutoff, cutoff).unwrap())
            .unwrap();
        assert_eq!(
            *status.borrow_and_update(),
            ShutdownStatus::DeadlineExceeded
        );
        controller.join().await;
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn query_is_bounded_includes_ended_and_end_fences_replacement() {
    let scheduler = make_scheduler(2);
    let worker = scheduler.spawn_worker_cooperative(Arc::new(Noop)).unwrap();
    let control = worker.stop_control();
    let context = control.operation_context();
    let a = scheduler
        .start_with_context(spec("a"), context.clone())
        .await
        .unwrap();
    let b = scheduler
        .start_with_context(spec("b"), context.clone())
        .await
        .unwrap();
    scheduler
        .stop_with_context(a.id, a.generation, "ended".into(), context.clone())
        .await
        .unwrap();
    let page = scheduler
        .query_with_context(
            ScheduleQuery {
                selection: ScheduleSelection::All,
                limit: 1,
                ..Default::default()
            },
            context.clone(),
        )
        .await
        .unwrap();
    assert_eq!(page.items[0].id, a.id);
    assert_eq!(page.next_after_id, Some(a.id));
    let next = scheduler
        .query_with_context(
            ScheduleQuery {
                selection: ScheduleSelection::All,
                limit: 1,
                after_id: page.next_after_id,
                ..Default::default()
            },
            context.clone(),
        )
        .await
        .unwrap();
    assert_eq!(next.items[0].id, b.id);
    assert!(next.next_after_id.is_none());
    assert_eq!(
        scheduler
            .query_with_context(
                ScheduleQuery {
                    selection: ScheduleSelection::Ended,
                    ..Default::default()
                },
                context.clone()
            )
            .await
            .unwrap()
            .items
            .len(),
        1
    );
    let replacement = scheduler
        .start_with_context(spec("a"), context.clone())
        .await
        .unwrap();
    assert!(matches!(
        crate::CronOperations::end(
            &scheduler,
            context.clone(),
            crate::EndSchedule {
                name: a.spec.name,
                generation: a.generation,
                reason: "stale".into()
            }
        )
        .await,
        Err(CronError::Fenced { .. })
    ));
    assert_eq!(
        scheduler.store().get(a.id).unwrap().unwrap().generation,
        replacement.generation
    );
    for limit in [0, 1001] {
        assert!(
            scheduler
                .query_with_context(
                    ScheduleQuery {
                        limit,
                        ..Default::default()
                    },
                    context.clone()
                )
                .await
                .is_err()
        );
    }
    assert!(
        scheduler
            .query_with_context(
                ScheduleQuery {
                    after_id: Some(ScheduleId::new(0)),
                    ..Default::default()
                },
                context
            )
            .await
            .is_err()
    );
    control.cancel_now();
    worker.join_owned().await.unwrap();
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn final_hook_stop_first_vetoes_and_admission_first_reports_committed_late() {
    for cleanup in [false, true] {
        for after_admission in [false, true] {
            let directory = tempfile::tempdir().unwrap();
            let path = directory.path().join("commit-race.db");
            let store =
                Arc::new(SqliteScheduleStore::open(&path, ScheduleLimits::default()).unwrap());
            let controller = Arc::new(crate::operation::Controller::new());
            let root = StoreOperationContext::root(Arc::clone(&controller));
            let context = if cleanup { root.terminal() } else { root };
            let (entered, wait_entered) = mpsc::channel();
            let (release, wait_release) = mpsc::channel();
            let physical = {
                let store = Arc::clone(&store);
                let context = context.clone();
                std::thread::spawn(move || {
                    scope::set_commit_gate(move |after| {
                        if after == after_admission {
                            entered.send(()).unwrap();
                            wait_release.recv().unwrap();
                        }
                    });
                    store.with_operation_context(&context, |store| {
                        store.create(
                            &NewSchedule {
                                spec: spec("commit-race"),
                                jitter: Duration::ZERO,
                                next_run_at: Timestamp::from_datetime(chrono::Utc::now()),
                            },
                            Timestamp::from_datetime(chrono::Utc::now()),
                        )
                    })
                })
            };
            wait_entered.recv_timeout(Duration::from_secs(5)).unwrap();
            let now = Instant::now();
            // This must acknowledge while the real hook is paused, proving no controller lock spans COMMIT.
            controller
                .stop(ShutdownBudget::new(now, now).unwrap())
                .unwrap();
            controller
                .stop(ShutdownBudget::new(now, now).unwrap())
                .unwrap();
            release.send(()).unwrap();
            let error = physical.join().unwrap().unwrap_err();
            let CronError::Store(error) = error else {
                panic!("typed transaction failure");
            };
            let outcome = error.transaction_outcome().unwrap();
            let independent = SqliteScheduleStore::open(&path, ScheduleLimits::default()).unwrap();
            assert!(outcome.completed_after_cutoff);
            if after_admission {
                assert_eq!(outcome.admission, CommitAdmission::Admitted);
                assert_eq!(outcome.persistence, PersistenceOutcome::Committed);
                assert_eq!(independent.list_active().unwrap().len(), 1);
            } else {
                assert_eq!(outcome.admission, CommitAdmission::Vetoed);
                assert_eq!(outcome.persistence, PersistenceOutcome::RolledBack);
                assert!(independent.list_active().unwrap().is_empty());
            }
            // Hooks and TLS are restored, so unrelated legacy work can commit.
            let now = Timestamp::from_datetime(chrono::Utc::now());
            store
                .create(
                    &NewSchedule {
                        spec: spec("baseline-restored"),
                        jitter: Duration::ZERO,
                        next_run_at: now,
                    },
                    now,
                )
                .unwrap();
        }
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn every_store_path_observes_shortening_while_waiting_on_real_connection_mutex() {
    for operation in 0..12 {
        let scheduler = make_scheduler(2);
        let row = scheduler.start(spec("existing")).unwrap();
        let store = Arc::clone(scheduler.store());
        let (release, holder) = hold(Arc::clone(&store));
        let controller = Arc::new(crate::operation::Controller::new());
        let context = StoreOperationContext::root(Arc::clone(&controller));
        let (entered, wait_entered) = mpsc::channel();
        let physical = std::thread::spawn(move || {
            scope::set_observer(move |stage| {
                if stage == scope::TestStage::LockWait {
                    let _ = entered.send(());
                }
            });
            store.with_operation_context(&context, |store| {
                let now = Timestamp::from_datetime(chrono::Utc::now());
                let flags = [(row.spec.name.clone(), false)];
                match operation {
                    0 => store.get(row.id).map(|_| ()),
                    1 => store.list_active().map(|_| ()),
                    2 => store.query(&ScheduleQuery::default()).map(|_| ()),
                    3 => store
                        .record_run(row.id, row.generation, now, &RunOutcome::Ok)
                        .map(|_| ()),
                    4 => store.set_next(row.id, row.generation, now).map(|_| ()),
                    5 => store.stop(row.id, row.generation, now, "stop").map(|_| ()),
                    6 => store.set_enabled(&row.spec.name, false).map(|_| ()),
                    7 => store
                        .disable(row.id, row.generation, now, "disable")
                        .map(|_| ()),
                    8 => store.set_enabled_many(&flags).map(|_| ()),
                    9 => store
                        .replace_and_set_enabled(
                            &NewSchedule {
                                spec: spec("replacement"),
                                jitter: Duration::ZERO,
                                next_run_at: now,
                            },
                            true,
                            now,
                            &flags,
                        )
                        .map(|_| ()),
                    10 => store
                        .create(
                            &NewSchedule {
                                spec: spec("new"),
                                jitter: Duration::ZERO,
                                next_run_at: now,
                            },
                            now,
                        )
                        .map(|_| ()),
                    _ => store
                        .create(
                            &NewSchedule {
                                spec: spec("existing"),
                                jitter: Duration::ZERO,
                                next_run_at: now,
                            },
                            now,
                        )
                        .map(|_| ()),
                }
            })
        });
        wait_entered.recv_timeout(Duration::from_secs(5)).unwrap();
        let now = Instant::now();
        controller
            .stop(ShutdownBudget::new(now, now).unwrap())
            .unwrap();
        // The cancellation result is available while the physical mutex is still held.
        let error = physical.join().unwrap().unwrap_err();
        assert!(
            matches!(error, CronError::Store(ref error) if error.kind() == StoreErrorKind::DeadlineExceeded),
            "path {operation}: {error}"
        );
        release.send(()).unwrap();
        holder.join().unwrap();
        assert_eq!(
            scheduler.store().get(row.id).unwrap().unwrap().last_outcome,
            None
        );
        assert_eq!(scheduler.store().list_active().unwrap().len(), 1);
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn sqlite_busy_progress_and_row_decode_observe_live_cutoff_and_restore_scope() {
    for stage in [
        scope::TestStage::Busy,
        scope::TestStage::Progress,
        scope::TestStage::Decode,
    ] {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("cron.db");
        let store = Arc::new(SqliteScheduleStore::open(&path, ScheduleLimits::default()).unwrap());
        let now = Timestamp::from_datetime(chrono::Utc::now());
        store
            .create(
                &NewSchedule {
                    spec: spec("existing"),
                    jitter: Duration::ZERO,
                    next_run_at: now,
                },
                now,
            )
            .unwrap();
        let peer = Connection::open(&path).unwrap();
        if stage == scope::TestStage::Busy {
            peer.execute_batch("BEGIN IMMEDIATE").unwrap();
        }
        let controller = Arc::new(crate::operation::Controller::new());
        let context = StoreOperationContext::root(Arc::clone(&controller));
        let (entered, wait_entered) = mpsc::channel();
        let (release, wait_release) = mpsc::channel();
        let physical = {
            let store = Arc::clone(&store);
            std::thread::spawn(move || {
                let once = std::cell::Cell::new(false);
                scope::set_observer(move |observed| {
                    if observed == stage && !once.replace(true) {
                        entered.send(()).unwrap();
                        wait_release.recv().unwrap();
                    }
                });
                store.with_operation_context(&context, |store| {
                match stage {
                    scope::TestStage::Busy => store.set_enabled(&ScheduleName::new("existing").unwrap(), false).map(|_| ()),
                    scope::TestStage::Progress => {
                        let connection = store.lock()?;
                        connection.query_row("WITH RECURSIVE numbers(n) AS (VALUES(0) UNION ALL SELECT n+1 FROM numbers WHERE n<100000000) SELECT sum(n) FROM numbers", [], |row| row.get::<_, i64>(0)).map(|_| ()).map_err(|error| store_error("progress_query", error))
                    }
                    _ => store.list_active().map(|_| ()),
                }
            })
            })
        };
        wait_entered.recv_timeout(Duration::from_secs(5)).unwrap();
        let now = Instant::now();
        controller
            .stop(ShutdownBudget::new(now, now).unwrap())
            .unwrap();
        release.send(()).unwrap();
        let error = physical.join().unwrap().unwrap_err();
        if stage == scope::TestStage::Busy {
            let CronError::Store(error) = error else {
                panic!("typed busy cancellation");
            };
            assert_eq!(error.kind(), StoreErrorKind::Indeterminate);
            assert_eq!(
                error.transaction_outcome().unwrap().persistence,
                PersistenceOutcome::Indeterminate
            );
            peer.execute_batch("ROLLBACK").unwrap();
            assert!(
                store.list_active().is_err(),
                "unknown mutation evidence quarantines this connection"
            );
            let independent = SqliteScheduleStore::open(&path, ScheduleLimits::default()).unwrap();
            assert!(independent.list_active().unwrap()[0].enabled);
        } else {
            assert!(
                matches!(error, CronError::Store(ref error) if error.kind() == StoreErrorKind::DeadlineExceeded)
            );
            assert!(
                store
                    .set_enabled(&ScheduleName::new("existing").unwrap(), true)
                    .unwrap()
            );
            assert_eq!(store.list_active().unwrap().len(), 1);
        }
    }
}

#[tokio::test]
async fn injected_lost_result_reports_indeterminate_without_claiming_a_physical_fsync_failure() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("lost-result.db");
    let store = SqliteScheduleStore::open(&path, ScheduleLimits::default()).unwrap();
    let context = crate::operation_context_probe(Instant::now() + Duration::from_secs(60));
    let error = store
        .with_operation_context(&context, |store| {
            let now = Timestamp::from_datetime(chrono::Utc::now());
            store.create(
                &NewSchedule {
                    spec: spec("lost-result"),
                    jitter: Duration::ZERO,
                    next_run_at: now,
                },
                now,
            )?;
            scope::lose_result();
            Err::<(), _>(
                StoreError::new(StoreErrorKind::Indeterminate, "injected_lost_result").into(),
            )
        })
        .unwrap_err();
    let CronError::Store(error) = error else {
        panic!("transaction evidence");
    };
    assert_eq!(error.kind(), StoreErrorKind::Indeterminate);
    assert_eq!(
        error.transaction_outcome().unwrap().admission,
        CommitAdmission::Admitted
    );
    assert_eq!(
        error.transaction_outcome().unwrap().persistence,
        PersistenceOutcome::Indeterminate
    );
    // Independent authorized observation proves the fixture committed, not causal certainty for the lost result.
    assert!(
        store.list_active().is_err(),
        "uncertain connection is quarantined"
    );
    let independent = SqliteScheduleStore::open(&path, ScheduleLimits::default()).unwrap();
    assert_eq!(independent.list_active().unwrap().len(), 1);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn implicit_update_commit_hook_races_preserve_generation_and_physical_outcome() {
    for cleanup in [false, true] {
        for after_admission in [false, true] {
            let directory = tempfile::tempdir().unwrap();
            let path = directory.path().join("commit-race.db");
            let store =
                Arc::new(SqliteScheduleStore::open(&path, ScheduleLimits::default()).unwrap());
            let now = Timestamp::from_datetime(chrono::Utc::now());
            let row = store
                .create(
                    &NewSchedule {
                        spec: spec("implicit"),
                        jitter: Duration::ZERO,
                        next_run_at: now,
                    },
                    now,
                )
                .unwrap();
            let controller = Arc::new(crate::operation::Controller::new());
            let context = StoreOperationContext::root(Arc::clone(&controller));
            let context = if cleanup { context.terminal() } else { context };
            let (entered, wait_entered) = mpsc::channel();
            let (release, wait_release) = mpsc::channel();
            let physical = {
                let store = Arc::clone(&store);
                let name = row.spec.name.clone();
                std::thread::spawn(move || {
                    scope::set_commit_gate(move |after| {
                        if after == after_admission {
                            entered.send(()).unwrap();
                            wait_release.recv().unwrap();
                        }
                    });
                    store.with_operation_context(&context, |store| store.set_enabled(&name, false))
                })
            };
            wait_entered.recv_timeout(Duration::from_secs(5)).unwrap();
            let now = Instant::now();
            controller
                .stop(ShutdownBudget::new(now, now).unwrap())
                .unwrap();
            controller
                .stop(ShutdownBudget::new(now, now).unwrap())
                .unwrap();
            release.send(()).unwrap();
            let error = physical.join().unwrap().unwrap_err();
            let CronError::Store(error) = error else {
                panic!("transaction evidence");
            };
            let outcome = error.transaction_outcome().unwrap();
            assert!(outcome.completed_after_cutoff);
            assert_eq!(
                outcome.admission,
                if after_admission {
                    CommitAdmission::Admitted
                } else {
                    CommitAdmission::Vetoed
                }
            );
            assert_eq!(
                outcome.persistence,
                if after_admission {
                    PersistenceOutcome::Committed
                } else {
                    PersistenceOutcome::RolledBack
                }
            );
            let independent = SqliteScheduleStore::open(&path, ScheduleLimits::default()).unwrap();
            let actual = independent.get(row.id).unwrap().unwrap();
            assert_eq!(actual.generation, row.generation);
            assert_eq!(actual.enabled, !after_admission);
        }
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn generation_fenced_terminal_cleanup_has_independent_admission_while_command_intake_is_full()
{
    struct GateJob {
        entered: tokio::sync::Notify,
        release: tokio::sync::Notify,
    }
    #[async_trait::async_trait]
    impl crate::Job for GateJob {
        async fn run(&self, _: &serde_json::Value, _: Instant) -> RunOutcome {
            self.entered.notify_one();
            self.release.notified().await;
            RunOutcome::Ok
        }
        async fn should_stop(&self, _: &serde_json::Value) -> crate::StopDecision {
            crate::StopDecision::Continue
        }
    }
    let clock = Arc::new(crate::ManualClock::new(
        "2026-10-19T00:00:00Z".parse().unwrap(),
    ));
    let scheduler = Arc::new(
        Scheduler::new(
            Arc::new(SqliteScheduleStore::open_in_memory(ScheduleLimits::default()).unwrap()),
            Arc::clone(&clock),
            Arc::new(crate::SeqJitter::default()),
        )
        .with_command_capacity(1)
        .unwrap(),
    );
    let row = scheduler.start(spec("cleanup")).unwrap();
    let job = Arc::new(GateJob {
        entered: tokio::sync::Notify::new(),
        release: tokio::sync::Notify::new(),
    });
    let worker = scheduler.spawn_worker_cooperative(job.clone()).unwrap();
    let control = worker.stop_control();
    let context = control.operation_context();
    clock.wait_parks_above(0).await;
    clock.advance_settled(chrono::Duration::seconds(60)).await;
    tokio::time::timeout(Duration::from_secs(5), job.entered.notified())
        .await
        .unwrap();
    let (release, holder) = hold(Arc::clone(scheduler.store()));
    let command = {
        let scheduler = Arc::clone(&scheduler);
        let context = context.clone();
        tokio::spawn(async move {
            scheduler
                .query_with_context(ScheduleQuery::default(), context)
                .await
        })
    };
    commands(&context, 1).await;
    let now = Instant::now();
    control
        .request_stop(
            ShutdownBudget::new(
                now + Duration::from_secs(60),
                now + Duration::from_secs(120),
            )
            .unwrap(),
        )
        .unwrap();
    job.release.notify_one();
    tokio::time::timeout(Duration::from_secs(5), async {
        while context.state.counts().0 < 2 {
            tokio::task::yield_now().await;
        }
    })
    .await
    .unwrap();
    assert_eq!(
        context.state.counts().1,
        1,
        "cleanup neither consumes nor bypasses public command capacity"
    );
    release.send(()).unwrap();
    holder.join().unwrap();
    command.await.unwrap().unwrap();
    let report = worker.join_owned().await.unwrap();
    assert!(!report.grace_exceeded);
    assert!(!report.cleanup_exceeded);
    assert_eq!(report.cleanup.persistence, PersistenceOutcome::Committed);
    let actual = scheduler.store().get(row.id).unwrap().unwrap();
    assert_eq!(actual.generation, row.generation);
    assert_eq!(actual.last_outcome, Some(RunOutcome::Ok));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn uncertain_worker_end_is_reported_once_without_automatic_mutation_replay() {
    struct DeleteJob(Arc<SqliteScheduleStore>);
    #[async_trait::async_trait]
    impl crate::Job for DeleteJob {
        async fn run(&self, _: &serde_json::Value, _: Instant) -> RunOutcome {
            panic!("delete stop check must prevent tick")
        }
        async fn should_stop(&self, _: &serde_json::Value) -> crate::StopDecision {
            self.0
                .lose_write_result
                .store(true, std::sync::atomic::Ordering::Release);
            crate::StopDecision::Delete("terminal".into())
        }
    }
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("end-loss.db");
    let store = Arc::new(SqliteScheduleStore::open(&path, ScheduleLimits::default()).unwrap());
    let clock = Arc::new(crate::ManualClock::new(
        "2026-10-19T00:00:00Z".parse().unwrap(),
    ));
    let scheduler = Scheduler::new(
        Arc::clone(&store),
        Arc::clone(&clock),
        Arc::new(crate::SeqJitter::default()),
    );
    let row = scheduler.start(spec("uncertain-end")).unwrap();
    let worker = scheduler
        .spawn_worker_cooperative(Arc::new(DeleteJob(Arc::clone(&store))))
        .unwrap();
    let control = worker.stop_control();
    clock.wait_parks_above(0).await;
    clock.advance_settled(chrono::Duration::seconds(60)).await;
    tokio::time::timeout(Duration::from_secs(5), async {
        while !store.quarantined.load(std::sync::atomic::Ordering::Acquire) {
            tokio::task::yield_now().await;
        }
    })
    .await
    .unwrap();
    clock.advance_settled(chrono::Duration::seconds(60)).await;
    assert_eq!(
        store
            .stop_attempts
            .load(std::sync::atomic::Ordering::Acquire),
        1,
        "uncertain end must not be replayed by retry bookkeeping"
    );
    control.cancel_now();
    worker.join_owned().await.unwrap();
    let independent = SqliteScheduleStore::open(&path, ScheduleLimits::default()).unwrap();
    let actual = independent.get(row.id).unwrap().unwrap();
    assert_eq!(actual.generation, row.generation);
    assert!(!actual.is_active());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn delayed_join_poll_reports_physical_completion_before_shortened_cutoffs() {
    let scheduler = make_scheduler(1);
    let worker = scheduler.spawn_worker_cooperative(Arc::new(Noop)).unwrap();
    let control = worker.stop_control();
    let now = Instant::now();
    control
        .request_stop(
            ShutdownBudget::new(
                now + Duration::from_secs(60),
                now + Duration::from_secs(120),
            )
            .unwrap(),
        )
        .unwrap();
    tokio::time::timeout(Duration::from_secs(5), async {
        while !worker.is_finished() {
            tokio::task::yield_now().await;
        }
    })
    .await
    .unwrap();
    // Physically complete before the second, already-current cutoff; no arbitrary sleep is needed.
    let cutoff = Instant::now();
    control
        .request_stop(ShutdownBudget::new(cutoff, cutoff).unwrap())
        .unwrap();
    let report = worker.join_owned().await.unwrap();
    assert!(!report.grace_exceeded);
    assert!(!report.cleanup_exceeded);
    assert_eq!(
        report.cleanup.persistence,
        PersistenceOutcome::NoMutationAttempted
    );
    assert!(!report.cleanup.completed_after_cutoff);
}

#[cfg(test)]
#[test]
fn queued_closures_keep_permits_and_recheck_scope_before_first_physical_store_call() {
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(2)
        .max_blocking_threads(1)
        .enable_all()
        .build()
        .unwrap();
    runtime.block_on(async {
        let (entered, wait_entered) = mpsc::channel();
        let (release, wait_release) = mpsc::channel();
        let occupied = tokio::task::spawn_blocking(move || {
            entered.send(()).unwrap();
            wait_release.recv().unwrap();
        });
        wait_entered.recv_timeout(Duration::from_secs(5)).unwrap();
        let scheduler = Arc::new(make_scheduler(1));
        let worker = scheduler.spawn_worker_cooperative(Arc::new(Noop)).unwrap();
        let control = worker.stop_control();
        let context = control.operation_context();
        let command = {
            let scheduler = Arc::clone(&scheduler);
            let context = context.clone();
            tokio::spawn(async move {
                scheduler
                    .start_with_context(spec("never-submitted-to-sqlite"), context)
                    .await
            })
        };
        commands(&context, 1).await;
        assert_reason(
            scheduler
                .query_with_context(ScheduleQuery::default(), context.clone())
                .await
                .unwrap_err(),
            "command_capacity_exhausted",
        );
        command.abort();
        let _ = command.await;
        control.cancel_now();
        assert_eq!(
            context.state.counts().1,
            1,
            "queued physical work still owns its permit after waiter cancellation"
        );
        release.send(()).unwrap();
        occupied.await.unwrap();
        worker.join_owned().await.unwrap();
        assert_eq!(context.state.counts(), (0, 0));
        assert!(scheduler.store().list_active().unwrap().is_empty());
    });
}
