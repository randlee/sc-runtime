use std::error::Error;
use std::fmt;
use std::sync::Arc;

const CODE_OPEN: &str = "DBSQ.OPEN";
const CODE_CHANNEL_CLOSED: &str = "DBSQ.CHANNEL_CLOSED";
const CODE_QUEUE_FULL: &str = "DBSQ.QUEUE_FULL";
const CODE_DEADLINE_EXCEEDED: &str = "DBSQ.DEADLINE_EXCEEDED";
const CODE_SQLITE: &str = "DBSQ.SQLITE";
const CODE_COMMIT: &str = "DBSQ.COMMIT";

const RECOVERY_OPEN: &str =
    "Create the parent directory if it is missing, then open the database again.";
const RECOVERY_CHANNEL_CLOSED: &str =
    "Reopen the database with SqliteWriter::open. The writer thread has exited.";
const RECOVERY_QUEUE_FULL: &str = "Retry the call. The job was not accepted.";
const RECOVERY_DEADLINE_EXCEEDED: &str = "Do not retry a non-idempotent write until you check the database. The job was accepted and may already be committed.";
const RECOVERY_SQLITE: &str =
    "Read the SQLite code and the source error. This crate does not retry the statement.";
const RECOVERY_COMMIT: &str =
    "Read the SQLite code and the source error. The commit did not succeed.";

/// Failure from opening the database or from a writer call.
///
/// [`Self::code`] is the stable code. [`Self::recovery`] is the next step.
/// Display text is `{code}: {symptom}; {recovery}`.
///
/// `QueueFull` means the job was not accepted and is safe to retry.
/// `DeadlineExceeded` means the job was accepted and the outcome is unknown:
/// the write may still commit, so a non-idempotent write must not be retried
/// until the caller checks the database. `ChannelClosed` means the writer
/// thread is gone; reopen with [`crate::SqliteWriter::open`].
///
/// `Error::source` on `Sqlite` and `Commit` is the underlying
/// [`rusqlite::Error`]. That type is part of this crate's public contract, so
/// the rusqlite 0.39 pin is the semver boundary. `Open` carries that same
/// rusqlite error, or the `std::io::Error` from starting the thread, when one
/// exists.
#[derive(Debug)]
#[non_exhaustive]
pub enum SqliteWriterError {
    /// The database file could not be opened or the writer thread did not start.
    ///
    /// Open did not accept a write. Fix the path and open again.
    Open {
        /// SQLite or thread-start message.
        message: String,
        /// Underlying rusqlite or I/O failure.
        ///
        /// Absent when the writer thread stops before open reports a result.
        source: Option<Box<dyn Error + Send + Sync>>,
    },

    /// The writer thread has exited, or the reply channel closed before a reply.
    ///
    /// The handle cannot be reused. Reopen the database. This variant is not
    /// a failed downcast and not a reply-shape mismatch.
    ChannelClosed,

    /// The bounded queue could not accept the job.
    ///
    /// The statement did not run. Retrying the call is safe.
    QueueFull,

    /// The job was accepted and no reply arrived within [`crate::WRITE_DEADLINE`].
    ///
    /// The outcome is unknown. The writer may still commit the statement after
    /// this error is returned. Do not retry a non-idempotent write until the
    /// database has been checked.
    DeadlineExceeded,

    /// rusqlite reported a statement or transaction failure.
    ///
    /// `source` is the original [`rusqlite::Error`], not a reconstructed
    /// `SqliteFailure`.
    Sqlite {
        /// SQLite extended result code.
        sqlite_code: i32,
        /// SQLite message without the numeric code.
        message: String,
        /// Underlying rusqlite error.
        ///
        /// Shared across every caller in a failed batch. `rusqlite::Error` is
        /// not `Clone` on rusqlite 0.39, and this crate does not build a new
        /// `SqliteFailure`.
        source: Arc<rusqlite::Error>,
    },

    /// The outer commit or a savepoint commit failed.
    ///
    /// The commit did not succeed. `source` is the original rusqlite error and
    /// `sqlite_code` is its extended result code.
    Commit {
        /// SQLite extended result code.
        sqlite_code: i32,
        /// Commit failure message.
        message: String,
        /// Underlying rusqlite error, shared with every caller of the commit.
        source: Arc<rusqlite::Error>,
    },
}

