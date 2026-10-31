# sc-cron Requirements

**Status:** Active as of 2026-10-28. The contract is [architecture.md](architecture.md), [ADR-RUN-0013](../architecture.md), and [ADR-RUN-0016](../architecture.md). Sprints `b-3` through `b-17` implement these items.

## REQ-CRON-0001: Workspace member

`crates/sc-cron` is a workspace member, ported from beads-cache
`crates/sc-cron` at `develop`. It depends on no other workspace crate and
uses `sc-observability-types` only for its injected canonical logger contract.
`sc-runtime` does not name it. The generated daemon may name it only as the
optional `cron` dependency selected by the embedding application; default
generated builds do not link it. Here "name" means a Cargo or other manifest
dependency. Documented wiring may reference the crate in comments and schema
descriptions.

The Phase B injection API is packaged at workspace version `0.1.1`, with
`sc-cron` publishable as the fifth public library. Its normal observability
edge is only `sc-observability-types` with caret `1.5.1`; `test-util` is enabled
only in dev-dependencies. During the authorized development exception that
edge inherits the local facade path; actual publication remains gated on
removing local paths and the real matching registry pair.

## REQ-CRON-0002: Normal crontab strings

A schedule uses a normal crontab string of five fields: minute, hour, day of month, month, and day of week. `*/5 * * * *` means every fifth minute.

## REQ-CRON-0003: Rust, Python, or Monty hook

sc-cron is a general-purpose scheduler for application-defined jobs. A schedule row names an in-process `Job`, a Python module, or `Monty { source }` under optional feature `monty`. Python and Monty receive the `sc-cron.hook.v1` object with `protocol`, `event`, `schedule`, `payload`, and `deadline`. It has no consumer-specific beads, bd-cache, or ATM field. The crate exposes no HTTP route, MCP tool, or binary. The consumer owns that API and configures schedule limits; sc-cron does not read sc-config.

Monty hosts are application-owned registrations in an immutable `MontyHostRegistry` installed with `Scheduler::with_monty_hosts`. The empty registry registers no application call. `MontyHostHandler: Send + Sync` accepts a `MontyHostCall` containing its validated `MontyHostName`, positional/keyword JSON arguments, schedule name, hook phase, and row deadline and returns `Result<serde_json::Value, CronError>`; matching closures are supported. `MontyHostName::new` validates the identifier and `as_str()` borrows it; registry keys use the same type while registration still accepts strings. Registration rejects invalid identifiers, duplicate names, and reserved `__hook`, `cron_stop`, and `cron_tick` bindings with `InvalidSpec`. Source can invoke registered functions but cannot install or replace handlers. An unregistered name fails with `HOST_UNDEFINED`. Bridge argument conversion and supplied CLI adapter shape rejection fail with `HOST_DENIED`; a custom handler's returned error, including `CronError::InvalidSpec`, records `HOST_CALL_FAILED` after configured-value redaction. Private typed dispatch preserves supplied-adapter errors without parsing custom error prose. The host checks the row deadline before and after a synchronous handler, but cannot preempt arbitrary application code.

Phase B supplies opt-in `sc_compose_run(subcommand, args)` and `atm_run(subcommand, args)` adapters through this same registry. `ScComposeHostConfig` and `AtmHostConfig` select non-empty executable paths (defaults `sc-compose` and `atm`) and working directories (default the daemon directory captured at configuration). Each adapter accepts one command/subcommand string and an optional list of string arguments, with no keyword arguments. It forwards the non-empty command/subcommand followed by those ordered arguments directly as argv, never through a shell. Every command/subcommand is supported; flags including `--root` are forwarded unchanged. Neither adapter forces render arguments or temp-root staging. Scripts choose arguments, not the configured executable or working directory. Both adapters share the existing blocking child driver in `monty_compose`, generalized for argv/cwd, with row deadline, process-group termination, and `MAX_HOOK_OUTPUT_BYTES` per stream. They do not call the async `process::supervise` from the blocking feed thread or introduce a runtime/`block_on` bridge. A completed child returns `{stdout, stderr, exit_code}`, including nonzero exits; script code decides how that result affects its hook. A deadline expiration records `TimedOut`; spawn, decoding, and capture-limit failures record stable host errors. The adapters receive the resolved environment snapshot from ADR-RUN-0016. Neither CLI is required when its adapter is unused. `render_template` is not a product requirement; an internal helper may remain an implementation convenience without becoming a mandatory registered host function.

