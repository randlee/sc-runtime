# `sc-config` Requirements

Crate-level requirements for `sc-config`. Repo-level requirements are in
[`../requirements.md`](../requirements.md); this crate's architecture and ADRs
are in [`architecture.md`](architecture.md). Extracted from
[`../sc-runtime-design.md`](../sc-runtime-design.md) ("decision N" refers to its
decision log). Entries follow `.claude/skills/plan-hardening/req-adr-format.md`:
the obligation, then **Why** and **Verified by**. Every id is binding and is
never reused.

## Purpose

`sc-config` gives any Rust program its configuration: JSON files with
environment-variable overrides, deserialised into the caller's own serde types.
It is useful on its own, outside any sc-runtime daemon. In a generated project
both the daemon and the CLI use it to load the same `AppConfig`, which is how
they agree on the endpoint and the instance root.

## Requirements

- `REQ-CONFIG-001` **Layered JSON files.**
  `load::<T>(app)` reads `config/default.json`, overlays `config/local.json`
  when that file exists, and deserialises the merged document into the caller's
  `T: DeserializeOwned`. Objects merge key by key, recursively; any other value
  (scalar, array, null) replaces the value beneath it.
  **Why:** `default.json` is checked in and documents every setting;
  `local.json` is gitignored and holds one machine's differences. Recursive
  object merge lets `local.json` change one nested key without restating its
  siblings. The design fixes the two file names ("Generated project") and
  leaves merge semantics open; this entry decides them.
  **Verified by:** unit tests for nested merge, array replacement and an absent
  `local.json`.

- `REQ-CONFIG-002` **Environment overrides.**
  Environment variables override file values. The variable name is the app name
  upper-cased with `-` as `_`, then the key path, joined by `__`:
  `MY_APP__DAEMON__PORT` overrides `daemon.port` for app `my-app`. A value that
  parses as JSON is used as that JSON value (`8080`, `true`, `["a"]`); any
  other value is a string.
  **Why:** deployment and CI change settings without editing files (design:
  "JSON files with environment-variable overrides"). `__` as the separator lets
  keys themselves contain `_`. JSON parsing lets numbers and booleans be
  overridden with the right type. The naming scheme is decided here; the design
  does not state one.
  **Verified by:** unit tests for a nested override, each JSON type, a plain
  string, and a variable belonging to a different app being ignored.

- `REQ-CONFIG-003` **Explicit directory and environment source.**
  A `Loader` accepts the config directory and the environment source (an
  iterator of key and value pairs) explicitly. `load(app)` is the convenience
  form using `./config` and the process environment.
  **Why:** tests must not depend on the working directory or mutate the real
  process environment, which is global and would break parallel tests
  (`NFR-REPO-005`).
  **Verified by:** the crate's own tests use `Loader` with a tempdir and an
  in-memory environment and pass when run in parallel.

- `REQ-CONFIG-004` **Errors that say what and where.**
  A missing `default.json`, an unreadable file, invalid JSON, an override whose
  path cannot be applied (for example descending into a scalar), and a merged
  document that does not deserialise into `T` are distinct `ConfigError`
  variants. Each carries the file path or variable name at fault and, where
  serde provides one, the location inside the document.
  **Why:** configuration errors are the first thing a new user hits; "invalid
  config" with no file or key is not actionable, and the generated `main.rs`
  reports this error straight to the terminal.
  **Verified by:** one test per variant asserting the variant and its payload.

## Non-functional requirements

- `NFR-CONFIG-001` **Every method returns a discriminated union; no panics.**
  Every public method returns `Result<T, ConfigError>` with a typed error enum.
  No `unwrap`, `expect`, `panic!` or panicking index is reachable from the
  public API.
  **Why:** stated directly in the design (decision 32). Config loads at the top
  of `main`, before logging exists, so a panic there is the least diagnosable
  failure a program can have.
  **Verified by:** source review for panicking constructs; tests feeding
  malformed files, malformed overrides and unreadable paths.

- `NFR-CONFIG-002` **Minimal dependencies.**
  Runtime dependencies are `serde` and `serde_json` only. The crate does not
  depend on `sc-observability`, on tokio, or on any other crate in this
  workspace.
  **Why:** the design lists exactly these dependencies and states "No
  dependency on `sc-observability`" (decision 34); a config crate that drags in
  a runtime or a logging stack cannot be the first thing a program loads.
  **Verified by:** `Cargo.toml`; `forbidden_edges` in the boundary manifest.

- `NFR-CONFIG-003` **Synchronous API in v0.1.**
  The API is synchronous. Automatic reloading, change notification and async
  interop are out of scope.
  **Why:** decision 33 records them as a note for later, possibly as a separate
  `sc-config-tokio` crate, and explicitly not in the initial plan.
  **Verified by:** no async function and no file watcher in the crate.
