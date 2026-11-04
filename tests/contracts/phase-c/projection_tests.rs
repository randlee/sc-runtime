//! C3-only generic projection proof; installed only into a generated test venue.
#[path = "../../../tests/support/executor_double.rs"]
mod executor;
use executor::RecordingExecutor;
use sc_command::{Envelope, ErrorKind, OpError};
use sc_cron_commands::{
    CronExecutor, CronOperation, CronRequest, CronResponse, EndResult, ListResult, ScheduleState,
    ScheduleView, TargetKind,
};
use serde_json::{Value, json};
use std::{
    sync::Arc,
    time::{Duration, Instant},
};

fn requests() -> Vec<CronRequest> {
    let mut values = vec![
        json!({"version":"1","command":"cron.end","request":{"name":"nightly","generation":3,"reason":"operator stopped"}}),
        json!({"version":"1","command":"cron.list","request":{"state":"all","limit":100,"cursor":null}}),
    ];
    for target in [
        json!({"kind":"rust"}),
        json!({"kind":"python","module":"/configured/nightly.py"}),
        json!({"kind":"monty","source":"result=1"}),
    ] {
        values.push(json!({"version":"1","command":"cron.start","request":{"name":"nightly","cron":"0 * * * *","timeout_ms":25000,"target":target,"payload":{},"environment":{"dotenv":[],"variables":{}}}}));
    }
    values
        .into_iter()
        .map(|body| serde_json::from_value(body).expect("canonical input"))
        .collect()
}

fn reply(request: &CronRequest) -> Envelope<CronResponse> {
    let response = match &request.operation {
        CronOperation::Start(start) => CronResponse::Started(ScheduleView {
            id: 7,
            generation: 3,
            name: start.name.clone(),
            cron: start.cron.clone(),
            target: match start.target {
                sc_cron_commands::ExecutionTarget::Rust => TargetKind::Rust,
                sc_cron_commands::ExecutionTarget::Python { .. } => TargetKind::Python,
                sc_cron_commands::ExecutionTarget::Monty { .. } => TargetKind::Monty,
            },
            enabled: true,
            state: ScheduleState::Active,
            next_run_ms: Some(1),
        }),
        CronOperation::End(_) => CronResponse::Ended(EndResult {
            id: 7,
            generation: 3,
        }),
        CronOperation::List(_) => CronResponse::Listed(ListResult {
            items: vec![],
            next_cursor: None,
        }),
    };
    Envelope::success(response)
}

fn verb(request: &CronRequest) -> &'static str {
    match request.operation {
        CronOperation::Start(_) => "start",
        CronOperation::End(_) => "end",
        CronOperation::List(_) => "list",
    }
}

async fn fixture(executor: Arc<RecordingExecutor>) -> sc_runtime::testing::DaemonFixture {
    let mut config = sc_runtime::DaemonConfig::default();
    config.endpoint_flag = Some("tcp:127.0.0.1:0".into());
    let http: Arc<dyn CronExecutor> = executor.clone();
    let mcp: Arc<dyn CronExecutor> = executor;
    sc_runtime::testing::DaemonFixture::start_with_mcp(
        &sc_transport::AppName::new("cron-projection-probe").expect("name"),
        config,
        || async { Ok(service::Stores::open().await) },
        move |_| daemon::cron_commands::router(http),
        move |stores| daemon::mcp::service_with_cron_executor(stores, mcp),
    )
    .await
    .expect("generic fixture")
}

async fn mcp(fixture: &sc_runtime::testing::DaemonFixture, method: &str, params: Value) -> Value {
    let url = format!(
        "http://{}/mcp",
        fixture.endpoint_str().strip_prefix("tcp:").expect("tcp")
    );
    reqwest::Client::new()
        .post(url)
        .header("accept", "application/json, text/event-stream")
        .json(&json!({"jsonrpc":"2.0","id":1,"method":method,"params":params}))
        .send()
        .await
        .expect("MCP response")
        .json::<Value>()
        .await
        .expect("MCP JSON")
}

