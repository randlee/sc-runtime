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

pub(crate) fn writer_thread(path: std::path::PathBuf, parts: WriterParts) {
    let _finish = FinishOnExit(parts.finished);
    let mut connection = match Connection::open(&path) {
        Ok(connection) => connection,
        Err(error) => {
            let _ = parts.ready.send(Err(SqliteWriterError::open(error)));
            return;
        }
    };
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

fn process_batch(connection: &mut Connection, batch: Vec<Job>) -> bool {
    let mut transaction = match connection.transaction_with_behavior(TransactionBehavior::Immediate)
    {
        Ok(transaction) => transaction,
        Err(error) => {
            let (code, message) = sqlite_parts(error);
            fail_batch(batch, code, message);
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

fn fail_batch(batch: Vec<Job>, code: i32, message: String) {
    for job in batch {
        send_error(
            reply_of(job),
            SqliteWriterError::sqlite_parts(code, message.clone()),
        );
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
    let commit_error = transaction.commit().err().map(|error| error.to_string());
    for (reply, step) in replies {
        let step = match &commit_error {
            Some(message) => apply_commit_failure(step, message),
            None => step,
        };
        send_step(reply, step);
    }
}

fn apply_commit_failure(step: Step, message: &str) -> Step {
    match step {
        Step::Rows(Ok(_)) => Step::Rows(Err(SqliteWriterError::Commit {
            message: message.to_owned(),
        })),
        Step::Rows(Err(error)) => Step::Rows(Err(error)),
        Step::Boxed(Ok(_)) => Step::Boxed(Err(SqliteWriterError::Commit {
            message: message.to_owned(),
        })),
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
        (reply, _) => reply.fail_closed(),
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
            let (code, message) = sqlite_parts(error);
            return Ok(work
                .into_iter()
                .map(|job| {
                    (
                        job.reply,
                        Step::Rows(Err(SqliteWriterError::sqlite_parts(code, message.clone()))),
                    )
                })
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
            let message = error.to_string();
            Ok(work
                .into_iter()
                .map(|job| {
                    (
                        job.reply,
                        Step::Rows(Err(SqliteWriterError::Commit {
                            message: message.clone(),
                        })),
                    )
                })
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
            Err(error) => Ok(Err(SqliteWriterError::Commit {
                message: error.to_string(),
            })),
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
            Err(error) => Ok(Err(SqliteWriterError::Commit {
                message: error.to_string(),
            })),
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

fn sqlite_parts(error: rusqlite::Error) -> (i32, String) {
    let code = error.sqlite_extended_error_code().unwrap_or(0);
    let message = match &error {
        rusqlite::Error::SqliteFailure(_, Some(message)) => message.clone(),
        other => other.to_string(),
    };
    (code, message)
}

pub(crate) fn execute_sql(
    connection: &Connection,
    sql: &str,
    params: &[SqlValue],
) -> Result<u64, SqliteWriterError> {
    let changed = connection
        .execute(sql, params_from_iter(params.iter()))
        .map_err(SqliteWriterError::sqlite)?;
    Ok(u64::try_from(changed).unwrap_or(u64::MAX))
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
