# `sc-command` Architecture

Crate-level architecture and ADRs for `sc-command`. Repo-level architecture is in
[`../architecture.md`](../architecture.md); this crate's requirements are in
[`requirements.md`](requirements.md); its boundary manifest is
`boundaries/sc-command/`. ADRs follow
`.claude/skills/plan-hardening/req-adr-format.md`. Every ADR here is `accepted`
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

## ADR

### ADR-COMMAND-001 The envelope is defined on `sc-observability-types`

| Field | Value |
|---|---|
| Status | accepted |
| Date | 2026-09-19 |
| Source | design "The crates"; applies repo ADR-005 |
| Relates to | `REQ-COMMAND-002`, `NFR-COMMAND-002` |

**Context.** The sc-ai-cli envelope's error code and remediation types already
exist in `sc-observability-types`. Redefining them here would create two
definitions of the same codes.

**Decision.** `OpError` uses those types. This is the one observability
dependency anywhere in these crates, and it is the types crate only.

**Consequences.** Error codes match between responses and logs. `sc-command`
tracks `sc-observability-types` versions.

**Rejected.** Local copies of the types; depending on `sc-observability`.

**Enforced by.** The boundary manifest.

### ADR-COMMAND-002 Conversions, not wrappers

| Field | Value |
|---|---|
| Status | accepted |
| Date | 2026-09-19 |
| Source | design "API surface"; applies repo ADR-001 |
| Relates to | `REQ-COMMAND-003..005`, `NFR-REPO-003` |

**Context.** A framework could supply its own handler type, extractor or macro
that produces envelopes. That would put it between the project and Axum and
rmcp.

**Decision.** REST and MCP edges are reached by `into()` and `into_mcp()` on
ordinary `Result<T, OpError>` values inside ordinary axum handlers and rmcp
tools. The crate defines no handler, extractor or macro and wraps no axum or
rmcp type.

**Consequences.** A handler is a normal axum handler whose last line is
`.into()`. Projects that want a different response shape for one route simply
do not call it.

**Rejected.** A handler macro; an envelope extractor or middleware layer.

**Enforced by.** `arch-qa`; `NFR-REPO-003`.

### ADR-COMMAND-003 The `server` feature gates axum and rmcp

| Field | Value |
|---|---|
| Status | accepted |
| Date | 2026-09-19 |
| Source | design "Repository layout"; applies repo ADR-004 |
| Relates to | `NFR-COMMAND-001` |

**Context.** The CLI needs `Envelope` and `OpError` and must not link Axum or
rmcp.

**Decision.** Both conversions sit behind a `server` feature that is off by
default. Only `sc-runtime` and the generated daemon enable it.

**Consequences.** CI builds and tests the crate both ways.

**Rejected.** A separate `sc-command-server` crate.

**Enforced by.** `cargo tree` evidence; the boundary manifest.
