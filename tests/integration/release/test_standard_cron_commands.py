"""Real selected release consumer: canonical generator, native root and actual UI calls."""
import json
import copy
import os
import shutil
import sys
from pathlib import Path
import tempfile
import unittest
import jsonschema

from tests.integration.generation.generate_owned import cargo_generate
from tests.integration.generation.owned_process import run
from tests.integration.release.test_published_runtime_ecosystem import check_daemon_closure

ROOT = Path(__file__).resolve().parents[3]
RUST_PROOF = r'''

use sc_runtime::{DaemonConfig, testing::DaemonFixture};
use serde_json::{Value, json};
use std::{sync::Arc, time::Duration};

fn request(command: &str, body: Value) -> Value {
    json!({"version":"1", "command":command, "request":body})
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn real_selected_root_shares_native_state_between_http_mcp_cli() {
    let root = tempfile::tempdir().unwrap();
    let log = Arc::new(sc_command::observability::NoopCanonicalLog::named_or_default("c4-proof"));
    let bindings = daemon::cron_commands::RegisteredTargets::default()
        .with_python("/configured/nightly.py")
        .with_monty(
            "def cron_stop(hook):\n    return False\ndef cron_tick(hook):\n    return None\n",
        );
    let services = daemon::composition::CronServices::default()
        .with_target_bindings(Arc::new(bindings))
        .with_command_capacity(3);
    let opening_root = root.path().to_path_buf();
    let (fixture, runtime) = mixed_fixture(
        move || services.start(&daemon::cron::CronConfig::default(), &opening_root, log),
        root.path(),
    )
    .await;
    let mut examples = Vec::new();
    let mut responses = Vec::new();
    // Every target uses the same verbs and mutates the same persisted scheduler.
    for (index, target) in [json!({"kind":"rust"}),
        json!({"kind":"python", "module":"/configured/nightly.py"}),
        json!({"kind":"monty", "source":"def cron_stop(hook):\n    return False\ndef cron_tick(hook):\n    return None\n"})].into_iter().enumerate() {
        let name = format!("job-{index}");
        let start = request("cron.start", json!({"name":name,"cron":"0 * * * *","timeout_ms":1000,
            "target":target,"payload":{"job":"heartbeat"},"environment":{"dotenv":[],"variables":{}}}));
        examples.push(start.clone());
        let started: Value = fixture.client().post("/ops/cron.start", &start).await.unwrap();
        responses.push(started.clone());
        assert_eq!(started["ok"], true, "{started}");
        let listed = request("cron.list",json!({"state":"active","limit":10,"cursor":null}));
        examples.push(listed.clone());
        let http: Value = fixture.client().post("/ops/cron.list", &listed).await.unwrap();
        let rpc = mcp_rpc(&fixture, json!({"jsonrpc":"2.0","id":1,"method":"tools/call",
            "params":{"name":"cron.list","arguments":listed}})).await;
        assert_eq!(find_envelope(&rpc).unwrap(), &http);
        let args = vec!["--json".into(),"--endpoint".into(),fixture.endpoint_str(),
            "cron".into(),"list".into(),"--request".into(),listed.to_string()];
        let (_, text) = cli::execute(cli::parse(&args).unwrap()).await;
        assert_eq!(serde_json::from_str::<Value>(&text).unwrap(), http);
        let end = request("cron.end", json!({"name":name,"generation":1,"reason":"proof complete"}));
        examples.push(end.clone());
        let ended: Value = fixture.client().post("/ops/cron.end", &end).await.unwrap();
        assert_eq!(ended["ok"], true, "{ended}");
        let conflict: Value = fixture.client().post("/ops/cron.end", &end).await.unwrap();
        assert_eq!(conflict["ok"], false);
        responses.push(conflict);
    }
    let mut cursor = Value::Null;
    let mut ids = Vec::new();
    for _ in 0..3 {
        let page = request("cron.list",json!({"state":"ended","limit":1,"cursor":cursor}));
        let response: Value = fixture.client().post("/ops/cron.list",&page).await.unwrap();
        let items=response["data"]["data"]["items"].as_array().unwrap();
        assert_eq!(items.len(),1);
        ids.push(items[0]["id"].as_u64().unwrap());
        cursor=response["data"]["data"]["next_cursor"].clone();
    }
    assert!(cursor.is_null());
    assert_eq!(ids,vec![1,2,3]);
    for state in ["active","ended","all"] {
        let page=request("cron.list",json!({"state":state,"limit":10,"cursor":null}));
        let http=call_surface(&fixture,0,page.clone()).await;
        for surface in [1,2] { assert_eq!(call_surface(&fixture,surface,page.clone()).await,http); }
    }
    let tools = mcp_rpc(
        &fixture,
        json!({"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}),
    )
    .await;
    for name in ["cron.start", "cron.end", "cron.list"] {
        let tool = tools["result"]["tools"]
            .as_array()
            .unwrap()
            .iter()
            .find(|tool| tool["name"] == name)
            .unwrap();
        assert!(tool["inputSchema"].get("oneOf").is_some(), "{tool}");
    }
    let openapi: Value = fixture.client().get("/openapi.json").await.unwrap();
    std::fs::write(
        std::env::var("C4_SCHEMA_RECEIPT").unwrap(),
        serde_json::to_vec(
            &json!({"tools":tools,"openapi":openapi,"examples":examples,"responses":responses}),
        )
        .unwrap(),
    )
    .unwrap();
    // Native stop closes command admission while the fixture runtime still owns the worker.
    runtime.shutdown().await.unwrap();
    fixture.shutdown().await.unwrap();
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

use sc_cron::{
    CronError, Generation, NewSchedule, RunOutcome, Schedule, ScheduleId, ScheduleName,
    SchedulePage, ScheduleQuery, ScheduleStore, Timestamp,
};

async fn mixed_fixture<S, C, J>(
    start: impl FnOnce() -> Result<daemon::composition::CronRuntime<S, C, J>, sc_cron::CronError>
    + Send
    + 'static,
    root: &std::path::Path,
) -> (DaemonFixture, daemon::composition::CronRuntime<S, C, J>)
where
    S: sc_cron::ScheduleStore + 'static,
    C: sc_cron::Clock + 'static,
    J: sc_cron::JitterSource + 'static,
{
    let widget_path = root.join("widgets.db");
    let mut config = DaemonConfig::default();
    config.endpoint_flag = Some("tcp:127.0.0.1:0".into());
    let (tx, rx) = tokio::sync::oneshot::channel();
    let fixture = DaemonFixture::start_with_mcp(
        &sc_transport::AppName::new(daemon::APP_NAME).unwrap(),
        config,
        move || async move {
            // The real builder acquires its instance lock before opening stores/startup.
            let stores = service::Stores::open(widget_path)
                .await
                .map_err(daemon::map_store_open)?;
            let runtime = start().map_err(|_| {
                sc_runtime::RuntimeError::stores_failed(
                    "C4.CRON_START",
                    "cron start failed",
                    "inspect the consumer configuration",
                    None,
                )
            })?;
            let executor = runtime.executor();
            assert!(tx.send(runtime).is_ok());
            Ok((stores, executor))
        },
        |(stores, executor)| {
            daemon::routes::router(stores).merge(daemon::cron_commands::router(executor))
        },
        |(stores, executor)| daemon::mcp::service_with_cron_executor(stores, executor),
    )
    .await
    .unwrap();
    (fixture, rx.await.unwrap())
}

struct EventReceiver {
    service: sc_cron::observability::ServiceName,
    events: tokio::sync::watch::Sender<Vec<String>>,
}
impl sc_cron::observability::CanonicalLog for EventReceiver {
    fn service_name(&self) -> &sc_cron::observability::ServiceName {
        &self.service
    }
    fn emit(
        &self,
        event: sc_cron::observability::LogEvent,
    ) -> Result<(), sc_cron::observability::EventError> {
        self.events
            .send_modify(|events| events.push(event.action.as_str().to_owned()));
        Ok(())
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn actual_registered_rust_python_monty_execute_through_selected_native_root() {
    use sc_cron::{
        ManualClock, RunOutcome, ScheduleLimits, ScheduleStore, Scheduler, SeqJitter,
        SqliteScheduleStore,
    };
    let root = tempfile::tempdir().unwrap();
    let python = root.path().join("tick.py");
    std::fs::write(&python,"import json,sys\nif sys.argv[1]=='cron.stop':\n print(json.dumps({'action':'continue'}))\nelse:\n data=json.load(sys.stdin)\n assert data['payload']['job']=='heartbeat'\n").unwrap();
    let monty = "def cron_stop(hook):\n    return {'action': 'continue'}\ndef cron_tick(hook):\n    return None\n";
    let clock = Arc::new(ManualClock::new("2026-10-19T00:00:00Z".parse().unwrap()));
    let store = Arc::new(SqliteScheduleStore::open_in_memory(ScheduleLimits::default()).unwrap());
    let (events, mut observed) = tokio::sync::watch::channel(Vec::new());
    let scheduler = Scheduler::new_with_logger(
        Arc::clone(&store),
        Arc::clone(&clock),
        Arc::new(SeqJitter::default()),
        Arc::new(EventReceiver {
            service: sc_cron::observability::ServiceName::new("c4-execution").unwrap(),
            events,
        }),
    );
    let bindings = daemon::cron_commands::RegisteredTargets::default()
        .with_python(python.to_str().unwrap())
        .with_monty(monty);
    let services =
        daemon::composition::CronServices::default().with_target_bindings(Arc::new(bindings));
    let (fixture, runtime) = mixed_fixture(
        move || services.start_with_scheduler(scheduler),
        root.path(),
    )
    .await;
    for (index, target) in [
        json!({"kind":"rust"}),
        json!({"kind":"python","module":python}),
        json!({"kind":"monty","source":monty}),
    ]
    .into_iter()
    .enumerate()
    {
        let start = request(
            "cron.start",
            json!({"name":format!("execution-{index}"),"cron":"0 * * * * *",
            "timeout_ms":10000,"target":target,"payload":{"job":"heartbeat"},"environment":{"dotenv":[],"variables":{}}}),
        );
        let response: Value = fixture
            .client()
            .post("/ops/cron.start", &start)
            .await
            .unwrap();
        assert_eq!(response["ok"], true, "{response}");
    }
    tokio::time::timeout(Duration::from_secs(5), clock.wait_parks_above(0))
        .await
        .unwrap();
    tokio::time::timeout(
        Duration::from_secs(5),
        clock.advance_settled(chrono::Duration::seconds(60)),
    )
    .await
    .unwrap();
    tokio::time::timeout(
        Duration::from_secs(15),
        observed.wait_for(|events| {
            events
                .iter()
                .filter(|action| action.as_str() == "run.finished")
                .count()
                == 3
        }),
    )
    .await
    .unwrap()
    .unwrap();
    let rows = store.list_active().unwrap();
    assert_eq!(rows.len(), 3);
    for row in rows {
        assert_eq!(row.last_outcome, Some(RunOutcome::Ok));
    }
    // Native stop closes command admission while the fixture runtime still owns the worker.
    runtime.shutdown().await.unwrap();
    fixture.shutdown().await.unwrap();
}

#[derive(Clone, Copy)]
enum GateMode {
    Query,
    AfterCreate,
}
struct GateStore {
    store: sc_cron::SqliteScheduleStore,
    mode: GateMode,
    entered: std::sync::mpsc::Sender<()>,
    gate: (std::sync::Mutex<bool>, std::sync::Condvar),
}
impl GateStore {
    fn new(mode: GateMode) -> (Arc<Self>, std::sync::mpsc::Receiver<()>) {
        let (entered, received) = std::sync::mpsc::channel();
        (
            Arc::new(Self {
                store: sc_cron::SqliteScheduleStore::open_in_memory(
                    sc_cron::ScheduleLimits::default(),
                )
                .unwrap(),
                mode,
                entered,
                gate: (std::sync::Mutex::new(false), std::sync::Condvar::new()),
            }),
            received,
        )
    }
    fn hold(&self) {
        self.entered.send(()).unwrap();
        let (mutex, changed) = &self.gate;
        let mut released = mutex.lock().unwrap();
        while !*released {
            released = changed.wait(released).unwrap();
        }
    }
    fn release(&self) {
        *self.gate.0.lock().unwrap() = true;
        self.gate.1.notify_all();
    }
}
// Unwind releases real native work before DaemonFixture's physical Drop join.
struct ReleaseGate(Arc<GateStore>);
impl Drop for ReleaseGate {
    fn drop(&mut self) {
        self.0.release();
    }
}

impl ScheduleStore for GateStore {
    fn operation_support(&self) -> sc_cron::StoreOperationSupport {
        self.store.operation_support()
    }
    fn with_operation_context<T>(
        &self,
        context: &sc_cron::StoreOperationContext,
        operation: impl FnOnce(&Self) -> Result<T, CronError>,
    ) -> Result<T, CronError> {
        self.store
            .with_operation_context(context, |_| operation(self))
    }
    fn query(&self, query: &ScheduleQuery) -> Result<SchedulePage, CronError> {
        if matches!(self.mode, GateMode::Query) {
            self.hold();
        }
        self.store.query(query)
    }
    fn create(&self, schedule: &NewSchedule, at: Timestamp) -> Result<Schedule, CronError> {
        let row = self.store.create(schedule, at)?;
        if matches!(self.mode, GateMode::AfterCreate) {
            self.hold();
        }
        Ok(row)
    }
    fn get(&self, id: ScheduleId) -> Result<Option<Schedule>, CronError> {
        self.store.get(id)
    }
    fn list_active(&self) -> Result<Vec<Schedule>, CronError> {
        self.store.list_active()
    }
    fn record_run(
        &self,
        id: ScheduleId,
        generation: Generation,
        at: Timestamp,
        outcome: &RunOutcome,
    ) -> Result<bool, CronError> {
        self.store.record_run(id, generation, at, outcome)
    }
    fn set_next(
        &self,
        id: ScheduleId,
        generation: Generation,
        next: Timestamp,
    ) -> Result<bool, CronError> {
        self.store.set_next(id, generation, next)
    }
    fn stop(
        &self,
        id: ScheduleId,
        generation: Generation,
        at: Timestamp,
        reason: &str,
    ) -> Result<bool, CronError> {
        self.store.stop(id, generation, at, reason)
    }
    fn set_enabled(&self, name: &ScheduleName, enabled: bool) -> Result<bool, CronError> {
        self.store.set_enabled(name, enabled)
    }
    fn disable(
        &self,
        id: ScheduleId,
        generation: Generation,
        at: Timestamp,
        reason: &str,
    ) -> Result<bool, CronError> {
        self.store.disable(id, generation, at, reason)
    }
    fn set_enabled_many(&self, flags: &[(ScheduleName, bool)]) -> Result<bool, CronError> {
        self.store.set_enabled_many(flags)
    }
    fn replace_and_set_enabled(
        &self,
        schedule: &NewSchedule,
        enabled: bool,
        at: Timestamp,
        flags: &[(ScheduleName, bool)],
    ) -> Result<Option<Schedule>, CronError> {
        self.store
            .replace_and_set_enabled(schedule, enabled, at, flags)
    }
}

async fn await_entries(received: std::sync::mpsc::Receiver<()>, count: usize) {
    tokio::task::spawn_blocking(move || {
        for _ in 0..count {
            received.recv_timeout(Duration::from_secs(5)).unwrap();
        }
    })
    .await
    .unwrap();
}

async fn call_surface(fixture: &DaemonFixture, surface: usize, body: Value) -> Value {
    match surface % 3 {
        0 => fixture
            .client()
            .post("/ops/cron.list", &body)
            .await
            .unwrap(),
        1 => {
            let rpc = mcp_rpc(
                fixture,
                json!({"jsonrpc":"2.0","id":1,"method":"tools/call",
                "params":{"name":"cron.list","arguments":body}}),
            )
            .await;
            find_envelope(&rpc).unwrap().clone()
        }
        _ => {
            let args = vec![
                "--json".into(),
                "--endpoint".into(),
                fixture.endpoint_str(),
                "cron".into(),
                "list".into(),
                "--request".into(),
                body.to_string(),
            ];
            let (_, text) = cli::execute(cli::parse(&args).unwrap()).await;
            serde_json::from_str(&text).unwrap()
        }
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn shared64_is_physical_across_mixed_uis_executors_and_dropped_waiters() {
    use sc_cron::{Scheduler, ShutdownBudget, SystemClock, SystemJitter};
    use std::future::Future;
    let root = tempfile::tempdir().unwrap();
    let (store, received) = GateStore::new(GateMode::Query);
    let scheduler = Scheduler::new(
        Arc::clone(&store),
        Arc::new(SystemClock),
        Arc::new(SystemJitter),
    );
    // No capacity override: this must exercise the actual standard64 default.
    let (fixture, runtime) = mixed_fixture(
        move || daemon::composition::CronServices::default().start_with_scheduler(scheduler),
        root.path(),
    )
    .await;
    let fixture = Arc::new(fixture);
    let second = sc_cron_commands::StandardCronExecutor::new(
        runtime.scheduler(),
        Arc::new(daemon::cron_commands::RegisteredTargets::default()),
        runtime.stop_control().operation_context(),
    );
    let release = ReleaseGate(Arc::clone(&store));
    let list = request("cron.list", json!({"state":"all","limit":10,"cursor":null}));
    let mut waiters = Vec::new();
    for surface in 0..64 {
        let fixture = Arc::clone(&fixture);
        let body = list.clone();
        waiters.push(tokio::spawn(async move {
            call_surface(&fixture, surface, body).await
        }));
    }
    await_entries(received, 64).await;
    for surface in 0..3 {
        let refused = call_surface(&fixture, surface, list.clone()).await;
        assert_eq!(
            refused["error"]["code"], "CRON.COMMAND_CAPACITY_EXHAUSTED",
            "{refused}"
        );
    }
    let request: sc_cron_commands::CronRequest = serde_json::from_value(list.clone()).unwrap();
    let extra = sc_cron_commands::CronExecutor::execute(
        &second,
        request.clone(),
        std::time::Instant::now() + Duration::from_secs(5),
    )
    .await;
    assert_eq!(
        serde_json::to_value(extra).unwrap()["error"]["code"],
        "CRON.COMMAND_CAPACITY_EXHAUSTED"
    );
    for waiter in &waiters {
        waiter.abort();
    }
    for waiter in waiters {
        let _ = waiter.await;
    }
    let extra = sc_cron_commands::CronExecutor::execute(
        &second,
        request,
        std::time::Instant::now() + Duration::from_secs(5),
    )
    .await;
    assert_eq!(
        serde_json::to_value(extra).unwrap()["error"]["code"],
        "CRON.COMMAND_CAPACITY_EXHAUSTED",
        "dropping UI waiters must not release native slots"
    );
    let now = std::time::Instant::now();
    runtime
        .stop_control()
        .request_stop(ShutdownBudget::new(now, now).unwrap())
        .unwrap();
    let mut physical_join = Box::pin(runtime.shutdown());
    let pending = std::future::poll_fn(|cx| {
        std::task::Poll::Ready(physical_join.as_mut().poll(cx).is_pending())
    })
    .await;
    assert!(
        pending,
        "physical shutdown completed while admitted native work was held"
    );
    // Independent stop has closed intake while all64 ordinary slots are retained.
    let stopped = call_surface(&fixture, 0, list).await;
    assert_eq!(stopped["ok"], false);
    drop(release);
    physical_join.await.unwrap();
    drop(second);
    Arc::try_unwrap(fixture)
        .ok()
        .unwrap()
        .shutdown()
        .await
        .unwrap();
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn already_committed_mutation_remains_owned_and_never_claims_rollback() {
    use sc_cron::{Scheduler, ShutdownBudget, SystemClock, SystemJitter};
    use std::future::Future;
    let root = tempfile::tempdir().unwrap();
    let (store, received) = GateStore::new(GateMode::AfterCreate);
    let scheduler = Scheduler::new(
        Arc::clone(&store),
        Arc::new(SystemClock),
        Arc::new(SystemJitter),
    );
    let (fixture, runtime) = mixed_fixture(
        move || daemon::composition::CronServices::default().start_with_scheduler(scheduler),
        root.path(),
    )
    .await;
    let fixture = Arc::new(fixture);
    let release = ReleaseGate(Arc::clone(&store));
    let client = fixture.client().clone();
    let body = request(
        "cron.start",
        json!({"name":"admitted","cron":"0 * * * *","timeout_ms":1000,
        "target":{"kind":"rust"},"payload":{"job":"heartbeat"},"environment":{"dotenv":[],"variables":{}}}),
    );
    let waiter = tokio::spawn(async move {
        client
            .post::<Value, _>("/ops/cron.start", &body)
            .await
            .unwrap()
    });
    await_entries(received, 1).await;
    assert_eq!(
        store.store.list_active().unwrap().len(),
        1,
        "commit physically happened before cutoff"
    );
    let now = std::time::Instant::now();
    runtime
        .stop_control()
        .request_stop(ShutdownBudget::new(now, now).unwrap())
        .unwrap();
    let mut physical_join = Box::pin(runtime.shutdown());
    assert!(
        std::future::poll_fn(|cx| std::task::Poll::Ready(
            physical_join.as_mut().poll(cx).is_pending()
        ))
        .await
    );
    drop(release);
    let response = waiter.await.unwrap();
    assert_eq!(
        response["ok"], false,
        "a cutoff must not produce bounded success: {response}"
    );
    let details = response["error"]["details"].to_string();
    assert!(
        details.contains("committed"),
        "known committed mutation must retain persistence truth: {response}"
    );
    assert!(!details.contains("rolled_back"), "{response}");
    physical_join.await.unwrap();
    assert!(
        store
            .store
            .get(sc_cron::ScheduleId::new(1))
            .unwrap()
            .is_some()
    );
    Arc::try_unwrap(fixture)
        .ok()
        .unwrap()
        .shutdown()
        .await
        .unwrap();
}

#[tokio::test]
async fn finite_override_validation_precedes_worker_startup() {
    for capacity in [0, 65537] {
        let scheduler = sc_cron::Scheduler::new(
            Arc::new(
                sc_cron::SqliteScheduleStore::open_in_memory(sc_cron::ScheduleLimits::default())
                    .unwrap(),
            ),
            Arc::new(sc_cron::SystemClock),
            Arc::new(sc_cron::SystemJitter),
        );
        assert!(
            daemon::composition::CronServices::default()
                .with_command_capacity(capacity)
                .start_with_scheduler(scheduler)
                .is_err()
        );
    }
    let (store, received) = GateStore::new(GateMode::Query);
    let scheduler = sc_cron::Scheduler::new(
        Arc::clone(&store),
        Arc::new(sc_cron::SystemClock),
        Arc::new(sc_cron::SystemJitter),
    );
    let runtime = daemon::composition::CronServices::default()
        .with_command_capacity(1)
        .start_with_scheduler(scheduler)
        .unwrap();
    let release = ReleaseGate(Arc::clone(&store));
    let executor = runtime.executor();
    let request: sc_cron_commands::CronRequest = serde_json::from_value(request(
        "cron.list",
        json!({"state":"all","limit":1,"cursor":null}),
    ))
    .unwrap();
    let admitted_request = request.clone();
    let admitted = tokio::spawn(async move {
        executor
            .execute(
                admitted_request,
                std::time::Instant::now() + Duration::from_secs(5),
            )
            .await
    });
    await_entries(received, 1).await;
    let refused = runtime
        .executor()
        .execute(request, std::time::Instant::now() + Duration::from_secs(5))
        .await;
    assert_eq!(
        serde_json::to_value(refused).unwrap()["error"]["code"],
        "CRON.COMMAND_CAPACITY_EXHAUSTED"
    );
    drop(release);
    admitted.await.unwrap().into_result().unwrap();
    runtime.shutdown().await.unwrap();
}

struct ConsumerPolicy;
impl sc_cron_commands::CronExecutor for ConsumerPolicy {
    fn execute(&self, _: sc_cron_commands::CronRequest, _: std::time::Instant)
        -> sc_cron_commands::ExecutorFuture<'_> {
        Box::pin(async { sc_command::Envelope::failure(sc_command::OpError::from_code(
            sc_command::ErrorKind::Unavailable,"APP.CRON_POLICY","consumer policy declined this command")
            .with_suggestion("follow the application's scheduling policy")) })
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn consumer_executor_and_cleanup_own_policy_and_physical_completion() {
    let root=tempfile::tempdir().unwrap();
    let store=Arc::new(sc_cron::SqliteScheduleStore::open_in_memory(sc_cron::ScheduleLimits::default()).unwrap());
    let scheduler=sc_cron::Scheduler::new(Arc::clone(&store),Arc::new(sc_cron::SystemClock),Arc::new(sc_cron::SystemJitter));
    let (release,accepted)=tokio::sync::oneshot::channel::<()>();
    let (entered,observed)=tokio::sync::oneshot::channel::<()>();
    let completed=Arc::new(std::sync::atomic::AtomicBool::new(false));
    let completion=Arc::clone(&completed);
    let owned=tokio::spawn(async move { accepted.await.unwrap(); completion.store(true,std::sync::atomic::Ordering::SeqCst); });
    let services=daemon::composition::CronServices::default()
        .with_executor(Arc::new(ConsumerPolicy))
        .with_command_cleanup(move || async move {
            entered.send(()).unwrap();
            owned.await.unwrap();
            // A reported custom cleanup error still cannot skip the native physical join.
            Err(sc_cron::CronError::InvalidSpec("application cleanup report".into()))
        });
    let (fixture,runtime)=mixed_fixture(move || services.start_with_scheduler(scheduler),root.path()).await;
    let body=request("cron.list",json!({"state":"all","limit":1,"cursor":null}));
    let expected=call_surface(&fixture,0,body.clone()).await;
    assert_eq!(expected["error"]["code"],"APP.CRON_POLICY");
    for surface in [1,2] { assert_eq!(call_surface(&fixture,surface,body.clone()).await,expected); }
    assert!(store.list_active().unwrap().is_empty());
    let control=runtime.stop_control();
    let joined=tokio::spawn(runtime.shutdown());
    observed.await.unwrap();
    assert!(!joined.is_finished(),"custom physical work was released early");
    release.send(()).unwrap();
    assert!(joined.await.unwrap().is_err());
    assert!(completed.load(std::sync::atomic::Ordering::SeqCst));
    assert_eq!(*control.subscribe_status().borrow(),sc_cron::ShutdownStatus::Joined);
    fixture.shutdown().await.unwrap();
}
'''


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
                run([sys.executable,str(ROOT/"scripts/new_project.py"),"--patch-checkout",str(app)],timeout=30,check=True)
            else:
                self.assertNotIn("[patch.crates-io]", (app/"Cargo.toml").read_text())
            env = dict(os.environ, CARGO_BUILD_JOBS="2", CARGO_PROFILE_DEV_DEBUG="0", CARGO_PROFILE_TEST_DEBUG="0")
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
                    if example["command"] == "cron.start":
                        invalid=copy.deepcopy(example)
                        del invalid["request"]["payload"]
                        self.assertFalse(validator.is_valid(invalid))
                        invalid=copy.deepcopy(example)
                        invalid["request"]["target"]={"kind":"unregistered_kind"}
                        self.assertFalse(validator.is_valid(invalid))
                        invalid=copy.deepcopy(example)
                        del invalid["request"]["target"]
                        self.assertFalse(validator.is_valid(invalid))
                        for payload in (None,["opaque",1],"opaque"):
                            valid=copy.deepcopy(example)
                            valid["request"]["payload"]=payload
                            validator.validate(valid)
            normal = run(["cargo","+1.96.1","tree","-p","c4-selected-cli","-e","normal","--prefix","none"],
                         cwd=app,env=env,timeout=120,check=True,capture_output=True,text=True).stdout
            for forbidden in ("sc-cron v", "rusqlite v", "libsqlite3-sys v", "pydantic-monty v", "axum v", "rmcp v"):
                self.assertNotIn(forbidden, normal)
            metadata = run(["cargo","+1.96.1","metadata",*(["--locked"] if registry else []),"--format-version","1","--all-features"],
                           cwd=app,env=env,timeout=120,check=True,capture_output=True,text=True)
            check_daemon_closure(json.loads(metadata.stdout),registry=registry)
            self.run_documented_consumer_schema(app, env, registry)

    def run_documented_consumer_schema(self, app, env, registry):
        # Execute C5's existing public application-schema example unchanged in this consumer.
        source=ROOT/"tests/integration/cron-consumer"
        proof=app/".c4-consumer-schema-proof"
        (proof/"src").mkdir(parents=True)
        (proof/"tests/support").mkdir(parents=True)
        for relative in ("src/lib.rs", "tests/application_schema.rs"):
            shutil.copyfile(source/relative,proof/relative)
        manifest=(source/"Cargo.toml").read_text()
        dependency='version = "=0.2.0"' if registry else 'path = '+json.dumps(str(ROOT/"crates/sc-cron"))
        manifest=manifest.replace('path = "../../../crates/sc-cron"',dependency)
        (proof/"Cargo.toml").write_text(manifest)
        support=(source/"tests/support/mod.rs").read_text().replace(
            '../../../../../crates/sc-cron/tests/hook_environment/owned_child.rs',
            (ROOT/"crates/sc-cron/tests/hook_environment/owned_child.rs").as_posix())
        (proof/"tests/support/mod.rs").write_text(support)
        build=(source/"build.rs").read_text().replace(
            '../../../crates/sc-cron/tests/hook_environment',
            (ROOT/"crates/sc-cron/tests/hook_environment").as_posix())
        (proof/"build.rs").write_text(build)
        run(["cargo","test","--manifest-path",str(proof/"Cargo.toml"),"--test","application_schema"],
            cwd=app,env=env,timeout=600,check=True)

    def test_default_off_has_no_adapter_or_cron_and_mcp_stays_disabled(self):
        with tempfile.TemporaryDirectory(prefix="c4-default-off-") as temporary:
            temp=Path(temporary)
            values=temp/"values.toml"
            values.write_text('[values]\ndb="none"\nmcp=false\nlicence="MIT"\ncron=false\ncron_commands=false\n')
            app=temp/"app"
            cargo_generate(ROOT,app,"c4-default-off",values)
            if os.environ.get("SC_RUNTIME_RELEASE") != "1":
                run([sys.executable,str(ROOT/"scripts/new_project.py"),"--patch-checkout",str(app)],timeout=30,check=True)
            else:
                self.assertNotIn("[patch.crates-io]",(app/"Cargo.toml").read_text())
            for package in ("c4-default-off-cli", "c4-default-off-daemon"):
                tree=run(["cargo","tree","-p",package,"-e","normal","--prefix","none"],
                    cwd=app,timeout=120,check=True,capture_output=True,text=True).stdout
                self.assertNotIn("sc-cron v",tree)
                self.assertNotIn("sc-cron-commands v",tree)
                self.assertNotIn("pydantic-monty v",tree)
                if package.endswith("-cli"):
                    for forbidden in ("rusqlite v", "libsqlite3-sys v", "axum v", "rmcp v"):
                        self.assertNotIn(forbidden,tree)
            self.assertFalse((app/"crates/daemon/src/mcp.rs").exists())
            runtime=run(["cargo","tree","-p","sc-runtime","-e","normal","--prefix","none"],
                cwd=app,timeout=120,check=True,capture_output=True,text=True).stdout
            self.assertNotIn("sc-cron v",runtime)
            self.assertNotIn("sc-cron-commands v",runtime)

    def test_untouched_generated_standard_sequence(self):
        # Keep recipe qualification independent: one failure cannot conceal UI/lifecycle findings.
        with tempfile.TemporaryDirectory(prefix="c4-standard-") as temporary:
            temp=Path(temporary)
            values=temp/"values.toml"
            values.write_text('[values]\ndb="sqlite"\nmcp=true\nlicence="MIT"\ncron=true\ncron_commands=true\n')
            app=temp/"app"
            cargo_generate(ROOT,app,"c4-selected",values)
            if os.environ.get("SC_RUNTIME_RELEASE") != "1":
                run([sys.executable,str(ROOT/"scripts/new_project.py"),"--patch-checkout",str(app)],timeout=30,check=True)
            else:
                self.assertNotIn("[patch.crates-io]",(app/"Cargo.toml").read_text())
            env=dict(os.environ,CARGO_BUILD_JOBS="2",CARGO_PROFILE_DEV_DEBUG="0",CARGO_PROFILE_TEST_DEBUG="0")
            for recipe in ("setup","lint","test","build-cli","build-daemon"):
                run(["just",recipe],cwd=app,env=env,timeout=600,check=True)
