---
id: c-3
phase: c
status: planned
closure_type: integration
target_boundary: runtime-template-ecosystem-integration
track: ecosystem
wave: 2
branch: sprint/c-3-runtime-ecosystem-upgrade
worktree: sprint/c-3-runtime-ecosystem-upgrade
requirements:
  - REQ-RUN-0002
  - REQ-RUN-0004
  - REQ-RUN-0005
  - REQ-RUN-0301
  - REQ-RUN-0302
  - REQ-RUN-0307
  - REQ-RUN-0503
  - REQ-RUN-0701
  - NFR-RUN-0007
  - NFR-RUN-0010
  - NFR-RT-0003
  - REQ-RT-0001
  - REQ-RT-0004
  - REQ-CMD-0005
  - NFR-CMD-0001
  - NFR-CMD-0002
adrs:
  - ADR-CMD-0001
  - ADR-CMD-0002
  - ADR-CMD-0003
  - ADR-RT-0001
  - ADR-RT-0002
  - ADR-RT-0003
  - ADR-RT-0004
  - ADR-RT-0005
  - ADR-RUN-0002
  - ADR-RUN-0003
  - ADR-RUN-0012
  - ADR-RUN-0202
  - ADR-RUN-0203
  - ADR-RUN-0301
  - ADR-RUN-0302
  - ADR-RUN-0303
  - ADR-RUN-0401
  - ADR-RUN-0402
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
  - template/Cargo.toml
  - template/workspace.template.toml
  - template/crates/daemon/Cargo.toml
  - template/crates/daemon/src/
  - template/crates/api-types/
  - template/crates/cli/
  - template/crates/service/
  - scripts/new_project.py
  - template/scripts/consumer_checks.py
  - .github/workflows/template.yml
  - tests/unit/generation/test_runtime_ecosystem_upgrade.py
must_follow: [c-1]
parallel_safe: [c-2]
recommended_agent: solar
recommended_model: sol
vertical_rationale: DaemonBuilder's public OpenApiRouter type, the generated daemon, the public sc-runtime version, and the checkout-patched type-composition proof are one consumer compatibility contract. A separate runtime or template layer cannot compile against the other at the new Utoipa major without a fake bridge; `c-4` separately owns the post-publication registry proof.
---

# Sprint c-3 — runtime ecosystem upgrade

Dependency execution: consume `c-1`'s frozen ADR-RUN-0203 package-identity
matrix by merging each parent development push before every development/fix
round. Parent QA is not a start barrier; the parent PR merges before this
PR completes.

## Goal

Upgrade the runtime's RMCP and OpenAPI ecosystem without creating two type
universes in the generated application. The completed consumer must use the
exact stable compatible versions frozen by `c-1` (the current planning
candidates are RMCP 3.8.0, Utoipa 6.0.0, and Utoipa-Axum 0.3.0) and produces
the complete checkout-patched prepublication proof for `sc-runtime 0.2.0`. The separately
authorized `c-4` owns publication and registry consumption.
The upgrade explicitly leaves the rusqlite/sqlx compatibility boundary intact.

## Upgrade contract

Before editing manifests, consume the binding `c-1` amendment to
**ADR-RUN-0203**. It replaces the old 0.2/5.5.0/3.7 evidence matrix with the
exact Utoipa/RMCP matrix, including exact package-identity counts;
this sprint may not leave the obsolete pin evidence active. The implementation
must satisfy that amendment:

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

1. Use the exact stable versions selected and frozen by `c-1` in
   ADR-RUN-0203. A later replacement is out of scope unless its owner first
   amends that ADR and reruns the `c-1` contract probe; prereleases are
   excluded.
2. Update every direct root and template manifest that shares the runtime
   composition: `rmcp`, `utoipa`, and `utoipa-axum`. Update source for the
   released API deltas while preserving the one Axum router, generated routes,
   OpenAPI document, and stateless MCP service behavior.
3. Change the public `sc-runtime` package to `0.2.0` and update only its
   root/template manifest requirements and checkout-patched fixtures. `c-4`
   exclusively owns release metadata, release workflows, the registry fixture,
   and authorization evidence. Do not version-bump unaffected standalone crates.
