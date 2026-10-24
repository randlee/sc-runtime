//! Widget create and get. Adapters call these functions and do not open SQL.

use std::path::PathBuf;

use api_types::Widget;
use sc_command::{ErrorKind, OpError};
use sc_observability_types::ErrorCode;
use sc_runtime::RuntimeError;

const NOT_FOUND: &str = "WIDGET.NOT_FOUND";
const NAME_BLANK: &str = "WIDGET.NAME_BLANK";
const NOT_FOUND_CAUSE: &str = "the id is not in the store";
const NAME_BLANK_CAUSE: &str = "name is empty or only whitespace";

/// Example stores value. v0.1 holds the SQLite store and nothing else.
#[derive(Debug, Clone)]
pub struct Stores {
    sqlite: store_sqlite::Store,
}

/// Opens the SQLite file and wraps a failed open as [`RuntimeError::StoresFailed`].
///
/// Store failures stay this runtime error. They are not widget codes.
///
/// # Errors
///
/// Returns [`RuntimeError::StoresFailed`] when the database cannot be opened.
pub async fn open_stores(path: PathBuf) -> Result<Stores, RuntimeError> {
    let sqlite = store_sqlite::open(&store_sqlite::OpenConfig { path })
        .await
        .map_err(|err| {
            RuntimeError::stores_failed(
                "STORE.OPEN",
                "the widget database did not open",
                "check the database path",
                Some(err.to_string()),
            )
        })?;
    Ok(Stores { sqlite })
}

/// Creates a widget. A duplicate name is allowed.
///
/// # Errors
///
/// Returns `WIDGET.NAME_BLANK` when `name` is empty or only whitespace, and an
/// internal error when the insert fails.
pub async fn create_widget(stores: &Stores, name: &str) -> Result<Widget, OpError> {
    if name.trim().is_empty() {
        return Err(widget_error(
            ErrorKind::InvalidInput,
            NAME_BLANK,
            NAME_BLANK_CAUSE,
            "send a non-empty name",
        ));
    }
    let row = store_sqlite::insert_widget(&stores.sqlite, name)
        .await
        .map_err(store_query)?;
    Ok(Widget {
        id: row.id,
        name: row.name,
    })
}

/// Gets a widget by id.
///
/// # Errors
///
/// Returns `WIDGET.NOT_FOUND` when the id is absent, and an internal error
/// when the query fails.
pub async fn get_widget(stores: &Stores, id: &str) -> Result<Widget, OpError> {
    let row = store_sqlite::get_widget(&stores.sqlite, id)
        .await
        .map_err(store_query)?;
    row.map(|row| Widget {
        id: row.id,
        name: row.name,
    })
    .ok_or_else(|| {
        widget_error(
            ErrorKind::NotFound,
            NOT_FOUND,
            NOT_FOUND_CAUSE,
            "create the widget, then get the returned id",
        )
    })
}

fn widget_error(kind: ErrorKind, code: &'static str, cause: &str, action: &str) -> OpError {
    OpError::new(kind, ErrorCode::new_static(code), cause)
        .with_suggestion(action)
        .with_details(serde_json::json!({ "cause": cause }))
}

fn store_query(err: store_sqlite::StoreError) -> OpError {
    OpError::new(
        ErrorKind::Internal,
        ErrorCode::new_static("STORE.QUERY"),
        err.to_string(),
    )
}
