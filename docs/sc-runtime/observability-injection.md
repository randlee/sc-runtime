# Injectable observability for generated projects

**Status:** Phase B b-13 contract (implementation planned)
**Related:** [ADR-RUN-0003](../architecture.md), [ADR-RUN-0004](../architecture.md), [ADR-RUN-0007](../architecture.md), [ADR-RUN-0011](../architecture.md), [ADR-RUN-0014](../architecture.md), [ADR-RUN-0303](../architecture.md), [ADR-RT-0002](architecture.md), [ADR-TRN-0006](../sc-transport/architecture.md), [REQ-RUN-0309](../requirements.md), [NFR-RUN-0002](../requirements.md), [NFR-RUN-0010](../requirements.md), [NFR-RT-0002](requirements.md)

## Goal

A project rendered from `template/` uses the caret-compatible requirement
**`sc-observability` 1.5.1** (with exact `1.5.1` lock resolution) in
`main`, logs to the same on-disk layout as other SC consumers, and **injects**
that logger into workspace libraries (`sc-runtime`, `sc-cron`, …) so they emit
**`LogEvent`** records with canonical **`TargetCategory`** / **`ActionName`**
fields—not ad hoc `tracing` key/value pairs that never reach JSONL sinks.

Libraries MUST NOT call `LoggerBuilder` or depend on `sc-observability`. The
daemon owns construction; libraries accept this object-safe handle defined by
the matching published **`sc-observability-types`** patch:

```rust
pub trait CanonicalLog: Send + Sync {
    fn service_name(&self) -> &ServiceName;
    fn emit(&self, event: LogEvent) -> Result<(), v2::EventError>;
}

pub struct NoopCanonicalLog { /* service name only */ }
impl NoopCanonicalLog {
    pub fn new(service_name: ServiceName) -> Self;
    pub fn named_or_default(service: &'static str) -> Self;
}
pub type SharedCanonicalLog = std::sync::Arc<dyn CanonicalLog>;
```

`sc-observability` 1.5.1 implements the trait for `v2::Logger` by calling
`Logger::try_log`, or an equivalent operation guaranteed never to wait. It
must not call `Logger::log`, `enqueue_blocking`, or any blocking/backpressure
path. Queue-full and disconnected rejection returns an immediate
`EventError`; library callers apply the best-effort policy below.
This repository consumes those releases; it does not own their source or
publication. `Logger` is not assumed to be `Clone`.

`named_or_default` never exposes validation failure: the runtime and cron
legacy constructors use the fixed names `sc-runtime` and `sc-cron`, with the
upstream Noop identity as fallback. They never convert or unwrap an
application name. The real generated application identity appears only after
the daemon injects its production logger.

With the upstream `test-util` feature, `RecordingCanonicalLog::new` exposes
ordered cloned events through `events()` and `clear()`, while
`RejectingCanonicalLog::new` exposes `attempts()` and returns a fresh routing
`EventError` for every emit. Both are Arc-safe and recover poisoned test-only
locks without panicking. Library crates enable this feature only in
dev-dependencies.

## Generated daemon responsibilities

Order is fixed ([REQ-RUN-0309](../requirements.md)):

1. Load config with `sc-config`.
2. Resolve **`log_root`**: explicit `logging.log_root`, else the transport
   **instance root** (same precedence as today in `template/crates/daemon`).
3. Build **`LoggerConfig::default_for(ServiceName::new(APP_NAME)?, log_root)`**
   and apply `LogSettings` from config (level, console/file sinks).
4. **`LoggerBuilder::build()`** → store it in `Arc<Logger>` until `main`
   returns, and coerce an Arc clone to `SharedCanonicalLog`.
5. Pass a **shareable handle** into assembly:
   - **`Daemon::builder(...).logger(handle)`** in any builder state; it adds
   no typestate or run step.
   - **`Scheduler::open_with_logger(..., handle)`** when `cron` is enabled;
     that constructor creates
     **`SystemJitter::with_logger(handle.clone())`** internally.
