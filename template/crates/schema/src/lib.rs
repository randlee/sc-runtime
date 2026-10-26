//! Row types and the shared [`EngineStore`] trait for paired database engines.
//!
//! Each engine crate keeps that engine's statements. This crate holds the row
//! structs, the table membership description, and the trait the generated
//! stores implement. A consumer may still declare a different trait per store.
//! `EngineStore` is used with static dispatch.
//!
//! [`EngineStore`], [`StoreError`], and [`OpenConfig`] are consumer-owned
//! copies generated into the project. The trait shape is the only
//! cross-project contract. Promoting them into a published crate is a Phase C
//! question, not a b-4 change.
//!
//! `EngineStore` is the adaptor seam (`read_rows` and `write_rows` only).
//! Application operations such as insert and get stay free functions on the
//! concrete store. Adding Postgres means writing those functions again in
//! `store-postgres` and pointing `service` at them. A project that picks an
//! engine at runtime can use an enum that delegates to the concrete stores.
//! `dyn EngineStore` is not a supported form.
//!
//! # Examples
//!
//! ```
//! use schema::{EngineStore, OpenConfig, RowRecord, StoreError};
//!
//! struct MemoryStore {
//!     rows: Vec<RowRecord>,
//! }
//!
//! impl EngineStore for MemoryStore {
//!     fn open(
//!         cfg: &OpenConfig,
//!     ) -> impl std::future::Future<Output = Result<Self, StoreError>> + Send {
//!         let _ = cfg.url.clone();
//!         async move { Ok(Self { rows: Vec::new() }) }
//!     }
//!
//!     fn read_rows(
//!         &self,
//!         table: &str,
//!     ) -> impl std::future::Future<Output = Result<Vec<RowRecord>, StoreError>> + Send {
//!         let rows = self
//!             .rows
//!             .iter()
//!             .filter(|row| row.table == table)
//!             .cloned()
//!             .collect();
//!         async move { Ok(rows) }
//!     }
//!
//!     fn write_rows(
//!         &self,
//!         table: &str,
//!         rows: &[RowRecord],
//!     ) -> impl std::future::Future<Output = Result<(), StoreError>> + Send {
//!         let _ = (table, rows);
//!         async move { Ok(()) }
//!     }
//! }
//! ```
//!
//! A type that does not implement [`EngineStore`] cannot be used as one:
//!
//! ```compile_fail
//! use schema::EngineStore;
//!
//! struct PathStore;
//!
//! fn require_engine_store<T: EngineStore>() {}
//!
//! fn check() {
//!     require_engine_store::<PathStore>();
//! }
//! ```

mod membership;

pub use membership::{EngineMembership, Membership, MembershipError};

/// Most rows one [`EngineStore::read_rows`] or [`EngineStore::write_rows`] call may move.
pub const MAX_PAIRING_ROWS: usize = 10_000;

/// Largest [`RowRecord::payload`] [`EngineStore::write_rows`] accepts.
pub const MAX_ROW_PAYLOAD_BYTES: usize = 1024 * 1024;

/// How to open one engine's database.
///
/// ADR-RUN-0301 fixes this as one public `url` string. Engines do not grow a
/// second config type in this phase.
///
/// * SQLite reads a filesystem path from `url` (for example `widgets.db`).
///   This template does not parse a `sqlite:` URL.
/// * Postgres reads a database URL (`postgres://` or `postgresql://`).
/// * MySQL reads a database URL (`mysql://`).
///
/// Pool size, TLS, and other engine options belong in that URL's query until
/// a second field is justified. Do not put a filesystem path in a Postgres or
/// MySQL config, or a database URL in the SQLite config.
///
/// The struct is [`non_exhaustive`]. Crates other than this one construct it
/// with [`OpenConfig::new`], which sets `url`.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct OpenConfig {
    /// Connection target. A filesystem path for SQLite, a database URL for
    /// Postgres and MySQL.
    pub url: String,
}

impl OpenConfig {
    /// Builds a config whose `url` is `target`.
    ///
    /// `target` is a filesystem path for SQLite and a database URL for
    /// Postgres and MySQL.
    #[must_use]
    pub fn new(target: impl Into<String>) -> Self {
        Self { url: target.into() }
    }
}

