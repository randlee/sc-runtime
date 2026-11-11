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
| **Owned manifests** | `sc-cron`, `sc-runtime-db-sqlite`, and `tests/integration/cron-consumer` declare the qualified rusqlite compatibility range below; no exact or tilde pin. |
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
| `Cargo.lock` at the qualified high endpoint, with one native SQLite identity; the temporary low-end CI lock does not restrict consumer manifests | REQ-RUN-0002 |
| **Optional rider:** drop unused direct `tracing` on `sc-cron` (boundary + NFR-CRON-0001 text) in the same `Cargo.toml` edit | [sweep ELIMINATE](issue-133-dependency-sweep.md#eliminate) |

### Exact `Cargo.toml` dependency lines

`crates/sc-cron/Cargo.toml`:

```toml
rusqlite = { version = ">=0.41.0, <0.43", default-features = false, features = ["bundled", "hooks"] }
```

`crates/sc-runtime-db-sqlite/Cargo.toml`:

```toml
rusqlite = { version = ">=0.41.0, <0.43", default-features = false, features = ["bundled"] }
```

`tests/integration/cron-consumer/Cargo.toml` — same floor as `sc-cron`.

**Compatible consumer updates (ANC-016):** 0.41.x is the Cargo-compatible line
for a 0.41.0 minimum; its compatible patch upgrades remain allowed without
another sc-runtime release. Version 0.42 is a separate Cargo compatibility
line, qualified independently through hook, writer, and consumer tests. The
two-line range is broader than the default 0.41 range and permits compatible
patches within both tested lines. If 0.42 is incompatible, the normal
`"0.41.0"` requirement still permits every Cargo-compatible 0.41.x upgrade.

No extra upper bound narrows the normal Cargo-compatible lines in this sprint.
The `<0.43` delimiter ends the separately qualified 0.42 line. If an additional
tighter cap becomes necessary, a TOML comment immediately beside that version
requirement in every owning manifest must identify the concrete API,
native-link, or toolchain break requiring that specific cap (ANC-019).

### Source touch set (not `trace_v2`)

Adjust these call sites when either supported rusqlite line requires it:

- `crates/sc-cron/src/sqlite_scope.rs` — `progress_handler`, `commit_hook`, `rollback_hook`, `busy_handler`, `InterruptHandle`
- `crates/sc-cron/src/consumer_contract.rs` — cooperative store / hook paths exercised by `rusqlite_hooks_and_interrupt_are_available` and store tests

## Acceptance criteria

1. **boundary:BOUNDARY-sc-cron** — `cargo test -p sc-cron` passes at rusqlite **0.41.0** and the latest version admitted by the final manifest. The inventory snapshot's high endpoint is **0.42.0** when supported; both separately qualified lines retain their compatible patch updates. Record the tested compatibility envelope and final manifest requirement before closure.
2. **boundary:BOUNDARY-sc-runtime-db-sqlite** — `cargo test -p sc-runtime-db-sqlite` passes at both ends of that same pair; no `sqlx-resolve` fixture remains.
3. **boundary:cron-consumer** — The separate cron-consumer lockfile resolves and `cargo test -p cron-consumer-proof` passes at both the same minimum and latest allowed endpoints. Each endpoint has one linkable libsqlite3-sys identity; root-workspace success alone does not qualify the consumer resolve.
4. **ADR-RUN-0012** — Architecture text matches the ruling table; 0.39 / single-`libsqlite3-sys` with sqlx 0.9 language is removed.
5. **req:133-eliminate-sc-cron** (optional) — With the rider shipped: `cargo tree -p sc-cron --edges normal -i tracing` shows no `sc-cron` parent; boundary and NFR text match the manifest.

## Owned paths

- `crates/sc-cron/Cargo.toml`
- `crates/sc-cron/src/**`
- `crates/sc-runtime-db-sqlite/Cargo.toml`
- `crates/sc-runtime-db-sqlite/src/**`
- `crates/sc-runtime-db-sqlite/tests/**` (excluding deleted fixture tree)
- `tests/integration/cron-consumer/**`
- `docs/sc-runtime-db-sqlite/architecture.md`
- `docs/architecture.md` (ADR-RUN-0012 section only)
- `boundaries/sc-cron/sc-cron.toml` (rider only)
- `docs/sc-cron/requirements.md` (NFR-CRON-0001 rider only)
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
- Lowest/highest rusqlite check: resolve the root and cron-consumer workspaces separately to 0.41.0, run their affected tests, then resolve each to the latest allowed endpoint and repeat. CI wiring is owned by d-1b. Preserve the tested versions and final range in concise validation evidence; restore the high lockfiles after low-end checks.
