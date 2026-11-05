//! Generic cron projections with an application-owned executor and deadline.
use std::time::Instant;
use sc_command::{Envelope, ErrorKind, OpError};
use sc_cron_commands::{CronExecutor, CronOperation, CronRequest, CronResponse};

/// Executes the same canonical request once, preserving the caller's deadline.
/// A mismatched UI verb is rejected before the application executor is called.
pub async fn execute(executor: &dyn CronExecutor, verb: &str, request: CronRequest,
                     deadline: Instant) -> Envelope<CronResponse> {
    let actual = match &request.operation {
        CronOperation::Start(_) => "cron.start",
        CronOperation::End(_) => "cron.end",
        CronOperation::List(_) => "cron.list",
    };
    if actual != verb { return invalid_request(); }
    executor.execute(request, deadline).await
}

/// Value-free canonical error for an invalid versioned request or UI verb.
pub fn invalid_request() -> Envelope<CronResponse> {
    Envelope::failure(OpError::from_code(ErrorKind::InvalidInput,
        "CRON.COMMAND.INVALID_INPUT", "invalid versioned cron request or command verb")
        .with_suggestion("send one version-1 request matching the selected cron command"))
}
