//! SQLite store for the example widget operation.
//!
//! [`OpenConfig::url`] is the database file path. Postgres and MySQL read a
//! database URL from the same field. The read pool allows [`READ_POOL_SIZE`]
//! connections. The write pool maximum is 1. Every `INSERT`, `UPDATE`, and
//! `DELETE` uses that write pool. There is no write actor and no queue.

use std::path::PathBuf;
use std::time::Duration;

use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePool, SqlitePoolOptions};

#[doc(inline)]
pub use schema::{EngineStore, OpenConfig, RowRecord, StoreCode, StoreError, WidgetRow};

/// Example choice: how many read connections the pool opens.
const READ_POOL_SIZE: u32 = 4;

/// How long a handler waits for a pooled connection before failing.
const ACQUIRE_TIMEOUT: Duration = Duration::from_secs(5);

/// Read pool plus a single-connection write pool on the same file.
#[derive(Debug, Clone)]
pub struct Store {
    read: SqlitePool,
    write: SqlitePool,
}

impl EngineStore for Store {
    fn open(
        cfg: &OpenConfig,
    ) -> impl std::future::Future<Output = Result<Self, StoreError>> + Send {
        crate::open(cfg)
    }

    fn read_rows(
        &self,
        table: &str,
    ) -> impl std::future::Future<Output = Result<Vec<RowRecord>, StoreError>> + Send {
        read_table_rows(self, table)
    }

    fn write_rows(
        &self,
        table: &str,
        rows: &[RowRecord],
    ) -> impl std::future::Future<Output = Result<(), StoreError>> + Send {
        write_table_rows(self, table, rows)
    }
}

fn store_error(code: StoreCode, err: impl std::fmt::Display) -> StoreError {
    StoreError {
        code,
        message: err.to_string(),
    }
}

fn pool_error(err: sqlx::Error) -> StoreError {
    if matches!(err, sqlx::Error::PoolTimedOut) {
        return StoreError {
            code: StoreCode::Connection,
            message: "sqlite pool acquire timed out".to_owned(),
        };
    }
    store_error(StoreCode::Connection, err)
}

/// Opens the database at `config.url`, enables WAL, and runs migrations.
///
/// `url` is a filesystem path. The parent directory is created when it is
/// missing.
///
/// # Errors
///
/// Returns [`StoreCode::Connection`] when the file or a pool cannot be opened,
/// and [`StoreCode::Migration`] when a migration fails.
pub async fn open(config: &OpenConfig) -> Result<Store, StoreError> {
    let path = PathBuf::from(&config.url);
    if let Some(parent) = path.parent()
        && !parent.as_os_str().is_empty()
    {
        tokio::fs::create_dir_all(parent)
            .await
            .map_err(sqlx::Error::Io)
            .map_err(pool_error)?;
    }
    let options = SqliteConnectOptions::new()
        .filename(&path)
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
        .map_err(|err| store_error(StoreCode::Migration, err))?;
    Ok(Store { read, write })
}

/// Inserts one widget on the write pool and returns the stored row.
///
/// Duplicate names are allowed. The identifier comes from SQLite.
///
/// # Errors
///
/// Returns [`StoreCode::Query`] when the insert fails.
pub async fn insert_widget(store: &Store, name: &str) -> Result<WidgetRow, StoreError> {
    let row = sqlx::query!(
        "INSERT INTO widgets (id, name) VALUES (lower(hex(randomblob(16))), ?1) RETURNING id, name",
        name,
    )
    .fetch_one(&store.write)
    .await
    .map_err(|err| store_error(StoreCode::Query, err))?;
    Ok(WidgetRow {
        id: row.id,
        name: row.name,
    })
}

/// Reads one widget by id from the read pool.
///
/// # Errors
///
/// Returns [`StoreCode::Query`] when the query fails.
pub async fn get_widget(store: &Store, id: &str) -> Result<Option<WidgetRow>, StoreError> {
    let row = sqlx::query!("SELECT id, name FROM widgets WHERE id = ?1", id,)
        .fetch_optional(&store.read)
        .await
        .map_err(|err| store_error(StoreCode::Query, err))?;
    Ok(row.map(|row| WidgetRow {
        id: row.id,
        name: row.name,
    }))
}

async fn read_table_rows(store: &Store, table: &str) -> Result<Vec<RowRecord>, StoreError> {
    match table {
        "widgets" => read_widget_rows(store).await,
        _ => Err(store_error(
            StoreCode::Query,
            format!("unknown table {table}"),
        )),
    }
}

async fn read_widget_rows(store: &Store) -> Result<Vec<RowRecord>, StoreError> {
    let rows = sqlx::query!("SELECT id, name FROM widgets")
        .fetch_all(&store.read)
        .await
        .map_err(|err| store_error(StoreCode::Query, err))?;
    rows.into_iter()
        .map(|row| record_from_columns(row.id, row.name))
        .collect()
}

