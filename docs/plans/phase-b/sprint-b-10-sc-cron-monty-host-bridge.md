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

Named values for the snippet bind `render_template` via
`MontyObject::function` and `__hook` via the existing `json_to_monty` value.

```rust
// crates/sc-cron/src/monty_host.rs — program name is injectable for tests.
pub(crate) fn render_template(
    template: &str,
    context: &serde_json::Value,
    compose_program: &str,
) -> Result<String, String>;
```

Production passes `compose_program = "sc-compose"`. The missing-binary test
passes a name that is not installed. No `std::env::set_var` and no PATH
rewrite (`NFR-RUN-0005`).

Progress loop inside `execute_feed` (after taking the session's repl by value):

```rust
// feed_start consumes the repl; on Complete, put repl back in the session.
match repl.feed_start(snippet, &inputs)? {
    monty::ReplProgress::FunctionCall(call) if call.function_name == "render_template" => {
        // Positional render_template(template: str, context: dict).
        let template_ref = call.args.arg(0).ok_or("render_template: missing arg0")?;
        let template = template_ref.as_str().ok_or("render_template: arg0 must be str")?;
        let context_ref = call.args.arg(1).ok_or("render_template: missing arg1")?;
        let context = monty_host::monty_value_to_json(context_ref)?; // pairs() on dicts; inverse of json_to_monty
        let rendered = monty_host::render_template(template, &context, compose_program)?;
        call.resume(
            monty_types::ExtFunctionResult::Return(monty_types::MontyObject::string(rendered)),
            monty_types::PrintWriter::Disabled,
        )?
    }
    monty::ReplProgress::Complete { repl, value } => { /* parse value; store repl in session */ }
    // Any other FunctionCall name, OsCall, or unknown progress → FeedDone Failed, session None
}
```

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

| Condition | Result |
|---|---|
| Inside `render_template`, the configured compose program cannot be started (missing binary) | `RunOutcome::Failed` whose detail is `CronError::HookFailed(...).recorded()`; repl dropped. Feeds that never call `render_template` still run when the binary is absent. |
| CLI starts and exits non-zero, or Monty raises | `RunOutcome::Failed` (no `HookFailed`); repl dropped |
| After the CLI returns, host-measured elapsed time exceeds the row's remaining budget | `RunOutcome::TimedOut`; same checkpoint rule as `b-9`. Outcome is recorded only after `Command` returns. |
| Monty feed-time-limit while the VM is executing (not suspended on host) | `RunOutcome::TimedOut` as today |

Monty's `ResourceTracker` pauses `max_feed_duration` while suspended on a host
`FunctionCall`; CLI wall time is **not** on the VM feed clock. The host times
the child with `Instant`. If the child never exits, the blocking-pool thread
stays occupied (same accepted stall as a Monty hook that never reaches another
checkpoint); there is no second kill while the child is alive.

### Dependency (`REQ-CRON-0001`, `REQ-RUN-0005`)

Feature `monty` stays optional. No new workspace crate and no dependency on
another workspace crate. Do not add `monty`, `monty-types`, or `sc-compose` to
`boundaries/sc-cron` `allowed_dependencies`: sc-lint rejects non-workspace
names there (observed on the `b-9` close). `Cargo.toml` and `Cargo.lock` stay
as `b-9` left them. Default CI (Rust 1.94.1) does not enable `monty` and does
not gain an unconditional Monty dev-dependency. Bridge tests are
`cfg(feature = "monty")` and run on Rust 1.96.1.

## ADR-RUN-0013 amendment

On the `b-9` text of point 4, add: under feature `monty`, a hook may call the
host function `render_template(template, context)`; sc-cron resumes it by
spawning `sc-compose` on `PATH` against a temp root the hook cannot choose;
`OsCall` and any other function name fail the feed; the tick does not start
CPython. Add a consequence naming sprint `b-10`.

`REQ-CRON-0003` gains one sentence for that call. Amend the `b-9` sentence that
`HookFailed` is only a failure to **start** the hook: a missing compose binary
during `render_template` is after the hook has started and still uses
`HookFailed` in the recorded detail. `docs/sc-cron/architecture.md` shows the
signature above. `docs/sc-cron/hooks-sc-compose.md` states that production
ships in `execute_feed` (which may call `feed_start` internally) and that the
standalone lab `MontyRepl::feed_start` in the bench is not the production path.

## Deliverables

1. `REQ-CRON-0003`: `monty_host::render_template` and the `ReplProgress` loop in `execute_feed`, with the signature and deny rules above.
2. `REQ-CRON-0005`: the error table above, still seven variants.
3. `REQ-CRON-0001` and `REQ-RUN-0005`: no workspace dependency, no boundary-manifest edit, no `Cargo.lock` edit.
4. `NFR-RUN-0005`: the bridge test uses `tempfile::TempDir`, injects the compose program name (not PATH mutation), and does not call `std::env::set_var`.
5. ADR-RUN-0013 point 4, the new consequence, the `REQ-CRON-0003` sentence, and the hooks note.

## Acceptance criteria

Root: `boundary:sc-cron` and ADR-RUN-0013 as amended here.

- `cargo test -p sc-cron --features monty --test monty_host_bridge` on Rust 1.96.1 **must execute** (not skip): success path with real `sc-compose` on `PATH` (`render` with `--root`, `--mode file`, `--file`, `--var-file`, stdout body, `--output` omitted); deny paths for another function name and for `OsCall`; non-zero CLI → `Failed` without `HookFailed`. When `sc-compose` is absent, those cases **fail** the test. A separate case uses a non-installed program name for the missing-binary path → `HookFailed` and no repl. Skips are not an acceptance outcome.
- `cargo test -p sc-cron` with the default features does not compile the `monty` crates. `cargo tree -p sc-cron -e normal --prefix none` prints no crate whose name starts with `monty`.
- `boundaries/sc-cron/sc-cron.toml` has no `monty`, `monty-types`, or `sc-compose` entry added by this sprint.
- `docs/architecture.md` point 4 names `render_template` and the `sc-compose` spawn. `docs/sc-cron/requirements.md` has the new `REQ-CRON-0003` sentence. `docs/sc-cron/hooks-sc-compose.md` calls `execute_feed` the production path.
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
