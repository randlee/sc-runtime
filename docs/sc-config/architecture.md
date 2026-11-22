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
the shared SC requirement and ADR templates. Every ADR here is Active
and binding; an ADR changes only through a later ADR that names it.

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

Proposed additions ([ADR-CFG-0004](#adr-cfg-0004-the-caller-defines-the-layers-sc-config-provides-the-engine)):
`Layers` (ordered stack, push order is precedence), the unsealed
`LayerSource` trait, `FileSource`, `EnvSource`, `ValueSource`, `Source` /
`SourceKind` / `KeyPath` provenance types, `Provenance` and `ResolvedRow`,
and `Loader::layers()`. `ConfigError` becomes `#[non_exhaustive]`.

## How loading works

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
   is what fails those names. a-2 created the single-member root lint tree
   (`members = ["crates/sc-config"]`). a-5 extends that workspace to four
   members; this crate's manifest stays the one this sprint writes.
   `[public] facade` lists every `lib.rs` export. sc-lint 0.8.0 parses that
   comma-separated list and does not compare it to the crate. The
   `facade-mismatch` fixture under `examples/spike/boundary-proof/` exports
   `present` and `extra` while its facade names only
   `facade_mismatch::present`; `sc-lint lint sc-boundary` on a copy of that
   tree exits 0. `facade_lists_every_export` is the check that this
   manifest stays aligned with `lib.rs`. The process-test helper
   `sc-config-load-helper` is autodiscovered from `src/bin/` in this
   checkout and excluded from the published package. It is not a facade item.

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
**Source:** extensibility review of PR #153 (`sc-config` `layers`, sc-agent team), 2026-11-22; amends [ADR-CFG-0002](architecture.md) Context ("The design fixes the layers") and Decision item 6 ("The only file format is JSON")  

### Context

ADR-CFG-0002 fixed three layers (`default.json`, `local.json`, environment)
and JSON. sc-agent needs more: command-line flags above the environment,
several files in its own order, aliased variable names and per-leaf
provenance for `config show`. PR #153 met that by adding a second public
loader with a closed `Source` enum (flag, env, file, default), a fixed
flags > env > files precedence, JSON only, a hard-coded 1 MiB cap,
exhaustive public structs, a separate `LayersError` with no codes, and a
per-layer `decode` into `T` that ADR-CFG-0002 item 4 forbids. The two
loaders already differ on environment ordering, file checks and error
semantics.

Every one of those fixed choices is right for sc-agent and wrong for some
other consumer: a daemon that wants files over environment, a tool reading
TOML, a service with a secrets source, keys with upper case. The only
in-repo consumers of `sc-config` today are the template CLI and daemon,
which use `load` and `AppName`, so the public API can be opened now at no
migration cost.

### Decision

1. `sc-config` provides one layered engine (`Layers`) over an ordered list of
   `LayerSource` trait objects. The caller chooses the sources and their
   order; `sc-config` assigns no source kind a rank
   ([REQ-CFG-0005](requirements.md)).
2. `LayerSource` is public, object safe and unsealed. Built-in `FileSource`,
   `EnvSource` and `ValueSource` implement it; consumers add their own
   ([REQ-CFG-0006](requirements.md)).
3. Every behaviour a consumer could reasonably want differently is a builder
   parameter whose default equals today's behaviour: file optionality, size
   cap (off by default), file format, environment separator, key case,
   aliases and an accept list ([REQ-CFG-0006](requirements.md)).
4. Amends ADR-CFG-0002 item 6: the built-in format is still JSON and the
   crate still depends only on `serde` and `serde_json`
   ([NFR-CFG-0002](requirements.md)), but a `FileSource` accepts a
   caller-supplied parser to `serde_json::Value`, so consumers are not
   limited to JSON.
5. ADR-CFG-0002 items 1 to 5 stay binding: layers fold as
   `serde_json::Value` with the existing `merge`, and the result is
   deserialised into `T` once. Provenance is computed from the layer values,
   never by deserialising a layer ([REQ-CFG-0007](requirements.md)).
6. Provenance uses an open `Source { kind, name }` with a `Custom` kind,
   runtime-built `KeyPath`s, typed serialisable rows and caller-marked
   redaction ([REQ-CFG-0007](requirements.md)).
7. `Loader` and `load` become a preset stack on the engine and keep their
   documented behaviour. The crate has one file reader, one override
   applier and one `merge` ([REQ-CFG-0008](requirements.md)).
8. `ConfigError` stays the only error type, becomes `#[non_exhaustive]`, and
   gains `TooLarge`, `ParseFormat` and `Source` with codes and recovery
   steps; `MissingDefault` becomes `MissingFile`
   ([REQ-CFG-0009](requirements.md)). Error inventory additions to
   ADR-CFG-0001:

   | Failure mode | Stable code | Cause payload | Recovery steps |
   |---|---|---|---|
   | A required file is absent | `CFG.MISSING_FILE` (replaces `CFG.MISSING_DEFAULT`) | `MissingFile.path` | Create that file, or mark the source optional. |
   | A capped file exceeds its limit | `CFG.TOO_LARGE` | `TooLarge.path`, `TooLarge.limit` | Shrink the file or raise the limit the program set. |
   | A caller-supplied format parser failed | `CFG.PARSE_FORMAT` | `ParseFormat.path`, `.message`, `.line`, `.column` | Fix the file at the reported place. |
   | A consumer-defined source failed | `CFG.SOURCE` | `Source.source`, `Source.message` | Follow the message from that source. |

9. Public types follow [NFR-CFG-0004](requirements.md): non-exhaustive,
   builder-constructed, no `&'static str` inputs.
10. `sc-config` never parses command lines. Flags enter as a `ValueSource`
    built by the consumer's own argument parser.
11. `sc-config` stays synchronous ([NFR-CFG-0003](requirements.md));
    `LayerSource::load` is a synchronous call. An async source is the job of
    a separate crate.

### Consequences

- sc-agent builds its stack in its own code, for example
  `Loader::layers()` plus a `ValueSource` of flags, or a fully custom
  `[files…, EnvSource with aliases, flags]`. Its order lives in sc-agent.
- PR #153 must be reworked onto this design before it merges: its `Inputs`,
  `Key`, `LayersError`, closed `Source` and per-layer `decode` are replaced.
- Renaming `MissingDefault` to `MissingFile` and adding `#[non_exhaustive]`
  break exhaustive matches once. The crate is pre-1.0 and the only in-repo
  callers print the error, so the cost is one minor-version bump.
- A type error is still reported against the merged document. `Deserialize`
  now lists every contributing `Source`, and a consumer can use provenance
  to point at the layer that set a failing key.
- Boxed sources cost one allocation per layer per load; loads happen at
  start-up, so this is negligible.

### Alternatives Considered

- Merge PR #153 as is and generalise later. Rejected: every public type in it
  is exhaustive, so generalising is a breaking change for each new consumer,
  and its per-layer decode contradicts ADR-CFG-0002.
- Generic sources (`Layers<S: LayerSource>` or a tuple of sources) instead of
  `Box<dyn LayerSource>`. Rejected: a heterogeneous, runtime-assembled stack
  (files found by search, optional profiles) needs trait objects; static
  dispatch saves nothing measurable at start-up.
- Per-layer typed validation to attribute type errors to one layer (PR #153's
  `decode` per layer). Rejected: it requires `T` to deserialise from a
  partial layer, which forces `#[serde(default)]` everywhere or fails on
  valid partial layers, and ADR-CFG-0002 item 4 forbids it. Provenance plus
  the `sources` list gives attribution without it.
- Ship TOML and YAML support in `sc-config`. Rejected: adds parser
  dependencies against NFR-CFG-0002; the parser hook gives consumers the
  same capability.
- A fixed precedence with an override hook. Rejected: still privileges one
  order; push order is simpler and complete.

### Implementation

**Enforced by:** the success criteria of [REQ-CFG-0005](requirements.md)
through [REQ-CFG-0009](requirements.md) and [NFR-CFG-0004](requirements.md),
and the `arch-qa` review agent, which checks every change to
`crates/sc-config` for a second load pipeline, a built-in source rank, a
public type without `#[non_exhaustive]`, an error type other than
`ConfigError`, or a per-layer deserialisation into `T`.

### Related Documents

- [ADR-CFG-0001](architecture.md): `ConfigError` contract and inventory,
  extended by item 8
- [ADR-CFG-0002](architecture.md): value merge and single deserialisation,
  amended by items 4 and 5
- [ADR-CFG-0003](architecture.md): synchronous, two dependencies; unchanged
- [REQ-CFG-0005](requirements.md) through [REQ-CFG-0009](requirements.md),
  [NFR-CFG-0004](requirements.md)
- PR #153: the `layers` proposal this decision replaces
