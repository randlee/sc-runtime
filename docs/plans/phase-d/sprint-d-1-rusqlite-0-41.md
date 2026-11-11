---
id: d-1
phase: d
status: planned
closure_type: boundary
target_boundary: BOUNDARY-sc-cron
track: supply-chain
wave: 1
branch: sprint/d-1-rusqlite-0-41
worktree: sprint/d-1-rusqlite-0-41
pr_target: integrate/phase-d
vertical_rationale: Issue #132 needs one lockfile and rusqlite 0.41 graph across BOUNDARY-sc-cron and BOUNDARY-sc-runtime-db-sqlite plus cron-consumer-proof; splitting crates would fork the advisory fix. d-2 README paths are parallel_safe and disjoint.
recommended_agent: rust-developer
recommended_model: grok-4.7-high-fast
requirements:
  - REQ-RUN-0002
  - REQ-RUN-0005
  - REQ-DBSQ-0001
  - REQ-CRON-0001
  - NFR-CRON-0001
adrs:
  - ADR-RUN-0003
  - ADR-RUN-0012
  - ADR-RUN-0013
github_issues:
  - 132
parallel_safe:
  - d-2
must_follow: []
---

# Sprint d-1 — rusqlite 0.41 advisory floor

## Goal

Implement [issue #132](https://github.com/randlee/sc-runtime/issues/132) through
boundary-rooted dependency and hook code changes across **BOUNDARY-sc-cron** and
**BOUNDARY-sc-runtime-db-sqlite**. Publish, README, workspace `just test`, and
RUSTSEC closure on the published **0.2.1** artifact stay in **d-2**.

## Frozen sqlx / rusqlite ruling (required)

This sprint **commits** the architecture change; it is not an optional note.

| Decision | Ruling |
| --- | --- |
| **Owned manifests** | `sc-cron`, `sc-runtime-db-sqlite`, and `tests/integration/cron-consumer` pin rusqlite **0.41** as below. |
| **sqlx 0.9 / template `store-sqlite`** | No sqlx major bump in Phase D. Same-process sharing of sqlx 0.9 sqlite (`libsqlite3-sys` &lt; 0.38) and the writer at rusqlite 0.41 (`libsqlite3-sys` ≥ 0.39) is **not supported** on the 0.2.1 line. |
| **`sqlx-resolve` fixture** | **Delete** `crates/sc-runtime-db-sqlite/tests/fixtures/sqlx-resolve/` and remove the gate from architecture text. Replace with ADR-RUN-0012 / crate-architecture text that states the limitation above and points at cron-only + writer integration tests. |
| **Generated daemon** | A project that enables **both** template SQLx store and this writer in one process needs a follow-on sqlx/rusqlite alignment sprint (out of Phase D). Phase D only proves **cron + writer** and maintainer integration paths. |

## Deliverables

| Deliverable | Serves |
| --- | --- |
| Exact `rusqlite` dependency lines (below) in `sc-cron`, `sc-runtime-db-sqlite`, `cron-consumer` | REQ-DBSQ-0001, REQ-CRON-0001 |
| Hook / interrupt call sites in `sqlite_scope.rs` and `consumer_contract.rs` compile and pass existing hook tests | REQ-CRON-0001, NFR-CRON-0001 |
| ADR-RUN-0012 amendment + `docs/sc-runtime-db-sqlite/architecture.md` replace 0.39 pin with ruling table above | REQ-DBSQ-0001, ADR-RUN-0012 |
| Delete `sqlx-resolve` fixture; update crate tests accordingly | REQ-DBSQ-0001 |
| `Cargo.lock` with single rusqlite 0.41 graph for owned crates | REQ-RUN-0002 |

### Exact `Cargo.toml` dependency lines

`crates/sc-cron/Cargo.toml`:

```toml
rusqlite = { version = "0.41", default-features = false, features = ["bundled", "hooks"] }
```

`crates/sc-runtime-db-sqlite/Cargo.toml`:

```toml
rusqlite = { version = "0.41", default-features = false, features = ["bundled"] }
```

`tests/integration/cron-consumer/Cargo.toml` — same line as `sc-cron` (direct or path dependency aligned to 0.41).

### Source touch set (not `trace_v2`)

Own and adjust only if the 0.41 bump requires API changes:

- `crates/sc-cron/src/sqlite_scope.rs` — `progress_handler`, `commit_hook`, `rollback_hook`, `busy_handler`, `InterruptHandle`
- `crates/sc-cron/src/consumer_contract.rs` — cooperative store / hook paths exercised by `rusqlite_hooks_and_interrupt_are_available` and store tests

## Acceptance criteria

1. **boundary:BOUNDARY-sc-cron** — `cargo test -p sc-cron` passes with workspace `rusqlite` at **0.41.0** (low end) and at the highest 0.41.x resolved in `Cargo.lock` after `cargo update -p rusqlite`.
2. **boundary:BOUNDARY-sc-runtime-db-sqlite** — `cargo test -p sc-runtime-db-sqlite` passes at both ends of that same pair; no `sqlx-resolve` fixture remains.
3. **boundary:cron-consumer** — `cargo test -p cron-consumer-proof` passes at the lockfile’s rusqlite 0.41 graph.
4. **ADR-RUN-0012** — Architecture text matches the ruling table; 0.39 / single-`libsqlite3-sys` with sqlx 0.9 language is removed.

## Owned paths

- `crates/sc-cron/Cargo.toml`
- `crates/sc-cron/src/**`
- `crates/sc-runtime-db-sqlite/Cargo.toml`
- `crates/sc-runtime-db-sqlite/src/**`
- `crates/sc-runtime-db-sqlite/tests/**` (excluding deleted fixture tree)
- `tests/integration/cron-consumer/**`
- `docs/sc-runtime-db-sqlite/architecture.md`
- `docs/architecture.md` (ADR-RUN-0012 section only)
- `Cargo.lock`

## This sprint does not close

- RUSTSEC / `cargo deny` smoke on the **published 0.2.1** consumer path (**d-2**).
- Workspace `just lint` and `just test` on `integrate/phase-d` (**d-2**).
- Any README, root or crate (**d-2**).
- crates.io publish, workspace/crate version bumps, kit publish (**d-2**).
- Issue #131 consumer docs (**d-2**).

## Validation

- `cargo test -p sc-cron`
- `cargo test -p sc-runtime-db-sqlite`
- `cargo test -p cron-consumer-proof`
- Lowest/highest rusqlite check: `cargo update -p rusqlite --precise 0.41.0 && cargo test -p sc-cron -p sc-runtime-db-sqlite` then restore lockfile high pin and repeat (document commands in PR body).
