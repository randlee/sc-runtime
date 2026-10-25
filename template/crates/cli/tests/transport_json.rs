//! One host-safe `--json` vector for every `TransportError` variant.

use std::process::Command;

use cli::APP_NAME;
use sc_command::Envelope;
use sc_transport::{AppName, Client, Endpoint, TransportError};
use serde::Serialize;
use serde_json::Value;

#[test]
fn invalid_name_vector() {
    let error = AppName::new("my app").expect_err("name");
    assert_mapped(&error);
    assert_eq!(error.code(), "APP.INVALID_NAME");
}

#[test]
fn invalid_endpoint_vector() {
    let error = "not-an-endpoint".parse::<Endpoint>().expect_err("endpoint");
    assert_mapped(&error);
    assert_eq!(error.code(), "TRANSPORT.INVALID_ENDPOINT");
}

#[test]
#[cfg(windows)]
fn missing_tcp_port_vector() {
    let app = AppName::new(APP_NAME).expect("app");
    let error =
        sc_transport::resolve_endpoint(&app, None, None, &sc_transport::TransportConfig::default())
            .expect_err("port");
    assert_mapped(&error);
    assert_eq!(error.code(), "TRANSPORT.MISSING_TCP_PORT");
}

#[test]
fn instance_root_unavailable_vector() {
    if std::env::var_os("WIDGET_VECTOR_CHILD").is_some() {
        let app = AppName::new(APP_NAME).expect("app");
        let error = sc_transport::instance_root(&app).expect_err("base");
        assert_mapped(&error);
        assert_eq!(error.code(), "TRANSPORT.INSTANCE_ROOT_UNAVAILABLE");
        return;
    }
    let status = Command::new(std::env::current_exe().expect("exe"))
        .args(["instance_root_unavailable_vector", "--exact"])
        .env("WIDGET_VECTOR_CHILD", "1")
        .env_remove("HOME")
        .env_remove("XDG_RUNTIME_DIR")
        .env_remove("LOCALAPPDATA")
        .env_remove("USERPROFILE")
        .status()
        .expect("child");
    assert!(status.success());
}

#[cfg(unix)]
#[tokio::test]
async fn daemon_not_running_vector() {
    let dir = tempfile::tempdir().expect("temp");
    let endpoint = Endpoint::Uds(dir.path().join("missing.sock"));
    assert_not_running(endpoint).await;
}

#[cfg(not(unix))]
#[tokio::test]
async fn daemon_not_running_vector() {
    let endpoint = "tcp:127.0.0.1:1".parse().expect("port 1");
    assert_not_running(endpoint).await;
}

async fn assert_not_running(endpoint: Endpoint) {
    let client = Client::new(endpoint).expect("client");
    let error = client.get::<Value>("/health").await.expect_err("down");
    assert_mapped(&error);
    assert_eq!(error.code(), "DAEMON.NOT_RUNNING");
}

#[tokio::test]
async fn response_decode_vector() {
    let listener = tokio::net::TcpListener::bind((std::net::Ipv4Addr::LOCALHOST, 0))
        .await
        .expect("bind");
    let address = listener.local_addr().expect("addr");
    let server = tokio::spawn(async move {
        let (mut stream, _) = listener.accept().await.expect("accept");
        let mut buf = [0_u8; 1024];
        let _ = tokio::io::AsyncReadExt::read(&mut stream, &mut buf).await;
        let body = b"not-json";
        let header = format!(
            "HTTP/1.1 200 OK\r\ncontent-length: {}\r\nconnection: close\r\n\r\n",
            body.len()
        );
        tokio::io::AsyncWriteExt::write_all(&mut stream, header.as_bytes())
            .await
            .expect("head");
        tokio::io::AsyncWriteExt::write_all(&mut stream, body)
            .await
            .expect("body");
    });
    let endpoint = Endpoint::Tcp(address);
    let client = Client::new(endpoint).expect("client");
    let error = client.get::<Value>("/health").await.expect_err("decode");
    assert_mapped(&error);
    assert_eq!(error.code(), "TRANSPORT.RESPONSE_DECODE");
    server.abort();
}