6. Map generated cron config `0` or `null` to unlimited and positive values
   to `ScheduleLimits` at scheduler open; sc-cron never loads config.
7. Run the daemon and retain scheduler state for the same lifetime.
8. Drain/join runtime and scheduler workers, drop every shared handle, then
   consume the owner with `Observability::shutdown(self)`.

The template module `daemon::observability` centralises step 3–4 so generated
`main.rs` stays a thin orchestration layer.

For application authors, the normal interface is deliberately two calls:
`.logger(handle)` on the runtime builder and `Scheduler::open_with_logger`
when cron is enabled. Applications own construction, configuration, and
shutdown of the concrete logger; neither library imports application config
or requires the full observability runtime.

The cron contract also exposes `SqliteScheduleStore::open_with_logger` and
`supervise_with_logger` and propagates the handle through compose, host, and
run internals. `Scheduler::open_with_logger` is the only public logger-aware
scheduler construction path and installs one Arc into store, concrete
`SystemJitter`, and `RunCoord`; there is no public wrapper jitter or
post-construction `Scheduler::with_logger`. Existing `open`, jitter, and
`supervise` entry points remain source-compatible by delegating with the
fixed `sc-cron` Noop identity. Both supervision functions return
`Result<Captured, CronError>`.

Generated stderr errors are stable agent-facing strings in the established
`CODE cause — recovery` form:
`DAEMON.OBSERVABILITY cause — recovery`,
`DAEMON.CRON_OPEN cause — recovery`, and
`DAEMON.OBSERVABILITY_SHUTDOWN cause — recovery`. Bare `Display`/`Debug`,
`anyhow`, and `Box<dyn Error>` reporting is forbidden. A build/open/run error
remains primary if shutdown also fails; existing config, transport, and
runtime failures keep their exact code, cause, and recovery strings.
Shutdown becomes primary only after otherwise successful execution.

## Default logging layout (initial consumers)

Defaults match **`sc-observability`** built-in sinks:

| Setting | Generated default | Notes |
|--------|-------------------|--------|
| `logging.enableConsoleSink` | `true` | stderr JSONL-style events |
| `logging.enableFileSink` | `true` | durable logs under instance |
| `logging.log_root` | omitted | falls back to **instance root** |
| On-disk path | `<log_root>/logs/<service>.log.jsonl` | `DEFAULT_LOG_DIR_NAME` + service file name |

Override **`SC_LOG_ROOT`** only when `log_root` is empty in config (logger
crate behaviour). Instance root still comes from `sc_transport::instance_root`
or `daemon.transport.instance_root`.

Console-only deployments may set `enableFileSink: false` in config; the
**template default** keeps file logging on so new projects match production SC
consumers without extra steps.

## Library responsibilities (target)

| Crate | Today | Target |
|-------|--------|--------|
| `sc-runtime` | `tracing` spans in router | Injected logger; HTTP lifecycle as `LogEvent` |
| `sc-cron` | `tracing` with informal `subsystem`/`action` | Injected logger; stable `cron.*` targets |
| `sc-command` | `sc-observability-types` for errors only | unchanged wire types |
| `sc-config` / `sc-transport` | silent | optional inject later; not required for v1 |

Tests use **`Noop`** / in-memory fakes implementing the same types trait—no
`sc-observability` in dev-dependencies of library crates unless the test is an
integration test in the **daemon** crate.

Every library emission is best effort. An `EventError` must not replace or
modify an HTTP response, `RuntimeError`, `CronError`, `RunOutcome`, stop
decision, store result, or deadline. Queue-full and disconnected rejection
returns immediately under ADR-RUN-0007 and NFR-RUN-0002.
`NoopCanonicalLog` returns `Ok(())`.

## Canonical event mapping

