---
requirements:
  - REQ-RUN-0002
  - REQ-RUN-0004
  - REQ-RUN-0005
  - REQ-RUN-0307
  - REQ-RUN-0503
  - REQ-RUN-0701
  - NFR-RUN-0006
  - NFR-RT-0003
adrs:
  - ADR-RUN-0002
  - ADR-RUN-0003
  - ADR-RUN-0005
  - ADR-RUN-0012
  - ADR-RUN-0202
  - ADR-RUN-0203
  - ADR-RUN-0401
---

# Phase bc — consumer lint completeness and runtime ecosystem release

## Objective

Phase `bc` closes two independent adoption gaps left after Phase b:

1. A newly generated consumer must install a complete `sc-lint` 0.8.x
   toolchain with `just setup`, then run the pre-populated boundary,
   portability, and runtime analyzers through its ordinary `just lint`
   surface. The template must not silently reduce the consumer to only fmt
   and clippy.
2. The public runtime/template pair must move atomically to the current
   compatible `rmcp`, `utoipa`, and `utoipa-axum` releases. The generated
   consumer must use the same Utoipa type generation as the published
   `sc-runtime`, rather than resolve two incompatible OpenAPI stacks.

The tracks are deliberately independent: the first owns only the generated
consumer lint contract; the second owns the public runtime-to-template
compatibility release. Each sprint closes its own end-to-end acceptance
criteria, so this phase has no artificial phase-wide integration sprint.

## Entry state and non-negotiable constraints

- The phase integration branch is `integrate/phase-bc`, cut from `develop`
  only after PR #109 has landed. That PR establishes the shared sc-lint 0.8.0
  release floor. This plan does not fold its changes into the phase work.
- The lockfile audit observed the candidate upgrade set `rmcp 3.8.0`,
  `utoipa 6.0.0`, and `utoipa-axum 0.3.0`. At execution time, the owner of
  sprint `bc-2` rechecks the registry and uses the newest stable compatible
  releases; a newer release is acceptable only after the same compatibility
  evidence passes.
- `rusqlite` remains `0.39.x`. Its `libsqlite3-sys 0.37` resolver constraint
  must remain compatible with template `sqlx 0.9` (`libsqlite3-sys <0.38`).
  No sprint upgrades `rusqlite`, SQLx, or the SQLite writer's package graph.
- `sc-runtime` exposes `utoipa_axum::OpenApiRouter` in its builder surface.
  Moving that public type from Utoipa 5 to 6 is a pre-1.0 breaking change.
  The release therefore publishes `sc-runtime 0.2.0`, and the template
  consumes that exact new minor line. The other standalone library crates
  are not version-bumped unless their manifests actually change.
- `template/` remains consumer code. It may configure analyzer commands, but
  it may not gain a hand-written `boundaries/`, clippy, rustfmt, or deny
  policy. This retains REQ-RUN-0307 and NFR-RUN-0006's ownership split.

## Boundary map

| Track | Boundary / composition root | Contract change | Sprint | Owned-path boundary |
| --- | --- | --- | --- | --- |
| consumer lint | generated consumer lint contract | `setup` installs working 0.8.x binaries; default/full lint profiles invoke `sc-boundary`, `sc-portability`, and `sc-runtime` with no template-owned policy | `bc-1` | `template/sc-lint.toml`, bootstrap/Just integration, generation fixture tests and template CI |
| ecosystem release | `sc-runtime` public builder plus generated daemon | one Utoipa 6 type universe, RMCP 3.8 API adaptation, published `sc-runtime 0.2.0`, and a template that resolves exactly that universe | `bc-2` | runtime manifests/source/tests, template manifests/daemon source, lockfile, release proof, fixture CI |

`bc-2` is intentionally vertical. `DaemonBuilder` accepts ordinary
`utoipa_axum::OpenApiRouter`, so the runtime implementation, the generated
daemon, package versions, and release-mode consumer proof are one public
compatibility unit. Splitting them would produce a non-compiling intermediate
or require a fake public dependency contract; it creates no useful parallel
work. The rationale is the permitted "thin contract, one implementer, one
consumer" exception from the sprint-planning guidelines.

## Tracks, waves, and ownership

