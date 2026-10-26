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
