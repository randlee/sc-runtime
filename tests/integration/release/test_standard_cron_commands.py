"""Real selected release consumer: canonical generator, native root and actual UI calls."""
import json
import os
from pathlib import Path
import tempfile
import unittest
import jsonschema

from tests.integration.generation.generate_owned import cargo_generate
from tests.integration.generation.owned_process import run
from tests.integration.release.test_published_runtime_ecosystem import check_daemon_closure

ROOT = Path(__file__).resolve().parents[3]
RUST_PROOF = 'use std::{sync::Arc, time::Duration};\nuse serde_json::{Value, json};\nuse sc_runtime::{DaemonConfig, testing::DaemonFixture};\n\nfn request(command: &str, body: Value) -> Value {\n    json!({"version":"1", "command":command, "request":body})\n}\n\n#[tokio::test(flavor = "multi_thread", worker_threads = 2)]\nasync fn real_selected_root_shares_native_state_between_http_mcp_cli() {\n    let root = tempfile::tempdir().unwrap();\n    let log = Arc::new(sc_command::observability::NoopCanonicalLog::named_or_default("c4-proof"));\n    let bindings = daemon::cron_commands::RegisteredTargets::default()\n        .with_python("/configured/nightly.py")\n        .with_monty("def cron_stop(hook):\\n    return False\\ndef cron_tick(hook):\\n    return None\\n");\n    let runtime = daemon::composition::CronServices::default()\n        .with_target_bindings(Arc::new(bindings)).with_command_capacity(3)\n        .start(&daemon::cron::CronConfig::default(), root.path(), log).unwrap();\n    let executor = runtime.executor();\n    let stores = service::Stores::open(root.path().join("widgets.db")).await.unwrap();\n    let mut config = DaemonConfig::default();\n    config.endpoint_flag = Some("tcp:127.0.0.1:0".into());\n    let state = (stores, executor);\n    let fixture = DaemonFixture::start_with_mcp(\n        &sc_transport::AppName::new(daemon::APP_NAME).unwrap(), config,\n        || async { Ok(state) },\n        |(stores, executor)| daemon::routes::router(stores).merge(daemon::cron_commands::router(executor)),\n        |(stores, executor)| daemon::mcp::service_with_cron_executor(stores, executor),\n    ).await.unwrap();\n    let mut examples = Vec::new();\n    let mut responses = Vec::new();\n    // Every target uses the same verbs and mutates the same persisted scheduler.\n    for (index, target) in [json!({"kind":"rust"}),\n        json!({"kind":"python", "module":"/configured/nightly.py"}),\n        json!({"kind":"monty", "source":"def cron_stop(hook):\\n    return False\\ndef cron_tick(hook):\\n    return None\\n"})].into_iter().enumerate() {\n        let name = format!("job-{index}");\n        let start = request("cron.start", json!({"name":name,"cron":"0 * * * *","timeout_ms":1000,\n            "target":target,"payload":{"job":"heartbeat"},"environment":{"dotenv":[],"variables":{}}}));\n        examples.push(start.clone());\n        let started: Value = fixture.client().post("/ops/cron.start", &start).await.unwrap();\n        responses.push(started.clone());\n        assert_eq!(started["ok"], true, "{started}");\n        let listed = request("cron.list",json!({"state":"active","limit":10,"cursor":null}));\n        examples.push(listed.clone());\n        let http: Value = fixture.client().post("/ops/cron.list", &listed).await.unwrap();\n        let rpc = mcp_rpc(&fixture, json!({"jsonrpc":"2.0","id":1,"method":"tools/call",\n            "params":{"name":"cron.list","arguments":listed}})).await;\n        assert_eq!(find_envelope(&rpc).unwrap(), &http);\n        let args = vec!["--json".into(),"--endpoint".into(),fixture.endpoint_str(),\n            "cron".into(),"list".into(),"--request".into(),listed.to_string()];\n        let (_, text) = cli::execute(cli::parse(&args).unwrap()).await;\n        assert_eq!(serde_json::from_str::<Value>(&text).unwrap(), http);\n        let end = request("cron.end", json!({"name":name,"generation":1,"reason":"proof complete"}));\n        examples.push(end.clone());\n        let ended: Value = fixture.client().post("/ops/cron.end", &end).await.unwrap();\n        assert_eq!(ended["ok"], true, "{ended}");\n        let conflict: Value = fixture.client().post("/ops/cron.end", &end).await.unwrap();\n        assert_eq!(conflict["ok"], false);\n        responses.push(conflict);\n    }\n    let tools = mcp_rpc(&fixture,json!({"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}})).await;\n    for name in ["cron.start", "cron.end", "cron.list"] {\n        let tool = tools["result"]["tools"].as_array().unwrap().iter().find(|tool| tool["name"] == name).unwrap();\n        assert!(tool["inputSchema"].get("oneOf").is_some(), "{tool}");\n    }\n    let openapi: Value = fixture.client().get("/openapi.json").await.unwrap();\n    std::fs::write(std::env::var("C4_SCHEMA_RECEIPT").unwrap(),\n        serde_json::to_vec(&json!({"tools":tools,"openapi":openapi,"examples":examples,"responses":responses})).unwrap()).unwrap();\n    fixture.shutdown().await.unwrap();\n    runtime.shutdown().await.unwrap();\n}\nasync fn mcp_rpc(fixture: &DaemonFixture, payload: Value) -> Value {\n    let endpoint = fixture.endpoint();\n    let mut builder = reqwest::Client::builder();\n    let url = match endpoint {\n        sc_transport::Endpoint::Tcp(addr) => format!("http://{addr}/mcp"),\n        sc_transport::Endpoint::Uds(path) => {\n            #[cfg(unix)]\n            {\n                builder = builder.unix_socket(path.clone());\n            }\n            #[cfg(not(unix))]\n            let _ = path;\n            "http://localhost/mcp".to_owned()\n        }\n        _ => "http://localhost/mcp".to_owned(),\n    };\n    let http = builder\n        .connect_timeout(Duration::from_secs(2))\n        .timeout(Duration::from_secs(2))\n        .build()\n        .expect("client");\n    let response = http\n        .post(url)\n        .header("accept", "application/json, text/event-stream")\n        .header("content-type", "application/json")\n        .json(&payload)\n        .send()\n        .await\n        .expect("mcp");\n    assert!(response.status().is_success(), "{}", response.status());\n    response.json().await.expect("mcp json")\n}\n\nfn find_envelope(value: &Value) -> Option<&Value> {\n    let looks_like_envelope = value.get("version").is_some()\n        && value.get("ok").is_some()\n        && value.get("data").is_some()\n        && value.get("error").is_some();\n    if looks_like_envelope {\n        return Some(value);\n    }\n    match value {\n        Value::Object(map) => map.values().find_map(find_envelope),\n        Value::Array(items) => items.iter().find_map(find_envelope),\n        _ => None,\n    }\n}\n'


