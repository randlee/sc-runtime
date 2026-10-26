use std::collections::BTreeMap;
use std::path::Path;
use std::sync::Mutex;

use schema::{
    EngineMembership, EngineStore, Membership, OpenConfig, RowRecord, StoreCode, StoreError,
    WidgetId, WidgetRow, check_read_limit, check_write_limit,
};

mod common;

fn membership_from_toml(text: &str) -> Result<Membership, schema::MembershipError> {
    common::membership_from_toml(text)
}

#[test]
fn membership_describes_shared_and_engine_sided_names() {
    let membership = membership_from_toml(include_str!("../membership.toml")).expect("membership");
    assert_eq!(membership.shared(), &["widgets".to_owned()]);
    let sqlite = membership.only_in().get("sqlite").expect("sqlite side");
    assert_eq!(sqlite.tables(), &["local_notes".to_owned()]);
    assert_eq!(sqlite.columns(), &["widgets.host_id".to_owned()]);
    let postgres = membership.only_in().get("postgres").expect("postgres side");
    assert_eq!(postgres.tables(), &["remote_audit".to_owned()]);
    assert!(postgres.columns().is_empty());
    let mysql = membership.only_in().get("mysql").expect("mysql side");
    assert_eq!(mysql.tables(), &["remote_audit".to_owned()]);
    let engines: Vec<_> = membership.engines().collect();
    assert_eq!(engines, ["mysql", "postgres", "sqlite"]);
}

#[test]
fn empty_membership_is_valid_and_shared_schema_needs_no_file() {
    let membership = Membership::empty();
    assert!(membership.shared().is_empty());
    assert!(membership.only_in().is_empty());
    let parsed = membership_from_toml("shared = []\n").expect("empty document");
    assert_eq!(parsed, Membership::empty());
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    assert!(
        root.join("src/lib.rs").is_file(),
        "EngineStore lives in the crate even when membership.toml is absent"
    );
}

#[test]
fn membership_rejects_a_shared_table_listed_as_one_sided() {
    let mut only_in = BTreeMap::new();
    only_in.insert(
        "sqlite".to_owned(),
        EngineMembership::new(vec!["widgets".to_owned()], Vec::new()),
    );
    let err = Membership::try_new(vec!["widgets".to_owned()], only_in).expect_err("overlap");
    assert!(err.to_string().contains("shared"));
}

#[test]
fn store_error_display_includes_the_code_and_has_no_source() {
    let err = StoreError {
        code: StoreCode::Query,
        message: "missing column".to_owned(),
    };
    assert_eq!(err.to_string(), "Query: missing column");
    assert!(std::error::Error::source(&err).is_none());
    assert!(!err.is_retryable());
    assert!(!err.remediation().is_empty());
    let retry = StoreError::new(StoreCode::Unavailable, "sqlite pool acquire timed out");
    assert!(retry.is_retryable());
    assert!(retry.to_string().starts_with("Unavailable:"));
}

#[test]
fn widget_row_id_is_a_newtype() {
    let row = WidgetRow {
        id: WidgetId::from_column("w1"),
        name: "bolt".to_owned(),
    };
    assert_eq!(row.id.as_str(), "w1");
    assert_eq!(row.name, "bolt");
    let bytes = serde_json::to_vec(&row).expect("json");
    let loaded: WidgetRow = serde_json::from_slice(&bytes).expect("row");
    assert_eq!(loaded, row);
}

#[test]
fn payload_encoding_round_trips_through_the_stub() {
    let payload = pairing_payload();
    let value: serde_json::Value = serde_json::from_slice(&payload).expect("object");
    assert!(value["note"].is_null());
    assert_eq!(value["big"], "9007199254740993");
    assert_eq!(value["blob"], "Ymlu");
    assert_eq!(value["at"], "2026-10-26T15:04:05Z");
    assert_eq!(value["ok"], true);
    let encoded = serde_json::to_vec(&value).expect("encode");
    assert_eq!(encoded, payload);
}

#[test]
fn pairing_limits_reject_unbounded_batches() {
    let row = RowRecord {
        table: "widgets".to_owned(),
        payload: br#"{"id":"w1","name":"bolt"}"#.to_vec(),
    };
    let too_many = vec![row; schema::MAX_PAIRING_ROWS + 1];
    let err = check_write_limit(&too_many).expect_err("rows");
    assert_eq!(err.code, StoreCode::Query);
    let huge = RowRecord {
        table: "widgets".to_owned(),
        payload: vec![b'x'; schema::MAX_ROW_PAYLOAD_BYTES + 1],
    };
    let err = check_write_limit(std::slice::from_ref(&huge)).expect_err("bytes");
    assert_eq!(err.code, StoreCode::Query);
    let err = check_read_limit("widgets", schema::MAX_PAIRING_ROWS + 1).expect_err("read");
    assert_eq!(err.code, StoreCode::Query);
    assert!(check_read_limit("widgets", schema::MAX_PAIRING_ROWS).is_ok());
}

