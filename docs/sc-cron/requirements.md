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

## REQ-CRON-0002: Normal crontab strings

A schedule uses a normal crontab string of five fields: minute, hour, day of month, month, and day of week. `*/5 * * * *` means every fifth minute.

## REQ-CRON-0003: Rust, Python, or Monty hook

A schedule row names the in-process `Job`, a Python module, or a third target, `Monty { source }`, when the crate is built with feature `monty`. Python and Monty receive the `sc-cron.hook.v1` object with `protocol`, `event`, `schedule`, `payload`, and `deadline`; it has no beads, bd-cache, or ATM field. The crate exposes no HTTP route, MCP tool, or binary. A consumer may build one that inserts rows. The embedder may pass an optional maximum number of schedule rows when it opens the scheduler; this crate does not read sc-config. Writers that share one schedule database must open it with the same maximum. A Monty `cron_tick` or `cron_stop` may call the host functions `render_template(template, context)` and `sc_compose_run(subcommand, args)`, as amended [ADR-RUN-0013](../architecture.md) decision point 4 names them. `sc_compose_run` allows only subcommand `render` on the same temp-root path as `render_template`; `bead`, `init`, and `extract` fail with `COMPOSE_SUBCOMMAND`. The host runs the embedder-configured compose program (default basename `sc-compose` on `PATH`, set via [`MontyHostConfig`](../../crates/sc-cron/src/monty_host.rs) on [`Scheduler::with_monty_host`](../../crates/sc-cron/src/scheduler.rs); hook source cannot override it) and does not start CPython. The blocking-pool thread stalls until that child exits. After the child exits, a host `Instant` past the row budget records `RunOutcome::TimedOut`. Stdout longer than the hook output cap records `Failed`. A missing compose program during `render_template` records `RunOutcome::Failed` naming the program (not `HookFailed`). A Monty feed that raises still lets worker shutdown finish. The library does not panic from hook source.

## REQ-CRON-0004: One configuration table

The crate creates one schedule table. A test writes a row without constructing `SqliteWriter`. The table is the configuration. The crate does not migrate the consumer's other tables. An embedder-supplied row limit counts enabled and disabled rows, and a new name over that limit is `CronError::InvalidSpec`. Every writer that opens one schedule database must pass identical `ScheduleLimits`. The limit is not stored in the table; a writer opened with a different limit, including unlimited, does not enforce another writer's cap.

## REQ-CRON-0005: One error enum

Fallible public functions return `CronError`, including both
`process::supervise` and `process::supervise_with_logger`, which return
`Result<Captured, CronError>`. The variants are `BadExpression`, `InvalidSpec`, `AlreadyActive`, `NotFound`, `HookFailed`, `Store`, and `Fenced`, with codes `BAD_EXPRESSION`, `INVALID_SPEC`, `ALREADY_ACTIVE`, `NOT_FOUND`, `HOOK_FAILED`, `STORE_FAILED`, and `FENCED`.

With feature `monty`, a feed may record `RunOutcome::Failed` or `RunOutcome::TimedOut` instead of returning `CronError`. Stable code tokens appear in `Failed` detail strings as `CODE: message; recovery`. [`RunOutcome::failure_parts`](../../crates/sc-cron/src/run_failure.rs) and [`RunFailureParts`](../../crates/sc-cron/src/run_failure.rs) parse that form for agents and UIs. `ComposeHostError` is crate-private and is not an eighth public variant. The inventory matches [architecture.md](architecture.md):

