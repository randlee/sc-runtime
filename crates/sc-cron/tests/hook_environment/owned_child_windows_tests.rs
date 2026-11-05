//! Native tests share the actual Rust bridge entry and hold identity handles.
use super::*;
use std::cell::{Cell, RefCell};

const WITNESS: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/hook_environment/owned_child_witness.py"
);
fn wait(path: &Path) {
    let deadline = Instant::now() + Duration::from_secs(5);
    while !path.exists() {
        assert!(
            Instant::now() < deadline,
            "missing readiness: {}",
            path.display()
        );
        std::thread::sleep(POLL); // allow: bounded test-only causal readiness poll
    }
}
fn states(root: &Path, phase: &str) -> Vec<bool> {
    std::fs::write(root.join(phase), b"observe").expect("release observer");
    let result = root.join(format!("{phase}-result"));
    wait(&result);
    serde_json::from_slice(&std::fs::read(result).expect("read observation")).expect("states")
}

fn witness(mode: &str) {
    let root = tempfile::tempdir().expect("witness root");
    let bridge = root.path().join("bridge.py");
    let directory = Path::new(WITNESS).parent().expect("fixture directory");
    std::fs::write(&bridge, format!(
        "import sys\nfrom pathlib import Path\nsys.path.insert(0,{directory:?})\nfrom owned_child_witness import bridge\nsys.exit(bridge({mode:?},Path({root:?}),sys.argv[1]))\n",
        directory = directory.to_string_lossy(), root = root.path().to_string_lossy(),
    )).expect("bridge fixture");
    let observer: RefCell<Option<Bridge>> = RefCell::new(None);
    let before = RefCell::new(Vec::new());
    let released = Cell::new(false);
    let controller = || {
        if root.path().join("pids").exists() && !released.get() {
            let mut command = Command::new(super::super::super::python_program());
            command
                .args([WITNESS, "observe"])
                .arg(root.path())
                .arg(mode)
                .stdin(Stdio::null())
                .stdout(Stdio::null())
                .stderr(Stdio::inherit());
            *observer.borrow_mut() = Some(Bridge::spawn(command).expect("observer"));
            wait(&root.path().join("observer-ready"));
            // All three process handles exist before release or expiry.
            std::fs::write(root.path().join("release"), b"release").expect("release target");
            if mode == "timeout" {
                std::fs::write(root.path().join("expire"), b"expire").expect("expire ready target");
            }
            released.set(true);
        }
        mode == "stall" && released.get()
    };
    let mut command = Command::new(super::super::super::python_program());
    command
        .args([WITNESS, "target"])
        .arg(root.path())
        .arg(if mode == "timeout" {
            "timeout"
        } else {
            "success"
        });
    let result = output(
        &mut command,
        super::super::super::common::WAIT,
        &bridge,
        controller,
        || {
            assert!(released.get(), "completion preceded causal readiness");
            *before.borrow_mut() = states(root.path(), "before");
        },
    );
    // The same retained handles are checked again after emergency cleanup.
    let after = states(root.path(), "after");
    observer
        .borrow_mut()
        .as_mut()
        .expect("observer started")
        .stop()
        .expect("observer cleanup");
    assert!(
        after.iter().all(|state| *state),
        "emergency cleanup leaked a fixture process: {after:?}"
    );
    if mode == "omit" {
        assert!(
            before.borrow().iter().any(|state| !state),
            "cleanup omission was masked"
        );
        assert!(result.status.success());
    } else {
        assert!(
            before.borrow().iter().all(|state| *state),
            "inner cleanup leaked before harness release: {:?}",
            before.borrow()
        );
        assert_eq!(
            result.status.success(),
            mode == "success",
            "{}",
            String::from_utf8_lossy(&result.stderr)
        );
        if mode == "timeout" || mode == "stall" {
            assert_eq!(result.status.code(), Some(124));
        }
    }
}

#[test]
fn normal_bridge_cleans_descendants_before_emergency_release() {
    witness("success");
}
#[test]
fn ready_target_expiry_cleans_descendants_before_emergency_release() {
    witness("timeout");
}
#[test]
fn cleanup_omission_is_detected_before_emergency_release() {
    witness("omit");
}
#[test]
fn stalled_bridge_is_expired_and_physically_reaped_through_rust_entry() {
    witness("stall");
}