Every library event sets `version` to `OBSERVATION_SCHEMA_VERSION`, timestamp
to `Timestamp::now_utc()`, service from the injected handle, default process
identity, and no trace or state transition. HTTP request IDs use the top-level
`CorrelationId` field; cron run IDs use top-level `correlation_id`. Stable
failure codes live only in `Diagnostic.code`, never in the arbitrary fields
map. Successful events and arbitrary application 5xx responses have no
diagnostic.

The shared fields map uses string values for method, path, schedule, target,
hook stage, and secret-free error detail; it uses non-negative integers for
status, `elapsed_ms`, `deadline_remaining_ms`, attempt, and pid. Outcomes are
the typed labels `success`, `client_error`, `server_error`, `timeout`,
`fenced`, `partial`, `stopped`, or `failed`. A diagnostic repeats the event
timestamp and supplies its stable code, summary/cause, and remediation.
Request and run IDs are not duplicated into fields.

Cron terminal events add the typed label `refused`. They map `RunOutcome`
variants directly: `Ok` to `success`, `Refused` to `refused`, `Failed` to
`failed`, and `TimedOut` to `timeout`. Consumer-provided refused/failed prose
is copied to `reason`/`error` fields and is never parsed into a diagnostic
code; internal resolver and join sites emit their separately fixed origin
diagnostics before returning the persisted outcome.

## Boundary changes required

Before library code can accept injection:

1. **Amend ADR-RUN-0004 and ADR-RUN-0007** — allow
   **`sc-observability-types`** (not the logging
   runtime) on `sc-runtime` and `sc-cron`; consume the matching published
   types/runtime **1.5.1** patches. `sc-transport` remains out of this cut.
2. Update **`REQ-RUN-0005`** forbidden/allowed edges and run `sc-lint-boundary`.
3. Amend ADR-RUN-0003/0303 and ADR-CMD-0001 so the library and generated
   feature graphs, and the command crate's types dependency, are consistent.
4. Resolve the open **`tracing` bridge** item in [requirements](../requirements.md):
   prefer explicit **`LogEvent`** emission via injection; drop library `tracing`
   once migrated.

The implementation sequence is [b-13 through b-17](../plans/phase-b/phase-b-plan.md#sprint-index).
The `tracing` bridge question is resolved in favor of direct canonical
emission. Shared manifest dependency cleanup may follow after the staged
migration; it is not required to change the event contract.

The repository public crates move to workspace version `0.1.1`; b-13 makes
`sc-cron` publishable but performs no publication. b-17 owns the existing
sc-publish consumer update, real installation and complete output sync from
the exact pinned kit revision, authorization/order proof, and registry-only
generated build.
The currently pinned kit is not a valid b-17 input because it installs an
upstream `docs/sprints/**` artifact. b-17 remains blocked until an exact later
revision excludes that class; its inventory gate must enumerate installed
destinations and reject any `docs/sprints/**` path.
This contract does not authorize or perform publication of either upstream
observability crate.

## Acceptance for template

- Rendered `config/default.json` enables **file** and **console** sinks.
- Rendered `main` resolves `log_root` from instance root when unset.
- Rendered `daemon::observability::Observability` retains `Arc<Logger>` and
  returns `SharedCanonicalLog` without cloning `Logger`.
- Rendered runtime and real scheduler receive the same handle; one HTTP event
  and `cron.scheduler/logger.attached` parse from the same JSONL file.
- Isolated CI sends real SIGINT and SIGTERM to separate generated daemons on
  temporary instance roots and proves bounded drain/join, cleanup, handle
  drop, and consuming logger shutdown. The dedicated `ubuntu-latest` job sets
  `SC_RUNTIME_GHA_EPHEMERAL_SIGNAL_TEST=1`; the test also requires
  `GITHUB_ACTIONS=true` and `RUNNER_ENVIRONMENT=github-hosted`, so the marker
  alone cannot enable it on a developer host. This is not a host-safe local
  test.
- No `sc-observability-otlp` in the template; OTel remains opt-in on the
  consumer `daemon` crate.
