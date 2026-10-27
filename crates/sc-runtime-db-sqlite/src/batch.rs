//! Writer thread: one `BEGIN IMMEDIATE`, savepoint groups, reply after commit.
//!
//! Contiguous `execute` jobs share one savepoint. A `transaction` body runs on
//! its own savepoint and is not folded into that group. The caller is answered
//! only after the outer commit. A failed member rolls the group savepoint back
//! and the group is replayed one statement at a time.

use std::any::Any;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::mpsc::{Receiver, RecvTimeoutError, SyncSender, TryRecvError};
use std::thread;
use std::time::{Duration, Instant};

use rusqlite::params_from_iter;
use rusqlite::types::{ToSql, ToSqlOutput, Value, ValueRef};
use rusqlite::{Connection, Transaction, TransactionBehavior};

use crate::BATCH_TIME_BUDGET;
use crate::conn::WriterConn;
use crate::error::SqliteWriterError;
use crate::value::SqlValue;

pub(crate) type TransactionBody =
    Box<dyn FnOnce(&mut WriterConn) -> Result<Box<dyn Any + Send>, SqliteWriterError> + Send>;

pub(crate) struct ExecuteWork {
    pub(crate) sql: String,
    pub(crate) params: Vec<SqlValue>,
    pub(crate) reply: Reply,
}

pub(crate) enum Job {
    Execute(ExecuteWork),
    Transaction {
        body: TransactionBody,
        reply: Reply,
    },
    #[cfg(test)]
    Panic {
        reply: Reply,
    },
}

pub(crate) enum Reply {
    Rows(ReplySlot<u64>),
    Value(ReplySlot<Box<dyn Any + Send>>),
    #[cfg(test)]
    Unit(ReplySlot<()>),
}

impl Reply {
    fn fail_closed(self) {
        match self {
            Self::Rows(slot) => slot.send(Err(SqliteWriterError::ChannelClosed)),
            Self::Value(slot) => slot.send(Err(SqliteWriterError::ChannelClosed)),
            #[cfg(test)]
            Self::Unit(slot) => slot.send(Err(SqliteWriterError::ChannelClosed)),
        }
    }
}

pub(crate) struct ReplySlot<T> {
    sender: Option<SyncSender<Result<T, SqliteWriterError>>>,
}

impl<T> ReplySlot<T> {
    pub(crate) fn new(sender: SyncSender<Result<T, SqliteWriterError>>) -> Self {
        Self {
            sender: Some(sender),
        }
    }

    fn send(mut self, result: Result<T, SqliteWriterError>) {
        if let Some(sender) = self.sender.take() {
            let _ = sender.send(result);
        }
    }
}

impl<T> Drop for ReplySlot<T> {
    fn drop(&mut self) {
        if let Some(sender) = self.sender.take() {
            let _ = sender.send(Err(SqliteWriterError::ChannelClosed));
        }
    }
}

enum Step {
    Rows(Result<u64, SqliteWriterError>),
    Boxed(Result<Box<dyn Any + Send>, SqliteWriterError>),
}

pub(crate) struct WriterParts {
    pub(crate) jobs: Receiver<Job>,
    pub(crate) ready: SyncSender<Result<(), SqliteWriterError>>,
    pub(crate) queued: std::sync::Arc<AtomicUsize>,
    pub(crate) finished: std::sync::Arc<AtomicBool>,
}

/// Fixed `busy_timeout`. `open` takes no options, so this stays a crate constant.
const BUSY_TIMEOUT: Duration = Duration::from_secs(5);

pub(crate) fn writer_thread(path: std::path::PathBuf, parts: WriterParts) {
    let _finish = FinishOnExit(parts.finished);
    let mut connection = match Connection::open(&path) {
        Ok(connection) => connection,
        Err(error) => {
            let _ = parts.ready.send(Err(SqliteWriterError::open_sqlite(error)));
            return;
        }
    };
    // Pragmas run before the first transaction. `foreign_keys` is a no-op
    // inside a transaction, and `journal_mode` cannot change there either.
    if let Err(error) = configure_connection(&connection) {
        let _ = parts.ready.send(Err(SqliteWriterError::open_sqlite(error)));
        return;
    }
    if parts.ready.send(Ok(())).is_err() {
        return;
    }

    loop {
        let first = match parts.jobs.recv() {
            Ok(job) => {
                note_received(&parts.queued);
                job
            }
            Err(_) => break,
        };
        let mut batch = vec![first];
        collect_batch(&parts.jobs, &mut batch, &parts.queued);
        let batch_result = catch_unwind(AssertUnwindSafe(|| process_batch(&mut connection, batch)));
        let panicked = batch_result.unwrap_or(true);
        if panicked {
            break;
        }
    }
}

