//! Widget create and get. Adapters call these functions and do not open SQL.

use std::path::PathBuf;

use api_types::{CreateWidget, GetWidget, Widget, WidgetId, WidgetName};
use sc_command::{ErrorKind, OpError};
use sc_observability_types::ErrorCode;

const NOT_FOUND: &str = "WIDGET.NOT_FOUND";
const NAME_BLANK: &str = "WIDGET.NAME_BLANK";
const NOT_FOUND_CAUSE: &str = "the id is not in the store";
const NAME_BLANK_CAUSE: &str = "name is empty or only whitespace";
const STORE_RECOVERY: &str = "check the database path, permissions, and free disk; a pool acquire timeout means the store is saturated";

/// Example stores value. v0.1 holds the SQLite store and nothing else.
#[derive(Debug, Clone)]
pub struct Stores {
    sqlite: store_sqlite::Store,
}

/// Failure opening the widget database.
///
/// The daemon maps this into `sc_runtime::RuntimeError`. This crate does not
/// depend on `sc-runtime`.
#[derive(Debug)]
pub struct StoreOpenError {
    /// Stable code. v0.1 uses `STORE.OPEN`.
    pub code: &'static str,
    /// What happened.
    pub cause: &'static str,
    /// How to recover.
    pub recovery: &'static str,
    /// Store detail.
    pub detail: String,
}

impl std::fmt::Display for StoreOpenError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "{} {} — {}",
            self.code, self.cause, self.recovery
        )
    }
}

impl std::error::Error for StoreOpenError {}

impl Stores {
    /// Opens the SQLite file at `path`.
    ///
    /// Store-open failures stay on this type. They are not widget codes.
    ///
    /// # Errors
    ///
    /// Returns [`StoreOpenError`] when the database cannot be opened.
    pub async fn open(path: PathBuf) -> Result<Self, StoreOpenError> {
        let sqlite = store_sqlite::open(&store_sqlite::OpenConfig { path })
            .await
            .map_err(|err| StoreOpenError {
                code: "STORE.OPEN",
                cause: "the widget database did not open",
                recovery: "check the database path, permissions, and free disk",
                detail: err.to_string(),
            })?;
        Ok(Self { sqlite })
    }
}

/// Creates a widget. A duplicate name is allowed.
///
/// # Errors
///
/// Returns `WIDGET.NAME_BLANK` when `name` is empty or only whitespace, and an
/// internal error when the insert fails.
pub async fn create_widget(stores: &Stores, input: CreateWidget) -> Result<Widget, OpError> {
    let name = input.name.as_str();
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
    stored_widget(row)
}

/// Gets a widget by id.
///
/// # Errors
///
/// Returns `WIDGET.NOT_FOUND` when the id is absent, and an internal error
/// when the query fails.
pub async fn get_widget(stores: &Stores, input: GetWidget) -> Result<Widget, OpError> {
    let row = store_sqlite::get_widget(&stores.sqlite, input.id.as_str())
        .await
        .map_err(store_query)?;
    match row {
        Some(row) => stored_widget(row),
        None => Err(widget_error(
            ErrorKind::NotFound,
            NOT_FOUND,
            NOT_FOUND_CAUSE,
            "create the widget, then get the returned id",
        )),
    }
}

fn stored_widget(row: store_sqlite::WidgetRow) -> Result<Widget, OpError> {
    let id = WidgetId::parse(&row.id).map_err(|_| {
        widget_error(
            ErrorKind::Internal,
            "STORE.QUERY",
            "the stored id is not a widget id",
            STORE_RECOVERY,
        )
    })?;
    let name = WidgetName::parse(&row.name).map_err(|_| {
        widget_error(
            ErrorKind::Internal,
            "STORE.QUERY",
            "the stored name is longer than the widget name limit",
            STORE_RECOVERY,
        )
    })?;
    Ok(Widget { id, name })
}

fn widget_error(kind: ErrorKind, code: &'static str, cause: &str, action: &str) -> OpError {
    OpError::new(kind, ErrorCode::new_static(code), cause)
        .with_suggestion(action)
        .with_details(serde_json::json!({ "cause": cause }))
}

fn store_query(err: store_sqlite::StoreError) -> OpError {
    let cause = err.to_string();
    OpError::new(
        ErrorKind::Internal,
        ErrorCode::new_static("STORE.QUERY"),
        cause.clone(),
    )
    .with_suggestion(STORE_RECOVERY)
    .with_details(serde_json::json!({ "cause": cause }))
}
