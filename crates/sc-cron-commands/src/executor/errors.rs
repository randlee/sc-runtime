//! Value-free mapping; request/provider prose never enters the public error.
use sc_command::{ErrorKind, OpError};
use sc_cron::{CommitAdmission, CronError, PersistenceOutcome, StoreErrorKind};

pub(super) fn invalid(reason: &'static str, recovery: &'static str) -> OpError {
    OpError::from_code(
        ErrorKind::InvalidInput,
        "CRON.INVALID_INPUT",
        "invalid cron request",
    )
    .with_details(serde_json::json!({"reason": reason}))
    .with_suggestion(recovery)
}

pub(super) fn unsupported() -> OpError {
    OpError::from_code(
        ErrorKind::Unavailable,
        "CRON.CAPABILITY_UNAVAILABLE",
        "target capability unavailable",
    )
    .with_suggestion("select an enabled target or explicitly enable the Monty feature")
}

pub(super) fn internal() -> OpError {
    OpError::from_code(
        ErrorKind::Internal,
        "CRON.INTERNAL",
        "cron operation failed",
    )
    .with_suggestion("inspect application-owned diagnostics; reconcile mutations before replay")
}

pub(super) fn native(error: CronError) -> OpError {
    match error {
        CronError::BadExpression(_) => invalid(
            "invalid_expression",
            "set cron to a valid supported cron expression",
        ),
        CronError::InvalidSpec(_) => invalid(
            "invalid_specification",
            "correct name, timeout, payload, target or environment using the documented schedule limits",
        ),
        CronError::AlreadyActive(_) | CronError::Fenced { .. } => OpError::from_code(
            ErrorKind::Conflict,
            "CRON.CONFLICT",
            "schedule generation conflicts",
        )
        .with_suggestion("read the current schedule generation before issuing a new operation"),
        CronError::NotFound(_) => {
            OpError::from_code(ErrorKind::NotFound, "CRON.NOT_FOUND", "schedule not found")
                .with_suggestion("list schedules and select the intended current row")
        }
        CronError::HookFailed(_) => unsupported(),
        CronError::Store(error) => store(error),
        _ => internal(),
    }
}

fn store(error: sc_cron::StoreError) -> OpError {
    let (code, message) = match error.kind() {
        StoreErrorKind::Cancelled => ("CRON.CANCELLED", "cron operation cancelled"),
        StoreErrorKind::DeadlineExceeded => {
            ("CRON.DEADLINE_EXCEEDED", "cron operation deadline exceeded")
        }
        StoreErrorKind::Indeterminate => (
            "CRON.INDETERMINATE",
            "cron persistence outcome is indeterminate",
        ),
        _ if error.context() == "command_capacity_exhausted" => (
            "CRON.COMMAND_CAPACITY_EXHAUSTED",
            "cron command capacity exhausted",
        ),
        StoreErrorKind::Busy | StoreErrorKind::Unavailable => {
            ("CRON.UNAVAILABLE", "cron operation unavailable")
        }
        _ => ("CRON.INTERNAL", "cron operation failed"),
    };
    let kind = if code == "CRON.INTERNAL" {
        ErrorKind::Internal
    } else {
        ErrorKind::Unavailable
    };
    let mut mapped = OpError::from_code(kind, code, message).with_suggestion(
        "reconcile any admitted mutation through a separately authorized read before replay",
    );
    if let Some(outcome) = error.transaction_outcome() {
        let admission = match outcome.admission {
            CommitAdmission::NotAttempted => "not_attempted",
            CommitAdmission::Vetoed => "vetoed",
            CommitAdmission::Admitted => "admitted",
        };
        let persistence = match outcome.persistence {
            PersistenceOutcome::NoMutationAttempted => "no_mutation_attempted",
            PersistenceOutcome::Committed => "committed",
            PersistenceOutcome::RolledBack => "rolled_back",
            PersistenceOutcome::Indeterminate => "indeterminate",
        };
        mapped = mapped.with_details(serde_json::json!({"admission":admission,
            "persistence":persistence,"completed_after_cutoff":outcome.completed_after_cutoff}));
    }
    if error.context() == "command_capacity_exhausted"
        && error
            .transaction_outcome()
            .is_none_or(|outcome| outcome.persistence == PersistenceOutcome::NoMutationAttempted)
    {
        mapped = mapped.with_suggestion("reduce concurrent commands or wait for owned work to finish before submitting a new request");
    }
    mapped
}
