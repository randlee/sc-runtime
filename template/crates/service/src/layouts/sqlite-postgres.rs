include!("common.rs");
include!("store_map.rs");

use std::path::PathBuf;

use schema::WidgetRow;

/// SQLite plus Postgres. Service depends on the adaptor, not on the store crates.
#[derive(Debug, Clone)]
pub struct Stores {
    sqlite: store_adaptor::SqliteStore,
    remote: store_adaptor::PostgresStore,
    flags: store_adaptor::TableFlags,
}

impl Stores {
    /// Opens SQLite at `sqlite` and Postgres at `remote`.
    ///
    /// `sqlite` is a filesystem path. `remote` is a Postgres URL. Each value
    /// becomes its own `OpenConfig { url }`. This function does not read a
    /// `postgres://` prefix to choose an engine, and it does not open SQLite
    /// with the remote URL.
    ///
    /// # Errors
    ///
    /// Returns [`StoreOpenError`] when membership rejects `widgets` or either
    /// database cannot be opened.
    pub async fn open(sqlite: PathBuf, remote: String) -> Result<Self, StoreOpenError> {
        let flags = store_adaptor::layout_flags().map_err(membership_error)?;
        flags
            .ensure_shared_table("widgets")
            .map_err(membership_error)?;
        let sqlite = store_adaptor::open_sqlite(&schema::OpenConfig::sqlite(sqlite))
            .await
            .map_err(open_error)?;
        let remote = store_adaptor::open_postgres(&schema::OpenConfig::postgres(remote))
            .await
            .map_err(open_error)?;
        Ok(Self {
            sqlite,
            remote,
            flags,
        })
    }

    async fn check_connectivity(&self) -> Result<(), OpError> {
        store_adaptor::probe_sqlite(&self.sqlite)
            .await
            .map_err(store_query)?;
        store_adaptor::probe_postgres(&self.remote)
            .await
            .map_err(store_query)
    }

    async fn insert_widget_row(&self, name: &str) -> Result<RowText, OpError> {
        let row: WidgetRow = store_adaptor::insert_widget(&self.sqlite, name)
            .await
            .map_err(store_query)?;
        store_adaptor::copy_shared_rows(
            &self.sqlite,
            &self.remote,
            &schema::TableName::widgets(),
            &self.flags,
            &store_adaptor::CopyRollup,
        )
        .await
        .map_err(store_query)?;
        text_row(row)
    }

    async fn fetch_widget_row(&self, id: &str) -> Result<Option<RowText>, OpError> {
        match store_adaptor::get_widget(&self.sqlite, id)
            .await
            .map_err(store_query)?
        {
            Some(row) => Ok(Some(text_row(row)?)),
            None => Ok(None),
        }
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
