use std::any::Any;
use std::fmt;
use std::path::Path;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::mpsc::{self, SyncSender, TrySendError};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use crate::batch::{self, Job, Reply, ReplySlot, TransactionBody, WriterParts};
use crate::error::SqliteWriterError;
use crate::value::SqlValue;
use crate::{CHANNEL_CAPACITY, SHUTDOWN_JOIN, WRITE_DEADLINE};

/// Handle for the single rusqlite writer thread.
///
/// Clones share the thread. Dropping the last handle asks the thread to exit
/// and joins it for up to [`SHUTDOWN_JOIN`].
///
/// `open`, `execute`, and `transaction` block the calling thread. From async
/// code, call them inside `tokio::task::spawn_blocking`. Do not drop the last
/// handle on a runtime worker: the join busy-waits for up to [`SHUTDOWN_JOIN`].
/// This crate does not add an async wrapper.
///
/// The writer thread is named `sqlite-writer:<file name>`, using the path's
/// file name. `:memory:` uses that string as the file name.
#[derive(Clone)]
#[must_use = "dropping the last handle joins the writer thread"]
pub struct SqliteWriter {
    shared: Arc<Shared>,
}

struct Shared {
    /// Taken in `Drop` so the writer thread observes a disconnected job channel.
    /// `SyncSender` is `Sync`, so the sender is not behind a mutex.
    sender: Option<SyncSender<Job>>,
    /// `JoinHandle` is `!Sync`. This mutex exists so the handle can live in a
    /// `Sync` writer. It does not guard the job sender.
    join: Mutex<Option<JoinHandle<()>>>,
    deadline: Duration,
    queued: Arc<AtomicUsize>,
    /// Set when the writer thread exits. Tests observe it after the last handle drops.
    #[cfg_attr(
        not(test),
        expect(
            dead_code,
            reason = "tests observe this flag after the last handle drops"
        )
    )]
    finished: Arc<AtomicBool>,
}

impl SqliteWriter {
    /// Opens `path` and starts the writer thread.
    ///
    /// `:memory:` opens a private in-memory database. A filesystem path creates
    /// the file when it is missing. The parent directory must already exist.
    ///
    /// Before the first transaction, the writer sets `foreign_keys = ON`, a
    /// 5 second `busy_timeout`, and `journal_mode = WAL`. An in-memory database
    /// stays on SQLite's memory journal. These settings are fixed. `open` does
    /// not take options.
    ///
    /// # Examples
    ///
    /// ```
    /// use sc_runtime_db_sqlite::{SqlValue, SqliteWriter};
    ///
    /// let writer = SqliteWriter::open(":memory:")?;
    /// writer.execute(
    ///     "CREATE TABLE note (body TEXT)",
    ///     &[],
    /// )?;
    /// let inserted = writer.execute(
    ///     "INSERT INTO note (body) VALUES (?1)",
    ///     &[SqlValue::Text("hello".to_owned())],
    /// )?;
    /// assert_eq!(inserted, 1);
    /// # Ok::<(), sc_runtime_db_sqlite::SqliteWriterError>(())
    /// ```
    ///
    /// # Errors
    ///
    /// Returns [`SqliteWriterError::Open`] when the file cannot be opened or the
    /// writer thread cannot start.
    pub fn open(path: impl AsRef<Path>) -> Result<Self, SqliteWriterError> {
        Self::open_inner(path.as_ref(), WRITE_DEADLINE)
    }

    /// Opens a writer whose calls wait `deadline` instead of [`WRITE_DEADLINE`].
    ///
    /// Production [`open`](Self::open) does not expose this knob.
    #[cfg(test)]
    pub(crate) fn open_with_deadline(
        path: impl AsRef<Path>,
        deadline: Duration,
    ) -> Result<Self, SqliteWriterError> {
        Self::open_inner(path.as_ref(), deadline)
    }

