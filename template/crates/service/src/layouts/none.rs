include!("common.rs");

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// In-memory rows. The mutex is `std::sync::Mutex` because the guard is never
/// held across `.await`: each method locks, updates the map, and drops the
/// guard before it returns. A multi-thread Tokio runtime may clone `Stores`
/// and call it from several tasks. Those tasks do not need `tokio::sync::Mutex`.
#[derive(Debug)]
struct Memory {
    /// Next id. Delete does not decrease this counter, so an id is not reused.
    next_id: u64,
    rows: HashMap<String, String>,
}

/// No database. Widget rows live in memory for the process.
#[derive(Clone, Debug)]
pub struct Stores {
    rows: Arc<Mutex<Memory>>,
}

impl Stores {
    /// Opens an empty in-memory store.
    ///
    /// This layout has no database, so opening cannot fail. The return type
    /// is `Self`, not a `Result`.
    pub async fn open() -> Self {
        Self {
            rows: Arc::new(Mutex::new(Memory {
                next_id: 0,
                rows: HashMap::new(),
            })),
        }
    }

    async fn check_connectivity(&self) -> Result<(), OpError> {
        let _ = self;
        Ok(())
    }

    async fn insert_widget_row(&self, name: &str) -> Result<RowText, OpError> {
        let mut rows = self.rows.lock().unwrap_or_else(|err| err.into_inner());
        rows.next_id = rows.next_id.saturating_add(1);
        let raw = format!("{:032x}", rows.next_id);
        let id = WidgetId::parse(&raw).map_err(|_| {
            widget_error(
                ErrorKind::Internal,
                "STORE.QUERY",
                "the stored id is not a widget id",
                STORE_RECOVERY,
            )
        })?;
        let stored_name = WidgetName::parse(name).map_err(|_| {
            widget_error(
                ErrorKind::Internal,
                "STORE.QUERY",
                "the stored name is longer than the widget name limit",
                STORE_RECOVERY,
            )
        })?;
        rows.rows.insert(raw, stored_name.as_str().to_owned());
        drop(rows);
        Ok(RowText {
            id,
            name: stored_name,
        })
    }

    async fn fetch_widget_row(&self, id: &str) -> Result<Option<RowText>, OpError> {
        let rows = self.rows.lock().unwrap_or_else(|err| err.into_inner());
        let Some(name) = rows.rows.get(id) else {
            return Ok(None);
        };
        let id = WidgetId::parse(id).map_err(|_| {
            widget_error(
                ErrorKind::Internal,
                "STORE.QUERY",
                "the stored id is not a widget id",
                STORE_RECOVERY,
            )
        })?;
        let name = WidgetName::parse(name).map_err(|_| {
            widget_error(
                ErrorKind::Internal,
                "STORE.QUERY",
                "the stored name is longer than the widget name limit",
                STORE_RECOVERY,
            )
        })?;
        Ok(Some(RowText { id, name }))
    }

    async fn remove_widget_row(&self, id: &str) -> Result<Remove, OpError> {
        let mut rows = self.rows.lock().unwrap_or_else(|err| err.into_inner());
        if rows.rows.remove(id).is_none() {
            return Ok(Remove::Missing);
        }
        Ok(Remove::Gone)
    }
}

#[cfg(test)]
mod tests {
    #[tokio::test]
    async fn delete_does_not_reuse_widget_ids() {
        let stores = super::Stores::open().await;
        let first = stores.insert_widget_row("one").await.expect("one");
        let second = stores.insert_widget_row("two").await.expect("two");
        stores
            .remove_widget_row(first.id.as_str())
            .await
            .expect("delete");
        let third = stores.insert_widget_row("three").await.expect("three");
        assert_ne!(third.id, first.id);
        assert_ne!(third.id, second.id);
        let still = stores
            .fetch_widget_row(second.id.as_str())
            .await
            .expect("fetch");
        assert!(still.is_some());
    }
}
