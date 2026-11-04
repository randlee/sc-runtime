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
