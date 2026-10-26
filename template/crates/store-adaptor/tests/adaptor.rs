use std::collections::BTreeMap;
use std::path::Path;
use std::sync::Mutex;

use schema::{
    EngineMembership, EngineStore, Membership, OpenConfig, RowRecord, StoreCode, StoreError,
    check_read_limit, check_write_limit,
};
use serde::Deserialize;
use store_adaptor::{Rollup, TableFlags, aggregate_host, copy_rows, copy_shared_rows};

#[derive(Debug, Deserialize)]
struct MembershipFile {
    #[serde(default)]
    shared: Vec<String>,
    #[serde(default)]
    only_in: BTreeMap<String, EngineFile>,
}

#[derive(Debug, Deserialize, Default)]
struct EngineFile {
    #[serde(default)]
    tables: Vec<String>,
    #[serde(default)]
    columns: Vec<String>,
}

fn membership_from_toml(text: &str) -> Result<Membership, schema::MembershipError> {
    let file: MembershipFile = toml::from_str(text).expect("membership toml");
    let only_in = file
        .only_in
        .into_iter()
        .map(|(engine, side)| (engine, EngineMembership::new(side.tables, side.columns)))
        .collect();
    Membership::try_new(file.shared, only_in)
}

fn template_membership() -> Membership {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../schema/membership.toml");
    let text = std::fs::read_to_string(path).expect("membership.toml");
    membership_from_toml(&text).expect("membership")
}

