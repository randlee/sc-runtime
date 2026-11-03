---
requirements:
  - REQ-CRON-0001
  - NFR-CRON-0001
  - REQ-RUN-0004
  - REQ-RUN-0002
  - REQ-RUN-0301
  - REQ-RUN-0503
  - REQ-RUN-0307
  - REQ-RUN-0702
  - NFR-RUN-0006
  - NFR-RUN-0007
  - REQ-CMD-0005
  - NFR-CMD-0001
  - NFR-CMD-0002
  - REQ-CRON-0002
  - REQ-CRON-0003
  - REQ-CRON-0004
  - REQ-CRON-0005
  - REQ-CRON-0019
  - REQ-CRON-0020
  - REQ-CRON-0021
  - NFR-CRON-0002
  - REQ-CRNC-0001
  - REQ-CRNC-0002
  - NFR-CRNC-0001
  - REQ-RUN-0332
  - REQ-RUN-0401
  - REQ-RUN-0402
  - REQ-RUN-0403
  - REQ-RUN-0001
  - REQ-RUN-0005
  - REQ-RT-0005
  - REQ-RT-0006
  - REQ-RUN-0701
  - REQ-RUN-0003
  - NFR-RUN-0001
  - REQ-DBSQ-0001
  - REQ-DBSQ-0005
  - REQ-RUN-0302
  - NFR-RUN-0010
  - NFR-RT-0003
  - REQ-RT-0001
  - REQ-RT-0004
  - REQ-RT-0002
  - NFR-RUN-0002
  - NFR-RUN-0005
  - REQ-CMD-0001
  - REQ-CMD-0002
  - REQ-CMD-0003
  - REQ-CMD-0006
  - NFR-RUN-0009
adrs:
  - ADR-RUN-0002
  - ADR-RUN-0005
  - ADR-RUN-0017
  - ADR-RUN-0202
  - ADR-RUN-0203
  - ADR-RUN-0303
  - ADR-RUN-0003
  - ADR-RUN-0013
  - ADR-RUN-0014
  - ADR-RUN-0016
  - ADR-RUN-0018
  - ADR-RUN-0019
  - ADR-RUN-0401
  - ADR-RUN-0402
  - ADR-CRNC-0001
  - ADR-RT-0001
  - ADR-RT-0004
  - ADR-RT-0005
  - ADR-CMD-0001
  - ADR-CMD-0002
  - ADR-CMD-0003
  - ADR-RT-0002
  - ADR-RT-0003
  - ADR-RUN-0001
  - ADR-RUN-0006
  - ADR-RUN-0007
  - ADR-RUN-0009
  - ADR-RUN-0011
  - ADR-RUN-0012
  - ADR-RUN-0301
  - ADR-RUN-0302
---

# Phase c — consumer lint, cron compatibility and runtime ecosystem release

Readiness: **not approved to begin implementation**. This approved scope
amendment requires fresh scope/critical delta review, QA and actual anchor
checks. Earlier PASS reports retain their historical baselines. Existing
anchor records/quotes are unchanged; newly discussed intent awaits its separate
capture packet, not silent adoption from external consumer evidence.

## Objective

Phase `c` closes the following adoption gaps left after Phase b:

1. A newly generated consumer must install the complete `sc-lint` **0.8.4**
   toolchain with `just setup`, then run the pre-populated boundary,
   portability, and runtime analyzers through its ordinary `just lint`
   surface. The template must not silently reduce the consumer to only fmt
   and clippy.
2. The public runtime/template pair must move atomically to the current
   compatible `rmcp`, `utoipa`, and `utoipa-axum` releases. The generated
   consumer must use the same Utoipa type generation as the published
   `sc-runtime`, rather than resolve two incompatible OpenAPI stacks.
3. Existing cron Rust/Python/Monty registration remains usable through a public
   nominal logging facade and cooperative owned store/shutdown/query protocol.
   Runtime Windows Ctrl-Break and real fixture cleanup cover consumer lifecycle
   seams. Cron is already ported; no new job kind or re-port is planned.
4. An optional standard start/end/list JSON adapter presents the same
   operation/result/error to CLI, HTTP and selected MCP. Default-off generated
   configuration leaves core usable alone; explicitly selected configuration
   wires the real shared handler into the existing UIs.

