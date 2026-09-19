# `sc-config` Architecture

**ID Range:** ADR-CFG-0001 through ADR-CFG-0003  
**Status:** Draft  
**Created:** 2026-09-19  
**Last Updated:** 2026-09-19  
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
| `load::<T>(app) -> Result<T, ConfigError>` | convenience: `./config` and the process environment |
| `Loader` | explicit config directory and environment source; `Loader::load::<T>()` |
| `ConfigError` | typed error enum, one variant per failure in [REQ-CFG-0004](requirements.md) |

## How loading works

1. Read `default.json` into a `serde_json::Value` (required).
2. Read `local.json` if present and merge it over the first value.
3. For each environment entry with the app's prefix, split the rest on `__`
   into a key path, parse the value as JSON or fall back to a string, and set
   it at that path.
4. Deserialise the merged value into `T` once.

---

## ADR-CFG-0001: Errors are values; the crate cannot panic

**Status:** Active  
**Decision Date:** 2026-09-19  
**Source:** design decision 32; applies repo [ADR-RUN-0006](../architecture.md)  

### Context

Config loading runs first in `main`, before observability is
initialised, and its failures are user mistakes (a typo in a JSON file) far
more often than bugs.

### Decision

Every public method returns `Result<T, ConfigError>`.
`ConfigError` is a hand-written enum implementing `std::error::Error` and
`Display`; each variant names the file or variable at fault. Nothing reachable
from the public API can panic.

### Consequences

The generated `main.rs` matches on the result and returns an
exit code. Hand-writing the error impl is a few dozen lines and is what keeps
the dependency list at two crates.

### Alternatives Considered

`thiserror` (a third dependency for one enum); `anyhow` (opaque
to callers); panicking on a missing `default.json`.

### Implementation

**Enforced by:** `req-qa` on [NFR-CFG-0001](requirements.md); `rust-best-practices-agent`.

### Related Documents

- [NFR-CFG-0001](requirements.md)
- [REQ-CFG-0004](requirements.md)

---

## ADR-CFG-0002: Merge as JSON values, deserialise once

**Status:** Active  
**Decision Date:** 2026-09-19  
**Source:** design "The crates"; decided here  

### Context

Layering can be done on typed structs (every field optional, then
merged) or on untyped JSON before deserialising. The typed route forces every
project to write a parallel "partial" struct.

### Decision

Files and overrides are merged as `serde_json::Value`, and the
result is deserialised into the caller's type in one step.

### Consequences

The caller's serde attributes, defaults and validation govern
the result, the crate needs no schema of its own, and a project's config type
is one ordinary struct. Type errors surface once, at the end, against the
merged document, so the error names a key path rather than a file; the error
variant therefore also lists which files and overrides contributed.

### Alternatives Considered

Partial-struct merging; a bespoke config DSL; TOML or YAML (the
design specifies JSON).

### Implementation

**Enforced by:** `arch-qa`.

### Related Documents

- [REQ-CFG-0001](requirements.md)
- [REQ-CFG-0002](requirements.md)

---

## ADR-CFG-0003: Synchronous and dependency-minimal

**Status:** Active  
**Decision Date:** 2026-09-19  
**Source:** design decisions 33, 34  

### Context

Reload, change notification and async interop are wanted
eventually. Adding them now would bring tokio and a file watcher into every
program that only wants to read a file once.

### Decision

`serde` and `serde_json` only, synchronous API. Reload,
notification and async belong to a possible later `sc-config-tokio` crate that
depends on this one, never the reverse.

### Consequences

A CLI pays nothing for config. Adding reload later is an
additive crate, not a breaking change here.

### Alternatives Considered

Feature-gated async inside this crate.

### Implementation

**Enforced by:** `forbidden_edges` in `boundaries/sc-config/`.

### Related Documents

- [NFR-CFG-0002](requirements.md)
- [NFR-CFG-0003](requirements.md)
