//! HTTP client over a resolved endpoint.

use std::time::Duration;

use serde::Serialize;
use serde::de::DeserializeOwned;

use crate::endpoint::Endpoint;
use crate::error::TransportError;

/// Connect timeout applied by [`Client::new`]. Not caller-configurable in v0.1.
const CONNECT_TIMEOUT: Duration = Duration::from_secs(2);

/// Whole-request timeout applied by [`Client::new`]. Not caller-configurable in v0.1.
const REQUEST_TIMEOUT: Duration = Duration::from_secs(2);

/// JSON HTTP client for one resolved [`Endpoint`].
///
/// `new` does not contact the daemon. `get` and `post` report an unreachable
/// daemon as [`TransportError::DaemonNotRunning`] and do not spawn a process
/// or retry. Response bodies are deserialized as the caller-chosen `T` for
/// every HTTP status. A body that is not `T` is [`TransportError::ResponseDecode`]
/// and keeps a bounded preview of that body.
///
/// # Examples
///
/// ```no_run
/// # #[tokio::main]
/// # async fn main() -> Result<(), sc_transport::TransportError> {
/// let endpoint = "tcp:127.0.0.1:9".parse()?;
/// let client = sc_transport::Client::new(endpoint)?;
/// #[derive(serde::Deserialize)]
/// struct Body {
///     ok: bool,
/// }
/// let body: Body = client.get("/health").await?;
/// # let _ = body;
/// # Ok(())
/// # }
/// ```
#[derive(Clone)]
pub struct Client {
    endpoint: Endpoint,
    http: reqwest::Client,
    base: String,
}

impl Client {
    /// Builds a reqwest client for `endpoint`.
    ///
    /// A UDS endpoint uses `reqwest::ClientBuilder::unix_socket`. A TCP
    /// endpoint uses `http://<socket-addr>`. The call does not connect.
    /// Connect and request timeouts are fixed at two seconds in v0.1.
    /// A future `ClientOptions` is the extension point for timeouts and
    /// default headers; this constructor stays and uses the defaults.
    ///
    /// # Errors
    ///
    /// Returns [`TransportError::Request`] when the reqwest client cannot be
    /// built, or on a non-Unix target when `endpoint` is a UDS path.
    pub fn new(endpoint: Endpoint) -> Result<Self, TransportError> {
        match &endpoint {
            Endpoint::Tcp(addr) => build(endpoint.clone(), None, format!("http://{addr}")),
            Endpoint::Uds(path) => {
                #[cfg(unix)]
                {
                    build(
                        endpoint.clone(),
                        Some(path.clone()),
                        "http://localhost".to_owned(),
                    )
                }
                #[cfg(not(unix))]
                {
                    let _ = path;
                    Err(TransportError::request(
                        endpoint,
                        "UDS endpoints are not supported on this platform",
                    ))
                }
            }
        }
    }

    /// Sends `GET path` and deserializes the JSON body as `T`.
    ///
    /// # Errors
    ///
    /// Returns [`TransportError::DaemonNotRunning`] when the UDS path is
    /// missing or the connection is refused, [`TransportError::TimedOut`] when
    /// the request times out, [`TransportError::ResponseDecode`] when the body
    /// bytes are not `T` (with a bounded `body_preview`), and
    /// [`TransportError::Request`] for any other client failure, including a
    /// failure to read the body after headers arrived.
    pub async fn get<T: DeserializeOwned>(&self, path: &str) -> Result<T, TransportError> {
        let response = self
            .http
            .get(request_url(&self.base, path))
            .send()
            .await
            .map_err(|err| classify(&self.endpoint, err))?;
        read_json(&self.endpoint, response).await
    }

    /// Sends `POST path` with `body` as JSON and deserializes the response as `T`.
    ///
    /// # Errors
    ///
    /// See [`Client::get`]. A serialization failure of `body` is
    /// [`TransportError::Request`].
    pub async fn post<T: DeserializeOwned, B: Serialize>(
        &self,
        path: &str,
        body: &B,
    ) -> Result<T, TransportError> {
        let response = self
            .http
            .post(request_url(&self.base, path))
            .json(body)
            .send()
            .await
            .map_err(|err| classify(&self.endpoint, err))?;
        read_json(&self.endpoint, response).await
    }
}