After the contract sprint establishes the authoritative shared decisions, the
lint and runtime-to-template implementation tracks are deliberately
independent. Their integration sprints close end-to-end implementation proof;
the authorized release sprint closes the separately external registry proof.
The phase has no artificial phase-wide integration sprint. The exact new
contracts and causal proof policy live in
[compatibility-contract.md](compatibility-contract.md), governed by the new
REQ/ADR target amendments. External beads-cache evidence is test/reference
input; actual consumer migration/QA remains external.

## Entry state and non-negotiable constraints

- The phase integration branch is `integrate/phase-c`, cut from `develop`
  only after PR #109 has landed. That PR establishes the shared sc-lint 0.8.4
  release floor. This plan does not fold its changes into the phase work.
- The lockfile audit observed the candidate upgrade set `rmcp 3.8.0`,
  `utoipa 6.0.0`, and `utoipa-axum 0.3.0`. `c-1` rechecks crates.io and
  executes the compatibility probe, then writes the exact newest stable
  compatible versions into ADR-RUN-0203. `c-3` consumes that frozen matrix;
  it may not substitute a later release without a new owned ADR amendment and
  repeat of `c-1`'s probe.
- `rusqlite` remains `0.39.x`. Its `libsqlite3-sys 0.37` resolver constraint
  must remain compatible with template `sqlx 0.9` (`libsqlite3-sys <0.38`).
  No sprint upgrades `rusqlite`, SQLx, or the SQLite writer's package graph.
- `sc-runtime` exposes `utoipa_axum::OpenApiRouter` in its builder surface.
  Moving that public type from Utoipa 5 to 6 is a pre-1.0 breaking change.
  The release therefore publishes `sc-runtime 0.2.0`, and the template
  consumes that exact new minor line. `sc-command` also moves to `0.2.0`:
  its public RMCP `IntoMcp`/`McpError` types change with the same stack.
  `sc-cron` consumes the command-owned logging trait through a versioned
  path dependency; the new cron public compatibility contract and optional
  adapter are both 0.2.0 in c-1's proven affected closure. Unaffected standalone
  libraries retain `0.1.1`.
- Selective publication is a contract to prove, not an assumed kit feature.
  The current pin's release-tag version checks cover all declared crates and
  its registry lookups use the tag version. `c-1` probes the canonical kit
  with affected package versions and unchanged literal-base packages. If it
  cannot validate and publish only the affected closure, the upstream kit
  capability gap blocks release work until an enhancement is qualified;
  do not bypass validation or bump unaffected packages to accommodate it.
- `template/` remains consumer code. Fresh generation installs an appropriate
  default boundary policy derived from the selected project configuration;
  after setup the consumer owns that policy. It does not copy this
  repository's maintainer policy. It may carry the sc-lint-owned 0.8.4
  profile entries approved by `c-1`, but no clippy, rustfmt, or deny policy. The amendment
  to REQ-RUN-0307, NFR-RUN-0006, ADR-RUN-0005, and the quality policy makes
  that exception explicit in the pushed contract consumed by the implementation sprints.

## Boundary map

| Track | Boundary / composition root | Contract change | Sprint | Owned-path boundary |
| --- | --- | --- | --- | --- |
| phase contract | consumer lint plus public runtime composition | the binding 0.8.4 consumer-profile and configuration-derived default boundary policy contract, its executable receipt/init-preservation protocol, the frozen Utoipa/RMCP package-identity matrix, and the release handoff | `c-1` | requirements, ADRs, quality-policy text, and a temporary contract probe recorded before deletion |
| consumer lint | generated consumer lint contract | `setup` installs working 0.8.4 binaries; ordinary `just lint` plus named default/full lint profiles invoke `sc-boundary`, `sc-portability`, and `sc-runtime` using the `c-1` contract | `c-2` | `template/sc-lint.toml`, bootstrap/Just integration, generation fixture tests and template CI |
| ecosystem integration | `sc-runtime` public builder plus generated daemon | one Utoipa 6 type universe and RMCP 3.8 API adaptation; the checkout-patched consumer proves the public composition before publication | `c-3` | runtime manifests/source/tests, template manifests/daemon source, lockfile, compatibility tests, fixture CI |
| ecosystem release | published `sc-runtime` plus registry consumer | authorized publication and the no-patch, registry-resolved consumer proof | `c-4` | release workflow/metadata and a release-mode fixture |
| cron core compatibility | existing `BOUNDARY-sc-cron` | same logging trait facade; shared real-Instant stop/admission context, physical owned join, bounded active/ended/all query and context-aware operations | `c-5` | cron source/tests and isolated external consumer; no shared Cargo writer |
| standard cron adapter | new `BOUNDARY-sc-cron-commands` | common versioned start/end/list DTO/Envelope plus optional scheduler executor; selected UI wiring belongs to c-3 | `c-6` | adapter source/tests/schema/docs; no template or shared manifest edits |