struct MemoryStore {
    rows: Mutex<Vec<RowRecord>>,
    reads: Mutex<u32>,
    writes: Mutex<u32>,
    fail_read: Option<(StoreCode, &'static str)>,
    fail_write: Option<(StoreCode, &'static str)>,
}

impl MemoryStore {
    fn with_rows(rows: Vec<RowRecord>) -> Self {
        Self {
            rows: Mutex::new(rows),
            reads: Mutex::new(0),
            writes: Mutex::new(0),
            fail_read: None,
            fail_write: None,
        }
    }

    fn failing_read(code: StoreCode, message: &'static str) -> Self {
        let mut store = Self::with_rows(Vec::new());
        store.fail_read = Some((code, message));
        store
    }

    fn failing_write(rows: Vec<RowRecord>, code: StoreCode, message: &'static str) -> Self {
        let mut store = Self::with_rows(rows);
        store.fail_write = Some((code, message));
        store
    }

    fn read_count(&self) -> u32 {
        *self.reads.lock().expect("reads")
    }

    fn write_count(&self) -> u32 {
        *self.writes.lock().expect("writes")
    }

    fn tables(&self) -> Vec<String> {
        let rows = self.rows.lock().expect("rows");
        let mut names: Vec<_> = rows.iter().map(|row| row.table.clone()).collect();
        names.sort();
        names.dedup();
        names
    }
}

impl EngineStore for MemoryStore {
    fn open(
        cfg: &OpenConfig,
    ) -> impl std::future::Future<Output = Result<Self, StoreError>> + Send {
        let _ = cfg.url.clone();
        async move { Ok(Self::with_rows(Vec::new())) }
    }

    fn read_rows(
        &self,
        table: &str,
    ) -> impl std::future::Future<Output = Result<Vec<RowRecord>, StoreError>> + Send {
        let result = if let Some((code, message)) = self.fail_read {
            Err(StoreError::new(code, message))
        } else {
            *self.reads.lock().expect("reads") += 1;
            let matched: Vec<_> = self
                .rows
                .lock()
                .expect("rows")
                .iter()
                .filter(|row| row.table == table)
                .cloned()
                .collect();
            check_read_limit(table, matched.len()).map(|()| matched)
        };
        async move { result }
    }

    fn write_rows(
        &self,
        table: &str,
        rows: &[RowRecord],
    ) -> impl std::future::Future<Output = Result<(), StoreError>> + Send {
        let result = if let Some((code, message)) = self.fail_write {
            Err(StoreError::new(code, message))
        } else if let Err(err) = check_write_limit(rows) {
            Err(err)
        } else if rows.iter().any(|row| row.table != table) {
            Err(StoreError::new(
                StoreCode::Query,
                "row table does not match the destination table",
            ))
        } else {
            self.rows.lock().expect("rows").extend(rows.iter().cloned());
            *self.writes.lock().expect("writes") += 1;
            Ok(())
        };
        async move { result }
    }
}

fn row(table: &str, payload: &str) -> RowRecord {
    RowRecord {
        table: table.to_owned(),
        payload: payload.as_bytes().to_vec(),
    }
}

struct CountRows;

impl Rollup for CountRows {
    fn rollup(&self, host_rows: &[RowRecord]) -> Vec<RowRecord> {
        let table = host_rows
            .first()
            .map(|row| row.table.clone())
            .unwrap_or_else(|| "widgets".to_owned());
        vec![row(&table, &format!(r#"{{"rows":{}}}"#, host_rows.len()))]
    }
}

struct DropListed {
    columns: Vec<String>,
}

impl Rollup for DropListed {
    fn rollup(&self, host_rows: &[RowRecord]) -> Vec<RowRecord> {
        host_rows
            .iter()
            .map(|host_row| {
                let mut text = String::from_utf8(host_row.payload.clone()).expect("utf-8 payload");
                for column in &self.columns {
                    let key = column
                        .split_once('.')
                        .map(|(_, field)| field)
                        .unwrap_or(column.as_str());
                    text = text.replace(&format!(r#","{key}":"h1""#), "");
                }
                RowRecord {
                    table: host_row.table.clone(),
                    payload: text.into_bytes(),
                }
            })
            .collect()
    }
}

#[test]
fn table_flags_follow_membership_toml() {
    let membership = template_membership();
    let flags = TableFlags::from_membership(&membership, "postgres");
    assert_eq!(flags.sqlite_only, vec!["local_notes".to_owned()]);
    assert_eq!(flags.remote_only, vec!["remote_audit".to_owned()]);
    assert_eq!(flags.column_on_one_side, vec!["widgets.host_id".to_owned()]);
    assert_eq!(membership.shared(), &["widgets".to_owned()]);
}

#[test]
fn mysql_remote_uses_the_same_one_sided_table() {
    let membership = template_membership();
    let flags = TableFlags::from_membership(&membership, "mysql");
    assert_eq!(flags.sqlite_only, vec!["local_notes".to_owned()]);
    assert_eq!(flags.remote_only, vec!["remote_audit".to_owned()]);
    assert_eq!(flags.column_on_one_side, vec!["widgets.host_id".to_owned()]);
}

#[test]
fn flags_take_columns_from_the_selected_remote_engine() {
    let mut only_in = BTreeMap::new();
    only_in.insert(
        "sqlite".to_owned(),
        EngineMembership::new(Vec::new(), vec!["widgets.host_id".to_owned()]),
    );
    only_in.insert(
        "postgres".to_owned(),
        EngineMembership::new(
            vec!["pg_audit".to_owned()],
            vec!["widgets.region".to_owned()],
        ),
    );
    only_in.insert(
        "mysql".to_owned(),
        EngineMembership::new(
            vec!["my_audit".to_owned()],
            vec!["widgets.shard".to_owned()],
        ),
    );
    let membership = Membership::try_new(vec!["widgets".to_owned()], only_in).expect("membership");
    let postgres = TableFlags::from_membership(&membership, "postgres");
    assert_eq!(postgres.remote_only, vec!["pg_audit".to_owned()]);
    assert_eq!(
        postgres.column_on_one_side,
        vec!["widgets.host_id".to_owned(), "widgets.region".to_owned()]
    );
    let mysql = TableFlags::from_membership(&membership, "mysql");
    assert_eq!(mysql.remote_only, vec!["my_audit".to_owned()]);
    assert_eq!(
        mysql.column_on_one_side,
        vec!["widgets.host_id".to_owned(), "widgets.shard".to_owned()]
    );
}

#[test]
fn duplicate_one_sided_column_is_named_once() {
    let mut only_in = BTreeMap::new();
    only_in.insert(
        "sqlite".to_owned(),
        EngineMembership::new(Vec::new(), vec!["widgets.host_id".to_owned()]),
    );
    only_in.insert(
        "postgres".to_owned(),
        EngineMembership::new(Vec::new(), vec!["widgets.host_id".to_owned()]),
    );
    let membership = Membership::try_new(vec!["widgets".to_owned()], only_in).expect("membership");
    let flags = TableFlags::from_membership(&membership, "postgres");
    assert_eq!(flags.column_on_one_side, vec!["widgets.host_id".to_owned()]);
}

#[test]
fn empty_membership_names_nothing_one_sided() {
    let flags = TableFlags::from_membership(&Membership::empty(), "postgres");
    assert!(flags.sqlite_only.is_empty());
    assert!(flags.remote_only.is_empty());
    assert!(flags.column_on_one_side.is_empty());
    assert!(flags.ensure_shared_table("widgets").is_ok());
}

#[tokio::test]
async fn copy_rows_moves_one_table_between_doubles() {
    let source = MemoryStore::with_rows(vec![
        row("widgets", r#"{"id":"w1","name":"bolt"}"#),
        row("local_notes", r#"{"note":"keep"}"#),
    ]);
    let dest = MemoryStore::with_rows(vec![row("remote_audit", r#"{"id":"a1"}"#)]);
    copy_rows(&source, &dest, "widgets").await.expect("copy");
    let copied = dest.read_rows("widgets").await.expect("read dest");
    assert_eq!(copied, vec![row("widgets", r#"{"id":"w1","name":"bolt"}"#)]);
    assert_eq!(
        dest.tables(),
        vec!["remote_audit".to_owned(), "widgets".to_owned()]
    );
    let notes = dest.read_rows("local_notes").await.expect("notes");
    assert!(notes.is_empty());
}

#[tokio::test]
async fn copy_shared_rows_refuses_one_sided_tables() {
    let flags = TableFlags::from_membership(&template_membership(), "postgres");
    let source = MemoryStore::with_rows(vec![
        row("widgets", r#"{"id":"w1","name":"bolt"}"#),
        row("local_notes", r#"{"note":"keep"}"#),
        row("remote_audit", r#"{"id":"a1"}"#),
    ]);
    let dest = MemoryStore::with_rows(Vec::new());

    copy_shared_rows(&source, &dest, "widgets", &flags)
        .await
        .expect("shared");
    let copied = dest.read_rows("widgets").await.expect("widgets");
    assert_eq!(copied.len(), 1);

    for table in ["local_notes", "remote_audit"] {
        let err = copy_shared_rows(&source, &dest, table, &flags)
            .await
            .expect_err(table);
        assert_eq!(err.code, StoreCode::NotFound);
        assert!(!err.remediation().is_empty());
        assert!(!err.is_retryable());
    }
    assert_eq!(dest.write_count(), 1);
    assert!(
        dest.read_rows("local_notes")
            .await
            .expect("local")
            .is_empty()
    );
}

#[tokio::test]
async fn copy_shared_rows_propagates_store_errors_without_a_partial_write() {
    let flags = TableFlags::from_membership(&Membership::empty(), "mysql");
    let source = MemoryStore::failing_read(StoreCode::NotFound, "unknown table extra");
    let dest = MemoryStore::with_rows(Vec::new());
    let err = copy_shared_rows(&source, &dest, "extra", &flags)
        .await
        .expect_err("read");
    assert_eq!(err.code, StoreCode::NotFound);
    assert_eq!(dest.write_count(), 0);

    let source = MemoryStore::with_rows(vec![row("widgets", r#"{"id":"w1"}"#)]);
    let dest =
        MemoryStore::failing_write(Vec::new(), StoreCode::Unavailable, "pool acquire timed out");
    let err = copy_rows(&source, &dest, "widgets")
        .await
        .expect_err("write");
    assert_eq!(err.code, StoreCode::Unavailable);
    assert!(err.is_retryable());
    assert_eq!(dest.write_count(), 0);
    assert!(dest.read_rows("widgets").await.expect("dest").is_empty());
}

#[tokio::test]
async fn aggregate_host_writes_the_consumer_rollup() {
    let flags = TableFlags::from_membership(&template_membership(), "postgres");
    let host = MemoryStore::with_rows(vec![
        row("widgets", r#"{"id":"w1","name":"bolt"}"#),
        row("widgets", r#"{"id":"w2","name":"nut"}"#),
        row("local_notes", r#"{"note":"stay"}"#),
    ]);
    let remote = MemoryStore::with_rows(Vec::new());
    aggregate_host(&host, &remote, "widgets", &flags, &CountRows)
        .await
        .expect("aggregate");
    let aggregated = remote.read_rows("widgets").await.expect("remote");
    assert_eq!(aggregated, vec![row("widgets", r#"{"rows":2}"#)]);
    let host_rows = host.read_rows("widgets").await.expect("host");
    assert_eq!(host_rows.len(), 2);
    assert_eq!(remote.write_count(), 1);
    assert!(
        remote
            .read_rows("local_notes")
            .await
            .expect("notes")
            .is_empty()
    );
}

#[tokio::test]
async fn aggregate_host_refuses_one_sided_tables_before_reading() {
    let flags = TableFlags::from_membership(&template_membership(), "mysql");
    let host = MemoryStore::failing_read(StoreCode::Query, "should not be read");
    let remote = MemoryStore::failing_write(Vec::new(), StoreCode::Query, "should not be written");
    for table in ["local_notes", "remote_audit"] {
        let err = aggregate_host(&host, &remote, table, &flags, &CountRows)
            .await
            .expect_err(table);
        assert_eq!(err.code, StoreCode::NotFound);
        assert!(err.to_string().contains(table));
    }
    assert_eq!(host.read_count(), 0);
    assert_eq!(remote.write_count(), 0);
}

#[tokio::test]
async fn rollup_sees_the_one_sided_column_and_can_drop_it() {
    let flags = TableFlags::from_membership(&template_membership(), "postgres");
    assert_eq!(flags.column_on_one_side, vec!["widgets.host_id".to_owned()]);
    let host = MemoryStore::with_rows(vec![row(
        "widgets",
        r#"{"id":"w1","name":"bolt","host_id":"h1"}"#,
    )]);
    let remote = MemoryStore::with_rows(Vec::new());
    let rollup = DropListed {
        columns: flags.column_on_one_side.clone(),
    };
    aggregate_host(&host, &remote, "widgets", &flags, &rollup)
        .await
        .expect("aggregate");
    let aggregated = remote.read_rows("widgets").await.expect("remote");
    assert_eq!(
        aggregated,
        vec![row("widgets", r#"{"id":"w1","name":"bolt"}"#)]
    );
}

#[tokio::test]
async fn empty_host_still_writes_the_rollup() {
    let flags = TableFlags::from_membership(&Membership::empty(), "postgres");
    let host = MemoryStore::with_rows(Vec::new());
    let remote = MemoryStore::with_rows(Vec::new());
    aggregate_host(&host, &remote, "widgets", &flags, &CountRows)
        .await
        .expect("empty host");
    let aggregated = remote.read_rows("widgets").await.expect("remote");
    assert_eq!(aggregated, vec![row("widgets", r#"{"rows":0}"#)]);
}

#[test]
fn manifest_depends_on_the_schema_crate_and_not_on_an_engine() {
    let manifest = include_str!("../Cargo.toml");
    let runtime = manifest
        .split("[dev-dependencies]")
        .next()
        .expect("dependencies");
    assert!(runtime.contains("schema"));
    assert!(!manifest.contains("sqlx"));
    assert!(!manifest.contains("../store-"));
    assert!(!runtime.contains("store-postgres"));
    assert!(!runtime.contains("store-mysql"));
    assert!(!runtime.contains("store-sqlite"));
}
