//! REST and CLI proof. MCP coverage lives in `widget_mcp.rs` when that option is on.

use std::ffi::OsStr;

use cli::{APP_NAME, Command};
use sc_runtime::DaemonConfig;
use sc_runtime::testing::DaemonFixture;
use sc_transport::{AppName, TransportConfig, resolve_endpoint};
use serde_json::{Value, json};

#[tokio::test]
async fn create_and_get_match_across_rest_and_cli() {
    let fixture = start().await;
    let endpoint = fixture.endpoint_str();
    let app = AppName::new(APP_NAME).expect("app");
    let mut transport = TransportConfig::default();
    transport.instance_root = Some(fixture.instance_root().to_path_buf());
    let resolved =
        resolve_endpoint(&app, Some(OsStr::new(&endpoint)), None, &transport).expect("resolve");
    assert_eq!(resolved.to_string(), endpoint);

    let created = rest(&fixture, "/ops/widget.create", json!({ "name": "demo" })).await;
    assert_eq!(created["ok"], true);
    let id = created["data"]["id"].as_str().expect("id").to_owned();

    let rest_get = rest(&fixture, "/ops/widget.get", json!({ "id": id })).await;
    let cli_get = cli_json(
        &endpoint,
        &["widget".into(), "get".into(), "--id".into(), id],
    )
    .await;
    assert_eq!(rest_get, cli_get);
    assert_eq!(cli_get["data"]["name"], "demo");

    let missing = "missing-widget-id";
    let rest_missing = rest(&fixture, "/ops/widget.get", json!({ "id": missing })).await;
    let cli_missing = cli_json(
        &endpoint,
        &["widget".into(), "get".into(), "--id".into(), missing.into()],
    )
    .await;
    assert_eq!(rest_missing, cli_missing);
    assert_widget_error(&rest_missing, "WIDGET.NOT_FOUND", "not_found");

    let rest_blank = rest(&fixture, "/ops/widget.create", json!({ "name": "  " })).await;
    let cli_blank = cli_json(
        &endpoint,
        &[
            "widget".into(),
            "create".into(),
            "--name".into(),
            "  ".into(),
        ],
    )
    .await;
    assert_eq!(rest_blank, cli_blank);
    assert_widget_error(&rest_blank, "WIDGET.NAME_BLANK", "invalid_input");

    fixture.shutdown().await.expect("shutdown");
}

#[tokio::test]
async fn report_only_does_not_create_daemon_artifacts() {
    let dir = tempfile::tempdir().expect("temp");
    let socket = dir.path().join("missing.sock");
    let endpoint = format!("uds:{}", socket.display());
    let args = [
        "--json".into(),
        "--endpoint".into(),
        endpoint,
        "widget".into(),
        "get".into(),
        "--id".into(),
        "absent".into(),
    ];
    let command = cli::parse(&args).expect("parse");
    assert!(!matches!(command, Command::DaemonStart));
    let (code, text) = cli::execute(command).await;
    assert_eq!(code, std::process::ExitCode::from(1));
    let body: Value = serde_json::from_str(&text).expect("json");
    assert_eq!(body["ok"], false);
    assert!(body["data"].is_null());
    assert_eq!(body["error"]["code"], "DAEMON.NOT_RUNNING");
    assert_eq!(
        body["error"]["suggested_action"],
        format!("run {APP_NAME} daemon start")
    );
    assert!(!socket.exists());
    assert!(!dir.path().join("daemon.lock").exists());
    assert!(!dir.path().join("daemon.sock").exists());
    assert!(matches!(
        cli::parse(&["daemon".into(), "start".into()]).expect("start"),
        Command::DaemonStart
    ));
    let binary = cli::daemon_binary().expect("binary");
    assert!(
        binary
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name == format!("{APP_NAME}-daemon"))
    );
}

#[tokio::test]
async fn cli_keeps_unknown_envelope_content() {
    let listener = tokio::net::TcpListener::bind((std::net::Ipv4Addr::LOCALHOST, 0))
        .await
        .expect("bind");
    let address = listener.local_addr().expect("addr");
    let body = json!({
        "version": "9",
        "ok": true,
        "data": {"id": "kept", "name": "demo", "extra": {"kept": true}},
        "error": null
    });
    let encoded = serde_json::to_vec(&body).expect("encode");
    let server = tokio::spawn(async move {
        let (mut stream, _) = listener.accept().await.expect("accept");
        let mut buf = [0_u8; 2048];
        let _ = tokio::io::AsyncReadExt::read(&mut stream, &mut buf).await;
        let header = format!(
            "HTTP/1.1 200 OK\r\ncontent-type: application/json\r\ncontent-length: {}\r\nconnection: close\r\n\r\n",
            encoded.len()
        );
        tokio::io::AsyncWriteExt::write_all(&mut stream, header.as_bytes())
            .await
            .expect("head");
        tokio::io::AsyncWriteExt::write_all(&mut stream, &encoded)
            .await
            .expect("body");
    });
    let endpoint = format!("tcp:{address}");
    let printed = cli_json(
        &endpoint,
        &[
            "widget".into(),
            "create".into(),
            "--name".into(),
            "demo".into(),
        ],
    )
    .await;
    assert_eq!(printed, body);
    server.abort();
}

async fn start() -> DaemonFixture {
    let app = AppName::new(APP_NAME).expect("app");
    DaemonFixture::start(
        &app,
        DaemonConfig::default(),
        open_temp,
        daemon::routes::router,
    )
    .await
    .expect("start")
}

async fn open_temp() -> Result<service::Stores, sc_runtime::RuntimeError> {
    let dir = tempfile::tempdir().expect("db dir");
    let stores = service::open_stores(dir.path().join("widgets.db")).await?;
    std::mem::forget(dir);
    Ok(stores)
}

async fn rest(fixture: &DaemonFixture, path: &str, body: Value) -> Value {
    fixture.client().post(path, &body).await.expect("rest")
}

async fn cli_json(endpoint: &str, tail: &[String]) -> Value {
    let mut args = vec!["--json".into(), "--endpoint".into(), endpoint.to_owned()];
    args.extend(tail.iter().cloned());
    let command = cli::parse(&args).expect("parse");
    let (_code, text) = cli::execute(command).await;
    serde_json::from_str(&text).expect("cli json")
}

fn assert_widget_error(body: &Value, code: &str, kind: &str) {
    assert_eq!(body["ok"], false);
    assert!(body["data"].is_null());
    assert_eq!(body["error"]["code"], code);
    assert_eq!(body["error"]["kind"], kind);
    assert!(body["error"]["details"]["cause"].is_string());
    assert!(body["error"]["suggested_action"].is_string());
}
