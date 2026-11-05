//! Causal lifecycle, private-cause and failure-cleanup regressions.
use super::*;
#[tokio::test(start_paused = true)]
async fn ordinary_thread_stop_notifies_while_admitted_native_commit_is_held() {
    if supervised("lifecycle::ordinary_thread_stop_notifies_while_admitted_native_commit_is_held") {
        return;
    }
    let control = crate::WorkerStopControl::new().unwrap();
    let context = control.operation_context();
    let controller = Arc::clone(&context.state);
    let timer = Arc::clone(&controller);
    let store = Arc::new(SqliteScheduleStore::open_in_memory(ScheduleLimits::default()).unwrap());
    let (entered, ready) = mpsc::channel();
    let (release, wait) = mpsc::channel();
    let guard = controller.register(false, None, true).unwrap();
    let operation = tokio::spawn(async move {
        controller
            .run(guard, move || {
                scope::set_commit_gate(move |after| {
                    if after {
                        let _ = entered.send(());
                        let _ = wait.recv_timeout(Duration::from_secs(5));
                    }
                });
                store.with_operation_context(&context, |store| {
                    let now = Timestamp::from_datetime(chrono::Utc::now());
                    store.create(
                        &NewSchedule {
                            spec: spec("held-native"),
                            jitter: Duration::ZERO,
                            next_run_at: now,
                        },
                        now,
                    )
                })
            })
            .await
    });
    tokio::task::spawn_blocking(move || ready.recv_timeout(Duration::from_secs(5)))
        .await
        .unwrap()
        .unwrap();
    let mut status = control.subscribe_status();
    let now = tokio::time::Instant::now().into_std();
    let grace = now + Duration::from_secs(10);
    let cleanup = now + Duration::from_secs(20);
    let other = control.clone();
    physical_join(std::thread::spawn(move || {
        other.request_stop(ShutdownBudget::new(grace, cleanup).unwrap())
    }))
    .unwrap()
    .unwrap();
    let observed = tokio::time::timeout(Duration::from_secs(30), async {
        // This witness belongs to the retained runtime timer, not synchronous
        // request_stop. Controlled time cannot reach G before timer admission.
        timer.timer_armed.notified().await;
        assert_eq!(*status.borrow(), ShutdownStatus::Draining);
        tokio::time::advance(Duration::from_secs(10)).await;
        status
            .wait_for(|s| *s == ShutdownStatus::DeadlineExceeded)
            .await
            .map(|_| ())
    })
    .await;
    // A second stop occurs only after the timer observation is recorded; it
    // cannot make a missing timer pass. Release native work on failure too.
    let other = control.clone();
    let shortened = physical_join(std::thread::spawn(move || {
        other.request_stop(
            ShutdownBudget::new(
                Instant::now() + Duration::from_secs(2),
                cleanup + Duration::from_secs(3),
            )
            .unwrap(),
        )
    }));
    let _ = release.send(());
    let result = operation.await.unwrap();
    observed.unwrap().unwrap();
    shortened.unwrap().unwrap();
    assert!(result.is_err());
    assert_eq!(
        control
            .operation_context()
            .state
            .budget()
            .unwrap()
            .cleanup(),
        cleanup
    );
}

#[tokio::test]
async fn cooperative_duplicate_start_is_typed_and_original_worker_still_joins() {
    if supervised("lifecycle::cooperative_duplicate_start_is_typed_and_original_worker_still_joins")
    {
        return;
    }
    let scheduler = make_scheduler(1);
    let worker = scheduler.spawn_worker_cooperative(Arc::new(Noop)).unwrap();
    let refused = scheduler.spawn_worker_cooperative(Arc::new(Noop));
    assert!(
        matches!(refused, Err(CronError::Store(ref e)) if e.context()=="worker_already_running")
    );
    worker.stop_control().cancel_now();
    tokio::time::timeout(Duration::from_secs(5), worker.join_owned())
        .await
        .unwrap()
        .unwrap();
}

