include!("common.rs");

use std::path::PathBuf;

use schema::{OpenConfig, WidgetRow};

/// SQLite-only stores. There is no adaptor.
#[derive(Debug, Clone)]
pub struct Stores {
    sqlite: store_sqlite::Store,
}

impl Stores {
    /// Opens the SQLite file at `path`.
    ///
    /// Constructs `OpenConfig { url }` for this store. `url` is that path.
    ///
    /// # Errors
    ///
    /// Returns [`StoreOpenError`] when the database cannot be opened.
    pub async fn open(path: PathBuf) -> Result<Self, StoreOpenError> {
        let url = path.to_string_lossy().into_owned();
        let config = OpenConfig::new(url);
        let sqlite = store_sqlite::open(&config).await.map_err(open_error)?;
        Ok(Self { sqlite })
    }

    async fn insert_widget_row(&self, name: &str) -> Result<RowText, OpError> {
        let row: WidgetRow = store_sqlite::insert_widget(&self.sqlite, name)
            .await
            .map_err(store_query)?;
        Ok(text_row(row))
    }

    async fn fetch_widget_row(&self, id: &str) -> Result<Option<RowText>, OpError> {
        Ok(store_sqlite::get_widget(&self.sqlite, id)
            .await
            .map_err(store_query)?
            .map(text_row))
    }

    async fn remove_widget_row(&self, id: &str) -> Result<Remove, OpError> {
        let row = store_sqlite::get_widget(&self.sqlite, id)
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
        recovery: "check the database path, permissions, and free disk",
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