struct FinishOnExit(std::sync::Arc<AtomicBool>);

impl Drop for FinishOnExit {
    fn drop(&mut self) {
        self.0.store(true, Ordering::SeqCst);
    }
}

fn note_received(queued: &AtomicUsize) {
    queued.fetch_sub(1, Ordering::SeqCst);
}

fn collect_batch(jobs: &Receiver<Job>, batch: &mut Vec<Job>, queued: &AtomicUsize) {
    let deadline = Instant::now() + BATCH_TIME_BUDGET;
    loop {
        match jobs.try_recv() {
            Ok(job) => {
                note_received(queued);
                batch.push(job);
            }
            Err(TryRecvError::Empty) => {
                if Instant::now() >= deadline {
                    return;
                }
                thread::yield_now();
            }
            Err(TryRecvError::Disconnected) => return,
        }
    }
}

fn configure_connection(connection: &Connection) -> Result<(), rusqlite::Error> {
    connection.busy_timeout(BUSY_TIMEOUT)?;
    connection.pragma_update(None, "foreign_keys", "ON")?;
    // WAL is the journal for a filesystem database. SQLite leaves an
    // in-memory database on the memory journal; that pragma still succeeds.
    connection.pragma_update(None, "journal_mode", "WAL")?;
    Ok(())
}

fn process_batch(connection: &mut Connection, batch: Vec<Job>) -> bool {
    let mut transaction = match connection.transaction_with_behavior(TransactionBehavior::Immediate)
    {
        Ok(transaction) => transaction,
        Err(error) => {
            fail_batch(batch, error);
            return false;
        }
    };
    let mut replies = Vec::new();
    let mut jobs = batch.into_iter().peekable();
    while let Some(job) = jobs.next() {
        match job {
            Job::Execute(work) => {
                let mut group = vec![work];
                while matches!(jobs.peek(), Some(Job::Execute(_))) {
                    if let Some(Job::Execute(next)) = jobs.next() {
                        group.push(next);
                    }
                }
                match run_execute_group(&mut transaction, group) {
                    Ok(done) => replies.extend(done),
                    Err(pending) => {
                        return fail_open_batch(transaction, pending, &mut jobs, replies);
                    }
                }
            }
            Job::Transaction { body, reply } => {
                match run_user_transaction(&mut transaction, body) {
                    Ok(result) => replies.push((reply, Step::Boxed(result))),
                    Err(()) => {
                        return fail_open_batch(transaction, vec![reply], &mut jobs, replies);
                    }
                }
            }
            #[cfg(test)]
            Job::Panic { reply } => {
                let _ = catch_unwind(AssertUnwindSafe(|| {
                    panic!("sqlite writer panicked before commit");
                }));
                return fail_open_batch(transaction, vec![reply], &mut jobs, replies);
            }
        }
    }
    commit_and_reply(transaction, replies);
    false
}

fn fail_open_batch<I>(
    transaction: Transaction<'_>,
    mut pending: Vec<Reply>,
    jobs: &mut I,
    replies: Vec<(Reply, Step)>,
) -> bool
where
    I: Iterator<Item = Job>,
{
    drop(transaction);
    pending.extend(jobs.map(reply_of));
    pending.extend(replies.into_iter().map(|(reply, _)| reply));
    for reply in pending {
        reply.fail_closed();
    }
    true
}

fn fail_batch(batch: Vec<Job>, error: rusqlite::Error) {
    let error = SqliteWriterError::sqlite(error);
    for job in batch {
        send_error(reply_of(job), error.share());
    }
}