C-1 updates existing command/cron allowed-dependent lists and creates the new
adapter boundary: adapter→defaults-off command, optional adapter→cron. Runtime
and command have no reverse cron/adapter edge. DTO defaults exclude cron,
SQLite, Tokio, Monty and server libraries. Generator shared prompt/schema/
answers/member/feature declarations are c-1's baseline; c-3 owns template
composition and atomic ecosystem manifests/lock after that baseline. C-5 and
c-6 consume c-1's compileable public contracts/double and have disjoint source
paths. A SQLite feature prerequisite must be hoisted before declaring c-5
parallel-safe; it cannot be an uncommitted c-3 manifest dependency.

The existing `BOUNDARY-sc-runtime` manifest allows transport and command
server dependencies plus its third-party runtime stack, permits no workspace
dependents, and forbids config, cron, SQLite-writer, SQLx, and full logging
runtime edges. `BOUNDARY-sc-command` permits `sc-runtime`/`sc-cron` dependents
and serde/serde_json/sc-observability-types dependencies; its server-feature
exceptions remain governed by NFR-CMD-0001/0002. The c-3 upgrade changes
versions inside those contracts, not dependency direction. The authoritative
lists remain `boundaries/sc-runtime/sc-runtime.toml` and
`boundaries/sc-command/sc-command.toml`. c-2 generates a separate consumer
policy from ADR-RUN-0303's selected application crate graph; it never vendors
these maintainer manifests.

`c-3` is intentionally vertical. `DaemonBuilder` accepts ordinary
`utoipa_axum::OpenApiRouter`, so the runtime implementation, the generated
daemon, package versions, and checkout-patched consumer proof are one public
compatibility unit. Splitting them would produce a non-compiling intermediate
or require a fake public dependency contract; it creates no useful parallel
work. `c-4` is separately vertical because only a published package can prove
registry consumption. The rationale is the permitted "thin contract, one
implementer, one consumer" exception from the sprint-planning guidelines.

## Tracks, waves, and ownership

| Wave | Track | Sprint | Closure | Target boundary | Branch | Owned paths |
| --- | --- | --- | --- | --- | --- | --- |
| 1 | contract | `c-1` | contract | phase-c-consumer-runtime-contract | `sprint/c-1-consumer-runtime-contract` | REQ/ADR/quality-policy amendments and a temporary contract probe |
| 2 | lint | `c-2` | integration | generated-consumer-lint | `sprint/c-2-consumer-sc-lint-full-profile` | template lint/bootstrap/recipes, generated-fixture validation, template CI |
| 2 | ecosystem | `c-3` | integration | runtime-template-ecosystem-integration | `sprint/c-3-runtime-ecosystem-upgrade` | runtime and template dependency/API surfaces, lockfile, compatibility tests, checkout fixture CI |
| 2 | cron | `c-5` | integration | sc-cron-consumer-compatibility | `sprint/c-5-cron-consumer-compatibility` | cron source/tests and isolated external consumer |
| 2 | cron-commands | `c-6` | boundary | sc-cron-commands | `sprint/c-6-standard-cron-commands` | adapter implementation/schema/tests/docs |
| 3 | release | `c-4` | integration | published-runtime-template-consumer | `sprint/c-4-runtime-ecosystem-release` | release workflow/metadata and release-mode fixture |

