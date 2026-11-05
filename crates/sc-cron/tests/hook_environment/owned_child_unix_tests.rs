//! Real cross-group descendants acknowledge readiness before release or expiry.
use super::*;
use std::cell::Cell;

fn witness(mode: &str) {
    let root = tempfile::tempdir().expect("session witness");
    let path = root.path().join("target.py");
    std::fs::write(
        &path,
        r#"import json, os, pathlib, subprocess, sys, time
root=pathlib.Path(sys.argv[1])
mode=sys.argv[2]
if mode == 'grandchild':
    (root/'grandchild-ready').write_text(str(os.getpid()))
    while True: time.sleep(.01)
if mode == 'child':
    grandchild=subprocess.Popen([sys.executable,__file__,str(root),'grandchild'],process_group=0)
    while not (root/'grandchild-ready').exists(): time.sleep(.01)
    temporary=root/'child-ready.tmp'
    temporary.write_text(json.dumps([os.getpid(),grandchild.pid]))
    temporary.replace(root/'child-ready')
    while True: time.sleep(.01)
child=subprocess.Popen([sys.executable,__file__,str(root),'child'],process_group=0)
while not (root/'child-ready').exists(): time.sleep(.01)
pids=[os.getpid(),*json.loads((root/'child-ready').read_text())]
temporary=root/'ready.tmp'
temporary.write_text(json.dumps(pids))
temporary.replace(root/'pids')
while not (root/'release').exists(): time.sleep(.01)
sys.exit(7 if mode == 'failure' else 0)
"#,
    )
    .expect("target script");
    let ready = Cell::new(false);
    let mut command = Command::new(super::super::super::python_program());
    command.arg(&path).arg(root.path()).arg(mode);
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        output(&mut command, super::super::super::common::WAIT, || {
            if root.path().join("pids").exists() && !ready.get() {
                ready.set(true);
                if mode == "panic" {
                    panic!("injected caller assertion after readiness");
                }
                if mode != "expiry" {
                    std::fs::write(root.path().join("release"), b"release")
                        .expect("release ready target");
                }
            }
            mode == "expiry" && ready.get()
        })
    }));
    assert!(ready.get(), "caller returned before causal readiness");
    if mode == "panic" {
        assert!(result.is_err());
    } else {
        let result = result.expect("completed caller");
        assert_eq!(
            result.status.code(),
            Some(match mode {
                "failure" => 7,
                "expiry" => 124,
                _ => 0,
            }),
            "{}",
            String::from_utf8_lossy(&result.stderr)
        );
    }
    assert_reclaimed(root.path());
}

fn assert_reclaimed(root: &std::path::Path) {
    let pids: Vec<u32> =
        serde_json::from_slice(&std::fs::read(root.join("pids")).expect("pids")).expect("pid JSON");
    // A regular file and nonblocking bounded wait prevent an observation pipe
    // from turning a failed cleanup into another hanging test.
    let mut captured = tempfile::tempfile().expect("inspection output");
    let mut inspect = Command::new("ps")
        .args(["-axo", "pid=,stat="])
        .stdout(Stdio::from(captured.try_clone().expect("inspection clone")))
        .stderr(Stdio::null())
        .stdin(Stdio::null())
        .spawn()
        .expect("inspect processes");
    assert!(
        bounded_wait(&mut inspect, Instant::now() + CLEANUP)
            .expect("bounded process inspection")
            .success()
    );
    let bytes = read(&mut captured);
    let listing = String::from_utf8_lossy(&bytes);
    for line in listing.lines() {
        let mut fields = line.split_whitespace();
        let Some(pid) = fields.next().and_then(|value| value.parse::<u32>().ok()) else {
            continue;
        };
        if pids.contains(&pid) {
            assert!(
                fields.next().is_some_and(|state| state.starts_with('Z')),
                "live descendant survived physical cleanup: {line}"
            );
        }
    }
}

#[test]
fn normal_completion_reclaims_ready_descendants_in_distinct_groups() {
    witness("success");
}
#[test]
fn failed_completion_reclaims_ready_descendants_in_distinct_groups() {
    witness("failure");
}
#[test]
fn causal_expiry_reclaims_ready_descendants_in_distinct_groups() {
    witness("expiry");
}

#[test]
fn caller_panic_reclaims_ready_descendants_in_distinct_groups() {
    witness("panic");
}

#[test]
fn expired_inspector_is_physically_reaped_before_error_returns() {
    let root = tempfile::tempdir().expect("inspector witness");
    let ready = root.path().join("ready");
    let mut child = Command::new(super::super::super::python_program())
        .args([
            "-c",
            "import pathlib,sys,time; pathlib.Path(sys.argv[1]).touch(); time.sleep(30)",
        ])
        .arg(&ready)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("inspector");
    let deadline = Instant::now() + CLEANUP;
    let result = bounded_wait_with(&mut child, deadline, || ready.exists());
    assert!(ready.exists(), "expiry preceded readiness");
    assert!(result.is_err(), "stalled inspector reported success");
    assert!(
        child.try_wait().expect("inspect actual exit").is_some(),
        "deadline discarded an unreaped inspector"
    );
}

#[test]
fn missing_session_inspection_refuses_before_actual_command_launch() {
    let root = tempfile::tempdir().expect("preflight witness");
    let launched = root.path().join("launched");
    let mut command = Command::new("/bin/sh");
    command
        .args(["-c", "printf launched > \"$1\"", "inspection-witness"])
        .arg(&launched)
        .env("PATH", "");
    let result = output(&mut command, super::super::super::common::WAIT, || false);
    assert!(
        !result.status.success(),
        "preflight failure must refuse dispatch"
    );
    assert!(!launched.exists(), "uninspectable command was launched");
    assert!(String::from_utf8_lossy(&result.stderr).contains("session inspection unavailable"));
}
