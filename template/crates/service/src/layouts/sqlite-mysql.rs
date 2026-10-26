include!("common.rs");

use std::path::PathBuf;

use schema::{OpenConfig, WidgetRow};
use store_adaptor::Rollup;

/// SQLite plus MySQL. Service depends on the adaptor, not on the store crates.
#[derive(Debug, Clone)]
pub struct Stores {
    sqlite: store_adaptor::SqliteStore,
    remote: Option<store_adaptor::MysqlStore>,
}

impl Stores {
    /// Opens SQLite at `path` and MySQL when `path` is a MySQL URL.
    ///
    /// Constructs `OpenConfig { url }` for each store that opens. The row
    /// struct comes from the schema crate. Membership for the copy pattern is
    /// read before the stores are returned.
    ///
    /// # Errors
    ///
    /// Returns [`StoreOpenError`] when a database cannot be opened or
    /// membership rejects the shared `widgets` table.
    pub async fn open(path: PathBuf) -> Result<Self, StoreOpenError> {
        let url = path.to_string_lossy().into_owned();
        let flags = store_adaptor::layout_flags().map_err(|detail| StoreOpenError {
            code: "STORE.OPEN",
            cause: "membership.toml could not be read",
            recovery: "fix crates/schema/membership.toml",
            detail,
        })?;
        flags.ensure_shared_table("widgets").map_err(open_error)?;
        let _ = store_adaptor::CopyRollup.rollup(&[], &flags.column_on_one_side);
        let config = OpenConfig::new(url.clone());
        let sqlite = store_adaptor::open_sqlite(&config)
            .await
            .map_err(open_error)?;
        let remote = if url.starts_with("mysql://") {
            Some(
                store_adaptor::open_mysql(&config)
                    .await
                    .map_err(open_error)?,
            )
        } else {
            None
        };
        Ok(Self { sqlite, remote })
    }

    async fn insert_widget_row(&self, name: &str) -> Result<RowText, OpError> {
        let _remote = self.remote.as_ref();
        let row: WidgetRow = store_adaptor::insert_widget(&self.sqlite, name)
            .await
            .map_err(store_query)?;
        Ok(text_row(row))
    }

    async fn fetch_widget_row(&self, id: &str) -> Result<Option<RowText>, OpError> {
        Ok(store_adaptor::get_widget(&self.sqlite, id)
            .await
            .map_err(store_query)?
            .map(text_row))
    }

    async fn remove_widget_row(&self, id: &str) -> Result<Remove, OpError> {
        let row = store_adaptor::get_widget(&self.sqlite, id)
            .await
            .map_err(store_query)?;
        if row.is_none() {
            return Ok(Remove::Missing);
        }
        Ok(Remove::Unsupported)
    }
}

fn text_row(row: WidgetRow) -> RowText {
    RowText {
        id: row.id.to_string(),
        name: row.name,
    }
}

fn open_error(err: schema::StoreError) -> StoreOpenError {
    StoreOpenError {
        code: "STORE.OPEN",
        cause: "the widget database did not open",
        recovery: "check the database path or URL, credentials, and free disk",
        detail: err.to_string(),
    }
}

fn store_query(err: schema::StoreError) -> OpError {
    let cause = err.to_string();
    OpError::new(
        sc_command::ErrorKind::Internal,
        sc_observability_types::ErrorCode::new_static("STORE.QUERY"),
        cause.clone(),
    )
    .with_suggestion(STORE_RECOVERY)
    .with_details(serde_json::json!({ "cause": cause }))
}
