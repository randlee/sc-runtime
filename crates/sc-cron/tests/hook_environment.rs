//! Shared policy, store, resolver, Python/Monty/CLI parity and redaction canaries.
#![expect(clippy::expect_used, reason = "integration assertions")]
mod common;
#[path = "hook_environment/model.rs"]
mod model;
#[cfg(feature = "monty")]
#[path = "hook_environment/monty.rs"]
mod monty;
#[path = "hook_environment/owned_child.rs"]
mod owned_child;
#[path = "hook_environment/resolution.rs"]
mod resolution;

use sc_cron::*;
use serde_json::json;
use std::sync::Arc;
use std::time::{Duration, Instant};

const BUDGET: Duration = Duration::from_secs(20);
fn python_program() -> &'static str {
    if cfg!(windows) { "python" } else { "python3" }
}

fn spec(name: &str, hook: HookTarget, environment: HookEnvironment) -> ScheduleSpec {
    ScheduleSpec::with_hook(name, "* * * * *", BUDGET, json!({}), hook)
        .expect("spec")
        .with_environment(environment)
        .expect("environment")
}
async fn due(fixture: &common::Fixture) -> RunOutcome {
    fixture.clock.advance(chrono::Duration::seconds(60));
    let runs = tokio::time::timeout(common::WAIT, fixture.scheduler.run_due(common::idle_job()))
        .await
        .expect("bounded due")
        .expect("due");
    assert_eq!(runs.len(), 1);
    runs[0].outcome.clone()
}
fn assert_code(outcome: &RunOutcome, code: &str) {
    assert_eq!(
        outcome.failure_parts().and_then(|p| p.code),
        Some(code),
        "{outcome:?}"
    );
}
fn python_file(root: &std::path::Path, source: &str) -> std::path::PathBuf {
    let path = root.join("hook.py");
    std::fs::write(&path, source).expect("hook file");
    path
}

#[derive(Default)]
struct RecordingResolver {
    calls: std::sync::Mutex<Vec<(String, String, String, Instant)>>,
}
#[async_trait::async_trait]
impl EnvironmentReferenceResolver for RecordingResolver {
    async fn resolve(
        &self,
        request: EnvironmentReferenceRequest<'_>,
    ) -> Result<Option<String>, EnvironmentReferenceError> {
        let debug = format!("{request:?}");
        assert!(!debug.contains(request.reference()));
        self.calls.lock().expect("calls").push((
            request.schedule().as_str().to_owned(),
            request.variable().as_str().to_owned(),
            request.reference().to_owned(),
            request.deadline(),
        ));
        match request.reference() {
            "missing" => Ok(None),
            "denied" => Err(EnvironmentReferenceError::Denied),
            "unavailable" => Err(EnvironmentReferenceError::Unavailable),
            "pending" => std::future::pending().await,
            "at-limit" => Ok(Some("x".repeat(MAX_ENV_VALUE_BYTES))),
            "oversize" => Ok(Some("r".repeat(MAX_ENV_VALUE_BYTES + 1))),
            other => Ok(Some(
                other.strip_prefix("value:").unwrap_or(other).to_owned(),
            )),
        }
    }
}
