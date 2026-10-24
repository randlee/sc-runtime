# `sc-config` Architecture

**ID Range:** ADR-CFG-0001 through ADR-CFG-0003  
**Status:** Draft  
**Created:** 2026-09-19  
**Last Updated:** 2026-10-23  
**Owner:** sc-runtime maintainers  

---

Crate-level architecture and ADRs for `sc-config`. Repo-level architecture is in
[`../architecture.md`](../architecture.md); this crate's requirements are in
[`requirements.md`](requirements.md); its boundary manifest is
`boundaries/sc-config/`. ADRs follow
the shared SC requirement and ADR templates. Every ADR here is Active
and binding; an ADR changes only through a later ADR that names it.

## Role

A leaf crate. It reads files and an environment source and returns the caller's
type, or a typed error. It knows nothing about daemons, transport, envelopes or
observability, and nothing in this workspace is a dependency of it.

## Public surface

| Item | Purpose |
|---|---|
| `AppName` | validated configuration identity; `new` / `TryFrom<&str>` rejects with `APP.INVALID_NAME` |
| `load::<T>(app: AppName) -> Result<T, ConfigError>` | convenience: `./config` and the process environment |
| `Loader::new(app, config_dir, environment) -> Loader` | stores an `AppName`, a `PathBuf`, and `Vec<(OsString, OsString)>` |
| `Loader::load::<T>() -> Result<T, ConfigError>` | same merge as `load`, using only those inputs |
| `ConfigError` | typed error enum; inventory below and [REQ-CFG-0004](requirements.md) |

## How loading works

1. Read `default.json` into a `serde_json::Value` (required).
2. Read `local.json` if present and merge it over the first value.
3. Keep environment entries whose encoded name starts with the app prefix.
   Apply them in ascending encoded-byte order of the variable name. Split the
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
   list. `allowed_dependencies` is `serde` and `serde_json`.
   `forbidden_edges` lists workspace-member targets only. This lint tree's
   only workspace member is `sc-config`, so that list is empty.
   `sc-observability-types` and the other crates.io names in the
   [REQ-RUN-0005](../requirements.md) policy table are not members of this
   tree, so they are not manifest edges. `cargo tree -p sc-config -e normal --prefix none`
   is what fails those names. a-5 extends the root workspace; this crate's
   manifest stays the one this sprint writes.

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
