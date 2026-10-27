use std::fmt;
use std::ptr::NonNull;

use crate::error::SqliteWriterError;
use crate::value::SqlValue;

/// The writer connection, usable only on the writer thread.
///
/// `WriterConn` is not `Sync`. A `transaction` body is the only caller.
/// [`Self::execute`] does not return rows. `SELECT`, `INSERT ... RETURNING`,
/// and `SELECT last_insert_rowid()` are unsupported. Supply the primary key
/// in the bound parameters, and read committed rows from a separate connection
/// after [`crate::SqliteWriter::transaction`] returns.
pub struct WriterConn {
    connection: NonNull<rusqlite::Connection>,
}

impl WriterConn {
    /// Builds a connection handle for the duration of one transaction body.
    ///
    /// # Safety
    ///
    /// `connection` must stay valid until this value is dropped. The caller uses
    /// the handle only on the writer thread. Methods take a shared borrow, matching
    /// `Connection::execute`.
    pub(crate) unsafe fn new(connection: &rusqlite::Connection) -> Self {
        Self {
            connection: NonNull::from_ref(connection),
        }
    }

    /// Runs one statement on the writer connection.
    ///
    /// `params` bind in order as `?1`, `?2`, and so on. The returned count is
    /// the number of rows the statement changed.
    ///
    /// # Errors
    ///
    /// Returns [`SqliteWriterError::Sqlite`] when rusqlite rejects the statement.
    pub fn execute(&mut self, sql: &str, params: &[SqlValue]) -> Result<u64, SqliteWriterError> {
        crate::batch::execute_sql(self.connection(), sql, params)
    }

    fn connection(&self) -> &rusqlite::Connection {
        // SAFETY: `new` requires the pointed-to connection to outlive this
        // value, and `transaction` drops `WriterConn` before the savepoint ends.
        unsafe { self.connection.as_ref() }
    }
}

impl fmt::Debug for WriterConn {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.debug_struct("WriterConn").finish_non_exhaustive()
    }
}
