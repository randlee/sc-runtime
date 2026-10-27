//! Reference test: Python hook renders via `sc_compose.render_template`.
//! Skips when `import sc_compose` is unavailable.

#![cfg(unix)]
#![expect(clippy::expect_used, reason = "integration test")]

mod common;

use std::path::PathBuf;
use std::process::Command;
use std::time::Duration;

use common::{Fixture, idle_job, shutdown};
use sc_cron::RunOutcome;
use serde_json::json;

fn sc_compose_available() -> bool {
    Command::new("python3")
        .args(["-c", "import sc_compose"])
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

fn fixture_module() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/sc_compose_tick.py")
}

#[tokio::test]
async fn python_hook_renders_with_sc_compose() {
    if !sc_compose_available() {
        eprintln!("skip: sc_compose not installed");
        return;
    }

    let module = fixture_module();
    let fixture = Fixture::new();
    let spec = sc_cron::ScheduleSpec::with_hook(
        "compose",
        "0 * * * * *",
        Duration::from_secs(30),
        json!({
            "template": "Hello {{ name }}",
            "context": {"name": "cron"},
        }),
        sc_cron::HookTarget::python(&module).expect("module path"),
    )
    .expect("spec");
    let schedule = fixture.start(spec);
    let worker = fixture.worker(idle_job()).await;
    fixture.step(3600).await;
    fixture.store.wait_recorded(&schedule, RunOutcome::Ok).await;
    shutdown(worker).await;
}
