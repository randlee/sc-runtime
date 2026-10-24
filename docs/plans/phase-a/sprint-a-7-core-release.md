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
  - NFR-RUN-0004
  - NFR-RUN-0005
  - NFR-RUN-0007
  - NFR-RUN-0009
adrs:
  - ADR-RUN-0002
  - ADR-RUN-0005
  - ADR-RUN-0008
  - ADR-RUN-0401
owned_paths:
  - "examples/spike/**"
  - "Cargo.toml"
  - "Cargo.lock"
  - "Justfile"
  - "README.md"
  - "LICENSE"
  - "release/**"
  - ".github/workflows/release*.yml"
  - ".github/workflows/crates.yml"
  - "scripts/release_*.py"
  - "docs/releases/v0.1.0.md"
  - "docs/development/sc-publish.md"
  - "docs/architecture.md"
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
waived or falsely closed. Vend **sc-publish** from `release/sc-publish-pin.toml` via
`install.py` and `release/sc-publish-consumer-input.json` (GitHub Release +
crates.io); see `docs/development/sc-publish.md`. `install.py` writes
`.github/workflows/release*.yml` and `scripts/release_*.py`. Those outputs
are committed as generated files and not hand-edited. The only authored
release inputs are `release/sc-publish-pin.toml`,
`release/sc-publish-consumer-input.json`, and
`docs/development/sc-publish.md`. This sprint does not author
`.github/workflows/crates-publish.yml`. It may edit
`.github/workflows/crates.yml` only to remove a leftover spike reference.
a-5 already keeps that workflow free of a spike-named job.

## Dependencies and parallel safety

- `must_follow a-6`: consumes the template tree, the answers schema, and the
  `--var-file` driver. Those three artifacts are the generated application
  this release publishes against.
- `parallel_safe a-8`: release paths and wizard paths are disjoint; a-8 consumes
  the already-settled schema and does not modify published crates.

## Deliverables

1. **REQ-RUN-0103** — Preserve spike evidence, then delete the throwaway source
   and all active root references before tagging.
2. **REQ-RUN-0001 / REQ-RUN-0002 / REQ-RUN-0005** — Final four-member library
   workspace, metadata, licences, doctests, boundary validation, and the
   sc-publish kit. `install.py` writes the release workflows. `cargo publish
   --dry-run -p` passes for each leaf. The workspace dry-run for `sc-runtime`
   passes until the leaves exist on the registry. The kit
   `release-preflight.yml` dry-run passes. Publish order is the three leaves,
   then `sc-runtime`.
3. **REQ-RUN-0702** — Publish leaf crates before sc-runtime, then run the
   fixture matrix against registry versions at the intended tag commit.
4. **REQ-RUN-0003** — Generated Cargo metadata and lockfiles use registry
   versions with no path/git/patch substitute for the four libraries.
5. **REQ-RUN-0503** — Release README and evidence explain the supported core
   generation path and name deferred layers without promising them.
6. **ADR-RUN-0002** — Amend ADR-RUN-0002 in `docs/architecture.md`, the
   publish and release contract. Do not add a release section to
   ADR-RUN-0001. The amendment states that sc-publish `install.py` is the
   only release-workflow owner, that hand-authored publish workflows
   (including `crates-publish.yml`) are forbidden, and that the GitHub
   Release for tag `v0.1.0` includes a workspace source archive
   `sc-runtime-{version}-source.tar.gz` produced by the kit `release_targets`
   linux row. Phase A ships no release binaries.

## Acceptance criteria

1. `req:REQ-RUN-0002` — each crate's tests, doctests, and metadata checks pass.
   `cargo publish --dry-run -p` passes for `sc-config`, `sc-transport`, and
   `sc-command`. The workspace dry-run for `sc-runtime` passes until the leaves
   exist on the registry. The kit `release-preflight.yml` dry-run passes.
   Publish order is the three leaves, then `sc-runtime`.
2. `req:REQ-RUN-0702` — registry inspection shows all four publications,
   sc-runtime published after its leaf dependencies, and the generated lockfile
   records registry sources. The registry-backed fixture runs a-6's amended
   REQ-RUN-0701 sequence in release mode. This criterion does not add a second
   pin, `cargo tree`, or observability check.
3. `req:REQ-RUN-0103` — the tagged tree contains no spike source or stale build,
   workflow, recipe, or README reference.
4. `req:REQ-RUN-0503` — release notes distinguish the shipped core/template path
   from deferred auto-start, attribute-generated CLI tests, and wizard work.
5. `ADR-RUN-0002` — the sprint diff amends ADR-RUN-0002 and does not add a
   release section to ADR-RUN-0001. The amended decision names sc-publish as
   the sole workflow owner and forbids hand-authored publish workflows. The
   GitHub Release for the tag includes
   `sc-runtime-{version}-source.tar.gz` from the kit (linux `release_targets`
   row; `release_binaries` remains empty).

## This sprint does not close

The wizard and all explicitly deferred application-policy layers remain open.

## Paths to delete

- `examples/spike/**`

## Required validation

Run the acceptance criteria, including publish dry-runs and the registry-backed
fixture in release mode. Actual publication requires separate release
authorization at execution time.
