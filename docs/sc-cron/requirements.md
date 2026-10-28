# sc-cron Requirements

**Status:** Active as of 2026-10-25. The contract is [architecture.md](architecture.md) and [ADR-RUN-0013](../architecture.md). Sprint `b-3` implements these items.

## REQ-CRON-0001: Workspace member

`crates/sc-cron` is a workspace member, ported from beads-cache `crates/sc-cron` at `develop`. It depends on no other workspace crate. The template does not name it. `sc-runtime` does not name it.

## REQ-CRON-0002: Normal crontab strings

A schedule uses a normal crontab string of five fields: minute, hour, day of month, month, and day of week. `*/5 * * * *` means every fifth minute.

## REQ-CRON-0003: Rust, Python, or Monty hook

A schedule row names the in-process `Job`, a Python module, or a third target, `Monty { source }`, when the crate is built with feature `monty`. Python and Monty receive the `sc-cron.hook.v1` object with `protocol`, `event`, `schedule`, `payload`, and `deadline`; it has no beads, bd-cache, or ATM field. The crate exposes no HTTP route, MCP tool, or binary. A consumer may build one that inserts rows. A Monty `cron_tick` or `cron_stop` may call the host function `render_template(template, context)`; the host runs the program named `sc-compose` on `PATH` (install the binary or symlink that name; there is no embedder override) and does not start CPython. The child and its process group are killed when the row's remaining host deadline elapses, and that feed records `RunOutcome::TimedOut`. Stdout longer than the hook output cap records `Failed`. A missing compose program during `render_template` records `RunOutcome::Failed` naming the program (not `HookFailed`).

## REQ-CRON-0004: One configuration table

The crate creates one schedule table. A test writes a row without constructing `SqliteWriter`. The table is the configuration. The crate does not migrate the consumer's other tables.

## REQ-CRON-0005: One error enum

Fallible public functions return `CronError`. The variants are `BadExpression`, `InvalidSpec`, `AlreadyActive`, `NotFound`, `HookFailed`, `Store`, and `Fenced`, with codes `BAD_EXPRESSION`, `INVALID_SPEC`, `ALREADY_ACTIVE`, `NOT_FOUND`, `HOOK_FAILED`, `STORE_FAILED`, and `FENCED`.

With feature `monty`, a feed may record `RunOutcome::Failed` or `RunOutcome::TimedOut` instead of returning `CronError`. Stable code tokens appear in `Failed` detail strings. `ComposeHostError` is crate-private and is not an eighth public variant. The inventory matches [architecture.md](architecture.md):

| Condition | Code | `RunOutcome` | Operator recovery |
|---|---|---|---|
| Compose program not found (`ProgramNotFound`) | `COMPOSE_NOT_FOUND` | `Failed` — detail names the program (not `HookFailed`). A passed host deadline does not turn this into `TimedOut`. | Install `sc-compose` on PATH or fix the injected program name in tests. |
| Temp root or staged file I/O (`TempRootFailed`, `StageFailed`) | `COMPOSE_SPAWN` | `Failed` | Check permissions and disk space for temp roots under `tempfile::TempDir`. |
| Context JSON encode (`ContextEncodeFailed`) | `COMPOSE_CONTEXT` | `Failed` | Fix the template context; it could not be encoded as JSON. |
| Program cannot be executed (`ProgramNotRunnable`) | `COMPOSE_NOT_RUNNABLE` | `Failed` | Check that the compose program is executable and allowed to start. |
| Stdout is not UTF-8 (`OutputNotUtf8`) | `COMPOSE_OUTPUT` | `Failed` | Fix the compose program so its stdout is UTF-8. |
| Stdout longer than the hook output cap (`OutputTooLarge`) | `COMPOSE_OUTPUT` | `Failed` | Shrink the rendered output. |
| CLI exits non-zero (`NonZeroExit`) | `COMPOSE_EXIT` | `Failed` | Fix template/context or run `sc-compose render` manually with the logged stderr. |
| Denied host call (wrong arity, type, or name) | `HOST_DENIED` | `Failed` | Fix hook source; only `render_template` is allowlisted. |
| Undefined name | `HOST_UNDEFINED` | `Failed` | Fix hook source; that name is not defined. |
| `OsCall` | `HOST_OS` | `Failed` — detail names the operation | Fix hook source; only `render_template` is allowlisted. |
| Monty raises during the feed | `HOST_RAISED` | `Failed` | The detail is the exception message. |
| `cron_stop` return value is not a valid stop reply | `HOST_REPLY` | `Failed` — detail is `stop check: HOST_REPLY:` then the parse error | Fix hook source; return an object with string `"action"` (`continue`, `disable`, or `delete`) and a non-empty `"reason"` when disabling or deleting. |
| Host deadline while the child runs, or the child returns after that deadline (`HostTimedOut`) |  | `TimedOut`, session dropped, spawned child killed | Reduce compose work or raise the row deadline; the VM clock was paused during the host call. |
| Monty feed-time-limit while the VM is executing (not during the host suspend) |  | `TimedOut` | Same checkpoint path as a Monty feed that never calls `render_template`. |

Host-deadline paths record bare `RunOutcome::TimedOut` with no code token in the stored outcome.

## NFR-CRON-0001: Allowed third-party dependencies

This item owns the third-party crates `sc-cron` may depend on. The `allowed_dependencies` list is the normal dependencies: `async-trait`, `chrono`, `cron`, `getrandom`, `process-wrap`, `rusqlite` (bundled), `serde_json`, `tempfile`, `tokio`, and `tracing`. `sc-lint-boundary` accepts only workspace package names, so `monty` and `monty-types` are not entries in that list. Feature `monty` stays optional and is enforced by the `sc-cron-tree-default` and `sc-cron-tree-monty` cargo-tree gates in `sc-lint.toml`: the default tree has no `monty` edge, and `--features monty` links `monty` and `monty-types` only. `monty-pool`, `monty-fs`, and `monty-runtime` are not allowed. No other workspace crate is allowed. [REQ-RUN-0005](../requirements.md) point 4 points here.
