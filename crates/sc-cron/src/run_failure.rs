//! Parsing [`RunOutcome::Failed`] details that carry stable codes.

use crate::RunOutcome;

/// Machine-readable parts of a failed run detail string.
///
/// sc-cron records Monty, compose, and worker failures as
/// `CODE: message; recovery` (stop checks prefix `stop check: `). Agents and
/// UIs should use [`RunOutcome::failure_parts`] instead of substring checks.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RunFailureParts<'a> {
    /// Stable token such as `COMPOSE_NOT_FOUND`, when present.
    pub code: Option<&'a str>,
    /// Human-readable failure text without the code or recovery tail.
    pub message: &'a str,
    /// Operator recovery hint after the final `; `, when present.
    pub recovery: Option<&'a str>,
}

impl RunFailureParts<'_> {
    /// Parses one failed-run detail string.
    #[must_use]
    pub fn parse<'a>(detail: &'a str) -> RunFailureParts<'a> {
        let detail = detail.strip_prefix("stop check: ").unwrap_or(detail);
        let Some((head, recovery)) = detail.rsplit_once("; ") else {
            return parse_without_recovery(detail);
        };
        let Some((code, message)) = head.split_once(": ") else {
            return RunFailureParts {
                code: None,
                message: detail,
                recovery: None,
            };
        };
        if is_failure_code(code) {
            RunFailureParts {
                code: Some(code),
                message,
                recovery: Some(recovery),
            }
        } else {
            RunFailureParts {
                code: None,
                message: detail,
                recovery: None,
            }
        }
    }
}

fn parse_without_recovery<'a>(detail: &'a str) -> RunFailureParts<'a> {
    let Some((code, message)) = detail.split_once(": ") else {
        return RunFailureParts {
            code: None,
            message: detail,
            recovery: None,
        };
    };
    if is_failure_code(code) {
        RunFailureParts {
            code: Some(code),
            message,
            recovery: None,
        }
    } else {
        RunFailureParts {
            code: None,
            message: detail,
            recovery: None,
        }
    }
}

fn is_failure_code(code: &str) -> bool {
    !code.is_empty()
        && code
            .chars()
            .all(|ch| ch.is_ascii_uppercase() || ch.is_ascii_digit() || ch == '_')
}

impl RunOutcome {
    /// When this outcome is [`RunOutcome::Failed`], splits code, message, and recovery.
    #[must_use]
    pub fn failure_parts(&self) -> Option<RunFailureParts<'_>> {
        match self {
            Self::Failed(detail) => Some(RunFailureParts::parse(detail)),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_compose_failure_with_recovery() {
        let outcome = RunOutcome::Failed(
            "COMPOSE_NOT_FOUND: tick: compose program not found: sc-compose; Install `sc-compose` on PATH.".into(),
        );
        let parts = outcome.failure_parts().expect("failed");
        assert_eq!(parts.code, Some("COMPOSE_NOT_FOUND"));
        assert!(parts.message.contains("compose program not found"));
        assert!(parts.recovery.is_some());
    }

    #[test]
    fn parses_stop_check_host_reply() {
        let outcome = RunOutcome::Failed(
            "stop check: HOST_REPLY: reply is not JSON: boom; Fix hook source.".into(),
        );
        let parts = outcome.failure_parts().expect("failed");
        assert_eq!(parts.code, Some("HOST_REPLY"));
        assert!(parts.message.contains("reply is not JSON"));
    }

    #[test]
    fn unstructured_failures_still_parse() {
        let outcome = RunOutcome::Failed("job panicked: boom".into());
        let parts = outcome.failure_parts().expect("failed");
        assert_eq!(parts.code, None);
        assert_eq!(parts.message, "job panicked: boom");
    }
}