MVP accepts application-supplied Monty source without signatures, a separate script-approval mechanism, or a consumer option enabling such a mechanism. Capability registration and environment policy are enforced independently. In-process Monty and custom Rust handlers retain stack-overflow, allocator-abort, panic, and non-cooperative deadline risks; they are not an isolation boundary for hostile code.

The stop/tick ordering, generation fencing, retained REPL, interpreter-checkpoint VM limits, and joined shutdown behavior remain as specified by ADR-RUN-0013. Host calls use the production `drive_feed` resume loop; the cfg(test) helper is not a separate production path.

## REQ-CRON-0004: One configuration table

The crate creates one schedule table. A test writes a row without constructing `SqliteWriter`. The table is the configuration. The release-v1 row includes canonical `hook_environment` text with exact empty default `{"dotenv":[],"variables":{}}`; it does not add a second environment or secret table. The crate does not migrate the consumer's other tables. An embedder-supplied row limit counts enabled and disabled rows, and a new name over that limit is `CronError::InvalidSpec`. Every writer that opens one schedule database must pass identical `ScheduleLimits`. The limit is not stored in the table; a writer opened with a different limit, including unlimited, does not enforce another writer's cap.

## REQ-CRON-0005: One error enum

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
| `.env` file invalid | `ENV_FILE_INVALID` | `Failed` — value-free detail | Fix the bounded data-file syntax; shell expansion is not supported. |
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

## REQ-CRON-0019: Explicit Python and Monty hook environment

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

## NFR-CRON-0001: Allowed third-party dependencies

This item owns the third-party crates `sc-cron` may depend on. The `allowed_dependencies` list is the normal dependencies: `async-trait`, `chrono`, `cron`, `getrandom`, `process-wrap`, `rusqlite` (bundled), `sc-observability-types`, `serde_json`, `tempfile`, `tokio`, and `tracing`. `sc-lint-boundary` accepts only workspace package names, so `monty` and `monty-types` are not entries in that list. Feature `monty` stays optional and is enforced by the `sc-cron-tree-default` and `sc-cron-tree-monty` cargo-tree gates in `sc-lint.toml`: the default tree has no `monty` edge, and `--features monty` links `monty` and `monty-types` only. `monty-pool`, `monty-fs`, and `monty-runtime` are not allowed. No other workspace crate is allowed. [REQ-RUN-0005](../requirements.md) point 4 points here.

The tool-crate exclusion in REQ-RUN-0005 applies to sc-cron: sc-compose and ATM crates are forbidden in production dependencies/imports; test and benchmark references are permitted. CLI process invocation does not create a crate dependency.

## NFR-CRON-0002: Environment isolation and redaction

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
trusted raw-output exception. Tests make Python,
Monty, both CLI adapter, and resolver paths echo canaries and prove redaction, positive
delivery, and absence of a non-allowlisted daemon secret. Resolution performs
file I/O off the async hot path, holds no store connection or synchronous lock
across await, and calls no `std::env::set_var` or `std::env::remove_var`.

The implemented registry builder uses `register(name, handler)`,
`with_sc_compose(ScComposeHostConfig)`, and `with_atm(AtmHostConfig)`, each
returning `Result<MontyHostRegistry, CronError>`. The generated development
example and custom closure setup are in [monty-generated-example.md](monty-generated-example.md).
The b-16 child environment is explicitly empty; the resolved-snapshot text
above is the b-17 successor contract.


### Shared hook environment implementation (B17)

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
