//! Shared compose-host errors and helpers for [`crate::monty_host`] and
//! [`crate::monty_compose`].

use std::fmt;

/// How `render_template` failed.
///
/// Every variant is recorded as [`crate::RunOutcome::Failed`].
/// After the child exits, the feed records [`crate::RunOutcome::TimedOut`]
/// when the host [`std::time::Instant`] is past the row budget.
/// These variants are not [`crate::CronError`] variants.
#[derive(Debug)]
pub(crate) enum ComposeHostError {
    /// `compose_program` was not found on `PATH` (or the injected path).
    ProgramNotFound {
        /// Program name or path that could not be started.
        program: String,
    },
    /// The temp root could not be created.
    TempRootFailed {
        /// Program name or path that would have been spawned.
        program: String,
        /// Filesystem error from `tempfile`.
        source: std::io::Error,
    },
    /// Writing the template or the context file failed.
    StageFailed {
        /// Program name or path that would have been spawned.
        program: String,
        /// Filesystem error from the temp root.
        source: std::io::Error,
    },
    /// The context value could not be encoded as JSON.
    ContextEncodeFailed {
        /// Program name or path that would have been spawned.
        program: String,
        /// Serializer error.
        source: serde_json::Error,
    },
    /// The program was found but could not be executed.
    ProgramNotRunnable {
        /// Program name or path passed to the process spawn.
        program: String,
        /// Spawn error other than not-found.
        source: std::io::Error,
    },
    /// The child exited zero and stdout was not UTF-8.
    OutputNotUtf8 {
        /// Program name or path that produced the stdout.
        program: String,
    },
    /// The child exited zero and stdout exceeded [`crate::MAX_HOOK_OUTPUT_BYTES`].
    OutputTooLarge {
        /// Program name or path that produced the stdout.
        program: String,
    },
    /// The child ran and exited non-zero.
    NonZeroExit {
        /// Exit status, or `None` when the process was ended by a signal.
        status: Option<i32>,
        /// Capped stderr text from the child.
        stderr: String,
    },
}

impl ComposeHostError {
    /// Stable token stored in the `RunOutcome::Failed` detail.
    #[must_use]
    pub(crate) const fn code(&self) -> &'static str {
        match self {
            Self::ProgramNotFound { .. } => "COMPOSE_NOT_FOUND",
            Self::TempRootFailed { .. } | Self::StageFailed { .. } => "COMPOSE_SPAWN",
            Self::ContextEncodeFailed { .. } => "COMPOSE_CONTEXT",
            Self::ProgramNotRunnable { .. } => "COMPOSE_NOT_RUNNABLE",
            Self::OutputNotUtf8 { .. } | Self::OutputTooLarge { .. } => "COMPOSE_OUTPUT",
            Self::NonZeroExit { .. } => "COMPOSE_EXIT",
        }
    }

    /// Returns what an operator can do about this failure.
    #[must_use]
    pub(crate) const fn recovery(&self) -> &'static str {
        match self {
            Self::ProgramNotFound { .. } => {
                "Install `sc-compose` on PATH or fix the injected program name in tests."
            }
            Self::TempRootFailed { .. } | Self::StageFailed { .. } => {
                "Check permissions and disk space for temp roots under `tempfile::TempDir`."
            }
            Self::ContextEncodeFailed { .. } => {
                "Fix the template context; it could not be encoded as JSON."
            }
            Self::ProgramNotRunnable { .. } => {
                "Check that the compose program is executable and allowed to start."
            }
            Self::OutputNotUtf8 { .. } => "Fix the compose program so its stdout is UTF-8.",
            Self::OutputTooLarge { .. } => {
                "Shrink the rendered output; stdout is capped at the hook output limit."
            }
            Self::NonZeroExit { .. } => {
                "Fix template/context or run `sc-compose render` manually with the logged stderr."
            }
        }
    }
}

impl fmt::Display for ComposeHostError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ProgramNotFound { program } => {
                write!(formatter, "compose program not found: {program}")
            }
            Self::TempRootFailed { program, source } => {
                write!(
                    formatter,
                    "compose program {program} temp root failed: {source}"
                )
            }
            Self::StageFailed { program, source } => {
                write!(
                    formatter,
                    "compose program {program} failed to stage files: {source}"
                )
            }
            Self::ContextEncodeFailed { program, source } => {
                write!(
                    formatter,
                    "compose program {program} context could not be encoded: {source}"
                )
            }
            Self::ProgramNotRunnable { program, source } => {
                write!(
                    formatter,
                    "compose program {program} could not be executed: {source}"
                )
            }
            Self::OutputNotUtf8 { program } => {
                write!(formatter, "compose program {program} stdout was not utf-8")
            }
            Self::OutputTooLarge { program } => {
                write!(
                    formatter,
                    "compose program {program} stdout exceeded the hook output cap"
                )
            }
            Self::NonZeroExit { status, stderr } => match status {
                Some(status) => write!(formatter, "compose exited {status}: {stderr}"),
                None => write!(formatter, "compose exited from a signal: {stderr}"),
            },
        }
    }
}

impl std::error::Error for ComposeHostError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::TempRootFailed { source, .. }
            | Self::StageFailed { source, .. }
            | Self::ProgramNotRunnable { source, .. } => Some(source),
            Self::ContextEncodeFailed { source, .. } => Some(source),
            Self::ProgramNotFound { .. }
            | Self::OutputNotUtf8 { .. }
            | Self::OutputTooLarge { .. }
            | Self::NonZeroExit { .. } => None,
        }
    }
}

/// Caps compose-child stderr the same way hook stderr is capped in messages.
#[must_use]
pub(crate) fn stderr_text(bytes: &[u8]) -> String {
    let text = String::from_utf8_lossy(bytes);
    crate::hook::stderr_in_message(&text).to_owned()
}
