# sc-cron Requirements

**Status:** Active

Active as of 2026-10-28. The contract is [architecture.md](architecture.md), [ADR-RUN-0013](../architecture.md), and [ADR-RUN-0016](../architecture.md). Sprints `b-3` through `b-17` implement these items.

# Requirement: Workspace member

**Status:** Active  
**Version:** 1.0.0  
**Created:** 2026-10-25  
**Last Updated:** 2026-11-12  
**Owner:** randlee  

---

## REQ-CRON-0001: Workspace member

**Status:** Active  

### Requirement Statement

Phase D retains cron policy and JSON-to/from-value adaptation; hooks receive values rather than JSON strings. `sc-sha` is permitted for SHA-256 cache keys. Here "name" means a Cargo or other manifest dependency. Documented wiring may reference the crate in comments and schema descriptions.


**MUST statements:**


- `crates/sc-cron` MUST be a workspace member.

- `sc-cron` MUST consume the single `sc-monty` implementation (the shared Monty interpreter crate).

- Every test function under `crates/sc-cron` before its migration onto `sc-monty` and `sc-storage` MUST still exist after that migration in `crates/sc-cron` or `crates/sc-monty`.

- Each assertion in those test functions MUST retain its expected value and condition after that migration.

- `cron.start` MUST use the `sc-command` `storage_write` function.

- `cron.end` MUST use the `sc-command` `storage_write` function.

- `cron.list` MUST use the `sc-command` `storage_read` function.

- Every `sc-observability-types` type used by `sc-cron`, including `LogEvent`, `Level` and `ServiceName`, MUST be imported directly from the registry `sc-observability-types` crate.

- Crates used only by tests MUST be declared under `[dev-dependencies]`.

- The unpublished `dev/observability-types-shim` package and its path/rewrite plumbing MUST be removed during implementation before initial publication.



**MUST NOT statements:**


- `sc-cron` MUST NOT directly depend on `monty`.

- `sc-cron` MUST NOT directly depend on `monty-types`.

- `sc-cron` MUST NOT retain an independent Monty interpreter engine for compile, session, host, limits, dump or load operations.

- `sc-cron-commands` MUST NOT enable the `sc-command/server` feature.

- `sc-cron-commands` MUST NOT enable any other dependency feature that adds the `axum` or `rmcp` crate to its dependency graph.

- `sc-runtime` MUST NOT name `sc-cron` as a dependency.
- `template/crates/daemon/Cargo.toml` MUST NOT name `sc-cron` except as the optional `cron` dependency selected by the embedding application.

- The default feature set of the generated daemon MUST NOT link `sc-cron`.

- The default feature set of `sc-cron` MUST NOT enable `sc-command/server`.

- The default feature set of `sc-cron` MUST NOT enable `sc-command/test-util`.

### Rationale

An independently packaged scheduler can be embedded without the generated template. Its sc-command dependency supplies the retained observability port, while sc-storage and sc-monty supply shared storage and interpreter behavior.

### Success Criteria

**Acceptance Criteria:**

- [ ] Isolated default/Monty consumers must prove no axum/rmcp/server leakage; workspace feature unification is not that proof.

## REQ-CRON-0002: Normal crontab strings

**Status:** Active  
**Version:** 1.0.0  
**Created:** 2026-10-25  
**Last Updated:** 2026-11-12
**Owner:** randlee  

### Requirement Statement

**MUST statements:**


- A schedule MUST use a normal crontab string of five fields: minute, hour, day of month, month, and day of week.
### Rationale

The beads-cache expression parser refused a five-field string; a normal crontab string is five fields, minute first (ADR-RUN-0013 Context).
### Success Criteria

**Acceptance Criteria:**

- [ ] `*/5 * * * *` means every fifth minute.

# Requirement: Rust, Python, or Monty hook

**Status:** Active  
**Version:** 1.0.0  
**Created:** 2026-10-25  
**Last Updated:** 2026-11-12  
**Owner:** randlee  

---

## REQ-CRON-0003: Rust, Python, or Monty hook

**Status:** Active  

### Requirement Statement

All Monty execution calls sc-monty. Retain the existing cron host adapters, capability behavior and unit/integration assertions as thin value-level adapters, not another interpreter host. A schedule supplies Monty or Python source plus JSON data, which the embedding application can enrich in the shared cron.start transform before persistence. cron.start and cron.end use storage_write with policy-checked CronStorage actions; cron.list uses storage_read. Internal scheduler lifecycle operations retain domain transactions because they are not user commands.

