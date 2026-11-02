---
requirements:
  - REQ-RUN-0002
  - REQ-RUN-0004
  - REQ-RUN-0005
  - REQ-RUN-0307
  - REQ-RUN-0503
  - REQ-RUN-0701
  - REQ-RUN-0702
  - NFR-RUN-0006
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
  - ADR-RUN-0005
  - ADR-RUN-0012
  - ADR-RUN-0202
  - ADR-RUN-0203
  - ADR-RUN-0401
  - ADR-RUN-0402
---

# Phase bc — consumer lint completeness and runtime ecosystem release

## Objective

Phase `bc` closes two independent adoption gaps left after Phase b:

1. A newly generated consumer must install the complete `sc-lint` **0.8.4**
   toolchain with `just setup`, then run the pre-populated boundary,
   portability, and runtime analyzers through its ordinary `just lint`
   surface. The template must not silently reduce the consumer to only fmt
   and clippy.
2. The public runtime/template pair must move atomically to the current
   compatible `rmcp`, `utoipa`, and `utoipa-axum` releases. The generated
   consumer must use the same Utoipa type generation as the published
   `sc-runtime`, rather than resolve two incompatible OpenAPI stacks.

After the contract sprint establishes the authoritative shared decisions, the
lint and runtime-to-template implementation tracks are deliberately
independent. Their integration sprints close end-to-end implementation proof;
the authorized release sprint closes the separately external registry proof.
The phase has no artificial phase-wide integration sprint.

## Entry state and non-negotiable constraints

- The phase integration branch is `integrate/phase-bc`, cut from `develop`
  only after PR #109 has landed. That PR establishes the shared sc-lint 0.8.4
  release floor. This plan does not fold its changes into the phase work.
- The lockfile audit observed the candidate upgrade set `rmcp 3.8.0`,
  `utoipa 6.0.0`, and `utoipa-axum 0.3.0`. `bc-1` rechecks crates.io and
  executes the compatibility probe, then writes the exact newest stable
  compatible versions into ADR-RUN-0203. `bc-3` consumes that frozen matrix;
  it may not substitute a later release without a new owned ADR amendment and
  repeat of `bc-1`'s probe.
- `rusqlite` remains `0.39.x`. Its `libsqlite3-sys 0.37` resolver constraint
  must remain compatible with template `sqlx 0.9` (`libsqlite3-sys <0.38`).
  No sprint upgrades `rusqlite`, SQLx, or the SQLite writer's package graph.
- `sc-runtime` exposes `utoipa_axum::OpenApiRouter` in its builder surface.
  Moving that public type from Utoipa 5 to 6 is a pre-1.0 breaking change.
  The release therefore publishes `sc-runtime 0.2.0`, and the template
  consumes that exact new minor line. The other standalone library crates
  are not version-bumped unless their manifests actually change.
- `template/` remains consumer code. It may carry only the narrowly declared,
  sc-lint-owned 0.8.4 profile entries approved by `bc-1`; it may not gain a
  hand-written `boundaries/`, clippy, rustfmt, or deny policy. The amendment
  to REQ-RUN-0307, NFR-RUN-0006, ADR-RUN-0005, and the quality policy makes
  that exception explicit before either implementation sprint begins.

## Boundary map

| Track | Boundary / composition root | Contract change | Sprint | Owned-path boundary |
| --- | --- | --- | --- | --- |
| phase contract | consumer lint plus public runtime composition | the binding 0.8.4 consumer-profile exception, its executable receipt/init-preservation protocol, the frozen Utoipa/RMCP package-identity matrix, and the release handoff | `bc-1` | requirements, ADRs, quality-policy text, and a temporary contract probe recorded before deletion |
| consumer lint | generated consumer lint contract | `setup` installs working 0.8.4 binaries; ordinary `just lint` plus named default/full lint profiles invoke `sc-boundary`, `sc-portability`, and `sc-runtime` using the `bc-1` contract | `bc-2` | `template/sc-lint.toml`, bootstrap/Just integration, generation fixture tests and template CI |
| ecosystem integration | `sc-runtime` public builder plus generated daemon | one Utoipa 6 type universe and RMCP 3.8 API adaptation; the checkout-patched consumer proves the public composition before publication | `bc-3` | runtime manifests/source/tests, template manifests/daemon source, lockfile, compatibility tests, fixture CI |
| ecosystem release | published `sc-runtime` plus registry consumer | authorized publication and the no-patch, registry-resolved consumer proof | `bc-4` | release workflow/metadata and a release-mode fixture |

`bc-3` is intentionally vertical. `DaemonBuilder` accepts ordinary
`utoipa_axum::OpenApiRouter`, so the runtime implementation, the generated
daemon, package versions, and checkout-patched consumer proof are one public
compatibility unit. Splitting them would produce a non-compiling intermediate
or require a fake public dependency contract; it creates no useful parallel
work. `bc-4` is separately vertical because only a published package can prove
registry consumption. The rationale is the permitted "thin contract, one
implementer, one consumer" exception from the sprint-planning guidelines.

## Tracks, waves, and ownership

