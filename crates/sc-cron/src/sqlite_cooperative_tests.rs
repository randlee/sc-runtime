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
        wait.recv().unwrap();
    });
    entered.recv().unwrap();
    (release, thread)
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
    let worker = scheduler.spawn_worker_cooperative(Arc::new(Noop)).unwrap();
    let control = worker.stop_control();
    let mut status = control.subscribe_status();
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
    release.send(()).unwrap();
    holder.join().unwrap();
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
            let store =
                Arc::new(SqliteScheduleStore::open_in_memory(ScheduleLimits::default()).unwrap());
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
            assert!(outcome.completed_after_cutoff);
            if after_admission {
                assert_eq!(outcome.admission, CommitAdmission::Admitted);
                assert_eq!(outcome.persistence, PersistenceOutcome::Committed);
                assert_eq!(store.list_active().unwrap().len(), 1);
            } else {
                assert_eq!(outcome.admission, CommitAdmission::Vetoed);
                assert_eq!(outcome.persistence, PersistenceOutcome::RolledBack);
                assert!(store.list_active().unwrap().is_empty());
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
