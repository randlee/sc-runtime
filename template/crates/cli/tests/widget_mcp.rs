//! REST, MCP, and CLI against one fixture. This file is omitted when mcp is off.

use cli::APP_NAME;
use sc_runtime::DaemonConfig;
use sc_runtime::testing::DaemonFixture;
use sc_transport::AppName;
use serde_json::{Value, json};

#[tokio::test]
async fn create_and_get_match_across_rest_mcp_and_cli() {
    let app = AppName::new(APP_NAME).expect("app");
    let fixture = DaemonFixture::start_with_mcp(
        &app,
        DaemonConfig::default(),
        open_temp,
        daemon::routes::router,
        daemon::mcp::service,
    )
    .await
    .expect("start");
    let endpoint = fixture.endpoint_str();

    let created = rest(&fixture, "/ops/widget.create", json!({ "name": "demo" })).await;
    assert_eq!(created["ok"], true);
    let id = created["data"]["id"].as_str().expect("id").to_owned();
    let mcp_created = mcp_call(&fixture, "widget_create", json!({ "name": "other" })).await;
    assert_eq!(mcp_created["ok"], true);
    let cli_created = cli_json(
        &endpoint,
        &[
            "widget".into(),
            "create".into(),
            "--name".into(),
            "third".into(),
        ],
    )
    .await;
    assert_eq!(cli_created["ok"], true);

    let rest_get = rest(&fixture, "/ops/widget.get", json!({ "id": id })).await;
    let mcp_get = mcp_call(&fixture, "widget_get", json!({ "id": id })).await;
    let cli_get = cli_json(
        &endpoint,
        &["widget".into(), "get".into(), "--id".into(), id],
    )
    .await;
    assert_eq!(rest_get, mcp_get);
    assert_eq!(rest_get, cli_get);

    let missing = json!({ "id": "missing-widget-id" });
    let rest_missing = rest(&fixture, "/ops/widget.get", missing.clone()).await;
    let mcp_missing = mcp_call(&fixture, "widget_get", missing.clone()).await;
    let cli_missing = cli_json(
        &endpoint,
        &[
            "widget".into(),
            "get".into(),
            "--id".into(),
            "missing-widget-id".into(),
        ],
    )
    .await;
    assert_eq!(rest_missing, mcp_missing);
    assert_eq!(rest_missing, cli_missing);
    assert_eq!(rest_missing["error"]["code"], "WIDGET.NOT_FOUND");
    assert_eq!(rest_missing["error"]["kind"], "not_found");
    assert_eq!(
        rest_missing["error"]["details"]["cause"],
        "the id is not in the store"
    );
    assert_eq!(
        rest_missing["error"]["suggested_action"],
        "create the widget, then get the returned id"
    );

    let blank = json!({ "name": " " });
    let rest_blank = rest(&fixture, "/ops/widget.create", blank.clone()).await;
    let mcp_blank = mcp_call(&fixture, "widget_create", blank).await;
    let cli_blank = cli_json(
        &endpoint,
        &[
            "widget".into(),
            "create".into(),
            "--name".into(),
            " ".into(),
        ],
    )
    .await;
    assert_eq!(rest_blank, mcp_blank);
    assert_eq!(rest_blank, cli_blank);
    assert_eq!(rest_blank["error"]["code"], "WIDGET.NAME_BLANK");
    assert_eq!(rest_blank["error"]["kind"], "invalid_input");
    assert_eq!(
        rest_blank["error"]["details"]["cause"],
        "name is empty or only whitespace"
    );
    assert_eq!(
        rest_blank["error"]["suggested_action"],
        "send a non-empty name"
    );

    fixture.shutdown().await.expect("shutdown");
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

async fn mcp_call(fixture: &DaemonFixture, name: &str, arguments: Value) -> Value {
    let endpoint = fixture.endpoint();
    let mut builder = reqwest::Client::builder();
    let url = match endpoint {
        sc_transport::Endpoint::Tcp(addr) => format!("http://{addr}/mcp"),
        sc_transport::Endpoint::Uds(path) => {
            #[cfg(unix)]
            {
                builder = builder.unix_socket(path.clone());
            }
            #[cfg(not(unix))]
            let _ = path;
            "http://localhost/mcp".to_owned()
        }
        _ => "http://localhost/mcp".to_owned(),
    };
    let http = builder.build().expect("client");
    let response = http
        .post(url)
        .header("accept", "application/json, text/event-stream")
        .header("content-type", "application/json")
        .json(&json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "tools/call",
            "params": { "name": name, "arguments": arguments }
        }))
        .send()
        .await
        .expect("mcp");
    assert!(response.status().is_success(), "{}", response.status());
    let body: Value = response.json().await.expect("mcp json");
    find_envelope(&body).cloned().expect("envelope")
}

fn find_envelope(value: &Value) -> Option<&Value> {
    let looks_like_envelope = value.get("version").is_some()
        && value.get("ok").is_some()
        && value.get("data").is_some()
        && value.get("error").is_some();
    if looks_like_envelope {
        return Some(value);
    }
    match value {
        Value::Object(map) => map.values().find_map(find_envelope),
        Value::Array(items) => items.iter().find_map(find_envelope),
        _ => None,
    }
}
