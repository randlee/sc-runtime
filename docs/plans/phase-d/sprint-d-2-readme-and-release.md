---
id: d-2
phase: d
status: planned
closure_type: integration
target_boundary: runtime-ecosystem-release-0-2-1
track: supply-chain
wave: 2
branch: sprint/d-2-readme-and-release
worktree: sprint/d-2-readme-and-release
pr_target: integrate/phase-d
requirements:
  - REQ-RUN-0002
  - REQ-RUN-0005
  - REQ-RUN-0702
  - NFR-RT-0001
adrs:
  - ADR-RUN-0012
  - ADR-RUN-0002
github_issues:
  - 131
  - 132
must_follow:
  - d-1
must_follow_rationale: d-2 publishes **0.2.1** only after d-1 merges rusqlite **0.41** `Cargo.lock` and crate graphs; the kit must not run on 0.2.0 advisory pins.
recommended_agent: rust-developer
recommended_model: grok-4.7-high-fast
---

# Sprint d-2 — README truth and 0.2.1 release

## Goal

Integration closure for issues **#131** and **#132**: consumer-facing docs,
registry truth, workspace validation, advisory smoke, and authorized **0.2.1**
publication after d-1 merges to `integrate/phase-d`.

## Documentation registry audit

Evidence: [registry-doc-audit.md](registry-doc-audit.md) (crates.io HTTP API and
0.2.0 `.crate` tarballs, 2026-11-11). Repo READMEs match those tarballs. Fix
the content gaps below; do not treat this as publish drift.

- **Six crates.io packages** at **0.2.0**, not yanked: `sc-config`,
  `sc-transport`, `sc-command`, `sc-runtime`, `sc-cron`, `sc-cron-commands`.
- **Zero PyPI.** `python_packages` and `python_distributions` are `[]`.
- **Root README is stale 0.1.0.** Four crate READMEs still open with a **0.1.0**
  publish banner.
- **`sc-cron-commands` Cargo.toml** needs `readme = "README.md"`.
- **`sc-runtime-db-sqlite`** README falsely claims crates.io publication.

**crates.io blurb** required on all six published READMEs:

```markdown
## Part of sc-runtime

This crate is part of [sc-runtime](https://github.com/randlee/sc-runtime), a Rust
scaffolding framework for JSON-first daemons, CLIs, generated stores, and optional
MCP. Publish order and template usage are documented in the repository README.
```

## Deliverables

| Deliverable | Serves |
| --- | --- |
| Root `README.md` — six crates, 0.2.x, wizard, MSRV 1.94.1 | REQ-RUN-0702 |
| Each published crate README — **Part of sc-runtime** scaffolding blurb | REQ-RUN-0702 |
| `sc-cron-commands/Cargo.toml` — `readme = "README.md"` | REQ-RUN-0002 |
| `sc-runtime` README — `test-util` / `instrument_router` (#131) | NFR-RT-0001 |
| `docs/releases/v0.2.1.md` | REQ-RUN-0702 |
| Workspace + published crate versions **0.2.1**; `Cargo.lock` after version bump | REQ-RUN-0002, REQ-RUN-0702 |
| sc-publish kit run → crates.io 0.2.1 per `docs/development/sc-publish.md` | REQ-RUN-0702 |

### README inventory

| Path |
| --- |
| `README.md` |
| `crates/sc-config/README.md` |
| `crates/sc-transport/README.md` |
| `crates/sc-command/README.md` |
| `crates/sc-runtime/README.md` |
| `crates/sc-runtime-db-sqlite/README.md` (workspace-only; no publish claim) |
| `crates/sc-cron/README.md` |
| `crates/sc-cron-commands/README.md` |

## Acceptance criteria

1. **req:131** — `sc-runtime` README documents `instrument_router` behind `test-util`.
2. **req:132** — Published **0.2.1** clears RUSTSEC-2026-0459 for the documented
   consumer path; `cargo deny check advisories` smoke recorded in release notes.
3. **crates.io blurb (REQ-RUN-0702)** — All six published crate READMEs include
   **Part of sc-runtime**; root README lists six crates at 0.2.x; `sc-runtime-db-sqlite`
   README has no crates.io publish claim.
4. **integration:crates.io** — After kit publish, each `https://crates.io/crates/<name>`
   at **0.2.1** renders README with the scaffolding sentence (crate page or
   `.crate` tarball; null API `readme` is not a failure).
5. **integration:workspace** — `just lint` and `just test` green on `integrate/phase-d`
   at **0.2.1** versions.
6. **integration:publish** — Kit publish completes; `release/sc-publish-consumer-input.json`
   intent unchanged except versions the kit rewrites.

## Owned paths

- `Cargo.toml` (`[workspace.package] version`)
- `Cargo.lock`
- `crates/sc-config/Cargo.toml`
- `crates/sc-transport/Cargo.toml`
- `crates/sc-command/Cargo.toml`
- `crates/sc-runtime/Cargo.toml`
- `crates/sc-cron/Cargo.toml`
- `crates/sc-cron-commands/Cargo.toml`
- `release/sc-publish-consumer-input.json`
- `template/Cargo.toml` (`workspace.dependencies` patch versions if kit requires 0.2.1)
- `README.md`
- `crates/sc-config/README.md`
- `crates/sc-transport/README.md`
- `crates/sc-command/README.md`
- `crates/sc-runtime/README.md`
- `crates/sc-runtime-db-sqlite/README.md`
- `crates/sc-cron/README.md`
- `crates/sc-cron-commands/README.md`
- `crates/sc-cron-commands/Cargo.toml` (`readme` key)
- `docs/releases/v0.2.1.md`
- `docs/project-plan.md` (phase d status only)
- `docs/plans/phase-d/registry-doc-audit.md` (mark closed)

## This sprint does not close

- `rusqlite` / sqlx ruling code (**d-1**).
- sc-dolt planning.
- PyPI (no packages in the publish manifest).

## Validation

- `just lint && just test` on integration PR head.
- Post-publish spot-check: six crates.io pages or tarballs @ 0.2.1 for scaffolding blurb.
- `cargo deny check advisories` smoke documented in `docs/releases/v0.2.1.md`.
