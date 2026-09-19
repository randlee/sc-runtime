# `sc-config` Architecture

Crate-level architecture and ADRs. Repo-level: [`../architecture.md`](../architecture.md).
Requirements: [`requirements.md`](requirements.md). Boundary manifest: `boundaries/sc-config/`.
All ADRs are `accepted` unless stated; an ADR changes only through a later ADR that names it.

## Role

A leaf crate. It reads files and an environment source and returns the
caller's type. It knows nothing about daemons, transport or observability.

## Public surface

`load`, `Loader` (explicit directory and environment source), `ConfigError`.

## ADR

### ADR-CONFIG-001 Errors are values; the crate cannot panic

Every public method returns `Result<T, ConfigError>`. `ConfigError` is a
hand-written enum implementing `std::error::Error`; each variant names the
file or variable at fault. Applies repo ADR-011.

### ADR-CONFIG-002 Layer by JSON value merge, then deserialise once

Files and overrides are merged as `serde_json::Value` and deserialised into
the caller's type in one step, so the caller's serde attributes and defaults
govern the result and the crate needs no schema of its own.

### ADR-CONFIG-003 Synchronous and dependency-minimal

`serde` and `serde_json` only. Async, reload and notification belong to a
possible later `sc-config-tokio` crate, never to this one.
