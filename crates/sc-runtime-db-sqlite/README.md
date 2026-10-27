# sc-runtime-db-sqlite

`sc-runtime-db-sqlite` 0.1.0 is published on crates.io under the MIT license. The repository is <https://github.com/randlee/sc-runtime>.

`sc-runtime-db-sqlite` keeps one rusqlite connection on one writer thread. Callers enqueue `execute` and `transaction` work and block until the reply that follows the commit. The crate does not implement a trait shared with sqlx, and template `store-sqlite` does not depend on it.

```rust
use sc_runtime_db_sqlite::{SqlValue, SqliteWriter};

fn main() -> Result<(), sc_runtime_db_sqlite::SqliteWriterError> {
    let writer = SqliteWriter::open(":memory:")?;
    writer.execute("CREATE TABLE note (id INTEGER PRIMARY KEY, body TEXT)", &[])?;
    let inserted = writer.execute(
        "INSERT INTO note (body) VALUES (?1)",
        &[SqlValue::Text("hello".to_owned())],
    )?;
    assert_eq!(inserted, 1);
    Ok(())
}
```

`SqliteWriter` is cheap to clone. Every clone shares the writer thread. Dropping the last handle joins that thread. `open` takes a filesystem path and nothing else.

`open`, `execute`, and `transaction` block the calling thread. From async code, call them inside `tokio::task::spawn_blocking`. Do not drop the last handle on a runtime worker. This crate does not add an async wrapper.

`open` sets `foreign_keys = ON`, a 5 second `busy_timeout`, and `journal_mode = WAL` before the first transaction. An in-memory database stays on SQLite's memory journal. Those settings are fixed.

`execute` and `WriterConn::execute` are write-only. They reject a statement that returns rows, including `SELECT`, `INSERT ... RETURNING`, and `SELECT last_insert_rowid()`. Put the new primary key in the bound parameters. Read committed rows from a separate connection after the call returns. The reply is sent only after the commit.

A `transaction` body must use only the `WriterConn` it is given. Do not capture a `SqliteWriter`. Calling `execute` or `transaction` on a captured handle deadlocks until the write deadline, because the body is running on the only writer thread.

`QueueFull` means the job was not accepted and is safe to retry. `DeadlineExceeded` means the job was accepted and the outcome is unknown: the write may still commit, so do not retry a non-idempotent write until you check the database. `ChannelClosed` means the writer thread is gone; reopen with `SqliteWriter::open`. `ReplyTypeMismatch` means the transaction reply was not the type the body returned. The write may already be committed. It is not `ChannelClosed`.

The writer thread is named `sqlite-writer:<file name>`.