sc-cron is a general-purpose scheduler for application-defined jobs. A schedule row names an in-process `Job`, a Python module, or `Monty { source }` under optional feature `monty`. Python and Monty receive the `sc-cron.hook.v1` object with `protocol`, `event`, `schedule`, `payload`, and `deadline`. It has no consumer-specific beads, bd-cache, or ATM field. The crate exposes no HTTP route, MCP tool, or binary. The consumer owns that API and configures schedule limits; sc-cron does not read sc-config.

Monty hosts are application-owned registrations in an immutable `MontyHostRegistry` installed with `Scheduler::with_monty_hosts`. The empty registry registers no application call. `MontyHostHandler: Send + Sync` accepts a `MontyHostCall` containing its validated `MontyHostName`, positional/keyword JSON arguments, schedule name, hook phase, and row deadline and returns `Result<serde_json::Value, CronError>`; matching closures are supported. `MontyHostName::new` validates the identifier and `as_str()` borrows it; registry keys use the same type while registration still accepts strings. Registration rejects invalid identifiers, duplicate names, and reserved `__hook`, `cron_stop`, and `cron_tick` bindings with `InvalidSpec`. Source can invoke registered functions but cannot install or replace handlers. An unregistered name fails with `HOST_UNDEFINED`. Bridge argument conversion and supplied CLI adapter shape rejection fail with `HOST_DENIED`; a custom handler's returned error, including `CronError::InvalidSpec`, records `HOST_CALL_FAILED` after configured-value redaction. Private typed dispatch preserves supplied-adapter errors without parsing custom error prose. The host checks the row deadline before and after a synchronous handler, but cannot preempt arbitrary application code.

Phase B supplies opt-in `sc_compose_run(subcommand, args)` and `atm_run(subcommand, args)` adapters through this same registry. `ScComposeHostConfig` and `AtmHostConfig` select non-empty executable paths (defaults `sc-compose` and `atm`) and working directories (default the daemon directory captured at configuration). Each adapter accepts one command/subcommand string and an optional list of string arguments, with no keyword arguments. It forwards the non-empty command/subcommand followed by those ordered arguments directly as argv, never through a shell. Every command/subcommand is supported; flags including `--root` are forwarded unchanged. Neither adapter forces render arguments or temp-root staging. Scripts choose arguments, not the configured executable or working directory. Both adapters share the existing blocking child driver in `monty_compose`, generalized for argv/cwd, with row deadline, process-group termination, and `MAX_HOOK_OUTPUT_BYTES` per stream. They do not call the async `process::supervise` from the blocking feed thread or introduce a runtime/`block_on` bridge. A completed child returns `{stdout, stderr, exit_code}`, including nonzero exits; script code decides how that result affects its hook. A deadline expiration records `TimedOut`; spawn, decoding, and capture-limit failures record stable host errors. The adapters receive the resolved environment snapshot from ADR-RUN-0016. Neither CLI is required when its adapter is unused. `render_template` is not a product requirement; an internal helper may remain an implementation convenience without becoming a mandatory registered host function.

MVP accepts application-supplied Monty source without signatures, a separate script-approval mechanism, or a consumer option enabling such a mechanism. Capability registration and environment policy are enforced independently. In-process Monty and custom Rust handlers retain stack-overflow, allocator-abort, panic, and non-cooperative deadline risks; they are not an isolation boundary for hostile code.

The stop/tick ordering, generation fencing, retained REPL, interpreter-checkpoint VM limits, and joined shutdown behavior remain as specified by ADR-RUN-0013. Host calls use the production `drive_feed` resume loop; the cfg(test) helper is not a separate production path.

### Rationale

Application-supplied hooks and value adapters let the reusable scheduler execute registered application behavior without implementing another interpreter host.

### Success Criteria

**Acceptance Criteria:**

- [ ] Host registry integration tests invoke custom, sc-compose, and ATM functions through one extension interface.
- [ ] Argument-forwarding tests cover render and non-render subcommands.
- [ ] MVP execution requires neither signatures nor approval configuration.

# Requirement: Application schedule storage and compiled-session cache

**Status:** Active  
**Version:** 1.0.0  
**Created:** 2026-10-25  
**Last Updated:** 2026-11-12  
**Owner:** randlee  

---

## REQ-CRON-0004: Application schedule storage and compiled-session cache

**Status:** Active  

### Requirement Statement

sc-cron owns CronRow for schedules and MontyBlobRow for compiled hook sessions. Generic storage engines contain no cron-specific schema or SQL.

**MUST statements:**

