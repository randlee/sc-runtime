//! Server-feature HTTP and MCP conversion tests.

use axum::body::to_bytes;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use rmcp::model::CallToolResult;
use rmcp::ErrorData as McpError;
use sc_command::{Envelope, ErrorKind, IntoMcp, OpError, SERIALIZATION_CODE};
use sc_observability_types::ErrorCode;
use serde::ser::SerializeStruct;
use serde::{Deserialize, Serialize, Serializer};
use serde_json::{json, Value};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct Sample {
    name: String,
}

fn sample() -> Sample {
    Sample {
        name: "sample".to_owned(),
    }
}

fn error_of(kind: ErrorKind) -> OpError {
    OpError::new(kind, ErrorCode::new_static("SAMPLE.FAILED"), "failed")
}

struct Boom;

impl Serialize for Boom {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("Boom", 1)?;
        state.serialize_field("nope", &BoomField)?;
        state.end()
    }
}

struct BoomField;

impl Serialize for BoomField {
    fn serialize<S>(&self, _serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        Err(serde::ser::Error::custom("boom"))
    }
}

async fn response_json(envelope: Envelope<impl Serialize>) -> (StatusCode, Value) {
    let response = envelope.into_response();
    let status = response.status();
    let content_type = response
        .headers()
        .get(axum::http::header::CONTENT_TYPE)
        .expect("content-type")
        .to_str()
        .expect("content-type text");
    assert_eq!(content_type, "application/json");
    let bytes = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("body");
    let value = serde_json::from_slice(&bytes).expect("json body");
    (status, value)
}

fn envelope_from_tool(result: &CallToolResult) -> Value {
    result
        .structured_content
        .clone()
        .expect("structured envelope")
}

#[tokio::test]
async fn success_response_is_200_json() {
    let envelope = Envelope::success(sample());
    let (status, body) = response_json(envelope.clone()).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        body,
        serde_json::to_value(&envelope).expect("envelope json")
    );
}

#[tokio::test]
async fn every_kind_maps_to_its_status() {
    for kind in ErrorKind::all() {
        let envelope = Envelope::<Sample>::failure(error_of(*kind));
        let (status, body) = response_json(envelope.clone()).await;
        assert_eq!(status.as_u16(), kind.http_status_code());
        assert_eq!(body["ok"], false);
        assert_eq!(
            body,
            serde_json::to_value(&envelope).expect("envelope json")
        );
        let range = if kind.is_caller_error() {
            400..500
        } else {
            500..600
        };
        assert!(range.contains(&status.as_u16()));
    }
}

#[tokio::test]
async fn serialization_failure_returns_500_without_panicking() {
    let envelope = Envelope::success(Boom);
    let (status, body) = response_json(envelope).await;
    assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
    let cause = body["error"]["details"]["cause"]
        .as_str()
        .expect("cause")
        .to_owned();
    assert!(cause.contains("boom"));
    let expected = serde_json::to_value(Envelope::<()>::failure(
        OpError::new(
            ErrorKind::Internal,
            ErrorCode::new_static(SERIALIZATION_CODE),
            "response envelope could not be serialized",
        )
        .with_details(json!({ "cause": cause })),
    ))
    .expect("failure envelope");
    assert_eq!(body, expected);
}

#[test]
fn success_tool_result_carries_the_envelope() {
    let result = Ok::<_, OpError>(sample()).into_mcp().expect("mcp");
    assert!(matches!(result.is_error, None | Some(false)));
    let envelope = envelope_from_tool(&result);
    assert_eq!(envelope["ok"], true);
    assert_eq!(envelope["data"]["name"], "sample");
    let response = Envelope::from(Ok::<_, OpError>(sample())).into_response();
    let body = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("runtime")
        .block_on(async move {
            let bytes = to_bytes(response.into_body(), usize::MAX)
                .await
                .expect("body");
            serde_json::from_slice::<Value>(&bytes).expect("json")
        });
    assert_eq!(envelope, body);
}

#[test]
fn failure_tool_result_sets_is_error_and_matches_http() {
    let op_error = error_of(ErrorKind::NotFound);
    let result = Err::<Sample, _>(op_error.clone()).into_mcp().expect("mcp");
    assert_eq!(result.is_error, Some(true));
    let envelope = envelope_from_tool(&result);
    assert_eq!(envelope["ok"], false);
    assert!(envelope["data"].is_null());
    assert_eq!(
        envelope["error"],
        serde_json::to_value(&op_error).expect("error")
    );
    let response = Envelope::from(Err::<Sample, _>(op_error)).into_response();
    let body = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("runtime")
        .block_on(async move {
            let bytes = to_bytes(response.into_body(), usize::MAX)
                .await
                .expect("body");
            serde_json::from_slice::<Value>(&bytes).expect("json")
        });
    assert_eq!(envelope, body);
}

#[test]
fn serialization_failure_on_mcp_returns_error_without_panicking() {
    let error = Ok::<_, OpError>(Boom).into_mcp().expect_err("mcp error");
    assert_mcp_serialization(&error);
}

fn assert_mcp_serialization(error: &McpError) {
    let data = error.data.clone().expect("data");
    assert_eq!(data["code"], SERIALIZATION_CODE);
}

#[test]
fn downstream_crate_calls_into_mcp() {
    let result: Result<(), OpError> = Ok(());
    let tool = result.into_mcp().expect("call");
    assert_ne!(tool.is_error, Some(true));
    let _ = json!({"ok": true});
}