4. Add graph assertions for the checkout-patched generated fixture. Run locked
   `cargo metadata --format-version 1 --all-features` for the runtime workspace
   and rendered fixture. A verifier starts from the `sc-runtime` and `daemon`
   resolve IDs, follows normal dependency edges, and requires exactly one
   `(name, version, source)` tuple each for `rmcp`, `utoipa`, and
   `utoipa-axum` across their union. Any second reachable identity fails even
   if both versions share a major number. `c-4` repeats this exact assertion
   solely in its no-patch release fixture.
5. Exercise the public composition seam, not merely compilation: generate a
   sqlite-MCP project, build the daemon's OpenAPI router, list/call the MCP
   example operation, and run the normal consumer command sequence.
6. Preserve and strengthen the SQLite resolver guard. It must fail if
   `rusqlite` leaves 0.39.x, if `libsqlite3-sys 0.38+` resolves, or if the
   template SQLx store ceases to resolve under its declared 0.9 range.
7. Hand the verified prepublication release input to `c-4`: the package
   version, no-patch fixture recipe, exact graph assertion, and registry
   visibility predicate. This sprint does not publish and does not claim a
   release-mode registry result.

## Acceptance criteria

Root: `req:REQ-RUN-0301`, `req:REQ-RUN-0302`, `req:REQ-RUN-0503`,
  `req:REQ-RT-0001`, `req:REQ-RT-0004`, `req:REQ-CMD-0005`.

- Root workspace tests and the runtime server-feature checks pass with the
  approved new RMCP/Utoipa versions. The existing router and MCP behavior is
  unchanged from the consumer's perspective.
- A checkout-patched generated sqlite-MCP project completes `just setup`,
  `just lint`, `just test`, `just build-cli`, and `just build-daemon` with no
  source edit. Its locked metadata closure contains exactly one reachable
  `(name, version, source)` tuple each for `rmcp`, `utoipa`, and
  `utoipa-axum` from the runtime/daemon roots.
- Both `template/Cargo.toml` and `template/workspace.template.toml` require
  the frozen `sc-runtime 0.2.0` line; the generation fixture fails if either
  source leaves `0.1.1` behind.
- The migration preserves NFR-RUN-0007's RMCP minor pin, REQ-RT-0001/0004's
  public builder/router and `IntoMcp` composition, and REQ-CMD-0005 plus
  NFR-CMD-0001/0002's server-feature and dependency boundaries.
- In the isolated fixture venue required by host-safety rules, a generated
  daemon with an in-flight HTTP/MCP request receives the established
  SIGINT/SIGTERM shutdown trigger and proves the documented drain-or-cancel
  outcome plus clean process exit.
- The runtime package's public router composition test fails to compile when
  linked against the former Utoipa 5 type, demonstrating that the intentional
  0.2.0 minor bump carries the breaking public dependency change rather than
  hiding it in a patch release.
- The two locked Cargo-metadata documents, parsed by the stated normal-edge
  reachability algorithm, contain exactly one identity each for `rmcp`,
  `utoipa`, and `utoipa-axum` across the runtime/daemon composition graph. A
  duplicate in either closure fails; unrelated duplicate crates do not waive
  this assertion.
- `crates/sc-runtime-db-sqlite/tests/sqlx_resolve.rs` and its companion tree
  assertions prove `rusqlite 0.39.x`, `libsqlite3-sys 0.37.x`, and template
  SQLx 0.9 compatibility; no 0.38+ native SQLite sys crate resolves.

## Required validation

- `cargo update --dry-run --verbose` evidence for the three selected direct
  upgrades, followed by locked workspace build/test and `just lint && just
  test`.
- Targeted RMCP/OpenAPI composition tests plus a generated checkout-patched
  fixture command sequence, including the isolated graceful-shutdown
  regression described above.
- Locked Cargo-metadata reachability assertions for the generated daemon,
  runtime, RMCP, Utoipa, and Utoipa-Axum; scoped resolver assertions for
  rusqlite, libsqlite3-sys, and SQLx.
- Release workflow dry-run/preflight and a handoff artifact for `c-4`; the
  release-mode fixture runs only in that named authorized release sprint.
- Full template fixture matrix on the integration head.

## This sprint does not close

- A rusqlite or SQLx upgrade. The lock stays on rusqlite 0.39.x until a
  separately planned compatibility change eliminates the `libsqlite3-sys`
  conflict.
- New OpenAPI, HTTP, CLI, or MCP product features.
- Consumer lint-profile work; `c-2` owns the sc-lint configuration and its
  installation proof.
