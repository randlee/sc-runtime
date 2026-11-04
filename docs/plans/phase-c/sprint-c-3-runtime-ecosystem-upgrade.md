---
id: c-3
phase: c
status: in_progress
closure_type: integration
target_boundary: runtime-template-ecosystem-integration
track: ecosystem
wave: 2
branch: sprint/c-3-runtime-ecosystem-upgrade
worktree: ../sc-runtime-worktrees/sprint/c-3-runtime-ecosystem-upgrade
requirements:
  - NFR-RT-0001
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
  - REQ-CMD-0001
  - REQ-CMD-0002
  - REQ-CMD-0003
  - REQ-CMD-0004
  - REQ-CMD-0006
  - NFR-CMD-0003
  - REQ-RT-0003
  - REQ-RT-0007
  - REQ-RT-0008
  - NFR-RT-0002
  - NFR-RT-0004
  - REQ-CRON-0002
  - REQ-CRON-0003
  - REQ-CRON-0004
  - REQ-CRON-0005
  - REQ-CRON-0019
  - NFR-CRON-0002
  - REQ-CRON-0020
  - REQ-CRON-0021
  - REQ-DBSQ-0002
  - REQ-DBSQ-0003
  - REQ-DBSQ-0004
  - NFR-RUN-0009
  - NFR-RUN-0002
  - NFR-RUN-0005
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
universes in the generated application. Upgrade RMCP and Utoipa to the exact
latest releases returned by c-1's dated crates.io query and frozen in its
ADR-RUN-0203 matrix, with the matching Utoipa-Axum release. Produce
the complete checkout-patched prepublication proof for `sc-runtime 0.2.0`. The separately
authorized `c-4` owns publication and registry consumption.
The upgrade explicitly leaves the rusqlite/sqlx compatibility boundary intact.

## Upgrade contract

Before editing manifests, consume the binding `c-1` amendment to
**ADR-RUN-0203**. It freezes the dated latest-release Utoipa/RMCP matrix,
including resolved-version equality and exact package-identity counts;
this sprint may not leave the obsolete pin evidence active. The implementation
must satisfy that amendment:

- `sc_runtime::DaemonBuilder` continues to accept/compose the ordinary
  Utoipa-Axum router type; the public type moves as one unit from Utoipa 5 to
  the probe-selected Utoipa release family.
- RMCP remains an optional server capability with the existing stateless
  streamable HTTP behavior. Released API migration stays inside the current
  runtime and generated MCP composition seams; it does not introduce a
  framework command registry or a public wrapper around RMCP.
- A 0.1.x consumer using the Utoipa 5 `OpenApiRouter` is not source
  compatible with the new public router type. `sc-runtime` therefore makes a
  0.1 → 0.2 minor bump; template registry requirements use `0.2.0` exactly.
- `rusqlite = 0.39.x` and the resolved `libsqlite3-sys 0.37.x` remain. SQLx
  0.9's `<0.38` native SQLite constraint is a release blocker, not a warning.

## Deliverables

1. Upgrade to c-1's exact latest released RMCP and Utoipa versions from its
   dated query, frozen in ADR-RUN-0203. Locked resolved versions must equal
   that matrix; mismatch fails. A replacement requires its owner first to
   amend that ADR and rerun the `c-1` contract probe; prereleases are
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
   release-mode registry result. C-5 is the dedicated cron compatibility sprint;
   c-4's existing c-3/c-5/c-6 prerequisites require its completion before 0.2.0
   publication. This handoff does not authorize publication or make c-3 wait
   for sibling implementation/QA before completing its own work.

## Acceptance criteria

- Consume c-1's committed sole DTO/schema feature baseline; activate schema
  only on selected daemon/service edges, never shared workspace/CLI defaults.
  Real RMCP Parameters<CronRequest> and typed Utoipa request/data projections
  compile against the actual adapter declarations and executor double, with
  typed Envelope<CronResponse>/OpError response schemas supplied by the actual
  command/schema owner rather than a foreign impl or model copy. They use
  frozen schemars/Utoipa identities and no c-6 implementation dependency.
  Verify version/tag/required/target schema shapes and keep disabled MCP and
  isolated CLI graphs intact. No foreign orphan impl, copied wire DTO or
  generic object schema replaces the same typed contract.

REQ-RUN-0503 is a supporting generated-project constraint here; c-2 alone
owns its phase requirement root. Retain this sprint's full checkout proof.

