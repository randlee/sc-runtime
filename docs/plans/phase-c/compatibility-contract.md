# Phase C compatibility contract

Status: approved scope, planned implementation; readiness pending fresh scope,
critical, QA and actual anchor checks. This file is the single contract
specification consumed by c-1/c-3/c-5/c-6/c-4; existing anchor records and quotes
remain unchanged. User-approved new facts await the parent's separate capture
packet. External consumer evidence informs tests, not scope authority.

## Requirements and decisions

REQ-CRON-0020/0021, REQ-CRNC-0001/0002, NFR-CRNC-0001 and REQ-RUN-0332
define the new target. ADR-RUN-0018/0019 and ADR-CRNC-0001 govern placement.
Existing REQ-CRON-0003 covers Rust/Python/Monty; no port/new kind/catalog.
REQ-RT-0005/0006 adds Windows Ctrl-Break and real fixture cleanup in c-3.

## Contract signatures frozen by c-1

These are required callable shapes, not claims of existing implementation.
Existing APIs keep source-compatible legacy defaults.

```rust
pub mod observability {
    pub use sc_command::observability::{CanonicalLog, NoopCanonicalLog, SharedCanonicalLog};
    pub use sc_observability_types::{LogEvent, ServiceName};
    pub use sc_observability_types::v2::EventError;
}
#[derive(Clone)]
pub struct StoreOperationContext { /* private shared controller */ }
impl StoreOperationContext {
    pub fn deadline(&self) -> Option<std::time::Instant>;
    pub fn stop_requested(&self) -> bool;
    pub fn check(&self) -> Result<(), CronError>;
}
pub enum StoreOperationSupport { Legacy, Cooperative }
pub trait ScheduleStore: Send + Sync {
    // Existing methods retained. New defaults reject before invoking closure.
    fn operation_support(&self) -> StoreOperationSupport { StoreOperationSupport::Legacy }
    fn with_operation_context<T>(&self, context: &StoreOperationContext,
        operation: impl FnOnce(&Self) -> Result<T, CronError>) -> Result<T, CronError>
        where Self: Sized;
    fn query(&self, query: &ScheduleQuery) -> Result<SchedulePage, CronError>;
    // Both new methods have fixed unsupported default bodies, not required methods.
}
pub struct ShutdownBudget { /* checked absolute G/K, K >= G */ }
impl ShutdownBudget {
    pub fn new(grace: std::time::Instant, cleanup: std::time::Instant)
        -> Result<Self, CronError>;
}
#[derive(Clone)]
pub struct WorkerStopControl { /* independently callable; no join ownership */ }
impl WorkerStopControl {
    pub fn request_stop(&self, budget: ShutdownBudget) -> Result<(), CronError>;
    pub fn cancel_now(&self);
    pub fn operation_context(&self) -> StoreOperationContext;
    pub fn subscribe_status(&self) -> tokio::sync::watch::Receiver<ShutdownStatus>;
}
impl WorkerHandle {
    pub fn stop_control(&self) -> WorkerStopControl;
    pub async fn join_owned(self) -> Result<ShutdownReport, CronError>;
}
pub struct ShutdownReport {
    pub grace_exceeded: bool, pub cleanup_exceeded: bool,
    pub cleanup_persisted: bool,
}
pub enum ScheduleSelection { Active, Ended, All }
pub struct ScheduleQuery {
    pub selection: ScheduleSelection, pub name: Option<ScheduleName>,
    pub after_id: Option<ScheduleId>, pub limit: u32, // 1..=1000
}
pub struct SchedulePage {
    pub items: Vec<Schedule>, pub next_after_id: Option<ScheduleId>,
}
// Scheduler<S,C,J>, existing bounds retained:
// spawn_worker_cooperative(job: Arc<dyn Job>) -> Result<WorkerHandle,CronError>
// async start_with_context(spec: ScheduleSpec, context: StoreOperationContext)
//     -> Result<Schedule,CronError>
// async stop_with_context(id: ScheduleId, generation: Generation, reason: String,
//     context: StoreOperationContext) -> Result<Schedule,CronError>
// async query_with_context(query: ScheduleQuery, context: StoreOperationContext)
//     -> Result<SchedulePage,CronError>
```

