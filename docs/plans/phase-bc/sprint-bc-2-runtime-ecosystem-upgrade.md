---
id: bc-2
phase: bc
status: planned
closure_type: integration
target_boundary: runtime-template-ecosystem-release
track: ecosystem
wave: 1
branch: sprint/bc-2-runtime-ecosystem-upgrade
worktree: sprint/bc-2-runtime-ecosystem-upgrade
requirements:
  - REQ-RUN-0002
  - REQ-RUN-0004
  - REQ-RUN-0005
  - REQ-RUN-0301
  - REQ-RUN-0302
  - REQ-RUN-0307
  - REQ-RUN-0503
  - REQ-RUN-0701
  - NFR-RT-0003
adrs:
  - ADR-RUN-0002
  - ADR-RUN-0003
  - ADR-RUN-0012
  - ADR-RUN-0202
  - ADR-RUN-0203
  - ADR-RUN-0401
owned_paths:
  - Cargo.toml
  - Cargo.lock
  - crates/sc-command/Cargo.toml
  - crates/sc-command/src/
  - crates/sc-runtime/Cargo.toml
  - crates/sc-runtime/src/
  - crates/sc-runtime/tests/
  - crates/sc-runtime-db-sqlite/Cargo.toml
  - crates/sc-runtime-db-sqlite/tests/sqlx_resolve.rs
  - boundaries/sc-runtime/sc-runtime.toml
  - template/Cargo.toml
  - template/crates/daemon/Cargo.toml
  - template/crates/daemon/src/
  - template/crates/api-types/
  - template/crates/cli/
  - template/crates/service/
  - scripts/new_project.py
  - template/scripts/consumer_checks.py
  - .github/workflows/template.yml
  - .github/workflows/release.yml
  - .github/workflows/crates-publish.yml
  - release/
  - docs/architecture.md
  - tests/unit/generation/test_runtime_ecosystem_upgrade.py
must_follow: []
parallel_safe: [bc-1]
recommended_agent: solar
recommended_model: sol
vertical_rationale: DaemonBuilder's public OpenApiRouter type, the generated daemon, the public sc-runtime version, and release-mode registry resolution are one consumer compatibility contract. A separate runtime or template layer cannot be production-ready or compile against the other at the new Utoipa major without a fake bridge.
---

# Sprint bc-2 — runtime ecosystem upgrade

## Goal

Upgrade the runtime's RMCP and OpenAPI ecosystem without creating two type
universes in the generated application. The completed consumer must use the
approved latest stable compatible versions (planning candidates: RMCP 3.8.0,
Utoipa 6.0.0, and Utoipa-Axum 0.3.0) with a published `sc-runtime 0.2.0`.
The upgrade explicitly leaves the rusqlite/sqlx compatibility boundary intact.

## Upgrade contract

Before editing manifests, record an Active architecture amendment covering:

- `sc_runtime::DaemonBuilder` continues to accept/compose the ordinary
  Utoipa-Axum router type; the public type moves as one unit from Utoipa 5 to
  the approved Utoipa 6 family.
- RMCP remains an optional server capability with the existing stateless
  streamable HTTP behavior. Any 3.8 API migration stays inside the current
  runtime and generated MCP composition seams; it does not introduce a
  framework command registry or a public wrapper around RMCP.
- A 0.1.x consumer using the Utoipa 5 `OpenApiRouter` is not source
  compatible with the new public router type. `sc-runtime` therefore makes a
  0.1 → 0.2 minor bump; template registry requirements use `0.2.0` exactly.
- `rusqlite = 0.39.x` and the resolved `libsqlite3-sys 0.37.x` remain. SQLx
  0.9's `<0.38` native SQLite constraint is a release blocker, not a warning.

## Deliverables

1. Recheck crates.io immediately before the edit and record the resolved
   versions in the PR. If a candidate is superseded, use the newest stable
   replacement only if all criteria below pass; prereleases are excluded.
