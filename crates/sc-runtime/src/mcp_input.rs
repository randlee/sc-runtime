//! Bounded JSON syntax validation before MCP normalizes request arguments.

use std::collections::HashSet;
use std::fmt;

use axum::body::Body;
use axum::extract::Request;
use axum::http::{Method, StatusCode, header};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use serde::de::{self, MapAccess, SeqAccess, Visitor};
use serde::{Deserialize, Deserializer};

/// Keeps the ordinary MCP service's own body limit and original request data.
pub(crate) async fn validate(request: Request, next: Next, limit: usize) -> Response {
    let json = request
        .headers()
        .get(header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .is_some_and(|value| {
            value
                .split(';')
                .next()
                .is_some_and(|mime| mime.trim().eq_ignore_ascii_case("application/json"))
        });
    if request.method() != Method::POST || !json {
        return next.run(request).await;
    }
    let (parts, body) = request.into_parts();
    let bytes = match axum::body::to_bytes(body, limit).await {
        Ok(bytes) => bytes,
        Err(_) => {
            return reject(
                StatusCode::PAYLOAD_TOO_LARGE,
                -32600,
                "MCP request body cannot be read within its configured limit",
            );
        }
    };
    if serde_json::from_slice::<UniqueJson>(&bytes).is_err() {
        return reject(
            StatusCode::BAD_REQUEST,
            -32700,
            "invalid JSON or duplicate object key",
        );
    }
    // RMCP still owns protocol/DTO interpretation. Reusing Parts and the exact
    // bytes preserves headers, caller extensions and the original deadline.
    next.run(Request::from_parts(parts, Body::from(bytes)))
        .await
}

fn reject(status: StatusCode, code: i32, message: &'static str) -> Response {
    (
        status,
        axum::Json(serde_json::json!({
            "jsonrpc":"2.0", "id":null, "error":{"code":code,"message":message}
        })),
    )
        .into_response()
}

/// Syntax-only visitor: no second wire model and no retained payload values.
struct UniqueJson;

impl<'de> Deserialize<'de> for UniqueJson {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserializer.deserialize_any(Self)
    }
}

impl<'de> Visitor<'de> for UniqueJson {
    type Value = UniqueJson;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("JSON with unique keys in each object")
    }

    fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Self::Value, A::Error> {
        let mut keys = HashSet::new();
        while let Some(key) = map.next_key::<String>()? {
            if !keys.insert(key) {
                return Err(de::Error::custom("duplicate JSON object key"));
            }
            map.next_value::<UniqueJson>()?;
        }
        Ok(UniqueJson)
    }

    fn visit_seq<A: SeqAccess<'de>>(self, mut sequence: A) -> Result<Self::Value, A::Error> {
        while sequence.next_element::<UniqueJson>()?.is_some() {}
        Ok(UniqueJson)
    }

    fn visit_bool<E: de::Error>(self, _: bool) -> Result<Self::Value, E> {
        Ok(UniqueJson)
    }
    fn visit_i64<E: de::Error>(self, _: i64) -> Result<Self::Value, E> {
        Ok(UniqueJson)
    }
    fn visit_u64<E: de::Error>(self, _: u64) -> Result<Self::Value, E> {
        Ok(UniqueJson)
    }
    fn visit_f64<E: de::Error>(self, _: f64) -> Result<Self::Value, E> {
        Ok(UniqueJson)
    }
    fn visit_str<E: de::Error>(self, _: &str) -> Result<Self::Value, E> {
        Ok(UniqueJson)
    }
    fn visit_unit<E: de::Error>(self) -> Result<Self::Value, E> {
        Ok(UniqueJson)
    }
}

#[cfg(test)]
mod tests {
    use super::UniqueJson;

    #[test]
    fn distinct_objects_and_quoted_json_preserve_valid_input() {
        for input in [
            r#"{"a":{"x":1},"b":{"x":2},"text":"{\"x\":1,\"x\":2}"}"#,
            r#"[null,true,false,-1,1,1.25,"text",{},[]]"#,
        ] {
            assert!(serde_json::from_str::<UniqueJson>(input).is_ok());
        }
    }

    #[test]
    fn nested_array_and_unicode_equivalent_duplicate_keys_are_rejected() {
        for input in [
            r#"{"request":{"limit":1,"limit":2}}"#,
            r#"[{"x":1,"x":2}]"#,
            r#"{"name":1,"n\u0061me":2}"#,
        ] {
            assert!(serde_json::from_str::<UniqueJson>(input).is_err());
        }
    }
}
