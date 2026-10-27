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
//! only one side. [`TableFlags::from_membership`] takes the remote engine as a
//! string. A name that is not `postgres`, `mysql`, or an engine key in the
//! file fails closed with [`schema::StoreCode::Membership`]. Sprint `b-8`
//! renders the layout key as a generated constant.
//!
//! Both patterns move one whole table on each call. [`copy_rows`] writes the
//! source rows unchanged. [`copy_shared_rows`] and [`aggregate_host`] pass
//! those rows and [`TableFlags::column_on_one_side`] to the consumer
//! [`Rollup`], then write what it returns. A call is bounded by
//! [`schema::MAX_PAIRING_ROWS`] (10_000 rows) and
//! [`schema::MAX_ROW_PAYLOAD_BYTES`] (1 MiB). A larger table or payload
//! returns [`schema::StoreCode::Query`] from the store.
//!
//! A repeated call replaces each copied row identity on the destination. It
//! does not insert a second copy of those identities, and it does not keep a
//! watermark. [`schema::EngineStore::write_rows`] upserts the batch and leaves
//! every other identity in the table (ADR-RUN-0307), so a second scheduled run
//! does not return [`schema::StoreCode::Conflict`]. A watermark, or
//! aggregation that can fail, is a third adaptor the consumer writes.
//!
//! The crate depends on the schema crate. It does not depend on sqlx or on a
//! `store-*` crate, and it contains no engine SQL.
//!
//! # Examples
//!
//! ```
//! use schema::{EngineStore, OpenConfig, RowRecord, StoreError, TableName};
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
//! #         table: &schema::TableName,
//! #     ) -> impl std::future::Future<Output = Result<Vec<RowRecord>, StoreError>> + Send {
//! #         let rows = self
//! #             .rows
//! #             .lock()
//! #             .expect("rows")
//! #             .iter()
//! #             .filter(|row| &row.table == table)
//! #             .cloned()
//! #             .collect();
//! #         async move { Ok(rows) }
//! #     }
//! #
//! #     fn write_rows(
//! #         &self,
//! #         table: &schema::TableName,
//! #         rows: &[RowRecord],
//! #     ) -> impl std::future::Future<Output = Result<(), StoreError>> + Send {
//! #         let copied: Vec<_> = rows
//! #             .iter()
//! #             .map(|row| RowRecord {
//! #                 table: table.clone(),
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
//! #     fn rollup(
//! #         &self,
//! #         host_rows: &[RowRecord],
//! #         _one_sided_columns: &[String],
//! #     ) -> Vec<RowRecord> {
//! #         let count = host_rows.len().to_string();
//! #         let mut payload = vec![b'{'];
//! #         payload.extend_from_slice(b"\"rows\":");
//! #         payload.extend_from_slice(count.as_bytes());
//! #         payload.push(b'}');
//! #         vec![RowRecord {
//! #             table: TableName::widgets(),
//! #             payload,
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
//!         table: TableName::widgets(),
//!         payload: br#"{"id":"w1","name":"bolt"}"#.to_vec(),
//!     }]),
//! };
//! let remote = Memory {
//!     rows: std::sync::Mutex::new(Vec::new()),
//! };
//! copy_shared_rows(&local, &remote, &schema::TableName::widgets(), &flags, &CountHost).await?;
//! aggregate_host(&local, &remote, &schema::TableName::widgets(), &flags, &CountHost).await?;
//! # Ok(())
//! # }
//! ```

include!("shared.rs");
