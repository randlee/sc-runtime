//! Copy and host-aggregate paths between a local SQLite store and one remote
//! SQL engine.
//!
//! The copy path is a local SQLite database with the remote schema, for faster
//! or offline access. [`copy_rows`] reads one table through [`EngineStore::read_rows`]
//! and writes it through [`EngineStore::write_rows`]. [`copy_shared_rows`] is
//! that copy after [`TableFlags`] rejects a table that exists on only one side.
//!
//! The host-aggregate path is one SQLite database for this host and a Postgres
//! database that aggregates many hosts. The same function serves MySQL when
//! MySQL is the layout's one sqlx engine: pass that engine's store as the
//! remote side. [`aggregate_host`] reads this host, asks the consumer's
//! [`Rollup`] to map the rows, and writes the aggregate. The application calls
//! it once per host.
//!
//! This crate starts no timer. The application schedules the call. A pattern
//! other than copy or host-aggregate is a file the consumer writes. The
//! sc-runtime crates do not move rows between stores.
//!
//! [`TableFlags`] comes from [`schema::Membership`]. Generation input is the
//! schema crate's `membership.toml`, not an answers-schema property. That file
//! names a sqlite-only table, a remote-only table, and a column that exists on
//! only one side.
//!
//! The crate depends on the schema crate. It does not depend on sqlx or on a
//! `store-*` crate, and it contains no engine SQL.
//!
//! # Examples
//!
//! ```
//! use schema::{EngineStore, OpenConfig, RowRecord, StoreError};
//! use store_adaptor::{Rollup, TableFlags, aggregate_host, copy_shared_rows};
//!
//! # struct Memory {
//! #     rows: std::sync::Mutex<Vec<RowRecord>>,
//! # }
//! #
//! # impl EngineStore for Memory {
//! #     fn open(
//! #         cfg: &OpenConfig,
//! #     ) -> impl std::future::Future<Output = Result<Self, StoreError>> + Send {
//! #         let _ = cfg;
//! #         async { Ok(Self { rows: std::sync::Mutex::new(Vec::new()) }) }
//! #     }
//! #
//! #     fn read_rows(
//! #         &self,
//! #         table: &str,
//! #     ) -> impl std::future::Future<Output = Result<Vec<RowRecord>, StoreError>> + Send {
//! #         let rows = self
//! #             .rows
//! #             .lock()
//! #             .expect("rows")
//! #             .iter()
//! #             .filter(|row| row.table == table)
//! #             .cloned()
//! #             .collect();
//! #         async move { Ok(rows) }
//! #     }
//! #
//! #     fn write_rows(
//! #         &self,
//! #         table: &str,
//! #         rows: &[RowRecord],
//! #     ) -> impl std::future::Future<Output = Result<(), StoreError>> + Send {
//! #         let copied: Vec<_> = rows
//! #             .iter()
//! #             .map(|row| RowRecord {
//! #                 table: table.to_owned(),
//! #                 payload: row.payload.clone(),
//! #             })
//! #             .collect();
//! #         self.rows.lock().expect("rows").extend(copied);
//! #         async move { Ok(()) }
//! #     }
//! # }
//! #
//! # struct CountHost;
//! #
//! # impl Rollup for CountHost {
//! #     fn rollup(&self, host_rows: &[RowRecord]) -> Vec<RowRecord> {
//! #         vec![RowRecord {
//! #             table: "widgets".to_owned(),
//! #             payload: format!(r#"{{"rows":{}}}"#, host_rows.len()).into_bytes(),
//! #         }]
//! #     }
//! # }
//! #
//! # #[tokio::main(flavor = "current_thread")]
//! # async fn main() -> Result<(), StoreError> {
//! let flags = TableFlags {
//!     sqlite_only: vec!["local_notes".to_owned()],
//!     remote_only: vec!["remote_audit".to_owned()],
//!     column_on_one_side: vec!["widgets.host_id".to_owned()],
//! };
//! let local = Memory {
//!     rows: std::sync::Mutex::new(vec![RowRecord {
//!         table: "widgets".to_owned(),
//!         payload: br#"{"id":"w1","name":"bolt"}"#.to_vec(),
//!     }]),
//! };
//! let remote = Memory {
//!     rows: std::sync::Mutex::new(Vec::new()),
//! };
//! copy_shared_rows(&local, &remote, "widgets", &flags).await?;
//! aggregate_host(&local, &remote, "widgets", &flags, &CountHost).await?;
//! # Ok(())
//! # }
//! ```

