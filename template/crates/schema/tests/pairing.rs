use std::sync::Mutex;

use schema::{EngineStore, Membership, OpenConfig, RowRecord, StoreCode, StoreError, WidgetRow};

#[test]
fn membership_describes_a_shared_table_and_one_sided_columns() {
    let membership = Membership::bundled().expect("membership");
    assert_eq!(membership.shared(), &["widgets".to_owned()]);
    assert_eq!(membership.sqlite_only(), &["local_notes".to_owned()]);
    assert_eq!(membership.remote_only(), &["remote_audit".to_owned()]);
    assert_eq!(
        membership.column_on_one_side(),
        &["widgets.host_id".to_owned()]
    );
}

#[test]
fn store_error_display_includes_the_code_and_has_no_source() {
    let err = StoreError {
        code: StoreCode::Query,
        message: "missing column".to_owned(),
    };
    assert_eq!(err.to_string(), "Query: missing column");
    assert!(std::error::Error::source(&err).is_none());
}

#[test]
fn widget_row_is_the_schema_row() {
    let row = WidgetRow {
        id: "w1".to_owned(),
        name: "bolt".to_owned(),
    };
    assert_eq!(row.id, "w1");
    assert_eq!(row.name, "bolt");
}

#[test]
fn schema_manifest_does_not_depend_on_sqlx() {
    let manifest = include_str!("../Cargo.toml");
    assert!(
        !manifest.contains("sqlx"),
        "the schema crate must not depend on sqlx"
    );
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
            return Err(StoreError {
                code: StoreCode::Connection,
                message: format!("{engine} url is empty"),
            });
        }
        Ok(Self {
            engine,
            url: cfg.url.clone(),
            rows: Mutex::new(Vec::new()),
        })
    }

    fn read_rows(&self, table: &str) -> Result<Vec<RowRecord>, StoreError> {
        let rows = self
            .rows
            .lock()
            .unwrap_or_else(|err| err.into_inner())
            .iter()
            .filter(|row| row.table == table)
            .cloned()
            .collect();
        Ok(rows)
    }

    fn write_rows(&self, table: &str, rows: &[RowRecord]) -> Result<(), StoreError> {
        let mut guard = self.rows.lock().unwrap_or_else(|err| err.into_inner());
        for row in rows {
            if row.table != table {
                return Err(StoreError {
                    code: StoreCode::Query,
                    message: format!("{} row table does not match {table}", self.engine),
                });
            }
            guard.push(row.clone());
        }
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

async fn round_trip<S>(store: &S, table: &str)
where
    S: EngineStore,
{
    let row = RowRecord {
        table: table.to_owned(),
        payload: br#"{"id":"w1","name":"bolt"}"#.to_vec(),
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
    let postgres = PostgresStub::open(&OpenConfig {
        url: "postgres://paired.example/widgets".to_owned(),
    })
    .await
    .expect("postgres open");
    let mysql = MysqlStub::open(&OpenConfig {
        url: "mysql://paired.example/widgets".to_owned(),
    })
    .await
    .expect("mysql open");
    round_trip(&postgres, "widgets").await;
    round_trip(&mysql, "widgets").await;
    assert_eq!(mysql.engine(), "mysql");
    assert_eq!(postgres.0.url, "postgres://paired.example/widgets");
}

#[tokio::test]
async fn stub_open_rejects_an_empty_url() {
    let err = PostgresStub::open(&OpenConfig { url: String::new() })
        .await
        .expect_err("empty url");
    assert_eq!(err.code, StoreCode::Connection);
    assert!(err.to_string().starts_with("Connection:"));
    assert!(std::error::Error::source(&err).is_none());
}
