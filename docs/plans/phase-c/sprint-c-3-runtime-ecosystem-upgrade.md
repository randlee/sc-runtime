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
  - REQ-CRON-0001
  - NFR-CRON-0001
  - REQ-RUN-0002
  - REQ-RUN-0003
  - NFR-RUN-0001
  - REQ-RT-0005
  - REQ-DBSQ-0001
  - REQ-DBSQ-0005
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
  - REQ-RT-0002
  - REQ-RT-0006
  - REQ-RUN-0332
  - REQ-CRNC-0001
  - REQ-CRNC-0002
  - NFR-CRNC-0001
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
  - ADR-RUN-0001
  - ADR-RUN-0006
  - ADR-RUN-0007
  - ADR-RUN-0009
  - ADR-RUN-0011
  - ADR-RUN-0003
  - ADR-RUN-0012
  - ADR-RUN-0017
  - ADR-RUN-0202
  - ADR-RUN-0203
  - ADR-RUN-0301
  - ADR-RUN-0302
  - ADR-RUN-0303
  - ADR-RUN-0401
  - ADR-RUN-0402
  - ADR-RUN-0018
  - ADR-RUN-0019
  - ADR-CRNC-0001
owned_paths:
  - Cargo.toml
  - Cargo.lock
  - crates/sc-command/Cargo.toml
  - crates/sc-command/src/
  - crates/sc-cron/Cargo.toml
  - crates/sc-cron-commands/Cargo.toml
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
  - scripts/check_observability_dependencies.py
  - template/scripts/consumer_checks.py
  - .github/workflows/template.yml
  - tests/unit/generation/test_runtime_ecosystem_upgrade.py
must_follow: [c-1]
parallel_safe: [c-2, c-5, c-6]
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
3. Change public `sc-command` and `sc-runtime` packages to `0.2.0` and update
   their runtime/template requirements, normal/dev edges and checkout-patched
   fixtures together. Apply c-1's frozen affected closure to cron's command
   requirement and package version if needed; prove a single shared logging
   trait identity rather than forcing a stale registry command dependency.
   Make package versions explicit where required to keep unaffected
   config/transport at 0.1.1; do not bump them with workspace inheritance. `c-4`
   exclusively owns release metadata, release workflows, the registry fixture,
   and authorization evidence. Do not version-bump unaffected standalone crates.
   Update the existing mandatory `scripts/check_observability_dependencies.py`
   command-version/path alignment checks to c-1's frozen affected-package
   contract. Retain the exact defaults-off normal edge, no server/test-util
   activation, no alias/target/build bypass, and isolated default/Monty graph
   checks. Cover accepted new and rejected stale versions in this sprint's
   existing regression venue; do not disable or weaken the guard.
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
7. Hand the verified prepublication release input to `c-4`: every affected
   package version and dependency order, qualified selective-publish kit pin,
   no-patch fixture recipe, exact graph assertion, and registry
   visibility predicate. This sprint does not publish and does not claim a
   release-mode registry result.

## Acceptance criteria

Additional implementation deliverables: REQ-RT-0005/0006 and
REQ-RUN-0332/REQ-CRNC-0002, consuming c-1's compatibility contract:

- Register and poll both Windows Ctrl-C/Ctrl-Break before lock/store setup/bind,
  preserving early observed signals and Unix/private triggers. Native isolated
  child-process tests exercise real console events, in-flight requests, cleanup,
  clean exit/restart; no host daemon processes under NFR-RUN-0010.
- Add portable `DaemonFixture::start_with_cleanup` through the existing
  internal runner and production `run_with_cleanup`. Empty cleanup preserves
  ordinary constructors. Test once-only cleanup on normal/startup failure,
  unused closure drop, retained stores/logger/instance lock, primary error,
  endpoint order, parallel fixtures and physical join after timeout/drop.
- Preserve continuous endpoint ownership: use existing port-zero bind and
  actual endpoint/health handshake, never reserve/drop/rebind or retries/sleeps/
  timeout workarounds. Implement c-1's additive production
  run_with_cleanup_and_bound callback in the same runner for real-signal child
  IPC; retain the listener and report actual endpoint after bind, then health.
  Existing run methods delegate no-op; no transport API/version edit is needed.
  Concurrent port-zero fixtures return distinct nonzero endpoints agreeing
  with client/accessors/health. Causal competing bind to the same owned endpoint
  fails before readiness; planted reserve/drop handoff permits takeover. No
  pre-bind success receipt, weakened real-signal test or implicit endpoint change.
- Prepare generic CLI/HTTP/optional-MCP projections taking Arc<dyn CronExecutor>
  and forwarding the sole CronRequest/deadline through c-1's contract. Test
  them using the canonical executor double copied only to generated tests/
  support; no production double exports or concrete StandardCronExecutor import.
  Default-off generation remains functional. Actual selected factory/root
  registration is c-4 after real c-5/c-6 artifacts exist; this sprint does not
  enable incomplete selected commands or claim their production acceptance.
  Core runtime gains no cron dependency and MCP remains independently selected.
- C-3 alone owns shared affected package/version/template manifests and later
  Cargo.lock migration after c-1's skeleton baseline. Preserve c-1's optional
  adapter feature declarations and rusqlite 0.39 `bundled,hooks` prerequisite
  in the cron manifest/lock. This is a sequential c-1→c-3 handoff, not concurrent
  manifest writing or removal of c-3's later migration ownership. Include the
  adapter's exact 0.2.0 release contract.
  Cron implementation/tests belong solely to c-5; adapter implementation/tests
  solely to c-6. No serial sibling dependency is introduced by shared version.

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
  the frozen `sc-command 0.2.0` and `sc-runtime 0.2.0` lines and c-1's
  affected cron version. Tests reject a stale affected-package requirement
  while permitting unchanged config/transport 0.1.1. Source composition and
  packaged manifests preserve the same shared logging trait and RMCP types.
- The migration preserves NFR-RUN-0007's RMCP minor pin, REQ-RT-0001/0004's
  public builder/router and `IntoMcp` composition, and REQ-CMD-0005 plus
  NFR-CMD-0001/0002's server-feature and dependency boundaries.
  REQ-CMD-0005 is consumed with c-1's updated version identity and unchanged
  conversion/error semantics. NFR-RUN-0001 still isolates CLI dependencies;
  REQ-RT-0005 governs shutdown proof below; REQ-DBSQ-0001/0005 govern the
  independent SQLite writer and resolver guard. The observability guard
  accepts only c-1's aligned versions and rejects stale versions and forbidden
  feature/alias/target/build edges in isolated default and Monty consumers.
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
- Actual selected standard-command binding and composition, or product features
  beyond the explicitly approved generic projections. C-4 owns final template
  binding/registration and real shared-command integration.
- Consumer lint-profile work; `c-2` owns the sc-lint configuration and its
  installation proof.
