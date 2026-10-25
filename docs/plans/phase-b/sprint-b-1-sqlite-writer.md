---
id: b-1
phase: b
status: planned
closure_type: boundary
target_boundary: sc-runtime-db-sqlite
track: writer
wave: 1
branch: sprint/b-1-sqlite-writer
worktree: sprint/b-1-sqlite-writer
requirements:
  - REQ-RUN-0001
  - REQ-RUN-0005
  - REQ-RUN-0308
  - NFR-RUN-0005
  - NFR-RUN-0009
  - REQ-DBSQ-0001
  - REQ-DBSQ-0002
  - REQ-DBSQ-0003
  - REQ-DBSQ-0004
  - REQ-DBSQ-0005
adrs:
  - ADR-RUN-0012
  - ADR-RUN-0301
  - ADR-RUN-0011
  - ADR-RUN-0003
  - ADR-RUN-0005
  - ADR-RUN-0006
owned_paths:
  - crates/sc-runtime-db-sqlite/
  - boundaries/sc-runtime-db-sqlite/
  - boundaries/sc-runtime/sc-runtime.toml
  - boundaries/sc-config/sc-config.toml
  - boundaries/sc-transport/sc-transport.toml
  - boundaries/sc-command/sc-command.toml
  - Cargo.toml
  - Cargo.lock
  - docs/sc-runtime-db-sqlite/
  - docs/sc-runtime-db/
  - docs/sc-runtime-db-sqlx/
  - docs/sc-runtime-db-fsqlite/
must_follow: []
parallel_safe: [b-2]
recommended_agent: solar
recommended_model: sol
vertical_rationale: One new crate. The public types and the writer thread live in the same files, so a contract sprint would share every path with the implementation sprint.
---

# Sprint b-1 — rusqlite writer

## Goal

Ship `sc-runtime-db-sqlite` as specified in [architecture.md](../../sc-runtime-db-sqlite/architecture.md). Delete the withdrawn portable-layer docs. Forbid every existing library crate from depending on the new one, and the new one from depending on them.

## Public types

Implement `SqliteWriter`, `WriterConn`, `SqlValue`, `SqliteWriterError`, `CHANNEL_CAPACITY`, `BATCH_TIME_BUDGET`, `WRITE_DEADLINE`, and `SHUTDOWN_JOIN` with the signatures and variants in [docs/sc-runtime-db-sqlite/architecture.md](../../sc-runtime-db-sqlite/architecture.md). `execute` and `transaction` are inherent methods. `SqliteWriterError` implements `Display` and `std::error::Error`.

## Writer machine

Copy the atm-core writer with those types substituted for its `WriteOp`, `WriteOpResult`, `AtmError`, and `SharedDbTarget`. Do not copy task admission, task lifecycle, or the diagnostic lane.

## Boundaries

`boundaries/sc-runtime-db-sqlite/sc-runtime-db-sqlite.toml`:

- `owner_package = "sc-runtime-db-sqlite"`.
- `allowed_dependents = []`.
- `allowed_dependencies` lists `rusqlite` and the small crates the writer needs (`thiserror` if used). It does not list a workspace crate, `sqlx`, or `sc-observability-types`.
- `forbidden_edges`: `sc-runtime-db-sqlite -> sc-config`, `-> sc-transport`, `-> sc-command`, `-> sc-runtime`.
- `[public] facade` lists the public types above.

Add `sc-runtime-db-sqlite` to the forbidden edges of `sc-config`, `sc-transport`, `sc-command`, and `sc-runtime`. No file under `template/` names this crate.

## Tasks

1. REQ-RUN-0001: add the member to the workspace `Cargo.toml` and implement the types and the writer thread.
2. REQ-RUN-0005: add the boundary manifest and the four forbidden edges.
3. ADR-RUN-0012: delete `docs/sc-runtime-db/`, `docs/sc-runtime-db-sqlx/`, and `docs/sc-runtime-db-fsqlite/`. Keep `docs/sc-runtime-db-sqlite/` matching the code.
4. ADR-RUN-0012, NFR-RUN-0005, NFR-RUN-0009: tests for open, one `execute` round-trip, two concurrent `execute` calls with reply-after-commit, a `transaction` body that is not interleaved with another caller's statements, savepoint replay, shutdown join, `QueueFull`, `DeadlineExceeded`, and writer-thread panic. Each test opens the writer on a `tempfile::TempDir` path or on `:memory:`. No test uses a fixed path. Tests do not set process environment variables. A `#[cfg(test)]` constructor may shorten `WRITE_DEADLINE`; production `open` does not. Savepoint replay: when statement B in a concurrent batch A, B, C fails, the batch savepoint rolls back, A and C are replayed one statement at a time and their callers receive `Ok`, B's caller receives the error, and the database contains A's and C's rows. Shutdown: dropping the last `SqliteWriter` joins the writer thread within `SHUTDOWN_JOIN`. Queue full: with the writer held, filling `CHANNEL_CAPACITY` and sending one more call returns `Err(QueueFull)` before `WRITE_DEADLINE`. Deadline: a held writer returns `Err(DeadlineExceeded)` for a job that was accepted and not answered. Panic: a panic on the writer thread after enqueue and before commit makes every enqueued caller receive `Err(ChannelClosed)` and return.

## Required validation

- `cargo build --workspace` exits 0.
- `cargo test -p sc-runtime-db-sqlite` exits 0.
- `sc-lint lint sc-boundary` exits 0 with the new manifest.
- `rg sc-runtime-db-sqlite template` prints nothing.
- `cargo tree -p sc-runtime-db-sqlite -e normal --prefix none` prints no line `sqlx`.
- `rg -n 'thread::sleep|sleep_until|block_on' crates/sc-runtime-db-sqlite/src` exits 1.
- `rg -n yield_now crates/sc-runtime-db-sqlite/src` exits 0.

## Non-goals

No sqlx implementer. No facade method. No change to template `store-sqlite`. No edit to `sc-lint.toml` (sprint `b-2` owns that file).
