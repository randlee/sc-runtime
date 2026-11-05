//! Retain bounded original HTTP bytes before RMCP normalizes tool arguments.

use axum::body::{Body, Bytes};
use axum::extract::Request;
use axum::http::{Method, StatusCode, header, request::Parts};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};

#[derive(Clone)]
struct OriginalBody(Bytes);

/// Reads original MCP HTTP JSON bytes retained before argument normalization.
///
/// Application adapters may deserialize their own canonical request from these
/// bytes when duplicate struct fields matter. The application owns validation;
/// opaque JSON values and unrelated tools receive no extra parsing policy.
/// Returns `None` for requests outside the JSON POST path mounted by `.mcp`.
/// The service's configured body limit and the original request deadline apply.
pub fn mcp_request_body(parts: &Parts) -> Option<&[u8]> {
    parts
        .extensions
        .get::<OriginalBody>()
        .map(|body| body.0.as_ref())
}

pub(crate) async fn retain(request: Request, next: Next, limit: usize) -> Response {
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
    let (mut parts, body) = request.into_parts();
    let bytes = match axum::body::to_bytes(body, limit).await {
        Ok(bytes) => bytes,
        Err(_) => {
            return (StatusCode::PAYLOAD_TOO_LARGE, [(header::CONTENT_TYPE, "application/json")],
                r#"{"jsonrpc":"2.0","id":null,"error":{"code":-32600,"message":"MCP request body cannot be read within its configured limit"}}"#).into_response();
        }
    };
    parts.extensions.insert(OriginalBody(bytes.clone()));
    next.run(Request::from_parts(parts, Body::from(bytes)))
        .await
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{Router, routing::post};
    use std::time::Instant;

    #[tokio::test]
    async fn retained_bytes_headers_and_deadline_preserve_opaque_duplicate_keys() {
        let deadline = Instant::now();
        let body = r#"{"params":{"arguments":{"payload":{"x":1,"x":2}}}}"#;
        let route = Router::new()
            .route(
                "/mcp",
                post(move |request: Request| async move {
                    let (parts, body_stream) = request.into_parts();
                    assert_eq!(mcp_request_body(&parts), Some(body.as_bytes()));
                    assert_eq!(parts.extensions.get::<Instant>(), Some(&deadline));
                    assert_eq!(parts.headers["x-probe"], "kept");
                    let bytes = axum::body::to_bytes(body_stream, 1024).await.unwrap();
                    assert_eq!(bytes, body.as_bytes());
                    StatusCode::OK
                }),
            )
            .layer(axum::middleware::from_fn(|request, next| {
                retain(request, next, 1024)
            }));
        let mut request = Request::builder()
            .method("POST")
            .uri("/mcp")
            .header(header::CONTENT_TYPE, "application/json; charset=utf-8")
            .header("x-probe", "kept")
            .body(Body::from(body))
            .unwrap();
        request.extensions_mut().insert(deadline);
        use tower::ServiceExt;
        assert_eq!(
            route.oneshot(request).await.unwrap().status(),
            StatusCode::OK
        );
    }
}
