# Design correction proposal for COMPAT-SCOPE-001 / 002

Design assist only. Read scope-R1 FAIL report at reviewed commit `e67ad9aec145dcc7db0593cbb85d4ec9bba6ef97`, current compatibility contract/sprint docs, and cached rusqlite 0.39.0 primary source. This does not close either finding or constitute an independent recheck PASS. No product/plan/Git/Beads writes or Cargo builds.

## 001 — exact interfaces, test-only injection, honest integration cut

**Select the honest recut.** c-3 implements generic UI transport/projection preparation, runtime ecosystem migration, Ctrl-Break and fixture cleanup. It does not claim the new selected cron production binding already works without c-6. c-4 owns the final small selected-template binding and real composed prepublication/registry acceptance after consuming c-3/c-5/c-6. User-visible final behavior remains actual selected CLI/HTTP/MCP commands, default OFF; this is a scheduling correction, not a scope reduction or documentation-only substitute.

Use one canonical wire type named **CronRequest**; remove every reference to undefined CronCommandRequest. One versioned struct plus tagged operation enum represents the existing examples without competing DTO identities:

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

CronExecutor is a callable interface only; c-1 need not provide a fake production implementation. It has no sc-cron types and uses std Future/Pin, so the default contract does not enable Tokio, SQLite, Monty, Axum, RMCP or command/server. The passed deadline is the real monotonic request deadline from the frozen stop-clock contract, not a new relative timer or JSON field. DTOs and callable interface are usable without the optional scheduler implementation. A UI client implementation may send the request over the existing transport; server composition injects the real executor.

Freeze the following **core-native** operation port in `crates/sc-cron/src/consumer_contract.rs` (c-1 owns declaration/types and registry re-export). It imports no sc-cron-commands or sc-command envelope types; this prevents a new reverse edge/cycle. Query/page types are the already selected bounded active/ended/all stable-id protocol; EndSchedule/EndedSchedule are native typed structs, not wire DTO aliases.

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

c-5 implements CronOperations **for the real Scheduler** using the same admitted-operation registry/context, fenced end lookup and bounded query. No production mock/default success body implements this trait in c-1. Existing CRUD/Job/HookTarget APIs remain unchanged. The concrete implementation belongs in a c-5-owned module such as `crates/sc-cron/src/consumer_operations.rs`; that file can be introduced by c-5 without editing c-1's declaration file. c-1 may predeclare the module only if it has real buildable content; it need not reference a nonexistent module. Any later private module-registration edit in lib.rs is explicitly c-5 ownership after the c-1 baseline handoff, not a sibling write conflict.

Define the previously missing target-binding interface, behind adapter feature `scheduler`, in `crates/sc-cron-commands/src/scheduler_contract.rs`:

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

This hook authorizes/resolves an application-configured target before persistence. Rust resolves to the already configured Job and opaque persisted payload (identity mapping is valid for one Job); Python/Monty resolve to the existing HookTarget constructors under application policy. It does not install a dynamic function, add a handler catalog, read invoking-agent environment or sign/certify a host. Environment DTO→validated HookEnvironment conversion remains the executor's deterministic core conversion under REQ-CRON-0019; target resolution does not replace that policy. Return a fixed typed error for an unavailable target and write nothing. A selected generated application supplies its real binding implementation at c-4 composition; c-6 tests inject an explicit deterministic fixture binding.

Freeze a child request context constructor in c-1: `StoreOperationContext::with_deadline(&self, deadline: Instant) -> StoreOperationContext`, sharing stop/cancel/admission state and taking the earliest operation deadline, never extending G/K. The production executor constructor is c-6 code, not a c-1 stub:

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

`execute` validates CronRequest version/shape/limits, derives the child context, resolves target for start, converts into native core types, calls one corresponding CronOperations method, and converts the result/error to the canonical Envelope. It forwards exactly the child context and native expected generation; it does not wrap a concrete Scheduler or synthesize an independent timeout/ownership policy. Production construction supplies `Arc<Scheduler<...>>` after c-5 implements CronOperations. The adapter's optional dependency is `sc-cron` with defaults disabled; core never depends back on adapter.

