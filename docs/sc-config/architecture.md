# `sc-config` Architecture

**ID Range:** ADR-CFG-0001 through ADR-CFG-0004  
**Status:** Draft  
**Created:** 2026-09-19  
**Last Updated:** 2026-11-22  
**Owner:** sc-runtime maintainers  

---

Crate-level architecture and ADRs for `sc-config`. Repo-level architecture is in
[`../architecture.md`](../architecture.md); this crate's requirements are in
[`requirements.md`](requirements.md); its boundary manifest is
`boundaries/sc-config/`. ADRs follow
the shared SC requirement and ADR templates. Only ADRs with **Status: Active**
are binding; an Active ADR changes only through a later ADR that names it. An
ADR with **Status: Proposed** binds nothing, is not cited as governing by any
sprint plan, and is not enforced by `arch-qa` until a later change makes it
Active.

## Role

A leaf crate. It reads files and an environment source and returns the caller's
type, or a typed error. It knows nothing about daemons, transport, envelopes or
observability, and nothing in this workspace is a dependency of it.

## Public surface

| Item | Purpose |
|---|---|
| `AppName` | validated configuration identity; `new` / `TryFrom<&str>` rejects with `APP.INVALID_NAME`. `env_prefix` upper-cases ASCII letters and maps both `-` and `_` to `_`, then appends `__`, so `my-app` and `my_app` share `MY_APP__` |
| `load::<T>(app: AppName) -> Result<T, ConfigError>` | convenience: `./config` and the process environment |
| `Loader::new(app, config_dir, environment) -> Loader` | stores an `AppName`, a `PathBuf`, and `Vec<(OsString, OsString)>` |
| `Loader::load::<T>() -> Result<T, ConfigError>` | same merge as `load`, using only those inputs |
| `ConfigError` | typed error enum; inventory below and [REQ-CFG-0004](requirements.md) |

