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
`monty_run::execute_feed`, which today calls `MontyRepl::call_function` and
does not resume host functions. That feed loop cannot be hoisted into a
contract sprint ahead of `b-9`: `b-9` already closed the Monty target as
`call_function` only. A new contract sprint that only names `render_template`
would be a thin sprint with one implementer (`b-10`) and one consumer
(`b-11`).

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

## Contract

`execute_feed` drives the call with `ReplProgress` instead of a single
`call_function`. The repl is still not dropped while a feed is on the stack
(`b-9`). Host work runs on the same blocking-pool thread as the feed
(ADR-RUN-0007, ADR-RUN-0011): no `block_on`, no sleep, no mutex held across
`.await`.

```rust
// crates/sc-cron/src/monty_host.rs
pub(crate) fn render_template(
    template: &str,
    context: &serde_json::Value,
) -> Result<String, String>;
```

Registration, before the feed:

```rust
let mut inputs = monty_types::NamedValues::new();
inputs.push(
    "render_template",
    monty_types::MontyObject::function("render_template".to_owned(), None),
);
```

Resume loop inside `execute_feed` (same shape the lab uses, now on the
production repl). A denied call returns `FeedDone` with `session: None` and
`FeedResult::Outcome(RunOutcome::Failed(...))`. It does not add a `CronError`
variant. `render_template`'s `Err` takes that same path. `Ok` resumes:

```rust
monty::ReplProgress::FunctionCall(call) if call.function_name == "render_template" => {
    let rendered = crate::monty_host::render_template(template, context)?;
    call.resume(
        monty_types::ExtFunctionResult::Return(monty_types::MontyObject::string(rendered)),
        monty_types::PrintWriter::Disabled,
    )?
}
```

`FunctionCall` for any other name, and `OsCall`, take the denied `FeedDone`
path. `ReplProgress::Complete` ends the feed. Any other progress value is
denied the same way.

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
| `sc-compose` is not on `PATH` before the first host call | `CronError::HookFailed` (`HOOK_FAILED`), `RunOutcome::Failed`, no repl left |
| CLI starts and exits non-zero, or Monty raises | `RunOutcome::Failed` after the feed returns |
| Feed returns a feed-time-limit error | `RunOutcome::TimedOut`, same checkpoint rule as `b-9` |

Wall time of the CLI counts toward the feed budget. Do not record `TimedOut`
while the CLI or the interpreter is still running.

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

`REQ-CRON-0003` gains one sentence for that call. `docs/sc-cron/architecture.md`
shows the signature above. `docs/sc-cron/hooks-sc-compose.md` states that the
production path ships in `execute_feed`, and that the standalone
`MontyRepl::feed_start` lab is not that path.

## Deliverables

1. `REQ-CRON-0003`: `monty_host::render_template` and the `ReplProgress` loop in `execute_feed`, with the signature and deny rules above.
2. `REQ-CRON-0005`: the error table above, still seven variants.
3. `REQ-CRON-0001` and `REQ-RUN-0005`: no workspace dependency, no boundary-manifest edit, no `Cargo.lock` edit.
4. `NFR-RUN-0005`: the bridge test uses `tempfile::TempDir` and does not set process environment variables.
5. ADR-RUN-0013 point 4, the new consequence, the `REQ-CRON-0003` sentence, and the hooks note.

## Acceptance criteria

Root: `boundary:sc-cron` and ADR-RUN-0013 as amended here.

- `cargo test -p sc-cron --features monty --test monty_host_bridge` on Rust 1.96.1: a Monty `cron_tick` that calls `render_template` on a string template gets the CLI stdout back; a call to any other name, and an `OsCall`, record `Failed` and leave no repl; a missing `sc-compose` records `HookFailed`; a non-zero CLI status records `Failed` without `HookFailed`.
- The test skips when `sc-compose` is not on `PATH`, except the missing-binary case, which uses a `PATH` that does not contain it.
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