### Exact test-double import and admission-context setup

c-1 owns canonical `tests/contracts/phase-c/operation_double.rs` and `tests/contracts/phase-c/executor_double.rs` with explicit allowed-test-double paths. Both files are **integration-test modules**, not exported from the library, and not included in production package module graphs:

```rust
// c-6 integration test; file compiled only in this test binary.
#[path = "../../../tests/contracts/phase-c/operation_double.rs"] mod operations;
use operations::RecordingOperations;
// RecordingOperations implements sc_cron::CronOperations, records context/
// native arguments, and returns explicitly queued typed replies/errors.
// It is not a Scheduler and needs no c-5 implementation.

// c-3 rendered projection integration test: generation driver copies the
// canonical executor_double.rs ONLY to generated tests/support/, then uses
// #[path = "support/executor_double.rs"] mod executor;
// RecordingExecutor implements only CronExecutor and queues typed Envelopes.
```

Since Rust cannot import another crate's `tests/` as a library module, adapter repository integration tests use the explicit relative path to the canonical support source (the shown three-parent path is from crates/sc-cron-commands/tests/). The generation validation driver copies executor_double.rs only into each isolated generated fixture's tests/support directory before compiling the projection integration test; it never adds a library module, public test API or support source to generated production src. c-1 owns canonical support definitions and the copy/import protocol; c-3 owns the fixture driver implementation in its existing validation venue. The plan must name these exact paths and ensure any crate-local package test that lacks repository tests/contracts carries its own test-only packaged fixture or excludes that repository-only test deliberately; do not assume arbitrary relative paths survive cargo packaging.

Test context creation must also be concrete: if c-1 supports an actual public standalone operation-root constructor, the tests use it and production worker/command composition still must obtain its shared context from WorkerStopControl, not construct an unrelated root. Otherwise c-1 adds the minimum probe to sc-cron's **existing non-default test-utils feature**, declared/re-exported only with that feature (for example operation_context_probe(deadline: Instant)), backed by the real context/admission state, with no worker/SQL task. Only an adapter dev-dependency enables it; default isolated normal-only graph checks cannot rely on dev-feature unification. Do not construct fake private state or require c-5 worker behavior just to instantiate c-6 tests. Select/document one of these already buildable mechanisms before the correction is considered complete. Neither approach exports the recording doubles from production. Context identity/deadline forwarding, one operation per verb, target refusal/no call, fences and error mapping are meaningful tests; these are not real Scheduler/SQL lifecycle PASS.

c-3 generic projection functions accept `Arc<dyn CronExecutor>` (or `&dyn CronExecutor` if lifetime local) and call exactly `executor.execute(request, deadline)`. Tests inject RecordingExecutor and assert JSON parity without touching c-6 implementation. A real production factory cannot exist before c-6, and c-1 must not supply a success/unsupported stub to pretend otherwise.

### Integration ownership recut

Move actual selected production root binding to **c-4**, since it consumes c-3's generic projection functions, c-5's Scheduler/CronOperations implementation and c-6's StandardCronExecutor. Add the corresponding source ownership and acceptance to c-4; remove those actual-binding criteria from c-3. New c-4 files may include `template/crates/daemon/src/cron_commands.rs` and the minimum existing root call sites `template/crates/daemon/src/{composition,lib,main,routes,mcp}.rs` and `template/crates/cli/src/{lib,main}.rs` as actually needed. Earlier-wave c-3 changes those existing files for its ecosystem work; c-4 explicitly takes the later composition handoff. c-6 never touches template sources. No concurrent sibling ownership, and no new serial edge is justified merely by a shared file.

c-3's generated project still builds for its existing ecosystem acceptance; it does **not** enable an unimplemented optional cron command registration or claim the final selected feature works yet. It supplies tested generic adapter functions and shared DTO identity. c-4 completes the selected configuration and must build/test the fully selected generated project **before** publishing, then repeat against registry artifacts. Default-off generation remains functional at every merge. This defers only honest composition closure to its actual owner and does not ship a final incomplete selected feature.

