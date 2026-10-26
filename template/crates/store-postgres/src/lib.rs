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
//! it. There is no write actor and no queue. TLS settings such as `sslmode`
//! belong in the database URL. Pool size and acquire timeout are the constants
//! in this crate.
//!
//! Live tests read `POSTGRES_DATABASE_URL` and skip when it is unset.
//! `REQUIRE_POSTGRES=1` makes those tests panic instead of skipping. A CI job
//! that provisions Postgres sets that variable. Sibling engine crates copy the
//! same switch, `REQUIRE_<ENGINE>=1`.

use std::time::Duration;

use futures_util::TryStreamExt;
use schema::{EngineStore, MAX_ROW_PAYLOAD_BYTES, RowRecord, check_read_limit, check_write_limit};
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

// The table check and RowRecord codec match store-sqlite. They stay in this
// crate until sprint b-6, the pairing owner, can move them into the schema
// crate. This sprint's owned path is template/crates/store-postgres/ only.
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

/// SQLSTATE values that can succeed later with the same inputs.
///
/// `40001` is serialization_failure, `40P01` is deadlock_detected, and
/// `57P03` is cannot_connect_now.
const TRANSIENT_SQLSTATES: &[&str] = &["40001", "40P01", "57P03"];

fn transient_driver(err: &sqlx::Error) -> bool {
    matches!(
        err,
        sqlx::Error::PoolTimedOut | sqlx::Error::PoolClosed | sqlx::Error::Io(_)
    ) || err
        .as_database_error()
        .and_then(sqlx::error::DatabaseError::code)
        .is_some_and(|code| TRANSIENT_SQLSTATES.contains(&code.as_ref()))
}

fn constraint_violation(err: &sqlx::Error) -> bool {
    err.as_database_error().is_some_and(|db| {
        db.is_unique_violation() || db.is_foreign_key_violation() || db.is_check_violation()
    })
}

fn driver_message(err: &sqlx::Error) -> String {
    redact_credentials(&err.to_string())
}

/// Replaces a URL password so a driver message cannot echo credentials.
fn redact_credentials(message: &str) -> String {
    let mut rest = message;
    let mut out = String::new();
    while let Some(scheme) = rest.find("://") {
        let split = scheme + 3;
        out.push_str(&rest[..split]);
        rest = &rest[split..];
        let end = rest
            .find(['/', ' ', '\n', '\r', '\t'])
            .unwrap_or(rest.len());
        out.push_str(&redact_authority(&rest[..end]));
        rest = &rest[end..];
    }
    out.push_str(rest);
    out
}

fn redact_authority(authority: &str) -> String {
    let Some(at) = authority.rfind('@') else {
        return authority.to_owned();
    };
    let userinfo = &authority[..at];
    let Some(colon) = userinfo.find(':') else {
        return authority.to_owned();
    };
    let mut redacted = String::with_capacity(authority.len());
    redacted.push_str(&userinfo[..colon]);
    redacted.push_str(":***@");
    redacted.push_str(&authority[at + 1..]);
    redacted
}

fn from_driver(fallback: StoreCode, err: sqlx::Error) -> StoreError {
    if matches!(err, sqlx::Error::PoolTimedOut) {
        return StoreError::new(StoreCode::Unavailable, "postgres pool acquire timed out");
    }
    if transient_driver(&err) {
        return StoreError::new(StoreCode::Unavailable, driver_message(&err));
    }
    if constraint_violation(&err) {
        return StoreError::new(StoreCode::Conflict, driver_message(&err));
    }
    StoreError::new(fallback, driver_message(&err))
}