impl SqliteWriterError {
    /// Returns the stable code for this failure.
    #[must_use]
    pub fn code(&self) -> &'static str {
        match self {
            Self::Open { .. } => CODE_OPEN,
            Self::ChannelClosed => CODE_CHANNEL_CLOSED,
            Self::QueueFull => CODE_QUEUE_FULL,
            Self::DeadlineExceeded => CODE_DEADLINE_EXCEEDED,
            Self::Sqlite { .. } => CODE_SQLITE,
            Self::Commit { .. } => CODE_COMMIT,
        }
    }

    /// Returns the recovery step for this failure.
    ///
    /// `QueueFull` is retryable. `DeadlineExceeded` is not a signal to retry
    /// a non-idempotent write. `ChannelClosed` means reopen.
    #[must_use]
    pub fn recovery(&self) -> &'static str {
        match self {
            Self::Open { .. } => RECOVERY_OPEN,
            Self::ChannelClosed => RECOVERY_CHANNEL_CLOSED,
            Self::QueueFull => RECOVERY_QUEUE_FULL,
            Self::DeadlineExceeded => RECOVERY_DEADLINE_EXCEEDED,
            Self::Sqlite { .. } => RECOVERY_SQLITE,
            Self::Commit { .. } => RECOVERY_COMMIT,
        }
    }

    pub(crate) fn sqlite(error: rusqlite::Error) -> Self {
        let sqlite_code = sqlite_code(&error);
        let message = sqlite_message(&error);
        Self::Sqlite {
            sqlite_code,
            message,
            source: Arc::new(error),
        }
    }

    pub(crate) fn commit(error: rusqlite::Error) -> Self {
        let sqlite_code = sqlite_code(&error);
        let message = sqlite_message(&error);
        Self::Commit {
            sqlite_code,
            message,
            source: Arc::new(error),
        }
    }

    /// Copies a sqlite or commit failure so each caller in a batch holds the
    /// same underlying rusqlite error.
    pub(crate) fn share(&self) -> Self {
        match self {
            Self::Sqlite {
                sqlite_code,
                message,
                source,
            } => Self::Sqlite {
                sqlite_code: *sqlite_code,
                message: message.clone(),
                source: Arc::clone(source),
            },
            Self::Commit {
                sqlite_code,
                message,
                source,
            } => Self::Commit {
                sqlite_code: *sqlite_code,
                message: message.clone(),
                source: Arc::clone(source),
            },
            Self::Open { .. } | Self::ChannelClosed | Self::QueueFull | Self::DeadlineExceeded => {
                unreachable!("only sqlite and commit failures are shared across a batch")
            }
        }
    }

    pub(crate) fn open_sqlite(error: rusqlite::Error) -> Self {
        let message = error.to_string();
        Self::Open {
            message,
            source: Some(Box::new(error)),
        }
    }

    pub(crate) fn open_io(error: std::io::Error) -> Self {
        let message = error.to_string();
        Self::Open {
            message,
            source: Some(Box::new(error)),
        }
    }

    pub(crate) fn open_message(message: impl Into<String>) -> Self {
        Self::Open {
            message: message.into(),
            source: None,
        }
    }
}

impl fmt::Display for SqliteWriterError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Open { message, .. } => write_failure(
                formatter,
                self.code(),
                format_args!("open failed: {message}"),
                self.recovery(),
            ),
            Self::ChannelClosed => write_failure(
                formatter,
                self.code(),
                format_args!("writer channel closed"),
                self.recovery(),
            ),
            Self::QueueFull => write_failure(
                formatter,
                self.code(),
                format_args!("write queue full"),
                self.recovery(),
            ),
            Self::DeadlineExceeded => write_failure(
                formatter,
                self.code(),
                format_args!("write deadline exceeded"),
                self.recovery(),
            ),
            Self::Sqlite {
                sqlite_code,
                message,
                ..
            } => write_failure(
                formatter,
                self.code(),
                format_args!("sqlite error {sqlite_code}: {message}"),
                self.recovery(),
            ),
            Self::Commit {
                sqlite_code,
                message,
                ..
            } => write_failure(
                formatter,
                self.code(),
                format_args!("commit failed: sqlite error {sqlite_code}: {message}"),
                self.recovery(),
            ),
        }
    }
}

impl Error for SqliteWriterError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Open { source, .. } => source
                .as_ref()
                .map(|error| error.as_ref() as &(dyn Error + 'static)),
            Self::Sqlite { source, .. } | Self::Commit { source, .. } => Some(source.as_ref()),
            Self::ChannelClosed | Self::QueueFull | Self::DeadlineExceeded => None,
        }
    }
}

fn write_failure(
    formatter: &mut fmt::Formatter<'_>,
    code: &str,
    symptom: impl fmt::Display,
    recovery: &str,
) -> fmt::Result {
    write!(formatter, "{code}: {symptom}; {recovery}")
}

fn sqlite_code(error: &rusqlite::Error) -> i32 {
    error
        .sqlite_error()
        .map(|sqlite| sqlite.extended_code)
        .unwrap_or(0)
}

fn sqlite_message(error: &rusqlite::Error) -> String {
    match error {
        rusqlite::Error::SqliteFailure(_, Some(message)) => message.clone(),
        other => other.to_string(),
    }
}
