# `sc-config` Requirements

**ID Range:** REQ-CFG-0001 through REQ-CFG-0004; NFR-CFG-0001 through NFR-CFG-0003  
**Status:** Draft  
**Created:** 2026-09-19  
**Last Updated:** 2026-09-19  
**Owner:** sc-runtime maintainers  

---

Crate-level requirements for `sc-config`. Repo-level requirements are in
[`../requirements.md`](../requirements.md); this crate's architecture and ADRs
are in [`architecture.md`](architecture.md). Extracted from
[`../sc-runtime-design.md`](../sc-runtime-design.md) ("decision N" refers to its
decision log). Entries follow the shared SC requirement and ADR templates:
the obligation, then **Why** and **Verified by**. Every id is binding and is
never reused.

## Purpose

`sc-config` gives any Rust program its configuration: JSON files with
environment-variable overrides, deserialised into the caller's own serde types.
It is useful on its own, outside any sc-runtime daemon. In a generated project
both the daemon and the CLI use it to load the same `AppConfig`, which is how
they agree on the endpoint and the instance root.

## ID Ranges

| Range | Area | Scope |
|---|---|---|
| REQ-CFG-0001 through REQ-CFG-0004 | Requirements | - |
| NFR-CFG-0001 through NFR-CFG-0003 | Non-functional requirements | - |

---

## REQ-CFG-0001: Layered JSON files

**Status:** Active  

### Requirement Statement

`load::<T>(app)` reads `config/default.json`, overlays `config/local.json`
when that file exists, and deserialises the merged document into the caller's
`T: DeserializeOwned`. Objects merge key by key, recursively; any other value
(scalar, array, null) replaces the value beneath it.

### Rationale

`default.json` is checked in and documents every setting;
`local.json` is gitignored and holds one machine's differences. Recursive
object merge lets `local.json` change one nested key without restating its
siblings. The design fixes the two file names ("Generated project") and
leaves merge semantics open; this entry decides them.

### Success Criteria

Unit tests for nested merge, array replacement and an absent
`local.json`.

---

## REQ-CFG-0002: Environment overrides

**Status:** Active  

### Requirement Statement

Environment variables override file values. The variable name is the app name
upper-cased with `-` as `_`, then the key path, joined by `__`:
`MY_APP__DAEMON__PORT` overrides `daemon.port` for app `my-app`. A value that
parses as JSON is used as that JSON value (`8080`, `true`, `["a"]`); any
other value is a string.

### Rationale

Deployment and CI change settings without editing files (design:
"JSON files with environment-variable overrides"). `__` as the separator lets
keys themselves contain `_`. JSON parsing lets numbers and booleans be
overridden with the right type. The naming scheme is decided here; the design
does not state one.

### Success Criteria

Unit tests for a nested override, each JSON type, a plain
string, and a variable belonging to a different app being ignored.

---

## REQ-CFG-0003: Explicit directory and environment source

**Status:** Active  

### Requirement Statement

A `Loader` accepts the config directory and the environment source (an
iterator of key and value pairs) explicitly. `load(app)` is the convenience
form using `./config` and the process environment.

### Rationale

Tests must not depend on the working directory or mutate the real
process environment, which is global and would break parallel tests
([NFR-RUN-0005](../requirements.md)).

### Success Criteria

The crate's own tests use `Loader` with a tempdir and an
in-memory environment and pass when run in parallel.

---

## REQ-CFG-0004: Errors that say what and where

**Status:** Active  

### Requirement Statement

A missing `default.json`, an unreadable file, invalid JSON, an override whose
path cannot be applied (for example descending into a scalar), and a merged
document that does not deserialise into `T` are distinct `ConfigError`
variants. Each carries the file path or variable name at fault and, where
serde provides one, the location inside the document.

### Rationale

Configuration errors are the first thing a new user hits; "invalid
config" with no file or key is not actionable, and the generated `main.rs`
reports this error straight to the terminal.

### Success Criteria

One test per variant asserting the variant and its payload.

---

## NFR-CFG-0001: Every method returns a discriminated union; no panics

**Status:** Active  

### Requirement Statement

Every public method returns `Result<T, ConfigError>` with a typed error enum.
No `unwrap`, `expect`, `panic!` or panicking index is reachable from the
public API.

### Rationale

Stated directly in the design (decision 32). Config loads at the top
of `main`, before logging exists, so a panic there is the least diagnosable
failure a program can have.

### Success Criteria

Source review for panicking constructs; tests feeding
malformed files, malformed overrides and unreadable paths.

---

## NFR-CFG-0002: Minimal dependencies

**Status:** Active  

### Requirement Statement

Runtime dependencies are `serde` and `serde_json` only. The crate does not
depend on `sc-observability`, on tokio, or on any other crate in this
workspace.

### Rationale

The design lists exactly these dependencies and states "No
dependency on `sc-observability`" (decision 34); a config crate that drags in
a runtime or a logging stack cannot be the first thing a program loads.

### Success Criteria

`Cargo.toml`; `forbidden_edges` in the boundary manifest.

---

## NFR-CFG-0003: Synchronous API in v0.1

**Status:** Active  

### Requirement Statement

The API is synchronous. Automatic reloading, change notification and async
interop are out of scope.

### Rationale

Decision 33 records them as a note for later, possibly as a separate
`sc-config-tokio` crate, and explicitly not in the initial plan.

### Success Criteria

No async function and no file watcher in the crate.