#[tokio::test]
async fn all_three_ui_projections_forward_same_requests_results_and_deadlines() {
    let executor = Arc::new(RecordingExecutor::default());
    let fixture = fixture(executor.clone()).await;
    for request in requests() {
        let body = serde_json::to_value(&request).expect("wire");
        let expected = reply(&request);
        let expected_json = serde_json::to_value(&expected).expect("response");
        for transport in ["http", "mcp", "cli"] {
            executor
                .replies
                .lock()
                .expect("queue")
                .push_back(expected.clone());
            let before = Instant::now();
            let received = match transport {
                "http" => fixture
                    .client()
                    .post::<Value, _>(&format!("/ops/cron.{}", verb(&request)), &body)
                    .await
                    .expect("HTTP envelope"),
                "mcp" => mcp(
                    &fixture,
                    "tools/call",
                    json!({"name":format!("cron.{}",verb(&request)),"arguments":body}),
                )
                .await["result"]["structuredContent"]
                    .clone(),
                _ => {
                    let command = cli::parse(&[
                        "cron".into(),
                        verb(&request).into(),
                        "--request".into(),
                        body.to_string(),
                        "--json".into(),
                        "--endpoint".into(),
                        fixture.endpoint_str(),
                    ])
                    .expect("CLI parse");
                    let (status, text) = cli::execute(command).await;
                    assert_eq!(status, std::process::ExitCode::SUCCESS);
                    serde_json::from_str(&text).expect("CLI envelope")
                }
            };
            assert_eq!(received, expected_json, "{transport}");
            let after = Instant::now();
            let (observed, deadline) = executor
                .calls
                .lock()
                .expect("calls")
                .pop()
                .expect("one executor call");
            assert_eq!(observed, request);
            assert!(
                deadline >= before + Duration::from_secs(30)
                    && deadline <= after + Duration::from_secs(30)
            );
            assert!(
                executor.calls.lock().expect("calls").is_empty(),
                "exactly one call"
            );
        }
    }
    fixture.shutdown().await.expect("shutdown");
}

#[tokio::test]
async fn actual_tool_and_openapi_schemas_use_versioned_typed_contract() {
    let executor = Arc::new(RecordingExecutor::default());
    let fixture = fixture(executor).await;
    let tools = mcp(&fixture, "tools/list", json!({})).await;
    let tools = tools["result"]["tools"]
        .as_array()
        .expect("actual tools list");
    for name in ["cron.start", "cron.end", "cron.list"] {
        let tool = tools
            .iter()
            .find(|tool| tool["name"] == name)
            .expect("standard verb");
        let variants = tool["inputSchema"]["oneOf"]
            .as_array()
            .expect("same canonical request union");
        assert_eq!(variants.len(), 3);
        for variant in variants {
            assert_eq!(variant["additionalProperties"], false);
            assert_eq!(variant["properties"]["version"]["enum"], json!(["1"]));
            assert_eq!(
                variant["required"],
                json!(["version", "command", "request"])
            );
        }
    }
    let spec: Value = fixture
        .client()
        .get("/openapi.json")
        .await
        .expect("OpenAPI");
    let components = &spec["components"]["schemas"];
    for verb in ["start", "end", "list"] {
        let operation = &spec["paths"][format!("/ops/cron.{verb}")]["post"];
        assert_eq!(
            operation["requestBody"]["content"]["application/json"]["schema"]["$ref"],
            "#/components/schemas/CronRequest"
        );
        let reference =
            operation["responses"]["200"]["content"]["application/json"]["schema"]["$ref"]
                .as_str()
                .expect("typed envelope ref");
        let name = reference
            .strip_prefix("#/components/schemas/")
            .expect("component");
        assert!(
            components[name]["oneOf"].is_array(),
            "canonical success/failure alternatives"
        );
        assert!(
            components[name]["oneOf"][0]["properties"]["data"]["oneOf"].is_array(),
            "typed CronResponse union"
        );
    }
    assert_eq!(
        components["CronRequest"]["oneOf"]
            .as_array()
            .expect("typed request")
            .len(),
        3
    );
    fixture.shutdown().await.expect("shutdown");
}

