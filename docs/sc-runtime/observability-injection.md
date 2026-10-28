# Injectable observability for generated projects

**Status:** Active contract (library wiring in progress)  
**Related:** [ADR-RUN-0004](../architecture.md), [REQ-RUN-0309](../requirements.md), [NFR-RT-0002](requirements.md)

## Goal

A project rendered from `template/` constructs **`sc-observability` 1.5.0** in
`main`, logs to the same on-disk layout as other SC consumers, and **injects**
that logger into workspace libraries (`sc-runtime`, `sc-cron`, …) so they emit
**`LogEvent`** records with canonical **`TargetCategory`** / **`ActionName`**
fields—not ad hoc `tracing` key/value pairs that never reach JSONL sinks.

Libraries MUST NOT call `LoggerBuilder` or depend on `sc-observability`. The
daemon owns construction; libraries accept an injected handle defined on
**`sc-observability-types`** (planned: a small `CanonicalLog` or equivalent
trait implemented by `sc_observability::v2::Logger` in the project only).

## Generated daemon responsibilities

Order is fixed ([REQ-RUN-0309](../requirements.md)):

1. Load config with `sc-config`.
2. Resolve **`log_root`**: explicit `logging.log_root`, else the transport
   **instance root** (same precedence as today in `template/crates/daemon`).
3. Build **`LoggerConfig::default_for(ServiceName::new(APP_NAME)?, log_root)`**
   and apply `LogSettings` from config (level, console/file sinks).
4. **`LoggerBuilder::build()`** → keep the `Logger` alive until `main` returns.
5. Pass a **shareable handle** into assembly:
   - **`Daemon::builder(...).logger(handle)`** (API pending on `sc-runtime`).
   - **`Scheduler::new(..., logger)`** when the template wires `sc-cron`
     (pending on `sc-cron`).
6. Run the daemon. Project routes and services may also take the same handle.

The template module `daemon::observability` centralises step 3–4 so generated
`main.rs` stays a thin orchestration layer.

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

## Boundary changes required

Before library code can accept injection:

1. **Amend ADR-RUN-0004** — allow **`sc-observability-types`** (not the logging
   runtime) on `sc-runtime`, `sc-transport`, and `sc-cron`; add the emit trait
   to **`sc-observability-types`** 1.5.x (or next patch).
2. Update **`REQ-RUN-0005`** forbidden/allowed edges and run `sc-lint-boundary`.
3. Resolve the open **`tracing` bridge** item in [requirements](../requirements.md):
   prefer explicit **`LogEvent`** emission via injection; drop library `tracing`
   once migrated.

Until step 1 ships, generated projects **construct and retain** the logger
(process-wide sinks work) but libraries continue to use `tracing` without
correlation into JSONL.

## Acceptance for template

- Rendered `config/default.json` enables **file** and **console** sinks.
- Rendered `main` resolves `log_root` from instance root when unset.
- Rendered `daemon::observability::Observability::init` documents the handle
  passed to future `.logger(...)` / scheduler wiring.
- No `sc-observability-otlp` in the template; OTel remains opt-in on the
  consumer `daemon` crate.
