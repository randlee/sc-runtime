---
id: b-10
phase: b
status: planned
closure_type: boundary
target_boundary: sc-cron
track: cron-monty
wave: 4
branch: sprint/b-10-sc-cron-monty-host-bridge
worktree: sprint/b-10-sc-cron-monty-host-bridge
integration_branch: integrate/phase-b-monty
requirements:
  - REQ-CRON-0001
  - REQ-CRON-0003
  - REQ-CRON-0005
  - REQ-RUN-0005
  - NFR-RUN-0002
  - NFR-RUN-0005
adrs:
  - ADR-RUN-0013
  - ADR-RUN-0011
  - ADR-RUN-0007
owned_paths:
  - crates/sc-cron/src/lib.rs
  - crates/sc-cron/src/monty_run.rs
  - crates/sc-cron/src/monty_host.rs
  - crates/sc-cron/tests/monty_host_bridge.rs
  - docs/sc-cron/architecture.md
  - docs/sc-cron/requirements.md
  - docs/sc-cron/hooks-sc-compose.md
  - docs/architecture.md
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

## This sprint does not close

- Campaign JSON, the benchmark-run skill, and `site/reports/benchmark-monty/`.
  Those are `b-11`, which consumes `render_template`.
- `crates/sc-cron/tests/sc_compose_bench.rs`. `b-11` owns that harness.
- Phase-end QA on `integrate/phase-b`.
- An in-process compose engine. This sprint spawns `sc-compose` on `PATH`.
- `monty-pool`, `monty-fs`, `monty-runtime`, and any PyO3 import inside the VM.

## Contract (monty 1.4.0)

`MontyRepl::call_function` cannot resume host functions on monty 1.4.0; only
`feed_start` yields `ReplProgress`. Production therefore **replaces** the
`call_function` path inside `execute_feed` with `feed_start` on a one-line
snippet (`cron_tick(__hook)` or `cron_stop(__hook)`). Compile still uses
`feed_run` of the hook source as today. The repl is not dropped while a feed
is on the stack except on denied paths below (`b-9`). Host work runs on the
same blocking-pool thread as the feed (ADR-RUN-0007, ADR-RUN-0011): no
`block_on`, no sleep, no mutex held across `.await`.

Named values for the snippet bind **`render_template`** (reserved host
binding; hook authors call this bare name) via `MontyObject::function` and
`__hook` via the existing `json_to_monty` value. No other ext fn names are
registered in this sprint.

```rust
// crates/sc-cron/src/monty_host.rs — crate-private; program name injectable for tests.
pub(crate) enum ComposeHostError {
    ProgramNotFound { program: String },
    SpawnFailed { program: String, detail: String },
    NonZeroExit { status: i32, stderr: String },
}

impl ComposeHostError {
    pub(crate) fn recovery(&self) -> &'static str { /* see error table */ }
}

pub(crate) fn render_template(
    template: &str,
    context: &serde_json::Value,
    compose_program: &str,
) -> Result<String, ComposeHostError>;
```

Production passes `compose_program = "sc-compose"`. The missing-binary test
passes a name that is not installed. No `std::env::set_var` and no PATH
rewrite (`NFR-RUN-0005`).