| Condition | Code | `RunOutcome` | Operator recovery |
|---|---|---|---|
| Compose program not found (`ProgramNotFound`) | `COMPOSE_NOT_FOUND` | `Failed` — detail names the program (not `HookFailed`). A passed host deadline does not turn this into `TimedOut`. | Install `sc-compose` on PATH or fix the injected program name in tests. |
| Temp root or staged file I/O (`TempRootFailed`, `StageFailed`) | `COMPOSE_SPAWN` | `Failed` | Check permissions and disk space for temp roots under `tempfile::TempDir`. |
| Context JSON encode (`ContextEncodeFailed`) | `COMPOSE_CONTEXT` | `Failed` | Fix the template context; it could not be encoded as JSON. |
| Program cannot be executed (`ProgramNotRunnable`) | `COMPOSE_NOT_RUNNABLE` | `Failed` | Check that the compose program is executable and allowed to start. |
| Stdout is not UTF-8 (`OutputNotUtf8`) | `COMPOSE_OUTPUT` | `Failed` | Fix the compose program so its stdout is UTF-8. |
| Stdout longer than the hook output cap (`OutputTooLarge`) | `COMPOSE_OUTPUT` | `Failed` | Shrink the rendered output. |
| CLI exits non-zero (`NonZeroExit`) | `COMPOSE_EXIT` | `Failed` | Fix template/context or run `sc-compose render` manually with the logged stderr. |
| Denied `sc-compose` subcommand (`SubcommandDenied`) | `COMPOSE_SUBCOMMAND` | `Failed` | Call `sc_compose_run` with subcommand `render` only. `bead`, `init`, and `extract` are denied. |
| Denied host call (wrong arity, type, or name) | `HOST_DENIED` | `Failed` | Fix hook source; only `render_template` and `sc_compose_run` are allowlisted. |
| Undefined name | `HOST_UNDEFINED` | `Failed` | Fix hook source; that name is not defined. |
| Monty `OsCall` other than `Getenv` or `GetEnviron` | `HOST_OS` | `Failed` — detail names the operation | Fix hook source; only the two environment reads plus `render_template` and `sc_compose_run` are allowlisted. |
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
| After the CLI returns, host `Instant` exceeds the row budget |  | `TimedOut`, session dropped | Reduce compose work or raise the row deadline; the VM clock was paused during the host call. |
| Monty feed-time-limit while the VM is executing (not during the host suspend) |  | `TimedOut` | Same checkpoint path as a Monty feed that never calls `render_template`. |

Host-deadline paths record bare `RunOutcome::TimedOut` with no code token in the stored outcome.

## REQ-CRON-0019: Explicit Python and Monty hook environment

Python and Monty schedule rows share one `HookEnvironment`; existing
constructors default it to empty. A daemon-scoped `HookEnvironmentHost` names
which daemon-process variables may be inherited, authorizes absolute `.env`
files or canonical roots, and supplies an optional reference resolver. Every
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
other OS call remains denied. Render-only compose children receive that map
and do not inherit the daemon environment. sc-cron never mutates the daemon
process environment. Full semantics, limits, file authorization, and the
release-v1 schema baseline are [ADR-RUN-0016](../architecture.md).

Every hook/provider-controlled failure detail or stop reason has each distinct
non-empty resolved value replaced with `[REDACTED_ENV]` before byte capping and
before fixed codes/recovery, outcomes, events, traces, or persisted reasons are
assembled. Stable text, metadata, and the canonical environment configuration
write are exempt; deliberate set values remain plaintext schedule data. Python
output, Monty exception/return text, and compose stderr are included. Trusted
direct callers may read raw public `HookOutput` fields, but `HookOutput` debug
formatting omits their contents. This exact-value replacement mitigates
accidental propagation; trusted hooks can transform or exfiltrate values and
remain outside an untrusted-code boundary.

## NFR-CRON-0001: Allowed third-party dependencies

This item owns the third-party crates `sc-cron` may depend on. The `allowed_dependencies` list is the normal dependencies: `async-trait`, `chrono`, `cron`, `getrandom`, `process-wrap`, `rusqlite` (bundled), `sc-observability-types`, `serde_json`, `tempfile`, `tokio`, and `tracing`. `sc-lint-boundary` accepts only workspace package names, so `monty` and `monty-types` are not entries in that list. Feature `monty` stays optional and is enforced by the `sc-cron-tree-default` and `sc-cron-tree-monty` cargo-tree gates in `sc-lint.toml`: the default tree has no `monty` edge, and `--features monty` links `monty` and `monty-types` only. `monty-pool`, `monty-fs`, and `monty-runtime` are not allowed. No other workspace crate is allowed. [REQ-RUN-0005](../requirements.md) point 4 points here.

## NFR-CRON-0002: Environment isolation and redaction

Environment names use `[A-Za-z_][A-Za-z0-9_]*`. Construction, persistence,
row decode, and execution enforce 128 entries, 16 KiB per stored/resolved
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
the original spelling. Python and compose executables are
resolved to absolute paths from daemon launch policy before child `PATH` is
cleared.

Set values, resolved values, file contents, and reference strings never appear
in configuration `Debug`, scheduled errors, recovery text, `RunOutcome`,
tracing, canonical events, or persisted stop reasons. Direct `HookOutput`
field access is the sole trusted raw-output exception. Tests make Python,
Monty, compose, and resolver paths echo canaries and prove redaction, positive
delivery, and absence of a non-allowlisted daemon secret. Resolution performs
file I/O off the async hot path, holds no store connection or synchronous lock
across await, and calls no `std::env::set_var` or `std::env::remove_var`.