Proposed additions, not binding until [ADR-CFG-0004](#adr-cfg-0004-the-caller-defines-the-layers-sc-config-provides-the-engine)
is Active: `Layers`, `Resolved`, `Layered<T>`, the unsealed `LayerSource`
trait and `Layer`; `FileSource`, `FormatError`, `EnvSource`, `KeyCase`,
`ValueMode`, `ValueSource`; `Source`, `SourceKind`, `KeyPath`, `Provenance`,
`ResolvedRow`; `Loader::layers()`; `ConfigError` becomes `#[non_exhaustive]`
and gains `MissingFile`, `TooLarge`, `ParseFormat`, `UnknownKey`,
`InvalidKeyPath`, `EnvSetting`, `SourceFailed`, `source_failed`,
`source_failed_with` and `report`.

## How loading works

This is `Loader`'s behaviour. Its rules are owned by
[REQ-CFG-0001](requirements.md) (files and `merge`) and
[REQ-CFG-0002](requirements.md) (environment); the steps below summarise
them. When [ADR-CFG-0004](#adr-cfg-0004-the-caller-defines-the-layers-sc-config-provides-the-engine)
is Active, the same steps run as `Loader`'s preset stack on the layered engine
([REQ-CFG-0008](requirements.md)).

1. Read `default.json` into a `serde_json::Value` (required).
2. Read `local.json` if present and merge it over the first value.
3. Keep environment entries whose encoded name starts with
   `AppName::env_prefix`. That prefix upper-cases ASCII letters and maps
   both `-` and `_` to `_`. Apply matching entries in ascending encoded-byte
   order of the variable name. Split the
   rest on `__` into a key path, parse the value as JSON or fall back to a
   string, and set it at that path. A longer name that extends a shorter one
   wins. An empty segment, or a path through a non-object, is
   `CFG.OVERRIDE`. A prefix match that is not Unicode is `CFG.NON_UNICODE`.
4. Deserialise the merged value into `T` once.

---

## ADR-CFG-0001: `sc-config` returns `ConfigError` values and never panics

**Status:** Active  
**Decision Date:** 2026-09-19  
**Source:** sc-runtime design, 2026-09-19; applies the repository rule that library errors are typed values ([ADR-RUN-0006](../architecture.md)); the `ConfigError` variant set is decided in this document  

**Proposed amendment:** [ADR-CFG-0004](#adr-cfg-0004-the-caller-defines-the-layers-sc-config-provides-the-engine) item 5 amends items 3 and 4 and adds the inventory rows marked Proposed below.

### Context

`sc-config` (`crates/sc-config`) is the standalone configuration crate: it
reads JSON files, applies environment-variable overrides, and deserialises the
result into the caller's serde type. A program calls it first in `main`,
before logging or any other error reporting is initialised. Its failures are
user mistakes, such as a typo in a JSON file or a missing file, far more often
than bugs. A panic at that point prints a Rust backtrace that tells the user
nothing about which file to fix, and an agent driving the program cannot parse
it. The design states that every `sc-config` method returns a discriminated
union, in Rust an enum such as `Result<T, ConfigError>` with a typed error
enum, and that there are no panics. The design also limits the crate's
dependencies to `serde` and `serde_json`, which rules out error-helper crates.

### Decision

1. Every public function and method of `sc-config` that can fail returns
   `Result<T, ConfigError>`.
2. `ConfigError` is one public enum, written by hand in `crates/sc-config`.
   It implements `std::fmt::Debug`, `std::fmt::Display` and
   `std::error::Error` without a derive macro from another crate.
3. Decided in this document: `ConfigError` has a distinct variant for each
   of: `default.json` missing; a
   file that exists but cannot be read; a file that is not valid JSON; an
   environment override whose key path passes through a non-object value; the
   merged document failing to deserialise into the caller's type.
4. Each variant carries, as typed fields, the file path or the environment
   variable name at fault ([REQ-CFG-0004](requirements.md)).
5. No code reachable from the public API panics. The repository rule
   ([NFR-RUN-0009](../requirements.md)) applies with no commented-exception
   allowance: no `unwrap`, `expect`, `panic!`, `unreachable!`, `todo!`,
   `unimplemented!`, or panicking `[]` indexing, reachable from a public
   function.
6. Stricter than the repository rule, decided in this document: `sc-config`
   also bans `assert!`, `assert_eq!` and `assert_ne!` in code reachable from a
   public function, and non-test code reads the process environment with
   `std::env::vars_os`, not `std::env::vars`, which panics on a variable that
   is not valid Unicode.
7. `thiserror` and `anyhow` are not dependencies of `sc-config`.
8. `Loader::new` only stores a validated `AppName`, a directory, and the
   caller's environment pairs, so it returns `Loader`. `AppName::new` can
   fail and returns `Result<AppName, ConfigError>`. Only operations that can
   fail return `Result` ([NFR-CFG-0001](requirements.md)).
9. `sc_config::AppName` uses the shared predicate: ASCII `A-Z`, `a-z`, `0-9`,
   `-`, and `_`, length 1 through 64 bytes. Display is `as_str` or
   `AsRef<str>`. The type does not implement `Deref`. Invalid names are
   `APP.INVALID_NAME`.

Error inventory. One row per `ConfigError` variant. The cause payload is the
variant's typed fields. Path, line, and column stay on the variants that
carry a source location.

| Failure mode | Stable code | Cause payload | Recovery steps |
|---|---|---|---|
| Application name rejected at construction | `APP.INVALID_NAME` | `InvalidName.name`, the rejected string | Use 1 to 64 bytes of ASCII letters, digits, `-`, and `_`. |
| `default.json` is absent | `CFG.MISSING_DEFAULT` | `MissingDefault.path`, the full path looked up | Create that file and load again. |
| A present file cannot be read | `CFG.READ` | `Read.path` and `Read.source` (`std::io::Error`) | Make the file readable, or replace a directory at that path with a file. |
| File content is not valid JSON, including non-UTF-8 | `CFG.PARSE` | `Parse.path`, `Parse.line`, `Parse.column`, and `Parse.message` | Fix the JSON at that line and column. The file must be UTF-8. |
| Override path is empty or walks through a non-object | `CFG.OVERRIDE` | `Override.variable`, the full variable name | Drop the empty segment, or change the name so the path does not descend through a scalar, array, or null. |
| Merged document does not match `T` | `CFG.DESERIALIZE` | `Deserialize.message`, `Deserialize.files`, and `Deserialize.variables` | Align the merged JSON with `T`. The listed files and variables are the inputs that were applied. |
| Prefix-matching environment entry is not Unicode | `CFG.NON_UNICODE` | `NonUnicode.variable`, the name as `OsString` | Use a Unicode name and value. Names that do not start with the prefix are ignored. |
| (Proposed, ADR-CFG-0004) A required non-default file is absent | `CFG.MISSING_FILE` | `MissingFile.path` | Create that file, or mark the source optional. |
| (Proposed) A capped file exceeds its limit | `CFG.TOO_LARGE` | `TooLarge.path`, `TooLarge.limit` | Shrink the file or raise the limit the program set. |
| (Proposed) A caller-supplied format parser failed | `CFG.PARSE_FORMAT` | `ParseFormat.path`, `.message`, optional `.line` and `.column` | Fix the file at the reported place. |
| (Proposed) A file holds a key outside its accepted set | `CFG.UNKNOWN_KEY` | `UnknownKey.path`, `UnknownKey.key` | Remove the key from that file, or set it in the layer that accepts it. |
| (Proposed) A key path is empty or has an empty segment | `CFG.KEY_PATH` | `InvalidKeyPath.input`, `.reason` | Give a non-empty path with non-empty segments; quote a segment that contains `.`. |
| (Proposed) An environment source setting is empty | `CFG.ENV_SETTING` | `EnvSetting.setting`, `.value` | Give a non-empty prefix, separator or alias name. |
| (Proposed) A consumer-defined source failed | `CFG.SOURCE` | `SourceFailed.origin`, `.message`, optional cause | Follow the message from that source. |
| (Proposed) Deserialize also carries sources | `CFG.DESERIALIZE` | adds `Deserialize.sources`, every contributing source in precedence order | Align the merged JSON with `T`; `Layers::resolve` shows which source set each key. |

### Consequences

- The `main.rs` of a generated project matches on the result of
  `sc_config::load(AppName::new("my-app")?)`, prints the error, and returns an exit code. It
  needs no panic handler.
- Callers can branch on the variant, for example to treat a missing
  `default.json` differently from a malformed one.
- The hand-written `Display` and `Error` implementations cost a few dozen
  lines. That cost is what keeps the dependency list at two crates.
- Adding a failure mode later means adding a variant, which is a breaking
  change for callers that match exhaustively.

### Alternatives Considered

- `thiserror` to derive `Display` and `Error`. Rejected: it adds a third
  dependency and a proc-macro compile step to save a few dozen lines for one
  enum.
- `anyhow::Error` as the public error type. Rejected: it is opaque, so a
  caller cannot branch on the failure or read the file path as data, and it is
  also an extra dependency.
- Panicking on a missing `default.json`, on the grounds that the program
  cannot continue anyway. Rejected: whether to continue is the caller's
  decision, and a panic before logging exists is the least diagnosable failure
  a program can have.

### Implementation

**Enforced by:** the `req-qa` review agent (`.claude/agents/req-qa.md`)
checking code changes against [NFR-CFG-0001](requirements.md), which holds the
grep patterns and malformed-input tests; the `rust-best-practices-agent`
review agent (`.claude/agents/rust-best-practices-agent.md`) checking for
panicking constructs and opaque error types in the public API.

### Related Documents

- [NFR-CFG-0001](requirements.md): public API returns
  `Result<T, ConfigError>`; no panics
- [REQ-CFG-0004](requirements.md): the `ConfigError` variants and the data
  each carries
- [ADR-RUN-0006](../architecture.md): the repository-wide rule that public
  library APIs return typed errors and do not panic
- [NFR-RUN-0009](../requirements.md): owner of the list of banned panicking
  constructs for the four library crates

---

## ADR-CFG-0002: Merge layers as `serde_json::Value`, deserialise once

**Status:** Active  
**Decision Date:** 2026-09-19  
**Source:** sc-runtime design, 2026-09-19 (JSON files with environment overrides); merge strategy and environment-variable naming scheme decided in this document  

**Proposed amendment:** [ADR-CFG-0004](#adr-cfg-0004-the-caller-defines-the-layers-sc-config-provides-the-engine) item 5 amends the Context, item 3 and item 6. Items 1, 2, 4 and 5 stay binding, and the Enforced-by check of one `serde_json::from_value::<T>` call applies to the single call after `Layers::resolve()`.

### Context

`sc-config` (`crates/sc-config`) builds one configuration from three layers,
lowest precedence first: `config/default.json` (checked in),
`config/local.json` (gitignored, optional), and environment variables. The
result must be the caller's own serde type, for example a generated project's
`AppConfig` struct. The design fixes the layers and the JSON format. It does
not say how layers are combined, and it states no naming scheme for the
environment variables. The naming scheme `<APP>__<KEY>__<KEY>` is decided in
this document ([REQ-CFG-0002](requirements.md)).

Layering can be done in two places. It can be done on typed structs: each
layer is deserialised into a "partial" version of the caller's type in which
every field is `Option`, and the partials are merged field by field. Or it can
be done on untyped JSON before any typed deserialisation happens. The typed
route forces every project to write and maintain a second, parallel struct for
each config struct, or forces this crate to ship a derive macro to generate
one.

### Decision

1. Each file is parsed into a `serde_json::Value`.
2. The `local.json` value is merged over the `default.json` value as untyped
   JSON: two objects merge key by key, recursively; in every other case the
   upper layer's value replaces the lower layer's value, arrays and `null`
   included ([REQ-CFG-0001](requirements.md)).
3. Each environment override is applied to the same `serde_json::Value`: the
   variable name gives a key path, the variable's value is parsed as JSON or
   else taken as a string, and it is set at that path
   ([REQ-CFG-0002](requirements.md)).
4. The final `serde_json::Value` is deserialised into the caller's
   `T: serde::de::DeserializeOwned` exactly once, with
   `serde_json::from_value`. No layer is deserialised into `T` on its own.
5. `sc-config` defines no schema, no partial-struct trait and no derive macro.
6. The only file format is JSON.

### Consequences

- A project's config type is one ordinary `#[derive(Deserialize)]` struct.
  Its serde attributes, `#[serde(default)]` values and any validation in its
  `Deserialize` implementation govern the result.
- `local.json` and the environment may hold any subset of keys, because no
  layer has to be a complete `T`.
- A type error surfaces once, at the end, against the merged document. The
  value's origin is no longer known at that point, so the error cannot name
  the one file at fault. The deserialisation variant of `ConfigError`
  therefore carries the list of files that were read and the names of the
  environment variables that were applied
  ([REQ-CFG-0004](requirements.md)).
- Syntax errors are still attributed exactly, because each file is parsed on
  its own before merging.
- The whole configuration is held in memory as a `serde_json::Value` during
  the load. Config files are small, so this is acceptable.

### Alternatives Considered

- Partial-struct merging: deserialise each layer into an all-`Option` copy of
  the caller's type and merge field by field. Rejected: every project would
  maintain a parallel struct per config struct, or this crate would need a
  proc-macro, which breaks the two-dependency limit.
- A bespoke configuration language with its own layering syntax. Rejected: it
  is more code and a new format for users to learn, where JSON plus a merge
  function does the job.
- TOML or YAML files. Rejected: the design specifies JSON, and each would add
  a parser dependency beyond `serde` and `serde_json`.

### Implementation

**Enforced by:** the `arch-qa` review agent (`.claude/agents/arch-qa.md`),
which checks every plan and code change touching `crates/sc-config` against
this decision: one `serde_json::from_value::<T>` call on the merged value, no
per-layer typed deserialisation, no partial-struct trait or macro, no parser
other than `serde_json`. The merge and override unit tests required by
[REQ-CFG-0001](requirements.md) and [REQ-CFG-0002](requirements.md) check the
behaviour.

### Related Documents

- [REQ-CFG-0001](requirements.md): file layers and the `merge` rules
- [REQ-CFG-0002](requirements.md): environment-variable naming, value parsing
  and path rules
- [REQ-CFG-0004](requirements.md): the deserialisation error lists the
  contributing files and variables

---

## ADR-CFG-0003: `sc-config` is synchronous with two dependencies

**Status:** Active  
**Decision Date:** 2026-09-19  
**Source:** sc-runtime design, 2026-09-19 (dependency list; reload, notification and async deferred)  

### Context

`sc-config` (`crates/sc-config`) is linked into every daemon and every CLI
built on this repository, and is meant to be usable by any Rust program on its
own. It is the first call in `main`. Automatic reloading, change notification
and async interop are wanted eventually. Building them now would bring an
async runtime (`tokio`) and a file watcher into every program that only reads
its configuration once at start-up, including small CLIs where compile time
and binary size matter. The design lists the crate's dependencies as `serde`
and `serde_json`, states that `sc-observability` and its OTel export crate
(`sc-observability-otlp`) stay independent of `sc-config`, and records the
reload features as a note for later, possibly as a separate
`sc-config-tokio` crate.

### Decision

1. The `[dependencies]` table of `crates/sc-config/Cargo.toml` contains
   exactly `serde` and `serde_json`.
2. `sc-config` does not depend on `tokio`, on `sc-observability`, on
   `sc-observability-otlp`, or on any other crate of this workspace
   (`sc-transport`, `sc-command`, `sc-runtime`).
3. The public API is synchronous: no `async fn`, no returned `Future`, no
   spawned thread or task.
4. `sc-config` does not watch files, reload, or notify callers of changes.
5. If reload, change notification or async interop is built later, it goes
   into a separate crate, possibly named `sc-config-tokio`, that depends on
   `sc-config`. `sc-config` never depends on that crate, and these features
   are not added to `sc-config` behind a Cargo feature.
6. `boundaries/sc-config/sc-config.toml` has an empty `allowed_dependents`
   list, and `allowed_dependencies` is `serde` and `serde_json`. Its
   `forbidden_edges` and the forbidden-name policy are owned by that manifest
   and [REQ-RUN-0005](../requirements.md); `cargo tree -p sc-config -e normal --prefix none`
   fails the crates.io names in that policy. `[public] facade` lists every
   `lib.rs` export, checked by the test `facade_lists_every_export`
   (`crates/sc-config/tests/facade.rs`), because `sc-lint` does not compare
   the facade with the crate. The process-test helper
   `sc-config-load-helper` is autodiscovered from `src/bin/`, excluded from
   the published package, and is not a facade item.

### Consequences

- A CLI pays nothing for configuration beyond `serde` and `serde_json`, which
  it already links for its request and response types.
- `load` can be called before an async runtime exists and from inside an
  async `main` alike. A load is a few small file reads, so blocking briefly at
  start-up is acceptable.
- Logging configuration can be read before logging is initialised, because
  this crate does not depend on `sc-observability`. The program passes the
  loaded values to `sc-observability` itself.
- Adding reload later is an additive new crate, not a breaking change to this
  one.
- A program that wants live reload in v0.1 must call `load` again itself.

### Alternatives Considered

- Async and reload support inside `sc-config` behind a Cargo feature.
  Rejected: cargo unifies the features of a crate across all packages built
  in one invocation, so under `cargo build --workspace` one crate enabling
  the feature would pull `tokio` into every other user of `sc-config` in that
  build, the CLI included; it also doubles the API surface to test. The same
  unification applies to the `server` feature of `sc-transport` and
  `sc-command`. Whether workspace-wide builds are exempt from the rule that a
  CLI links no server-side dependency, or those crates must be split into
  client and server crates instead, is recorded as **OPEN** in
  [NFR-RUN-0001](../requirements.md) and
  [ADR-RUN-0003](../architecture.md), to be measured by the spike. This ADR
  does not depend on the outcome: `sc-config` has no such feature either way.

### Implementation

**Enforced by:** `boundaries/sc-config/sc-config.toml`, checked by
`sc-lint lint sc-boundary`. That manifest's `forbidden_edges` lists
workspace-member targets only; this tree has none, so the list is empty.
[REQ-RUN-0005](../requirements.md) still owns the full forbidden-name policy.
The `Cargo.toml` inspection in [NFR-CFG-0002](requirements.md) and
`cargo tree -p sc-config -e normal --prefix none` fail a line beginning with
a forbidden crate name followed by a space, including
`sc-observability-types`. The `async fn` and `.await` grep in
[NFR-CFG-0003](requirements.md) covers the synchronous rule.

### Related Documents

- [NFR-CFG-0002](requirements.md): dependencies are `serde` and `serde_json`
  only
- [NFR-CFG-0003](requirements.md): synchronous API in v0.1; reload,
  notification and async out of scope
- [ADR-RUN-0007](../architecture.md): the repository is async end to end on
  Tokio, and records `sc-config` as the synchronous exception
- [REQ-RUN-0005](../requirements.md): owner of the workspace-internal edges
  and forbidden edges of all four crates
- [NFR-RUN-0001](../requirements.md) and [ADR-RUN-0003](../architecture.md):
  hold the OPEN on cargo feature unification under `cargo build --workspace`

---

## ADR-CFG-0004: The caller defines the layers; `sc-config` provides the engine

**Status:** Proposed  
**Decision Date:** 2026-11-22  
**Source:** extensibility review of PR #153 (`sc-config` `layers`, sc-agent team), 2026-11-22, and the owner's consumer-choice ruling of 2026-11-22. When Active, amends [ADR-CFG-0002](architecture.md) Context ("The design fixes the layers"), item 3 and item 6, and amends [ADR-CFG-0001](architecture.md) items 3 and 4 and its error inventory, as stated in Decision item 5  

### Context

ADR-CFG-0002 fixed three layers (`default.json`, `local.json`, environment)
and JSON. sc-agent needs more: command-line flags above the environment,
several files in its own order, aliased and string-typed variables,
per-file key sets and per-leaf provenance for `config show`. PR #153 met that
with a second public loader: a closed `Source` enum, a fixed
flags > env > files precedence, JSON only, a hard-coded 1 MiB cap,
exhaustive public structs, a separate `LayersError` with no codes, and a
per-layer `decode` into `T` that ADR-CFG-0002 item 4 forbids. The two
loaders already differ on environment ordering, file checks and error
semantics.

Each fixed choice is right for sc-agent and wrong for some other consumer: a
daemon that wants files over environment, a tool reading TOML, a service
with a secrets source, mixed-case keys. In-repo, `sc-config` is used by the
template CLI and daemon. They call `load` and `AppName`, and the daemon's
test `config_startup_uses_each_library_code_once`
(`template/crates/daemon/src/main.rs:258-290`) constructs every
`ConfigError` variant by literal.

### Decision

1. `sc-config` provides one layered engine over an ordered list of
   `LayerSource` trait objects, which receive the accumulated value
   read-only and return a layer. The caller chooses the sources and their
   order; `sc-config` assigns no source kind a rank. The requirements are
   [REQ-CFG-0005](requirements.md) through [REQ-CFG-0009](requirements.md)
   and [NFR-CFG-0004](requirements.md); this ADR does not restate them.
2. Behaviour a consumer may need differently is a builder parameter whose
   default equals `develop` at `9f61527`. The one exception is `merge`,
   which stays fixed because provenance, `replace_at` and REQ-CFG-0001 are
   defined against it; a consumer needing other rules combines values in its
   own source. Parameters and their consumers:

   | Parameter | Consumer |
   |---|---|
   | caller-ordered layers, `ValueSource`, `Source::flag`/`default_values` | sc-agent (flags over env over files; PR #153) |
   | `FileSource::accepts`, `UnknownKey` | sc-agent (REQ-SAG-0019, ADR-SAG-0021) |
   | `EnvSource::alias`, `alias_string`, `string_paths` | sc-agent (`SC_AGENT_MODEL`; REQ-SAG-0020) |
   | `Provenance::source_of`, `rows` | sc-agent (`config show`) |
   | `FileSource::optional`, `max_bytes`, `format`; `EnvSource::separator`, `key_case`, `value_mode`, `only` | owner's consumer-choice ruling, 2026-11-22: no consumer is limited to one consumer's choice |

3. ADR-CFG-0002 items 1, 2, 4 and 5 stay binding: layers fold as
   `serde_json::Value` with the one `merge`, and the result is deserialised
   into `T` once; provenance is computed from layer values, never by
   deserialising a layer.
4. `sc-config` never parses command lines (flags enter as a `ValueSource`)
   and stays synchronous ([NFR-CFG-0003](requirements.md)); `LayerSource`
   is `Send + Sync` and its `load` is synchronous.
5. Amendments, effective when this ADR is Active:
   - ADR-CFG-0002 Context: the layers are caller-defined; the
     `default.json`, `local.json`, environment stack is `Loader`'s preset
     ([REQ-CFG-0008](requirements.md)).
   - ADR-CFG-0002 item 3: overrides are applied by `EnvSource` to a working
     copy of the value accumulated below it, with the same rules, and enter
     the fold as a layer with `replace_at` paths; the result is identical.
   - ADR-CFG-0002 item 6: the built-in format is JSON and the crate still
     depends only on `serde` and `serde_json`; a `FileSource` accepts a
     caller-supplied parser.
   - ADR-CFG-0001 items 3 and 4 and its inventory: `ConfigError` gains the
     variants of [REQ-CFG-0009](requirements.md) item 4, becomes
     `#[non_exhaustive]`, and `Deserialize` gains `sources`. The inventory
     rows are added to the ADR-CFG-0001 table, which stays the only
     inventory. `MissingDefault` and `CFG.MISSING_DEFAULT` are unchanged.

### Consequences

- sc-agent builds its stack in its own code, for example
  `Loader::layers()` plus a `ValueSource` of flags, or a custom
  `[files…, EnvSource with aliases, flags]`. Its order lives in sc-agent.
- PR #153 is reworked onto this design before it merges: its `Inputs`,
  `Key`, `LayersError`, closed `Source` and per-layer `decode` are replaced.
  sc-agent 0.3.x keeps building from tag `sc-config-pin-0.3` (`eb293db`)
  until a release carries this design; the tag is kept.
- Migration in the activating change: the template daemon test at
  `template/crates/daemon/src/main.rs:258-290` adds `sources` to its
  `ConfigError::Deserialize` literal, and any exhaustive `match` on
  `ConfigError` outside the crate gains a wildcard arm. Variants stay
  constructible by literal. This is a breaking change for the pre-1.0
  crate: one minor-version bump.
- A type error is still reported against the merged document.
  `Deserialize` lists every contributing `Source`, and `resolve()` gives a
  caller the provenance to point at the layer that set a failing key.
- Array elements are not tracked in provenance, a consequence of replacing
  arrays whole.
- Boxed sources cost one allocation per layer per load; loads happen at
  start-up, so this is negligible.
- `Source`, `KeyPath` and `ResolvedRow` derive `Serialize` with `serde`'s
  `derive` feature, which `crates/sc-config/Cargo.toml` already enables;
  no crate is added ([NFR-CFG-0002](requirements.md)).

### Alternatives Considered

- Merge PR #153 as is and generalise later. Rejected: every public type in it
  is exhaustive, so generalising is a breaking change for each new consumer,
  and its per-layer decode contradicts ADR-CFG-0002.
- Sources that return an independent overlay with no view of the base.
  Rejected: an environment override could then neither fail through a scalar
  nor replace an object whole, so REQ-CFG-0002 could not hold.
- Sources that take and return the whole accumulated value. Rejected: the
  engine could no longer compute provenance from what each layer set.
- Generic sources (`Layers<S>` or a tuple) instead of
  `Box<dyn LayerSource>`. Rejected: a runtime-assembled stack (files found by
  search, optional profiles) needs trait objects; static dispatch saves
  nothing measurable at start-up.
- Per-layer typed validation to attribute type errors to one layer.
  Rejected: it requires `T` to deserialise from a partial layer, and
  ADR-CFG-0002 item 4 forbids it. Provenance gives attribution without it.
- Ship TOML and YAML support in `sc-config`. Rejected: adds parser
  dependencies against NFR-CFG-0002; the parser hook gives consumers the
  same capability.
- A fixed precedence with an override hook. Rejected: still privileges one
  order; push order is simpler and complete.
- Redaction of sensitive values in provenance rows. Deferred with no work
  item: no consumer has named the need, and `ResolvedRow` is
  `#[non_exhaustive]`, so it can be added without breaking callers.

### Implementation

**Enforced by:** when Active, the success criteria of
[REQ-CFG-0005](requirements.md) through [REQ-CFG-0009](requirements.md) and
[NFR-CFG-0004](requirements.md), all run by `cargo test -p sc-config`, and
the `arch-qa` review agent, which checks every change to `crates/sc-config`
for a second load pipeline, a built-in source rank, an error type other than
`ConfigError`, or a deserialisation into `T` other than the single call
after `resolve()`. The boundary manifest's `[public] facade` is updated in
the same change as the new exports, checked by `facade_lists_every_export`.
While Proposed, nothing here is enforced.

### Related Documents

- [ADR-CFG-0001](architecture.md): `ConfigError` contract and the only
  error inventory, amended by item 5
- [ADR-CFG-0002](architecture.md): value merge and single deserialisation,
  amended by item 5
- [ADR-CFG-0003](architecture.md): synchronous, two dependencies; unchanged
- [REQ-CFG-0005](requirements.md) through [REQ-CFG-0009](requirements.md),
  [NFR-CFG-0004](requirements.md)
- PR #153: the `layers` proposal this decision replaces
