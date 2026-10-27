//! SQLite store for the example widget operation.
//!
//! [`schema::StoreTarget::SqlitePath`] is a filesystem path for this engine.
//! Postgres and MySQL take a database URL in their own [`schema::StoreTarget`]
//! variants. This
//! crate does not re-export [`EngineStore`] or [`RowRecord`]: the adaptor
//! imports those from the schema crate. It re-exports the open config, the
//! error, and [`WidgetRow`] because a one-store `service` depends on this
//! crate and not on the schema crate.
//!
//! [`EngineStore`] is the adaptor seam (`read_rows` and `write_rows`).
//! [`insert_widget`] and [`get_widget`] are the application functions for this
//! engine. They are not on the trait. A Postgres store writes its own
//! functions and `service` points at those.
//!
//! The read pool allows [`READ_POOL_SIZE`] connections. The write pool
//! maximum is 1. Every `INSERT`, `UPDATE`, and `DELETE` uses that write pool.
//! There is no write actor and no queue.

use std::time::Duration;

use futures_util::TryStreamExt;
use schema::{EngineStore, RowRecord, StoreTarget, TableName, check_read_limit, check_write_limit};
use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePool, SqlitePoolOptions};

#[doc(inline)]
pub use schema::{OpenConfig, StoreCode, StoreError, WidgetId, WidgetRow};

/// Example choice: how many read connections the pool opens.
const READ_POOL_SIZE: u32 = 4;

/// How long a handler waits for a pooled connection before failing.
const ACQUIRE_TIMEOUT: Duration = Duration::from_secs(5);

/// The only table this engine's pairing path dispatches.
const WIDGETS: &str = "widgets";

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
        table: &TableName,
    ) -> impl std::future::Future<Output = Result<Vec<RowRecord>, StoreError>> + Send {
        read_table_rows(self, table)
    }

    fn write_rows(
        &self,
        table: &TableName,
        rows: &[RowRecord],
    ) -> impl std::future::Future<Output = Result<(), StoreError>> + Send {
        write_table_rows(self, table, rows)
    }
}

/// The one table-name decision for read and write.
fn known_table(table: &TableName) -> Result<(), StoreError> {
    if table.as_str() == WIDGETS {
        Ok(())
    } else {
        Err(StoreError::new(
            StoreCode::NotFound,
            format!("unknown table {table}"),
        ))
    }
}

fn from_driver(fallback: StoreCode, err: sqlx::Error) -> StoreError {
    if matches!(err, sqlx::Error::PoolTimedOut) {
        return StoreError::new(StoreCode::Unavailable, "sqlite pool acquire timed out");
    }
    if let Some(db) = err.as_database_error()
        && (db.is_unique_violation() || db.is_foreign_key_violation() || db.is_check_violation())
    {
        return StoreError::new(StoreCode::Conflict, err.to_string());
    }
    StoreError::new(fallback, err.to_string())
}

