---
id: c-4
phase: c
status: planned
closure_type: integration
target_boundary: published-runtime-template-consumer
track: release
wave: 3
branch: sprint/c-4-runtime-ecosystem-release
worktree: sprint/c-4-runtime-ecosystem-release
requirements:
  - REQ-RUN-0002
  - REQ-RUN-0003
  - REQ-RUN-0301
  - REQ-RUN-0503
  - REQ-RUN-0702
adrs:
  - ADR-RUN-0002
  - ADR-RUN-0017
  - ADR-RUN-0203
  - ADR-RUN-0401
  - ADR-RUN-0402
owned_paths:
  - install.py
  - README.sc-publish.md
  - .github/workflows/*release*.yml
  - .github/workflows/*publish*.yml
  - .github/actions/
  - .github/scripts/
  - .claude/agents/publisher.md
  - .claude/skills/prerelease/
  - .codex/agents/publisher.md
  - .codex/skills/prerelease/
  - .cursor/commands/cursor-publish.md
  - .cursor/skills/prerelease/
  - release/
  - tests/integration/release/test_published_runtime_ecosystem.py
must_follow: [c-3]
parallel_safe: []
recommended_agent: solar
recommended_model: sol
vertical_rationale: Registry visibility can only be verified after the exact public package is authorized and published. The release workflow, visibility wait, and no-patch consumer are a single production integration closure and cannot be honestly claimed by a prepublication implementation sprint.
---

# Sprint c-4 — runtime ecosystem authorized release

Dependency execution: consume `c-3`'s prepared 0.2.0 package and composition
proof by merging each parent development push before every development/fix
round. Parent QA is not a start barrier; the parent PR merges before this
PR completes. Publication retains the release authorization gate below.

## Goal

Close the production release contract left deliberately open by `c-3`:
publish the approved `sc-runtime 0.2.0` only with release authorization, then
prove a newly generated consumer resolves it from crates.io without a checkout
patch.

## Deliverables

1. Use the existing authorization and publication workflow for `sc-runtime
   0.2.0` and every affected package frozen by c-1. Change only canonical
   `release/sc-publish-consumer-input.json` and the qualified exact
   `release/sc-publish-pin.toml`, then rerun its installer. Include its actual
   destination inventory in this sprint's release-owned paths; any additional
   generated destination requires an explicit ownership update before writing.
   Do not hand-edit vendored workflows/scripts. Use the c-1 proven selective
   contract to publish sc-command 0.2.0 before dependent cron/runtime packages,
   checking each exact version's registry visibility before its dependent.
   Unaffected packages stay at 0.1.1 and are not republished. `v0.2.0` names
   the qualified affected-package release according to c-1's tag contract;
   do not infer credentials or publish authority from this plan.
2. Wait for registry visibility, then generate the `sqlite-mcp` release-mode
   fixture with no `[patch.crates-io]` entry and run its standard no-edit
   sequence: `just setup`, `just lint`, `just test`, `just build-cli`, and
   `just build-daemon`.
3. Run locked `cargo metadata --format-version 1 --all-features` **only in the
   no-patch generated fixture**. Starting at the rendered `daemon` resolve ID,
   follow normal dependency edges and enforce exactly one reachable
   `(name, version, source)` tuple each for `rmcp`, `utoipa`, and
   `utoipa-axum`. Record sc-command and sc-runtime at exactly 0.2.0 and any
   affected cron version frozen by c-1, all from crates.io rather than path,
   git or patch sources. Assert unaffected packages retain their approved
   0.1.1 versions and no stale command RMCP type reaches the daemon.
4. Store the authorization reference, publication reference, visibility
   evidence, no-patch assertion, and package-identity receipt in the release
   PR/check output.

## Acceptance criteria

Root: `req:REQ-RUN-0002`, `req:REQ-RUN-0503`, `req:REQ-RUN-0702`,
`adr:ADR-RUN-0203`.

- An authorized release record identifies `sc-runtime 0.2.0`; the release
  workflow and per-affected-package registry visibility checks succeed in
  dependency order. Canonical installer repeat dry-run is clean, version
  checks accept the proven mixed package set, and unchanged packages are
  excluded from publication without bypassing validation.
- The generated no-patch fixture resolves that crates.io package and completes
  the full standard command sequence without a generated-source edit.
- The release fixture’s locked metadata closure has one identity each for
  `rmcp`, `utoipa`, and `utoipa-axum`; any duplicate reaching the rendered
  daemon fails the test. Both `sc-command` and `sc-runtime` are exactly 0.2.0
  from crates.io, plus the affected cron version fixed by c-1 when required;
  no local checkout supplies an affected package.
- The fixture uses ADR-RUN-0401/0402's established noninteractive generation
  driver and values-file contract; it introduces no alternate renderer.

## Required validation

- Existing release preflight and publication checks, plus the no-patch fixture
  and exact package-identity assertions after visibility.
- Root `just lint && just test` on the composed integration head.

## This sprint does not close

- Dependency/API migration (`c-3`) or consumer lint configuration (`c-2`).
