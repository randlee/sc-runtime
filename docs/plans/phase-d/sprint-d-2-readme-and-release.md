---
id: d-2
phase: d
status: planned
closure_type: integration
target_boundary: runtime-ecosystem-release-0-2-1
track: supply-chain
wave: 1
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
parallel_safe:
  - d-1
must_follow: []
publish_must_follow:
  - d-1
  - d-1b
publish_must_follow_rationale: Publish tail consumes merged `Cargo.lock` and manifest ranges from **d-1** (rusqlite) and **d-1b** (#133 rmcp/`>=` floors). README/#131 doc wave is not gated on supply-chain sprints; merge-forward d-1 and d-1b before publish tail.
recommended_agent: rust-developer
recommended_model: grok-4.7-high-fast
---

# Sprint d-2 — README truth and 0.2.1 release

## Goal

Integration closure for **#131** and **#132**: consumer READMEs and registry
truth can land in parallel with d-1; **0.2.1** publish and advisory clearance
run only after d-1 merges to `integrate/phase-d`.

Evidence (read-only): [registry-doc-audit.md](registry-doc-audit.md).

**Standard blurb** (insert on each published crate README):

```markdown
## Part of sc-runtime

This crate is part of [sc-runtime](https://github.com/randlee/sc-runtime), a Rust
scaffolding framework for JSON-first daemons, CLIs, generated stores, and optional
MCP. Publish order and template usage are documented in the repository README.
```

## Deliverables

| Deliverable | Serves |
| --- | --- |
| `README.md` — six crates, 0.2.x line, wizard, MSRV 1.94.1 | REQ-RUN-0702 |
| `crates/sc-config/README.md` — blurb; fix 0.1.0 banner | REQ-RUN-0702 |
| `crates/sc-transport/README.md` — blurb; fix 0.1.0 banner | REQ-RUN-0702 |
| `crates/sc-command/README.md` — blurb; fix 0.1.0 banner | REQ-RUN-0702 |
| `crates/sc-runtime/README.md` — blurb, template pointer, `test-util` / `instrument_router` (#131) | REQ-RUN-0702, NFR-RT-0001 |
| `crates/sc-cron/README.md` — blurb; ecosystem framing | REQ-RUN-0702 |
| `crates/sc-cron-commands/README.md` — blurb | REQ-RUN-0702 |
| `crates/sc-runtime-db-sqlite/README.md` — workspace-only; **no** crates.io publish claim | REQ-RUN-0702 |
| `crates/sc-cron-commands/Cargo.toml` — `readme = "README.md"` | REQ-RUN-0002 |
| `docs/releases/v0.2.1.md` | REQ-RUN-0702 |
| Workspace + published crate versions **0.2.1**; `Cargo.lock` after version bump (**publish tail**) | REQ-RUN-0002, REQ-RUN-0702 |
| sc-publish kit → crates.io **0.2.1** per `docs/development/sc-publish.md` (**publish tail**) | REQ-RUN-0702 |

## Acceptance criteria

### Doc wave (may start before d-1 merges)

1. **req:131** — `crates/sc-runtime/README.md` documents `instrument_router` behind `test-util`.
2. **req:RUN-0702-blurbs** — All six published crate READMEs include the standard blurb; root README lists six crates at 0.2.x (not four @ 0.1.0).
3. **req:RUN-0702-sqlite-readme** — `crates/sc-runtime-db-sqlite/README.md` does not claim crates.io publication.
4. **integration:readme-manifest** — `crates/sc-cron-commands/Cargo.toml` sets `readme = "README.md"`; `cargo package -p sc-cron-commands --list` includes `README.md`.

### Publish tail (after d-1 merge + merge-forward)

5. **req:132** — Published **0.2.1** clears RUSTSEC-2026-0459 on the documented consumer path (`cargo deny check advisories` recorded in `docs/releases/v0.2.1.md`).
6. **integration:pre-publish** — Kit dry-run or `cargo package --list` on each published crate shows README with scaffolding blurb in the artifact.
7. **integration:post-publish** — Each `https://crates.io/crates/<name>` @ **0.2.1** shows the blurb (crate page or `.crate` tarball).
8. **integration:workspace** — `just lint` and `just test` green on `integrate/phase-d` at **0.2.1** versions.

## Closure sequence

1. Land doc-wave deliverables on the sprint branch; run doc-wave ACs and sprint QA-1 on that head.
2. Merge-forward d-1 into this branch when d-1 development pushes (before publish tail edits).
3. Bump versions, regenerate `Cargo.lock`, run publish tail deliverables.
4. Merge to `integrate/phase-d`; authorized kit publish.
5. Record post-publish registry verification (AC 7) in sprint close-out / release notes.

## Owned paths

### Doc wave (disjoint from d-1)

- `README.md`
- `crates/sc-config/README.md`
- `crates/sc-transport/README.md`
- `crates/sc-command/README.md`
- `crates/sc-runtime/README.md`
- `crates/sc-runtime-db-sqlite/README.md`
- `crates/sc-cron/README.md`
- `crates/sc-cron-commands/README.md`
- `crates/sc-cron-commands/Cargo.toml` (`readme` key only)

### Publish tail (after d-1; may touch manifests d-1 edited — version fields only)

- `Cargo.toml` (`[workspace.package] version`)
- `Cargo.lock`
- `crates/sc-config/Cargo.toml`
- `crates/sc-transport/Cargo.toml`
- `crates/sc-command/Cargo.toml`
- `crates/sc-runtime/Cargo.toml`
- `crates/sc-cron/Cargo.toml`
- `crates/sc-cron-commands/Cargo.toml` (version field)
- `release/sc-publish-consumer-input.json`
- `template/Cargo.toml` (`workspace.dependencies` 0.2.1 pins if required)
- `docs/releases/v0.2.1.md`
- `docs/project-plan.md` (phase d status only)
- `docs/plans/phase-d/registry-doc-audit.md` (mark closed)

## This sprint does not close

- `rusqlite` / sqlx ruling code (**d-1**).
- sc-dolt planning.
- PyPI (empty in publish manifest).

## Validation

- Doc wave: `cargo package -p sc-cron-commands --list` (README present).
- Publish tail: `just lint && just test` on integration PR head; `cargo deny check advisories`; post-publish crates.io or tarball spot-check per AC 7.