/// Which store operation failed.
///
/// The set is closed for this phase and marked [`non_exhaustive`] so a later
/// engine crate can add a variant without breaking matches in this crate's
/// callers. Match with a wildcard outside this crate.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum StoreCode {
    /// Opening the database or a pooled connection failed.
    Connection,
    /// Reading or writing rows failed, including a batch or payload over the limit.
    Query,
    /// Applying migrations failed.
    Migration,
    /// The named table is not stored by this engine.
    NotFound,
    /// A constraint rejected the write.
    Conflict,
    /// The pool or engine was temporarily unavailable. This variant is retryable.
    Unavailable,
}

impl StoreCode {
    /// What the caller should do next.
    #[must_use]
    pub fn remediation(self) -> &'static str {
        match self {
            Self::Connection => {
                "check the database path or URL, credentials, permissions, and free disk"
            }
            Self::Query => {
                "fix the table, the payload, or the batch size; do not retry the same request"
            }
            Self::Migration => {
                "repair the migration files; do not retry a failed migration unchanged"
            }
            Self::NotFound => {
                "use a table this engine stores; one-sided tables are named by membership"
            }
            Self::Conflict => {
                "change the row so it meets the table constraints; do not retry the same write"
            }
            Self::Unavailable => {
                "retry after a short wait; the pool or engine was temporarily unavailable"
            }
        }
    }

    /// Whether the same call can succeed later without a change to its inputs.
    #[must_use]
    pub fn is_retryable(self) -> bool {
        matches!(self, Self::Unavailable)
    }
}

/// Failure returned by an engine store.
///
/// The engine maps its driver error into [`StoreCode`] and `message` before
/// it returns. `message` is that driver text, so the caller can act on the
/// code and still read the cause. This type does not wrap the driver value:
/// the schema crate must not name sqlx or rusqlite, so
/// [`Error::source`](std::error::Error::source) is absent.
///
/// # Examples
///
/// ```
/// use schema::{StoreCode, StoreError};
///
/// let err = StoreError::new(StoreCode::Unavailable, "sqlite pool acquire timed out");
/// assert!(err.is_retryable());
/// assert!(!err.remediation().is_empty());
/// assert!(std::error::Error::source(&err).is_none());
/// ```
#[derive(Debug, PartialEq, Eq)]
pub struct StoreError {
    /// Kind of failure.
    pub code: StoreCode,
    /// Engine message, with the driver error already rendered into text.
    pub message: String,
}

impl StoreError {
    /// Builds an error whose driver cause is already rendered into `message`.
    #[must_use]
    pub fn new(code: StoreCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }

    /// Recovery text for [`StoreError::code`].
    #[must_use]
    pub fn remediation(&self) -> &'static str {
        self.code.remediation()
    }

    /// Whether [`StoreError::code`] is retryable.
    #[must_use]
    pub fn is_retryable(&self) -> bool {
        self.code.is_retryable()
    }
}

impl std::fmt::Display for StoreError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{:?}: {}", self.code, self.message)
    }
}

impl std::error::Error for StoreError {}

/// One row moving between stores.
///
/// # Payload encoding
///
/// [`RowRecord::payload`] is a UTF-8 JSON object. Keys are column names.
/// Every engine uses this encoding so a row copied to another engine does not
/// pick a second shape:
///
/// | Column value | JSON value |
/// | --- | --- |
/// | SQL NULL | `null` |
/// | boolean | `true` or `false` |
/// | integer in `-(2^53)+1 ..= 2^53-1` | JSON number |
/// | integer outside that range | decimal string |
/// | text | JSON string |
/// | bytes | standard base64 string, no whitespace |
/// | timestamp | RFC 3339 string |
///
/// The widget example is text-only (`id` and `name` strings), which is this
/// table's text row. Cross-engine copy of other column types is still this
/// encoding; the adaptor does not invent another one.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RowRecord {
    /// Table this row belongs to.
    pub table: String,
    /// UTF-8 JSON object. Keys are column names. See the encoding table above.
    pub payload: Vec<u8>,
}

/// Opaque widget identifier stored in the `widgets.id` column.
///
/// This is the store-boundary newtype. `api-types` has its own `WidgetId`
/// that validates application input. Construct this value from a database
/// column or from text the caller already validated.
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(transparent)]
pub struct WidgetId(String);

