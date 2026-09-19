# `sc-command` Architecture

**ID Range:** ADR-CMD-0001 through ADR-CMD-0003  
**Status:** Draft  
**Created:** 2026-09-19  
**Last Updated:** 2026-09-19  
**Owner:** sc-runtime maintainers  

---

Crate-level architecture and ADRs for `sc-command`. Repo-level architecture is in
[`../architecture.md`](../architecture.md); this crate's requirements are in
[`requirements.md`](requirements.md); its boundary manifest is
`boundaries/sc-command/`. ADRs follow
the shared SC requirement and ADR templates. Every ADR here is Active
and binding; an ADR changes only through a later ADR that names it.

## Role

A leaf crate of plain data types plus two feature-gated conversions. It owns
the wire shape of every response. It owns no routing, registry, transport or
handler.

## Public surface

| Item | Feature | Purpose |
|---|---|---|
| `Envelope<T>` | default | `{version, ok, data, error}` |
| `OpError`, `ErrorKind` | default | typed error with code and suggested action |
| `From<Result<T, OpError>> for Envelope<T>` | default | `.into()` in handlers |
| `impl IntoResponse for Envelope<T>` | `server` | REST edge |
| `IntoMcp::into_mcp` | `server` | MCP edge |

---

## ADR-CMD-0001: The envelope is defined on `sc-observability-types`

**Status:** Active  
**Decision Date:** 2026-09-19  
**Source:** design "The crates"; applies repo [ADR-RUN-0004](../architecture.md)  

### Context

The sc-ai-cli envelope's error code and remediation types already
exist in `sc-observability-types`. Redefining them here would create two
definitions of the same codes.

### Decision

`OpError` uses those types. This is the one observability
dependency anywhere in these crates, and it is the types crate only.

### Consequences

Error codes match between responses and logs. `sc-command`
tracks `sc-observability-types` versions.

### Alternatives Considered

Local copies of the types; depending on `sc-observability`.

### Implementation

**Enforced by:** The boundary manifest.

### Related Documents

- [REQ-CMD-0002](requirements.md)
- [NFR-CMD-0002](requirements.md)

---

## ADR-CMD-0002: Conversions, not wrappers

**Status:** Active  
**Decision Date:** 2026-09-19  
**Source:** design "API surface"; applies repo [ADR-RUN-0001](../architecture.md)  

### Context

A framework could supply its own handler type, extractor or macro
that produces envelopes. That would put it between the project and Axum and
rmcp.

### Decision

REST and MCP edges are reached by `into()` and `into_mcp()` on
ordinary `Result<T, OpError>` values inside ordinary axum handlers and rmcp
tools. The crate defines no handler, extractor or macro and wraps no axum or
rmcp type.

### Consequences

A handler is a normal axum handler whose last line is
`.into()`. Projects that want a different response shape for one route simply
do not call it.

### Alternatives Considered

A handler macro; an envelope extractor or middleware layer.

### Implementation

**Enforced by:** `arch-qa`; [NFR-RUN-0003](../requirements.md).

### Related Documents

- `[REQ-CMD-0003](requirements.md) through [REQ-CMD-0005](requirements.md)`
- [NFR-RUN-0003](../requirements.md)

---

## ADR-CMD-0003: The `server` feature gates axum and rmcp

**Status:** Active  
**Decision Date:** 2026-09-19  
**Source:** design "Repository layout"; applies repo [ADR-RUN-0003](../architecture.md)  

### Context

The CLI needs `Envelope` and `OpError` and must not link Axum or
rmcp.

### Decision

Both conversions sit behind a `server` feature that is off by
default. Only `sc-runtime` and the generated daemon enable it.

### Consequences

CI builds and tests the crate both ways.

### Alternatives Considered

A separate `sc-command-server` crate.

### Implementation

**Enforced by:** `cargo tree` evidence; the boundary manifest.

### Related Documents

- [NFR-CMD-0001](requirements.md)
