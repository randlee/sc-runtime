//! Application cleanup retains singleton and successful store ownership.

use super::*;

const WAIT: Duration = Duration::from_secs(5);

async fn cleanup_blocks_restart(startup_error: bool) {
    let dir = tempfile::tempdir().expect("root");
    let config = rooted_config(dir.path());
    let owner_config = config.clone();
    let (ready, serving) = oneshot::channel();
    let (stop, stopping) = oneshot::channel();
    let (worker_tx, worker_rx) = oneshot::channel();
    let (draining, drained) = oneshot::channel();
    let (release, released) = oneshot::channel();
    let store = Arc::new(());
    let store_lifetime = Arc::downgrade(&store);
    let task = tokio::spawn(async move {
        Daemon::builder(&app(), &owner_config)
            .stores(move || async move {
                let worker = tokio::spawn(async move {
                    let _ = released.await;
                });
                worker_tx.send(worker).expect("worker owner");
                if startup_error {
                    Err(RuntimeError::stores_failed(
                        "STORE.OPEN",
                        "failed",
                        "retry",
                        None,
                    ))
                } else {
                    Ok(store)
                }
            })
            .routes(|_| OpenApiRouter::new())
            .with_control(RunControl::detached(Some(ready), Some(stopping)))
            .run_with_cleanup(move || async move {
                if !startup_error {
                    assert!(
                        store_lifetime.upgrade().is_some(),
                        "stores survive through cleanup"
                    );
                }
                let worker = worker_rx.await.expect("started worker");
                draining.send(()).expect("drain notice");
                worker.await.expect("worker joined");
            })
            .await
    });
    if !startup_error {
        tokio::time::timeout(WAIT, serving)
            .await
            .expect("ready bound")
            .expect("serving");
        stop.send(()).expect("stop");
    }
    tokio::time::timeout(WAIT, drained)
        .await
        .expect("drain bound")
        .expect("draining");
    let opened = Arc::new(AtomicBool::new(false));
    let opened_second = Arc::clone(&opened);
    let second = Daemon::builder(&app(), &config)
        .stores(move || async move {
            opened_second.store(true, Ordering::SeqCst);
            Ok(())
        })
        .routes(|()| OpenApiRouter::new())
        .with_control(RunControl::detached(None, None))
        .run();
    let result = tokio::time::timeout(WAIT, second)
        .await
        .expect("contender bound");
    assert!(matches!(result, Err(RuntimeError::LockHeld { .. })));
    assert!(
        !opened.load(Ordering::SeqCst),
        "replacement cannot start its worker while old worker drains"
    );
    release.send(()).expect("release drain");
    let result = tokio::time::timeout(WAIT, task)
        .await
        .expect("join bound")
        .expect("join");
    assert_eq!(result.is_err(), startup_error);
    let (_stop, stopping) = oneshot::channel();
    drop(_stop);
    Daemon::builder(&app(), &config)
        .stores(|| async { Ok(()) })
        .routes(|()| OpenApiRouter::new())
        .with_control(RunControl::detached(None, Some(stopping)))
        .run()
        .await
        .expect("replacement starts after old worker join");
}

#[tokio::test]
async fn cleanup_holds_instance_and_stores_until_worker_joins() {
    cleanup_blocks_restart(false).await;
}

#[tokio::test]
async fn cleanup_holds_instance_after_worker_startup_error() {
    cleanup_blocks_restart(true).await;
}

#[tokio::test]
async fn early_error_drops_unused_startup_sender_before_cleanup() {
    let dir = tempfile::tempdir().expect("root");
    let mut config = rooted_config(dir.path());
    config.endpoint_flag = Some("not-an-endpoint".to_owned());
    let (sender, receiver) = oneshot::channel::<()>();
    let run = Daemon::builder(&app(), &config)
        .stores(move || async move {
            let _ = sender.send(());
            Ok(())
        })
        .routes(|()| OpenApiRouter::new())
        .with_control(RunControl::detached(None, None))
        .run_with_cleanup(move || async move {
            assert!(
                receiver.await.is_err(),
                "unused startup closure must drop first"
            );
        });
    assert!(
        tokio::time::timeout(WAIT, run)
            .await
            .expect("cleanup cannot deadlock")
            .is_err()
    );
}
