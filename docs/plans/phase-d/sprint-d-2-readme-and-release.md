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
  - 133
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

Integration closure for **#131**, **#132**, and **#133**: consumer READMEs and registry
truth can land in parallel with d-1; **0.2.1** publish and advisory clearance
run only after both d-1 and d-1b merge to `integrate/phase-d` and their combined
dependency-resolution CI passes.

Release preparation changes sc-runtime package version fields, not third-party
dependency requirements. Compatible third-party updates remain permitted by
the qualified requirements supplied by d-1/d-1b; the registry proof targets
sc-runtime 0.2.1 without imposing new third-party exact pins or tighter caps.

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

### Publish tail (after both dependency sprints merge + merge-forward)

5. **req:132/133-advisories** — A consumer of the published **0.2.1** crates resolves fixed SQLite and rmcp dependencies and clears **RUSTSEC-2026-0459** and **RUSTSEC-2026-0462** using `cargo deny check advisories`. Match the affected beads-cache configuration; inspect registry package identities in metadata/lockfiles and exclude local path/patch substitutes. Record the targeted advisory results and supported configuration in `docs/releases/v0.2.1.md`.
6. **integration:pre-publish** — Inspect each package archive: its README contains the scaffolding blurb. Each additional dependency upper bound has a TOML source-manifest comment immediately beside the requirement, naming the concrete breaking change requiring that specific bound (ANC-019). Check `Cargo.toml.orig` when Cargo normalizes the packaged manifest. For example, the Monty `<1.6` comments identify Rust 1.98 versus the supported Rust 1.96.1 toolchain.
7. **integration:post-publish** — Each `https://crates.io/crates/<name>` @ **0.2.1** shows the blurb (crate page or `.crate` tarball).
8. **integration:workspace** — `just lint` and `just test` pass on the final **0.2.1** integration candidate. The existing production-tool dependency guard verifies that published crates' normal/build graphs contain neither sc-compose nor ATM crates, including transitive identities (ANC-012); external CLI invocation remains permitted.
9. **req:133-consumer-updates** — The released manifests contain the qualified ranges from both dependency sprints. Required minimum/latest CI passes on their combined integrated revision, including generated-template and separate consumer resolves. A registry consumer can update a compatible dependency within the supported range while keeping sc-runtime at 0.2.1.
10. **req:131-published-helper** — A registry consumer enables `sc-runtime/test-util` and compiles the documented `testing::instrument_router` usage against released 0.2.1, preserving PR131's public signature. README presence alone does not demonstrate that the release contains the helper.

## Closure sequence

1. Land doc-wave deliverables on the sprint branch; run doc-wave ACs and sprint QA-1 on that head.
2. Merge-forward the integration revision containing **both d-1 and d-1b** before publish-tail edits. Verify both dependency changes are present; a dependency label alone does not satisfy this handoff.
3. Bump versions, regenerate `Cargo.lock`, and run combined workspace, minimum/latest, package, and pre-publication checks on that exact candidate. A range or manifest change after qualification invalidates affected checks.
4. Merge to `integrate/phase-d`; authorized kit publish.
5. Run the released registry-consumer advisory, compatible-update, and helper checks; record post-publish verification in release notes. Close #132/#133 only after their released-consumer acceptance passes. A publish command or README check alone cannot close either security issue.

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

### Publish tail (after d-1 and d-1b; shared manifest edits are version fields only)

- `Cargo.toml` (`[workspace.package] version`)
- `Cargo.lock`
- `crates/sc-config/Cargo.toml`
- `crates/sc-transport/Cargo.toml`
- `crates/sc-command/Cargo.toml`
- `crates/sc-runtime/Cargo.toml`
- `crates/sc-cron/Cargo.toml`
- `crates/sc-cron-commands/Cargo.toml` (version field)
- `release/sc-publish-consumer-input.json`
- `template/Cargo.toml` (release-compatible sc-* 0.2.1 requirements)
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
