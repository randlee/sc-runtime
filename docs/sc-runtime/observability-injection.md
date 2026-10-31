# Injectable observability for generated projects

**Status:** Initial release shim implemented in source under
[ADR-RUN-0017](../architecture.md#adr-run-0017-initial-release-observability-shim).
Release evidence remains subject to validation and authorized publication.
**Plan:** [Initial release observability shim](../plans/phase-b/initial-release-observability-shim.md).
**Cleanup:** [Issue 97](https://github.com/randlee/sc-runtime/issues/97), future
work outside the initial release epic.

## Initial 0.1.1 release contract

The generated application uses the real registry `sc-observability = "=1.5.0"`
and `sc-observability-types = "=1.5.0"`. One temporary unconditional
`sc_command::observability` module owns this public port:

```rust
pub trait CanonicalLog: Send + Sync {
    fn service_name(&self) -> &sc_observability_types::ServiceName;
    fn emit(&self, event: sc_observability_types::LogEvent)
        -> Result<(), sc_observability_types::v2::EventError>;
}
pub type SharedCanonicalLog = std::sync::Arc<dyn CanonicalLog>;
```

The same module supplies `NoopCanonicalLog::{new, named_or_default}` and
non-default std-only `test-util` recorder/rejector doubles. Runtime uses its
existing command dependency; cron temporarily uses command 0.1.1 with defaults
off/no features. Canonical values/errors remain direct actual registry types.
Command temporarily pins types `=1.5.0` to bound the inspected validator and
identity; runtime/cron retain compatible 1.5.0 requirements. These exact pins
intentionally prevent incompatible newer-types consumer selections until the
coordinated upstream migration. Command adds no dependency or server feature.

A private inline generated `LoggerAdapter` holds an Arc to the real v2 Logger,
forwards identity, and calls only `try_log(event)`. Ordinary native EventErrors
pass through unchanged. It adds no queue, sink or lifecycle policy. Build one
Logger, create one shared adapter, and clone that same handle into runtime and
cron. Library normal/build graphs must exclude full logger/OTLP directly,
aliased or transitively in all production feature combinations. Config and
transport have no types dependency. All sc-compose/ATM production exclusions
remain binding. No sixth crate is published.

The former local types 1.5.1 facade and development adapter 0.0.0 have been
removed, together with their paths/lock identities and generation rewrites.
The normal template now contains the application-only bridge directly.

## Explicit panic and admission limits

ADR-RUN-0017 makes two narrow policy amendments. Noop's fixed fallback
`ServiceName::new("noop")` may retain its invariant expect only under exact
inspected types 1.5.0 and fixed-literal validation tests. No caller/environment
value may reach it. This is an explicit exception to ADR-RUN-0006/NFR-RUN-0009/
NFR-CMD-0003, not a comment-only waiver or a broad no-panic claim.

Published 1.5.0 `try_log` can panic on poisoned internal runtime state. Catch
unwind locally around only that call using the justified AssertUnwindSafe
boundary; return fixed value-free `EventError::Routing` with code
`DAEMON.OBSERVABILITY_ADMISSION_PANIC`, cause `Logging admission panicked.`
and recovery `Restart the daemon to restore logging; the application operation
is unaffected.` Use infallible `ErrorCode::new_static`, `ErrorContext::new`,
`Remediation::recoverable` and an empty command list. Discard panic payload;
never include event/provider values or recursively log the failure.

This containment does not repair poison, catch panic=abort or suppress the
application-owned panic hook, which may print before unwinding. It catches no
arbitrary receiver/startup/shutdown callback and installs no global hook.
Healthy admission does not wait for queue capacity or perform sink I/O, but
internal locks may contend: it is not wait-free. Custom receivers remain
trusted prompt-return/no-panic boundaries. Native rejection and contained
unwind are best effort and never change business outcomes.

## Generated daemon responsibilities

Order is fixed ([REQ-RUN-0309](../requirements.md)):

1. Load config with `sc-config`.
2. Resolve **`log_root`**: explicit `logging.log_root`, else the transport
   **instance root** (same precedence as today in `template/crates/daemon`).
3. Build **`LoggerConfig::default_for(ServiceName::new(APP_NAME)?, log_root)`**
   and apply `LogSettings` from config (level, console/file sinks).
4. **`LoggerBuilder::build()`** → store it in `Arc<Logger>` until `main`
   returns. For initial release, derive the command-owned `SharedCanonicalLog`
   through the private inline adapter around an Arc clone. Direct upstream
   Arc coercion is future capability-proven cleanup.
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
run internals. Generic `Scheduler::new_with_logger(store, clock, jitter, handle)`
preserves application-supplied components and installs the handle in
`RunCoord` and worker/hook paths; those components own their internal logging.
`Scheduler::open_with_logger` constructs its logger-aware SQLite store and
`SystemJitter` and delegates generic construction, sharing one Arc throughout.
The generic constructor emits `logger.attached` once; the helper does not
duplicate it. There is no public wrapper jitter or post-construction setter.
Existing `new`, `open`, jitter, and
`supervise` entry points remain source-compatible by delegating with the
fixed `sc-cron` Noop identity. Both supervision functions return `Result<Captured, CronError>` and use the
same supervision implementation. Their legacy argument lists remain unchanged.
`PythonHook::new_with_logger(module, handle)` installs the handle before
invocation; legacy `PythonHook::new` installs the fixed `sc-cron` Noop.

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
| `sc-runtime` | Injected logger; HTTP lifecycle as `LogEvent` | command-owned port during initial release |
| `sc-cron` | Injected logger; stable `cron.*` targets | same command-owned port |
| `sc-command` | registry types for errors | temporary neutral port/Noop/test doubles; unchanged wire types |
| `sc-config` / `sc-transport` | silent | optional inject later; not required for v1 |

Tests use **`Noop`** / in-memory fakes implementing the same command-owned trait—no
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

The HTTP `path` field is the Axum matched-route template (for example
`/widgets/{id}`), never the raw URI, query string, or path-parameter value; an
unmatched route uses the fixed value `<unmatched>`. The shared fields map uses
string values for method, path, schedule, target, hook stage, and secret-free
error detail; it uses non-negative integers for
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

## Release enforcement and upstream cleanup

[ADR-RUN-0017](../architecture.md#adr-run-0017-initial-release-observability-shim)
and the [bounded implementation plan](../plans/phase-b/initial-release-observability-shim.md)
name exact paths, tests and boundary amendments. Expand the observability
identity guard to all five normal/build closures. Enforce only the temporary
cron-command edge and prove isolated cron default/Monty feature trees have no
command/server, axum or rmcp leakage. Retain all other negative boundaries.
Regenerate kit-owned preflight files through the qualified installer.

The initial release publishes the same five 0.1.1 SDKs in existing dependency
order after a single verified all-five Cargo dry run. Clean registry-only
`--release` generation/proof runs after all five publications, before accepting
the same-source `v0.1.1` tag. Human authorization, credentials and controlled-host
benchmark requirements remain. No publication is authorized by this plan.

[Issue 97](https://github.com/randlee/sc-runtime/issues/97) tracks cleanup outside
the initial release epic. Actual matching published 1.5.1 capability and direct
registry `Arc<v2::Logger>` to upstream SharedCanonicalLog proof trigger removal
of the local contract, inline adapter, exact pins and cron-command edge.
Registry indeterminacy fails readiness closed; a published but incompatible
pair blocks release for remediation. Working capability forbids another shim
release until cleanup restores the original cron boundary. A numeric version
alone proves nothing. A source-path re-export is allowed only if representative
consumer/custom-impl and upstream probes plus an explicit semver strategy
justify it; nominal old/new trait identity is not automatic. Use a coordinated
breaking SDK minor if necessary, never a permanent second contract.

Historically, the former sc-publish pin `16457e06ade6c0b003ff75addb7d694e9da41fe2` was
rejected because its actual installer/package destinations included upstream
`docs/sprints/**` history. The current pin, `9677ab083e7dd3feb258235a342e321ca1b45d8d`, is
the qualified and installed immutable revision. Its qualification enumerated all 85 actual
copy/render/package destinations, compared the complete inventory with real install outputs,
excluded `docs/sprints/**` destinations, and rejected traversal, absolute or duplicate
paths, unsafe symlinks, and package-unsafe paths. Upstream PR 137 remains draft and
unmerged; that status is distinct from qualification of the available immutable revision and
adds no merge prerequisite. Upstream source may retain its sprint history. Kit preparation
can proceed independently; final runtime composition still consumes approved parent merges.
This contract does not authorize or perform publication of either upstream observability
crate.

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

Cron invocation correlation uses the logical selected occurrence: numeric schedule
ID, generation and due timestamp form `cron-s<ID>-g<GEN>-t<SECONDS>-n<NANOSECONDS>`.
The private launch/lifetime state captures it before next-run advancement and
preserves it across Python supervision, Monty stop/tick feeds and CLI cleanup.
It is canonical top-level `correlation_id` only. It identifies occurrences within
one scheduler/store domain, not globally unique attempts. Non-run lifecycle,
reload, store, jitter and skipped pre-launch events have no correlation ID.
Legacy standalone Python/process entrypoints also retain `None`.