Additional implementation deliverables: REQ-RT-0005/0006 and
REQ-RUN-0332/REQ-CRNC-0002, consuming c-1's compatibility contract:

- Consume c-1's amended NFR-RT-0001 exact seven-method public surface and
  ADR-RT-0001/0005 reconciliation. Rustdoc/default and test-util checks prove
  Ready-only terminal alternatives, same-state logger, unchanged startup
  order and no registry, macro, library wrapper or extra callback mechanism.

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
- Implement public CLI/HTTP/enabled-MCP projection handlers, all using the same
  canonical versioned CronRequest JSON contract: `cron.start`, `cron.end`,
  `cron.list`. Transport-specific wrappers remain permitted. The same
  start/end/list schedule-management verbs apply to Rust, Python and optional
  Monty target selections through that contract's options. Consumers may omit
  the entire command set (`cron_commands=false`, default); then no standard
  cron commands are required. Forward request/deadline to an application-supplied
  Arc<dyn CronExecutor>, letting consumers replace default command/lifecycle/
  error behavior with their owned implementation and responsibilities from c-1.
  Preserve opaque application Stores pass-through (REQ-RT-0003). Test with the
  canonical executor double copied only into generated tests/support, never
  production exports; no concrete StandardCronExecutor import. Actual factory/root
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

Root: `req:REQ-RUN-0301`, `req:REQ-RUN-0302`,
  `req:REQ-RT-0001`, `req:REQ-RT-0004`, `req:REQ-CMD-0005`.

- Root workspace tests and the runtime server-feature checks pass with the
  approved new RMCP/Utoipa versions. The existing router and MCP behavior is
  unchanged from the consumer's perspective.
- A checkout-patched generated sqlite-MCP project completes `just setup`,
  `just lint`, `just test`, `just build-cli`, and `just build-daemon` with no
  source edit. Its locked metadata closure contains exactly one reachable
  `(name, version, source)` tuple each for `rmcp`, `utoipa`, and
  `utoipa-axum` from the runtime/daemon roots. Both locked metadata documents
  use the stated normal-edge reachability algorithm: a duplicate in either
  closure fails; unrelated duplicates cannot waive this assertion.
- Both `template/Cargo.toml` and `template/workspace.template.toml` require
  the frozen `sc-command 0.2.0` and `sc-runtime 0.2.0` lines and c-1's
  affected cron version. Tests reject a stale affected-package requirement
  while permitting unchanged config/transport 0.1.1. Source composition and
  packaged manifests preserve the same shared logging trait and RMCP types.
- The migration preserves NFR-RUN-0007's RMCP minor pin, REQ-RT-0001/0004's
  public builder/router and `IntoMcp` composition, and REQ-CMD-0005 plus
  NFR-CMD-0001/0002's server-feature and dependency boundaries.
  REQ-CMD-0005 is consumed with c-1's updated version identity and unchanged
  conversion/error semantics; NFR-CMD-0001 consumes c-1's amended minor series
  with remaining dependency/server restrictions unchanged. NFR-RUN-0001's
  narrow selected defaults-only adapter exception still isolates CLI dependencies;
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

## Development checkpoint

Consumed C1 `dede889b11dcbe1495e2fe4ccdbe30d57b84bec9` and integration
`9ec54e6fec76d756bab2671e4abf3deaa88aa122`. The first checkpoint repairs
existing generated-source findings exposed by C2's full consumer profiles.
The coordinator authorized the narrow additional ownership of generated schema,
store and adaptor manifests and the three SQL store implementations for these
repairs. Package metadata inherits application workspace values and direct
application path dependencies retain their matching 0.1.1 application version.
Store implementations own their helpers; existing public free-function APIs,
SQL statements and engine behavior remain unchanged. The Unix extension import
has an explicit item gate and the isolated logger fixture polls with an async
timer. Template CI exercises ordinary, default and full consumer lint profiles.

Eight actual generated configurations passed released sc-lint 0.8.4 boundary,
portability and runtime analyzers with C2's selected consumer policies. Boundary
results retain the analyzer's nonfatal trait-self-loop signals; no policy or
severity was changed. Root formatting and strict all-target Clippy pass. Full
consumer behavior and the remaining ecosystem/runtime deliverables are still
in progress; this checkpoint does not claim sprint completion.