- Critical path: **3 sprints** (`c-1` → `c-3` → `c-4`).
- Width: **4 sprints**.
- Sprint count: **6**. Added substantial independent cron and adapter
  boundaries raise usable width without increasing the critical path.
- `c-1` establishes the authoritative consumer and dependency contracts.
  Its development push triggers merge-forward into `c-2`, `c-3`, `c-5` and `c-6` before
  every child development/fix round; parent QA is not a start barrier, and
  the `c-1` PR must merge before any child PR completes.
  All four wave-2 siblings are `parallel_safe` with disjoint owned paths.
  `c-2` must not modify runtime/template dependency manifests; `c-3` must
  not change `template/sc-lint.toml`, bootstrap behavior, or consumer lint
  profile names. C-5 owns cron implementation/tests, c-6 adapter implementation/
  tests. `c-4` consumes `c-3`'s prepared ecosystem/UI implementation,
  c-5's real query/lifecycle and c-6's concrete handler:
  every parent development push triggers merge-forward before each
  child development/fix round, parent QA is not a start barrier, and the
  parent PR must merge before the `c-4` PR completes. `c-4` owns the authorized
  registry proof; it does not modify runtime or template source.
- Each sibling branch is cut from `integrate/phase-c`; each gets a distinct
  worktree and a PR to that integration branch. The final integration PR is
  `integrate/phase-c` to `develop`.

## Feature-closure table

| Feature-level criterion | Sole owner | Proof |
| --- | --- | --- |
| The consumer-profile exception and package-identity/release contracts are unambiguous before implementation starts. | `c-1` | An executable isolated 0.8.4 fresh-policy/init-preservation probe and a temporary metadata probe pass before the amended REQ/ADR/quality-policy text freezes their exact results and the authorized release handoff. |
| A generated consumer installs and executes the complete configured sc-lint toolchain without source edits. | `c-2` | Generated fixture runs `just setup`, ordinary `just lint`, and named default/full profiles; the installed `sc-lint` launcher’s analyzer invocation receipt proves all three analyzers actually ran. |
| A runtime/template consumer compiles with one RMCP/Utoipa/Utoipa-Axum universe at their approved latest releases. | `c-3` | Checkout-patched fixture and targeted type-composition tests assert exactly one package identity for each contract package on the runtime/daemon composition graph. |
| Published `sc-runtime 0.2.0` can be consumed by the template without a checkout patch. | `c-4` | The authorized release workflow publishes after prepublication checks; once the package is visible, the release-mode fixture resolves it from crates.io and passes the consumer command sequence before release closure. |
| SQLite compatibility remains stable. | `c-3` | The existing `sqlx_resolve` test and tree assertion prove `rusqlite 0.39.x` with `libsqlite3-sys 0.37`, and no `libsqlite3-sys 0.38` appears. |
| Existing cron hooks, facade and cooperative owned store/query/lifecycle work through the public library. | `c-5` | Real causal blocked-operation/SQLite/physical-join tests and isolated Rust/Python/Monty consumer; fixed budget/cleanup/error contract, no detached success. |
| Standard command adapter closes its boundary against admitted-operation contract. | `c-6` | Strict JSON/Envelope/error/feature graph and contract-double tests; no real-worker claim. |
| Windows Ctrl-Break and fixture cleanup preserve production lifecycle ownership. | `c-3` | Native isolated signal/restart proof plus coordinated portable cleanup/lock/thread tests. |
| Selected generated CLI/HTTP/MCP actually share start/end/list for all three existing targets. | `c-4` | Real composed checkout fixture before publication and exact registry fixture after visibility, normalized data/error parity and minimal default-off graph. |

## Required decisions and exact compatibility contract

Sprint `c-1` supplies the pushed contract consumed by the implementation
sprints: **ADR-RUN-0203, “Library versions on one router; one struct for
both schemas.”** The amendment replaces its old 0.2/5.5.0/3.7 evidence matrix
with the approved Utoipa 6/RMCP 3.8 matrix and retires obsolete pin evidence.
It must state all of the following, so a future dependency update cannot
recreate the split-type failure:

```rust
// The public composition seam remains ordinary library types.
pub fn routes() -> utoipa_axum::OpenApiRouter { /* generated consumer */ }

sc_runtime::Daemon::builder()
    .routes(routes)
    .mcp(mcp_service);
```

