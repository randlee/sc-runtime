//! REST, MCP, and CLI against one fixture. This file is omitted when mcp is off.

use std::time::Duration;

use cli::APP_NAME;
use sc_runtime::DaemonConfig;
use sc_runtime::testing::DaemonFixture;
use sc_transport::AppName;
use serde_json::{Value, json};

#[tokio::test]
async fn create_and_get_match_across_rest_mcp_and_cli() {
    tokio::time::timeout(Duration::from_secs(30), exercise())
        .await
        .expect("mcp fixture deadline");
}

async fn exercise() {
    let app = AppName::new(APP_NAME).expect("app");
    let fixture = DaemonFixture::start_with_mcp(
        &app,
        fixture_config(),
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

    let names = tool_names(&fixture).await;
    #[cfg(feature = "general-db")]
    {
        for required in ["widget_create", "widget_get", "widget_delete", "delete_row"] {
            assert!(
                names.iter().any(|name| name == required),
                "missing {required} in {names:?}"
            );
        }
        let missing_id = "missing-widget-id";
        let rest_delete = rest(
            &fixture,
            api_types::WIDGET_DELETE_PATH,
            json!({ "id": missing_id }),
        )
        .await;
        let rest_row = rest(
            &fixture,
            api_types::DELETE_ROW_PATH,
            json!({ "table": "widgets", "id": missing_id }),
        )
        .await;
        let mcp_delete = mcp_call(&fixture, "widget_delete", json!({ "id": missing_id })).await;
        let mcp_row = mcp_call(
            &fixture,
            "delete_row",
            json!({ "table": "widgets", "id": missing_id }),
        )
        .await;
        let cli_delete = cli_json(
            &endpoint,
            &[
                "widget".into(),
                "delete".into(),
                "--id".into(),
                missing_id.into(),
            ],
        )
        .await;
        let cli_row = cli_json(
            &endpoint,
            &[
                "delete".into(),
                "row".into(),
                "widgets".into(),
                missing_id.into(),
            ],
        )
        .await;
        assert_eq!(rest_delete["error"]["code"], "ROW.NOT_FOUND");
        assert_eq!(rest_delete, mcp_delete);
        assert_eq!(rest_delete, cli_delete);
        assert_eq!(rest_row, mcp_row);
        assert_eq!(rest_row, cli_row);
        assert_eq!(rest_delete["error"]["code"], rest_row["error"]["code"]);
        assert_eq!(rest_delete["error"]["kind"], rest_row["error"]["kind"]);
        assert_eq!(
            rest_delete["error"]["details"],
            rest_row["error"]["details"]
        );
    }
    #[cfg(not(feature = "general-db"))]
    {
        let mut listed = names;
        listed.sort();
        assert_eq!(
            listed,
            vec!["widget_create".to_owned(), "widget_get".to_owned()]
        );
        assert!(
            cli::parse(&[
                "delete".into(),
                "row".into(),
                "widgets".into(),
                "missing-widget-id".into(),
            ])
            .is_err(),
            "delete row is registered"
        );
        assert!(
            cli::parse(&[
                "widget".into(),
                "delete".into(),
                "--id".into(),
                "missing-widget-id".into(),
            ])
            .is_err(),
            "widget delete is registered"
        );
        let router = daemon::routes::openapi_router();
        let spec = router.get_openapi();
        let paths: Vec<&str> = spec.paths.paths.keys().map(String::as_str).collect();
        assert!(!paths.contains(&api_types::WIDGET_DELETE_PATH), "{paths:?}");
        assert!(!paths.contains(&api_types::DELETE_ROW_PATH), "{paths:?}");
    }

    fixture.shutdown().await.expect("shutdown");
}

fn fixture_config() -> DaemonConfig {
    #[cfg(unix)]
    {
        DaemonConfig::default()
    }
    #[cfg(not(unix))]
    {
        let mut config = DaemonConfig::default();
        // Port 0 binds an ephemeral port. The fixture reads the bound address
        // from the listener after `sc-transport` accepts the socket.
        config.endpoint_flag = Some("tcp:127.0.0.1:0".to_owned());
        config
    }
}

include!("open/{{db}}.rs");

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

async fn tool_names(fixture: &DaemonFixture) -> Vec<String> {
    let body = mcp_rpc(
        fixture,
        json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "tools/list",
            "params": {}
        }),
    )
    .await;
    body["result"]["tools"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|tool| tool.get("name").and_then(Value::as_str))
        .map(str::to_owned)
        .collect()
}

async fn mcp_call(fixture: &DaemonFixture, name: &str, arguments: Value) -> Value {
    let body = mcp_rpc(
        fixture,
        json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "tools/call",
            "params": { "name": name, "arguments": arguments }
        }),
    )
    .await;
    find_envelope(&body).cloned().expect("envelope")
}

async fn mcp_rpc(fixture: &DaemonFixture, payload: Value) -> Value {
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
    let http = builder
        .connect_timeout(Duration::from_secs(2))
        .timeout(Duration::from_secs(2))
        .build()
        .expect("client");
    let response = http
        .post(url)
        .header("accept", "application/json, text/event-stream")
        .header("content-type", "application/json")
        .json(&payload)
        .send()
        .await
        .expect("mcp");
    assert!(response.status().is_success(), "{}", response.status());
    response.json().await.expect("mcp json")
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
