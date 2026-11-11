---
id: d-1b
phase: d
status: planned
closure_type: boundary
target_boundary: BOUNDARY-sc-command
track: supply-chain
wave: 1
branch: sprint/d-1b-dependency-ranges
worktree: sprint/d-1b-dependency-ranges
pr_target: integrate/phase-d
vertical_rationale: Issue #133 spans MCP (`rmcp`), OpenAPI (`utoipa`/`schemars`), and template daemon manifests; one lockfile pass avoids conflicting range edits. Does not touch rusqlite (d-1).
requirements:
  - REQ-RUN-0002
  - REQ-RUN-0005
  - NFR-RUN-0007
adrs:
  - ADR-RUN-0003
github_issues:
  - 133
parallel_safe:
  - d-1
  - d-2
must_follow: []
---

# Sprint d-1b — dependency range sweep (#133)

## Goal

Implement the **non-rusqlite** portions of [#133](https://github.com/randlee/sc-runtime/issues/133): replace tight pins with **`>=` tested floors**, prioritize **rmcp ≥ 3.9** (RUSTSEC-2026-0462), and add **minimal-versions + latest** validation hooks. Inventory: [issue-133-dependency-sweep.md](issue-133-dependency-sweep.md).

**Does not** change rusqlite/sqlx ruling (d-1). **Does not** publish (d-2).

## Blockers (do not claim “latest”)

- **sqlx 0.9** template stores stay on 0.9.x; no sqlx major in this sprint.
- **rusqlite** lines owned by d-1 only.
- Any dep whose latest major fails compile: document `>= floor, <next-major` and **comment why**.

## Deliverables

| Deliverable | Serves |
| --- | --- |
| `rmcp` `>=3.9.0` in `sc-runtime`, `sc-command`, `template/crates/daemon`, contract/integration test manifests | #133, RUSTSEC-2026-0462 |
| `monty` / `monty-types` `>=1.4.0, <1.6` on `sc-cron` optional deps | #133 — blocks `cargo update` → rustc 1.98 monty 1.6 on CI 1.96.1 |
| Replace `~3.8`, `=3.8.0` MCP pins with ranged specs + comments | #133 |
| `utoipa` / `schemars` optional deps: `>=` current floor, bounded upper if major breaks | #133 |
| Template `workspace.dependencies`: ranges where kit allows (see sweep table) | #133 point 4 |
| `Justfile` or documented CI recipe for `cargo +nightly update -Z minimal-versions` smoke | #133 point 3 |
| `Cargo.lock` updated for range graph (coordinate merge with d-1) | REQ-RUN-0002 |
| Remove unused direct `tracing` on `sc-runtime`; align boundary + NFR-RT-0003 / arch text | [sweep ELIMINATE](issue-133-dependency-sweep.md#eliminate) |

### rmcp target lines (initial)

```toml
rmcp = { version = ">=3.9.0, <4", default-features = false, features = [ /* unchanged */ ] }
```

## Acceptance criteria

1. **boundary:BOUNDARY-sc-command** — `cargo test -p sc-command` with rmcp at **3.9.0** floor and at highest `<4` resolved in lock.
2. **boundary:sc-runtime** — `cargo test -p sc-runtime` (MCP feature paths) at same pair.
3. **boundary:template-daemon** — template `cargo check -p daemon` (or repo-standard template recipe) with updated rmcp range.
4. **req:133-pins** — No remaining `~` or bare `=` on **third-party** deps in owned manifests except commented exceptions listed in PR.
5. **req:133-minimal** — Documented minimal-versions command succeeds on PR head (or records known nightly/toolchain prerequisite).
6. **req:133-eliminate-sc-runtime** — No direct `tracing` line on `sc-runtime`; `boundaries/sc-runtime/sc-runtime.toml` and NFR-RT-0003 match; `cargo test -p sc-runtime` passes. Transitive `tracing` via `axum` / `rmcp` may remain.

## Owned paths

- `crates/sc-command/Cargo.toml`
- `crates/sc-runtime/Cargo.toml`
- `boundaries/sc-runtime/sc-runtime.toml`
- `docs/sc-runtime/requirements.md` (NFR-RT-0003 / dependency exact-set only)
- `docs/sc-runtime/architecture.md` (matching dependency sentence only)
- `crates/sc-cron/Cargo.toml` (optional `monty` / `monty-types` caps only — **not** rusqlite or `tracing`; see sweep §3)
- `crates/sc-cron-commands/Cargo.toml` (utoipa/schemars only)
- `template/Cargo.toml`
- `template/crates/daemon/Cargo.toml`
- `tests/contracts/phase-c/application-control/Cargo.toml`
- `tests/integration/cron-consumer/schema/Cargo.toml`
- `Justfile` (minimal-versions recipe only)
- `Cargo.lock`

## This sprint does not close

- rusqlite / sqlx ADR (**d-1**).
- README / crates.io (**d-2**).
- **0.2.1** version field bumps (**d-2** publish tail).

## Validation

- `cargo test -p sc-command -p sc-runtime`
- Minimal-versions + latest commands from deliverable recipe (PR body cites exact invocations).
