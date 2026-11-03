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

```rust
// DTO-only crate; concrete tagged serde representation follows JSON above.
pub enum CronRequest { Start(StartRequest), End(EndRequest), List(ListRequest) }
pub enum CronResponse { Started(ScheduleView), Ended(EndResult), Listed(ListResult) }
// scheduler feature: application-injected target bindings, no global registry.
pub async fn execute<S,C,J>(scheduler: &Scheduler<S,C,J>,
    context: &StoreOperationContext, request: CronCommandRequest,
    bindings: &dyn ApplicationTargetBindings) -> sc_command::Envelope<CronResponse>;
```

Generation adds `cron_commands` boolean default false, requires existing cron
when true, and preserves independently disabled MCP. c-1 owns prompt/schema/
answers/default/manifest declarations; c-3 owns actual CLI/HTTP/MCP template
composition calling this handler. c-6 never edits those template sources.
c-3 tests against c-1 double; c-4 proves real cross-UI parity against c-5/c-6,
checkout-patched before publication and no-patch after registry visibility.

## Proof and publication

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
