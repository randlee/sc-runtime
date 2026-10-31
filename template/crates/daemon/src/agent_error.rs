//! Stable agent-facing process failures and shutdown result precedence.

/// Formats a secret-free cause with its stable code and recovery.
#[must_use]
pub fn format_agent_error(code: &str, cause: impl std::fmt::Display, recovery: &str) -> String {
    format!("{code} {cause} — {recovery}")
}

/// Preserves an operation failure and reports a logger shutdown failure second.
///
/// The caller has already drained and dropped runtime and scheduler consumers.
/// `shutdown` is the result of consuming the observability owner.
///
/// # Errors
/// Returns the operation failure, or shutdown failure if the operation succeeded.
pub fn finish(
    primary: Result<(), String>,
    shutdown: Result<(), sc_observability::v2::ShutdownError>,
) -> Result<(), String> {
    let shutdown = shutdown.map_err(|cause| {
        format_agent_error(
            "DAEMON.OBSERVABILITY_SHUTDOWN",
            cause,
            "inspect sink health and retry shutdown",
        )
    });
    match (primary, shutdown) {
        (Err(primary), Err(secondary)) => {
            eprintln!("{secondary}");
            Err(primary)
        }
        (Err(primary), Ok(())) => Err(primary),
        (Ok(()), shutdown) => shutdown,
    }
}

#[cfg(test)]
mod tests {
    use super::{finish, format_agent_error};
    use sc_observability_types::{ErrorCode, ErrorContext, Remediation, v2::ShutdownError};

    fn shutdown_failure() -> ShutdownError {
        ShutdownError::Drain {
            context: Box::new(ErrorContext::new(
                ErrorCode::new_static("TEST.DRAIN"),
                "sink drain failed",
                Remediation::not_recoverable("inspect sink"),
            )),
        }
    }

    #[test]
    fn operation_remains_primary_and_shutdown_only_fails_success() {
        let primary = format_agent_error(
            "RUNTIME.BIND_FAILED",
            "endpoint occupied",
            "read the cause and retry",
        );
        assert_eq!(finish(Err(primary.clone()), Ok(())), Err(primary.clone()));
        assert_eq!(
            finish(Err(primary.clone()), Err(shutdown_failure())),
            Err(primary)
        );
        let error =
            finish(Ok(()), Err(shutdown_failure())).expect_err("shutdown must fail process");
        assert!(error.starts_with("DAEMON.OBSERVABILITY_SHUTDOWN "));
        assert!(error.contains("sink drain failed"));
        assert!(error.ends_with(" — inspect sink health and retry shutdown"));
        assert_eq!(finish(Ok(()), Ok(())), Ok(()));
    }
}