c-1 freezes concrete query/status/error enums and contract double. Query uses
active/ended/all, optional name, ascending id, after-id cursor, positive limit
at most 1000; default 100. No new table or advertised history via list_active.
Cancelled store operations use typed fixed cancellation/store errors with
stage/id/generation only; no SQL/provider prose. c-1 records new code inventory
without changing legacy row outcomes or fabricating successful cleanup.

## Admission, budget and physical ownership

All tracked worker/command operations register before submission and before
blocking locks, including initial load, reload, query, decode and every write.
The same live context reaches SQLite busy/progress/interrupt and row loops.
Legacy capability rejects cooperative admission before work; forwarding stores
must forward both capability and scope. No second timeout-only protocol.

First stop closes launch/new mutation admission. Admitted ordinary work drains
to G, then cancels/rolls back; no late ordinary commit. Separate restricted
cleanup can only write terminal bookkeeping for the stopped id/generation
under existing transaction fencing until K. It cannot launch jobs, create,
restart, enable or invoke external calls. At K it cannot start/retry/commit;
rollback/restoration precede unlock. Failed cleanup reports typed failure and
cleanup_persisted=false, with no unbounded retries or hidden budget.

Stop time uses real std::time::Instant independently of schedule Clock.
One synchronized monotonic phase/epoch/controller owns earliest G/K.
Direct stop/shortening wakes waiters and interrupts SQL independently of the
worker loop; register/shorten/wake cannot lose a race. Timer wakes recheck real
time; callbacks consult the same latched state. ManualClock does not advance
stop time and paused Tokio is not a real SQLite timing proof.

Caller absolute budgets accept equal deadlines. Existing consumer reference
uses T>0: G=s+T, K=s+2T once; second stop cancels immediately without extending
K. T=0: G=K=s, immediate cancellation/kill and no cleanup-write allowance.
There is no invented positive C0 or literal instantaneous physical join.
The consumer's no-time-left lease expiry exception remains its own QA concern.

One owner physically joins admitted closures/tasks after waiters disappear.
Supported cooperative implementations must prove completion by K. A provider
overrun publishes typed expiry via independent status while join remains
owned; early notification never counts as bounded completion. Arbitrary Rust
native/self-spawned descendants or synchronous Monty hosts cannot be forcibly
preempted. Preserve ordinary three-target functionality without hard claims.

## Portable fixture entrypoint

Keep endpoint ownership continuous: the existing transport can bind
tcp:127.0.0.1:0 and return Listener::local_endpoint while retaining the same
listener. Runtime's private bound-ready handshake and DaemonFixture actual
endpoint accessor already use this path. Cleanup preserves it: no reserve-port/
drop/rebind, retries, sleeps or timeout workaround. Health confirmation follows
actual binding, not store readiness.

Real-signal child tests use the production run path, not the private fixture
trigger. C-1 freezes this additive ready-state DaemonBuilder method; c-3
implements it in the existing bind/cleanup runner. C-4 releases that runtime
0.2.0 artifact. Existing run/run_with_cleanup delegate with a no-op callback;
no transport API/version change is needed.

```rust
pub async fn run_with_cleanup_and_bound<FBound, FCleanup, Cleanup>(
    self, on_bound: FBound, cleanup: FCleanup,
) -> Result<(), RuntimeError>
where FBound: FnOnce(sc_transport::Endpoint) + Send + 'static,
      FCleanup: FnOnce() -> Cleanup,
      Cleanup: Future<Output=()>;
```