fn reply_of(job: Job) -> Reply {
    match job {
        Job::Execute(work) => work.reply,
        Job::Transaction { reply, .. } => reply,
        #[cfg(test)]
        Job::Panic { reply } => reply,
    }
}

fn commit_and_reply(transaction: Transaction<'_>, replies: Vec<(Reply, Step)>) {
    let commit_error = transaction.commit().err().map(SqliteWriterError::commit);
    for (reply, step) in replies {
        let step = match &commit_error {
            Some(error) => apply_commit_failure(step, error),
            None => step,
        };
        send_step(reply, step);
    }
}

fn apply_commit_failure(step: Step, error: &SqliteWriterError) -> Step {
    match step {
        Step::Rows(Ok(_)) => Step::Rows(Err(error.share())),
        Step::Rows(Err(error)) => Step::Rows(Err(error)),
        Step::Boxed(Ok(_)) => Step::Boxed(Err(error.share())),
        Step::Boxed(Err(error)) => Step::Boxed(Err(error)),
    }
}

fn send_error(reply: Reply, error: SqliteWriterError) {
    match reply {
        Reply::Rows(slot) => slot.send(Err(error)),
        Reply::Value(slot) => slot.send(Err(error)),
        #[cfg(test)]
        Reply::Unit(slot) => slot.send(Err(error)),
    }
}

fn send_step(reply: Reply, step: Step) {
    match (reply, step) {
        (Reply::Rows(slot), Step::Rows(result)) => slot.send(result),
        (Reply::Value(slot), Step::Boxed(result)) => slot.send(result),
        (Reply::Rows(_), Step::Boxed(_)) => {
            unreachable!("an execute reply is a row count")
        }
        (Reply::Value(_), Step::Rows(_)) => {
            unreachable!("a transaction reply is a boxed value")
        }
        #[cfg(test)]
        (Reply::Unit(_), _) => {
            unreachable!("a panic job does not send a step")
        }
    }
}

fn run_execute_group(
    transaction: &mut Transaction<'_>,
    work: Vec<ExecuteWork>,
) -> Result<Vec<(Reply, Step)>, Vec<Reply>> {
    if work.len() == 1 {
        let mut work = work;
        let Some(job) = work.pop() else {
            return Ok(Vec::new());
        };
        return match run_one(transaction, &job.sql, &job.params) {
            Ok(result) => Ok(vec![(job.reply, Step::Rows(result))]),
            Err(()) => Err(vec![job.reply]),
        };
    }

    let savepoint = match transaction.savepoint() {
        Ok(savepoint) => savepoint,
        Err(error) => {
            let error = SqliteWriterError::sqlite(error);
            return Ok(work
                .into_iter()
                .map(|job| (job.reply, Step::Rows(Err(error.share()))))
                .collect());
        }
    };

    let mut counts = Vec::with_capacity(work.len());
    for job in &work {
        match catch_unwind(AssertUnwindSafe(|| {
            execute_sql(&savepoint, &job.sql, &job.params)
        })) {
            Ok(Ok(count)) => counts.push(count),
            Ok(Err(_)) => {
                drop(savepoint);
                return replay(transaction, work);
            }
            Err(_) => {
                drop(savepoint);
                return Err(work.into_iter().map(|job| job.reply).collect());
            }
        }
    }

    match savepoint.commit() {
        Ok(()) => Ok(work
            .into_iter()
            .zip(counts)
            .map(|(job, count)| (job.reply, Step::Rows(Ok(count))))
            .collect()),
        Err(error) => {
            let error = SqliteWriterError::commit(error);
            Ok(work
                .into_iter()
                .map(|job| (job.reply, Step::Rows(Err(error.share()))))
                .collect())
        }
    }
}

fn replay(
    transaction: &mut Transaction<'_>,
    work: Vec<ExecuteWork>,
) -> Result<Vec<(Reply, Step)>, Vec<Reply>> {
    let mut pending = work.into_iter();
    let mut done = Vec::new();
    while let Some(job) = pending.next() {
        match run_one(transaction, &job.sql, &job.params) {
            Ok(result) => done.push((job.reply, Step::Rows(result))),
            Err(()) => {
                let mut replies = done.into_iter().map(|(reply, _)| reply).collect::<Vec<_>>();
                replies.push(job.reply);
                replies.extend(pending.map(|job| job.reply));
                return Err(replies);
            }
        }
    }
    Ok(done)
}

