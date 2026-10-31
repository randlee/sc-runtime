//! Typed failures shared by the blocking CLI driver and Monty dispatch.

use std::{fmt, io};

/// Adapter failures never borrow diagnostic codes from application errors.
#[derive(Debug)]
pub(crate) enum ComposeHostError {
    Spawn { program: String, source: io::Error },
    Output { program: String, source: io::Error },
    TimedOut,
}

impl ComposeHostError {
    pub(crate) const fn code(&self) -> &'static str {
        match self {
            Self::Spawn { .. } => "HOST_SPAWN",
            Self::Output { .. } => "HOST_OUTPUT",
            Self::TimedOut => "HOST_TIMEOUT",
        }
    }
    pub(crate) const fn recovery(&self) -> &'static str {
        match self {
            Self::Spawn { .. } => "Check the configured executable and working directory.",
            Self::Output { .. } => "Return valid UTF-8 output within each stream's output cap.",
            Self::TimedOut => "Finish the CLI command within the schedule deadline.",
        }
    }
}
impl fmt::Display for ComposeHostError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Spawn { program, source } => write!(f, "cannot spawn {program}: {source}"),
            Self::Output { program, source } => write!(f, "cannot capture {program}: {source}"),
            Self::TimedOut => f.write_str("CLI row deadline expired"),
        }
    }
}
impl std::error::Error for ComposeHostError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Spawn { source, .. } | Self::Output { source, .. } => Some(source),
            Self::TimedOut => None,
        }
    }
}
