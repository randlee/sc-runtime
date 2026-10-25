# sc-runtime-db-sqlite Architecture

**Status:** Active as of 2026-10-25. Binding decision: [ADR-RUN-0012](../architecture.md). Sprint `b-1` implements this file. It replaces the withdrawn `StorageBackend` text.

## What this crate is

One rusqlite connection on one thread. Callers enqueue and wait for the reply that follows the commit. The crate does not implement a trait shared with sqlx. Template `store-sqlite` does not depend on it.

## Public types

```rust
pub struct SqliteWriter { /* sender; connection stays on the writer thread */ }

impl SqliteWriter {
    pub fn open(path: impl AsRef<std::path::Path>) -> Result<Self, SqliteWriterError>;
    pub fn execute(&self, sql: &str, params: &[SqlValue]) -> Result<u64, SqliteWriterError>;
    pub fn transaction<T, F>(&self, body: F) -> Result<T, SqliteWriterError>
    where
        F: FnOnce(&mut WriterConn) -> Result<T, SqliteWriterError> + Send + 'static,
        T: Send + 'static;
}

impl std::fmt::Debug for SqliteWriter { /* does not print SQL or row bytes */ }

pub struct WriterConn { /* the connection, usable only on the writer thread */ }

impl WriterConn {
    pub fn execute(&mut self, sql: &str, params: &[SqlValue]) -> Result<u64, SqliteWriterError>;
}

impl std::fmt::Debug for WriterConn { /* does not print SQL or row bytes */ }

#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum SqlValue {
    Null,
    Integer(i64),
    Real(f64),
    Text(String),
    Blob(Vec<u8>),
}

#[non_exhaustive]
pub enum SqliteWriterError {
    Open { message: String },
    ChannelClosed,
    QueueFull,
    DeadlineExceeded,
    Sqlite { code: i32, message: String, source: rusqlite::Error },
    Commit { message: String },
}

/// Matches the measured atm-core queue. Callers block in the reply wait, not in a second queue.
pub const CHANNEL_CAPACITY: usize = 256;
/// Drain budget. The writer calls `yield_now` inside this window. It does not sleep.
pub const BATCH_TIME_BUDGET: std::time::Duration = std::time::Duration::from_millis(1);
/// How long `execute` and `transaction` wait for a reply after the call starts.
pub const WRITE_DEADLINE: std::time::Duration = std::time::Duration::from_secs(10);
/// How long dropping the last handle waits for the writer thread.
pub const SHUTDOWN_JOIN: std::time::Duration = std::time::Duration::from_secs(5);
```

`SqliteWriterError` implements `Display` and `std::error::Error`. Display text is `open failed: {message}`, `writer channel closed`, `write queue full`, `write deadline exceeded`, `sqlite error {code}: {message}`, and `commit failed: {message}`. `Error::source` on `Sqlite` returns the `rusqlite::Error`.

`SqliteWriter` is `Clone + Send + Sync`. `WriterConn` is not `Sync`. `execute` on `SqliteWriter` is the batchable job. Contiguous `execute` calls share one savepoint inside one `BEGIN IMMEDIATE` transaction. A `transaction` body runs alone on the writer thread and calls `WriterConn::execute`. It is not folded into another caller's batch. The reply is sent only after the outer commit. On any member failure the savepoint is dropped and that batch is replayed one statement at a time.

`execute` and `transaction` wait up to `WRITE_DEADLINE` for the reply. If the channel cannot accept the job before that deadline, the error is `QueueFull`. If the job was accepted and the reply does not arrive in time, the error is `DeadlineExceeded`.

If the writer thread exits for any reason, including a panic, every pending caller receives `Err(ChannelClosed)` and does not block. The thread catches the panic with `std::panic::catch_unwind` and drains pending reply senders with `ChannelClosed` before the thread ends.

Dropping the last `SqliteWriter` joins the writer thread within `SHUTDOWN_JOIN`. The drain uses `yield_now` inside `BATCH_TIME_BUDGET`. The batch path does not emit `tracing`. That lane is not part of this crate.

`open` takes a path only ([ADR-RUN-0012](../architecture.md) point 7). Tests may use a `#[cfg(test)]` constructor that shortens `WRITE_DEADLINE`. That constructor is not public.

## What this crate is not

No read pool. No r2d2. No `query` cursor shared with another backend. No migration macro. No `PluginContext`. No method on `DaemonBuilder`.