| Wave | Track | Sprint | Closure | Target boundary | Branch | Owned paths |
| --- | --- | --- | --- | --- | --- | --- |
| 1 | contract | `bc-1` | contract | phase-bc-consumer-runtime-contract | `sprint/bc-1-consumer-runtime-contract` | REQ/ADR/quality-policy amendments and a temporary contract probe |
| 2 | lint | `bc-2` | integration | generated-consumer-lint | `sprint/bc-2-consumer-sc-lint-full-profile` | template lint/bootstrap/recipes, generated-fixture validation, template CI |
| 2 | ecosystem | `bc-3` | integration | runtime-template-ecosystem-integration | `sprint/bc-3-runtime-ecosystem-upgrade` | runtime and template dependency/API surfaces, lockfile, compatibility tests, checkout fixture CI |
| 3 | release | `bc-4` | integration | published-runtime-template-consumer | `sprint/bc-4-runtime-ecosystem-release` | release workflow/metadata and release-mode fixture |

- Critical path: **3 sprints** (`bc-1` → `bc-3` → `bc-4`).
- Width: **2 sprints**.
- Sprint count: **4**.
- `bc-1` establishes the authoritative consumer and dependency contracts and
  must finish before `bc-2` or `bc-3`. `bc-2` and `bc-3` are `parallel_safe`:
  they have disjoint owned paths.
  `bc-2` must not modify runtime/template dependency manifests; `bc-3` must
  not change `template/sc-lint.toml`, bootstrap behavior, or consumer lint
  profile names. `bc-4` must follow `bc-3` and owns the authorized registry
  proof; it does not modify runtime or template source.
- Both branches are cut from `integrate/phase-bc`; each gets a distinct
  worktree and a PR to that integration branch. The final integration PR is
  `integrate/phase-bc` to `develop`.

## Feature-closure table

| Feature-level criterion | Sole owner | Proof |
| --- | --- | --- |
| The consumer-profile exception and package-identity/release contracts are unambiguous before implementation starts. | `bc-1` | An executable isolated 0.8.4 no-manifest/init-preservation probe and a temporary metadata probe pass before the amended REQ/ADR/quality-policy text freezes their exact results and the authorized release handoff. |
| A generated consumer installs and executes the complete configured sc-lint toolchain without source edits. | `bc-2` | Generated fixture runs `just setup`, ordinary `just lint`, and named default/full profiles; the installed `sc-lint` launcher’s analyzer invocation receipt proves all three analyzers actually ran. |
| A runtime/template consumer compiles with one RMCP/Utoipa/Utoipa-Axum universe at their approved latest releases. | `bc-3` | Checkout-patched fixture and targeted type-composition tests assert exactly one package identity for each contract package on the runtime/daemon composition graph. |
| Published `sc-runtime 0.2.0` can be consumed by the template without a checkout patch. | `bc-4` | The authorized release workflow publishes only after the release-mode fixture resolves the crates.io package and passes the consumer command sequence. |
| SQLite compatibility remains stable. | `bc-3` | The existing `sqlx_resolve` test and tree assertion prove `rusqlite 0.39.x` with `libsqlite3-sys 0.37`, and no `libsqlite3-sys 0.38` appears. |

## Required decisions and exact compatibility contract

Sprint `bc-1` amends the active architecture before either implementation
sprint starts: **ADR-RUN-0203, “Library versions on one router; one struct for
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
  Template registry requirements and release-mode tests use `0.2.0` exactly;
  local development retains only the driver-owned `[patch.crates-io]` path.
- The SQLite writer remains an independent rusqlite building block. The
  template stores remain SQLx; no dependency edit creates an edge between the
  two layers.
- `bc-4` is the named, separately authorized release handoff. It owns
  publication and the post-visibility registry check; `bc-3` stops at a
  complete checkout-patched prepublication proof and may not claim the
  registry-consumer criterion.

## Out of scope

- Updating `rusqlite` above `0.39.x`, updating SQLx, or resolving the
  `libsqlite3-sys 0.38` incompatibility.
- Adding consumer boundary manifests or copying this repository's boundary
  policy into generated projects.
- New runtime features, new MCP tools, OpenAPI routes, or changes to the
  builder's intended composition responsibility.
- Replacing `sc-lint`'s installation/bootstrap mechanism or implementing new
  analyzer rules in sc-runtime.
- Publishing anything without the existing release authorization and CI
  gates. `bc-4` plans that authorized handoff; this plan does not itself grant
  credentials or publish authority.

## Phase validation and exit

The integration PR may close only after both sprint acceptance sets pass on
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
   reachability algorithm. After `bc-4`'s authorized publication, the
   release-mode fixture runs that algorithm solely in the no-patch generated
   fixture and proves `sc-runtime 0.2.0` has a crates.io registry source.
5. The SQLite resolver guard continues to prove the pinned 0.39/0.37
   rusqlite/libsqlite3-sys pairing.

The phase is complete when its final PR reaches `develop` **after `bc-4`'s
authorized release handoff has recorded a successful registry-consumer proof**.
