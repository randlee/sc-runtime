//! HTTP and MCP conversions. Compiled only with feature `server`.

#![cfg(feature = "server")]

use axum::body::Body;
use axum::http::{HeaderValue, StatusCode, header};
use axum::response::{IntoResponse, Response};
use rmcp::ErrorData;
use rmcp::model::CallToolResult;
use sc_observability_types::ErrorCode;
use serde::Serialize;

use crate::{Envelope, ErrorKind, OpError, SERIALIZATION_CODE};

/// Wire `suggested_action` for [`SERIALIZATION_CODE`].
///
/// The static fallback body uses this same sentence so a second
/// serialization failure still tells the caller what to do.
const SERIALIZATION_RECOVERY: &str = "Return a payload that serializes to JSON. The HTTP edge responds 500 with this code and does not panic. The MCP edge returns rmcp ErrorData internal error carrying this code and does not panic.";

/// rmcp tool error.
///
/// rmcp 3.7 names this type [`ErrorData`]. Tool handlers import that type as
/// `McpError`. This alias is the name used by [`IntoMcp::into_mcp`].
pub type McpError = ErrorData;

mod seal {
    pub trait Seal {}
}

/// Converts `Result<T, OpError>` into an rmcp tool result.
///
/// This trait is sealed by a private supertrait. The only implementation is
/// `Result<T, OpError>`. Downstream crates can call
/// [`into_mcp`](Self::into_mcp) and cannot implement the trait.
///
/// Dispatch is static. `into_mcp` takes `self` by value and the trait requires
/// `Sized`, so `dyn IntoMcp` is not a supported API.
///
/// An [`OpError`] becomes `Ok(CallToolResult)` with `is_error` set. It does
/// not become `Err(McpError)`. `Err` is reserved for a payload that fails to
/// serialize; that error is rmcp's internal error and carries
/// [`SERIALIZATION_CODE`].
///
/// # Examples
///
/// ```
/// use sc_command::IntoMcp;
///
/// let result: Result<&str, sc_command::OpError> = Ok("ready");
/// let tool = result.into_mcp()?;
/// assert_ne!(tool.is_error, Some(true));
/// # Ok::<(), sc_command::McpError>(())
/// ```
///
/// ```compile_fail
/// struct Rogue;
/// impl sc_command::IntoMcp for Rogue {
///     fn into_mcp(self) -> Result<rmcp::model::CallToolResult, sc_command::McpError> {
///         Ok(rmcp::model::CallToolResult::default())
///     }
/// }
/// ```
///
/// ```compile_fail
/// fn refuses_dyn(_: &dyn sc_command::IntoMcp) {}
/// ```
pub trait IntoMcp: seal::Seal + Sized {
    /// Converts this result into an rmcp tool result that carries the envelope.
    ///
    /// # Errors
    ///
    /// Returns [`McpError`] when the envelope cannot be serialized. An
    /// [`OpError`] is not an error of this function; it is a tool result with
    /// `is_error` set.
    fn into_mcp(self) -> Result<CallToolResult, McpError>;
}

impl<T> seal::Seal for Result<T, OpError> where T: Serialize {}

impl<T> IntoMcp for Result<T, OpError>
where
    T: Serialize,
{
    fn into_mcp(self) -> Result<CallToolResult, McpError> {
        let envelope = Envelope::from(self);
        let failed = !envelope.is_ok();
        let value = serde_json::to_value(&envelope).map_err(|error| {
            McpError::internal_error(
                "response envelope could not be serialized",
                Some(serde_json::json!({
                    "code": SERIALIZATION_CODE,
                    "cause": error.to_string(),
                })),
            )
        })?;
        let result = if failed {
            CallToolResult::structured_error(value)
        } else {
            CallToolResult::structured(value)
        };
        Ok(result)
    }
}

impl<T> IntoResponse for Envelope<T>
where
    T: Serialize,
{
    fn into_response(self) -> Response {
        let status = match &self {
            Envelope::Success { .. } => StatusCode::OK,
            Envelope::Failure { error, .. } => status_code(error.kind),
        };
        match serde_json::to_vec(&self) {
            Ok(body) => json_response(status, body),
            Err(error) => json_response(StatusCode::INTERNAL_SERVER_ERROR, failure_body(&error)),
        }
    }
}

fn status_code(kind: ErrorKind) -> StatusCode {
    StatusCode::from_u16(kind.http_status_code()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR)
}

fn json_response(status: StatusCode, body: Vec<u8>) -> Response {
    match Response::builder()
        .status(status)
        .header(
            header::CONTENT_TYPE,
            HeaderValue::from_static("application/json"),
        )
        .body(Body::from(body))
    {
        Ok(response) => response,
        Err(_) => {
            let mut response = Response::new(Body::from(static_failure_body()));
            *response.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
            response.headers_mut().insert(
                header::CONTENT_TYPE,
                HeaderValue::from_static("application/json"),
            );
            response
        }
    }
}

fn failure_body(error: &serde_json::Error) -> Vec<u8> {
    let envelope = Envelope::<()>::failure(serialization_error(error));
    serde_json::to_vec(&envelope).unwrap_or_else(|_| static_failure_body())
}

fn serialization_error(error: &serde_json::Error) -> OpError {
    OpError::new(
        ErrorKind::Internal,
        ErrorCode::new_static(SERIALIZATION_CODE),
        "response envelope could not be serialized",
    )
    .with_details(serde_json::json!({ "cause": error.to_string() }))
    .with_suggestion(SERIALIZATION_RECOVERY)
}

fn static_failure_body() -> Vec<u8> {
    br#"{"version":"1","ok":false,"data":null,"error":{"kind":"internal","code":"ENVELOPE.SERIALIZATION","message":"response envelope could not be serialized","details":null,"suggested_action":"Return a payload that serializes to JSON. The HTTP edge responds 500 with this code and does not panic. The MCP edge returns rmcp ErrorData internal error carrying this code and does not panic."}}"#
        .to_vec()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn static_failure_body_matches_the_serialization_suggestion() {
        let envelope = Envelope::<()>::failure(
            OpError::new(
                ErrorKind::Internal,
                ErrorCode::new_static(SERIALIZATION_CODE),
                "response envelope could not be serialized",
            )
            .with_suggestion(SERIALIZATION_RECOVERY),
        );
        let rendered = serde_json::to_vec(&envelope).expect("envelope");
        assert_eq!(rendered, static_failure_body());
        let parse_error = serde_json::from_str::<u32>("nope").expect_err("not a number");
        assert_eq!(
            serialization_error(&parse_error).suggested_action(),
            Some(SERIALIZATION_RECOVERY)
        );
    }
}