- The root runtime and generated daemon resolve exactly one package identity
  (name plus version plus source) each for `utoipa`, `utoipa-axum`, and
  `rmcp`; no adapter, re-export, or conversion bridges old and new types.
  The proof runs locked `cargo metadata --format-version 1 --all-features`
  for the runtime workspace and for the rendered daemon fixture. A verifier
  starts at the `sc-runtime` and `daemon` resolve-node IDs, follows only normal
  dependency edges, and requires one distinct `(name, version, source)` tuple
  per contract name across their union. Any second reachable identity fails.
- The runtime server feature and generated MCP server resolve the approved
  RMCP 3.8 package identity and preserve the existing stateless streamable-HTTP
  composition.
- The `sc-runtime` public crate version changes from `0.1.1` to `0.2.0`.
  Command, cron and new cron-commands are `0.2.0` in the affected closure.
  Publication follows dependency order: command first, then cron and runtime,
  then cron-commands after cron visibility.
  Template registry requirements and release-mode tests use the frozen versions exactly;
  local development retains only the driver-owned `[patch.crates-io]` path.
- The SQLite writer remains an independent rusqlite building block. The
  template stores remain SQLx; no dependency edit creates an edge between the
  two layers.
- `c-4` is the named, separately authorized release handoff. It owns
  publication and the post-visibility registry check; `c-3` stops at a
  complete checkout-patched prepublication proof and may not claim the
  registry-consumer criterion.

## Out of scope

- Updating `rusqlite` above `0.39.x`, updating SQLx, or resolving the
  `libsqlite3-sys 0.38` incompatibility.
- Copying this repository's maintainer boundary policy into generated
  projects, or overwriting an existing consumer-owned boundary policy.
- Features beyond the explicitly approved lifecycle compatibility and optional
  standard cron projections. No core runtime cron edge, mandatory command
  installation, ATM-specific fields, new cron port/job type/catalog, unrelated
  CRUD expansion or consumer-product migration/QA waiver.
- Replacing `sc-lint`'s installation/bootstrap mechanism or implementing new
  analyzer rules in sc-runtime.
- Publishing anything without the existing release authorization and CI
  gates. `c-4` plans that authorized handoff; this plan does not itself grant
  credentials or publish authority.

## Phase validation and exit

The integration PR may close only after all six sprint acceptance sets pass on
the composed head:

1. `just lint && just test` succeeds, including the current configured
   boundary validation.
2. The template fixture matrix passes all supported answers files. The
   sqlite-MCP fixture records `just setup`, `just lint`, `just test`,
   `just build-cli`, and `just build-daemon` with no generated-source edit.
3. The lint fixture proves ordinary `just lint` and named default/full profiles
   execute the three named sc-lint analyzers after a clean `just setup`.
4. The checkout-patched ecosystem fixture proves exactly one package identity
   each for Utoipa, Utoipa-Axum, and RMCP by the locked Cargo-metadata
   reachability algorithm. After `c-4`'s authorized publication, the
   release-mode fixture runs that algorithm solely in the no-patch generated
   fixture and proves `sc-runtime 0.2.0` has a crates.io registry source.
5. The SQLite resolver guard continues to prove the pinned 0.39/0.37
   rusqlite/libsqlite3-sys pairing.
6. C-5 proves existing three-target registration, real cooperative store/query
   cancellation, restricted cleanup and physical joins; c-3 proves Windows
   signals and real fixture cleanup. No timeout status substitutes for joins.
7. C-4 proves selected standard commands across actual CLI/HTTP/optional MCP,
   both checkout and registry, with c-5/c-6 implementations and default-off
   feature graphs. Earlier boundary/double evidence is not this composition.
8. Fresh scope/critical delta review, manager QA and actual Plan Anchor
   testability/fixtures/full-sprint gates cover the amended six-sprint plan.
   Historic review reports/seals remain intact; unfinished gates keep readiness
   false regardless of source-code or preparation checkpoint success.

The phase is complete when its final PR reaches `develop` **after `c-4`'s
authorized release handoff has recorded a successful registry-consumer proof**.