impl std::fmt::Debug for Client {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Client")
            .field("endpoint", &self.endpoint)
            .finish()
    }
}

fn build(
    endpoint: Endpoint,
    unix: Option<std::path::PathBuf>,
    base: String,
) -> Result<Client, TransportError> {
    let builder = reqwest::Client::builder()
        .connect_timeout(CONNECT_TIMEOUT)
        .timeout(REQUEST_TIMEOUT);
    #[cfg(unix)]
    let builder = if let Some(path) = unix {
        builder.unix_socket(path)
    } else {
        builder
    };
    #[cfg(not(unix))]
    let builder = {
        let _ = unix;
        builder
    };
    let http = builder
        .build()
        .map_err(|err| TransportError::request(endpoint.clone(), err.to_string()))?;
    Ok(Client {
        endpoint,
        http,
        base,
    })
}

fn request_url(base: &str, path: &str) -> String {
    let base = base.trim_end_matches('/');
    let path = path.trim_start_matches('/');
    format!("{base}/{path}")
}

const BODY_PREVIEW_LIMIT: usize = 512;

async fn read_json<T: DeserializeOwned>(
    endpoint: &Endpoint,
    response: reqwest::Response,
) -> Result<T, TransportError> {
    let status = response.status().as_u16();
    let bytes = match response.bytes().await {
        Ok(bytes) => bytes,
        // A failure while reading the body is a transfer failure, classified
        // the same way as `send`: timeout, down daemon, or request.
        Err(err) => return Err(classify(endpoint, err)),
    };
    match serde_json::from_slice::<T>(&bytes) {
        Ok(value) => Ok(value),
        Err(err) => Err(TransportError::response_decode(
            status,
            err.to_string(),
            body_preview(&bytes),
        )),
    }
}

fn body_preview(bytes: &[u8]) -> String {
    let truncated = bytes.len() > BODY_PREVIEW_LIMIT;
    let shown = &bytes[..bytes.len().min(BODY_PREVIEW_LIMIT)];
    let mut preview = String::from_utf8_lossy(shown).into_owned();
    if truncated {
        preview.push_str("...");
    }
    preview
}

fn classify(endpoint: &Endpoint, err: reqwest::Error) -> TransportError {
    if err.is_timeout() {
        return TransportError::timed_out(endpoint.clone(), err.to_string());
    }
    if is_daemon_down(&err) {
        return TransportError::daemon_not_running(endpoint.clone(), err.to_string());
    }
    TransportError::request(endpoint.clone(), err.to_string())
}

fn is_daemon_down(err: &reqwest::Error) -> bool {
    match io_kind(err) {
        Some(std::io::ErrorKind::NotFound) | Some(std::io::ErrorKind::ConnectionRefused) => true,
        Some(_) => false,
        None if err.is_connect() => {
            let text = err.to_string().to_ascii_lowercase();
            text.contains("connection refused")
                || text.contains("no such file")
                || text.contains("not found")
        }
        None => false,
    }
}

fn io_kind(err: &reqwest::Error) -> Option<std::io::ErrorKind> {
    let mut current = Some(err as &(dyn std::error::Error + 'static));
    while let Some(item) = current {
        if let Some(io) = item.downcast_ref::<std::io::Error>() {
            return Some(io.kind());
        }
        current = item.source();
    }
    None
}

#[cfg(test)]
mod tests {
    use super::body_preview;

    #[test]
    fn preview_keeps_a_bounded_lossy_prefix() {
        let long = vec![b'a'; 600];
        let preview = body_preview(&long);
        assert_eq!(preview, format!("{}...", "a".repeat(512)));
        assert_eq!(body_preview(&[0xff]), "\u{FFFD}");
    }
}