impl WidgetId {
    /// Wraps a database-returned identifier without re-validating it.
    #[must_use]
    pub fn from_column(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    /// Column text.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl AsRef<str> for WidgetId {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl std::ops::Deref for WidgetId {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::fmt::Display for WidgetId {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.0)
    }
}

/// One widget row.
///
/// `id` is the column text wrapped as [`WidgetId`], not a bare [`String`], so
/// a widget identifier is not interchangeable with a name at this boundary.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct WidgetRow {
    /// Store-assigned identifier.
    pub id: WidgetId,
    /// Stored name.
    pub name: String,
}

/// Rejects a [`EngineStore::write_rows`] batch that is over the pairing limits.
///
/// # Errors
///
/// Returns [`StoreCode::Query`] when `rows` is longer than [`MAX_PAIRING_ROWS`]
/// or one payload is longer than [`MAX_ROW_PAYLOAD_BYTES`].
pub fn check_write_limit(rows: &[RowRecord]) -> Result<(), StoreError> {
    if rows.len() > MAX_PAIRING_ROWS {
        return Err(StoreError::new(
            StoreCode::Query,
            format!(
                "batch of {} rows exceeds the limit of {MAX_PAIRING_ROWS}",
                rows.len()
            ),
        ));
    }
    for row in rows {
        if row.payload.len() > MAX_ROW_PAYLOAD_BYTES {
            return Err(StoreError::new(
                StoreCode::Query,
                format!(
                    "row payload of {} bytes exceeds the limit of {MAX_ROW_PAYLOAD_BYTES}",
                    row.payload.len()
                ),
            ));
        }
    }
    Ok(())
}

/// Rejects a [`EngineStore::read_rows`] result that is over [`MAX_PAIRING_ROWS`].
///
/// `rows` is the number of rows already read, including the one about to be kept.
///
/// # Errors
///
/// Returns [`StoreCode::Query`] when `rows` is greater than [`MAX_PAIRING_ROWS`].
pub fn check_read_limit(table: &str, rows: usize) -> Result<(), StoreError> {
    if rows > MAX_PAIRING_ROWS {
        return Err(StoreError::new(
            StoreCode::Query,
            format!("table {table} exceeds the limit of {MAX_PAIRING_ROWS} rows"),
        ));
    }
    Ok(())
}

/// Shared store operations for a paired engine.
///
/// Generated stores implement this trait. This is the adaptor seam, not the
/// application query API. Consumer code may still use a different trait per
/// store. `dyn EngineStore` is not a supported form.
///
/// Implementations call [`check_read_limit`] and [`check_write_limit`] so a
/// pairing copy cannot load or write an unbounded batch.
pub trait EngineStore: Send + Sync {
    /// Opens the database described by `cfg`.
    ///
    /// # Errors
    ///
    /// Returns [`StoreError`] with [`StoreCode::Connection`] when the database
    /// cannot be opened, [`StoreCode::Unavailable`] when a pool acquire times
    /// out, and [`StoreCode::Migration`] when migrations fail.
    fn open(cfg: &OpenConfig) -> impl std::future::Future<Output = Result<Self, StoreError>> + Send
    where
        Self: Sized;

    /// Reads rows of `table`, stopping at [`MAX_PAIRING_ROWS`].
    ///
    /// Each [`RowRecord::payload`] follows the encoding table on [`RowRecord`].
    ///
    /// # Errors
    ///
    /// Returns [`StoreError`] with [`StoreCode::NotFound`] when this engine
    /// does not store `table`, and [`StoreCode::Query`] when the read fails
    /// or the table is over the row limit.
    fn read_rows(
        &self,
        table: &str,
    ) -> impl std::future::Future<Output = Result<Vec<RowRecord>, StoreError>> + Send;

    /// Writes `rows` into `table`.
    ///
    /// The batch must be at most [`MAX_PAIRING_ROWS`] long, and each payload
    /// at most [`MAX_ROW_PAYLOAD_BYTES`].
    ///
    /// # Errors
    ///
    /// Returns [`StoreError`] with [`StoreCode::Query`] when a row cannot be
    /// written or a limit is exceeded, [`StoreCode::Conflict`] when a
    /// constraint rejects a row, and [`StoreCode::NotFound`] when this engine
    /// does not store `table`.
    fn write_rows(
        &self,
        table: &str,
        rows: &[RowRecord],
    ) -> impl std::future::Future<Output = Result<(), StoreError>> + Send;
}
