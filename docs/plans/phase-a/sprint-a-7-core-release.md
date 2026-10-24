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
  - ADR-RUN-0001
  - ADR-RUN-0002
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
  - "release/**"
  - ".github/workflows/release*.yml"
  - "docs/releases/v0.1.0.md"
  - "docs/development/sc-publish.md"
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
`sc-observability-types` versions match the a-1 record. Repeat the a-6
`cargo tree -p cli -e normal` check on that fixture as release evidence under
REQ-RUN-0702. Vend **sc-publish** from `release/sc-publish-pin.toml` via
`install.py` and `release/sc-publish-consumer-input.json` (GitHub Release +
crates.io); see `docs/development/sc-publish.md`. `.github/workflows/release*.yml`
and any `release_*.py` are `install.py` output, committed as generated files
and not hand-edited. The only authored release inputs are
`release/sc-publish-pin.toml`, `release/sc-publish-consumer-input.json`, and
`docs/development/sc-publish.md`. This sprint does not author
`.github/workflows/crates-publish.yml`.

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
6. **REQ-RUN-0702** — Release evidence, not a second acceptance root. The
   oracle for the sc-lint 0.8.0 pin is the a-6 consumer install and the a-5
   maintainer pin: `template/sc-lint.toml` pins `minimum_version` to `0.8.0`,
   `template/.sc-lint/bootstrap.py` is the matching helper, and the
   registry-backed fixture runs `just setup && just lint` before test and
   build. The oracle for CLI isolation is a-6's NFR-RUN-0001 close:
   `cargo tree -p cli -e normal` on that fixture lists none of the server-only
   crates that check names. The oracle for observability is a-6's
   REQ-RUN-0309 close and the a-1 record: the template daemon requirement and
   the fixture lockfile match those versions, and the daemon manifest does
   not require `sc-observability-otlp`.

## Acceptance criteria

1. `req:REQ-RUN-0002` — each crate's tests, doctests, and metadata checks pass.
   `cargo publish --dry-run -p` passes for `sc-config`, `sc-transport`, and
   `sc-command`. The workspace dry-run for `sc-runtime` passes until the leaves
   exist on the registry. The kit `release-preflight.yml` dry-run passes.
   Publish order is the three leaves, then `sc-runtime`.
2. `req:REQ-RUN-0702` — registry inspection shows all four publications,
   sc-runtime published after its leaf dependencies, and the generated lockfile
   records registry sources. The same criterion is the only release-evidence
   root for the earlier closures, and it does not re-open them. The sc-lint
   0.8.0 pin matches the a-6 consumer install: `template/sc-lint.toml` pins
   `minimum_version` to `0.8.0`, `template/.sc-lint/bootstrap.py` is that
   helper, and `just setup && just lint` exits 0 on the registry-backed
   fixture. The a-6 NFR-RUN-0001 oracle passes: `cargo tree -p cli -e normal`
   on that fixture omits every server-only crate that check names. The a-6
   REQ-RUN-0309 oracle passes: template daemon dependency text and the fixture
   lockfile match the a-1 recorded `sc-observability` and
   `sc-observability-types` versions, and the daemon manifest does not require
   `sc-observability-otlp`.
3. `req:REQ-RUN-0103` — the tagged tree contains no spike source or stale build,
   workflow, recipe, or README reference.
4. `req:REQ-RUN-0503` — release notes distinguish the shipped core/template path
   from deferred auto-start, attribute-generated CLI tests, and wizard work.

## This sprint does not close

The wizard and all explicitly deferred application-policy layers remain open.

## Paths to delete

- `examples/spike/**`

## Required validation

Run final root validation after spike removal, package and publish dry-runs,
publication in dependency order, registry-backed fixture generation,
`just setup && just lint` in that generated project, generated
lockfile-source inspection, the sc-lint 0.8.0 pin check on
`template/sc-lint.toml` and `template/.sc-lint/bootstrap.py`, the a-6 CLI
isolation check (`cargo tree -p cli -e normal`), the a-1 observability version
check on the template and fixture lockfile, the kit `release-preflight.yml`
dry-run, and exact commit/tag checks. Those pin, isolation, and observability
commands are REQ-RUN-0702 release evidence. Actual publication requires
separate release authorization at execution time.
