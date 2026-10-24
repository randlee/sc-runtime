mod common;

use std::net::TcpListener as StdTcpListener;

use sc_transport::{Client, Endpoint, TransportError};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct Named {
    name: String,
}

fn assert_down(err: &TransportError, endpoint: &Endpoint) {
    match err {
        TransportError::DaemonNotRunning {
            code,
            endpoint: got,
            cause,
            ..
        } => {
            assert_eq!(*code, "DAEMON.NOT_RUNNING");
            assert_eq!(got, endpoint);
            assert!(!cause.is_empty());
        }
        other => panic!("expected DaemonNotRunning, got {other:?}"),
    }
    let rendered = format!("{err:?}");
    assert!(!rendered.contains("suggested_action"));
    assert!(!rendered.contains("my-app"));
    assert!(!rendered.contains("other-app"));
}

#[cfg(unix)]
async fn uds_endpoint(name: &str) -> (tempfile::TempDir, Endpoint, tokio::net::UnixListener) {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join(name);
    let listener = tokio::net::UnixListener::bind(&path).unwrap();
    (dir, Endpoint::Uds(path), listener)
}

#[tokio::test]
#[cfg(unix)]
async fn post_and_get_over_uds() {
    let (_dir, endpoint, listener) = uds_endpoint("daemon.sock").await;
    let server = common::spawn_unix(listener);
    let client = Client::new(endpoint).unwrap();
    let echoed: Named = client
        .post(
            "/echo",
            &Named {
                name: "widget".to_owned(),
            },
        )
        .await
        .unwrap();
    assert_eq!(echoed.name, "widget");
    let fixed: Named = client.get("/doc").await.unwrap();
    assert_eq!(fixed.name, "fixed");
    server.abort();
}

#[tokio::test]
async fn post_and_get_over_tcp() {
    let listener = tokio::net::TcpListener::bind((std::net::Ipv4Addr::LOCALHOST, 0))
        .await
        .unwrap();
    let addr = listener.local_addr().unwrap();
    let server = common::spawn_tcp(listener);
    let client = Client::new(Endpoint::Tcp(addr)).unwrap();
    let echoed: Named = client
        .post(
            "/echo",
            &Named {
                name: "widget".to_owned(),
            },
        )
        .await
        .unwrap();
    assert_eq!(echoed.name, "widget");
    let fixed: Named = client.get("/doc").await.unwrap();
    assert_eq!(fixed.name, "fixed");
    server.abort();
}

#[tokio::test]
async fn undecodable_body_is_not_daemon_not_running() {
    let listener = tokio::net::TcpListener::bind((std::net::Ipv4Addr::LOCALHOST, 0))
        .await
        .unwrap();
    let addr = listener.local_addr().unwrap();
    let server = common::spawn_tcp(listener);
    let client = Client::new(Endpoint::Tcp(addr)).unwrap();
    let err = client.get::<Named>("/bad").await.expect_err("not json");
    match err {
        TransportError::ResponseDecode {
            status,
            ref body_preview,
            ..
        } => {
            assert_eq!(status, 200);
            assert!(body_preview.contains("not json"), "{body_preview}");
        }
        other => panic!("expected ResponseDecode, got {other:?}"),
    }
    assert!(!matches!(err, TransportError::DaemonNotRunning { .. }));
    assert_eq!(err.code(), "TRANSPORT.RESPONSE_DECODE");
    server.abort();
}

#[tokio::test]
async fn non_success_status_still_deserializes() {
    let listener = tokio::net::TcpListener::bind((std::net::Ipv4Addr::LOCALHOST, 0))
        .await
        .unwrap();
    let addr = listener.local_addr().unwrap();
    let server = common::spawn_tcp(listener);
    let client = Client::new(Endpoint::Tcp(addr)).unwrap();
    let body: Named = client.get("/missing").await.unwrap();
    assert_eq!(body.name, "missing");
    server.abort();
}

#[tokio::test]
async fn hung_route_times_out() {
    let listener = tokio::net::TcpListener::bind((std::net::Ipv4Addr::LOCALHOST, 0))
        .await
        .unwrap();
    let addr = listener.local_addr().unwrap();
    let server = common::spawn_tcp(listener);
    let client = Client::new(Endpoint::Tcp(addr)).unwrap();
    let err = client.get::<Named>("/hang").await.expect_err("hang");
    assert!(matches!(err, TransportError::TimedOut { .. }), "{err:?}");
    assert!(!matches!(err, TransportError::DaemonNotRunning { .. }));
    assert!(!matches!(err, TransportError::ResponseDecode { .. }));
    assert_eq!(err.code(), "TRANSPORT.TIMED_OUT");
    server.abort();
}

#[tokio::test]
#[cfg(unix)]
async fn missing_socket_is_daemon_not_running() {
    let dir = tempfile::tempdir().unwrap();
    let endpoint = Endpoint::Uds(dir.path().join("missing.sock"));
    let client = Client::new(endpoint.clone()).unwrap();
    let err = client.get::<Named>("/doc").await.expect_err("missing");
    assert_down(&err, &endpoint);
    let again = Client::new(endpoint.clone()).unwrap();
    let second = again.get::<Named>("/doc").await.expect_err("missing again");
    assert_down(&second, &endpoint);
}

#[tokio::test]
#[cfg(unix)]
async fn stale_socket_is_daemon_not_running() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("stale.sock");
    let listener = std::os::unix::net::UnixListener::bind(&path).unwrap();
    drop(listener);
    let endpoint = Endpoint::Uds(path);
    let client = Client::new(endpoint.clone()).unwrap();
    let err = client.get::<Named>("/doc").await.expect_err("stale");
    assert_down(&err, &endpoint);
}

#[tokio::test]
async fn refused_tcp_is_daemon_not_running() {
    let listener = StdTcpListener::bind((std::net::Ipv4Addr::LOCALHOST, 0)).unwrap();
    let addr = listener.local_addr().unwrap();
    drop(listener);
    let endpoint = Endpoint::Tcp(addr);
    let client = Client::new(endpoint.clone()).unwrap();
    let err = client.get::<Named>("/doc").await.expect_err("refused");
    assert_down(&err, &endpoint);
}

#[tokio::test]
#[cfg(unix)]
async fn directory_path_is_a_request_error() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("not-socket");
    std::fs::create_dir(&path).unwrap();
    let endpoint = Endpoint::Uds(path);
    let client = Client::new(endpoint).unwrap();
    let err = client.get::<Named>("/doc").await.expect_err("directory");
    assert!(matches!(err, TransportError::Request { .. }), "{err:?}");
    assert_eq!(err.code(), "TRANSPORT.REQUEST");
}
