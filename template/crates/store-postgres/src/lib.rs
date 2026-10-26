//! Postgres store for the example widget operation.
//!
//! [`OpenConfig::url`] is a `postgres://` or `postgresql://` database URL.
//! This crate does not re-export [`EngineStore`] or [`RowRecord`]: the adaptor
//! imports those from the schema crate. It re-exports the open config, the
//! error, and [`WidgetRow`] because a one-store `service` depends on this
//! crate and not on the schema crate.
//!
//! [`EngineStore`] is the adaptor seam (`read_rows` and `write_rows`).
//! [`insert_widget`] and [`get_widget`] are the application functions for this
//! engine. They are not on the trait. A SQLite store writes its own functions
//! and `service` points at those.
//!
//! Postgres uses one pool of [`POOL_SIZE`] connections. Reads and writes share
//! it. There is no write actor and no queue.

use std::time::Duration;

use futures_util::TryStreamExt;
use schema::{EngineStore, RowRecord, check_read_limit, check_write_limit};
use sqlx::postgres::{PgPool, PgPoolOptions};

#[doc(inline)]
pub use schema::{OpenConfig, StoreCode, StoreError, WidgetId, WidgetRow};

/// Example choice: how many connections the single pool opens.
const POOL_SIZE: u32 = 10;

/// How long a handler waits for a pooled connection before failing.
const ACQUIRE_TIMEOUT: Duration = Duration::from_secs(5);

/// The only table this engine's pairing path dispatches.
const WIDGETS: &str = "widgets";