- CronRow (the cron-owned schedule row) MUST be stored through Storage<CronRow> in the application database, using the application SQLite file by default.
- The schedule row MUST preserve canonical hook_environment data (the JSON document specifying the Python/Monty hook environment) with empty default {"dotenv":[],"variables":{}}.
- The configured schedule-row limit MUST count enabled and disabled rows.
- CronStorage MUST reject a new schedule row at the configured limit with CronError::InvalidSpec.
- Each writer MUST use the embedding application's ScheduleLimits (the configuration that supplies the schedule-row limit).
- Schedule writes MUST reject hook source that sc-monty (the shared Monty interpreter crate) cannot compile before the schedule's first tick.
- Compiled sessions MUST be captured immediately after compilation, before any hook runs.
- Compiled-session blobs (serialized bytes of compiled Monty hook sessions stored through `Storage<MontyBlobRow>`) MUST use the SHA-256 digest of the exact hook source computed by the sc-sha crate plus the Monty session dump version (Monty's dump format version) as their key.
- The schedule-row source MUST remain authoritative for matching a cached blob.
- Missing, stale, corrupt, unreadable-version or missing-hook cache entries (stored compiled-session blobs) MUST trigger recompilation from source and replacement of the pre-run blob.
- Cache rows (MontyBlobRow records for compiled hook sessions) MUST use sc-cron-owned generic row schemas (table, key and column descriptors).

- Cache rows (MontyBlobRow records for compiled hook sessions) MUST use sc-cron-owned migrations.

**MUST NOT statements:**

- sc-cron MUST NOT create a separate cron database by default.
- sc-cron MUST NOT persist hook run state in compiled-session blobs.
- The configuration schema MUST NOT add a separate environment or secret table.
- sc-cron MUST NOT migrate unrelated consumer tables (application tables other than the cron-owned rows).

### Rationale

Schedules and compiled sessions use cron-owned rows in the application database. Reloading schedules loads every row into memory; an uncapped row count is unsafe when schedules come from operators or configuration, and sc-cron must not depend on `sc-config` (ADR-RUN-0014).

### Success Criteria

The ported cron conformance tests preserve schedule fields and row-limit behavior. Compiled-cache tests cover source/version match, invalid source at submit, pre-run snapshot timing and each cache-miss condition.

## REQ-CRON-0005: One error enum

**Status:** Active  
**Version:** 1.0.0  
**Created:** 2026-10-25  
**Last Updated:** 2026-10-25  
**Owner:** randlee  

### Requirement Statement

Phase C planned amendment: cooperative operation failures remain CronError::Store
with non-exhaustive StoreErrorKind additions Cancelled, DeadlineExceeded and
Indeterminate, and optional typed TransactionOutcome accessor. No second error
enum/channel or raw SQL/provider prose is added. DeadlineExceeded may carry
confirmed committed-late evidence and must not trigger automatic retry;
Indeterminate never asserts rollback/not-persisted. Existing legacy variants,
stable top-level codes and row outcomes remain unchanged. REQ-CRON-0020 and
ADR-RUN-0018 govern the exact admission/persistence report policy.

Fallible library entry points return `CronError`, including both
`process::supervise` and `process::supervise_with_logger`, which return
`Result<Captured, CronError>`. The variants are `BadExpression`, `InvalidSpec`, `AlreadyActive`, `NotFound`, `HookFailed`, `Store`, and `Fenced`, with codes `BAD_EXPRESSION`, `INVALID_SPEC`, `ALREADY_ACTIVE`, `NOT_FOUND`, `HOOK_FAILED`, `STORE_FAILED`, and `FENCED`.

Application-implemented callback protocols retain their documented results.
`EnvironmentReferenceResolver::resolve` returns the value-free
`EnvironmentReferenceError::{Denied, Unavailable}` protocol; sc-cron maps
those failures or a missing reference to fixed `ENV_REFERENCE` outcomes.
Provider prose is never a second public library error channel.

With feature `monty`, a feed may record `RunOutcome::Failed` or `RunOutcome::TimedOut` instead of returning `CronError`. Stable code tokens appear in `Failed` detail strings as `CODE: message; recovery`. [`RunOutcome::failure_parts`](../../crates/sc-cron/src/run_failure.rs) and [`RunFailureParts`](../../crates/sc-cron/src/run_failure.rs) parse that form for agents and UIs. Host-adapter errors remain internal and do not add a public CronError variant. The inventory matches [architecture.md](architecture.md):

| Condition | Code | `RunOutcome` | Operator recovery |
|---|---|---|---|
| Bridge argument conversion or supplied CLI adapter shape rejected before execution | `HOST_DENIED` | `Failed` | Fix the call's JSON conversion or the supplied adapter's documented argument shape. |
| Custom application handler returns an error, including `InvalidSpec` | `HOST_CALL_FAILED` | `Failed` | Fix the application handler; configured environment values are redacted from the recorded detail. |
| CLI executable missing or not runnable | `HOST_SPAWN` | `Failed` | Fix the configured executable path or install the external CLI. |
| CLI output is invalid UTF-8 or exceeds the per-stream cap | `HOST_OUTPUT` | `Failed` | Return bounded UTF-8 stdout and stderr. |
| CLI exits nonzero | — | Script receives result | Inspect `exit_code` and bounded `stderr`; the adapter does not turn an application CLI result into an automatic hook failure. |
| Undefined name | `HOST_UNDEFINED` | `Failed` | Fix hook source; that name is not defined. |
| Monty `OsCall` other than `Getenv` or `GetEnviron` | `HOST_OS` | `Failed` — detail names the operation | Use the configured host registry and Getenv/GetEnviron; other OS operations remain denied. |
| Allowlisted daemon variable is not Unicode or exceeds the value cap | `ENV_PROCESS_INVALID` | `Failed` — value-free detail | Restart the daemon with a bounded UTF-8 value or remove the name from the allowlist. |
| `.env` path outside daemon policy | `ENV_FILE_DENIED` | `Failed` — value-free detail | Authorize the canonical file/root or remove the schedule source. |
| Required `.env` file missing | `ENV_FILE_MISSING` | `Failed` — value-free detail | Restore the absolute file or mark the source optional. |
| `.env` file invalid | `ENV_FILE_INVALID` | `Failed` — value-free detail | Fix the bounded data-file syntax. Shell expansion is not supported. |
| Reference missing or resolver failed | `ENV_REFERENCE` | `Failed` — value-free detail | Configure the daemon resolver and restore the referenced value. |
| Resolved entry-count or encoded-block limit exceeded | `ENV_SNAPSHOT_LIMIT` | `Failed` — value-free detail | Remove variables or reduce values before retrying. |
| Monty raises during the feed | `HOST_RAISED` | `Failed` | The detail is the exception message. |
| `cron_stop` return value is not a valid stop reply | `HOST_REPLY` | `Failed` — detail is `stop check: HOST_REPLY:` then the parse error | Fix hook source; return an object with string `"action"` (`continue`, `disable`, or `delete`) and a non-empty `"reason"` when disabling or deleting. |
| Monty feed task join failed (cancelled blocking-pool task) | `FEED_TASK_FAILED` | `Failed` | Retry the run; if it persists, inspect blocking-pool capacity and hook source. |
| Monty feed task panicked (worker join) | `FEED_JOIN_FAILED` | `Failed` | Fix the panic in the hook or host and retry the schedule. |
| Monty due task panicked or was cancelled | `DUE_JOIN_FAILED` | `Failed` | Retry the schedule; if it persists, inspect the Monty due task and hook source. |
| Monty due path used for a non-Monty hook | `MONTY_RESOLVE` | `Failed` | Run this row on its Rust or Python target, or set `HookTarget::Monty`. |
| `cron_tick` returned a stop action (disable/delete) | `MONTY_TICK` | `Failed` | Return normally from `cron_tick`; disable and delete belong in `cron_stop`. |
| After the CLI returns, host `Instant` exceeds the row budget |  | `TimedOut`, session dropped | Reduce CLI work or raise the row deadline; the VM clock was paused during the host call. |
| Monty feed-time-limit while the VM is executing (not during the host suspend) |  | `TimedOut` | Interpreter checkpoints govern VM execution; host handlers use the row deadline. |

Host-deadline paths record bare `RunOutcome::TimedOut` with no code token in the stored outcome.
### Rationale

Every cross-boundary failure exposes stable codes, causes, and recovery/remediation; agents must be able to act on failures without reading source (framework charter). The variant set and codes match the beads-cache crate that sc-cron ports.
### Success Criteria

**Acceptance Criteria:**

- [ ] The public error type in crates/sc-cron is CronError.

## REQ-CRON-0019: Explicit Python and Monty hook environment

**Status:** Active  
**Version:** 1.0.0  
**Created:** 2026-10-28  
**Last Updated:** 2026-11-02  
**Owner:** randlee  

### Requirement Statement

Python and Monty schedule rows share one `HookEnvironment`; existing
constructors default it to empty. A daemon-scoped `HookEnvironmentHost` names
which daemon-process variables may be inherited, authorizes absolute `.env`
files or canonical roots, and supplies an optional reference resolver. The
generated administrator config exposes empty-by-default `allow_process`,
`allow_dotenv_files`, and `allow_dotenv_roots` arrays. Every
host default is empty. The crate never infers or captures an invoking caller's
environment. The generated daemon passes a prebuilt host policy into scheduler
composition so an application-provided resolver is preserved.

Reference resolution receives the schedule name, variable name, opaque
reference, and row deadline. The provider returns only `Denied`, `Unavailable`,
missing, or a value; sc-cron emits no provider prose. Resolution is bounded by
the row deadline, and expiration is `RunOutcome::TimedOut` before hook source
runs.

The schedule environment contains ordered absolute `.env` sources followed
by name-keyed `Set`, `Reference`, or `Clear` operations. One immutable map is
resolved before `cron.stop` and reused for `cron.tick`: allowlisted daemon
values, then files in declaration order, then schedule operations. Explicit
set values are canonical plaintext schedule data. Daemon, file, and reference
values are never persisted. A non-empty environment on a Rust row is invalid;
trusted Rust jobs keep their existing `Job` interface.

Reference locators are non-secret plaintext registry metadata; only their
resolved values remain external. Every intermediate/final map has at most 128
entries, every stored/resolved value is at most 16 KiB, and the encoded final
block is at most 32,000 UTF-8 bytes. Per-source oversize failures and aggregate
`ENV_SNAPSHOT_LIMIT` failures are value-free.

Python starts with exactly the resolved map rather than the daemon's complete
environment. Monty source initialization and hook feeds answer `Getenv` and
`GetEnviron` from the same map with ordinary absent/default behavior; every
other OS call remains denied. Both sc_compose_run and atm_run children receive that map
and do not inherit the daemon environment. sc-cron never mutates the daemon
process environment. Full semantics, limits, file authorization, and the
release-v1 schema baseline are [ADR-RUN-0016](../architecture.md).

Every hook/provider-controlled failure detail or stop reason has each distinct
non-empty resolved value replaced with `[REDACTED_ENV]` before byte capping and
before fixed codes/recovery, outcomes, events, traces, or persisted reasons are
assembled. Stable text, metadata, and the canonical environment configuration
write are exempt; deliberate set values remain plaintext schedule data. Python
output, Monty exception/return text, and both CLI adapters' stdout/stderr are included. Trusted
direct callers may read raw public `HookOutput` fields, but `HookOutput` debug
formatting omits their contents. This exact-value replacement mitigates
accidental propagation; trusted hooks can transform or exfiltrate values and
remain outside an untrusted-code boundary.
### Rationale

Owner (ANC-034): "python/monty having consistent settings would be least confusing" (ANC-037): "Never capture or persist inherited host values accidentally" ADR-RUN-0016: environment variables are not uniformly secrets. Durable values such as `ATM_IDENTITY` and `ATM_TEAM` belong with a schedule. API keys may instead be daemon-wide inherited values, explicit schedule values, references resolved by an application secret provider, or values loaded from an authorized absolute `.env` file.
### Success Criteria

**Acceptance Criteria:**

- [ ] The same environment cases produce equivalent Python and Monty results.
- [ ] An allowed daemon value is delivered and an unlisted value is absent.
- [ ] Two schedules set distinct identities and clearing removes a prior value.
- [ ] Ambient canaries are absent from persisted rows without explicit set input.
- [ ] A consumer resolver supplies a referenced value without persisting the result.
- [ ] An authorized absolute dotenv file supplies hook values.
- [ ] A top-level getenv reads the configured snapshot before cron_stop or cron_tick.

## NFR-CRON-0001: Allowed third-party dependencies

**Version:** 1.0.0  
**Created:** 2026-10-27  
**Last Updated:** 2026-11-12
**Owner:** randlee  
**Status:** Active  

### Requirement Statement

This item owns the third-party crates `sc-cron` may depend on. `sc-lint-boundary` accepts only workspace package names. The cargo-tree gates `sc-cron-tree-default` and `sc-cron-tree-monty` in `sc-lint.toml` enforce the optional Monty feature. CLI process invocation does not create a crate dependency.


**MUST statements:**


- The normal dependencies of `sc-cron` MUST be `async-trait`, `chrono`, `cron`, `getrandom`, `process-wrap`, `sc-observability-types`, `serde`, `serde_json`, `tempfile`, `tokio`, `tracing`, `sc-storage`, `sc-command`, `sc-monty` and `sc-sha`, subject to the feature conditions in this record.

- The `monty` feature MUST remain optional.

- The default dependency tree MUST have no `sc-monty` edge.

- The `monty` feature MUST use `sc-monty` for the interpreter.



**MUST NOT statements:**

- The resolved `sc-cron` normal dependency graph with default Cargo features MUST NOT contain `monty` (the published interpreter crate).


- `sc-cron` MUST NOT depend directly on `rusqlite` after its schedule store moves to `Storage<CronRow>`.

- `sc-cron` MUST NOT depend directly on `monty`.

- `sc-cron` MUST NOT depend directly on `monty-types`.

- `sc-cron` MUST NOT link `monty-pool`.

- `sc-cron` MUST NOT link `monty-fs`.

- `sc-cron` MUST NOT link `monty-runtime`.

- Production dependencies MUST NOT include a crate from the sc-compose repository other than `sc-sha`.

- Imports in non-test, non-benchmark production code MUST NOT include a crate from the sc-compose repository (including `sc-compose`, `sc-composer` and `sc-composer-beads`) other than `sc-sha`.

- Production dependencies MUST NOT include an agent-team-mail workspace crate (including `agent-team-mail` and `atm-core`).

- Imports in non-test, non-benchmark production code MUST NOT include an agent-team-mail workspace crate (including `agent-team-mail` and `atm-core`).

### Rationale

Test and benchmark references to sc-compose and ATM are permitted. `monty` runs in-process when the host controls the code; `monty-pool` runs the interpreter in worker subprocesses for untrusted code. The Monty hook target is the trusted case, and a subprocess pool would spend the process spawn the target exists to avoid. A consumer who does not enable `monty` does not link the interpreter.
### Success Criteria

**Acceptance Criteria:**

- [ ] The `sc-cron-tree-default` and `sc-cron-tree-monty` cargo-tree gates in `sc-lint.toml` enforce that feature `monty` stays optional: the default tree has no `sc-monty` edge, and `--features monty` uses the shared interpreter through `sc-monty`.
### Related Documents

**Requirements:**

- [REQ-RUN-0005](../requirements.md) — point 4 points here

## NFR-CRON-0002: Environment isolation and redaction

**Status:** Active  
**Version:** 1.0.0  
**Created:** 2026-10-28  
**Last Updated:** 2026-10-28  
**Owner:** randlee  

### Requirement Statement

Environment names use `[A-Za-z_][A-Za-z0-9_]*`. Construction, persistence,
row decode, and execution enforce 256 bytes per name, 16 `.env` sources, 128 entries, 16 KiB per stored/resolved
value, 4 KiB per reference locator/path, a 32,000-byte encoded final block, a
256 KiB canonical document, and a 1 MiB dotenv file. Names,
paths, values, and references are UTF-8 and NUL-free; ASCII case-fold name
collisions are rejected on every platform. An allowlisted non-Unicode daemon
value fails as `ENV_PROCESS_INVALID`. `.env` files are bounded UTF-8 data
files: no interpolation, command substitution, duplicate names, or NUL.
Canonical path authorization follows symlinks before checking an exact
file/root policy; authorization precedes missing classification. Lookup and
enumeration preserve spelling on Unix; Windows runtime keys are uppercased for
both Python and Monty and lookup is case-insensitive, while stored config keeps
the original spelling. Python, sc-compose, and ATM executables are resolved before
child `PATH` is cleared: basenames use daemon launch `PATH`/Windows `PATHEXT`,
absolute paths are retained, and relative paths with components use the
daemon current directory captured when hook/host configuration is built.

Set values, resolved values, file contents, and reference strings are never
copied from those sources into configuration `Debug`, scheduled errors,
recovery text, `RunOutcome`, tracing, canonical events, or persisted stop
reasons. Stable metadata that happens to equal a short value is not rewritten.
`ChildEnv`, `ChildSpec`, and `PythonHook` use value-free custom `Debug` just
like the environment model. Direct `HookOutput` field access is the sole
trusted raw-output exception. Resolution performs
file I/O off the async hot path, holds no store connection or synchronous lock
across await, and calls no `std::env::set_var` or `std::env::remove_var`.

The implemented registry builder uses `register(name, handler)`,
`with_sc_compose(ScComposeHostConfig)`, and `with_atm(AtmHostConfig)`, each
returning `Result<MontyHostRegistry, CronError>`. The generated development
example and custom closure setup are in [monty-generated-example.md](monty-generated-example.md).
The b-16 child environment is explicitly empty; the resolved-snapshot text
above is the b-17 successor contract.

Shared hook environment implementation (B17):

`sc-cron::HookEnvironment` is the validated, canonical schedule policy;
`HookEnvironmentHost` is separately injected daemon authorization. Names use
portable ASCII identifiers and reject ASCII case collisions. Administrator
process-name, exact dotenv-file, and dotenv-root allowlists default empty.
Absolute normalized file paths are authorized after canonicalization, including
nearest-existing-ancestor resolution for missing suffixes. Bounded regular-file
reads run on the blocking pool. The dotenv parser never evaluates shell or
reads ambient values. Application-owned `EnvironmentReferenceResolver` receives
schedule, variable, opaque locator, and row deadline; missing, denied, unavailable
and invalid provider results use fixed value-free `ENV_REFERENCE` diagnostics.

One occurrence resolves daemon → ordered files → Set/Reference/Clear into an
immutable snapshot before stop, then reuses it for tick, Monty initialization,
and both opt-in CLI adapters. Python starts with `ChildEnv::Only`; direct trusted
callers can explicitly choose Inherit. Monty permits only environment OS reads,
including top-level reads, and denies registered application host calls during
initialization. Retained module globals initialize once per generation;
function-time getenv uses the current occurrence. Executables resolve before
child PATH clearing, independently of configured child working directories.
Windows snapshot names are uppercase and getenv lookup ignores case. Rust jobs
retain their application process environment and reject nonempty schedule policy.

Explicit Set values and opaque reference metadata are intentional plaintext
configuration. Resolved process/file/reference values never persist in registry
writes. Runtime details and reasons redact distinct nonempty snapshot values,
longest first, before caps and fixed diagnostic/recovery assembly. Debug exposes
names/kinds/counts only; trusted raw HookOutput fields remain accessible.
Exact replacement mitigates accidental disclosure and does not isolate hostile
source. The complete first-release SQLite schema is version 1 with canonical
`hook_environment` default `{"dotenv":[],"variables":{}}`; semantic schema checks
reject incompatible unreleased development tables without migrating/overwriting.
The library remains general purpose, with open application host registrations,
no signing/approval gates, and no production dependency on sc-compose or ATM.
### Rationale

ADR-RUN-0016: the Python hook inherited the complete daemon environment while Monty denied all OS calls. That is an inconsistent capability boundary and has caused difficult identity/team bugs when a daemon starts under one agent and later executes another agent's schedule. The library must support environment choices without silently capturing host values. Owner (ANC-037): "Never capture or persist inherited host values accidentally"
### Success Criteria

**Acceptance Criteria:**

- [ ] Tests make Python, Monty, both CLI adapter, and resolver paths echo canaries and prove redaction, positive delivery, and absence of a non-allowlisted daemon secret.

## Phase C approved target contract (planned, not implemented)

The following additive requirements describe the approved next release; they
do not claim current implementation or alter Phase B provenance. The existing
core remains free of product HTTP/MCP/CLI APIs. A separate optional standard
command adapter is permitted by ADR-RUN-0019.

## REQ-CRON-0020: Public receiver and cooperative owned shutdown

**Version:** 1.0.0  
**Created:** 2026-11-02  
**Last Updated:** 2026-11-03  
**Owner:** randlee  
**Status:** Approved  

### Requirement Statement

This target contract is written against `ScheduleStore` and `SqliteScheduleStore`, which PRD-CRON-2 replaces with `Storage<CronRow>`. It has not been re-planned.

The crate exposes the same nominal canonical logging receiver and supporting

values through `sc_cron::observability`; no parallel trait is introduced.

Existing constructors and legacy store implementations remain source compatible.

The open ScheduleStore and generic Scheduler seam permits application-owned

columns/tables through custom stores; bundled-schema preservation does not

forbid consumer schemas or add product fields to Schedule/command DTOs.

Cooperative custom providers must truthfully declare support and forward the

live operation scope, fencing/query semantics and outcome/ownership contract

using their own mechanisms. SQLite-specific callbacks apply to the bundled

SqliteScheduleStore, not to every provider. Legacy/custom integrations may use

the existing caller-owned path with explicitly documented guarantees.

An opt-in cooperative mode rejects unsupported stores before starting work.

It installs one shared operation context before initial load, queue submission,

lock acquisition, reload, decode and all persistence/retry/end paths. Context

checks and provider interruption observe directly mutable

deadlines while blocked. No guard waits indefinitely before consulting context.

Independent stop control closes admission, records only earlier deadlines and

wakes/interrupts operations while an owning join runs. Grace and cleanup are

absolute `std::time::Instant` deadlines in a real stop domain distinct from

the schedule `Clock`. Equal deadlines are valid: T=0 cancels immediately and

grants no cleanup-write budget. Physical joins may take time; no literal

zero-duration or arbitrary native/custom-host preemption guarantee is made.

All admitted worker and context-aware command tasks remain physically owned

until completion even if their async waiter is cancelled. The human-approved

commit admission/outcome contract forbids new ordinary commit admission at/after

G and cleanup commit admission at/after K; it does not promise universal physical

commit/fsync completion by those cutoffs. For bundled SQLite, final commit_hook admission briefly

serializes with live stop state/class/generation/real time, releases the

controller mutex before COMMIT/native I/O, and records stop-first/admission-first.

Second stop remains independently callable while the commit path is blocked.

Explicit and implicit writes share the fence. Confirmed committed-late,

confirmed rollback, no-mutation and indeterminate outcomes are typed distinctly;

expiry/Drop cannot imply rollback or not-persisted. No replay of uncertain

mutations or expired-context readback. Bounded separately authorized readback

may reconcile observed state without claiming exactly-once identity. Uncertain

connection state remains owned/quarantined; restore known hooks/TLS baseline

on safe paths. No effects occur after actual physical release. Cleanup expiry

and actual persisted result are reported honestly after join; no bool false

stands for unknown, no instantaneous T=0 join or universal 2T guarantee.

Legacy direct calls remain

caller-owned. Preserve bundled schema, generation fencing, stop-before-tick, existing

Rust/Python/Monty hooks and REQ-CRON-0019 environment policy.
### Rationale

Make the existing cron library consumable through its public logging facade and cooperative operation/lifecycle contract, preserving the already ported Rust, Python and optional Monty execution targets. The framework charter leaves application stores, schemas and wiring to the consumer. Owner (ANC-017): "it is not sc-runtime job to guarantee a customer can not misuse the core libraries."
### Success Criteria

**Acceptance Criteria:**

- [ ] The cron README and isolated external consumer prove one custom field and the chosen cooperative-or-legacy lifecycle through public scheduling/query/end APIs.
### Implementation Notes

Phase C approved target contract (planned, not implemented).

## REQ-CRON-0021: Tracked public operations and bounded schedule query

**Status:** Approved  
**Version:** 1.0.0  
**Created:** 2026-11-02  
**Last Updated:** 2026-11-02  
**Owner:** randlee  

### Requirement Statement

The standard Scheduler path has finite shared intake for public context-aware
start/end/query roots: default64, validated pre-start capacity 1..=65536,
immediate fixed typed unavailable refusal before registered/queued work when
full. One permit remains owned through actual physical completion even when
its async waiter is dropped. Stop/deadline/cancellation admission is atomic;
no hidden capacity-wait queue or automatic replay. Worker dispatch and restricted
internal cleanup retain separate existing ownership and cannot serve as a
public-command bypass. This completes REQ-CRON-0020's admission promise under
NFR-RUN-0002 and ADR-RUN-0018; it does not alter embedder row limits or impose
this mechanism on custom CronOperations/CronExecutor policy. The Phase C
contract defines exact configuration, reason mapping and causal proof owners.

The scheduler provides async context-aware start, generation-fenced stop and
query operations using the same tracked admission and cancellation protocol.
They are required by standard command execution; transport timeout cannot detach
a blocking mutation. A bounded query supports active/ended/all selection,
optional name, ascending stable id continuation and a positive capped limit.
It reads the existing table; `list_active` alone cannot satisfy this contract.
A legacy store's new capability defaults to a fixed unsupported error. End by
name resolves a specific id/generation and cannot end a replacement generation.
The public object-safe native CronOperations port returns boxed Result futures
for start/end/query with StoreOperationContext; it carries no adapter DTOs.
Scheduler's real implementation belongs to c-5. with_deadline derives a child
sharing stop/admission state and the earliest deadline; no independent registry.
No new job kind, job registry, second database or arbitrary hard-delete command
is implied. Ordinary registration continues through REQ-CRON-0003 APIs.

C1 application-owned control construction:

The ordinary-feature `WorkerStopControl::new()` factory creates a live
operation scope inside its owning Tokio runtime, without a Scheduler,
framework worker or test-utils. Applications implementing CronOperations own
intake, physical work and joins. After requesting stop and physically joining
that work, `finish_application_shutdown()` acknowledges completion and
revokes existing and newly obtained scopes even when G was in the future.
The runtime must remain alive for independent status notifications. A checked
ShutdownBudget has private fields and read-only grace()/cleanup() accessors;
K cannot precede G. Existing framework workers retain WorkerHandle-owned joins.
### Rationale

This completes the existing pre-submission admission/physical-ownership promise of REQ-CRON-0020/0021 and ADR-RUN-0018 under NFR-RUN-0002's nonblocking request path and the existing service-hardening production checklist section 7.
### Success Criteria

**Acceptance Criteria:**

- [ ] Real query returns ended as well as active rows with deterministic bounded continuation; malformed limits/cursors fail; replacement races preserve generation fencing.
- [ ] Cancellation of command waiters leaves no untracked work.
### Implementation Notes

Phase C approved target contract (planned, not implemented).