| Wave | Track | Sprint | Closure | Target boundary | Branch | Owned paths |
| --- | --- | --- | --- | --- | --- | --- |
| 1 | lint | `bc-1` | integration | generated-consumer-lint | `sprint/bc-1-consumer-sc-lint-full-profile` | template lint/bootstrap/recipes, generated-fixture validation, template CI, requirement text |
| 1 | ecosystem | `bc-2` | integration | runtime-template-ecosystem-release | `sprint/bc-2-runtime-ecosystem-upgrade` | runtime and template dependency/API surfaces, lockfile, compatibility tests, release proof, and architecture amendment |

- Critical path: **1 sprint**.
- Width: **2 sprints**.
- Sprint count: **2**.
- `bc-1` and `bc-2` are `parallel_safe`: they have disjoint owned paths.
  `bc-1` must not modify runtime/template dependency manifests; `bc-2` must
  not change `template/sc-lint.toml`, bootstrap behavior, or consumer lint
  profile names.
- Both branches are cut from `integrate/phase-bc`; each gets a distinct
  worktree and a PR to that integration branch. The final integration PR is
  `integrate/phase-bc` to `develop`.

## Feature-closure table

| Feature-level criterion | Sole owner | Proof |
| --- | --- | --- |
| A generated consumer installs and executes the complete configured sc-lint toolchain without source edits. | `bc-1` | Generated fixture runs `just setup`, then both default and full lint profiles; a binary-presence check and analyzer invocation receipt prove all three analyzers actually ran. |
| A runtime/template consumer compiles with one RMCP/Utoipa/Utoipa-Axum universe at their approved latest releases. | `bc-2` | Checkout-patched fixture plus release-mode fixture resolve one matching version family; `cargo tree -d` and targeted type-composition tests reject duplicate Utoipa stacks. |
| Published `sc-runtime 0.2.0` can be consumed by the template without a checkout patch. | `bc-2` | Existing release workflow publishes only after the release-mode fixture resolves the crates.io package and passes the consumer command sequence. |
| SQLite compatibility remains stable. | `bc-2` | The existing `sqlx_resolve` test and tree assertion prove `rusqlite 0.39.x` with `libsqlite3-sys 0.37`, and no `libsqlite3-sys 0.38` appears. |

## Required decisions and exact compatibility contract

Sprint `bc-2` records an amendment to the active runtime architecture before
changing code. The amendment must state all of the following, so a future
dependency update cannot recreate the split-type failure:

```rust
// The public composition seam remains ordinary library types.
pub fn routes() -> utoipa_axum::OpenApiRouter { /* generated consumer */ }

sc_runtime::Daemon::builder()
    .routes(routes)
    .mcp(mcp_service);
```

- The root runtime and generated daemon resolve the same major `utoipa` and
  `utoipa-axum` family; no adapter, re-export, or conversion bridges version
  5 and version 6 types.
- The runtime server feature and generated MCP server resolve the same RMCP
  3.8-compatible family and preserve the existing stateless streamable-HTTP
  composition.
- The `sc-runtime` public crate version changes from `0.1.1` to `0.2.0`.
  Template registry requirements and release-mode tests use `0.2.0` exactly;
  local development retains only the driver-owned `[patch.crates-io]` path.
- The SQLite writer remains an independent rusqlite building block. The
  template stores remain SQLx; no dependency edit creates an edge between the
  two layers.

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
  gates. The phase supplies release evidence; it does not grant credentials
  or publish authority.

## Phase validation and exit

The integration PR may close only after both sprint acceptance sets pass on
the composed head:

1. `just lint && just test` succeeds, including the current configured
   boundary validation.
2. The template fixture matrix passes all supported answers files. The
   sqlite-MCP fixture records `just setup`, `just lint`, `just test`,
   `just build-cli`, and `just build-daemon` with no generated-source edit.
3. The lint fixture proves default and full profiles execute the three named
   sc-lint analyzers after a clean `just setup`.
4. The ecosystem fixture proves both the checkout-patched and release-mode
   consumer resolution paths have exactly one Utoipa/Utoipa-Axum family,
   the approved RMCP family, and `sc-runtime 0.2.0` in release mode.
5. The SQLite resolver guard continues to prove the pinned 0.39/0.37
   rusqlite/libsqlite3-sys pairing.

The phase is complete when its final PR reaches `develop`; publication itself
remains the separately authorized release workflow described in `bc-2`.
