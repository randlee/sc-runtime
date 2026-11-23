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
- **template:** the generated consumer upgrades to sc-lint 0.8.6, with a
  synchronized bootstrap. The generated SQLite concurrency regression now
  uses two independent stores. (#129)

### Fixed

- **sc-runtime:** downstream tests that relied on
  `testing::instrument_router` compile again (see Added). (#131)

### Security

- No dependency security fixes are in this release yet. The RUSTSEC
  dependency updates in #158 had not merged into `develop` when this
  changelog was written. Add them here if they land before the release.

### Internal (not user-visible)

- Repository tooling, agent and QA-process updates: #134, #137, #138, #139,
  #141, #142, #143, #145, #146, #147, #150, #155, #156. CI fixes for template
  generation (merge 2052e9e).
- Documentation and planning only: Phase C closeout (#126), v0.2.0
  announcement (#125, #127), honest-work guidance (#130), PRD v2.1 (#148),
  Slotted Nanoservice PRD draft (#144, #149), sc-config layered design
  (#154).

[0.3.0]: https://github.com/randlee/sc-runtime/compare/v0.2.0...develop