#[test]
fn schema_manifest_does_not_depend_on_sqlx() {
    let manifest = include_str!("../Cargo.toml");
    assert!(
        !manifest.contains("sqlx"),
        "the schema crate must not depend on sqlx"
    );
    let deps = manifest
        .split("[dev-dependencies]")
        .next()
        .expect("dependencies");
    assert!(
        !deps.contains("toml"),
        "toml stays a dev-dependency until the adaptor reads membership.toml"
    );
}

#[test]
fn workspace_manifests_match_except_the_checkout_exclude() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let generated = root.join("workspace.template.toml");
    if !generated.is_file() {
        return;
    }
    let checkout = std::fs::read_to_string(root.join("Cargo.toml")).expect("checkout manifest");
    let generated = std::fs::read_to_string(generated).expect("generated manifest");
    assert_eq!(
        lines_without_checkout_exclude(&checkout),
        generated.lines().collect::<Vec<_>>()
    );
}

#[test]
fn checkout_exclude_compare_ignores_line_endings() {
    let checkout = [
        "[workspace]",
        "resolver = \"2\"",
        "members = [\"crates/*\"]",
        "# cargo-generate replaces this file with workspace.template.toml, which has",
        "# no exclude.",
        "exclude = [\"crates/cli\", \"crates/daemon\", \"crates/service\"]",
        "",
        "[workspace.package]",
    ]
    .join("\r\n");
    let generated = [
        "[workspace]",
        "resolver = \"2\"",
        "members = [\"crates/*\"]",
        "",
        "[workspace.package]",
    ];
    assert_eq!(lines_without_checkout_exclude(&checkout), generated);
}

fn lines_without_checkout_exclude(text: &str) -> Vec<&str> {
    let mut lines = Vec::new();
    let mut skipping = false;
    for line in text.lines() {
        if line.starts_with("# cargo-generate replaces this file") {
            skipping = true;
            continue;
        }
        if skipping {
            if line.starts_with("exclude = ") {
                skipping = false;
            }
            continue;
        }
        lines.push(line);
    }
    lines
}

#[test]
fn architecture_lists_adr_run_0305_once() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../docs/architecture.md");
    if !path.is_file() {
        return;
    }
    let text = std::fs::read_to_string(path).expect("architecture");
    let headings = text
        .lines()
        .filter(|line| line.starts_with("## ADR-RUN-0305:"))
        .count();
    assert_eq!(headings, 1);
}

fn pairing_payload() -> Vec<u8> {
    br#"{"at":"2026-10-26T15:04:05Z","big":"9007199254740993","blob":"Ymlu","note":null,"ok":true}"#
        .to_vec()
}

#[derive(Debug)]
struct EngineStub {
    engine: &'static str,
    url: String,
    rows: Mutex<Vec<RowRecord>>,
}

impl EngineStub {
    fn open(engine: &'static str, cfg: &OpenConfig) -> Result<Self, StoreError> {
        if cfg.url.is_empty() {
            return Err(StoreError::new(
                StoreCode::Connection,
                format!("{engine} url is empty"),
            ));
        }
        Ok(Self {
            engine,
            url: cfg.url.clone(),
            rows: Mutex::new(Vec::new()),
        })
    }

    fn read_rows(&self, table: &str) -> Result<Vec<RowRecord>, StoreError> {
        let rows: Vec<RowRecord> = self
            .rows
            .lock()
            .unwrap_or_else(|err| err.into_inner())
            .iter()
            .filter(|row| row.table == table)
            .cloned()
            .collect();
        check_read_limit(table, rows.len())?;
        Ok(rows)
    }

    fn write_rows(&self, table: &str, rows: &[RowRecord]) -> Result<(), StoreError> {
        check_write_limit(rows)?;
        if rows.iter().any(|row| row.table != table) {
            return Err(StoreError::new(
                StoreCode::Query,
                format!("{} row table does not match {table}", self.engine),
            ));
        }
        let mut guard = self.rows.lock().unwrap_or_else(|err| err.into_inner());
        guard.extend(rows.iter().cloned());
        Ok(())
    }
}

#[derive(Debug)]
struct PostgresStub(EngineStub);

#[derive(Debug)]
struct MysqlStub(EngineStub);

impl EngineStore for PostgresStub {
    fn open(
        cfg: &OpenConfig,
    ) -> impl std::future::Future<Output = Result<Self, StoreError>> + Send {
        let opened = EngineStub::open("postgres", cfg);
        async move { opened.map(Self) }
    }

    fn read_rows(
        &self,
        table: &str,
    ) -> impl std::future::Future<Output = Result<Vec<RowRecord>, StoreError>> + Send {
        let rows = self.0.read_rows(table);
        async move { rows }
    }

