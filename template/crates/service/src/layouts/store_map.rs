// Maps `schema::StoreError` into the service error types.
//
// Store layouts include this file. The no-database layout does not. The
// comment is `//` because this file is included after `common.rs`.

fn store_code_name(code: schema::StoreCode) -> &'static str {
    match code {
        schema::StoreCode::Connection => "STORE.CONNECTION",
        schema::StoreCode::Query => "STORE.QUERY",
        schema::StoreCode::Migration => "STORE.MIGRATION",
        schema::StoreCode::NotFound => "STORE.NOT_FOUND",
        schema::StoreCode::Conflict => "STORE.CONFLICT",
        schema::StoreCode::Unavailable => "STORE.UNAVAILABLE",
        schema::StoreCode::Membership => "STORE.MEMBERSHIP",
        _ => "STORE.QUERY",
    }
}

fn store_kind(code: schema::StoreCode) -> ErrorKind {
    match code {
        schema::StoreCode::NotFound => ErrorKind::NotFound,
        schema::StoreCode::Conflict => ErrorKind::Conflict,
        schema::StoreCode::Unavailable | schema::StoreCode::Connection => ErrorKind::Unavailable,
        schema::StoreCode::Query
        | schema::StoreCode::Migration
        | schema::StoreCode::Membership => ErrorKind::Internal,
        _ => ErrorKind::Internal,
    }
}

fn open_error(err: schema::StoreError) -> StoreOpenError {
    let code = store_code_name(err.code);
    let recovery = err.remediation();
    let detail = err.to_string();
    StoreOpenError {
        code,
        cause: err.message,
        recovery,
        detail,
    }
}

/// Membership failures keep the store code and say the file that failed.
///
/// One-store layouts include this file and do not read `membership.toml`.
#[allow(dead_code)]
fn membership_error(err: schema::StoreError) -> StoreOpenError {
    let code = store_code_name(err.code);
    let recovery = err.remediation();
    let detail = err.to_string();
    StoreOpenError {
        code,
        cause: format!("membership.toml: {}", err.message),
        recovery,
        detail,
    }
}

fn store_query(err: schema::StoreError) -> OpError {
    let code = store_code_name(err.code);
    let kind = store_kind(err.code);
    let retryable = err.is_retryable();
    let recovery = err.remediation();
    OpError::new(
        kind,
        sc_observability_types::ErrorCode::new_static(code),
        err.message,
    )
    .with_suggestion(recovery)
    .with_details(serde_json::json!({
        "code": code,
        "retryable": retryable,
    }))
}

fn text_row(row: schema::WidgetRow) -> Result<RowText, OpError> {
    let id = WidgetId::parse(row.id.as_str()).map_err(|_| {
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
    Ok(RowText { id, name })
}
