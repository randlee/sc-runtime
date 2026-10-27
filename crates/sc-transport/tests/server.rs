use axum::routing::get;
use axum::{Json, Router};
use sc_transport::{Client, Endpoint, TransportError, bind};
use serde::{Deserialize, Serialize};

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

#[derive(Debug, Serialize, Deserialize)]
struct Health {
    ok: bool,
}

fn health_router() -> Router {
    Router::new().route("/health", get(|| async { Json(Health { ok: true }) }))
}

async fn assert_health(endpoint: &Endpoint) {
    let client = Client::new(endpoint.clone()).unwrap();
    let body: Health = client.get("/health").await.unwrap();
    assert!(body.ok);

    let raw = match endpoint {
        Endpoint::Tcp(_) => reqwest::Client::builder().build().unwrap(),
        #[cfg(unix)]
        Endpoint::Uds(path) => reqwest::Client::builder()
            .unix_socket(path.as_path())
            .build()
            .unwrap(),
        #[cfg(not(unix))]
        Endpoint::Uds(_) => panic!("uds"),
        _ => panic!("unsupported endpoint"),
    };
    let url = match endpoint {
        Endpoint::Tcp(addr) => format!("http://{addr}/health"),
        Endpoint::Uds(_) => "http://localhost/health".to_owned(),
        _ => panic!("unsupported endpoint"),
    };
    let response = raw.get(url).send().await.unwrap();
    assert_eq!(response.status(), 200);
    let text = response.text().await.unwrap();
    assert!(text.contains("true"), "{text}");
}

#[tokio::test]
#[cfg(unix)]
async fn uds_listener_serves_the_router() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("daemon.sock");
    let listener = bind(&Endpoint::Uds(path.clone())).await.unwrap();
    let mode = std::fs::metadata(&path).unwrap().permissions().mode();
    assert_eq!(mode & 0o077, 0);
    let endpoint = listener.local_endpoint().unwrap();
    let server = tokio::spawn(async move {
        axum::serve(listener, health_router()).await.unwrap();
    });
    assert_health(&endpoint).await;
    server.abort();
}

#[tokio::test]
async fn tcp_listener_reports_the_bound_port() {
    let listener = bind(&Endpoint::Tcp("127.0.0.1:0".parse().unwrap()))
        .await
        .unwrap();
    let endpoint = listener.local_endpoint().unwrap();
    let Endpoint::Tcp(addr) = &endpoint else {
        panic!("expected tcp");
    };
    assert_ne!(addr.port(), 0);
    let server = tokio::spawn(async move {
        axum::serve(listener, health_router()).await.unwrap();
    });
    assert_health(&endpoint).await;
    server.abort();
}

#[tokio::test]
#[cfg(unix)]
async fn one_router_serves_uds_and_tcp() {
    let dir = tempfile::tempdir().unwrap();
    let uds = bind(&Endpoint::Uds(dir.path().join("daemon.sock")))
        .await
        .unwrap();
    let tcp = bind(&Endpoint::Tcp("127.0.0.1:0".parse().unwrap()))
        .await
        .unwrap();
    let uds_endpoint = uds.local_endpoint().unwrap();
    let tcp_endpoint = tcp.local_endpoint().unwrap();
    let uds_server = tokio::spawn(async move {
        axum::serve(uds, health_router()).await.unwrap();
    });
    let tcp_server = tokio::spawn(async move {
        axum::serve(tcp, health_router()).await.unwrap();
    });
    assert_health(&uds_endpoint).await;
    assert_health(&tcp_endpoint).await;
    uds_server.abort();
    tcp_server.abort();
}

#[tokio::test]
async fn occupied_tcp_address_is_a_bind_error() {
    let held = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = held.local_addr().unwrap();
    let err = bind(&Endpoint::Tcp(addr)).await.expect_err("in use");
    assert!(matches!(err, TransportError::Bind { .. }), "{err:?}");
    assert_eq!(err.code(), "TRANSPORT.BIND");
    drop(held);
}

#[tokio::test]
#[cfg(unix)]
async fn missing_parent_directory_is_a_bind_error() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("missing").join("daemon.sock");
    let err = bind(&Endpoint::Uds(path)).await.expect_err("missing dir");
    assert!(matches!(err, TransportError::Bind { .. }), "{err:?}");
}

#[tokio::test]
#[cfg(unix)]
async fn stale_socket_is_replaced_and_serves() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("daemon.sock");
    let stale = std::os::unix::net::UnixListener::bind(&path).unwrap();
    drop(stale);
    assert!(path.exists());
    let listener = bind(&Endpoint::Uds(path.clone())).await.unwrap();
    let endpoint = listener.local_endpoint().unwrap();
    let server = tokio::spawn(async move {
        axum::serve(listener, health_router()).await.unwrap();
    });
    assert_health(&endpoint).await;
    server.abort();
}

#[tokio::test]
#[cfg(unix)]
async fn live_listener_keeps_its_socket_when_a_second_bind_fails() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("daemon.sock");
    let endpoint = Endpoint::Uds(path.clone());
    let listener = bind(&endpoint).await.unwrap();
    let bound = listener.local_endpoint().unwrap();
    let server = tokio::spawn(async move {
        axum::serve(listener, health_router()).await.unwrap();
    });
    let err = bind(&endpoint).await.expect_err("busy");
    assert!(
        matches!(err, TransportError::EndpointBusy { .. }),
        "{err:?}"
    );
    assert_eq!(err.code(), "TRANSPORT.ENDPOINT_BUSY");
    drop(err);
    assert!(path.exists());
    assert_health(&bound).await;
    server.abort();
}

#[tokio::test]
#[cfg(unix)]
async fn dropping_a_listener_removes_the_socket_and_keeps_the_lock_file() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("daemon.sock");
    let listener = bind(&Endpoint::Uds(path.clone())).await.unwrap();
    drop(listener);
    let mut lock_name = path.as_os_str().to_owned();
    lock_name.push(".lock");
    assert!(!path.exists());
    assert!(std::path::Path::new(&lock_name).exists());
}

#[tokio::test]
#[cfg(windows)]
async fn windows_uds_bind_reports_unsupported_endpoint() {
    let err = bind(&Endpoint::Uds(std::path::PathBuf::from("daemon.sock")))
        .await
        .expect_err("uds");
    assert!(
        matches!(err, TransportError::UnsupportedEndpoint { .. }),
        "{err:?}"
    );
    assert_eq!(err.code(), sc_transport::CODE_UNSUPPORTED_ENDPOINT);
    assert!(err.recovery().contains("tcp:"), "{}", err.recovery());
}

#[tokio::test]
#[cfg(unix)]
async fn regular_file_is_not_unlinked() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("daemon.sock");
    std::fs::write(&path, b"plain").unwrap();
    let err = bind(&Endpoint::Uds(path.clone())).await.expect_err("file");
    assert!(matches!(err, TransportError::NotASocket { .. }), "{err:?}");
    assert_eq!(err.code(), "TRANSPORT.NOT_A_SOCKET");
    assert_eq!(std::fs::read(&path).unwrap(), b"plain");
}