    fn open_inner(path: &Path, deadline: Duration) -> Result<Self, SqliteWriterError> {
        let (job_tx, job_rx) = mpsc::sync_channel(CHANNEL_CAPACITY);
        let (ready_tx, ready_rx) = mpsc::sync_channel(1);
        let queued = Arc::new(AtomicUsize::new(0));
        let finished = Arc::new(AtomicBool::new(false));
        let parts = WriterParts {
            jobs: job_rx,
            ready: ready_tx,
            queued: Arc::clone(&queued),
            finished: Arc::clone(&finished),
        };
        let thread_path = path.to_path_buf();
        let handle = thread::Builder::new()
            .name(writer_thread_name(path))
            .spawn(move || batch::writer_thread(thread_path, parts))
            .map_err(SqliteWriterError::open_io)?;
        match ready_rx.recv_timeout(SHUTDOWN_JOIN) {
            Ok(Ok(())) => {}
            Ok(Err(error)) => {
                let _ = handle.join();
                return Err(error);
            }
            Err(_) => {
                let _ = handle.join();
                return Err(SqliteWriterError::open_message(
                    "writer thread stopped before the database opened",
                ));
            }
        }
        Ok(Self {
            shared: Arc::new(Shared {
                sender: Some(job_tx),
                join: Mutex::new(Some(handle)),
                deadline,
                queued,
                finished,
            }),
        })
    }

    /// Enqueues one statement and waits until the batch that contains it commits.
    ///
    /// Contiguous `execute` calls share one savepoint inside one
    /// `BEGIN IMMEDIATE` transaction. The reply is the number of rows changed,
    /// and it is sent only after that outer commit.
    ///
    /// # Errors
    ///
    /// Returns [`SqliteWriterError::QueueFull`] when the queue cannot accept the
    /// job, [`SqliteWriterError::DeadlineExceeded`] when the job was accepted
    /// and no reply arrives in time, [`SqliteWriterError::ChannelClosed`] when
    /// the writer thread has exited, and [`SqliteWriterError::Sqlite`] or
    /// [`SqliteWriterError::Commit`] when the statement or the commit fails.
    pub fn execute(&self, sql: &str, params: &[SqlValue]) -> Result<u64, SqliteWriterError> {
        let started = Instant::now();
        let (sender, receiver) = mpsc::sync_channel(1);
        let job = Job::Execute(batch::ExecuteWork {
            sql: sql.to_owned(),
            params: params.to_vec(),
            reply: Reply::Rows(ReplySlot::new(sender)),
        });
        self.enqueue(job)?;
        batch::wait_reply(&receiver, self.shared.deadline, started)
    }

    /// Runs `body` alone on the writer thread.
    ///
    /// The body is not folded into another caller's `execute` batch. Its
    /// statements use [`crate::WriterConn::execute`]. The reply is sent only
    /// after the outer commit. Dropping the body's savepoint rolls its writes
    /// back when `body` returns an error.
    ///
    /// The body must use only the [`crate::WriterConn`] it is given. Do not
    /// capture a [`SqliteWriter`]: `execute` or `transaction` on a captured
    /// handle waits for this body to finish, and the body is waiting on that
    /// call, so the caller blocks until [`crate::WRITE_DEADLINE`]. Do not drop
    /// the last [`SqliteWriter`] from the body. That drop joins the writer
    /// thread from the writer thread.
    ///
    /// `WriterConn::execute` does not return rows. `SELECT`,
    /// `INSERT ... RETURNING`, and `SELECT last_insert_rowid()` are
    /// unsupported. Put the new primary key in the bound parameters. Read
    /// committed rows from a separate connection after this method returns.
    ///
    /// # Errors
    ///
    /// Returns the same queue, deadline, and channel errors as [`Self::execute`].
    /// An error returned by `body` is returned to this caller after the rest of
    /// the outer batch commits or fails. A panic inside `body` closes the writer,
    /// and this call returns [`SqliteWriterError::ChannelClosed`].
    /// [`SqliteWriterError::ReplyTypeMismatch`] means the reply value was not
    /// `T`. That mismatch does not panic, and it is not `ChannelClosed`.
    pub fn transaction<T, F>(&self, body: F) -> Result<T, SqliteWriterError>
    where
        F: FnOnce(&mut crate::WriterConn) -> Result<T, SqliteWriterError> + Send + 'static,
        T: Send + 'static,
    {
        let started = Instant::now();
        let (sender, receiver) = mpsc::sync_channel(1);
        let wrapped: TransactionBody = Box::new(move |connection| {
            body(connection).map(|value| Box::new(value) as Box<dyn Any + Send>)
        });
        let job = Job::Transaction {
            body: wrapped,
            reply: Reply::Value(ReplySlot::new(sender)),
        };
        self.enqueue(job)?;
        let boxed = batch::wait_reply(&receiver, self.shared.deadline, started)?;
        downcast_transaction_reply(boxed)
    }