/// Opens the database at `config.url` and runs migrations.
///
/// `url` is a `postgres://` or `postgresql://` database URL. TLS settings such
/// as `sslmode` belong in that URL's query. Pool size ([`POOL_SIZE`]) and the
/// acquire timeout ([`ACQUIRE_TIMEOUT`]) are constants in this consumer-owned
/// crate, not URL parameters. A filesystem path is rejected.
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
/// times out, the pool is closed, the driver reports an I/O failure, or
/// Postgres returns SQLSTATE `40001`, `40P01`, or `57P03`, and
/// [`StoreCode::Migration`] when a migration fails.
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
/// times out, the pool is closed, the driver reports an I/O failure, or
/// Postgres returns SQLSTATE `40001`, `40P01`, or `57P03`.
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
/// [`StoreCode::Unavailable`] when the pool times out, the pool is closed,
/// the driver reports an I/O failure, or Postgres returns SQLSTATE `40001`,
/// `40P01`, or `57P03`.
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
    // REQ-RUN-0308 / ADR-RUN-0301: read_rows rejects a payload over 1 MiB.
    if payload.len() > MAX_ROW_PAYLOAD_BYTES {
        return Err(StoreError::new(
            StoreCode::Query,
            format!(
                "row payload of {} bytes exceeds the limit of {MAX_ROW_PAYLOAD_BYTES}",
                payload.len()
            ),
        ));
    }
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

    fn postgres_required() -> bool {
        std::env::var("REQUIRE_POSTGRES")
            .ok()
            .is_some_and(|value| value == "1")
    }

    /// `Ok(true)` runs the live test, `Ok(false)` skips it, and `Err` is the
    /// panic text when `REQUIRE_POSTGRES=1` and the URL is missing.
    fn live_database_gate(url: Option<&str>, required: bool) -> Result<bool, &'static str> {
        let present = url.is_some_and(|value| !value.is_empty());
        if present {
            Ok(true)
        } else if required {
            Err("REQUIRE_POSTGRES=1 but POSTGRES_DATABASE_URL is unset")
        } else {
            Ok(false)
        }
    }

    fn enforce_live_database(gate: Result<bool, &'static str>) -> bool {
        match gate {
            Ok(run) => run,
            Err(message) => panic!("{message}"),
        }
    }

    async fn live_database() -> Option<Ephemeral> {
        let url = configured_url();
        if !enforce_live_database(live_database_gate(url.as_deref(), postgres_required())) {
            eprintln!(
                "skipping: POSTGRES_DATABASE_URL is unset; set it to run live tests, or REQUIRE_POSTGRES=1 to fail instead of skipping"
            );
            return None;
        }
        Ephemeral::create().await
    }

    fn is_uuid(value: &str) -> bool {
        let bytes = value.as_bytes();
        if bytes.len() != 36 {
            return false;
        }
        bytes.iter().enumerate().all(|(index, byte)| match index {
            8 | 13 | 18 | 23 => *byte == b'-',
            _ => byte.is_ascii_hexdigit(),
        })
    }

    #[derive(Debug)]
    struct SqlStateError(&'static str);

    impl std::fmt::Display for SqlStateError {
        fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            formatter.write_str(self.0)
        }
    }

    impl std::error::Error for SqlStateError {}

    impl sqlx::error::DatabaseError for SqlStateError {
        fn message(&self) -> &str {
            self.0
        }

        fn code(&self) -> Option<std::borrow::Cow<'_, str>> {
            Some(std::borrow::Cow::Borrowed(self.0))
        }

        fn kind(&self) -> sqlx::error::ErrorKind {
            sqlx::error::ErrorKind::Other
        }

        fn as_error(&self) -> &(dyn std::error::Error + Send + Sync + 'static) {
            self
        }

        fn as_error_mut(&mut self) -> &mut (dyn std::error::Error + Send + Sync + 'static) {
            self
        }

        fn into_error(self: Box<Self>) -> Box<dyn std::error::Error + Send + Sync + 'static> {
            self
        }
    }

    #[test]
    fn read_rows_rejects_payload_over_one_mib() {
        let name = "n".repeat(MAX_ROW_PAYLOAD_BYTES);
        let err = record_from_columns("id".to_owned(), name).expect_err("payload");
        assert_eq!(err.code, StoreCode::Query);
        assert!(!err.is_retryable());
        assert!(err.message.contains("exceeds the limit"));

        let record = record_from_columns("abc".to_owned(), "bolt".to_owned()).expect("record");
        assert!(record.payload.len() <= MAX_ROW_PAYLOAD_BYTES);
        assert_eq!(record.table, WIDGETS);
    }

    #[test]
    fn transient_driver_errors_are_unavailable() {
        let cases = [
            ("pool closed", sqlx::Error::PoolClosed),
            (
                "io",
                sqlx::Error::Io(std::io::Error::other("connection reset")),
            ),
            ("40001", sqlx::Error::from(SqlStateError("40001"))),
            ("40P01", sqlx::Error::from(SqlStateError("40P01"))),
            ("57P03", sqlx::Error::from(SqlStateError("57P03"))),
        ];
        for (label, err) in cases {
            let mapped = from_driver(StoreCode::Query, err);
            assert_eq!(mapped.code, StoreCode::Unavailable, "{label}");
            assert!(mapped.is_retryable(), "{label}");
            assert!(std::error::Error::source(&mapped).is_none(), "{label}");
        }

        let timed_out = from_driver(StoreCode::Query, sqlx::Error::PoolTimedOut);
        assert_eq!(timed_out.code, StoreCode::Unavailable);
        assert!(timed_out.is_retryable());
        assert_eq!(timed_out.message, "postgres pool acquire timed out");
    }

    #[test]
    fn driver_message_redacts_url_credentials() {
        let err = sqlx::Error::Io(std::io::Error::other(
            "connect postgres://widget_user:super-secret-password@127.0.0.1:5432/widgets failed",
        ));
        let mapped = from_driver(StoreCode::Connection, err);
        assert!(!mapped.message.contains("super-secret-password"));
        assert!(mapped.message.contains("widget_user"));
        assert!(mapped.message.contains("127.0.0.1"));
        assert!(!mapped.to_string().contains("super-secret-password"));
    }

    #[test]
    fn require_postgres_skips_unless_forced() {
        assert!(!live_database_gate(None, false).expect("skip"));
        assert!(live_database_gate(Some("postgres://localhost/widgets"), false).expect("present"));
        assert_eq!(
            live_database_gate(Some(""), true).expect_err("empty"),
            "REQUIRE_POSTGRES=1 but POSTGRES_DATABASE_URL is unset"
        );
    }

    #[test]
    #[should_panic(expected = "REQUIRE_POSTGRES=1 but POSTGRES_DATABASE_URL is unset")]
    fn require_postgres_panics_when_the_url_is_unset() {
        enforce_live_database(live_database_gate(None, true));
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
    async fn open_does_not_echo_url_credentials() {
        let err = open(&OpenConfig::new(
            "postgres://widget_user:super-secret-password@localhost:not-a-port/widgets",
        ))
        .await
        .expect_err("open");
        assert_eq!(err.code, StoreCode::Connection);
        assert!(!err.message.contains("super-secret-password"));
        assert!(!err.to_string().contains("super-secret-password"));
        assert!(!err.is_retryable());
    }

    #[tokio::test]
    async fn fresh_database_provisions_gen_random_uuid() {
        let Some(db) = live_database().await else {
            return;
        };
        let store = open(&OpenConfig::new(db.url.clone())).await.expect("open");
        let installed: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM pg_extension WHERE extname = 'pgcrypto')",
        )
        .fetch_one(&store.pool)
        .await
        .expect("pgcrypto");
        assert!(installed, "migration installs pgcrypto");
        let row = insert_widget(&store, "bolt").await.expect("insert");
        assert!(is_uuid(row.id.as_str()), "{}", row.id.as_str());
        let loaded = get_widget(&store, &row.id).await.expect("get");
        assert_eq!(loaded.expect("row").name, "bolt");
        drop(store);
        db.close().await;
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