2. Update every direct root and template manifest that shares the runtime
   composition: `rmcp`, `utoipa`, and `utoipa-axum`. Update source for the
   released API deltas while preserving the one Axum router, generated routes,
   OpenAPI document, and stateless MCP service behavior.
3. Change the public `sc-runtime` package to `0.2.0`; update the generated
   template's registry requirement and all release metadata/fixtures that
   assert a package version. Do not version-bump unaffected standalone crates.
4. Add graph assertions for both a checkout-patched generated fixture and a
   release-mode generated fixture. They prove one resolved RMCP family and
   one resolved Utoipa/Utoipa-Axum family across runtime and generated daemon,
   and fail on a 5/6 duplicate split.
5. Exercise the public composition seam, not merely compilation: generate a
   sqlite-MCP project, build the daemon's OpenAPI router, list/call the MCP
   example operation, and run the normal consumer command sequence.
6. Preserve and strengthen the SQLite resolver guard. It must fail if
   `rusqlite` leaves 0.39.x, if `libsqlite3-sys 0.38+` resolves, or if the
   template SQLx store ceases to resolve under its declared 0.9 range.
7. Extend the release proof so the release-mode consumer is generated with no
   `[patch.crates-io]` entry, resolves published `sc-runtime 0.2.0`, and
   passes after the package is visible in the registry. This is a preflight
   and retry-safe release check; actual publish still requires authorization.

## Acceptance criteria

Root: `req:REQ-RUN-0301`, `req:REQ-RUN-0302`, `req:REQ-RUN-0503`.

- Root workspace tests and the runtime server-feature checks pass with the
  approved new RMCP/Utoipa versions. The existing router and MCP behavior is
  unchanged from the consumer's perspective.
- A checkout-patched generated sqlite-MCP project completes `just setup`,
  `just lint`, `just test`, `just build-cli`, and `just build-daemon` with no
  source edit. Its resolved graph has one Utoipa major family and one
  Utoipa-Axum family; its runtime and daemon use the same family.
- A release-mode generated project has no checkout patch, resolves
  `sc-runtime 0.2.0` from crates.io, and passes the same sequence after the
  registry visibility check succeeds.
- The runtime package's public router composition test fails to compile when
  linked against the former Utoipa 5 type, demonstrating that the intentional
  0.2.0 minor bump carries the breaking public dependency change rather than
  hiding it in a patch release.
- `cargo tree -d` (scoped to the generated daemon/runtime graph) contains no
  Utoipa 5/6 or Utoipa-Axum 0.2/0.3 split. Unrelated duplicate crates are not
  used to waive this assertion.
- `crates/sc-runtime-db-sqlite/tests/sqlx_resolve.rs` and its companion tree
  assertions prove `rusqlite 0.39.x`, `libsqlite3-sys 0.37.x`, and template
  SQLx 0.9 compatibility; no 0.38+ native SQLite sys crate resolves.

## Required validation

- `cargo update --dry-run --verbose` evidence for the three selected direct
  upgrades, followed by locked workspace build/test and `just lint && just
  test`.
- Targeted RMCP/OpenAPI composition tests plus a generated checkout-patched
  fixture command sequence.
- Scoped `cargo tree` / `cargo tree -d` assertions for the generated daemon,
  runtime, RMCP, Utoipa, Utoipa-Axum, rusqlite, libsqlite3-sys, and SQLx.
- Release workflow dry-run/preflight and release-mode fixture after the
  authorized `sc-runtime 0.2.0` publication has registry visibility.
- Full template fixture matrix on the integration head.

## This sprint does not close

- A rusqlite or SQLx upgrade. The lock stays on rusqlite 0.39.x until a
  separately planned compatibility change eliminates the `libsqlite3-sys`
  conflict.
- New OpenAPI, HTTP, CLI, or MCP product features.
- Consumer lint-profile work; `bc-1` owns the sc-lint configuration and its
  installation proof.