async fn write_table_rows(
    store: &Store,
    table: &str,
    rows: &[RowRecord],
) -> Result<(), StoreError> {
    match table {
        "widgets" => write_widget_rows(store, rows).await,
        _ => Err(store_error(
            StoreCode::Query,
            format!("unknown table {table}"),
        )),
    }
}

async fn write_widget_rows(store: &Store, rows: &[RowRecord]) -> Result<(), StoreError> {
    let mut tx = store
        .write
        .begin()
        .await
        .map_err(|err| store_error(StoreCode::Connection, err))?;
    for row in rows {
        if row.table != "widgets" {
            return Err(store_error(
                StoreCode::Query,
                "row table does not match the destination table",
            ));
        }
        let widget = widget_from_payload(&row.payload)?;
        sqlx::query!(
            "INSERT INTO widgets (id, name) VALUES (?1, ?2)",
            widget.id,
            widget.name,
        )
        .execute(&mut *tx)
        .await
        .map_err(|err| store_error(StoreCode::Query, err))?;
    }
    tx.commit()
        .await
        .map_err(|err| store_error(StoreCode::Query, err))?;
    Ok(())
}

fn record_from_columns(id: String, name: String) -> Result<RowRecord, StoreError> {
    let payload = serde_json::to_vec(&WidgetRow { id, name })
        .map_err(|err| store_error(StoreCode::Query, err))?;
    Ok(RowRecord {
        table: "widgets".to_owned(),
        payload,
    })
}

fn widget_from_payload(payload: &[u8]) -> Result<WidgetRow, StoreError> {
    serde_json::from_slice(payload).map_err(|err| store_error(StoreCode::Query, err))
}

/// Counts widgets from the read pool.
///
/// # Errors
///
/// Returns [`StoreCode::Query`] when the query fails.
#[cfg(test)]
async fn count_widgets(store: &Store) -> Result<i64, StoreError> {
    let count = sqlx::query_scalar!(r#"SELECT COUNT(*) as "count!: i64" FROM widgets"#)
        .fetch_one(&store.read)
        .await
        .map_err(|err| store_error(StoreCode::Query, err))?;
    Ok(count)
}

/// Reads `PRAGMA journal_mode` from the write pool.
///
/// # Errors
///
/// Returns [`StoreCode::Query`] when the pragma fails.
#[cfg(test)]
async fn journal_mode(store: &Store) -> Result<String, StoreError> {
    sqlx::query_scalar::<_, String>("PRAGMA journal_mode")
        .fetch_one(&store.write)
        .await
        .map_err(|err| store_error(StoreCode::Query, err))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn db_url(dir: &std::path::Path) -> String {
        dir.join("widgets.db").to_string_lossy().into_owned()
    }

    async fn empty() -> (tempfile::TempDir, Store) {
        let dir = tempfile::tempdir().expect("temp dir");
        let store = open(&OpenConfig {
            url: db_url(dir.path()),
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

    #[tokio::test]
    async fn engine_store_round_trips_widget_rows() {
        let (dir, _) = empty().await;
        let store = <Store as EngineStore>::open(&OpenConfig {
            url: db_url(dir.path()),
        })
        .await
        .expect("trait open");
        let written = RowRecord {
            table: "widgets".to_owned(),
            payload: br#"{"id":"abc","name":"bolt"}"#.to_vec(),
        };
        store
            .write_rows("widgets", &[written])
            .await
            .expect("write");
        let rows = store.read_rows("widgets").await.expect("read");
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].table, "widgets");
        let payload: serde_json::Value =
            serde_json::from_slice(&rows[0].payload).expect("json object");
        assert_eq!(payload["id"], "abc");
        assert_eq!(payload["name"], "bolt");
    }

    #[tokio::test]
    async fn failed_write_rolls_back() {
        let (_dir, store) = empty().await;
        let good = RowRecord {
            table: "widgets".to_owned(),
            payload: br#"{"id":"abc","name":"bolt"}"#.to_vec(),
        };
        let bad = RowRecord {
            table: "widgets".to_owned(),
            payload: b"not-json".to_vec(),
        };
        let err = store
            .write_rows("widgets", &[good, bad])
            .await
            .expect_err("bad payload");
        assert_eq!(err.code, StoreCode::Query);
        assert!(err.to_string().starts_with("Query:"));
        assert!(std::error::Error::source(&err).is_none());
        let rows = store.read_rows("widgets").await.expect("read");
        assert!(rows.is_empty());
    }

    #[tokio::test]
    async fn open_failure_uses_connection_code() {
        let dir = tempfile::tempdir().expect("temp dir");
        let blocker = dir.path().join("not-a-directory");
        std::fs::write(&blocker, b"x").expect("blocker");
        let err = open(&OpenConfig {
            url: blocker.join("widgets.db").to_string_lossy().into_owned(),
        })
        .await
        .expect_err("open");
        assert_eq!(err.code, StoreCode::Connection);
        assert!(err.to_string().starts_with("Connection:"));
    }
}
