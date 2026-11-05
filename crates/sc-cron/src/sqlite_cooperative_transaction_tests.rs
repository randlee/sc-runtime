//! Native transaction admission races and persistence evidence.
use super::*;

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn final_hook_stop_first_vetoes_and_admission_first_reports_committed_late() {
    if supervised(
        "transactions::final_hook_stop_first_vetoes_and_admission_first_reports_committed_late",
    ) {
        return;
    }
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
                            let _ = wait_release.recv_timeout(Duration::from_secs(5));
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
            let physical = PhysicalThread::new(physical, release.clone());
            wait_entered.recv_timeout(Duration::from_secs(5)).unwrap();
            let now = Instant::now();
            // This must acknowledge while the real hook is paused, proving no controller lock spans COMMIT.
            controller
                .stop(ShutdownBudget::new(now, now).unwrap())
                .unwrap();
            controller
                .stop(ShutdownBudget::new(now, now).unwrap())
                .unwrap();
            let _ = release.send(());
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

#[tokio::test]
async fn injected_lost_result_reports_indeterminate_without_claiming_a_physical_fsync_failure() {
    if supervised(
        "transactions::injected_lost_result_reports_indeterminate_without_claiming_a_physical_fsync_failure",
    ) {
        return;
    }
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
    if supervised(
        "transactions::implicit_update_commit_hook_races_preserve_generation_and_physical_outcome",
    ) {
        return;
    }
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
                            let _ = wait_release.recv_timeout(Duration::from_secs(5));
                        }
                    });
                    store.with_operation_context(&context, |store| store.set_enabled(&name, false))
                })
            };
            let physical = PhysicalThread::new(physical, release.clone());
            wait_entered.recv_timeout(Duration::from_secs(5)).unwrap();
            let now = Instant::now();
            controller
                .stop(ShutdownBudget::new(now, now).unwrap())
                .unwrap();
            controller
                .stop(ShutdownBudget::new(now, now).unwrap())
                .unwrap();
            let _ = release.send(());
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
