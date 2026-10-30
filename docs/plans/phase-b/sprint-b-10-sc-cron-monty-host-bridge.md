---
id: b-10
phase: b
status: complete
closure_type: boundary
target_boundary: sc-cron
track: cron-monty
wave: 4
branch: sprint/b-10-sc-cron-monty-host-bridge
worktree: sprint/b-10-sc-cron-monty-host-bridge
integration_branch: integrate/phase-b-monty
requirements:
  - REQ-CRON-0001
  - REQ-CRON-0002
  - REQ-CRON-0003
  - REQ-CRON-0004
  - REQ-CRON-0005
  - NFR-CRON-0001
  - REQ-RUN-0005
  - NFR-RUN-0002
  - NFR-RUN-0004
  - NFR-RUN-0005
  - NFR-RUN-0008
  - NFR-RUN-0009
adrs:
  - ADR-RUN-0013
  - ADR-RUN-0011
  - ADR-RUN-0007
owned_paths:
  - crates/sc-cron/src/lib.rs
  - crates/sc-cron/src/hook.rs
  - crates/sc-cron/src/monty_run.rs
  - crates/sc-cron/src/monty_host.rs
  - crates/sc-cron/src/monty_compose.rs
  - crates/sc-cron/src/scheduler/run_due.rs
  - crates/sc-cron/src/worker/worker_runs.rs
  - crates/sc-cron/tests/monty_host_bridge.rs
  - crates/sc-cron/tests/monty_hook.rs
  - crates/sc-cron/tests/fixtures/
  - crates/sc-cron/README.md
  - crates/sc-cron/Cargo.toml
  - docs/sc-cron/architecture.md
  - docs/sc-cron/requirements.md
  - docs/sc-cron/hooks-sc-compose.md
  - docs/architecture.md
  - docs/project-plan.md
must_follow: [b-9]
parallel_safe: [b-8]
recommended_agent: solar
recommended_model: sol
---

# Sprint b-10 — Monty host bridge for sc-compose

Plan only. No crate change lands in the planning PR. Implementation targets
`integrate/phase-b-monty` (see [phase-b-integration-close.md](phase-b-integration-close.md)).
`b-9` is on `integrate/phase-b`. This sprint does not reopen it.

## Relations

`must_follow` `b-9`. The artifact is `HookTarget::Monty` plus
`monty_run::execute_feed`, which today uses `call_function` and cannot resume
host functions on monty 1.4.0. This sprint switches production feeds to
`feed_start` plus the progress loop below. A separate contract-only sprint
would be thin (one implementer, one consumer).

`parallel_safe` with `b-8`. Store generation does not consume this ext fn,
and the paths do not intersect. The sprint is also parallel to phase-end
review on `integrate/phase-b`: it does not edit that branch.

## Goal

`REQ-CRON-0003`: a Monty `cron_tick` or `cron_stop` may call the host function
`render_template`. The host runs the Rust `sc-compose` CLI. The tick does not
start CPython.

`REQ-CRON-0002` and `REQ-CRON-0004` stay as `b-3` and `b-9` left them: a normal
schedule string is five fields, minute first, and the crate keeps one schedule
table. This sprint does not parse expressions and does not add a table.

## This sprint does not close

- Campaign JSON, the benchmark-run skill, and `site/reports/benchmark-monty/`.
  Those are `b-11`, which consumes `render_template`.
- `crates/sc-cron/tests/sc_compose_bench.rs`. `b-11` owns that harness.
- Phase-end QA on `integrate/phase-b`.
- An in-process compose engine. This sprint spawns the program selected by
  the embedder, whose default is the basename `sc-compose` on `PATH`.
- `monty-pool`, `monty-fs`, `monty-runtime`, and any PyO3 import inside the VM.

## Contract (monty 1.4.0)

The `render_template` / `ComposeHostError` contract below is the same contract
as `docs/sc-cron/architecture.md` and `crates/sc-cron/src/monty_host.rs`:
`ComposeProgram`, and nine crate-private `ComposeHostError` variants.
`CronError` stays the seven public variants.

