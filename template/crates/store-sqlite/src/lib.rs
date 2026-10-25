//! SQLite store for the example widget operation.
//!
//! Example choice, not an extension rule: [`OpenConfig`] is a path. A later
//! project can open a different file or a different pool size without treating
//! this struct as a framework type.
//!
//! Example choice, not an extension rule: the read pool allows
//! [`READ_POOL_SIZE`] connections. The write pool maximum is 1. Every
//! `INSERT`, `UPDATE`, and `DELETE` uses that write pool. There is no write
//! actor and no queue.

use std::path::PathBuf;
use std::time::Duration;

use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePool, SqlitePoolOptions};

/// Example choice: how many read connections the pool opens.
const READ_POOL_SIZE: u32 = 4;

/// How long a handler waits for a pooled connection before failing.
const ACQUIRE_TIMEOUT: Duration = Duration::from_secs(5);

/// Failure from the SQLite store. Callers do not need a direct sqlx dependency.
#[derive(Debug)]
pub struct StoreError {
    message: String,
}

impl std::fmt::Display for StoreError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for StoreError {}

fn store_error(err: impl std::fmt::Display) -> StoreError {
    StoreError {
        message: err.to_string(),
    }
}

fn pool_error(err: sqlx::Error) -> StoreError {
    if matches!(err, sqlx::Error::PoolTimedOut) {
        return StoreError {
            message: "sqlite pool acquire timed out".to_owned(),
        };
    }
    store_error(err)
}

/// One widget row.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WidgetRow {
    /// Store-assigned identifier.
    pub id: String,
    /// Stored name.
    pub name: String,
}

/// Example choice: open the database at `path`.
#[derive(Debug, Clone)]
pub struct OpenConfig {
    /// SQLite file. The parent directory is created when it is missing.
    pub path: PathBuf,
}

/// Read pool plus a single-connection write pool on the same file.
#[derive(Debug, Clone)]
pub struct Store {
    read: SqlitePool,
    write: SqlitePool,
}

/// Opens `config.path`, enables WAL, and runs migrations.
///
/// # Errors
///
/// Returns the sqlx error when the file, the pool, or a migration fails.
pub async fn open(config: &OpenConfig) -> Result<Store, StoreError> {
    if let Some(parent) = config.path.parent()
        && !parent.as_os_str().is_empty()
    {
        tokio::fs::create_dir_all(parent)
            .await
            .map_err(sqlx::Error::Io)
            .map_err(pool_error)?;
    }
    let options = SqliteConnectOptions::new()
        .filename(&config.path)
        .create_if_missing(true)
        .journal_mode(SqliteJournalMode::Wal)
        .busy_timeout(Duration::from_secs(5));
    let write = SqlitePoolOptions::new()
        .max_connections(1)
        .acquire_timeout(ACQUIRE_TIMEOUT)
        .connect_with(options.clone())
        .await
        .map_err(pool_error)?;
    let read = SqlitePoolOptions::new()
        .max_connections(READ_POOL_SIZE)
        .acquire_timeout(ACQUIRE_TIMEOUT)
        .connect_with(options)
        .await
        .map_err(pool_error)?;
    sqlx::migrate!("./migrations")
        .run(&write)
        .await
        .map_err(store_error)?;
    Ok(Store { read, write })
}

/// Inserts one widget on the write pool and returns the stored row.
///
/// Duplicate names are allowed. The identifier comes from SQLite.
///
/// # Errors
///
/// Returns the sqlx error when the insert fails.
pub async fn insert_widget(store: &Store, name: &str) -> Result<WidgetRow, StoreError> {
    sqlx::query_as!(
        WidgetRow,
        "INSERT INTO widgets (id, name) VALUES (lower(hex(randomblob(16))), ?1) RETURNING id, name",
        name,
    )
    .fetch_one(&store.write)
    .await
    .map_err(store_error)
}

/// Reads one widget by id from the read pool.
///
/// # Errors
///
/// Returns the sqlx error when the query fails.
pub async fn get_widget(store: &Store, id: &str) -> Result<Option<WidgetRow>, StoreError> {
    sqlx::query_as!(WidgetRow, "SELECT id, name FROM widgets WHERE id = ?1", id,)
        .fetch_optional(&store.read)
        .await
        .map_err(store_error)
}

/// Counts widgets from the read pool.
///
/// # Errors
///
/// Returns the sqlx error when the query fails.
#[cfg(test)]
async fn count_widgets(store: &Store) -> Result<i64, StoreError> {
    let count = sqlx::query_scalar!(r#"SELECT COUNT(*) as "count!: i64" FROM widgets"#)
        .fetch_one(&store.read)
        .await
        .map_err(store_error)?;
    Ok(count)
}

/// Reads `PRAGMA journal_mode` from the write pool.
///
/// # Errors
///
/// Returns the sqlx error when the pragma fails.
#[cfg(test)]
async fn journal_mode(store: &Store) -> Result<String, StoreError> {
    sqlx::query_scalar::<_, String>("PRAGMA journal_mode")
        .fetch_one(&store.write)
        .await
        .map_err(store_error)
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn empty() -> (tempfile::TempDir, Store) {
        let dir = tempfile::tempdir().expect("temp dir");
        let store = open(&OpenConfig {
            path: dir.path().join("widgets.db"),
        })
        .await
        .expect("open");
        (dir, store)
    }

    #[tokio::test]
    async fn empty_database_migrates_and_uses_wal() {
        let (_dir, store) = empty().await;
        let mode = journal_mode(&store).await.expect("journal");
        assert_eq!(mode, "wal");
        assert_eq!(count_widgets(&store).await.expect("count"), 0);
    }

    #[tokio::test]
    async fn concurrent_inserts_do_not_lock() {
        let (_dir, store) = empty().await;
        let mut tasks = Vec::new();
        for index in 0..100 {
            let store = store.clone();
            tasks.push(tokio::spawn(async move {
                insert_widget(&store, &format!("widget-{index}")).await
            }));
        }
        for task in tasks {
            let row = task.await.expect("join").expect("insert");
            assert!(!row.id.is_empty());
        }
        assert_eq!(count_widgets(&store).await.expect("count"), 100);
    }
}