The graph remains c1→{c2,c3,c5,c6}, {c3,c5,c6}→c4, path3/width4/count6, now grounded in actual callable artifacts rather than concrete sibling calls. c-4's integration vertical rationale now includes small final template wiring; its old “release metadata only/no template source” prohibition must be amended explicitly. No hidden c3→c6→c5 dependency and no production stub.

## 002 — explicit SQLite prerequisite and sequential manifest handoff

c-1 must add `crates/sc-cron/Cargo.toml` to owned_paths and explicitly establish:

```toml
rusqlite = { version = "0.39", default-features = false,
             features = ["bundled", "hooks"] }
```

Its baseline lock update covers the initial adapter membership/contracts and this feature qualification; preserve rusqlite 0.39.x/libsqlite3-sys 0.37.x and existing SQLx compatibility. Feature `hooks=[]` itself introduces no new dependency package, but buildable baseline/feature qualification must be proved by the responsible implementation task. This design review did not build it.

**Sequential handoff, not removal:** c-1 owns this feature-only prerequisite and baseline lock first. After that baseline merges, c-3 retains owned_paths for the same cron manifest/Cargo.lock to perform the later ecosystem version/path alignment, preserving bundled+hooks. c-5 owns no shared manifest/lock writes and compiles against c-1's prerequisite. Both earlier/later owners are explicit in different waves; do not remove the manifest wholesale from c-3 or leave a conditional hoist decision after parallel execution begins.

Cached primary source under `/Users/randlee/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/rusqlite-0.39.0/` establishes:

- `Cargo.toml:110`: hooks is a declared feature; `src/lib.rs:126-127` gates the hooks module on it.
- `src/hooks/mod.rs:425-438`: progress_handler exists, accepts `FnMut() -> bool + Send + 'static`, returning true interrupts execution. It can capture the shared dynamic context; it does not run while a caller is blocked before entering SQLite.
- `src/lib.rs:1018-1019,1269-1287`: get_interrupt_handle and Send+Sync InterruptHandle::interrupt are available without an additional optional feature. Controller must register the handle without needing the held connection mutex; interrupt applies to currently executing SQL, not a queued/acquiring operation.
- `src/busy.rs:26,57`: busy_timeout and busy_handler are available without hooks; busy_handler takes **Option<fn(i32)->bool>**, not a capturing closure. It cannot directly capture Arc<StoreOperationContext>. Use the already required calling-thread operation scope/TLS to expose the current context to a static busy callback, with strict RAII restoration, or bounded statement-specific retry outside SQLite. Freeze one implementation strategy before coding; a closure example that captures context would not compile against this version.
- `src/busy.rs:15-24,46-54`: only one busy handler exists; setting busy_timeout/PRAGMA busy_timeout replaces a custom handler. Do not install a callback and then overwrite it with a timeout, or claim both remain active. Restore the store's known baseline explicitly on all paths. The APIs do not expose a general getter for arbitrary previous callbacks, so no promise to restore unknown foreign hooks is justified.

Recommended c-5 approach: operation-scope TLS holds the current cloned context only while executing on that same blocking thread; the static busy callback consults it and returns false on expiry/cancel, with any bounded wait woken by deadline shortening. The progress callback captures that context. An independently registered interrupt handle provides prompt active-SQL cancellation. Cancellation-aware connection acquisition happens before either SQL callback can help. No store connection lock held across await; no raw unsafe SQLite FFI required (the crate forbids unsafe). Test callback cleanup/TLS restoration, nested/forwarded adapter scopes or explicit rejection of unsupported nesting, no stale callback after unlock, and a real busy/active SQL/held-mutex cancellation matrix. Consumer connection owners must agree the known baseline or integrate their own scope correctly; this remains actual consumer adapter QA.

This proposal identifies the source-supported facilities and exact compile prerequisites. The author must amend the normative docs, then the assigned independent reviewer verifies the fix; these scratch recommendations do not close COMPAT-SCOPE-001/002.