Invoke on_bound once only after successful binding with the actual Endpoint
while retaining the listener; pre-bind failure invokes none. Preserve existing
cleanup/error/lock ordering. This reports binding, not health readiness. Child
IPC reports that endpoint and the recipient confirms health before real OS
signals. No new public shutdown trigger or duplicate lifecycle is introduced.
Callbacks return promptly and handle a dropped receipt receiver without
panicking; callers retain/await the owning run future. Port-zero is explicit
configuration, preserving endpoint precedence and ordinary private UDS fixtures.
C-3 tests concurrent port-zero fixtures with distinct nonzero actual endpoints
and agreeing client/accessors, and a causal competing bind to that exact owned
endpoint fails between receipt and readiness. A reserve/drop negative must
permit takeover. Pre-bind failure emits no receipt and still preserves cleanup.

```rust
pub async fn start_with_cleanup<S,FStores,Fut,FRoutes,FCleanup,Cleanup>(
    app: &AppName, config: DaemonConfig, stores: FStores,
    routes: FRoutes, cleanup: FCleanup,
) -> Result<DaemonFixture, RuntimeError>
where S: Clone + Send + Sync + 'static,
      FStores: FnOnce() -> Fut + Send + 'static,
      Fut: Future<Output=Result<S,RuntimeError>> + Send + 'static,
      FRoutes: FnOnce(S) -> OpenApiRouter + Send + 'static,
      FCleanup: FnOnce() -> Cleanup + Send + 'static,
      Cleanup: Future<Output=()> + Send + 'static;
```

C-3 implements this additive constructor using the current internal runner and
production run_with_cleanup; c-1 freezes the signature with existing generic
type identities. Preserve once-only cleanup on startup errors and normal stop,
primary error, retained stores/logger/instance lock, endpoint order and owned
fixture-thread joins. Ordinary constructors delegate with empty cleanup.

## Standard JSON and selected UI wiring

```json
{"version":"1","command":"cron.start","request":{"name":"nightly","cron":"0 * * * *","timeout_ms":25000,"target":{"kind":"python","module":"/configured/nightly.py"},"payload":{},"environment":{"dotenv":[],"variables":{}}}}
{"version":"1","command":"cron.end","request":{"name":"nightly","generation":1,"reason":"operator stopped"}}
{"version":"1","command":"cron.list","request":{"state":"all","limit":100,"cursor":null}}
```

Rust target is `{kind:rust}` with existing app Job/payload dispatcher; Monty
is `{kind:monty,source:...}` with explicit feature/capability. No dynamic Rust
function/catalog, ATM phase/team/until, six-field-only grammar or new database.
Use canonical Envelope version1 and typed Started/Ended/Listed result data;
list exposes safe id/generation/name/expression/target-kind/enabled/state/next
time plus next_cursor, not secret environment/source/payload. Strict DTOs
reject unknown/duplicate fields and overflow. c-1 binds existing core field
limits plus a finite aggregate JSON limit and namespaced codes:
invalid_input for bad expression/spec, conflict for active/fenced,
not_found for absent rows, unavailable for busy/cancelled/capability,
internal for unexpected failures; retain existing Envelope/OpError semantics.

### Injectable operation/executor contracts (c-1 baseline)

CronRequest below is the sole wire identity; there is no CronCommandRequest.
All referenced DTO fields, strict unknown/duplicate/version validation and
serialization are frozen by c-1. Flattened serde derives alone do not enforce
that validation. CronExecutor is default-feature, object-safe and uses only
std Future/Pin/Instant and DTO/Envelope types. No production implementation or
success/unsupported stub is supplied by c-1.