Progress loop inside `execute_feed` (repl taken by value; state is
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
            let rendered = monty_host::render_template(template, &context, compose_program)?;
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

Do not use `?` to return from `execute_feed` (which returns `FeedDone`). Map
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

Allowlist is `render_template` only. `compose_file` is out of scope. `OsCall`
and any other function name fail the feed.

### Errors (`REQ-CRON-0005`)

No new `CronError` variant.

| Condition | `RunOutcome` / detail | Operator `recovery()` (maps to existing `CronError::recovery()` text where applicable) |
|---|---|---|
| Configured compose program not found (`ProgramNotFound`) | `Failed` — detail names the program (not `HookFailed`) | Install `sc-compose` on PATH or fix the injected program name in tests. |
| Spawn or I/O error starting the child (`SpawnFailed`) | `Failed` | Check permissions and disk space for temp roots under `tempfile::TempDir`. |
| CLI exits non-zero (`NonZeroExit`) | `Failed` | Fix template/context or run `sc-compose render` manually with the logged stderr. |
| Monty raises or denied host call (wrong arity/type/name) | `Failed` | Fix hook source; only `render_template` is allowlisted. |
| After CLI returns, host `Instant` exceeds row budget | `TimedOut` | Reduce compose work or raise row deadline; VM clock was paused during the host call. |
| Monty feed-time-limit while VM executing (not on host suspend) | `TimedOut` | Same as `b-9` checkpoint path. |

Feeds that never call `render_template` still run when the compose binary is absent.

Monty's `ResourceTracker` pauses `max_feed_duration` while suspended on a host
`FunctionCall`; CLI wall time is **not** on the VM feed clock. The host times
the child with `Instant`. If the child never exits, the blocking-pool thread
stays occupied (same accepted stall as a Monty hook that never reaches another
checkpoint); there is no second kill and no outcome recorded while the child
is alive.

### Dependency (`REQ-CRON-0001`, `REQ-RUN-0005`)

Feature `monty` stays optional. No new workspace crate and no dependency on
another workspace crate. Reuse the existing **`tempfile`** dependency (already
on `sc-cron` for Python hooks); do not add a second copy under `monty` only.
Do not add `monty`, `monty-types`, or `sc-compose` to `boundaries/sc-cron`
`allowed_dependencies`. `Cargo.toml` / `Cargo.lock` change only if `tempfile`
is not already present (it is on the integrated tree). Default CI (Rust 1.94.1)
does not enable `monty`. Bridge tests are `cfg(feature = "monty")` on Rust
1.96.1.

**Testing layout (`REQ-RUN-0005`, boundary):** inject/deny/resume unit tests
live in `crates/sc-cron/src/monty_host.rs` under
`#[cfg(all(test, feature = "monty"))]`. `crates/sc-cron/tests/monty_host_bridge.rs`
exercises only the public **`Scheduler`** / `execute_feed` path end-to-end; it
does not import `monty_host` directly.

## ADR-RUN-0013 amendment

On the `b-9` text of point 4, add: under feature `monty`, a hook may call the
host function `render_template(template, context)`; sc-cron resumes it by
spawning `sc-compose` on `PATH` against a temp root the hook cannot choose;
`OsCall` and any other function name fail the feed; the tick does not start
CPython. **Replace** the `b-9` Monty timeout sentence for time inside
`render_template`: `ResourceTracker` pauses `max_feed_duration` while the feed
is suspended on that host call; the host times the child with `Instant`; if the
child returns after the row's remaining budget, the outcome is
`RunOutcome::TimedOut` and the repl is dropped when that feed returns; a child
that never exits is the accepted blocking-pool stall with no second kill and
no outcome while it is alive (this is **not** a Monty feed-time-limit error).
Add a consequence naming sprint `b-10`.

`REQ-CRON-0003` gains one sentence for that call. A missing compose program
during `render_template` records `RunOutcome::Failed` naming the program (not
`HookFailed`). `docs/sc-cron/architecture.md` shows the signature and error
table above. `docs/sc-cron/hooks-sc-compose.md` states that production ships in
`execute_feed` (which may call `feed_start` internally) and that the standalone
lab `MontyRepl::feed_start` in the bench is not the production path.

## Deliverables

1. `REQ-CRON-0003`: `monty_host::render_template` and the `ReplProgress` loop in `execute_feed`, with the signature and deny rules above.
2. `REQ-CRON-0005`: the error table above, still seven variants.
3. `REQ-CRON-0001` and `REQ-RUN-0005`: no workspace dependency, no boundary-manifest edit, no `Cargo.lock` edit.
4. `NFR-RUN-0005`: the bridge test uses `tempfile::TempDir`, injects the compose program name (not PATH mutation), and does not call `std::env::set_var`.
5. ADR-RUN-0013 point 4 (including the `render_template` host-timeout sentence above), the new consequence, the `REQ-CRON-0003` sentence, and the hooks note.

## Acceptance criteria

Root: `boundary:sc-cron` and ADR-RUN-0013 as amended here.

- `cargo test -p sc-cron --features monty --test monty_host_bridge` on Rust 1.96.1 **must execute** (not skip): success path with real `sc-compose` on `PATH` (`render` with `--root`, `--mode file`, `--file`, `--var-file`, stdout body, `--output` omitted); deny paths for another function name and for `OsCall`; non-zero CLI → `Failed`. When `sc-compose` is absent, those cases **fail** the test. A separate case uses a non-installed program name → `Failed` detail names that program and repl is dropped. Skips are not an acceptance outcome.
- `cargo test -p sc-cron` with the default features does not compile the `monty` crates. `cargo tree -p sc-cron -e normal --prefix none` prints no crate whose name starts with `monty`.
- `boundaries/sc-cron/sc-cron.toml` has no `monty`, `monty-types`, or `sc-compose` entry added by this sprint.
- `docs/architecture.md` point 4 names `render_template`, the `sc-compose` spawn, and the host-timeout rule for time inside `render_template`. `docs/sc-cron/requirements.md` has the new `REQ-CRON-0003` sentence. `docs/sc-cron/hooks-sc-compose.md` calls `execute_feed` the production path.
- `rg -n 'std::thread::sleep|tokio::time::sleep|block_on' crates/sc-cron/src` exits 1.

## Required validation

- `cargo test -p sc-cron` exits 0 on Rust 1.94.1.
- `cargo test -p sc-cron --features monty` exits 0 on Rust 1.96.1.
- `cargo build --workspace` exits 0.
- `sc-lint lint sc-boundary --root . --config sc-lint.toml` exits 0.
- The `rg` gate above exits 1.

## Non-goals

No `monty-pool`. No sandbox. No second allowlisted function. No edit to
`sc_compose_bench.rs`, `Justfile`, `scripts/benchmark/`, or
`site/reports/`. No phase-close fix on `integrate/phase-b`.