#[tokio::test]
async fn timed_out_vector() {
    let listener = tokio::net::TcpListener::bind((std::net::Ipv4Addr::LOCALHOST, 0))
        .await
        .expect("bind");
    let address = listener.local_addr().expect("addr");
    let server = tokio::spawn(async move {
        let (mut stream, _) = listener.accept().await.expect("accept");
        let mut buf = [0_u8; 1024];
        let _ = tokio::io::AsyncReadExt::read(&mut stream, &mut buf).await;
        std::future::pending::<()>().await;
        drop(stream);
    });
    let endpoint = Endpoint::Tcp(address);
    let client = Client::new(endpoint).expect("client");
    let error = client.get::<Value>("/health").await.expect_err("timeout");
    assert_mapped(&error);
    assert_eq!(error.code(), "TRANSPORT.TIMED_OUT");
    server.abort();
}

#[tokio::test]
async fn request_vector() {
    let endpoint = "tcp:127.0.0.1:9".parse::<Endpoint>().expect("endpoint");
    let client = Client::new(endpoint).expect("client");
    let error = client
        .post::<Value, _>("/health", &Boom)
        .await
        .expect_err("request");
    assert_mapped(&error);
    assert_eq!(error.code(), "TRANSPORT.REQUEST");
}

#[tokio::test]
async fn bind_vector() {
    let dir = tempfile::tempdir().expect("temp");
    let path = dir.path().join("missing-parent").join("app.sock");
    let endpoint = Endpoint::Uds(path);
    let error = sc_transport::bind(&endpoint).await.expect_err("bind");
    assert_mapped(&error);
    assert_eq!(error.code(), "TRANSPORT.BIND");
}

#[tokio::test]
#[cfg(unix)]
async fn endpoint_busy_vector() {
    let dir = tempfile::tempdir().expect("temp");
    let endpoint = Endpoint::Uds(dir.path().join("app.sock"));
    let held = sc_transport::bind(&endpoint).await.expect("first");
    let error = sc_transport::bind(&endpoint).await.expect_err("busy");
    assert_mapped(&error);
    assert_eq!(error.code(), "TRANSPORT.ENDPOINT_BUSY");
    drop(held);
}

#[tokio::test]
#[cfg(unix)]
async fn not_a_socket_vector() {
    let dir = tempfile::tempdir().expect("temp");
    let path = dir.path().join("not.sock");
    std::fs::write(&path, b"regular").expect("file");
    let endpoint = Endpoint::Uds(path);
    let error = sc_transport::bind(&endpoint).await.expect_err("file");
    assert_mapped(&error);
    assert_eq!(error.code(), "TRANSPORT.NOT_A_SOCKET");
}

fn assert_mapped(error: &TransportError) {
    let operation = cli::map_transport(error);
    let envelope: Envelope<Value> = Envelope::failure(operation);
    let value = serde_json::to_value(&envelope).expect("envelope");
    assert_eq!(value["ok"], false);
    assert!(value["data"].is_null());
    assert_eq!(value["version"], "1");
    let body = &value["error"];
    assert_eq!(body["code"], error.code());
    assert!(body["message"].is_string());
    assert!(body.get("details").is_some());
    assert!(body.get("suggested_action").is_some());
    if error.code() == "DAEMON.NOT_RUNNING" {
        assert_eq!(body["kind"], "unavailable");
        assert_eq!(body["message"], "the daemon is not running");
        assert_eq!(
            body["suggested_action"],
            format!("run {APP_NAME} daemon start")
        );
    } else {
        assert_eq!(body["kind"], "internal");
        assert!(body["suggested_action"].is_null());
        let message = body["message"].as_str().expect("message");
        assert!(
            message.starts_with(&format!("transport failure {} at ", error.code())),
            "{message}"
        );
    }
}

struct Boom;

impl Serialize for Boom {
    fn serialize<S>(&self, _serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        Err(serde::ser::Error::custom("boom"))
    }
}
