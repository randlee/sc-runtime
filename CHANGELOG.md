# Changelog

All notable changes to the sc-runtime SDK crates are documented here.
The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and the crates follow [Semantic Versioning](https://semver.org/) (pre-1.0:
a minor bump may break the API). All published crates share one version:
`sc-config`, `sc-transport`, `sc-command`, `sc-runtime`, `sc-cron` and
`sc-cron-commands`.

Releases before 0.3.0 are described in `site/announcements/`.

## [0.3.0] - Unreleased

### Breaking

- **sc-config:** `ConfigError` is now `#[non_exhaustive]` and has two new
  variants, `UnknownKey { path, key }` (code `CFG.UNKNOWN_KEY`) and
  `SourceFailed { origin, message }` (code `CFG.SOURCE`). (#157)
  *Migration:* add a wildcard arm (`_ => ...`) to every exhaustive `match`
  on `ConfigError`. You can also branch on `ConfigError::code()`, which is
  stable.
- **sc-config:** `ConfigError::MissingDefault` now means "a required
  configuration file is absent", for any required file, not only
  `default.json`. Its `Display` text changed from `default configuration file
  not found at <path>` to `required configuration file not found: <path>`,
  and its recovery text changed too. The code `CFG.MISSING_DEFAULT` did not
  change. (#157)
  *Migration:* match on the variant or on `code()`, not on the message text.
- **All crates:** the shared version is now 0.3.0. Update every `sc-*`
  requirement together (for example `sc-config = "0.3.0"`). A consumer that
  mixes 0.2.x and 0.3.x `sc-*` crates gets two copies of the shared types.
- **All crates (MSRV):** the workspace now uses one Rust toolchain,
  1.98.1. `rust-version` went from 1.94.1 to 1.98.1, and the release
  toolchain in `release/` matches. (#158)
  *Migration:* build with rustc >= 1.98.1 (`rustup toolchain install
  1.98.1`), and raise `rust-version` in your own manifests if you declare
  one.
- **sc-cron (`monty` feature):** `monty` and `monty-types` went from 1.4 to
  1.6. The 1.6 crates are the reason for the rustc 1.98 requirement. (#158)
  *Migration:* if you depend on `monty` directly next to `sc-cron`'s
  `monty` feature, move to 1.6 too so only one copy resolves.
- **sc-cron, sc-runtime-db-sqlite (SQLite):** `rusqlite` went from 0.39 to
  0.43, which resolves `libsqlite3-sys` 0.40. (#158)
  *Migration:* a consumer that also uses `sqlx` with SQLite needs
  `sqlx >= 0.9.1`, the first release that accepts `libsqlite3-sys` 0.40
  (`>=0.30.1, <0.41.0`). Any direct `rusqlite` dependency must move to 0.43,
  because the native `links = "sqlite3"` library can only resolve once.

### Added

- **sc-config:** a layered configuration engine that callers can extend.
  `Layers` stacks sources that implement the public (unsealed) `LayerSource`
  trait. A layer pushed later takes precedence over earlier ones. `Layers`
  resolves them into `Resolved` and then into the caller's type. (#157;
  design in #154, ADR-CFG-0004/0005, REQ-CFG-0005..0010)
- **sc-config:** built-in sources:
  - `FileSource`, for JSON files: required or optional. An `accepts` list
    rejects keys outside it with `ConfigError::UnknownKey`.
  - `EnvSource`, for environment variables: a literal prefix plus aliases.
  - `ValueSource`, for values supplied in code.
  (#157)
- **sc-config:** per-leaf provenance. `Resolved::rows()` returns
  `ResolvedRow` entries that record, for each key, which layer supplied its
  value. Each `Source` carries an open `SourceKind`. (#157)
- **sc-config:** `StandardLayers`, `StandardContext` and `Slot` give a fixed
  standard order of 7 slots: `Defaults`, `System`, `User`, `Project`,
  `Local`, `Env`, `Flags`. `insert(Slot, source)` adds a source to a slot.
  The user folder is found in this order: `with_home(dir)`; then the
  `<FAMILY>_HOME` variable (for example `SC_AGENT_HOME` for family
  `sc-agent`), which `with_home_var` can rename; then `<home>/.<family>/`.
  (#157)
- **sc-runtime:** `testing::instrument_router` is back behind the
  `test-util` feature, with its former five-argument signature. Published
  0.2.0 had removed it. It applies the current timeout and
  request-tracking/logging middleware, so tests can choose a short HTTP
  deadline. (#131)

### Changed

- **sc-config:** `Loader` is now a preset stack on the new `Layers` engine,
  so there is a single loading pipeline. Its observable behaviour is the
  same: the existing tests pass unedited. Dependencies are still only
  `serde` and `serde_json`. Loading is still synchronous. (#157)
- **template:** generated projects pin `sc-*` crates at 0.3.0. (this
  release)
- **template:** generated projects declare `rust-version = "1.98.1"`, and
  generated CI uses 1.98.1. The daemon template's `main.rs` adds
  `#![recursion_limit = "256"]`, which rustc 1.98 needs to lay out the
  daemon future when the MySQL store is generated. (#158)
  *Migration (existing generated projects):* add
  `#![recursion_limit = "256"]` to the daemon crate root if
  rustc 1.98 reports a query-depth overflow.
- **sc-command, sc-runtime, template daemon:** `rmcp` went from 3.8 to 3.9.
  (#158)
- **Compatibility rule:** anchor ANC-006 was amended to v4 with owner
  approval. `rusqlite` now tracks the newest release whose `libsqlite3-sys`
  the consuming repository's `sqlx` accepts. (#158)
- **template:** the generated consumer upgrades to sc-lint 0.8.6, with a
  synchronized bootstrap. The generated SQLite concurrency regression now
  uses two independent stores. (#129)

### Fixed

- **sc-runtime:** downstream tests that relied on
  `testing::instrument_router` compile again (see Added). (#131)

### Security

- RUSTSEC-2026-0459: `rusqlite` 0.39 -> 0.43 and `libsqlite3-sys`
  0.37 -> 0.40 (see the SQLite migration note under Breaking). (#158)
- RUSTSEC-2026-0462: `rmcp` 3.8 -> 3.9.0, in the workspace, the template
  daemon and the contract/consumer fixtures. (#158)
- RUSTSEC-2026-0592: `url` 2.5.8 -> 2.5.9, in all lockfiles. (#158)
- RUSTSEC-2026-0543 (chrono is unmaintained; this is not a vulnerability)
  is ignored in `deny.toml`. No chrono-free release exists: `cron` 0.17,
  `rmcp` 3.9 (through `schemars` 1.2.3) and `sc-cron` itself use chrono.
  Replacing chrono is tracked in #159. (#158)
- `deny.toml`: removed the stale ignores RUSTSEC-2024-0436 (paste) and
  RUSTSEC-2023-0089 (atomic-polyfill, monty 1.4). No crate in the resolved
  graph matches them anymore. (this release)

### Internal (not user-visible)

- Repository tooling, agent and QA-process updates: #134, #137, #138, #139,
  #141, #142, #143, #145, #146, #147, #150, #155, #156. CI fixes for template
  generation (merge 2052e9e).
- Documentation and planning only: Phase C closeout (#126), v0.2.0
  announcement (#125, #127), honest-work guidance (#130), PRD v2.1 (#148),
  Slotted Nanoservice PRD draft (#144, #149), sc-config layered design
  (#154).

[0.3.0]: https://github.com/randlee/sc-runtime/compare/v0.2.0...develop
