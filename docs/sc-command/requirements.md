# `sc-command` Requirements

Crate-level requirements. Repo-level requirements: [`../requirements.md`](../requirements.md).
Source: [`../sc-runtime-design.md`](../sc-runtime-design.md). Every id is binding and is never reused.

## Purpose

The sc-ai-cli command envelope for any JSON-first CLI or API, with
conversions into an axum response and an rmcp tool result.

## Requirements

- `REQ-COMMAND-001` `Envelope<T>` serialises as `{version, ok, data, error}`:
  `ok: true` carries `data` and a null `error`; `ok: false` carries `error`
  and a null `data`. It round-trips through serde for any `T` that does.
- `REQ-COMMAND-002` `OpError` carries `kind`, `code`, `message`, `details` and
  `suggested_action`, with its code and remediation types taken from
  `sc-observability-types`.
- `REQ-COMMAND-003` `Result<T, OpError>` converts into `Envelope<T>` with
  `into()`.
- `REQ-COMMAND-004` With the `server` feature, `Envelope<T>` implements axum
  `IntoResponse`, returning the JSON envelope with an HTTP status derived from
  the error kind.
- `REQ-COMMAND-005` With the `server` feature, `into_mcp()` converts
  `Result<T, OpError>` into `Result<CallToolResult, McpError>` carrying the
  same envelope, with the tool result flagged as an error when `ok` is false.
- `REQ-COMMAND-006` The envelope a CLI receives can be printed unchanged:
  serialising a deserialised envelope reproduces the same JSON value.

## Non-functional requirements

- `NFR-COMMAND-001` axum and rmcp are dependencies only behind the `server`
  feature, which is off by default. The default build depends on serde and
  `sc-observability-types`.
- `NFR-COMMAND-002` The crate depends on no other crate in this workspace and
  not on `sc-observability` itself.
- `NFR-COMMAND-003` No public function panics.