/// One Postgres pool on the database named by [`OpenConfig::url`].
#[derive(Debug, Clone)]
pub struct Store {
    pool: PgPool,
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

/// The one table-name decision for read and write.
fn known_table(table: &str) -> Result<(), StoreError> {
    if table == WIDGETS {
        Ok(())
    } else {
        Err(StoreError::new(
            StoreCode::NotFound,
            format!("unknown table {table}"),
        ))
    }
}

fn require_postgres_url(url: &str) -> Result<(), StoreError> {
    if url.starts_with("postgres://") || url.starts_with("postgresql://") {
        Ok(())
    } else {
        Err(StoreError::new(
            StoreCode::Connection,
            "postgres OpenConfig.url must be a postgres:// or postgresql:// database URL",
        ))
    }
}

fn from_driver(fallback: StoreCode, err: sqlx::Error) -> StoreError {
    if matches!(err, sqlx::Error::PoolTimedOut) {
        return StoreError::new(StoreCode::Unavailable, "postgres pool acquire timed out");
    }
    if let Some(db) = err.as_database_error()
        && (db.is_unique_violation() || db.is_foreign_key_violation() || db.is_check_violation())
    {
        return StoreError::new(StoreCode::Conflict, err.to_string());
    }
    StoreError::new(fallback, err.to_string())
}

/// Opens the database at `config.url` and runs migrations.
///
/// `url` is a `postgres://` or `postgresql://` database URL. Pool size and TLS
/// stay in that URL's query. A filesystem path is rejected.
///
/// # Examples
///
/// ```no_run
/// use store_postgres::{OpenConfig, open};
///
/// # async fn demo() -> Result<(), store_postgres::StoreError> {
/// let store = open(&OpenConfig::new("postgres://localhost/widgets")).await?;
/// let row = store_postgres::insert_widget(&store, "bolt").await?;
/// let loaded = store_postgres::get_widget(&store, &row.id).await?;
/// assert!(loaded.is_some());
/// # Ok(())
/// # }
/// ```
///
/// # Errors
///
/// Returns [`StoreCode::Connection`] when `url` is not a Postgres URL or a
/// pool cannot be opened, [`StoreCode::Unavailable`] when a pool acquire
/// times out, and [`StoreCode::Migration`] when a migration fails.
pub async fn open(config: &OpenConfig) -> Result<Store, StoreError> {
    require_postgres_url(&config.url)?;
    let pool = PgPoolOptions::new()
        .max_connections(POOL_SIZE)
        .acquire_timeout(ACQUIRE_TIMEOUT)
        .connect(&config.url)
        .await
        .map_err(|err| from_driver(StoreCode::Connection, err))?;
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .map_err(|err| from_driver(StoreCode::Migration, err.into()))?;
    Ok(Store { pool })
}

/// Inserts one widget and returns the stored row.
///
/// Duplicate names are allowed. The identifier comes from Postgres
/// `gen_random_uuid()`.
///
/// # Errors
///
/// Returns [`StoreCode::Query`] when the insert fails, [`StoreCode::Conflict`]
/// when a constraint rejects it, and [`StoreCode::Unavailable`] when the pool
/// times out.
pub async fn insert_widget(store: &Store, name: &str) -> Result<WidgetRow, StoreError> {
    let row = sqlx::query!(
        "INSERT INTO widgets (id, name) VALUES (gen_random_uuid()::text, $1) RETURNING id, name",
        name,
    )
    .fetch_one(&store.pool)
    .await
    .map_err(|err| from_driver(StoreCode::Query, err))?;
    Ok(WidgetRow {
        id: WidgetId::from_column(row.id),
        name: row.name,
    })
}

/// Reads one widget by id.
///
/// `id` is a [`WidgetId`] or any other column text, including a value the
/// application newtype already validated.
///
/// # Errors
///
/// Returns [`StoreCode::Query`] when the query fails, and
/// [`StoreCode::Unavailable`] when the pool times out.
pub async fn get_widget(
    store: &Store,
    id: impl AsRef<str>,
) -> Result<Option<WidgetRow>, StoreError> {
    let id = id.as_ref();
    let row = sqlx::query!("SELECT id, name FROM widgets WHERE id = $1", id)
        .fetch_optional(&store.pool)
        .await
        .map_err(|err| from_driver(StoreCode::Query, err))?;
    Ok(row.map(|row| WidgetRow {
        id: WidgetId::from_column(row.id),
        name: row.name,
    }))
}

async fn read_table_rows(store: &Store, table: &str) -> Result<Vec<RowRecord>, StoreError> {
    known_table(table)?;
    read_widget_rows(store).await
}

async fn read_widget_rows(store: &Store) -> Result<Vec<RowRecord>, StoreError> {
    let mut fetched = sqlx::query!("SELECT id, name FROM widgets").fetch(&store.pool);
    let mut rows = Vec::new();
    while let Some(row) = fetched
        .try_next()
        .await
        .map_err(|err| from_driver(StoreCode::Query, err))?
    {
        check_read_limit(WIDGETS, rows.len() + 1)?;
        rows.push(record_from_columns(row.id, row.name)?);
    }
    Ok(rows)
}

async fn write_table_rows(
    store: &Store,
    table: &str,
    rows: &[RowRecord],
) -> Result<(), StoreError> {
    known_table(table)?;
    check_write_limit(rows)?;
    write_widget_rows(store, rows).await
}

async fn write_widget_rows(store: &Store, rows: &[RowRecord]) -> Result<(), StoreError> {
    let mut tx = store
        .pool
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
        sqlx::query!("INSERT INTO widgets (id, name) VALUES ($1, $2)", id, name)
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
        table: WIDGETS.to_owned(),
        payload,
    })
}

fn widget_from_payload(payload: &[u8]) -> Result<WidgetRow, StoreError> {
    serde_json::from_slice(payload)
        .map_err(|err| StoreError::new(StoreCode::Query, err.to_string()))
}