#[tokio::test]
async fn application_override_errors_survive_all_ui_wrappers() {
    let executor = Arc::new(RecordingExecutor::default());
    let fixture = fixture(executor.clone()).await;
    let request = requests().remove(0);
    let body = serde_json::to_value(&request).expect("wire");
    let error: Envelope<CronResponse> = Envelope::failure(
        OpError::from_code(
            ErrorKind::Conflict,
            "APP.CRON.CONFLICT",
            "application-owned refusal",
        )
        .with_suggestion("follow application policy"),
    );
    let expected = serde_json::to_value(&error).expect("error wire");
    executor
        .replies
        .lock()
        .expect("queue")
        .push_back(error.clone());
    let received: Value = fixture
        .client()
        .post("/ops/cron.end", &body)
        .await
        .expect("HTTP error envelope");
    assert_eq!(received, expected);
    executor
        .replies
        .lock()
        .expect("queue")
        .push_back(error.clone());
    let received = mcp(
        &fixture,
        "tools/call",
        json!({"name":"cron.end","arguments":body}),
    )
    .await;
    assert_eq!(received["result"]["structuredContent"], expected);
    assert_eq!(received["result"]["isError"], true);
    executor.replies.lock().expect("queue").push_back(error);
    let command = cli::parse(&[
        "cron".into(),
        "end".into(),
        "--request".into(),
        body.to_string(),
        "--json".into(),
        "--endpoint".into(),
        fixture.endpoint_str(),
    ])
    .expect("CLI parse");
    let (status, text) = cli::execute(command).await;
    assert_ne!(status, std::process::ExitCode::SUCCESS);
    assert_eq!(
        serde_json::from_str::<Value>(&text).expect("CLI error envelope"),
        expected
    );
    fixture.shutdown().await.expect("shutdown");
}

#[tokio::test]
async fn malformed_or_mismatched_requests_never_reach_executor() {
    let executor = Arc::new(RecordingExecutor::default());
    let fixture = fixture(executor.clone()).await;
    let request = serde_json::to_value(requests().remove(0)).expect("end wire");
    let result: Value = fixture
        .client()
        .post("/ops/cron.start", &request)
        .await
        .expect("canonical rejection");
    assert_eq!(result["error"]["kind"], "invalid_input");
    assert!(
        cli::parse(&[
            "cron".into(),
            "start".into(),
            "--request".into(),
            request.to_string()
        ])
        .is_err()
    );
    let unknown = json!({"version":"1","command":"cron.list","request":{"state":"all","limit":1,"cursor":null,"unknown":true}});
    let result: Value = fixture
        .client()
        .post("/ops/cron.list", &unknown)
        .await
        .expect("strict rejection");
    assert_eq!(result["error"]["kind"], "invalid_input");
    assert!(executor.calls.lock().expect("calls").is_empty());
    fixture.shutdown().await.expect("shutdown");
}

#[tokio::test]
async fn shared_projection_forwards_exact_application_deadline_without_restarting_it() {
    let executor = RecordingExecutor::default();
    let request = requests().remove(0);
    executor
        .replies
        .lock()
        .expect("queue")
        .push_back(reply(&request));
    let deadline = Instant::now() + Duration::from_millis(123);
    service::cron_commands::execute(&executor, "cron.end", request.clone(), deadline).await;
    let calls = executor.calls.lock().expect("calls");
    assert_eq!(calls.len(), 1);
    assert_eq!(calls[0], (request, deadline));
}

#[tokio::test]
async fn duplicate_mcp_fields_are_rejected_before_execution() {
    let executor = Arc::new(RecordingExecutor::default());
    executor
        .replies
        .lock()
        .expect("queue")
        .push_back(Envelope::success(CronResponse::Listed(ListResult {
            items: vec![],
            next_cursor: None,
        })));
    let fixture = fixture(executor.clone()).await;
    let url = format!(
        "http://{}/mcp",
        fixture.endpoint_str().strip_prefix("tcp:").expect("tcp")
    );
    let raw = r#"{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"cron.list","arguments":{"version":"1","command":"cron.list","request":{"state":"all","limit":1,"limit":2,"cursor":null}}}}"#;
    let response = reqwest::Client::new()
        .post(url)
        .header("content-type", "application/json")
        .header("accept", "application/json, text/event-stream")
        .body(raw)
        .send()
        .await
        .expect("MCP")
        .json::<Value>()
        .await
        .expect("JSON");
    fixture.shutdown().await.expect("shutdown");
    assert!(
        executor.calls.lock().expect("calls").is_empty(),
        "duplicate fields must not reach executor"
    );
    assert!(
        response.get("error").is_some(),
        "duplicate input must return a protocol rejection"
    );
}
