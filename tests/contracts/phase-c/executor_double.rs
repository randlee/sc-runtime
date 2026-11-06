//! Canonical dev-only projection executor double; copy only into tests/support.
use std::collections::VecDeque;
use std::sync::Mutex;
use std::time::Instant;
use sc_command::Envelope;
use sc_cron_commands::{CronExecutor, CronRequest, CronResponse, ExecutorFuture};

/// Executor recording exact requests/deadlines and returning queued envelopes.
#[derive(Default)]
pub struct RecordingExecutor {
    /// Calls in invocation order.
    pub calls: Mutex<Vec<(CronRequest, Instant)>>,
    /// Explicit queued responses, including typed failures.
    pub replies: Mutex<VecDeque<Envelope<CronResponse>>>,
}
impl CronExecutor for RecordingExecutor {
    fn execute(&self, request: CronRequest, deadline: Instant) -> ExecutorFuture<'_> {
        self.calls.lock().expect("calls").push((request, deadline));
        let reply = self.replies.lock().expect("replies").pop_front().expect("queued executor reply");
        Box::pin(async move { reply })
    }
}