#[cfg(test)]
#[test]
fn cooperative_start_without_runtime_returns_error_without_claiming_worker() {
    if supervised(
        "lifecycle::cooperative_start_without_runtime_returns_error_without_claiming_worker",
    ) {
        return;
    }
    let scheduler = make_scheduler(1);
    assert!(
        matches!(scheduler.spawn_worker_cooperative(Arc::new(Noop)),Err(CronError::Store(ref e)) if e.context()=="worker_runtime_unavailable")
    );
    assert!(crate::WorkerStopControl::new().is_err());
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
    runtime.block_on(async {
        let worker = scheduler.spawn_worker_cooperative(Arc::new(Noop)).unwrap();
        worker.stop_control().cancel_now();
        worker.join_owned().await.unwrap();
    });
}

#[cfg(test)]
#[test]
fn physical_permit_survives_shutdown_of_its_async_runtime() {
    if supervised("lifecycle::physical_permit_survives_shutdown_of_its_async_runtime") {
        return;
    }
    let (stopped, worker_stopped) = mpsc::channel();
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(1)
        .on_thread_stop(move || {
            let _ = stopped.send(());
        })
        .enable_all()
        .build()
        .unwrap();
    let controller = Arc::new(crate::operation::Controller::new());
    let guard = controller.register(false, None, true).unwrap();
    let (entered, ready) = mpsc::channel();
    let (release, wait) = mpsc::channel();
    let cloned = Arc::clone(&controller);
    let _owner = runtime.spawn(async move {
        cloned
            .run(guard, move || {
                entered.send(()).unwrap();
                let _ = wait.recv_timeout(Duration::from_secs(5));
                Ok(())
            })
            .await
    });
    ready.recv_timeout(Duration::from_secs(5)).unwrap();
    runtime.shutdown_background();
    worker_stopped.recv_timeout(Duration::from_secs(5)).unwrap();
    assert_eq!(controller.counts(), (1, 1));
    let next = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
    next.block_on(async {
        assert!(
            tokio::time::timeout(Duration::ZERO, controller.join())
                .await
                .is_err()
        );
        let _ = release.send(());
        tokio::time::timeout(Duration::from_secs(5), controller.join())
            .await
            .unwrap();
        assert_eq!(controller.counts(), (0, 0));
    });
}

#[test]
fn cooperative_errors_preserve_private_causes_and_reasoned_recovery() {
    if supervised("lifecycle::cooperative_errors_preserve_private_causes_and_reasoned_recovery") {
        return;
    }
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
    let _entered = runtime.enter();
    let control = crate::WorkerStopControl::new().unwrap();
    let context = control.operation_context();
    let error = scope::with_scope(&context, || {
        Err::<(), _>(
            StoreError::new(StoreErrorKind::Unavailable, "provider_failed")
                .with_source(std::io::Error::other("PRIVATE_CAUSE_CANARY"))
                .into(),
        )
    })
    .unwrap_err();
    assert!(!format!("{error:?} {error}").contains("PRIVATE_CAUSE_CANARY"));
    let cause = std::error::Error::source(&error).unwrap();
    let mut deepest = cause;
    while let Some(next) = deepest.source() {
        deepest = next;
    }
    assert_eq!(deepest.to_string(), "PRIVATE_CAUSE_CANARY");
    let error = scope::with_scope(&context, || {
        scope::attempted().unwrap();
        scope::committed();
        control.cancel_now();
        Err::<(), _>(
            StoreError::new(StoreErrorKind::Unavailable, "provider_failed")
                .with_source(std::io::Error::other("CANCELLED_PROVIDER_CAUSE_CANARY"))
                .into(),
        )
    })
    .unwrap_err();
    assert!(
        matches!(&error, CronError::Store(e) if e.kind() == StoreErrorKind::DeadlineExceeded
        && e.transaction_outcome().unwrap().persistence == PersistenceOutcome::Committed
        && e.transaction_outcome().unwrap().completed_after_cutoff)
    );
    assert!(!format!("{error:?} {error}").contains("CANCELLED_PROVIDER_CAUSE_CANARY"));
    let mut cause = std::error::Error::source(&error).unwrap();
    while let Some(next) = cause.source() {
        cause = next;
    }
    assert_eq!(cause.to_string(), "CANCELLED_PROVIDER_CAUSE_CANARY");
    for (reason, word) in [
        ("command_capacity_exhausted", "owned work"),
        ("operation_stopping", "lifecycle"),
        ("operation_context_unsupported", "provider"),
        ("operation_context_nested", "scope"),
    ] {
        let error = crate::consumer_contract::unavailable(reason);
        assert!(error.recovery().contains(word), "{reason}");
        assert!(!error.recovery().contains("database path"));
    }
}