use schema::{EngineStore, Membership, RowRecord, StoreCode, StoreError};

/// Tables and columns that exist on only one of the two stores.
///
/// `sqlite_only` names a table stored on SQLite and not on the remote engine.
/// `remote_only` names a table stored on the remote engine and not on SQLite.
/// `column_on_one_side` names `table.column` values that exist on only one
/// side of a shared table. Build it with [`TableFlags::from_membership`] from
/// the schema crate's membership description.
///
/// Payloads are moved as stored. A one-sided column stays in the JSON object
/// so a consumer [`Rollup`] can use it (for example a host id) and can drop it
/// before the remote write. This crate does not parse the payload.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TableFlags {
    /// Tables stored only in SQLite.
    pub sqlite_only: Vec<String>,
    /// Tables stored only on the layout's remote engine.
    pub remote_only: Vec<String>,
    /// `table.column` names that exist on only one side.
    pub column_on_one_side: Vec<String>,
}

impl TableFlags {
    /// Flags for SQLite paired with `remote_engine`.
    ///
    /// `remote_engine` is the layout's one sqlx engine, `postgres` or `mysql`.
    /// An engine with no one-sided names contributes empty lists. Columns are
    /// the SQLite columns followed by that remote engine's columns, without
    /// duplicates.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::collections::BTreeMap;
    ///
    /// use schema::{EngineMembership, Membership};
    /// use store_adaptor::TableFlags;
    ///
    /// let mut only_in = BTreeMap::new();
    /// only_in.insert(
    ///     "sqlite".to_owned(),
    ///     EngineMembership::new(vec!["local_notes".to_owned()], vec!["widgets.host_id".to_owned()]),
    /// );
    /// only_in.insert(
    ///     "postgres".to_owned(),
    ///     EngineMembership::new(vec!["remote_audit".to_owned()], Vec::new()),
    /// );
    /// let membership = Membership::try_new(vec!["widgets".to_owned()], only_in).expect("membership");
    /// let flags = TableFlags::from_membership(&membership, "postgres");
    /// assert_eq!(flags.sqlite_only, vec!["local_notes".to_owned()]);
    /// assert_eq!(flags.remote_only, vec!["remote_audit".to_owned()]);
    /// assert_eq!(flags.column_on_one_side, vec!["widgets.host_id".to_owned()]);
    /// ```
    #[must_use]
    pub fn from_membership(membership: &Membership, remote_engine: &str) -> Self {
        let sqlite_only = side_tables(membership, "sqlite");
        let remote_only = side_tables(membership, remote_engine);
        let mut column_on_one_side = side_columns(membership, "sqlite");
        push_unique(
            &mut column_on_one_side,
            &side_columns(membership, remote_engine),
        );
        Self {
            sqlite_only,
            remote_only,
            column_on_one_side,
        }
    }

    /// Rejects a table that membership stores on only one side.
    ///
    /// Shared tables, and names membership does not list, return [`Ok`]. The
    /// store still decides whether it has the table.
    ///
    /// # Errors
    ///
    /// Returns [`StoreCode::NotFound`] when `table` is sqlite-only or
    /// remote-only. The message names which side stores it. Call
    /// [`StoreError::remediation`] for what to do next: choose a table both
    /// stores share.
    pub fn ensure_shared_table(&self, table: &str) -> Result<(), StoreError> {
        if lists(&self.sqlite_only, table) {
            return Err(StoreError::new(
                StoreCode::NotFound,
                format!(
                    "table {table} is stored only in SQLite; the copy and host-aggregate paths do not move it"
                ),
            ));
        }
        if lists(&self.remote_only, table) {
            return Err(StoreError::new(
                StoreCode::NotFound,
                format!(
                    "table {table} is stored only on the remote engine; the copy and host-aggregate paths do not move it"
                ),
            ));
        }
        Ok(())
    }
}

