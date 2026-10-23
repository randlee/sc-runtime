---
id: a-7
phase: a
status: planned
closure_type: integration
target_boundary: release-distribution
track: release
wave: 4
branch: sprint/a-7-core-release
worktree: sprint/a-7-core-release
recommended_agent: arch-ctm
recommended_model: terra
must_follow: [a-6]
parallel_safe: [a-8]
requirements:
  - REQ-RUN-0001
  - REQ-RUN-0002
  - REQ-RUN-0003
  - REQ-RUN-0004
  - REQ-RUN-0005
  - REQ-RUN-0103
  - REQ-RUN-0503
  - REQ-RUN-0701
  - REQ-RUN-0702
  - NFR-RUN-0001
  - NFR-RUN-0004
  - NFR-RUN-0005
  - NFR-RUN-0007
  - NFR-RUN-0008
  - NFR-RUN-0009
adrs:
  - ADR-RUN-0001
  - ADR-RUN-0002
  - ADR-RUN-0003
  - ADR-RUN-0005
  - ADR-RUN-0008
  - ADR-RUN-0401
owned_paths:
  - "examples/spike/**"
  - "Cargo.toml"
  - "Cargo.lock"
  - "justfile"
  - "README.md"
  - "LICENSE"
  - ".github/workflows/release.yml"
  - "docs/releases/v0.1.0.md"
---

# Sprint a-7 — core release

## Outcome and closure

Publish the four reusable crates and validate one generated application against
registry sources. The core release does not wait for the optional wizard
milestone and does not claim deferred application-policy layers are complete.

## Decision prerequisites

Confirm release credentials, actual crate versions, minimum supported Cargo,
MIT repository/crate licensing, and whether per-crate licence copies are
required. Deferred requirements must be recorded as deferred rather than
waived or falsely closed. Confirm the generated consumer pin is sc-lint
0.8.0 in `template/sc-lint.toml` (`minimum_version = "0.8.0"`) and
`template/.sc-lint/bootstrap.py`. Do not float that pin at tag time.
Confirm the template daemon's `sc-observability` requirement and the
registry-backed fixture lockfile's `sc-observability` and
`sc-observability-types` versions match the a-1 record. Repeat the a-5 CLI
isolation check on that fixture.

## Dependencies and parallel safety

- `must_follow a-6`: consumes a green JSON-driven generated application.
- `parallel_safe a-8`: release paths and wizard paths are disjoint; a-8 consumes
  the already-settled schema and does not modify published crates.

## Deliverables

1. **REQ-RUN-0103** — Preserve spike evidence, then delete the throwaway source
   and all active root references before tagging.
2. **REQ-RUN-0001 / REQ-RUN-0002 / REQ-RUN-0005** — Final four-member library
   workspace, metadata, licences, doctests, boundary validation, and publish
   dry-runs.
3. **REQ-RUN-0702** — Publish leaf crates before sc-runtime, then run the
   fixture matrix against registry versions at the intended tag commit.
4. **REQ-RUN-0003** — Generated Cargo metadata and lockfiles use registry
   versions with no path/git/patch substitute for the four libraries.
5. **REQ-RUN-0503** — Release README and evidence explain the supported core
   generation path and name deferred layers without promising them.
6. **REQ-RUN-0004 / NFR-RUN-0008** — Release gate for the sc-lint 0.8.0 pin.
   `template/sc-lint.toml` pins `minimum_version` to `0.8.0`, and
   `template/.sc-lint/bootstrap.py` is the matching 0.8.0 consumer helper.
   Registry-backed fixture CI runs `just setup && just lint` in the generated
   project before test and build. The source-maintainer install from a-5
   stays on the same 0.8.0 pin.
7. **NFR-RUN-0001 / ADR-RUN-0003** — Release gate runs the CLI isolation check
   closed in a-5 on the registry-backed generated project. When a-5 chose the
   exemption, `cargo tree -p cli -e normal` lists none of the server-only
   crates the ADR names. When a-5 chose a crate split, `cli` does not depend
   on server-enabled SC crates (RBP-F004).
8. **REQ-RUN-0309 / ADR-RUN-0004** — Release gate confirms the template daemon
   requirement and the registry-backed fixture lockfile resolve
   `sc-observability` and `sc-observability-types` to the exact versions a-1
   recorded. The generated daemon manifest does not require
   `sc-observability-otlp`.

## Acceptance criteria

1. `req:REQ-RUN-0002` — each crate's tests, doctests, metadata checks, and
   applicable publish dry-run pass independently.
2. `req:REQ-RUN-0702` — registry inspection shows all four publications,
   sc-runtime published after its leaf dependencies, and the generated lockfile
   records registry sources.
3. `req:REQ-RUN-0103` — the tagged tree contains no spike source or stale build,
   workflow, recipe, or README reference.
4. `req:REQ-RUN-0503` — release notes distinguish the shipped core/template path
   from deferred auto-start, attribute-generated CLI tests, and wizard work.
5. `req:NFR-RUN-0008` — `template/sc-lint.toml` pins `minimum_version` to
   `0.8.0`, and `template/.sc-lint/bootstrap.py` is the matching 0.8.0
   consumer helper. The registry-backed generated fixture runs
   `just setup && just lint` and exits 0. Release notes name that pin.
6. `req:NFR-RUN-0001` / `ADR-RUN-0003` — the registry-backed generated project
   passes the isolation check closed in a-5. Under the exemption,
   `cargo tree -p cli -e normal` omits every server-only crate the Active ADR
   names, and that command is part of the release workflow. Under a crate
   split, `cli`'s normal tree has no server-enabled SC crate (RBP-F004).
7. `req:REQ-RUN-0309` / `ADR-RUN-0004` — `template/` daemon dependency text and
   the registry-backed fixture lockfile match the a-1 recorded
   `sc-observability` and `sc-observability-types` versions. The generated
   daemon manifest does not require `sc-observability-otlp`.

## This sprint does not close

The wizard and all explicitly deferred application-policy layers remain open.

## Paths to delete

- `examples/spike/**`

## Required validation

Run final root validation after spike removal, package and publish dry-runs,
publication in dependency order, registry-backed fixture generation,
`just setup && just lint` in that generated project, generated
lockfile-source inspection, the sc-lint 0.8.0 pin check on
`template/sc-lint.toml` and `template/.sc-lint/bootstrap.py`, the a-5 CLI
isolation check (`cargo tree -p cli -e normal` when that was the closed
policy), the a-1 observability version check on the template and fixture
lockfile, and exact commit/tag checks. Actual publication requires separate
release authorization at execution time.