    fn write_rows(
        &self,
        table: &str,
        rows: &[RowRecord],
    ) -> impl std::future::Future<Output = Result<(), StoreError>> + Send {
        let written = self.0.write_rows(table, rows);
        async move { written }
    }
}

impl EngineStore for MysqlStub {
    fn open(
        cfg: &OpenConfig,
    ) -> impl std::future::Future<Output = Result<Self, StoreError>> + Send {
        let opened = EngineStub::open("mysql", cfg);
        async move { opened.map(Self) }
    }

    fn read_rows(
        &self,
        table: &str,
    ) -> impl std::future::Future<Output = Result<Vec<RowRecord>, StoreError>> + Send {
        let rows = self.0.read_rows(table);
        async move { rows }
    }

    fn write_rows(
        &self,
        table: &str,
        rows: &[RowRecord],
    ) -> impl std::future::Future<Output = Result<(), StoreError>> + Send {
        let written = self.0.write_rows(table, rows);
        async move { written }
    }
}

/// A consumer-written trait for one engine. It is not [`EngineStore`].
trait MysqlOnly {
    fn engine(&self) -> &'static str;
}

impl MysqlOnly for MysqlStub {
    fn engine(&self) -> &'static str {
        self.0.engine
    }
}

async fn round_trip<S>(store: &S, table: &str, payload: Vec<u8>)
where
    S: EngineStore,
{
    let row = RowRecord {
        table: table.to_owned(),
        payload,
    };
    store
        .write_rows(table, std::slice::from_ref(&row))
        .await
        .expect("write");
    let got = store.read_rows(table).await.expect("read");
    assert_eq!(got, vec![row]);
}

#[tokio::test]
async fn postgres_and_mysql_stubs_implement_engine_store() {
    let postgres = PostgresStub::open(&OpenConfig::new("postgres://paired.example/widgets"))
        .await
        .expect("postgres open");
    let mysql = MysqlStub::open(&OpenConfig::new("mysql://paired.example/widgets"))
        .await
        .expect("mysql open");
    round_trip(&postgres, "widgets", pairing_payload()).await;
    round_trip(&mysql, "widgets", br#"{"id":"w1","name":"bolt"}"#.to_vec()).await;
    assert_eq!(mysql.engine(), "mysql");
    assert_eq!(postgres.0.url, "postgres://paired.example/widgets");
}

#[tokio::test]
async fn stub_open_rejects_an_empty_url() {
    let err = PostgresStub::open(&OpenConfig::new(String::new()))
        .await
        .expect_err("empty url");
    assert_eq!(err.code, StoreCode::Connection);
    assert!(err.to_string().starts_with("Connection:"));
    assert!(std::error::Error::source(&err).is_none());
}

#[tokio::test]
async fn stub_rejects_an_oversized_write() {
    let store = PostgresStub::open(&OpenConfig::new("postgres://paired.example/widgets"))
        .await
        .expect("open");
    let row = RowRecord {
        table: "widgets".to_owned(),
        payload: vec![b'x'; schema::MAX_ROW_PAYLOAD_BYTES + 1],
    };
    let err = store
        .write_rows("widgets", &[row])
        .await
        .expect_err("limit");
    assert_eq!(err.code, StoreCode::Query);
}

async fn mismatched_batch_leaves_stub_unchanged<S>(store: &S)
where
    S: EngineStore,
{
    let kept = RowRecord {
        table: "widgets".to_owned(),
        payload: br#"{"id":"kept","name":"bolt"}"#.to_vec(),
    };
    store
        .write_rows("widgets", std::slice::from_ref(&kept))
        .await
        .expect("seed");
    let accepted = RowRecord {
        table: "widgets".to_owned(),
        payload: br#"{"id":"new","name":"nut"}"#.to_vec(),
    };
    let rejected = RowRecord {
        table: "other".to_owned(),
        payload: br#"{"id":"bad","name":"nope"}"#.to_vec(),
    };
    let err = store
        .write_rows("widgets", &[accepted, rejected])
        .await
        .expect_err("table mismatch");
    assert_eq!(err.code, StoreCode::Query);
    let rows = store.read_rows("widgets").await.expect("read");
    assert_eq!(rows, vec![kept]);
    let other = store.read_rows("other").await.expect("other");
    assert!(other.is_empty());
}

#[tokio::test]
async fn stub_write_rejects_a_later_table_mismatch_without_partial_rows() {
    let postgres = PostgresStub::open(&OpenConfig::new("postgres://paired.example/widgets"))
        .await
        .expect("postgres open");
    let mysql = MysqlStub::open(&OpenConfig::new("mysql://paired.example/widgets"))
        .await
        .expect("mysql open");
    mismatched_batch_leaves_stub_unchanged(&postgres).await;
    mismatched_batch_leaves_stub_unchanged(&mysql).await;
}