/// Consumer mapping from one host's rows into the remote aggregate.
///
/// The consumer owns the implementation. This crate does not choose how hosts
/// fold into the Postgres or MySQL aggregate. The trait is an extension point:
/// each consumer implements it, and the adaptor calls that implementation.
///
/// # Examples
///
/// ```
/// use schema::RowRecord;
/// use store_adaptor::Rollup;
///
/// struct CountRows;
///
/// impl Rollup for CountRows {
///     fn rollup(&self, host_rows: &[RowRecord]) -> Vec<RowRecord> {
///         let table = host_rows
///             .first()
///             .map(|row| row.table.clone())
///             .unwrap_or_else(|| "widgets".to_owned());
///         vec![RowRecord {
///             table,
///             payload: format!(r#"{{"rows":{}}}"#, host_rows.len()).into_bytes(),
///         }]
///     }
/// }
///
/// let rows = [RowRecord {
///     table: "widgets".to_owned(),
///     payload: br#"{"id":"w1"}"#.to_vec(),
/// }];
/// let rolled = CountRows.rollup(&rows);
/// assert_eq!(rolled.len(), 1);
/// assert_eq!(rolled[0].payload, br#"{"rows":1}"#);
/// ```
pub trait Rollup {
    /// Maps `host_rows` into the rows written to the remote aggregate.
    ///
    /// Each returned [`RowRecord::table`] should be the table
    /// [`aggregate_host`] writes. The adaptor does not rewrite it.
    fn rollup(&self, host_rows: &[RowRecord]) -> Vec<RowRecord>;
}

/// Copies one table from `source` to `dest`.
///
/// This is the copy function both engines use. It reads `table` and writes
/// those rows. It does not read [`TableFlags`] and it does not roll rows up.
/// [`copy_shared_rows`] applies membership and then calls this function.
///
/// # Errors
///
/// Returns the source [`EngineStore::read_rows`] error or the destination
/// [`EngineStore::write_rows`] error unchanged.
///
/// # Examples
///
/// ```
/// use schema::{EngineStore, OpenConfig, RowRecord, StoreError};
/// use store_adaptor::copy_rows;
///
/// # struct Memory {
/// #     rows: std::sync::Mutex<Vec<RowRecord>>,
/// # }
/// #
/// # impl EngineStore for Memory {
/// #     fn open(
/// #         cfg: &OpenConfig,
/// #     ) -> impl std::future::Future<Output = Result<Self, StoreError>> + Send {
/// #         let _ = cfg;
/// #         async { Ok(Self { rows: std::sync::Mutex::new(Vec::new()) }) }
/// #     }
/// #
/// #     fn read_rows(
/// #         &self,
/// #         table: &str,
/// #     ) -> impl std::future::Future<Output = Result<Vec<RowRecord>, StoreError>> + Send {
/// #         let rows = self
/// #             .rows
/// #             .lock()
/// #             .expect("rows")
/// #             .iter()
/// #             .filter(|row| row.table == table)
/// #             .cloned()
/// #             .collect();
/// #         async move { Ok(rows) }
/// #     }
/// #
/// #     fn write_rows(
/// #         &self,
/// #         table: &str,
/// #         rows: &[RowRecord],
/// #     ) -> impl std::future::Future<Output = Result<(), StoreError>> + Send {
/// #         let _ = table;
/// #         self.rows.lock().expect("rows").extend(rows.iter().cloned());
/// #         async move { Ok(()) }
/// #     }
/// # }
/// #
/// # #[tokio::main(flavor = "current_thread")]
/// # async fn main() -> Result<(), StoreError> {
/// let source = Memory {
///     rows: std::sync::Mutex::new(vec![RowRecord {
///         table: "widgets".to_owned(),
///         payload: br#"{"id":"w1","name":"bolt"}"#.to_vec(),
///     }]),
/// };
/// let dest = Memory {
///     rows: std::sync::Mutex::new(Vec::new()),
/// };
/// copy_rows(&source, &dest, "widgets").await?;
/// # let copied = dest.read_rows("widgets").await?;
/// # assert_eq!(copied.len(), 1);
/// # Ok(())
/// # }
/// ```
pub async fn copy_rows<S, D>(source: &S, dest: &D, table: &str) -> Result<(), StoreError>
where
    S: EngineStore,
    D: EngineStore,
{
    let rows = source.read_rows(table).await?;
    dest.write_rows(table, &rows).await
}