```rust
// sc-cron-commands::contract; available with default features, no sc-cron import.
#[derive(Serialize, Deserialize)]
pub struct CronRequest {
    pub version: String,
    #[serde(flatten)]
    pub operation: CronOperation,
}
#[derive(Serialize, Deserialize)]
#[serde(tag = "command", content = "request")]
pub enum CronOperation {
    #[serde(rename = "cron.start")] Start(StartRequest),
    #[serde(rename = "cron.end")] End(EndRequest),
    #[serde(rename = "cron.list")] List(ListRequest),
}
// StartRequest, EndRequest, ListRequest, ExecutionTarget, CronResponse,
// ScheduleView, EndResult and ListResult are the sole DTO definitions here.
// Exact-field/version/duplicate validation is part of the c-1 wire contract;
// flattened serde derives alone must not be claimed to enforce it.

pub type ExecutorFuture<'a> = std::pin::Pin<Box<dyn std::future::Future<
    Output = sc_command::Envelope<CronResponse>> + Send + 'a>>;

pub trait CronExecutor: Send + Sync {
    fn execute(&self, request: CronRequest, deadline: std::time::Instant)
        -> ExecutorFuture<'_>;
}
```

Core-native operation port in sc-cron/src/consumer_contract.rs, re-exported
at the crate root, imports no adapter DTO or Envelope types. C-5 implements
it for real Scheduler using tracked native operations. C-6 uses an operation
double and does not require that implementation.

```rust
// sc-cron::consumer_contract (public items re-exported at crate root).
pub type CronOperationFuture<'a, T> = std::pin::Pin<Box<dyn std::future::Future<
    Output = Result<T, CronError>> + Send + 'a>>;

pub struct EndSchedule {
    pub name: ScheduleName,
    pub generation: Generation,
    pub reason: String,
}
pub struct EndedSchedule {
    pub id: ScheduleId,
    pub generation: Generation,
}
pub trait CronOperations: Send + Sync {
    fn start(&self, context: StoreOperationContext, spec: ScheduleSpec)
        -> CronOperationFuture<'_, Schedule>;
    fn end(&self, context: StoreOperationContext, request: EndSchedule)
        -> CronOperationFuture<'_, EndedSchedule>;
    fn query(&self, context: StoreOperationContext, query: ScheduleQuery)
        -> CronOperationFuture<'_, SchedulePage>;
}
```

The adapter scheduler feature alone exposes the target-binding contract.
It authorizes existing targets and maps persisted opaque payload; it cannot
install code, a Rust function/catalog or a second worker registry. Environment
conversion retains REQ-CRON-0019. Unsupported target fails before a store call.

```rust
pub struct ResolvedTarget {
    pub hook: sc_cron::HookTarget,
    pub payload: serde_json::Value,
}
pub trait ApplicationTargetBindings: Send + Sync {
    fn resolve(&self, target: &ExecutionTarget, payload: serde_json::Value)
        -> Result<ResolvedTarget, sc_command::OpError>;
}
```

C-1 freezes StoreOperationContext::with_deadline(&self, deadline: Instant)
-> StoreOperationContext: same shared admission/cancellation state, earliest
operation deadline, never extending G/K. C-6 implements the following concrete
constructor and CronExecutor; c-1 does not supply a stub body. Execute validates
CronRequest, derives the child context, resolves target/environment, calls one
native operation and converts its result/error into the canonical Envelope.

```rust
// sc-cron-commands[scheduler]; c-6 implements this and CronExecutor.
pub struct StandardCronExecutor<O: sc_cron::CronOperations + ?Sized,
                                B: ApplicationTargetBindings + ?Sized> {
    operations: std::sync::Arc<O>,
    bindings: std::sync::Arc<B>,
    context: sc_cron::StoreOperationContext,
}
impl<O: sc_cron::CronOperations + ?Sized,
     B: ApplicationTargetBindings + ?Sized> StandardCronExecutor<O, B> {
    pub fn new(operations: std::sync::Arc<O>, bindings: std::sync::Arc<B>,
               context: sc_cron::StoreOperationContext) -> Self;
}
```

### Test-only injection and honest final composition

C-1 owns canonical crates/sc-cron-commands/tests/support/operation_double.rs
implementing CronOperations with queued native replies/errors, and
tests/contracts/phase-c/executor_double.rs
implementing CronExecutor with queued Envelopes. Both are integration-test
modules, declared in the core/adapter manifests' allowed-test-double paths,
never production exports or normal module graphs. The operation module is
package-local so adapter tests remain importable after packaging, without a
second copy or new CI-copy gate.

