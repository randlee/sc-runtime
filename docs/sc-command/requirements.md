# `sc-command` Requirements

Crate-level requirements for `sc-command`. Repo-level requirements are in
[`../requirements.md`](../requirements.md); this crate's architecture and ADRs
are in [`architecture.md`](architecture.md). Extracted from
[`../sc-runtime-design.md`](../sc-runtime-design.md) ("decision N" refers to its
decision log). Entries follow `.claude/skills/plan-hardening/req-adr-format.md`:
the obligation, then **Why** and **Verified by**. Every id is binding and is
never reused.

## Purpose

`sc-command` is the sc-ai-cli response envelope for any JSON-first CLI or API,
together with the two conversions that put it on the wire: into an axum
response and into an rmcp tool result. The daemon uses it to answer; the CLI
uses it to parse the answer and to print it for `--json`.

## Requirements

- `REQ-COMMAND-001` **The envelope.**
  `Envelope<T>` serialises as `{version, ok, data, error}`. `ok: true` carries
  `data` and a null `error`; `ok: false` carries `error` and a null `data`. It
  round-trips through serde for any `T` that does.
  **Why:** agents consume these responses; one fixed shape with an explicit
  `ok` means a caller never infers success from an HTTP status or from which
  fields are present (decision 8, sc-ai-cli convention).
  **Verified by:** serialisation tests for both arms; a round-trip test.

- `REQ-COMMAND-002` **`OpError`.**
  `OpError` carries `kind`, `code`, `message`, `details` and
  `suggested_action`, with its code and remediation types taken from
  `sc-observability-types`.
  **Why:** `code` is what automation branches on (`DAEMON.NOT_RUNNING`),
  `suggested_action` is what lets an agent recover without a human, and taking
  the types from `sc-observability-types` keeps error codes identical in
  responses and in logs.
  **Verified by:** construction and serialisation tests; the dependency on
  `sc-observability-types`.

- `REQ-COMMAND-003` **From a service result.**
  `Result<T, OpError>` converts into `Envelope<T>` with `into()`.
  **Why:** service functions return `Result<T, OpError>`; a handler's whole body
  is `service::create_widget(&s, input).await.into()`.
  **Verified by:** a test for each arm.

- `REQ-COMMAND-004` **Into an axum response (`server` feature).**
  With the `server` feature, `Envelope<T>` implements axum `IntoResponse`: the
  body is the JSON envelope and the HTTP status is derived from the error
  `kind` (success 200; caller errors 4xx; internal errors 5xx).
  **Why:** REST clients and proxies that only look at status still behave
  sensibly, while the envelope stays the source of truth. The kind-to-status
  table is decided here; the design specifies the conversion, not the mapping.
  **Verified by:** a test per kind asserting status and body.

- `REQ-COMMAND-005` **Into an rmcp tool result (`server` feature).**
  With the `server` feature, `into_mcp()` converts `Result<T, OpError>` into
  `Result<CallToolResult, McpError>` carrying the same envelope as the tool
  result content, with the result flagged as an error when `ok` is false.
  **Why:** an MCP tool body is
  `service::create_widget(&self.stores, input).await.into_mcp()`, and an MCP
  client sees the same envelope a REST client sees (`REQ-E2E-003`).
  **Verified by:** a test for each arm against rmcp's types.

- `REQ-COMMAND-006` **Printable unchanged.**
  Serialising a deserialised envelope reproduces the same JSON value, including
  unknown-to-the-CLI content inside `data` and `details`.
  **Why:** the CLI's `--json` prints the envelope it received unchanged; a lossy
  round-trip would make CLI output differ from the API.
  **Verified by:** a round-trip test over a captured daemon response.

## Non-functional requirements

- `NFR-COMMAND-001` **axum and rmcp only behind `server`.**
  Both are dependencies only behind the `server` feature, which is off by
  default. The default build depends on serde and `sc-observability-types`.
  **Why:** the CLI uses this crate and must link neither (`NFR-REPO-001`).
  **Verified by:** `cargo tree -p sc-command` with default features; builds and
  tests with and without the feature.

- `NFR-COMMAND-002` **Standalone; types only from observability.**
  The crate depends on no other crate in this workspace, and on
  `sc-observability-types` but not on `sc-observability` itself.
  **Why:** `REQ-REPO-002`, ADR-004 and ADR-005.
  **Verified by:** the boundary manifest.

- `NFR-COMMAND-003` **Errors are values.**
  No public function panics.
  **Why:** ADR-011. A conversion that panics would take down a request handler.
  **Verified by:** source review.
