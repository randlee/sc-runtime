// Widget operations shared by every database layout.
//
// Each layout file includes this one, then defines `Stores`.

use api_types::{CreateWidget, DeleteWidget, DeletedRow, GetWidget, Widget, WidgetId, WidgetName};
use sc_command::{ErrorKind, OpError};
use sc_observability_types::ErrorCode;

const NOT_FOUND: &str = "WIDGET.NOT_FOUND";
const NAME_BLANK: &str = "WIDGET.NAME_BLANK";
const ROW_NOT_FOUND: &str = "ROW.NOT_FOUND";
const ROW_UNKNOWN_TABLE: &str = "ROW.UNKNOWN_TABLE";
const ROW_DELETE_UNSUPPORTED: &str = "ROW.DELETE_UNSUPPORTED";
const NOT_FOUND_CAUSE: &str = "the id is not in the store";
const NAME_BLANK_CAUSE: &str = "name is empty or only whitespace";
const STORE_RECOVERY: &str = "check the database path, permissions, and free disk; a pool acquire timeout means the store is saturated";
const WIDGETS: &str = "widgets";

/// Text of one widget row after the layout has read it.
struct RowText {
    id: String,
    name: String,
}

/// What a layout did with one widget row.
#[allow(dead_code)]
enum Remove {
    /// The id is not stored.
    Missing,
    /// The layout removed the row.
    Gone,
    /// The row is stored, and this store crate has no delete statement.
    Unsupported,
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
    let row = stores.insert_widget_row(name).await?;
    stored_widget(&row.id, &row.name)
}

/// Gets a widget by id.
///
/// # Errors
///
/// Returns `WIDGET.NOT_FOUND` when the id is absent, and an internal error
/// when the query fails.
pub async fn get_widget(stores: &Stores, input: GetWidget) -> Result<Widget, OpError> {
    let row = stores.fetch_widget_row(input.id.as_str()).await?;
    match row {
        Some(row) => stored_widget(&row.id, &row.name),
        None => Err(widget_error(
            ErrorKind::NotFound,
            NOT_FOUND,
            NOT_FOUND_CAUSE,
            "create the widget, then get the returned id",
        )),
    }
}

/// Deletes one row. `delete row <table> <id>` and `widget delete` both call this.
///
/// # Errors
///
/// Returns `ROW.UNKNOWN_TABLE` when `table` is not `widgets`, `ROW.NOT_FOUND`
/// when the id is absent or not a widget id, and `ROW.DELETE_UNSUPPORTED`
/// when the store crate has no delete statement.
pub async fn delete_row(stores: &Stores, table: &str, id: &str) -> Result<DeletedRow, OpError> {
    if table != WIDGETS {
        return Err(widget_error(
            ErrorKind::NotFound,
            ROW_UNKNOWN_TABLE,
            "the table is not stored by this layout",
            "delete a row from widgets",
        ));
    }
    if WidgetId::parse(id).is_err() {
        return Err(missing_row());
    }
    match stores.remove_widget_row(id).await? {
        Remove::Missing => Err(missing_row()),
        Remove::Unsupported => Err(widget_error(
            ErrorKind::Internal,
            ROW_DELETE_UNSUPPORTED,
            "the store crate has no delete statement",
            "the general delete command is registered; the store crate does not remove the row yet",
        )),
        Remove::Gone => Ok(DeletedRow {
            table: table.to_owned(),
            id: id.to_owned(),
        }),
    }
}

/// Application command for the same delete as [`delete_row`] on `widgets`.
///
/// # Errors
///
/// Returns the same errors as [`delete_row`] for table `widgets`.
pub async fn delete_widget(stores: &Stores, input: DeleteWidget) -> Result<DeletedRow, OpError> {
    delete_row(stores, WIDGETS, input.id.as_str()).await
}

fn missing_row() -> OpError {
    widget_error(
        ErrorKind::NotFound,
        ROW_NOT_FOUND,
        "the id is not in the table",
        "create the row, then delete the returned id",
    )
}

fn stored_widget(id: &str, name: &str) -> Result<Widget, OpError> {
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
    Ok(Widget { id, name })
}

fn widget_error(kind: ErrorKind, code: &'static str, cause: &str, action: &str) -> OpError {
    OpError::new(kind, ErrorCode::new_static(code), cause)
        .with_suggestion(action)
        .with_details(serde_json::json!({ "cause": cause }))
}
