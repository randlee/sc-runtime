---
id: b-3
phase: b
status: planned
closure_type: boundary
target_boundary: sc-cron
track: cron
wave: 2
branch: sprint/b-3-sc-cron
worktree: sprint/b-3-sc-cron
requirements:
  - REQ-RUN-0001
  - REQ-RUN-0005
  - NFR-RUN-0005
  - NFR-RUN-0009
  - REQ-CRON-0001
  - REQ-CRON-0002
  - REQ-CRON-0003
  - REQ-CRON-0004
  - REQ-CRON-0005
adrs:
  - ADR-RUN-0013
  - ADR-RUN-0012
  - ADR-RUN-0002
  - ADR-RUN-0003
  - ADR-RUN-0005
  - ADR-RUN-0006
  - ADR-RUN-0007
  - ADR-RUN-0008
  - ADR-RUN-0011
owned_paths:
  - crates/sc-cron/
  - boundaries/sc-cron/
  - boundaries/sc-config/sc-config.toml
  - boundaries/sc-transport/sc-transport.toml
  - boundaries/sc-command/sc-command.toml
  - boundaries/sc-runtime/sc-runtime.toml
  - boundaries/sc-runtime-db-sqlite/sc-runtime-db-sqlite.toml
  - docs/sc-cron/
  - Cargo.lock
must_follow: [b-1]
blocked_by: beads-cache sprint b-8 (sprint/b-8-tick-python, REQ-CRON-0016, ADR-CRON-0009)
parallel_safe: [b-4]
recommended_agent: arch-ctm
recommended_model: terra
vertical_rationale: One ported crate. The expression parser and the scheduler live in the same files. b-1 lands the crates/* membership glob, which this sprint consumes and does not edit.
---

# Sprint b-3 — port sc-cron

## Blocker

This sprint copies `crates/sc-cron` after beads-cache sprint b-8 merges (`sprint/b-8-tick-python`, deliverable 6, REQ-CRON-0016, ADR-CRON-0009). At the start of the sprint, record the merged commit. If that tree drifts from the signatures in [architecture.md](../../sc-cron/architecture.md), re-review the sprint before the copy lands.

- Each schedule row chooses a Rust job or a Python module, so both can sit in the one table.
- `StopDecision::Stop` is removed. `Delete` replaces it.
- The crate's rustdoc and test fixtures carry no beads-cache names.
- The crate gains no beads-cache API, protocol, environment policy, or option.

beads-cache sprint b-9 edits comments, provenance, and tests, and does not bump the version. It does not hold this port. Maturin and PyO3 bindings stay out of this port.

`must_follow` is `b-1` because that sprint lands the `crates/*` membership glob and owns `Cargo.lock` in wave 1. This sprint does not edit `Cargo.toml` or the host boundary manifests. It updates `Cargo.lock` only to record `sc-cron`. No later sprint in this phase waits on this one.

## Goal

Port beads-cache `crates/sc-cron` into this workspace as specified in [architecture.md](../../sc-cron/architecture.md). It is an optional SQLite cron crate: one configuration table, a normal schedule string of five fields minute first, and a hook that is either a Rust function or `python3 <module> cron.tick` or `cron.stop` with one JSON object on stdin.

## Public types

Implement the types in [docs/sc-cron/architecture.md](../../sc-cron/architecture.md): `CronExpr`, `CronError`, `ScheduleSpec`, `HookTarget`, `Scheduler`, `Job`, `RunOutcome`, `StopDecision`. `HookTarget` is `Rust` or `Python { module }`.

Fallible public functions return `CronError`, the crate's one error enum. The variants are `BadExpression`, `InvalidSpec`, `AlreadyActive`, `Store`, and `Fenced`, with the codes in [ADR-RUN-0013](../../architecture.md) point 6.

## Deliverables

1. REQ-RUN-0001, REQ-CRON-0001: copy the crate into `crates/sc-cron`. The `crates/*` glob from `b-1` admits the directory. This sprint does not edit `Cargo.toml`. It updates `Cargo.lock` only to record `sc-cron`. After that update the glob resolves to the six library crates in the REQ-RUN-0001 table.
2. REQ-CRON-0002: a sc-cron schedule is a normal crontab string of five fields: minute, hour, day of month, month, and day of week. `*/5 * * * *` means every fifth minute. A six-field seconds-first string stays parseable only as the ADR-RUN-0013 point 2 compatibility note. A normal schedule string remains five fields, minute first.
3. REQ-CRON-0003: a `Rust` row calls `Job`. A `Python` row runs `python3 <module> cron.tick` or `python3 <module> cron.stop` and writes one JSON object to stdin. The object includes the schedule name and the row payload. It does not include ATM team or phase fields. The crate exposes no HTTP route, MCP tool, or binary.
4. REQ-CRON-0004: the crate creates one schedule table. A test writes a row without constructing `SqliteWriter`. The crate does not migrate the consumer's other tables.
5. REQ-RUN-0005: add `boundaries/sc-cron/sc-cron.toml` with `allowed_dependents = []` and forbidden edges to every other workspace crate. The crate exists at this sprint's close, so add `sc-cron` to the forbidden edges of `sc-config`, `sc-transport`, `sc-command`, `sc-runtime`, and `sc-runtime-db-sqlite`, and add `sc-runtime-db-sqlite` to the forbidden edges of `sc-cron`. Those manifest lines are the only edits to the `b-1` files.
6. NFR-RUN-0005: tests open the schedule file under a `tempfile::TempDir`. They do not set process environment variables.

## Acceptance criteria

Root: `boundary:sc-cron` and [ADR-RUN-0013](../../architecture.md).

- `cargo test -p sc-cron` exits 0, including the five-field parse, the Python stdin check, and a schedule write that does not construct `SqliteWriter`.
- `cargo tree -p sc-cron` shows no `sqlx` crate and no workspace crate.
- `StopDecision` is the enum in `docs/sc-cron/architecture.md`: `Continue`, `Disable(String)`, `Delete(String)`. `Stop` is absent.
- `cargo metadata --no-deps --format-version 1` lists `sc-cron` and the five crates from `b-1`. `Cargo.toml` is unchanged from `b-1`.

## Required validation

- `cargo test -p sc-cron` exits 0.
- `cargo build --workspace` exits 0.
- `sc-lint lint sc-boundary` exits 0 with the new manifest.
- `rg sc-cron template` prints nothing.
- `cargo tree -p sc-cron -e normal --prefix none` prints no line `sqlx` and no workspace crate.
- `rg -n 'std::thread::sleep|std::thread::sleep_until|tokio::time::sleep|tokio::time::sleep_until|Runtime::block_on|Handle::block_on|futures::executor::block_on' crates/sc-cron/src` exits 1.

## Non-goals

No beads CLI client. No Postgres store. No store-adaptor generator. No HTTP, MCP, or CLI API on this crate. No edit to `sc-lint.toml`. No edit to `Cargo.toml` or `crates/sc-runtime-db-sqlite/`. `Cargo.lock` changes only to record `sc-cron`.
