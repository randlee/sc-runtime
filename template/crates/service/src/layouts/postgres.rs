include!("common.rs");
include!("store_map.rs");

use schema::WidgetRow;

/// Postgres-only stores. There is no adaptor and no SQLite crate.
#[derive(Debug, Clone)]
pub struct Stores {
    postgres: store_postgres::Store,
}

impl Stores {
    /// Opens the Postgres database at `url`.
    ///
    /// Constructs `OpenConfig { url }` for this store. `url` is a `postgres://`
    /// or `postgresql://` URL. This function does not join `url` onto a
    /// filesystem path.
    ///
    /// # Errors
    ///
    /// Returns [`StoreOpenError`] when the database cannot be opened.
    pub async fn open(url: String) -> Result<Self, StoreOpenError> {
        let config = schema::OpenConfig::new(url);
        let postgres = store_postgres::open(&config).await.map_err(open_error)?;
        Ok(Self { postgres })
    }

    async fn insert_widget_row(&self, name: &str) -> Result<RowText, OpError> {
        let row: WidgetRow = store_postgres::insert_widget(&self.postgres, name)
            .await
            .map_err(store_query)?;
        text_row(row)
    }

    async fn fetch_widget_row(&self, id: &str) -> Result<Option<RowText>, OpError> {
        match store_postgres::get_widget(&self.postgres, id)
            .await
            .map_err(store_query)?
        {
            Some(row) => Ok(Some(text_row(row)?)),
            None => Ok(None),
        }
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