`MontyRepl::call_function` cannot resume host functions on monty 1.4.0; only
`feed_start` yields `ReplProgress`. Production therefore **replaces** the
`call_function` path inside `execute_feed_with_compose_program` (via
`drive_feed`) with `feed_start` on a one-line snippet (`cron_tick(__hook)` or
`cron_stop(__hook)`). cfg(test) `execute_feed` wraps the same loop with the
default basename only. Compile still uses
`feed_run` of the hook source as today. The repl is not dropped while a feed
is on the stack except on denied paths below (`b-9`). Host work runs on the
same blocking-pool thread as the feed (ADR-RUN-0007, ADR-RUN-0011): no
`block_on`, no sleep, no mutex held across `.await`.

Named values for the snippet bind **`render_template`** (reserved host
binding; hook authors call this bare name) via `MontyObject::function` and
`__hook` via the existing `json_to_monty` value. No other ext fn names are
registered in this sprint.

```rust
// crates/sc-cron/src/monty_host.rs — crate-private program handle. Embedder path:
// MontyHostConfig on Scheduler (default "sc-compose" on PATH). Tests pass another program.
pub(crate) struct ComposeProgram<'a>(&'a str);

pub(crate) enum ComposeHostError {
    ProgramNotFound { program: String },
    TempRootFailed { program: String, source: std::io::Error },
    StageFailed { program: String, source: std::io::Error },
    ContextEncodeFailed { program: String, source: serde_json::Error },
    ProgramNotRunnable { program: String, source: std::io::Error },
    OutputNotUtf8 { program: String },
    OutputTooLarge { program: String },
    NonZeroExit { status: Option<i32>, stderr: String },
    SubcommandDenied { subcommand: String },
}

impl ComposeHostError {
    pub(crate) const fn code(&self) -> &'static str;
    pub(crate) const fn recovery(&self) -> &'static str;
}

pub(crate) fn render_template(
    template: &str,
    context: &serde_json::Value,
    compose_program: ComposeProgram<'_>,
) -> Result<String, ComposeHostError>;
```

