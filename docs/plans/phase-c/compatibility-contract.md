# Phase C compatibility contract

Status: approved scope, planned implementation; readiness pending fresh scope,
critical, QA and actual anchor checks. This file is the single contract
specification consumed by c-1/c-3/c-5/c-6/c-4; existing anchor records and quotes
remain unchanged. User-approved new facts await the parent's separate capture
packet. External consumer evidence informs tests, not scope authority.

Critical R1 status: both findings are author-corrected pending independent
verification. The human explicitly approved the admission/outcome contract
in the preserved decision record: new commit admission stops at the cutoff,
physical ownership continues until completion, and late/indeterminate outcomes
remain accurate. No universal physical completion-by-2T guarantee is made.
Critical R2, QA and actual anchor gates still block implementation readiness.

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
pub struct ShutdownBudget { /* private checked absolute G/K, K >= G */ }
impl ShutdownBudget {
    pub fn new(grace: std::time::Instant, cleanup: std::time::Instant)
        -> Result<Self, CronError>;
    pub fn grace(&self) -> std::time::Instant;
    pub fn cleanup(&self) -> std::time::Instant;
}
#[derive(Clone)]
pub struct WorkerStopControl { /* independently callable; no join ownership */ }
impl WorkerStopControl {
    pub fn new() -> Result<Self,CronError>;
    pub fn finish_application_shutdown(&self) -> Result<(),CronError>;
    pub fn request_stop(&self, budget: ShutdownBudget) -> Result<(), CronError>;
    pub fn cancel_now(&self);
    pub fn operation_context(&self) -> StoreOperationContext;
    pub fn subscribe_status(&self) -> tokio::sync::watch::Receiver<ShutdownStatus>;
}
impl WorkerHandle {
    pub fn stop_control(&self) -> WorkerStopControl;
    pub async fn join_owned(self) -> Result<ShutdownReport, CronError>;
}
pub enum CommitAdmission { NotAttempted, Vetoed, Admitted }
pub enum PersistenceOutcome { NoMutationAttempted, Committed, RolledBack, Indeterminate }
pub struct TransactionOutcome {
    pub admission: CommitAdmission, pub persistence: PersistenceOutcome,
    pub completed_after_cutoff: bool,
}
pub struct ShutdownReport {
    pub grace_exceeded: bool, pub cleanup_exceeded: bool,
    pub cleanup: TransactionOutcome,
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
at most 1000; default 100. No new bundled table or advertised history via list_active.
Cancelled store operations use typed fixed cancellation/store errors with
stage/id/generation only; no SQL/provider prose. c-1 records new code inventory
without changing legacy row outcomes or fabricating successful cleanup.

## Supported defaults and consumer replacement

The framework charter leaves application stores, schemas and wiring to the
consumer. The built-in cooperative path below governs SqliteScheduleStore and
the standard Scheduler/StandardCronExecutor composition. SQLite commit_hook,
busy/progress callbacks and TLS are that provider's implementation, not a
requirement that every custom store use SQLite. A custom cooperative provider
must truthfully declare support, propagate the same live scope through every
operation, preserve fencing/query semantics and satisfy the documented
admission, outcome and physical-ownership contract using its own mechanisms.
Unsupported providers are rejected before work in this mode; wrapping a store
must forward support and scope rather than inherit an unsupported promise.

Consumers may instead use the existing legacy ScheduleStore/Scheduler path,
implement CronOperations for their application, or supply CronExecutor to the
public projection seam. These are supported replacement points for their own
schema and shutdown policy, not a claim that the built-in cooperative guarantee
holds for arbitrary code. Custom integrations own task admission/cancellation,
physical joins, generation/conflict handling, persistence outcome accuracy,
target authorization and deadline/error mapping. Document any different policy
explicitly; budget values alone are not arbitrary policy replacement. Standard
executor child contexts always retain the standard shared registry and earliest
deadline. Ordinary defensive validation and default safe behavior remain.

C-1 freezes this default/override guide without adding a plugin framework or
new API unless it identifies a precise missing public seam. C-5 extends the
existing cron README with a runnable custom ScheduleStore/schema example and
proves it in its existing isolated consumer venue: persist an application-owned
field, then schedule/query/end with the required generation semantics. The
example states whether it forwards cooperative support/scope/query or uses the
legacy caller-owned lifecycle; never certify every custom provider. Preserving
the bundled schema does not prohibit consumer-owned columns/tables, and does
not add arbitrary application fields to core Schedule or command DTOs.

C-6 extends its existing README with one selected-default recipe: explicit
generation options (cron plus cron_commands, optional Monty/MCP), working bundled
SQLite/store and target registration, explicit application worker startup,
start/end/list across selected UIs, and stop/join/outcome ownership. It links
the c-5 store example and explains how to replace CronOperations or CronExecutor
and which responsibilities transfer. No automatic job start, consumer-product
migration or private framework patching. C-4 consumes this exact documented
recipe in its existing generated-consumer proof before publication and again
against registry packages. C-5/c-6 prove their local examples independently;
the final real composition remains c-4's later-wave responsibility.

## Bounded default command intake

This completes the existing pre-submission admission/physical-ownership promise
of REQ-CRON-0020/0021 and ADR-RUN-0018 under NFR-RUN-0002's nonblocking request
path and the existing service-hardening production checklist section 7. It is
not a new user anchor, a row cap or a global scheduled-job in-flight lock:
ADR-RUN-0014's independent embedder row limits and ADR-RUN-0013's job semantics
remain unchanged.

The standard Scheduler-backed command path admits at most 64 context-aware
public start/end/query operation roots per Scheduler by default. One root
counts from admission, before registry insertion or blocking-pool submission,
through actual physical completion of all its owned store work. Nested store
calls share that root's permit; they neither acquire another nor bypass the
bound. All StandardCronExecutor instances, clones and CLI/HTTP/MCP projections
using that Scheduler share the same counter. Constructing another executor or
child context must not reset it. Every public context-aware operation enters
this gate, including direct calls outside the standard adapter.

C-1 freezes the narrow pre-start configuration method on Scheduler:
`with_command_capacity(self, capacity: u32) -> Result<Self, CronError>`.
Accept only 1..=65536; reject zero/out-of-range and attempts after any worker or
operation has started, before changing state. The default is 64, not unlimited.
The application owns selecting a finite override appropriate to its memory,
store and deployment; no new generator setting or universal tuning guarantee
is required. C-5 owns this same immutable per-Scheduler configuration and gate.

Admission atomically checks live stop/cancellation/deadline and the available
slot before creating registered/submitted work. Full capacity returns promptly
through existing CronError::Store/StoreErrorKind::Unavailable and canonical
namespaced unavailable mapping, with fixed value-free reason
`command_capacity_exhausted`, no mutation attempted and remediation to reduce
concurrency or retry a refused request later. No SQL/provider/payload values,
sleep/retry loop, waiter registry or hidden capacity-wait queue is created.
Cancelled/expired/stopping requests return their existing typed status instead
of entering work. Cancellation after admission never releases the slot merely
because the transport future was dropped: the core-owned physical completion
releases it exactly once, together with its tracked ownership. Stop closes
intake independently of capacity, then joins accepted work under existing G/K
and accurate persistence outcomes. Never automatically replay an admitted
uncertain mutation.

This bound counts public command roots, not already separately owned scheduled
job execution or worker initial-load/reload/bookkeeping operations. Those remain
subject to their existing worker dispatch and operation-ownership rules; they
must not be used as a fallback route for a saturated public command. Only
private worker/stop paths may classify internal operations. Restricted terminal
cleanup and stop control do not acquire a command slot, cannot launch new jobs
or accept ordinary commands, and remain joined, generation-fenced and K-limited.
The bound is not a claim that arbitrary application jobs/providers have bounded
concurrency or hard preemption. Consumers replacing CronOperations/CronExecutor
own their different finite intake/overload policy and must document it; custom
stores need not implement a semaphore or a SQLite-specific mechanism.

C-5 causal tests hold real store work, fill capacity and prove the next direct
operation is refused before registration/submission; drop admitted waiters and
verify slots remain occupied until physical completion. Exercise cancellation,
deadline, saturation-versus-stop races, and cleanup while full. C-6 maps the
fixed typed refusal without adding an independent queue/timeout policy. C-4's
existing real mixed-UI fixture shares one Scheduler across executors, exceeds
capacity under a held store, verifies bounded submitted work/refusal, issues
stop and physically joins accepted work. Configuration validation/default64
and valid finite overrides are regression cases; no new test venue or receipt.

## Admission, budget and physical ownership

In the built-in cooperative path, all worker/standard-command operations register before submission and blocking locks,
including initial load, reload, query, decode and every persistence path.
The live context reaches SQLite busy/progress/interrupt and row loops. Legacy
capability rejects cooperative admission before work; forwarding stores forward
both support and scope. Existing direct legacy calls remain caller-owned.

First stop closes new operation/launch admission. Already admitted ordinary
operations may drain to G. Restricted cleanup is a separate class only for
terminal bookkeeping of the stopped id/generation under existing fencing,
until K. It cannot create/restart/enable schedules or launch external work.
No new ordinary commit admission at/after G or cleanup commit admission at/
after K is permitted. No new cleanup transaction/retry begins after K.

Every explicit or implicit bundled SQLite write transaction in this mode installs the owned commit_hook.
Its final admission decision serializes briefly with live stop state, earliest
G/K, class/generation eligibility, cancellation and the same real Instant.
Stop-first vetoes; admission-first records its epoch/decision and may physically
complete later. Release the controller mutex before returning from the hook,
before COMMIT phases/native I/O. No SQL/reentrant connection use in the hook,
no controller mutex across COMMIT. Initial checks or earlier statement hooks
alone do not guard this final race. Second stop independently updates state,
wakes waiters and interrupts SQL even while an admitted commit is blocked.

Stop time is real std::time::Instant, independent of schedule Clock/ManualClock.
One monotonic phase/epoch/controller owns earliest G/K. Registration, shortening
and notification cannot lose races. Timer wakes recheck real time; callbacks
read the same latched state, not a stale snapshot. Caller budgets permit equal
G/K. T>0 uses G=s+T,K=s+2T once; second stop cancels immediately without extending
K. T=0 uses G=K=s: immediate cancellation and no new commit admission or cleanup
budget. A commit admitted earlier may finish; there is no hidden C0, guaranteed
instantaneous join or universal physical completion-by-2T promise.

Commit admission is a logical fence, not a physical fsync cutoff. SQLite may
complete an already admitted commit after G/K; busy/progress/interrupt/commit_hook
cannot universally preempt native commit/fsync. The operation remains owned.
Actual physical completion and persisted state determine TransactionOutcome:

| Observed case | Required outcome |
| --- | --- |
| No mutation attempted | NotAttempted + NoMutationAttempted |
| Commit vetoed, rollback explicitly confirmed | Vetoed + RolledBack |
| Admitted, confirmed committed before cutoff | Admitted + Committed, completed_after_cutoff=false |
| Admitted, confirmed committed late | Admitted + Committed, completed_after_cutoff=true; deadline failure remains |
| Explicit rollback confirmed after error | accurate admission plus RolledBack |
| Commit/rollback cannot be proved | accurate admission plus Indeterminate; never infer not persisted |

C-1 freezes fixed typed errors/status and report combinations. A bool false
cannot represent unknown or confirmed late persistence. Drop/expiry or
get_autocommit alone proves neither rollback nor the identity of a committed
mutation. Never automatically replay an uncertain create/restart/end/toggle;
generation fencing is not an exactly-once token. No new schema/idempotency
ledger is introduced in the bundled store to conceal uncertainty; application-owned
schema extensions remain permitted through a custom store.

StoreErrorKind gains fixed Cancelled, DeadlineExceeded and Indeterminate
classifications under its existing non-exhaustive policy. A planned
`StoreError::transaction_outcome(&self) -> Option<&TransactionOutcome>` accessor
exposes actual admission/persistence/deadline evidence without SQL/provider
prose. DeadlineExceeded may carry confirmed Committed; it is not permission
to retry. Indeterminate never becomes not-found/not-persisted. C-6 preserves
that value-free typed outcome in canonical error details and recommends only
separately authorized reconciliation, never automatic mutation replay.

Readback is only a bounded authorized reconciliation operation; it cannot
reopen an expired context. No remaining budget/proof means Indeterminate,
not extra time, blind replay or invented rollback. A later separately authorized
read can report observed id/generation/state without claiming causal identity.
Known hook/TLS/connection baseline is restored on proven safe paths; unsafe or
uncertain connection state remains owned/quarantined until cleanup completes.

One owner physically joins admitted closures/tasks after waiter cancellation.
Expiry publishes typed deadline status independently; it does not release
ownership or certify physical completion. No effects may occur after actual
owned release. Missed cutoff and confirmed late/indeterminate persistence are
reported accurately, never as successful bounded cleanup. Arbitrary Rust/native
self-spawned work or custom synchronous Monty hosts are not forcibly preempted.
The existing consumer's no-time-left lease-expiry fallback remains its own QA;
no uncertain release is reported successful.

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
endpoint fails between receipt and readiness. Do not retain a naked-port
reserve/drop takeover test: its unowned interval can race any local binder.
The continuously owned listener and immediate prebind closure-drop witnesses
prove the required ownership/order without that race. Pre-bind failure emits
no receipt and still preserves cleanup.

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

### Owned schema traits and feature baseline (c-1)

REQ-CRNC-0001, NFR-CRNC-0001 and ADR-RUN-0203's one-struct dual-schema
decision apply to the adapter-owned DTO family. C-1 supplies compileable sole
CronRequest/CronOperation and nested request/target/result DTO declarations,
including owner-local JsonSchema/ToSchema implementations, before c-3/c-6 run.
Generated api-types cannot implement foreign traits for these foreign DTOs;
it imports/re-exports the same types instead of introducing another wire model.

Freeze adapter `schema = ["dep:schemars", "dep:utoipa", "sc-command/schema"]`
and command `schema = ["dep:schemars", "dep:utoipa"]`, both independent optional
features. Dependencies are optional with defaults disabled unless required by the
qualified derives. Default features remain empty and do not select schema or
scheduler. Schema alone introduces neither cron, Tokio, SQLite, Monty, Axum,
RMCP nor command/server. Scheduler and schema remain independent; their
combination is explicitly tested. No schema dependency is added to CLI defaults.
C-1's existing compatibility probe records the exact schemars identity required
by the selected RMCP Parameters<T> bound and the same Utoipa identity used by
runtime/OpenApiRouter. It commits the adapter feature/manifest, allowed boundary
edges and actual trait implementations as its declaration baseline; c-3 retains
them during the later atomic ecosystem/version migration. This prerequisite is
owned contract work, not a dependency on c-6's executor implementation.

All public request/response DTOs exposed as schemas implement both traits under
schema, including nested variants and the canonical response wrapper/error
family. C-1 supplies sc-command-owned implementations on Envelope<T>, OpError
and ErrorKind; foreign leaf ErrorCode is described by its existing serialized
string contract inside those owner-local schemas, never by an orphan impl or
second error model. Envelope has custom serialization: its schema must manually
match the exact existing version/ok/data/error success/failure wire alternatives
and nullability, not derive the Rust enum internal variants. OpError retains
all five canonical wire fields and existing code/kind/detail/action shapes.
No runtime serialization or wire field changes are authorized. Both traits on
Envelope<CronResponse> compile under the same qualified identities as its
nested DTO/error family. The same strict versioned/tagged wire fields,
required/optional fields and target shapes must be represented; use owner-local
schema attributes/implementations where derives alone are insufficient. Duplicate
JSON keys and runtime-only validation remain parser/behavior checks rather than
false claims of schema expressiveness. C-1 compiles real Parameters<CronRequest>,
schema_for!(CronRequest) and Utoipa schema generation against the selected
identities in its existing probe, with the sole DTO supplied by the actual
adapter crate. A generic JSON/object schema or uncompiled JSON file cannot close
this contract. These proofs use declarations only, never a production executor
stub. The actual typed Envelope<CronResponse>/OpError response schema and its
serialized positive/negative fixtures must agree; generic object schemas,
copied envelope models or separately maintained adapter schema builders cannot
substitute for the canonical owner-local implementations.

C-3 activates schema only on selected daemon/service projection edges; shared
workspace and CLI dependency declarations remain defaults-only. Its real
RMCP Parameters<CronRequest>/typed Utoipa projections compile against c-1
declarations and the executor double; typed schema tests preserve the sole
versioned contract and disabled MCP. C-6 preserves/tests this schema surface
while implementing validation/executor behavior. C-4's existing real fixture
compares MCP tools/list input schemas and OpenAPI request/data schemas with the
same DTO's wire fixtures, then repeats packaged composition. Each selected
schema admits/rejects the specified structural examples; actual command checks
retain non-schema validation. No sibling-owned DTO/manifest edit or orphan impl
is needed, and isolated default CLI graphs remain lean.

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

C-1 retains operation_context_probe(deadline: Instant) -> StoreOperationContext
behind the existing non-default test-utils feature for focused test consumers.
C-6 instead uses the ordinary public WorkerStopControl::new/operation_context
factory inside a dev-only Tokio runtime, with no worker or SQL task.
C-6 constructs StandardCronExecutor with that real shared context,
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
no new post-cutoff commit admission/launches, accurate physical persistence
outcomes and no effects after owned physical release. Test-only forced transition
may prove propagation but not elapsed bounds. Separate real-Instant budget
tests prove G/K; watchdogs detect failure without adding product allowance.
At the real commit hook, pause after the last precheck and after admission's
allow decision, race stop-first/admission-first, shorten G/K and issue second
stop while COMMIT is paused without holding the controller mutex. Repeat
ordinary and cleanup, explicit and implicit writes; inspect actual rows and
generation from a separate connection after physical join. Preserve accurate
late/rollback/indeterminate outcomes. Injected lost-result tests prove reporting
only, not a reproduced physical fsync fault or universal completion bound.
c-3 owns real isolated Windows Ctrl-C/Ctrl-Break and portable cleanup fixture
proof. c-6 owns boundary schema/handler tests against the contract double.
c-4 alone closes actual start/end/list across all three targets and selected
CLI/HTTP/MCP interfaces, minimal/default-off/Monty graphs and exact registry
artifacts. Command→cron→cron-commands order and command→runtime dependency
order are visible before each dependent. Unrelated crates stay unchanged,
rusqlite stays 0.39.x; no publication without existing authorization.

### Declaration baseline wire inventory (C1)

The compileable sole DTO declarations live in sc-cron-commands::contract.
CronResponse uses `{result:"started"|"ended"|"listed",data:...}`. Started
contains ScheduleView; ended contains `{id,generation}`; listed contains
`{items,next_cursor}`. Safe ScheduleView fields are id, generation, name,
cron, target (kind only), enabled, state and nullable next_run_ms. Aggregate
request JSON is limited to 2MiB; core field/environment/source limits remain
binding. Semantic expression, positive generation/cursor, 1..=1000 page limit,
size and capability validation remain explicit executor checks. The strict
deserializer rejects top-level and nested struct duplicate/unknown fields and
unsupported versions without changing canonical Envelope serialization.
Schema compilation does not certify these runtime-only semantic checks.

### Application-owned native operation scope

`WorkerStopControl::new() -> Result<Self, CronError>` is an ordinary-feature,
worker-independent factory. Call it inside the owning Tokio runtime, which
must outlive stop notifications. It captures that runtime so `request_stop`
can be called from an ordinary synchronous thread. No Scheduler, framework
worker or `test-utils` is required. The returned context retains live shared
cancellation and earliest child deadlines. It can be passed to custom
`CronOperations` and the separately owned standard adapter constructor.

Factory-created controls do not admit, supervise or join arbitrary custom
provider work. The application closes intake, requests stop, physically joins
all provider tasks/native calls/cleanup, then calls
`finish_application_shutdown(&self) -> Result<(), CronError>`. This explicit
acknowledgement publishes `Joined`; it cannot verify untracked application
work. It refuses a missing stop, pending registered native work, or a control
obtained from `WorkerHandle::stop_control()`. Application acknowledgement revokes
existing and future contexts even before a future grace cutoff. Framework workers retain their
single physical owner `WorkerHandle::join_owned`. Acknowledgement never reopens
intake or revives a cancelled/expired context. Typed unavailable reasons are
`worker_runtime_unavailable`, `worker_already_running`, and
`operation_completion_owner_required`; stable top-level code remains STORE_FAILED.
