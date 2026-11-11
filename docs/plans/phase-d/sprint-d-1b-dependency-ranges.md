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

Published crates and generated-template manifests must permit every newer
Cargo SemVer-compatible dependency release within the supported API, native-link,
and Rust-toolchain contract. Consumers can select those releases with
`cargo update` without a new sc-runtime release (ANC-016). A narrower requirement
is permitted only for a demonstrated API, native-link, or supported-toolchain
incompatibility, recorded beside the owning manifest requirement.

**Does not** change rusqlite/sqlx ruling (d-1). **Does not** publish (d-2).

## Blockers (do not claim “latest”)

- **sqlx 0.9** template stores stay on 0.9.x; no sqlx major in this sprint.
- **rusqlite** lines owned by d-1 only.
- Any extra restriction requires its concrete incompatibility comment in the owning manifest; an arbitrary pin or a PR-only explanation does not qualify.

## Deliverables

| Deliverable | Serves |
| --- | --- |
| `rmcp` `"3.9.0"` (normal Cargo-compatible 3.x range) in `sc-runtime`, `sc-command`, `template/crates/daemon`, contract/integration test manifests | #133, RUSTSEC-2026-0462 |
| `monty` / `monty-types` `>=1.4.0, <1.6` on `sc-cron` optional deps | #133 — blocks `cargo update` → rustc 1.98 monty 1.6 on CI 1.96.1 |
| Replace `~3.8`, `=3.8.0` MCP pins with ranged specs + comments | #133 |
| `utoipa` / `schemars` optional deps: `>=` current floor, bounded upper if major breaks | #133 |
| Template `workspace.dependencies`: tested floors with the same compatible-update policy as the published libraries | #133 point 4 |
| Executable minimum/latest recipes wired into existing crate and generated-template CI, including separate consumer lockfiles | #133 point 3 |
| `Cargo.lock` updated for range graph (coordinate merge with d-1) | REQ-RUN-0002 |
| Remove unused direct `tracing` on `sc-runtime`; align boundary + NFR-RT-0003 / arch text | [sweep ELIMINATE](issue-133-dependency-sweep.md#eliminate) |
| Qualify the existing generation CLI setup and skill version checks at minimum sc-compose 1.8.0; keep compatible consumer overrides independent of the CI pin | REQ-RUN-0005, ANC-003, ANC-005, ANC-015, ANC-020 |

### rmcp target lines (initial)

```toml
rmcp = { version = "3.9.0", default-features = false, features = [ /* unchanged */ ] }
```

Every additional upper bound has its own TOML comment beside the dependency
requirement, identifying the concrete breaking change that requires that bound
(ANC-019). For the actual Monty toolchain restriction, preserve `optional = true`:

```toml
# Monty 1.6 requires Rust 1.98; the supported Monty CI toolchain is Rust 1.96.1.
monty = { version = ">=1.4.0, <1.6", optional = true }
# monty-types 1.6 requires Rust 1.98; Monty CI supports Rust 1.96.1.
monty-types = { version = ">=1.4.0, <1.6", optional = true }
```

`"3.9.0"` is Cargo's normal compatible range (`>=3.9.0, <4`), not an exact
pin. The floor is 3.9.0; the default compatibility boundary allows non-breaking
updates. Additional restrictions require an observed incompatibility and a
manifest comment. A broader range across compatibility lines requires explicit
API, native-link, and supported-toolchain validation. Resolve this policy with
the older blanket-`>=` inventory wording before claiming #133 complete.

## Acceptance criteria

1. **boundary:BOUNDARY-sc-command** — `cargo test -p sc-command` with rmcp at **3.9.0** floor and at highest `<4` resolved in lock.
2. **boundary:sc-runtime** — `cargo test -p sc-runtime` (MCP feature paths) at same pair.
3. **boundary:template-daemon** — Generated-template CI runs the affected daemon/MCP tests with both minimum and latest allowed dependency sets. Exercise affected store/schema features through their existing generated fixtures; a daemon-only compile does not qualify other changed template floors.
4. **req:133-pins** — Every third-party dependency row has an owner and a final compatible range across all repository manifests, including templates and separate test workspaces. An exact pin, tilde restriction, or tighter upper bound needs a demonstrated API/native-link/toolchain incompatibility explained beside the owning manifest requirement. Intentional negative compatibility fixtures retain their incompatible test identity with its explicit in-manifest test-purpose comment; they do not restrict production requirements.
5. **req:133-minimal** — Required CI executes minimum and latest resolves and affected tests on the final integrated manifests. Install the named nightly prerequisite in CI; merely documenting a recipe or missing prerequisite does not pass. Start with full minimal-versions; a named transitive-floor failure may justify direct-minimal-versions, with the limitation recorded. Verify actual resolved direct floors in that fallback.
6. **req:133-eliminate-sc-runtime** — No direct `tracing` line on `sc-runtime`; `boundaries/sc-runtime/sc-runtime.toml` and NFR-RT-0003 match; `cargo test -p sc-runtime` passes. Transitive `tracing` via `axum` / `rmcp` may remain.
7. **integration:generation-tool** — CI declares sc-compose CLI minimum_version **1.8.0** and installs its selected GitHub Release archive with checksum verification. Generation succeeds with sc-compose **1.8.0** when the commands it uses are compatible; `SC_COMPOSE` can select a compatible newer CLI independently of CI's reproducible pin.
8. **integration:skill-version-checks** — Repository skills invoking sc-compose accept installed versions equal to or above their declared minimum and reject lower or malformed versions. Use controlled version-output inputs plus the real report/generation command; an exact-version equality check fails this criterion.

## Owned paths

- `Cargo.toml` (`workspace.dependencies` only; release version belongs to d-2)
- `crates/sc-config/Cargo.toml` (third-party dependency requirements only)
- `crates/sc-transport/Cargo.toml` (third-party dependency requirements only)
- `crates/sc-command/Cargo.toml`
- `crates/sc-runtime/Cargo.toml`
- `boundaries/sc-runtime/sc-runtime.toml`
- `docs/sc-runtime/requirements.md` (NFR-RT-0003 / dependency exact-set only)
- `docs/sc-runtime/architecture.md` (matching dependency sentence only)
- `crates/sc-cron/Cargo.toml` (all non-rusqlite third-party ranges, including optional Monty and Windows dev dependencies; the optional tracing removal remains d-1)
- `crates/sc-runtime-db-sqlite/Cargo.toml` (non-rusqlite third-party ranges only)
- `crates/sc-cron-commands/Cargo.toml` (dependency ranges only; d-2 owns the readme key and release versions)
- `template/Cargo.toml` and `template/crates/**/Cargo.toml` (third-party ranges; d-2 owns the released sc-* version strings)
- `tests/**/Cargo.toml` and existing compatibility-fixture manifests (third-party ranges; d-1 owns rusqlite)
- `Justfile` (minimum/latest recipes)
- `.github/workflows/crates.yml` and `.github/workflows/template.yml` (required dependency-resolution validation)
- Root and affected separate consumer/fixture `Cargo.lock` files; generated-app locks are CI outputs
- `.sc/generation-tools.json`, `.github/actions/rust-build-cache/action.yml`, and `scripts/new_project.py` (CLI selection/install qualification only)
- `.claude/skills/plan-anchoring/scripts/change_report.py`, `.codex/skills/plan-anchoring/scripts/change_report.py`, `.cursor/skills/plan-anchoring/scripts/change_report.py` (minimum-version checks and report invocation only)

Use separate checkouts or restore locks between endpoint runs. Root `cargo
update` does not update nested workspaces. The recipes explicitly enumerate the
root, cron-consumer, schema consumer, application-control, and affected fixture
resolves. Preserve the negative utoipa-router test's purpose at both endpoints.
Run platform-specific dependencies on the existing Windows jobs and Monty on its
supported toolchain; a default-feature Linux pass does not qualify those floors.

Manifest sections shared with d-1/d-2 are coordinated edits in isolated sprint
branches. Merge-forward the latest integration state before updating locks;
integration has one lockfile writer at a time. Final combined CI runs after
both dependency sprints land, before the d-2 publish tail.

## This sprint does not close

- rusqlite / sqlx ADR (**d-1**).
- README / crates.io (**d-2**).
- **0.2.1** version field bumps (**d-2** publish tail).

## Validation

- `cargo test -p sc-command -p sc-runtime`
- Minimal-versions + latest commands from deliverable recipe (PR body cites exact invocations).