`compose_program` in the progress loop is that `ComposeProgram`. The embedder
sets it on [`Scheduler::with_monty_host`](../../crates/sc-cron/src/scheduler.rs)
via public [`MontyHostConfig`](../../crates/sc-cron/src/monty_host.rs) (default
`sc-compose` on `PATH`, like `PythonHook`'s default `python3`). Hook source
cannot override the path.

The missing-program unit tests pass a name that is not installed. No
`std::env::set_var` and no PATH rewrite (`NFR-RUN-0005`).

Progress loop inside `execute_feed_with_compose_program` (repl taken by value; state is
`ReplProgress`, not a single `match` on `feed_start`):

```rust
let mut progress = match repl.feed_start(snippet, &inputs, monty_types::PrintWriter::Disabled) {
    Ok(p) => p,
    Err(_) => return FeedDone { session: None, result: FeedResult::Outcome(failed(...)) },
};
loop {
    progress = match progress {
        monty::ReplProgress::FunctionCall(call) if call.function_name == "render_template" => {
            let template_ref = call.args.arg(0).ok_or_else(|| denied(...))?;
            let template = template_ref.as_str().ok_or_else(|| denied(...))?;
            let context_ref = call.args.arg(1).ok_or_else(|| denied(...))?;
            let context = monty_host::monty_value_to_json(context_ref)?;
            let rendered = monty_host::render_template(template, &context, compose_program)?; // ComposeProgram; map Err to FeedDone, do not return it from execute_feed_with_compose_program
            call.resume(
                monty_types::ExtFunctionResult::Return(monty_types::MontyObject::string(rendered)),
                monty_types::PrintWriter::Disabled,
            )? // Ok(next ReplProgress) — stay in loop (second render_template is allowed)
        }
        monty::ReplProgress::Complete { repl, value } => {
            // Only success exit: parse value with today's stop/tick rules; store repl in session
            break;
        }
        monty::ReplProgress::FunctionCall(_) | monty::ReplProgress::OsCall(_) => {
            return FeedDone { session: None, result: FeedResult::Outcome(failed(...)) };
        }
        _ => return FeedDone { session: None, result: FeedResult::Outcome(failed(...)) },
    };
}
```

Do not use `?` to return from `execute_feed_with_compose_program` (which returns `FeedDone`). Map
`ReplStartError` and denied paths to `FeedDone` with `session: None`.

Wrong arity, non-str template, or a context value `monty_value_to_json` cannot
represent → denied `FeedDone` with `RunOutcome::Failed` (no new `CronError`
variant). `monty_value_to_json` mirrors `json_to_monty` for null, bool, int,
float, string, list, and dict (`pairs()`); any other Monty type is denied.

`render_template` writes the template and the context JSON into a fresh
`tempfile::TempDir` and runs:

```text
sc-compose render --root <temp> --mode file --file t.j2 --var-file v.json
```

The Monty source does not choose a filesystem path. `--mode file` reads only
those two temp files. Stdout is the rendered string. The temp dir is removed
when the call returns.

Allowlist is `render_template(template, context)` and
`sc_compose_run(subcommand, args)` (production: `render` only; hook cannot
choose `--root` or disallowed subcommands). `compose_file` is out of scope.
`OsCall` and any other function name fail the feed.

### Errors (`REQ-CRON-0005`)

No new `CronError` variant.

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
| `OsCall` | `HOST_OS` | `Failed` — detail names the operation | Fix hook source; only `render_template` and `sc_compose_run` are allowlisted. |
| Monty raises during the feed | `HOST_RAISED` | `Failed` | The detail is the exception message. |
| `cron_stop` return value is not a valid stop reply | `HOST_REPLY` | `Failed` — detail is `stop check: HOST_REPLY:` then the parse error | Fix hook source; return an object with string `"action"` (`continue`, `disable`, or `delete`) and a non-empty `"reason"` when disabling or deleting. |
| Monty feed task join failed (cancelled blocking-pool task) | `FEED_TASK_FAILED` | `Failed` | Retry the run; if it persists, inspect blocking-pool capacity and hook source. |
| Monty feed task panicked (worker join) | `FEED_JOIN_FAILED` | `Failed` | Fix the panic in the hook or host and retry the schedule. |
| Monty due task panicked or was cancelled | `DUE_JOIN_FAILED` | `Failed` | Retry the schedule; if it persists, inspect the Monty due task and hook source. |
| Monty due path used for a non-Monty hook | `MONTY_RESOLVE` | `Failed` | Run this row on its Rust or Python target, or set `HookTarget::Monty`. |
| `cron_tick` returned a stop action (disable/delete) | `MONTY_TICK` | `Failed` | Return normally from `cron_tick`; disable and delete belong in `cron_stop`. |
| After the CLI returns, host `Instant` exceeds the row budget |  | `TimedOut`, session dropped | Reduce compose work or raise the row deadline; the VM clock was paused during the host call. |
| Monty feed-time-limit while the VM is executing (not during the host suspend) |  | `TimedOut` | Same checkpoint path as a Monty feed that never calls `render_template`. |

Feeds that never call `render_template` still run when the compose binary is absent.

Monty's `ResourceTracker` pauses `max_feed_duration` while suspended on a host
`FunctionCall`; CLI wall time is **not** on the VM feed clock. The blocking-pool thread stalls until the compose `Command` returns. After that child exits, a host `Instant` past the row budget records `TimedOut`. The feed then records bare `RunOutcome::TimedOut` (no code token in the stored outcome) and drops the repl.
Stdout past the hook output cap is `COMPOSE_OUTPUT`.

### Dependency (`REQ-CRON-0001`, `REQ-RUN-0005`)

Feature `monty` stays optional (`NFR-CRON-0001`). No new workspace crate and
no dependency on another workspace crate. Reuse the existing **`tempfile`**
dependency (already on `sc-cron` for Python hooks); do not add a second copy
under `monty` only. Do not add `monty`, `monty-types`, or `sc-compose` to
`boundaries/sc-cron` `allowed_dependencies`. `Cargo.lock` does not change.
`crates/sc-cron/Cargo.toml` gains one `[[test]]` target, justified below, and
no new dependency. Default CI (Rust 1.94.1) does not enable `monty`.

`NFR-RUN-0004`: `process::supervise` already kills a process group at an
expiry. It is an async future polled on a tokio runtime. `render_template`
runs on the blocking pool inside `spawn_blocking` (ADR-RUN-0007,
ADR-RUN-0011). Driving `supervise` from that thread would require
`block_on`, which the sleep gate rejects and which stalls a current-thread
runtime. The host stalls on `std::process::Child::wait` until the compose
child exits. It does not kill the child when the row budget elapses, and it
does not call `process::supervise`. After the child exits, a host `Instant`
past the row budget records `TimedOut`.

`NFR-RUN-0008`: `monty_host.rs`, `monty_compose.rs`, and `monty_run.rs` stay within 1000 source
lines. `NFR-RUN-0009`: public fallible functions still return `CronError`.
`ComposeHostError` is crate-private. Code on the feed path does not use
panicking slice indexing; `stderr_text` takes its prefix with `str::get`.

**Testing layout (`REQ-RUN-0005`, boundary):** inject/deny/resume unit tests
live in `crates/sc-cron/src/monty_host.rs` under
`#[cfg(all(test, feature = "monty"))]`. `crates/sc-cron/tests/monty_host_bridge.rs`
exercises only the public **`Scheduler`** / `run_due` → `drive_feed` path end-to-end; it
does not import `monty_host` directly. That file is an integration test, so a
`cfg` attribute on the library does not keep `cargo test` from compiling it.
`Cargo.toml` therefore names the target and sets `required-features = ["monty"]`:
a default `cargo test` skips the target and does not link the interpreter;
`cargo test -p sc-cron --features monty --test monty_host_bridge` builds it and
runs it. The same gate is already used for `monty_hook`.

## ADR-RUN-0013 amendment

On the `b-9` text of point 4, add: under feature `monty`, a hook may call the
host functions `render_template(template, context)` and
`sc_compose_run(subcommand, args)`. sc-cron resumes those calls inside
`drive_feed` via `execute_feed_with_compose_program` by spawning the compose program the embedder set with
`Scheduler::with_monty_host` and `MontyHostConfig`. The default, when the
embedder does not call `with_monty_host`, is the basename `sc-compose` on
`PATH` (install that binary or symlink the name). `MontyHostConfig::new`
accepts any non-empty path and returns `CronError::InvalidSpec` when the path
is empty. A basename is resolved on `PATH`; an absolute path, or any other
non-empty path, is spawned as given. Hook source cannot choose the program or
the temp root. `OsCall` and any other host function name fail the feed. The
tick does not start CPython. **Replace** the `b-9` Monty timeout sentence for
time inside the host call: `ResourceTracker` pauses `max_feed_duration` while
the feed is suspended on that host call; the blocking-pool thread stalls until
the compose `Command` returns; after that child exits, a host `Instant` past
the row budget records `RunOutcome::TimedOut` and drops the repl (this is
**not** a Monty feed-time-limit error). The `b-10` consequence names this
embedder program rule. The amendment is **Active** in
[ADR-RUN-0013](../../architecture.md): the embedder sets the compose program
through `MontyHostConfig`.

`REQ-CRON-0003` gains one sentence for those calls and for `MontyHostConfig` /
`Scheduler::with_monty_host`. A missing compose program during
`render_template` records `RunOutcome::Failed` naming the program (not
`HookFailed`). `docs/sc-cron/architecture.md` shows the signature, the embedder
program rule, and the error table above. `docs/sc-cron/hooks-sc-compose.md`
states that production ships in `drive_feed` /
`execute_feed_with_compose_program` (which may call `feed_start` internally;
cfg(test) `execute_feed` uses the default basename only), that the compose program is `MontyHostConfig` (default
`sc-compose` on `PATH`; hook source cannot choose the program or the temp
root), and that the standalone lab `MontyRepl::feed_start` in the bench is not
the production path.

## Deliverables

1. `REQ-CRON-0003`: `monty_host::render_template` and the `ReplProgress` loop in `execute_feed_with_compose_program` (production via `drive_feed`), with the signature and deny rules above.
2. `REQ-CRON-0005` and `NFR-RUN-0009`: the error table above, the same rows as `docs/sc-cron/architecture.md`. `CronError` stays seven variants. `ComposeHostError` is not an eighth public variant.
3. `REQ-CRON-0001`, `NFR-CRON-0001`, and `REQ-RUN-0005`: no workspace dependency, no boundary-manifest edit, no `Cargo.lock` edit. The `Cargo.toml` `[[test]]` entry only sets `required-features`.
4. `NFR-RUN-0005`: the bridge test uses `tempfile::TempDir`, injects the compose program name (not PATH mutation), and does not call `std::env::set_var`.
5. ADR-RUN-0013 point 4 (including the host-timeout sentence above and the Active embedder program rule: `Scheduler::with_monty_host` / `MontyHostConfig`, default basename `sc-compose` on `PATH`, any non-empty path legal, empty path `CronError::InvalidSpec`, hook source cannot choose the program or the temp root), the matching `b-10` consequence, the `REQ-CRON-0003` sentence, and the hooks note.
6. `REQ-CRON-0002` and `REQ-CRON-0004`: no expression-parser change and no new table.
7. `NFR-RUN-0004`: the compose child is waited with `std::process::Child::wait` on the blocking-pool thread, as justified above, and does not call `process::supervise` or stop the child at the row budget.
8. `NFR-RUN-0008`: `monty_host.rs`, `monty_compose.rs`, and `monty_run.rs` stay within 1000 source lines.

## Acceptance criteria

Root: `boundary:sc-cron` and ADR-RUN-0013 as amended here.

- `cargo test -p sc-cron --features monty --test monty_host_bridge` on Rust 1.96.1 **must execute** (not skip): success path with real `sc-compose` on `PATH` (stdout body); deny paths for another function name and for `OsCall`; non-zero CLI → `Failed`. When `sc-compose` is absent, those cases **fail** the test. Skips are not an acceptance outcome. This target does not inject a program name and does not call `with_monty_host`, so it exercises the default basename `sc-compose` on `PATH`. Production `Scheduler` spawns the program in `MontyHostConfig` (that default unless the embedder sets another non-empty path). Hook source cannot choose the program or the temp root.
- The missing-program case is the `monty_host` unit tests: `cargo test -p sc-cron --features monty --lib a_missing_program` on Rust 1.96.1. A non-installed program name → `RunOutcome::Failed` whose detail names that program (`COMPOSE_NOT_FOUND`, not `HOOK_FAILED`) and the repl is dropped (`a_missing_program_names_itself_and_does_not_spawn_on_a_bad_call`, `a_missing_program_on_the_feed_drops_the_repl`, `a_missing_program_is_failed_not_timed_out`).
- On Unix, `cargo test -p sc-cron --features monty --lib an_injected_program_sees_the_render_argv` checks the child argv: `render`, `--root`, `--mode`, `file`, `--file`, `--var-file`, and `--output` omitted.
- `cargo test -p sc-cron` with the default features does not compile the `monty` crates. `cargo tree -p sc-cron -e normal --prefix none` prints no crate whose name starts with `monty`.
- `boundaries/sc-cron/sc-cron.toml` has no `monty`, `monty-types`, or `sc-compose` entry added by this sprint.
- `docs/architecture.md` point 4 names `render_template`, `sc_compose_run`, `Scheduler::with_monty_host`, `MontyHostConfig` (default basename `sc-compose` on `PATH`; hook source cannot choose the program or the temp root), and the host-timeout rule for time inside the host call. `docs/sc-cron/requirements.md` has the `REQ-CRON-0003` sentence. `docs/sc-cron/hooks-sc-compose.md` names `drive_feed` / `execute_feed_with_compose_program` as the production path (cfg(test) `execute_feed` is the default-basename wrapper only) and names the same embedder program rule.
- `rg -n 'std::thread::sleep|tokio::time::sleep|block_on' crates/sc-cron/src` exits 1.

## Required validation

- `cargo test -p sc-cron` exits 0 on Rust 1.94.1.
- `cargo test -p sc-cron --features monty` exits 0 on Rust 1.96.1.
- `cargo build --workspace` exits 0.
- `sc-lint lint sc-boundary --root . --config sc-lint.toml` exits 0.
- The `rg` gate above exits 1.


Recorded 2026-10-27 on this branch after the compose `Child::wait` stall change:

- `cargo fmt -p sc-cron`
- `cargo clippy -p sc-cron --features monty --all-targets -- -D warnings` exited 0
- `cargo test -p sc-cron --features monty` exited 0 (including the `Child::wait` stall and post-exit host `Instant` `TimedOut` tests, and `monty_host_bridge`)
- `cargo build --workspace` exited 0
- `rg -n 'std::thread::sleep|tokio::time::sleep|block_on' crates/sc-cron/src` exited 1

The bridge test schedules `* * * * *`. `boundaries/sc-cron` was not edited, so the boundary lint was not re-run.

Scope note (ceremony screen): FTQ-003 (production-side kill of a stalled `sc-compose`) and `cargo nextest` are out of scope for this sprint. The bridge tests assume `sc-compose` finishes, per the acceptance above. A stalled child blocks in `Child::wait`; the feed records `TimedOut` from the host `Instant` only after the child exits.

## Non-goals

No `monty-pool`. No sandbox. No third host function beyond `render_template`
and `sc_compose_run`. No edit to
`sc_compose_bench.rs`, `Justfile`, `scripts/benchmark/`, or
`site/reports/`. No phase-close fix on `integrate/phase-b`.
