# `sc-command` Requirements

**ID Range:** REQ-CMD-0001 through REQ-CMD-0006; NFR-CMD-0001 through NFR-CMD-0003  
**Status:** Draft  
**Created:** 2026-09-19  
**Last Updated:** 2026-09-19  
**Owner:** sc-runtime maintainers  

---

Crate-level requirements for `sc-command`. Repo-level requirements are in
[`../requirements.md`](../requirements.md); this crate's architecture and ADRs
are in [`architecture.md`](architecture.md). Extracted from
[`../sc-runtime-design.md`](../sc-runtime-design.md) ("decision N" refers to its
decision log). Entries follow the shared SC requirement and ADR templates:
the obligation, then **Why** and **Verified by**. Every id is binding and is
never reused.

## Purpose

`sc-command` is the sc-ai-cli response envelope for any JSON-first CLI or API,
together with the two conversions that put it on the wire: into an axum
response and into an rmcp tool result. The daemon uses it to answer; the CLI
uses it to parse the answer and to print it for `--json`.

## ID Ranges

| Range | Area | Scope |
|---|---|---|
| REQ-CMD-0001 through REQ-CMD-0006 | Requirements | - |
| NFR-CMD-0001 through NFR-CMD-0003 | Non-functional requirements | - |

---

## REQ-CMD-0001: The envelope

**Status:** Active  

### Requirement Statement

`Envelope<T>` serialises as `{version, ok, data, error}`. `ok: true` carries
`data` and a null `error`; `ok: false` carries `error` and a null `data`. It
round-trips through serde for any `T` that does.

### Rationale

Agents consume these responses; one fixed shape with an explicit
`ok` means a caller never infers success from an HTTP status or from which
fields are present (decision 8, sc-ai-cli convention).

### Success Criteria

Serialisation tests for both arms; a round-trip test.

---

## REQ-CMD-0002: `OpError`

**Status:** Active  

### Requirement Statement

`OpError` carries `kind`, `code`, `message`, `details` and
`suggested_action`, with its code and remediation types taken from
`sc-observability-types`.

### Rationale

`code` is what automation branches on (`DAEMON.NOT_RUNNING`),
`suggested_action` is what lets an agent recover without a human, and taking
the types from `sc-observability-types` keeps error codes identical in
responses and in logs.

### Success Criteria

Construction and serialisation tests; the dependency on
`sc-observability-types`.

---

## REQ-CMD-0003: From a service result

**Status:** Active  

### Requirement Statement

`Result<T, OpError>` converts into `Envelope<T>` with `into()`.

### Rationale

Service functions return `Result<T, OpError>`; a handler's whole body
is `service::create_widget(&s, input).await.into()`.

### Success Criteria

A test for each arm.

---

## REQ-CMD-0004: Into an axum response (`server` feature)

**Status:** Active  

### Requirement Statement

With the `server` feature, `Envelope<T>` implements axum `IntoResponse`: the
body is the JSON envelope and the HTTP status is derived from the error
`kind` (success 200; caller errors 4xx; internal errors 5xx).

### Rationale

REST clients and proxies that only look at status still behave
sensibly, while the envelope stays the source of truth. The kind-to-status
table is decided here; the design specifies the conversion, not the mapping.

### Success Criteria

A test per kind asserting status and body.

---

## REQ-CMD-0005: Into an rmcp tool result (`server` feature)

**Status:** Active  

### Requirement Statement

With the `server` feature, `into_mcp()` converts `Result<T, OpError>` into
`Result<CallToolResult, McpError>` carrying the same envelope as the tool
result content, with the result flagged as an error when `ok` is false.

### Rationale

An MCP tool body is
`service::create_widget(&self.stores, input).await.into_mcp()`, and an MCP
client sees the same envelope a REST client sees ([REQ-RUN-0203](../requirements.md)).

### Success Criteria

A test for each arm against rmcp's types.

---

## REQ-CMD-0006: Printable unchanged

**Status:** Active  

### Requirement Statement

Serialising a deserialised envelope reproduces the same JSON value, including
unknown-to-the-CLI content inside `data` and `details`.

### Rationale

The CLI's `--json` prints the envelope it received unchanged; a lossy
round-trip would make CLI output differ from the API.

### Success Criteria

A round-trip test over a captured daemon response.

---

## NFR-CMD-0001: axum and rmcp only behind `server`

**Status:** Active  

### Requirement Statement

Both are dependencies only behind the `server` feature, which is off by
default. The default build depends on serde and `sc-observability-types`.

### Rationale

The CLI uses this crate and must link neither ([NFR-RUN-0001](../requirements.md)).

### Success Criteria

`cargo tree -p sc-command` with default features; builds and
tests with and without the feature.

---

## NFR-CMD-0002: Standalone; types only from observability

**Status:** Active  

### Requirement Statement

The crate depends on no other crate in this workspace, and on
`sc-observability-types` but not on `sc-observability` itself.

### Rationale

[REQ-RUN-0002](../requirements.md), [ADR-RUN-0003](../architecture.md) and [ADR-RUN-0004](../architecture.md).

### Success Criteria

The boundary manifest.

---

## NFR-CMD-0003: Errors are values

**Status:** Active  

### Requirement Statement

No public function panics.

### Rationale

[ADR-RUN-0006](../architecture.md). A conversion that panics would take down a request handler.

### Success Criteria

Source review.