/// Counts widgets.
///
/// # Errors
///
/// Returns [`StoreCode::Query`] when the query fails.
#[cfg(test)]
async fn count_widgets(store: &Store) -> Result<i64, StoreError> {
    let count = sqlx::query_scalar!(r#"SELECT COUNT(*) as "count!: i64" FROM widgets"#)
        .fetch_one(&store.pool)
        .await
        .map_err(|err| from_driver(StoreCode::Query, err))?;
    Ok(count)
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicU64, Ordering};

    use super::*;
    use schema::MAX_PAIRING_ROWS;
    use schema::MAX_ROW_PAYLOAD_BYTES;
    use sqlx::postgres::PgPoolOptions;

    fn configured_url() -> Option<String> {
        std::env::var("POSTGRES_DATABASE_URL")
            .ok()
            .filter(|url| !url.is_empty())
    }

    fn replace_database(url: &str, database: &str) -> String {
        let (base, query) = match url.split_once('?') {
            Some((base, query)) => (base, Some(query)),
            None => (url, None),
        };
        let scheme_end = base.find("://").map_or(0, |index| index + 3);
        let mut out = match base[scheme_end..].find('/') {
            Some(slash) => format!("{}/{}", &base[..scheme_end + slash], database),
            None => format!("{base}/{database}"),
        };
        if let Some(query) = query {
            out.push('?');
            out.push_str(query);
        }
        out
    }

    fn fresh_database_name() -> String {
        static NEXT: AtomicU64 = AtomicU64::new(1);
        let n = NEXT.fetch_add(1, Ordering::Relaxed);
        format!("b5_{}_{n}", std::process::id())
    }

    struct Ephemeral {
        admin: PgPool,
        name: String,
        url: String,
    }

    impl Ephemeral {
        async fn create() -> Option<Self> {
            let url = configured_url()?;
            let admin = PgPoolOptions::new()
                .max_connections(1)
                .acquire_timeout(ACQUIRE_TIMEOUT)
                .connect(&replace_database(&url, "postgres"))
                .await
                .expect("admin connection");
            let name = fresh_database_name();
            // The name is generated here and checked as an identifier. Postgres
            // cannot bind a database name as a parameter.
            sqlx::query(sqlx::AssertSqlSafe(statement_for_ident(
                "CREATE DATABASE",
                &name,
            )))
            .execute(&admin)
            .await
            .expect("create database");
            Some(Self {
                admin,
                url: replace_database(&url, &name),
                name,
            })
        }

        async fn close(self) {
            sqlx::query(sqlx::AssertSqlSafe(statement_for_ident(
                "DROP DATABASE",
                &self.name,
            )))
            .execute(&self.admin)
            .await
            .expect("drop database");
        }
    }

    fn statement_for_ident(statement: &str, name: &str) -> String {
        assert!(
            !name.is_empty()
                && name.bytes().all(|byte| {
                    byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_'
                }),
            "generated database name is an identifier"
        );
        if statement == "DROP DATABASE" {
            format!("{statement} \"{name}\" WITH (FORCE)")
        } else {
            format!("{statement} \"{name}\"")
        }
    }

    async fn live_database() -> Option<Ephemeral> {
        if configured_url().is_none() {
            eprintln!("skipping: POSTGRES_DATABASE_URL is unset");
            return None;
        }
        Ephemeral::create().await
    }

    #[tokio::test]
    async fn open_rejects_a_non_postgres_url() {
        for url in ["widgets.db", "", "mysql://localhost/widgets"] {
            let err = open(&OpenConfig::new(url)).await.expect_err("open");
            assert_eq!(err.code, StoreCode::Connection);
            assert!(err.to_string().starts_with("Connection:"));
            assert!(std::error::Error::source(&err).is_none());
            assert!(!err.is_retryable());
        }
    }

    #[tokio::test]
    async fn open_failure_uses_connection_code() {
        for url in [
            "postgres://localhost:not-a-port/widgets",
            "postgresql://localhost:not-a-port/widgets",
        ] {
            let err = open(&OpenConfig::new(url)).await.expect_err("open");
            assert_eq!(err.code, StoreCode::Connection, "{url}");
            assert!(err.to_string().starts_with("Connection:"));
            assert!(std::error::Error::source(&err).is_none());
            assert!(!err.is_retryable());
        }
    }

    #[tokio::test]
    async fn empty_database_migrates() {
        let Some(db) = live_database().await else {
            return;
        };
        let store = open(&OpenConfig::new(db.url.clone())).await.expect("open");
        assert_eq!(count_widgets(&store).await.expect("count"), 0);
        drop(store);
        db.close().await;
    }

    #[tokio::test]
    async fn concurrent_inserts_do_not_lock() {
        let Some(db) = live_database().await else {
            return;
        };
        let store = open(&OpenConfig::new(db.url.clone())).await.expect("open");
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
        drop(store);
        db.close().await;
    }

    #[tokio::test]
    async fn engine_store_round_trips_widget_rows() {
        let Some(db) = live_database().await else {
            return;
        };
        let store = <Store as EngineStore>::open(&OpenConfig::new(db.url.clone()))
            .await
            .expect("trait open");
        let written = RowRecord {
            table: WIDGETS.to_owned(),
            payload: br#"{"id":"abc","name":"bolt"}"#.to_vec(),
        };
        store.write_rows(WIDGETS, &[written]).await.expect("write");
        let rows = store.read_rows(WIDGETS).await.expect("read");
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
        drop(store);
        db.close().await;
    }

    #[tokio::test]
    async fn failed_write_rolls_back() {
        let Some(db) = live_database().await else {
            return;
        };
        let store = open(&OpenConfig::new(db.url.clone())).await.expect("open");
        let good = RowRecord {
            table: WIDGETS.to_owned(),
            payload: br#"{"id":"abc","name":"bolt"}"#.to_vec(),
        };
        let bad = RowRecord {
            table: WIDGETS.to_owned(),
            payload: b"not-json".to_vec(),
        };
        let err = store
            .write_rows(WIDGETS, &[good, bad])
            .await
            .expect_err("bad payload");
        assert_eq!(err.code, StoreCode::Query);
        assert!(err.to_string().starts_with("Query:"));
        assert!(std::error::Error::source(&err).is_none());
        assert!(!err.is_retryable());
        let rows = store.read_rows(WIDGETS).await.expect("read");
        assert!(rows.is_empty());
        drop(store);
        db.close().await;
    }

    #[tokio::test]
    async fn duplicate_id_is_a_conflict() {
        let Some(db) = live_database().await else {
            return;
        };
        let store = open(&OpenConfig::new(db.url.clone())).await.expect("open");
        let row = RowRecord {
            table: WIDGETS.to_owned(),
            payload: br#"{"id":"abc","name":"bolt"}"#.to_vec(),
        };
        store
            .write_rows(WIDGETS, std::slice::from_ref(&row))
            .await
            .expect("first");
        let err = store
            .write_rows(WIDGETS, &[row])
            .await
            .expect_err("duplicate");
        assert_eq!(err.code, StoreCode::Conflict);
        assert!(!err.is_retryable());
        assert!(!err.remediation().is_empty());
        drop(store);
        db.close().await;
    }

    #[tokio::test]
    async fn unknown_table_is_not_found() {
        let Some(db) = live_database().await else {
            return;
        };
        let store = open(&OpenConfig::new(db.url.clone())).await.expect("open");
        let err = store.read_rows("missing").await.expect_err("missing");
        assert_eq!(err.code, StoreCode::NotFound);
        let err = store
            .write_rows("missing", &[])
            .await
            .expect_err("missing write");
        assert_eq!(err.code, StoreCode::NotFound);
        drop(store);
        db.close().await;
    }

    #[tokio::test]
    async fn oversized_batch_and_payload_are_rejected() {
        let Some(db) = live_database().await else {
            return;
        };
        let store = open(&OpenConfig::new(db.url.clone())).await.expect("open");
        let too_many = vec![
            RowRecord {
                table: WIDGETS.to_owned(),
                payload: br#"{"id":"abc","name":"bolt"}"#.to_vec(),
            };
            MAX_PAIRING_ROWS + 1
        ];
        let err = store
            .write_rows(WIDGETS, &too_many)
            .await
            .expect_err("batch");
        assert_eq!(err.code, StoreCode::Query);
        assert_eq!(count_widgets(&store).await.expect("count"), 0);

        let huge = RowRecord {
            table: WIDGETS.to_owned(),
            payload: vec![b'x'; MAX_ROW_PAYLOAD_BYTES + 1],
        };
        let err = store
            .write_rows(WIDGETS, &[huge])
            .await
            .expect_err("payload");
        assert_eq!(err.code, StoreCode::Query);
        assert_eq!(count_widgets(&store).await.expect("count"), 0);
        drop(store);
        db.close().await;
    }
}