```rust
// c-6 integration tests, relative to crates/sc-cron-commands/tests/:
#[path = "support/operation_double.rs"] mod operations;
// c-3 driver copies canonical executor source ONLY into the generated test
// venue tests/support/executor_double.rs, never generated src/:
#[path = "support/executor_double.rs"] mod executor;
```

C-1 adds operation_context_probe(deadline: Instant) -> StoreOperationContext
behind sc-cron's existing non-default test-utils feature, backed by real
context/admission state with no worker or SQL task. Only the adapter dev edge
enables test-utils. C-6 constructs StandardCronExecutor with that real probe,
RecordingOperations and fixture bindings; checks cover child context/deadline,
exact native arguments, one operation per verb, refusal/no call, fences and
errors. A separate normal-only consumer proves default graph restrictions;
dev feature unification cannot mask forbidden edges.

C-3 generic UI projections accept Arc<dyn CronExecutor> and call exactly
execute(request, deadline). Projection tests inject the canonical executor
double through a test-only local copy; no concrete Scheduler or c-6 factory
is referenced. C-3 does not claim selected cron production registration works
yet. C-4 owns the real selected template binding module and minimal root calls:
Arc<Scheduler<...>> implementing CronOperations, application target bindings,
and StandardCronExecutor feed c-3 projections. C-4 proves the selected build
and actual behavior before publication and repeats against registry artifacts.
Default-off builds remain functional at every handoff. No hidden sibling edge
or fake production executor is needed.


Generation adds `cron_commands` boolean default false, requires existing cron
when true, and preserves independently disabled MCP. c-1 owns prompt/schema/
answers/default/manifest declarations; c-3 owns generic CLI/HTTP/MCP projections
calling CronExecutor. C-6 never edits template sources. C-3 tests against the
c-1 executor double; c-4 owns selected root binding and real cross-UI parity,
checkout-patched before publication and no-patch after registry visibility.

## Proof and publication

C-1 explicitly enables rusqlite 0.39 bundled+hooks in the cron manifest
and commits the feature/lock baseline. C-3 later retains both features while
migrating ecosystem versions/path edges; c-5 owns no manifest/lock writes.
Primary 0.39 source confirms progress_handler is hooks-gated and accepts a
capturing context; get_interrupt_handle needs no extra feature. busy_handler
accepts only Option<fn(i32)->bool>, not a capturing closure. Selected strategy:
RAII calling-thread operation TLS for a static busy callback, capturing progress
hook and independently registered interrupt handle; cancellable acquisition
precedes callbacks. Do not replace that handler with busy_timeout or PRAGMA
busy_timeout. Restore the store's known baseline and TLS on all paths; reject
unsupported nesting, promise no restoration of unknown foreign callbacks and
use no unsafe FFI. C-1 qualifies these APIs before wave 2; c-5 proves held-lock,
busy/active-SQL and callback/restoration behavior.

c-5 causal real-clock tests cover held mutex and second stop, blocked initial
load/reload/read/write/error/end, busy/progress/decode, rollback/hooks,
no post-expiry writes/launches and physical joins. Test-only forced transition
may prove propagation but not elapsed bounds. Separate real-Instant budget
tests prove G/K; watchdogs detect failure without adding product allowance.
c-3 owns real isolated Windows Ctrl-C/Ctrl-Break and portable cleanup fixture
proof. c-6 owns boundary schema/handler tests against the contract double.
c-4 alone closes actual start/end/list across all three targets and selected
CLI/HTTP/MCP interfaces, minimal/default-off/Monty graphs and exact registry
artifacts. Command→cron→cron-commands order and command→runtime dependency
order are visible before each dependent. Unrelated crates stay unchanged,
rusqlite stays 0.39.x; no publication without existing authorization.
