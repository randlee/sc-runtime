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
    Open {
        message: String,
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },
    ChannelClosed,
    QueueFull,
    DeadlineExceeded,
    Sqlite { sqlite_code: i32, message: String, source: std::sync::Arc<rusqlite::Error> },
    Commit { sqlite_code: i32, message: String, source: std::sync::Arc<rusqlite::Error> },
    ReplyTypeMismatch,
}

impl SqliteWriterError {
    pub fn code(&self) -> &'static str;
    pub fn recovery(&self) -> &'static str;
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

`SqlValue` does not implement rusqlite's `ToSql`. The writer thread converts parameters inside `execute`.

`SqliteWriterError` implements `Display` and `std::error::Error`. `code()` is the stable code. `recovery()` is the next step. Display text is `{code}: {symptom}; {recovery}`:

| Variant | Code | Symptom |
|---|---|---|
| `Open` | `DBSQ.OPEN` | `open failed: {message}` |
| `ChannelClosed` | `DBSQ.CHANNEL_CLOSED` | `writer channel closed` |
| `QueueFull` | `DBSQ.QUEUE_FULL` | `write queue full` |
| `DeadlineExceeded` | `DBSQ.DEADLINE_EXCEEDED` | `write deadline exceeded` |
| `Sqlite` | `DBSQ.SQLITE` | `sqlite error {sqlite_code}: {message}` |
| `Commit` | `DBSQ.COMMIT` | `commit failed: sqlite error {sqlite_code}: {message}` |
| `ReplyTypeMismatch` | `DBSQ.REPLY_TYPE_MISMATCH` | `transaction reply type mismatch` |

`Error::source` on `Sqlite` and `Commit` returns the original `rusqlite::Error`. The crate does not build a synthetic `SqliteFailure`. rusqlite 0.39's error is not `Clone`, so the variant stores `Arc<rusqlite::Error>` and every caller in a failed batch shares that same error. `Open`'s source is that rusqlite error, or the `std::io::Error` from starting the thread, when one exists. The `rusqlite::Error` behind `Sqlite.source` and `Commit.source` is public, so the rusqlite pin below is the semver boundary for this crate.

Retry follows the variant, not a shared flag:

- `QueueFull`: the job was not accepted. Retrying the call is safe.
- `DeadlineExceeded`: the job was accepted and the outcome is unknown. The write may still commit. Do not retry a non-idempotent write until the database has been checked.
- `ChannelClosed`: the writer thread is gone. Reopen with `SqliteWriter::open`. Do not reuse the handle.
- `Open`: no writer started. Create a missing parent directory and open again.
- `Sqlite`: the statement failed. Read `sqlite_code` and `source`. This crate does not retry it.
- `Commit`: the commit failed. Read `sqlite_code` and `source`.
- `ReplyTypeMismatch`: the transaction reply was not the type the body returned. The job was accepted and the write may already be committed. Do not retry a non-idempotent write until the database has been checked. This is not `ChannelClosed`.

`SqliteWriter` is `Clone + Send + Sync`. `WriterConn` is not `Sync`. `execute` on `SqliteWriter` is the batchable job. Contiguous `execute` calls share one savepoint inside one `BEGIN IMMEDIATE` transaction. A `transaction` body runs alone on the writer thread and calls `WriterConn::execute`. It is not folded into another caller's batch. The body must use only that `WriterConn`. Capturing a `SqliteWriter` and calling `execute` or `transaction` on it deadlocks until `WRITE_DEADLINE`, because the body occupies the only writer thread. The reply is sent only after the outer commit. On any member failure the savepoint is dropped and that batch is replayed one statement at a time.

`WriterConn::execute` is write-only. rusqlite rejects a statement that returns rows, so `SELECT`, `INSERT ... RETURNING`, and `SELECT last_insert_rowid()` are unsupported. Supply the primary key in the bound parameters. Read committed rows from a separate connection after the reply. The reply is sent only after the outer commit.

`open`, `execute`, and `transaction` block the calling thread. From async code, call them inside `tokio::task::spawn_blocking`. Do not drop the last handle on a runtime worker. This crate does not add an async wrapper.

`execute` and `transaction` wait up to `WRITE_DEADLINE` for the reply. If the channel cannot accept the job, the error is `QueueFull`. If the job was accepted and the reply does not arrive in time, the error is `DeadlineExceeded`.

If the writer thread exits for any reason, including a panic, every pending caller receives `Err(ChannelClosed)` and does not block. The thread catches the panic with `std::panic::catch_unwind` and drains pending reply senders with `ChannelClosed` before the thread ends. A failed downcast of a transaction reply returns `ReplyTypeMismatch` and does not panic. It is not `ChannelClosed`. A reply whose shape does not match the job is an invariant breach and panics on the writer thread. That panic is not `ReplyTypeMismatch`.

Dropping the last `SqliteWriter` joins the writer thread within `SHUTDOWN_JOIN`. The drain uses `yield_now` inside `BATCH_TIME_BUDGET`. The batch path does not emit `tracing`. That lane is not part of this crate. The writer thread is named `sqlite-writer:<file name>`. The job sender is a `SyncSender` and is not behind a mutex. The join handle is `!Sync`, so only that handle sits in a mutex.

`open` takes a path only ([ADR-RUN-0012](../architecture.md) point 7). Before the first transaction it sets three fixed connection settings: `PRAGMA foreign_keys = ON`, a 5 second `busy_timeout`, and `journal_mode = WAL` for a filesystem database. An in-memory database stays on SQLite's memory journal, because WAL needs a file. These are not `open` arguments. Tests may use a `#[cfg(test)]` constructor that shortens `WRITE_DEADLINE`. That constructor is not public.

`rusqlite` is pinned to the 0.39 line with default features off and the `bundled` feature on. That line resolves `libsqlite3-sys` 0.37, inside the range `sqlx` 0.9 sqlite accepts (`>=0.30.1, <0.38.0`). The default `cache` feature is rusqlite's statement cache and is unused here. `libsqlite3-sys` uses `links = "sqlite3"`, so a process that also links template `store-sqlite` must resolve one copy. `crates/sc-runtime-db-sqlite/tests/fixtures/sqlx-resolve` is the gate: its lockfile resolves this crate beside `sqlx` 0.9 sqlite, and the crate test checks that metadata contains one `libsqlite3-sys`. The `bundled` feature builds SQLite into this crate; sqlx links that same `libsqlite3-sys`. A later bump past 0.39 fails that gate. cargo-deny also sees `foldhash` (Zlib) through rusqlite's wasm-only `sqlite-wasm-rs` dependency, and `deny.toml` allows that OSI license.

## What this crate is not

No read pool. No r2d2. No `query` cursor shared with another backend. No migration macro. No `PluginContext`. No method on `DaemonBuilder`.
