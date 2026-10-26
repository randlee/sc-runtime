use std::fmt;

/// Failure from opening the database or from a writer call.
///
/// Display text is fixed: `open failed: {message}`, `writer channel closed`,
/// `write queue full`, `write deadline exceeded`, `sqlite error {code}: {message}`,
/// and `commit failed: {message}`.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum SqliteWriterError {
    /// The database file could not be opened or the writer thread did not start.
    #[error("open failed: {message}")]
    Open {
        /// SQLite or thread-start message.
        message: String,
    },

    /// The writer thread has exited, or the reply channel closed before a reply.
    #[error("writer channel closed")]
    ChannelClosed,

    /// The bounded queue could not accept the job.
    #[error("write queue full")]
    QueueFull,

    /// The job was accepted and no reply arrived within [`crate::WRITE_DEADLINE`].
    #[error("write deadline exceeded")]
    DeadlineExceeded,

    /// rusqlite reported a statement or transaction failure.
    #[error("sqlite error {code}: {message}")]
    Sqlite {
        /// SQLite extended result code.
        code: i32,
        /// SQLite message without the numeric code.
        message: String,
        /// Underlying rusqlite error.
        #[source]
        source: rusqlite::Error,
    },

    /// The outer commit or a savepoint commit failed.
    #[error("commit failed: {message}")]
    Commit {
        /// Commit failure message.
        message: String,
    },
}

impl SqliteWriterError {
    pub(crate) fn sqlite(error: rusqlite::Error) -> Self {
        let code = error.sqlite_extended_error_code().unwrap_or(0);
        let message = match &error {
            rusqlite::Error::SqliteFailure(_, Some(message)) => message.clone(),
            other => other.to_string(),
        };
        Self::Sqlite {
            code,
            message,
            source: error,
        }
    }

    pub(crate) fn sqlite_parts(code: i32, message: String) -> Self {
        let source =
            rusqlite::Error::SqliteFailure(rusqlite::ffi::Error::new(code), Some(message.clone()));
        Self::Sqlite {
            code,
            message,
            source,
        }
    }

    pub(crate) fn open(error: impl fmt::Display) -> Self {
        Self::Open {
            message: error.to_string(),
        }
    }
}
