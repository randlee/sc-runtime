include!("common.rs");

use std::path::PathBuf;

use schema::{OpenConfig, WidgetRow};

/// Postgres-only stores. There is no adaptor and no SQLite crate.
#[derive(Debug, Clone)]
pub struct Stores {
    postgres: store_postgres::Store,
}

impl Stores {
    /// Opens the Postgres database at `path`.
    ///
    /// Constructs `OpenConfig { url }` for this store. `url` is a Postgres URL.
    ///
    /// # Errors
    ///
    /// Returns [`StoreOpenError`] when the database cannot be opened.
    pub async fn open(path: PathBuf) -> Result<Self, StoreOpenError> {
        let url = path.to_string_lossy().into_owned();
        let config = OpenConfig::new(url);
        let postgres = store_postgres::open(&config).await.map_err(open_error)?;
        Ok(Self { postgres })
    }

    async fn insert_widget_row(&self, name: &str) -> Result<RowText, OpError> {
        let row: WidgetRow = store_postgres::insert_widget(&self.postgres, name)
            .await
            .map_err(store_query)?;
        Ok(text_row(row))
    }

    async fn fetch_widget_row(&self, id: &str) -> Result<Option<RowText>, OpError> {
        Ok(store_postgres::get_widget(&self.postgres, id)
            .await
            .map_err(store_query)?
            .map(text_row))
    }

    async fn remove_widget_row(&self, id: &str) -> Result<Remove, OpError> {
        let row = store_postgres::get_widget(&self.postgres, id)
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
        recovery: "check the database URL, credentials, and that Postgres is running",
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