    /// Enqueues a job that panics on the writer thread before commit.
    ///
    /// Every caller queued with that batch receives [`SqliteWriterError::ChannelClosed`].
    #[cfg(test)]
    pub(crate) fn panic_for_test(&self) -> Result<(), SqliteWriterError> {
        let started = Instant::now();
        let (sender, receiver) = mpsc::sync_channel(1);
        let job = Job::Panic {
            reply: Reply::Unit(ReplySlot::new(sender)),
        };
        self.enqueue(job)?;
        batch::wait_reply(&receiver, self.shared.deadline, started)
    }

    #[cfg(test)]
    pub(crate) fn queued_for_test(&self) -> usize {
        self.shared.queued.load(Ordering::SeqCst)
    }

    #[cfg(test)]
    pub(crate) fn finished_flag_for_test(&self) -> Arc<AtomicBool> {
        Arc::clone(&self.shared.finished)
    }

    fn enqueue(&self, job: Job) -> Result<(), SqliteWriterError> {
        let Some(sender) = self.shared.sender.as_ref() else {
            return Err(SqliteWriterError::ChannelClosed);
        };
        match sender.try_send(job) {
            Ok(()) => {
                self.shared.queued.fetch_add(1, Ordering::SeqCst);
                Ok(())
            }
            Err(TrySendError::Full(_)) => Err(SqliteWriterError::QueueFull),
            Err(TrySendError::Disconnected(_)) => Err(SqliteWriterError::ChannelClosed),
        }
    }
}

impl Drop for Shared {
    fn drop(&mut self) {
        // Disconnect the job channel before joining. The sender needs no
        // mutex. The join mutex exists only because `JoinHandle` is `!Sync`.
        self.sender.take();
        let handle = match self.join.get_mut() {
            Ok(join) => join.take(),
            Err(poisoned) => poisoned.into_inner().take(),
        };
        if let Some(handle) = handle {
            join_within(handle, SHUTDOWN_JOIN);
        }
    }
}

fn downcast_transaction_reply<T: 'static>(
    boxed: Box<dyn Any + Send>,
) -> Result<T, SqliteWriterError> {
    match boxed.downcast::<T>() {
        Ok(value) => Ok(*value),
        Err(_) => Err(SqliteWriterError::ReplyTypeMismatch),
    }
}

fn writer_thread_name(path: &Path) -> String {
    let base = path
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .filter(|name| !name.is_empty())
        .unwrap_or_else(|| "db".to_owned());
    format!("sqlite-writer:{base}")
}

fn join_within(handle: JoinHandle<()>, timeout: Duration) {
    let started = Instant::now();
    loop {
        if handle.is_finished() {
            let _ = handle.join();
            return;
        }
        if started.elapsed() >= timeout {
            return;
        }
        thread::yield_now();
    }
}

impl fmt::Debug for SqliteWriter {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("SqliteWriter")
            .field("deadline", &self.shared.deadline)
            .finish_non_exhaustive()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn transaction_reply_downcast_returns_the_body_type() {
        let boxed: Box<dyn Any + Send> = Box::new("ok".to_owned());
        let value = downcast_transaction_reply::<String>(boxed).expect("same type");
        assert_eq!(value, "ok");
    }

    #[test]
    fn transaction_reply_type_mismatch_is_not_channel_closed() {
        let boxed: Box<dyn Any + Send> = Box::new(1_u32);
        let error = downcast_transaction_reply::<String>(boxed).expect_err("type mismatch");
        assert!(matches!(error, SqliteWriterError::ReplyTypeMismatch));
        assert_ne!(error.code(), SqliteWriterError::ChannelClosed.code());
    }
}
