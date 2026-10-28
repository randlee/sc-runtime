include!("common.rs");
include!("store_map.rs");

use std::path::PathBuf;

use schema::WidgetRow;

/// SQLite-only stores. There is no adaptor.
#[derive(Debug, Clone)]
pub struct Stores {
    sqlite: store_sqlite::Store,
}

impl Stores {
    /// Opens the SQLite file at `path`.
    ///
    /// Constructs `OpenConfig { url }` for this store. `url` is that filesystem
    /// path. A relative path is already joined to the instance root by the daemon.
    ///
    /// # Errors
    ///
    /// Returns [`StoreOpenError`] when the database cannot be opened.
    pub async fn open(path: PathBuf) -> Result<Self, StoreOpenError> {
        let config = schema::OpenConfig::sqlite(path);
        let sqlite = store_sqlite::open(&config).await.map_err(open_error)?;
        Ok(Self { sqlite })
    }

    async fn check_connectivity(&self) -> Result<(), OpError> {
        store_sqlite::probe(&self.sqlite).await.map_err(store_query)
    }

    async fn insert_widget_row(&self, name: &str) -> Result<RowText, OpError> {
        let row: WidgetRow = store_sqlite::insert_widget(&self.sqlite, name)
            .await
            .map_err(store_query)?;
        text_row(row)
    }

    async fn fetch_widget_row(&self, id: &str) -> Result<Option<RowText>, OpError> {
        match store_sqlite::get_widget(&self.sqlite, id)
            .await
            .map_err(store_query)?
        {
            Some(row) => Ok(Some(text_row(row)?)),
            None => Ok(None),
        }
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