fn run_one(
    transaction: &mut Transaction<'_>,
    sql: &str,
    params: &[SqlValue],
) -> Result<Result<u64, SqliteWriterError>, ()> {
    let savepoint = match transaction.savepoint() {
        Ok(savepoint) => savepoint,
        Err(error) => return Ok(Err(SqliteWriterError::sqlite(error))),
    };
    let executed = catch_unwind(AssertUnwindSafe(|| execute_sql(&savepoint, sql, params)));
    match executed {
        Ok(Ok(count)) => match savepoint.commit() {
            Ok(()) => Ok(Ok(count)),
            Err(error) => Ok(Err(SqliteWriterError::commit(error))),
        },
        Ok(Err(error)) => {
            drop(savepoint);
            Ok(Err(error))
        }
        Err(_) => {
            drop(savepoint);
            Err(())
        }
    }
}

fn run_user_transaction(
    transaction: &mut Transaction<'_>,
    body: TransactionBody,
) -> Result<Result<Box<dyn Any + Send>, SqliteWriterError>, ()> {
    let savepoint = match transaction.savepoint() {
        Ok(savepoint) => savepoint,
        Err(error) => return Ok(Err(SqliteWriterError::sqlite(error))),
    };
    let outcome = {
        // SAFETY: `writer_conn` ends before `savepoint` is committed or dropped.
        // The body runs on this writer thread and cannot share the connection.
        let mut writer_conn = unsafe { WriterConn::new(&savepoint) };
        catch_unwind(AssertUnwindSafe(|| body(&mut writer_conn)))
    };
    match outcome {
        Ok(Ok(value)) => match savepoint.commit() {
            Ok(()) => Ok(Ok(value)),
            Err(error) => Ok(Err(SqliteWriterError::commit(error))),
        },
        Ok(Err(error)) => {
            drop(savepoint);
            Ok(Err(error))
        }
        Err(_) => {
            drop(savepoint);
            Err(())
        }
    }
}

struct Bound<'a>(&'a SqlValue);

impl ToSql for Bound<'_> {
    fn to_sql(&self) -> rusqlite::Result<ToSqlOutput<'_>> {
        match self.0 {
            SqlValue::Null => Ok(ToSqlOutput::Owned(Value::Null)),
            SqlValue::Integer(value) => Ok(ToSqlOutput::Owned(Value::Integer(*value))),
            SqlValue::Real(value) => Ok(ToSqlOutput::Owned(Value::Real(*value))),
            SqlValue::Text(value) => Ok(ToSqlOutput::Borrowed(ValueRef::Text(value.as_bytes()))),
            SqlValue::Blob(value) => Ok(ToSqlOutput::Borrowed(ValueRef::Blob(value))),
        }
    }
}

pub(crate) fn execute_sql(
    connection: &Connection,
    sql: &str,
    params: &[SqlValue],
) -> Result<u64, SqliteWriterError> {
    let changed = connection
        .execute(sql, params_from_iter(params.iter().map(Bound)))
        .map_err(SqliteWriterError::sqlite)?;
    Ok(rows_changed(changed))
}

fn rows_changed(changed: usize) -> u64 {
    const {
        assert!(
            usize::BITS <= u64::BITS,
            "a changed-row count must fit in u64"
        );
    }
    changed as u64
}

pub(crate) fn wait_reply<T>(
    receiver: &Receiver<Result<T, SqliteWriterError>>,
    deadline: Duration,
    started: Instant,
) -> Result<T, SqliteWriterError> {
    let remaining = deadline.saturating_sub(started.elapsed());
    match receiver.recv_timeout(remaining) {
        Ok(result) => result,
        Err(RecvTimeoutError::Timeout) => Err(SqliteWriterError::DeadlineExceeded),
        Err(RecvTimeoutError::Disconnected) => Err(SqliteWriterError::ChannelClosed),
    }
}
