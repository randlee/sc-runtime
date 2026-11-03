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
  - REQ-RUN-0301
  - REQ-RUN-0503
  - REQ-RUN-0702
adrs:
  - ADR-RUN-0002
  - ADR-RUN-0203
  - ADR-RUN-0401
  - ADR-RUN-0402
owned_paths:
  - .github/workflows/release.yml
  - .github/workflows/crates-publish.yml
  - release/
  - tests/integration/release/test_published_runtime_ecosystem.py
must_follow: [c-3]
parallel_safe: []
recommended_agent: solar
recommended_model: sol
vertical_rationale: Registry visibility can only be verified after the exact public package is authorized and published. The release workflow, visibility wait, and no-patch consumer are a single production integration closure and cannot be honestly claimed by a prepublication implementation sprint.
---

# Sprint c-4 — runtime ecosystem authorized release

## Goal

Close the production release contract left deliberately open by `c-3`:
publish the approved `sc-runtime 0.2.0` only with release authorization, then
prove a newly generated consumer resolves it from crates.io without a checkout
patch.

## Deliverables

1. Use the existing authorization and publication workflow for `sc-runtime
   0.2.0`; do not infer credentials or publish authority from this plan.
2. Wait for registry visibility, then generate the `sqlite-mcp` release-mode
   fixture with no `[patch.crates-io]` entry and run its standard no-edit
   sequence: `just setup`, `just lint`, `just test`, `just build-cli`, and
   `just build-daemon`.
3. Run locked `cargo metadata --format-version 1 --all-features` **only in the
   no-patch generated fixture**. Starting at the rendered `daemon` resolve ID,
   follow normal dependency edges and enforce exactly one reachable
   `(name, version, source)` tuple each for `rmcp`, `utoipa`, and
   `utoipa-axum`. Record that `sc-runtime` is exactly 0.2.0 and its source is
   the crates.io registry, not a path, git, or patch source.
4. Store the authorization reference, publication reference, visibility
   evidence, no-patch assertion, and package-identity receipt in the release
   PR/check output.

## Acceptance criteria

Root: `req:REQ-RUN-0002`, `req:REQ-RUN-0503`, `req:REQ-RUN-0702`,
`adr:ADR-RUN-0203`.

- An authorized release record identifies `sc-runtime 0.2.0`; the release
  workflow and registry visibility check succeed.
- The generated no-patch fixture resolves that crates.io package and completes
  the full standard command sequence without a generated-source edit.
- The release fixture’s locked metadata closure has one identity each for
  `rmcp`, `utoipa`, and `utoipa-axum`; any duplicate reaching the rendered
  daemon fails the test. Its `sc-runtime` package is exactly 0.2.0 from the
  crates.io registry, not a local checkout.
- The fixture uses ADR-RUN-0401/0402's established noninteractive generation
  driver and values-file contract; it introduces no alternate renderer.

## Required validation

- Existing release preflight and publication checks, plus the no-patch fixture
  and exact package-identity assertions after visibility.
- Root `just lint && just test` on the composed integration head.

## This sprint does not close

- Dependency/API migration (`c-3`) or consumer lint configuration (`c-2`).
