# Phase E: Release

**Version target:** 0.1.0
**Entry state:** Phase D complete (v0.1.0-beta.1 tagged)
**Exit state:** All workspace crates published to crates.io at v0.1.0; three SC consumer tools updated to published version

Related: [Project Plan](../project-plan.md) · [Phase D](../phase-D/phase-D-plan.md) · [PRD §14 Phase D (consumer onboarding)](../../prd/sc-runtime-prd.md)

---

## Objectives

Phase E is a hardening and release phase, not a feature phase. No new capability is added to the builder, no new crates are introduced (with the exception of the `sc-runtime-db-fsqlite` placeholder already created in Phase B). The objectives are:

- The sc-lint-version gate (`cargo-semver-checks`) passes as a mandatory release gate — no publish without it
- xwin cross-compilation CI validates Windows compilation from a macOS or Linux runner, catching named-pipe and `#[cfg(windows)]` failures before they reach consumers
- The public API surface is audited for semver stability; anything not ready for stability is sealed or pub(crate)-restricted before the 0.1.0 tag
- All workspace crates publish to crates.io in dependency order at v0.1.0
- Consumer tools (atm-core, Continuity, ci) are updated from `[patch.crates-io]` co-development mode to the published crates.io versions

---

## Tasks

| Task | Description |
|------|-------------|
| sc-lint-version | Add `sc-lint check full` (which includes `sc-lint-version` + `cargo-semver-checks`) to CI as a blocking check on the release workflow; must pass before the release tag is applied |
| xwin CI | Add `sc-lint check xwin` as a step in the CI workflow; runs `cargo xwin check` and `cargo xwin clippy` targeting `x86_64-pc-windows-msvc` from the macOS runner; any error blocks the release |
| API surface review | Audit all `pub` items across the workspace for semver stability intent; seal or restrict anything not ready for public stability commitment; document justified exceptions with `#[sc_lint(allow = ...)]` |
| README.md | Consumer-facing workspace README: what sc-runtime is, the composability matrix, quick start (Phase A pattern), link to docs/ for depth |
| CHANGELOG.md | Generated from git log covering all alpha and beta iterations; formatted per Keep a Changelog conventions |
| Dependency audit | `cargo audit` must report no vulnerabilities; review all dependency minimum version constraints and tighten where needed; escalate `[bans] duplicates = "warn"` to `"deny"` in `deny.toml` |
| Publish sequence | Publish in strict dependency order to crates.io (see sequence below) |
| sc-runtime-db-fsqlite | Publish the placeholder crate (boundary file, FUTURE annotation, zero implementation code) to reserve the crate name on crates.io before another party can claim it |
| Post-publish | Update atm-core, Continuity, and ci Cargo.toml files to remove `[patch.crates-io]` entries and replace with `sc-runtime = "0.1"`, `sc-runtime-core = "0.1"`, etc. |

---

## sc-lint-version Gate

`sc-lint-version` wraps `cargo-semver-checks` and treats any semver violation as a hard error. It must be wired as a blocking check on the release tag workflow. The check is distinct from the `sc-lint lint ci` gate used in regular development (which does not include the version check).

The CI workflow should have two lint jobs:

- `lint-dev`: runs `sc-lint lint ci` on every push and pull request
- `lint-release`: runs `sc-lint check full` only when a release tag is pushed; blocks the release on any failure

If `sc-lint-version` is not yet available in the installed sc-lint version, the release is blocked until it is. This is stated as a hard policy in ADR-14: a shared infrastructure crate with multiple consumers has maximum blast radius from a silent breaking change, making the version gate the highest-priority release gate.

---

## xwin CI Configuration

xwin validates Windows cross-compilation from macOS/Linux before the release. It runs as a separate CI job named `xwin` on the macOS runner:

```yaml
xwin:
  runs-on: macos-latest
  steps:
    - uses: actions/checkout@v4
    - name: Install sc-lint
      run: brew install randlee/tap/sc-lint
    - name: Install cargo-xwin
      run: cargo install cargo-xwin
    - name: Install Windows target
      run: rustup target add x86_64-pc-windows-msvc
    - name: sc-lint check xwin
      run: sc-lint check xwin
```

xwin remains in the `full` profile but is not in the `ci` (development) profile. The rationale from ADR-11: real Windows CI (`windows-latest` runner) remains authoritative for Windows correctness. xwin is a pre-release signal, not a replacement for Windows CI. If Windows CI is added to the workflow later, xwin becomes redundant and can be removed from the release gate.

---

## API Surface Review

The audit covers all `pub` items in every workspace crate. For each item, the question is: "If this changes in 0.1.1, is it a breaking change?" Items that are not ready for that commitment must be restricted before 0.1.0:

- Traits intended for internal use only: add sealed supertrait if not already sealed
- Types that are implementation details: restrict to `pub(crate)` or `pub(super)`
- Experimental builder methods: document as unstable with `#[doc(hidden)]` if needed for internal wiring but not consumer-stable
- Error variant sets: ensure every error enum is `#[non_exhaustive]` or genuinely stable; add `#[non_exhaustive]` to error enums where variant sets may grow

The `Plugin` trait is intentionally open — consumers implement it. Its method signatures are therefore a hard semver commitment from 0.1.0. Ensure the signatures are final before tagging.

The `StorageBackend` trait is sealed — consumers cannot implement it. Changes to its methods are still semver-breaking (consumers hold `Arc<dyn StorageBackend>` and call the methods), but the sealed design means the blast radius is limited to call sites, not implementations.

---

## Dependency Audit

Before publishing:

- Run `cargo audit` — must report zero vulnerabilities; any advisory that cannot be resolved blocks the release
- Review every dependency's minimum version: ensure the versions declared in `Cargo.toml` are actually the minimum that compile and pass tests; over-loose minimums cause resolution problems for consumers
- Escalate `[bans] duplicates = "deny"` in `deny.toml` — duplicate transitive dependencies are an error, not a warning, before publishing shared infrastructure
- Run `cargo deny check` — must pass completely

---

## Publish Sequence

Publish in strict dependency order. Each crate must be published and visible on crates.io before the next publish begins (allow ~30 seconds between publishes for index propagation).

1. `sc-runtime-core` — foundation; no workspace dependencies
2. `sc-runtime-db` — depends on `sc-runtime-core`
3. `sc-runtime-db-sqlite` — depends on `sc-runtime-db`
4. `sc-runtime-db-sqlx` — depends on `sc-runtime-db`
5. `sc-runtime-db-fsqlite` — depends on `sc-runtime-db` (placeholder; publishes to reserve name)
6. `sc-runtime-cli` — depends on `sc-runtime-core`
7. `sc-runtime-mcp-stdio` — depends on `sc-runtime-core`, `sc-runtime-cli`
8. `sc-runtime-daemon` — depends on `sc-runtime-core`, `sc-runtime-cli`
9. `sc-runtime-web` — depends on `sc-runtime-core`, `sc-runtime-daemon`
10. `sc-runtime` — depends on all of the above

Before publishing any crate, run `cargo publish --dry-run -p <crate-name>` to verify the package is well-formed. If dry-run fails, fix the issue before proceeding.

After all crates publish, run `cargo update` in a clean checkout to verify the published versions resolve correctly from crates.io with no `[patch]` blocks.

---

## Consumer Updates (Post-Publish)

After crates.io publish is confirmed, update each consumer tool:

**atm-core**: remove `[patch.crates-io]` entries; replace with `sc-runtime-core = "0.1"`, `sc-runtime-cli = "0.1"`, `sc-runtime-daemon = "0.1"`, and any other used crates; run `cargo test`; run `sc-lint lint fast`; verify all tests pass.

**Continuity**: remove `[patch.crates-io]` entries; update to published versions; run `cargo test`; run `sc-lint lint fast`.

**ci**: remove `[patch.crates-io]` entries; update to published versions; run `cargo test`; run `sc-lint lint fast`.

The recommendation is to update atm-core first (smallest surface area), confirm it works end-to-end, then proceed to Continuity and ci. This isolates any post-publish resolution issues to one repo at a time.

---

## README.md

The workspace README covers:

- One paragraph on what sc-runtime is and who it is for
- The composability matrix (from the PRD) showing all valid configurations
- Quick start section: the Phase A pattern (minimal CLI with one custom command) in under 20 lines of Rust
- Links to `docs/` for architecture, requirements, and the phase plan
- Environment variables table: `SC_RUNTIME_HOME`, `SC_RUNTIME_DB`
- Links to crates.io pages for each workspace crate

The README must enable a new consumer to understand what sc-runtime does and run the quick start pattern in under 10 minutes.

---

## Exit Criteria

All of the following must be true before the v0.1.0 tag is applied:

- `sc-lint check full` passes — includes `sc-lint-version` (cargo-semver-checks); this is a hard gate
- `sc-lint check xwin` passes in the release CI job — Windows cross-compilation clean from macOS runner
- `cargo publish --dry-run` succeeds for all 10 workspace crates
- `cargo audit` reports zero vulnerabilities
- `cargo deny check` passes with duplicates at `"deny"` level
- All workspace crates published to crates.io at v0.1.0 in the correct dependency order
- README.md enables a new consumer to reach working code in under 10 minutes
- At least one SC consumer tool (recommendation: atm-core) has removed its `[patch.crates-io]` block and is using the published v0.1.0 from crates.io with all tests passing
- All Phase A, B, C, and D exit criteria still pass on the 0.1.0 commit
- Git tag `v0.1.0` applied to the passing commit
