//! Row types and the shared [`EngineStore`] trait for paired database engines.
//!
//! Each engine crate keeps that engine's statements. This crate holds the row
//! structs, the table membership description, and the trait the generated
//! stores implement. A consumer may still declare a different trait per store.
//! `EngineStore` is used with static dispatch.
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

pub use membership::{Membership, MembershipError};

/// How to open one engine's database.
///
/// `url` is a file path for SQLite and a database URL for Postgres and MySQL.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OpenConfig {
    /// Database string for this engine.
    pub url: String,
}

/// Which store operation failed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StoreCode {
    /// Opening the database or a pooled connection failed.
    Connection,
    /// Reading or writing rows failed.
    Query,
    /// Applying migrations failed.
    Migration,
}

/// Failure returned by an engine store.
///
/// The engine maps its driver error into [`StoreCode`] and `message` before
/// it returns. This type does not wrap that driver error, so
/// [`Error::source`](std::error::Error::source) is absent.
#[derive(Debug, PartialEq, Eq)]
pub struct StoreError {
    /// Kind of failure.
    pub code: StoreCode,
    /// Engine message, with the driver error already rendered into text.
    pub message: String,
}

impl std::fmt::Display for StoreError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{:?}: {}", self.code, self.message)
    }
}

impl std::error::Error for StoreError {}

/// One row moving between stores.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RowRecord {
    /// Table this row belongs to.
    pub table: String,
    /// UTF-8 JSON object. Keys are column names.
    pub payload: Vec<u8>,
}

/// One widget row.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct WidgetRow {
    /// Store-assigned identifier.
    pub id: String,
    /// Stored name.
    pub name: String,
}

/// Shared store operations for a paired engine.
///
/// Generated stores implement this trait. Consumer code may still use a
/// different trait per store. `dyn EngineStore` is not a supported form.
pub trait EngineStore: Send + Sync {
    /// Opens the database described by `cfg`.
    ///
    /// # Errors
    ///
    /// Returns [`StoreError`] with [`StoreCode::Connection`] when the database
    /// cannot be opened, and [`StoreCode::Migration`] when migrations fail.
    fn open(cfg: &OpenConfig) -> impl std::future::Future<Output = Result<Self, StoreError>> + Send
    where
        Self: Sized;

    /// Reads every row of `table`.
    ///
    /// Each [`RowRecord::payload`] is a UTF-8 JSON object whose keys are
    /// column names.
    ///
    /// # Errors
    ///
    /// Returns [`StoreError`] with [`StoreCode::Query`] when the read fails.
    fn read_rows(
        &self,
        table: &str,
    ) -> impl std::future::Future<Output = Result<Vec<RowRecord>, StoreError>> + Send;

    /// Writes `rows` into `table`.
    ///
    /// # Errors
    ///
    /// Returns [`StoreError`] with [`StoreCode::Query`] when a row cannot be
    /// written.
    fn write_rows(
        &self,
        table: &str,
        rows: &[RowRecord],
    ) -> impl std::future::Future<Output = Result<(), StoreError>> + Send;
}
