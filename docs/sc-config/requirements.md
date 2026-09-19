# `sc-config` Requirements

Crate-level requirements. Repo-level requirements: [`../requirements.md`](../requirements.md).
Source: [`../sc-runtime-design.md`](../sc-runtime-design.md). Every id is binding and is never reused.

## Purpose

Configuration for any Rust program: JSON files with environment-variable
overrides, deserialised into the caller's serde types.

## Requirements

- `REQ-CONFIG-001` `load(app)` reads `config/default.json`, then overlays
  `config/local.json` when it exists, and deserialises the merged document
  into the caller's `T: DeserializeOwned`. Objects merge key by key; any other
  value is replaced.
- `REQ-CONFIG-002` Environment variables override file values. The variable
  name is the upper-cased app name, then the key path, joined by `__`
  (`MY_APP__DAEMON__PORT`). A value is parsed as JSON when it parses, and is a
  string otherwise.
- `REQ-CONFIG-003` The config directory and the environment source can be
  supplied explicitly, so callers and tests never depend on the process
  working directory or the real process environment.
- `REQ-CONFIG-004` A missing `default.json`, unreadable file, invalid JSON, an
  override that names no valid path, and a document that does not deserialise
  into `T` are distinct `ConfigError` variants that carry the file path or
  variable name.

## Non-functional requirements

- `NFR-CONFIG-001` Every public method returns a discriminated union
  (`Result<T, ConfigError>` with a typed error enum). No public method panics:
  no `unwrap`, `expect`, `panic!` or panicking index is reachable from the
  public API.
- `NFR-CONFIG-002` Runtime dependencies are `serde` and `serde_json` only.
  The crate does not depend on `sc-observability`, on tokio, or on any other
  crate in this workspace.
- `NFR-CONFIG-003` The API is synchronous. Reload, change notification and
  async interop are out of scope for v0.1.
