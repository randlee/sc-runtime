include!("common.rs");

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

/// No database. Widget rows live in memory for the process.
#[derive(Clone, Debug)]
pub struct Stores {
    rows: Arc<Mutex<HashMap<String, String>>>,
}

impl Stores {
    /// Accepts `path` and stores nothing on disk.
    ///
    /// # Errors
    ///
    /// This layout does not open a database, so it does not return an error.
    pub async fn open(path: PathBuf) -> Result<Self, StoreOpenError> {
        let _ = path;
        Ok(Self {
            rows: Arc::new(Mutex::new(HashMap::new())),
        })
    }

    async fn insert_widget_row(&self, name: &str) -> Result<RowText, OpError> {
        let mut rows = self.rows.lock().unwrap_or_else(|err| err.into_inner());
        let id = format!("{:032x}", rows.len().saturating_add(1));
        rows.insert(id.clone(), name.to_owned());
        Ok(RowText {
            id,
            name: name.to_owned(),
        })
    }

    async fn fetch_widget_row(&self, id: &str) -> Result<Option<RowText>, OpError> {
        let rows = self.rows.lock().unwrap_or_else(|err| err.into_inner());
        Ok(rows.get(id).map(|name| RowText {
            id: id.to_owned(),
            name: name.clone(),
        }))
    }

    async fn remove_widget_row(&self, id: &str) -> Result<Remove, OpError> {
        let mut rows = self.rows.lock().unwrap_or_else(|err| err.into_inner());
        if rows.remove(id).is_none() {
            return Ok(Remove::Missing);
        }
        Ok(Remove::Gone)
    }
}