class StandardCronCompositionTests(unittest.TestCase):
    def test_generated_selected_root_and_package_identity(self):
        with tempfile.TemporaryDirectory(prefix="c4-selected-") as temporary:
            temp = Path(temporary)
            values = temp / "values.toml"
            values.write_text('[values]\ndb="sqlite"\nmcp=true\nlicence="MIT"\ncron=true\ncron_commands=true\n')
            app = temp / "app"
            cargo_generate(ROOT, app, "c4-selected", values)
            registry = os.environ.get("SC_RUNTIME_RELEASE") == "1"
            if not registry:
                run(["python3",str(ROOT/"scripts/new_project.py"),"--patch-checkout",str(app)],timeout=30,check=True)
            else:
                self.assertNotIn("[patch.crates-io]", (app/"Cargo.toml").read_text())
            # Add only the consumer proof, never edit generated framework source.
            (app/"crates/cli/tests/phase_c_cron.rs").write_text(RUST_PROOF)
            env = dict(os.environ, CARGO_BUILD_JOBS="2", CARGO_PROFILE_DEV_DEBUG="0", CARGO_PROFILE_TEST_DEBUG="0")
            env["C4_SCHEMA_RECEIPT"] = str(temp/"schemas.json")
            run(["cargo","+1.96.1","test","-p","c4-selected-cli","--features","daemon/monty",
                 "--test","phase_c_cron"],cwd=app,env=env,timeout=600,check=True)
            receipt = json.loads((temp/"schemas.json").read_text())
            schemas = [tool["inputSchema"] for tool in receipt["tools"]["result"]["tools"]
                       if tool["name"] in ("cron.start", "cron.end", "cron.list")]
            api = receipt["openapi"]
            for verb in ("cron.start", "cron.end", "cron.list"):
                operation = api["paths"]["/ops/"+verb]["post"]
                schemas.append(dict(operation["requestBody"]["content"]["application/json"]["schema"],
                                    components=api["components"]))
                response = dict(operation["responses"]["200"]["content"]["application/json"]["schema"],
                                components=api["components"])
                for example in receipt["responses"]:
                    jsonschema.Draft202012Validator(response).validate(example)
            for schema in schemas:
                validator = jsonschema.Draft202012Validator(schema)
                for example in receipt["examples"]:
                    validator.validate(example)
                    for invalid in (dict(example, version="2"), dict(example, unknown=True),
                                    {key:value for key,value in example.items() if key != "request"}):
                        self.assertFalse(validator.is_valid(invalid), invalid)
            metadata = run(["cargo","+1.96.1","metadata","--locked","--format-version","1","--all-features"],
                           cwd=app,env=env,timeout=120,check=True,capture_output=True,text=True)
            check_daemon_closure(json.loads(metadata.stdout),registry=registry)