/// Copies a table that [`TableFlags`] says exists on both stores.
///
/// This is the copy pattern: a local SQLite database with the remote schema.
/// One-sided tables stay where membership put them. The move itself is
/// [`copy_rows`].
///
/// # Errors
///
/// Returns [`StoreCode::NotFound`] from [`TableFlags::ensure_shared_table`]
/// when `table` is sqlite-only or remote-only. Otherwise returns the
/// [`copy_rows`] error.
///
/// # Examples
///
/// ```
/// use schema::StoreCode;
/// use store_adaptor::TableFlags;
///
/// let flags = TableFlags {
///     sqlite_only: vec!["local_notes".to_owned()],
///     remote_only: vec!["remote_audit".to_owned()],
///     column_on_one_side: vec!["widgets.host_id".to_owned()],
/// };
/// assert!(flags.ensure_shared_table("widgets").is_ok());
/// let err = flags.ensure_shared_table("local_notes").expect_err("sqlite only");
/// assert_eq!(err.code, StoreCode::NotFound);
/// assert!(!err.remediation().is_empty());
/// ```
pub async fn copy_shared_rows<S, D>(
    source: &S,
    dest: &D,
    table: &str,
    flags: &TableFlags,
) -> Result<(), StoreError>
where
    S: EngineStore,
    D: EngineStore,
{
    flags.ensure_shared_table(table)?;
    copy_rows(source, dest, table).await
}

/// Rolls one host's rows into the remote aggregate.
///
/// This is the host-aggregate path. `host` is this host's SQLite database.
/// `remote` is the Postgres database that aggregates many hosts, or the MySQL
/// database when MySQL is the layout's sqlx engine. The application calls this
/// once per host. `rollup` is the consumer-owned mapping.
///
/// Sqlite-only and remote-only tables are not aggregated. A shared table is
/// read from the host, passed to `rollup` with its payload unchanged, and the
/// returned rows are written to `remote`.
///
/// # Errors
///
/// Returns [`StoreCode::NotFound`] when `table` is sqlite-only or remote-only,
/// before either store is called. Otherwise returns the host read error or the
/// remote write error.
///
/// # Examples
///
/// ```
/// use schema::RowRecord;
/// use store_adaptor::Rollup;
///
/// struct Identity;
///
/// impl Rollup for Identity {
///     fn rollup(&self, host_rows: &[RowRecord]) -> Vec<RowRecord> {
///         host_rows.to_vec()
///     }
/// }
///
/// let host_rows = [RowRecord {
///     table: "widgets".to_owned(),
///     payload: br#"{"id":"w1","host_id":"h1"}"#.to_vec(),
/// }];
/// assert_eq!(Identity.rollup(&host_rows), host_rows);
/// ```
pub async fn aggregate_host<H, R, U>(
    host: &H,
    remote: &R,
    table: &str,
    flags: &TableFlags,
    rollup: &U,
) -> Result<(), StoreError>
where
    H: EngineStore,
    R: EngineStore,
    U: Rollup,
{
    flags.ensure_shared_table(table)?;
    let host_rows = host.read_rows(table).await?;
    let aggregated = rollup.rollup(&host_rows);
    remote.write_rows(table, &aggregated).await
}

fn side_tables(membership: &Membership, engine: &str) -> Vec<String> {
    membership
        .only_in()
        .get(engine)
        .map(|side| side.tables().to_vec())
        .unwrap_or_default()
}

fn side_columns(membership: &Membership, engine: &str) -> Vec<String> {
    membership
        .only_in()
        .get(engine)
        .map(|side| side.columns().to_vec())
        .unwrap_or_default()
}

fn push_unique(names: &mut Vec<String>, extra: &[String]) {
    for name in extra {
        if !lists(names, name) {
            names.push(name.clone());
        }
    }
}

fn lists(names: &[String], table: &str) -> bool {
    names.iter().any(|name| name == table)
}