#[tokio::test]
async fn application_completion_refuses_pending_native_work_and_framework_join_bypass() {
    if supervised(
        "lifecycle::application_completion_refuses_pending_native_work_and_framework_join_bypass",
    ) {
        return;
    }
    let app = crate::WorkerStopControl::new().unwrap();
    assert!(app.finish_application_shutdown().is_err());
    let context = app.operation_context();
    let guard = context.state.register(false, None, true).unwrap();
    app.cancel_now();
    assert!(app.finish_application_shutdown().is_err());
    drop(guard);
    app.finish_application_shutdown().unwrap();
    assert_eq!(*app.subscribe_status().borrow(), ShutdownStatus::Joined);
    assert!(context.check().is_err());
    assert!(context.state.register(false, None, true).is_err());
    let early = crate::WorkerStopControl::new().unwrap();
    let early_context = early.operation_context();
    let future = Instant::now() + Duration::from_secs(60);
    early
        .request_stop(ShutdownBudget::new(future, future).unwrap())
        .unwrap();
    early.finish_application_shutdown().unwrap();
    assert!(
        early_context.check().is_err(),
        "completion must close the future-grace scope"
    );
    let scheduler = make_scheduler(1);
    let worker = scheduler.spawn_worker_cooperative(Arc::new(Noop)).unwrap();
    let framework = worker.stop_control();
    framework.cancel_now();
    assert!(framework.finish_application_shutdown().is_err());
    worker.join_owned().await.unwrap();
}

#[tokio::test]
async fn assertion_unwind_releases_and_joins_ignored_cancellation_fixture() {
    if supervised("lifecycle::assertion_unwind_releases_and_joins_ignored_cancellation_fixture") {
        return;
    }
    let scheduler = make_scheduler(1);
    let store = Arc::clone(scheduler.store());
    let (release, holder) = hold(Arc::clone(&store));
    let holder = HeldConnection(Some(release.clone()), Some(holder));
    let (entered, ready) = mpsc::channel();
    let (done, finished) = mpsc::channel();
    let physical = std::thread::spawn(move || {
        entered.send(()).unwrap();
        // This direct legacy call deliberately ignores cooperative cancellation.
        let result = store.list_active();
        done.send(()).unwrap();
        result
    });
    let physical = PhysicalThread::new(physical, release);
    ready.recv_timeout(Duration::from_secs(5)).unwrap();
    let control = crate::WorkerStopControl::new().unwrap();
    control.cancel_now();
    assert!(matches!(
        finished.try_recv(),
        Err(mpsc::TryRecvError::Empty)
    ));
    let failure = std::panic::catch_unwind(std::panic::AssertUnwindSafe(move || {
        let _physical = physical;
        panic!("injected assertion after native readiness");
    }));
    assert!(failure.is_err());
    finished.recv_timeout(Duration::from_secs(5)).unwrap();
    drop(holder);
}

#[cfg(test)]
#[test]
fn queued_closures_keep_permits_and_recheck_scope_before_first_physical_store_call() {
    if supervised(
        "lifecycle::queued_closures_keep_permits_and_recheck_scope_before_first_physical_store_call",
    ) {
        return;
    }
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
            let _ = wait_release.recv_timeout(Duration::from_secs(5));
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
        let _ = release.send(());
        occupied.await.unwrap();
        worker.join_owned().await.unwrap();
        assert_eq!(context.state.counts(), (0, 0));
        assert!(scheduler.store().list_active().unwrap().is_empty());
    });
}
