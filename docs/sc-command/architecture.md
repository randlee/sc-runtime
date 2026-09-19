# `sc-command` Architecture

Crate-level architecture and ADRs. Repo-level: [`../architecture.md`](../architecture.md).
Requirements: [`requirements.md`](requirements.md). Boundary manifest: `boundaries/sc-command/`.
All ADRs are `accepted` unless stated; an ADR changes only through a later ADR that names it.

## Role

A leaf crate of plain data types plus two feature-gated conversions. It owns
the wire shape of every response; it owns no routing, registry or transport.

## Public surface

`Envelope<T>`, `OpError`, `ErrorKind`; behind `server`: `IntoResponse for
Envelope<T>`, `IntoMcp::into_mcp`.

## ADR

### ADR-COMMAND-001 The envelope is defined on `sc-observability-types`

Error codes and remediation types come from `sc-observability-types`, the one
observability dependency in this repo. Applies repo ADR-005.

### ADR-COMMAND-002 Conversions, not wrappers

REST and MCP edges are reached by `into()` and `into_mcp()` on ordinary
`Result<T, OpError>` values. The crate wraps no axum or rmcp type and defines
no handler, extractor or macro. Applies repo ADR-001.

### ADR-COMMAND-003 `server` feature gates axum and rmcp

Both conversions sit behind `server`, off by default, so a CLI links neither.
Applies repo ADR-004.