#[test]
fn actual_stop_requires_empty_membership_and_rejects_omitted_proof() {
    let spawn = || {
        let mut command = Command::new(super::super::super::python_program());
        command
            .args(["-c", "import time; time.sleep(30)"])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null());
        Bridge::spawn(command).expect("owned completion witness")
    };
    let mut owned = spawn();
    // Establish leader exit independently before injecting nonempty membership.
    // Ignoring that membership must fail even on an already-reaped leader;
    // startup/termination scheduling cannot supply an accidental second poll.
    owned.child.start_kill().expect("terminate query witness");
    let deadline = Instant::now() + CLEANUP;
    while owned
        .child
        .inner_mut()
        .try_wait()
        .expect("leader readiness")
        .is_none()
    {
        assert!(Instant::now() < deadline, "leader readiness deadline");
        std::thread::sleep(POLL); // allow: bounded test-only leader-exit handshake
    }
    let mut queries = 0;
    owned
        .stop_with(|job| {
            queries += 1;
            let actual = job.query_process_id_list().map_err(std::io::Error::other)?;
            // A nonempty observation cannot establish completion even when the
            // leader is already gone. Subsequent observations use the real job.
            Ok(if queries == 1 {
                vec![usize::MAX]
            } else {
                actual
            })
        })
        .expect("wait for genuine empty membership");
    assert!(
        queries >= 2,
        "leader-only return omitted the completion wait"
    );
    assert!(
        owned
            .job
            .query_process_id_list()
            .expect("actual empty job")
            .is_empty()
    );

    let mut owned = spawn();
    let proof =
        owned.stop_with(|_| Err(std::io::Error::other("injected omitted completion proof")));
    assert!(
        proof.is_err(),
        "owner returned success without physical completion proof"
    );
    // The emergency guard remains owned after rejection and uses the same
    // absolute cleanup deadline to physically reclaim every member.
    owned
        .stop()
        .expect("emergency completion after rejected proof");
    assert!(
        owned
            .job
            .query_process_id_list()
            .expect("reclaimed job")
            .is_empty()
    );
}

#[test]
fn empty_accounting_does_not_bypass_unsignaled_retained_identity() {
    let mut command = Command::new(super::super::super::python_program());
    command
        .args(["-c", "import time; time.sleep(30)"])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    let mut owned = Bridge::spawn(command).expect("owned signal witness");
    owned.retain_members().expect("capture before termination");
    assert!(
        !owned.members.is_empty(),
        "witness had no retained identity"
    );
    owned.child.start_kill().expect("terminate witness");
    let deadline = Instant::now() + CLEANUP;
    while owned
        .child
        .inner_mut()
        .try_wait()
        .expect("leader exit")
        .is_none()
    {
        assert!(Instant::now() < deadline, "leader failed to exit");
        std::thread::sleep(POLL); // allow: bounded test-only leader-exit handshake
    }
    assert!(
        owned
            .job
            .query_process_id_list()
            .expect("real empty job")
            .is_empty()
    );
    let mut probes = 0;
    owned
        .stop_with_signals(
            |job| job.query_process_id_list().map_err(std::io::Error::other),
            |handles| {
                probes += 1;
                // Force precisely the failing order: accounting already empty and
                // leader reaped, but one retained identity has not signaled yet.
                if probes == 1 {
                    Ok(false)
                } else {
                    Bridge::signaled(handles)
                }
            },
        )
        .expect("physical identity completion");
    assert!(
        probes >= 2,
        "empty accounting bypassed physical signal proof"
    );
    assert!(Bridge::signaled(&mut owned.members).expect("actual retained signals"));
}

#[test]
fn recycled_identifier_retains_a_new_live_identity() {
    let spawn = || {
        let mut command = Command::new(super::super::super::python_program());
        command
            .args(["-c", "import time; time.sleep(30)"])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null());
        Bridge::spawn(command).expect("identity generation")
    };
    let mut old = spawn();
    old.retain_members().expect("first identity");
    let old_pid = old.child.id() as usize;
    old.stop().expect("first generation complete");
    let finished = old
        .members
        .remove(&old_pid)
        .expect("retained first identity");
    let mut current = spawn();
    let pid = current.child.id() as usize;
    // Deterministically present an earlier completed generation under the live
    // PID. No actual OS PID recycling or timing lottery is needed to test it.
    current.members.insert(pid, finished);
    current.retain_members().expect("retain current generation");
    assert!(
        current.members[&pid].len() >= 2,
        "finished identity suppressed a live generation"
    );
    assert!(
        current
            .members
            .get_mut(&pid)
            .expect("generations")
            .last_mut()
            .expect("live identity")
            .wait_timeout(Duration::ZERO)
            .expect("live check")
            .is_none()
    );
    current.stop().expect("all generations signaled");
    assert!(Bridge::signaled(&mut current.members).expect("actual completion"));
}
