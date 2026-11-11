---
id: d-2
phase: d
status: planned
closure_type: integration
target_boundary: runtime-ecosystem-release-0-2-1
track: supply-chain
wave: 2
branch: sprint/d-2-readme-and-release
worktree: ../sc-runtime-worktrees/sprint/d-2-readme-and-release
pr_target: integrate/phase-d
requirements:
  - REQ-RUN-0002
  - REQ-RUN-0702
  - REQ-RUN-0701
  - REQ-RUN-0005
  - NFR-RT-0001
adrs:
  - ADR-RT-0005
  - ADR-RUN-0012
github_issues:
  - 131
  - 132
must_follow:
  - d-1
recommended_agent: rust-developer
recommended_model: grok-4.7-high-fast
---

# Sprint d-2 — README truth and 0.2.1 release

## Goal

Integration closure for issues **#131** and **#132**: consumer-facing docs,
registry truth, and authorized **0.2.1** publication after d-1 merges to
`integrate/phase-d`.

## Documentation registry audit

Evidence: [registry-doc-audit.md](registry-doc-audit.md) (crates.io HTTP API and
0.2.0 `.crate` tarballs, 2026-11-11). Repo READMEs match those tarballs. Fix
the content gaps below; do not treat this as publish drift.

- **Six crates.io packages** at **0.2.0**, not yanked: `sc-config`,
  `sc-transport`, `sc-command`, `sc-runtime`, `sc-cron`, `sc-cron-commands`.
  `description` matches each `Cargo.toml`. None of those lines, and no README,
  says the crate is part of sc-runtime, a Rust scaffolding framework.
- **Zero PyPI.** `python_packages` and `python_distributions` are `[]`. No
  PyPI docs or publish work.
- **Root README is stale 0.1.0.** It still describes four crates and says
  crates.io publication needs authorization. `sc-config`, `sc-transport`,
  `sc-command`, and `sc-runtime` READMEs still open with a **0.1.0** publish
  banner. `sc-cron-commands` only mentions selecting `cron=true` when
  generating from sc-runtime (partial). `sc-cron` has no ecosystem framing.
- **`sc-cron-commands` Cargo.toml has no `readme` key.** The 0.2.0 tarball
  still contains `README.md` via Cargo default discovery. Set
  `readme = "README.md"` so the manifest matches the other five published
  crates.
- **`sc-runtime-db-sqlite`** is not on crates.io (`publish = false`; API:
  crate does not exist). Its README claims publication at 0.1.0.
- Version JSON `readme` is null, and `static.crates.io/readmes/...` 404s.
  README truth is the crate page or the `.crate` tarball.

**crates.io blurb** required on all six published READMEs (after the title,
or in place of the stale publish banner):

```markdown
## Part of sc-runtime

This crate is part of [sc-runtime](https://github.com/randlee/sc-runtime), a Rust
scaffolding framework for JSON-first daemons, CLIs, generated stores, and optional
MCP. Publish order and template usage are documented in the repository README.
```

The `sc-runtime` README also points at the template / `just new` path (it is
the kit `readme_dependency_crate`). An optional `description` suffix, "Part of
the sc-runtime Rust scaffolding framework.", does not replace the README
section.

## Deliverables

| Deliverable | Serves |
| --- | --- |
| Root `README.md` — six crates, 0.2.x, wizard, MSRV 1.94.1 | REQ-RUN-0701 |
| Each published crate README — **Part of sc-runtime** scaffolding blurb | REQ-RUN-0702 |
| `sc-cron-commands/Cargo.toml` — `readme = "README.md"` | REQ-RUN-0002 |
| `sc-runtime` README — `test-util` / `instrument_router` (#131) | NFR-RT-0001 |
| `docs/releases/v0.2.1.md` | REQ-RUN-0702 |
| sc-publish kit run → crates.io 0.2.1 | REQ-RUN-0702, #132 done-when |
| Close GitHub #131, #132 | — |

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
2. **req:132** — Published 0.2.1 clears RUSTSEC-2026-0459 for a documented consumer path.
3. **crates.io blurb (REQ-RUN-0702)** — Each of `sc-config`, `sc-transport`,
   `sc-command`, `sc-runtime`, `sc-cron`, and `sc-cron-commands` README includes
   the **Part of sc-runtime** section from Documentation registry audit and
   names sc-runtime as a Rust scaffolding framework. The `sc-runtime` README
   also points at template generation. Root README lists those six crates at
   0.2.x and does not present four crates or 0.1.0 as the current release.
   The `sc-runtime-db-sqlite` README does not claim crates.io publication.
4. **crates.io page** — `crates/sc-cron-commands/Cargo.toml` sets
   `readme = "README.md"`. After authorized **0.2.1** publication, each
   `https://crates.io/crates/<name>` page for that version renders a README
   that includes the scaffolding sentence. Judge the crate page or the
   published `.crate` tarball. A null API `readme` field is not a failure.
   `sc-runtime-db-sqlite` stays unpublished. `python_packages` stays empty.
5. **integration:publish** — Kit publish completes; `release/sc-publish-consumer-input.json` unchanged in intent except versions produced by kit.
6. **integration:qa** — Sprint QA-1 PASS, `reviewer_spawn_gate: pass`, required CI green.

## Owned paths

- `README.md`
- `crates/sc-config/README.md`
- `crates/sc-transport/README.md`
- `crates/sc-command/README.md`
- `crates/sc-runtime/README.md`
- `crates/sc-runtime-db-sqlite/README.md`
- `crates/sc-cron/README.md`
- `crates/sc-cron-commands/README.md`
- `crates/sc-cron-commands/Cargo.toml`
- `docs/releases/v0.2.1.md`
- `docs/project-plan.md` (phase d status only)
- `docs/plans/phase-d/registry-doc-audit.md` (mark closed)
- Release kit outputs per `docs/development/sc-publish.md` (no manual registry bypass)

## This sprint does not close

- `rusqlite` code changes (**d-1**).
- sc-dolt planning.
- PyPI (no packages in the publish manifest).

## Validation

- Post-0.2.1, each of the six crates.io crate pages (or the `.crate` tarball
  README) includes the Part of sc-runtime sentence.
- `cargo deny` smoke documented in release notes.