/// Opens the database at the SQLite path in `config`, enables WAL, and runs migrations.
///
/// The parent directory is created when it is missing. A Postgres or MySQL
/// URL is rejected.
///
/// # Errors
///
/// Returns [`StoreCode::Connection`] when the file or a pool cannot be opened,
/// [`StoreCode::Unavailable`] when a pool acquire times out, and
/// [`StoreCode::Migration`] when a migration fails.
pub async fn open(config: &OpenConfig) -> Result<Store, StoreError> {
    let StoreTarget::SqlitePath(path) = &config.target else {
        return Err(StoreError::new(
            StoreCode::Connection,
            "sqlite OpenConfig must be a filesystem path, not a database URL",
        ));
    };
    let path = path.clone();
    if let Some(parent) = path.parent()
        && !parent.as_os_str().is_empty()
    {
        tokio::fs::create_dir_all(parent)
            .await
            .map_err(sqlx::Error::Io)
            .map_err(|err| from_driver(StoreCode::Connection, err))?;
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
        .map_err(|err| from_driver(StoreCode::Connection, err))?;
    let read = SqlitePoolOptions::new()
        .max_connections(READ_POOL_SIZE)
        .acquire_timeout(ACQUIRE_TIMEOUT)
        .connect_with(options)
        .await
        .map_err(|err| from_driver(StoreCode::Connection, err))?;
    sqlx::migrate!("./migrations")
        .run(&write)
        .await
        .map_err(|err| from_driver(StoreCode::Migration, err.into()))?;
    Ok(Store { read, write })
}

/// Inserts one widget on the write pool and returns the stored row.
///
/// Duplicate names are allowed. The identifier comes from SQLite.
///
/// # Errors
///
/// Returns [`StoreCode::Query`] when the insert fails, [`StoreCode::Conflict`]
/// when a constraint rejects it, and [`StoreCode::Unavailable`] when the write
/// pool times out.
pub async fn insert_widget(store: &Store, name: &str) -> Result<WidgetRow, StoreError> {
    let row = sqlx::query!(
        "INSERT INTO widgets (id, name) VALUES (lower(hex(randomblob(16))), ?1) RETURNING id, name",
        name,
    )
    .fetch_one(&store.write)
    .await
    .map_err(|err| from_driver(StoreCode::Query, err))?;
    Ok(WidgetRow {
        id: WidgetId::from_column(row.id),
        name: row.name,
    })
}

/// Reads one widget by id from the read pool.
///
/// `id` is a [`WidgetId`] or any other column text, including a value the
/// application newtype already validated.
///
/// # Errors
///
/// Returns [`StoreCode::Query`] when the query fails, and
/// [`StoreCode::Unavailable`] when the read pool times out.
pub async fn get_widget(
    store: &Store,
    id: impl AsRef<str>,
) -> Result<Option<WidgetRow>, StoreError> {
    let id = id.as_ref();
    let row = sqlx::query!("SELECT id, name FROM widgets WHERE id = ?1", id,)
        .fetch_optional(&store.read)
        .await
        .map_err(|err| from_driver(StoreCode::Query, err))?;
    Ok(row.map(|row| WidgetRow {
        id: WidgetId::from_column(row.id),
        name: row.name,
    }))
}

async fn read_table_rows(store: &Store, table: &TableName) -> Result<Vec<RowRecord>, StoreError> {
    known_table(table)?;
    read_widget_rows(store).await
}

async fn read_widget_rows(store: &Store) -> Result<Vec<RowRecord>, StoreError> {
    let widgets = TableName::widgets();
    let mut fetched = sqlx::query!("SELECT id, name FROM widgets").fetch(&store.read);
    let mut rows = Vec::new();
    while let Some(row) = fetched
        .try_next()
        .await
        .map_err(|err| from_driver(StoreCode::Query, err))?
    {
        check_read_limit(&widgets, rows.len() + 1)?;
        rows.push(record_from_columns(row.id, row.name)?);
    }
    Ok(rows)
}

async fn write_table_rows(
    store: &Store,
    table: &TableName,
    rows: &[RowRecord],
) -> Result<(), StoreError> {
    known_table(table)?;
    check_write_limit(rows)?;
    write_widget_rows(store, rows).await
}

async fn write_widget_rows(store: &Store, rows: &[RowRecord]) -> Result<(), StoreError> {
    let mut tx = store
        .write
        .begin()
        .await
        .map_err(|err| from_driver(StoreCode::Connection, err))?;
    for row in rows {
        if row.table != WIDGETS {
            return Err(StoreError::new(
                StoreCode::Query,
                "row table does not match the destination table",
            ));
        }
        let widget = widget_from_payload(&row.payload)?;
        let id = widget.id.as_str();
        let name = widget.name.as_str();
        // Upsert this identity only. Deleting the table would drop a distinct
        // id such as `abc` when a later call writes `ABC`. The same id still
        // replaces, so a second copy does not conflict on insert.
        sqlx::query(
            "INSERT INTO widgets (id, name) VALUES (?1, ?2) \
             ON CONFLICT(id) DO UPDATE SET name = excluded.name",
        )
        .bind(id)
        .bind(name)
        .execute(&mut *tx)
        .await
        .map_err(|err| from_driver(StoreCode::Query, err))?;
    }
    tx.commit()
        .await
        .map_err(|err| from_driver(StoreCode::Query, err))?;
    Ok(())
}

fn record_from_columns(id: String, name: String) -> Result<RowRecord, StoreError> {
    let payload = serde_json::to_vec(&WidgetRow {
        id: WidgetId::from_column(id),
        name,
    })
    .map_err(|err| StoreError::new(StoreCode::Query, err.to_string()))?;
    Ok(RowRecord {
        table: TableName::widgets(),
        payload,
    })
}

/// Checks that the read pool can run a statement.
///
/// # Errors
///
/// Returns [`StoreCode::Query`] when the probe statement fails, and
/// [`StoreCode::Unavailable`] when the read pool times out.
pub async fn probe(store: &Store) -> Result<(), StoreError> {
    sqlx::query("SELECT 1")
        .fetch_optional(&store.read)
        .await
        .map_err(|err| from_driver(StoreCode::Query, err))?;
    Ok(())
}

fn widget_from_payload(payload: &[u8]) -> Result<WidgetRow, StoreError> {
    serde_json::from_slice(payload)
        .map_err(|err| StoreError::new(StoreCode::Query, err.to_string()))
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
        .map_err(|err| from_driver(StoreCode::Query, err))?;
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
        .map_err(|err| from_driver(StoreCode::Query, err))
}

#[cfg(test)]
mod tests {
    use super::*;
    use schema::MAX_PAIRING_ROWS;
    use schema::MAX_ROW_PAYLOAD_BYTES;

    fn db_url(dir: &std::path::Path) -> String {
        dir.join("widgets.db").to_string_lossy().into_owned()
    }

    async fn empty() -> (tempfile::TempDir, Store) {
        let dir = tempfile::tempdir().expect("temp dir");
        let store = open(&OpenConfig::sqlite(db_url(dir.path())))
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
        let store = <Store as EngineStore>::open(&OpenConfig::sqlite(db_url(dir.path())))
            .await
            .expect("trait open");
        let written = RowRecord {
            table: TableName::widgets(),
            payload: br#"{"id":"abc","name":"bolt"}"#.to_vec(),
        };
        store
            .write_rows(&TableName::widgets(), &[written])
            .await
            .expect("write");
        let rows = store.read_rows(&TableName::widgets()).await.expect("read");
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].table, WIDGETS);
        let payload: serde_json::Value =
            serde_json::from_slice(&rows[0].payload).expect("json object");
        assert_eq!(payload["id"], "abc");
        assert_eq!(payload["name"], "bolt");
        let loaded = get_widget(&store, WidgetId::from_column("abc"))
            .await
            .expect("get");
        assert_eq!(loaded.expect("row").id.as_str(), "abc");
        let upper = RowRecord {
            table: TableName::widgets(),
            payload: br#"{"id":"ABC","name":"nut"}"#.to_vec(),
        };
        store
            .write_rows(&TableName::widgets(), &[upper])
            .await
            .expect("case-distinct id");
        assert_eq!(count_widgets(&store).await.expect("count"), 2);
        let loaded = get_widget(&store, WidgetId::from_column("abc"))
            .await
            .expect("get");
        assert_eq!(loaded.expect("row").id.as_str(), "abc");
    }

    #[tokio::test]
    async fn failed_write_rolls_back() {
        let (_dir, store) = empty().await;
        let good = RowRecord {
            table: TableName::widgets(),
            payload: br#"{"id":"abc","name":"bolt"}"#.to_vec(),
        };
        let bad = RowRecord {
            table: TableName::widgets(),
            payload: b"not-json".to_vec(),
        };
        let err = store
            .write_rows(&TableName::widgets(), &[good, bad])
            .await
            .expect_err("bad payload");
        assert_eq!(err.code, StoreCode::Query);
        assert!(err.to_string().starts_with("Query:"));
        assert!(std::error::Error::source(&err).is_none());
        assert!(!err.is_retryable());
        let rows = store.read_rows(&TableName::widgets()).await.expect("read");
        assert!(rows.is_empty());
    }

    #[tokio::test]
    async fn second_write_replaces_rows() {
        let (_dir, store) = empty().await;
        let row = RowRecord {
            table: TableName::widgets(),
            payload: br#"{"id":"abc","name":"bolt"}"#.to_vec(),
        };
        store
            .write_rows(&TableName::widgets(), std::slice::from_ref(&row))
            .await
            .expect("first");
        let replaced = RowRecord {
            table: TableName::widgets(),
            payload: br#"{"id":"abc","name":"nut"}"#.to_vec(),
        };
        store
            .write_rows(&TableName::widgets(), std::slice::from_ref(&replaced))
            .await
            .expect("replace");
        let rows = store.read_rows(&TableName::widgets()).await.expect("read");
        assert_eq!(rows.len(), 1);
        let payload: serde_json::Value = serde_json::from_slice(&rows[0].payload).expect("json");
        assert_eq!(payload["name"], "nut");
    }

    #[tokio::test]
    async fn unknown_table_is_not_found() {
        let (_dir, store) = empty().await;
        let err = store
            .read_rows(&TableName::new("missing").expect("table"))
            .await
            .expect_err("missing");
        assert_eq!(err.code, StoreCode::NotFound);
        let err = store
            .write_rows(&TableName::new("missing").expect("table"), &[])
            .await
            .expect_err("missing write");
        assert_eq!(err.code, StoreCode::NotFound);
    }

    #[tokio::test]
    async fn oversized_batch_and_payload_are_rejected() {
        let (_dir, store) = empty().await;
        let too_many = vec![
            RowRecord {
                table: TableName::widgets(),
                payload: br#"{"id":"abc","name":"bolt"}"#.to_vec(),
            };
            MAX_PAIRING_ROWS + 1
        ];
        let err = store
            .write_rows(&TableName::widgets(), &too_many)
            .await
            .expect_err("batch");
        assert_eq!(err.code, StoreCode::Query);
        assert_eq!(count_widgets(&store).await.expect("count"), 0);

        let huge = RowRecord {
            table: TableName::widgets(),
            payload: vec![b'x'; MAX_ROW_PAYLOAD_BYTES + 1],
        };
        let err = store
            .write_rows(&TableName::widgets(), &[huge])
            .await
            .expect_err("payload");
        assert_eq!(err.code, StoreCode::Query);
        assert_eq!(count_widgets(&store).await.expect("count"), 0);
    }

    #[tokio::test]
    async fn open_failure_uses_connection_code() {
        let dir = tempfile::tempdir().expect("temp dir");
        let blocker = dir.path().join("not-a-directory");
        std::fs::write(&blocker, b"x").expect("blocker");
        let err = open(&OpenConfig::sqlite(
            blocker.join("widgets.db").to_string_lossy().into_owned(),
        ))
        .await
        .expect_err("open");
        assert_eq!(err.code, StoreCode::Connection);
        assert!(err.to_string().starts_with("Connection:"));
        assert!(!err.is_retryable());
    }
}
