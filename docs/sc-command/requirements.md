# `sc-command` Requirements

**ID Range:** REQ-CMD-0001 through REQ-CMD-0006; NFR-CMD-0001 through NFR-CMD-0003  
**Status:** Draft  
**Created:** 2026-09-19  
**Last Updated:** 2026-10-23  
**Owner:** sc-runtime maintainers  

---

Crate-level requirements for `sc-command`. Repo-level requirements are in
[`../requirements.md`](../requirements.md); this crate's architecture and ADRs
are in [`architecture.md`](architecture.md). Extracted from
[`../sc-runtime-design.md`](../sc-runtime-design.md). Entries follow the
shared SC requirement template: each has the sections **Requirement
Statement**, **Rationale** and **Success Criteria**, in that order. Every id
is binding and is never reused.

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

## REQ-CMD-0001: Response envelope type `Envelope<T>` and its JSON shape

**Status:** Active  
**Version:** 1.0.0  
**Created:** 2026-09-19  
**Last Updated:** 2026-11-12
**Owner:** randlee  

### Requirement Statement

Definitions: `sc-command` is the crate in `crates/sc-command`. A default-feature build enables the crate's default Cargo features. The success case is the `Envelope::Success` variant and the failure case is the `Envelope::Failure` variant. A JSON round trip serializes an envelope to JSON and deserializes it back. An envelope JSON equality assertion is a test assertion in `crates/sc-command` that the JSON serializations of two envelopes are equal. The envelope key set is exactly `version`, `ok`, `data` and `error`, with the success/failure values specified by the Key table below.

`Envelope<T>` is the sc-ai-cli response envelope: the one JSON shape every REST response, MCP tool result and CLI `--json` output uses.

| Key | JSON value in the success case | JSON value in the failure case |
|---|---|---|
| `version` | envelope contract version | envelope contract version |
| `ok` | `true` | `false` |
| `data` | the serialised `T` | `null` |
| `error` | `null` | the serialised `OpError` |

`error` is an `OpError`, the typed error with fields `kind`, `code`,
`message`, `details` and `suggested_action`
([REQ-CMD-0002](requirements.md)).

The `data` key and the `error` key are never omitted; a `null` value is serialised as a present key. (Decided in this document; the design gives only the key list.)

Key order in the serialised object is not significant.

A `data` of `null` therefore never by itself means "failure": for `T = ()` and for `T = Option<U>` a success envelope legitimately carries `data: null`, for `Option<U>` with `None`. (Decided in this document; the round-trip obligations below could not otherwise hold for those `T`.)

`Envelope<T>` implements `PartialEq` and `Debug` because the round-trip tests need them. (Decided in this document; [REQ-CMD-0002](requirements.md) requires the same traits of `OpError`.)

The contract version is the JSON string `"1"`. Constructors stamp that
value. Deserialization keeps the `version` string from the input, including
a value other than `"1"`.

`Envelope<T>` is an enum `Success { version: String, data: T }` or
`Failure { version: String, error: OpError }`, with a custom serde
implementation. Accessors are `version`, `is_ok`, `data`, and `error`. A
derived `Deserialize` on a struct of `Option` fields is not the
representation, because it would read a success `data: null` as "no data".

Deserialization rejects an object whose `ok` disagrees with its content
(`ok: true` with a non-null `error`, `ok: false` with a non-null `data`, or
`ok: false` with a null `error`), a missing key, an unknown key, or a value
of the wrong JSON type. The serde error includes the stable code
`ENVELOPE.DISCRIMINANT`. The `version` string does not select a different
key set. A value other than `"1"` is kept, and an unknown key is still
rejected. A new envelope key is a new contract. Unknown fields inside
`data` stay lossless ([REQ-CMD-0006](requirements.md)).

`Envelope::into_result` consumes the envelope and returns `Result<T, OpError>`.
It drops `version`.

**MUST statements:**

- With default features, the root module of `crates/sc-command` MUST export `Envelope<T>`.
- `serde_json::to_value` called on a failure `Envelope<T>` MUST return `Ok`.
- `serde_json::from_str::<sc_command::Envelope<T>>` called on a failure envelope JSON object with `version` `"1"` and a serialised `OpError` MUST return `Ok`.
- The JSON object produced by serialising `Envelope<T>` MUST have a key set equal to the envelope key set.
- In the JSON object produced by serialising a success `Envelope<T>`, the `error` key MUST map to JSON `null`.
- When a failure `Envelope<T>` is serialised to JSON, the `data` key of the resulting object MUST map to JSON `null`.
- Every envelope JSON equality assertion MUST compare `serde_json::Value` values.
- The case of an `Envelope<T>` returned by `serde_json::from_str` MUST be determined by the JSON value of the `ok` key.
- When `ok` is `true`, the envelope deserializer MUST deserialize `data` as `T` even when `data` is JSON `null`.
- When deserialization of an envelope with `ok: true` succeeds, the resulting `Envelope<T>` MUST be the success case.
- When `ok` is `false`, deserialization MUST deserialize the `error` value as `OpError`.
- When deserialization of an envelope with `ok: false` succeeds, the resulting `Envelope<T>` MUST be the failure case.
- For any `T: Serialize + DeserializeOwned + PartialEq`, a JSON round trip of a success `Envelope<T>` MUST yield a value equal to the original success envelope.
- For any `T: Serialize + DeserializeOwned + PartialEq`, a JSON round trip of a failure `Envelope<T>` MUST yield a value equal to the original failure envelope.
- When `T` implements `Debug`, `Envelope<T>` MUST implement `Debug`.
- When `T` implements `Clone`, `Envelope<T>` MUST implement `Clone`.
- When `T` implements `PartialEq`, `Envelope<T>` MUST implement `PartialEq`.

**MUST NOT statements:**

- An envelope JSON equality assertion MUST NOT compare byte strings.

### Rationale

Agents and scripts consume these responses. One fixed shape with an explicit
`ok` boolean means a caller never has to infer success from an HTTP status
code or from which keys happen to be present. Using the same type in the
daemon (to answer) and in the CLI (to parse and re-print) guarantees both
sides agree on the shape at compile time.

### Success Criteria

1. A unit test in `crates/sc-command` serialises a success envelope built
   from a sample `T` (a struct with at least one field) with
   `serde_json::to_value` and asserts: the object has exactly the keys
   `version`, `ok`, `data`, `error`; `ok` is `true`; `data` equals the
   serialised `T`; `error` is JSON `null`.
2. A unit test serialises a failure envelope built from a sample `OpError`
   and asserts: exactly the same four keys; `ok` is `false`; `data` is JSON
   `null`; `error` equals the serialised `OpError`.
3. A unit test asserts the `version` value is identical in the outputs of
   criteria 1 and 2.
4. A round-trip test, for the success case and for the failure case,
   serialises an `Envelope<T>` with `serde_json::to_string`, deserialises the
   string with `serde_json::from_str::<Envelope<T>>`, and asserts equality
   with the original value.
5. A round-trip test does the same for a success `Envelope<()>` and asserts:
   the serialised `data` is JSON `null`, `ok` is `true`, and the
   deserialised value equals the original success envelope (it is not the
   failure case and deserialisation does not fail).
6. A round-trip test does the same for `Envelope<Option<Sample>>` three
   times: success with `Some(sample)`, success with `None` (serialised
   `data` is JSON `null`, and the deserialised value equals the original
   success envelope holding `None`), and the failure case.
7. A compile-time check in a test requires
   `Envelope<Sample>: Debug + Clone + PartialEq` for a `Sample` that derives those three traits (for example
   a function `fn assert_traits<X: Debug + Clone + PartialEq>()` called with
   `Envelope<Sample>`).
8. `cargo test -p sc-command` (default features) runs the tests above and
   exits 0.
9. A unit test calls `into_result` on a success envelope and on a failure
   envelope and asserts `Ok(T)` and `Err(OpError)`.

---

## REQ-CMD-0002: Typed error `OpError` and its five fields

**Status:** Active  
**Version:** 1.0.0  
**Created:** 2026-09-19  
**Last Updated:** 2026-11-12
**Owner:** randlee  

### Requirement Statement

Definitions: CLI means command-line interface. JSON means JavaScript Object Notation. OpError is the error value with kind, code, message, details and suggested_action fields; ErrorKind is its classification enum. HTTP means Hypertext Transfer Protocol. API means application programming interface. ADR means architecture decision record. The canonical remediation type is the suggested_action representation supplied by sc-observability-types; a parallel remediation type would duplicate that representation. The five wire keys are `kind`, `code`, `message`, `details` and `suggested_action`. Equal JSON values are equal as `serde_json::Value` (REQ-CMD-0006).

Dependency requirement: `sc-command` declares registry `sc-observability-types`
with the tested floor `>=1.5.0` and no upper bound (REQ-RUN-0002, ANC-037).
An upper bound is any requirement that excludes a later version, including an
exact `=` pin, a tilde requirement or an explicit `<` limit. The historical
0.1.1 release used a temporary exact `=1.5.0` pin under ADR-RUN-0017; that pin
does not govern Phase D. OpError wire fields and serialization are unchanged.

`OpError` is the error half of the response envelope `Envelope<T>` ([REQ-CMD-0001](requirements.md)) and the error type service functions return in `Result<T, OpError>`. A generated project's `service` crate therefore uses `sc-command` with default features; the generated workspace's crate graph, including how `service` and `daemon` obtain `sc-command`, is owned by [REQ-RUN-0301](../requirements.md).

The keys `kind`, `code`, `message`, `details`, and `suggested_action` are always present. `details` and `suggested_action` are JSON `null` when the Rust field is empty. This item is the owner of which `OpError` keys are present on the wire; other items refer to "the keys defined by REQ-CMD-0002".

| Field | Meaning | Type |
|---|---|---|
| `kind` | coarse error category; the HTTP status is derived from it | `sc_command::ErrorKind`, a public enum exported from the crate root (the type name `ErrorKind` is decided in this document) |
| `code` | stable machine-readable code that automation branches on, for example `DAEMON.NOT_RUNNING` | `sc_observability_types::ErrorCode` |
| `message` | human-readable summary | `String` |
| `details` | structured machine-readable context | `Option<serde_json::Value>`; JSON `null` when absent |
| `suggested_action` | wire name for remediation | Rust field `remediation: Option<sc_observability_types::Remediation>`; on the wire a JSON string, or JSON `null` when absent |

`Display` for `OpError` is `{code}: {message}`. `Display` for `ErrorKind` is its wire spelling. There is no `From<anyhow::Error>` and no constructor from `Box<dyn Error>`. (The comparison traits are decided in this document; the round-trip tests need them, and `sc_observability_types::ErrorCode` already implements them. `Display` and `std::error::Error` let a consumer use `?` without a local wrapper.)

`ErrorCode` is a newtype over a string and serialises as a JSON string, so `code` appears on the wire as, for example, `"DAEMON.NOT_RUNNING"`.

(The design's only example shows a plain string.)

Serialising `Remediation::NotRecoverable` is the one projection that inserts `justification` into the wire `details` value.

`ErrorKind` variants and their wire spellings are `invalid_input`,
`not_found`, `conflict`, `unavailable`, and `internal`. The enum is
`#[non_exhaustive]`. An unrecognised `kind` string fails deserialization.
HTTP status is 400, 404, 409, 503, and 500 in that order, and that mapping
lives in `ErrorKind::http_status_code` only. `details` is
`Option<serde_json::Value>` and is JSON `null` when absent. The Rust field
behind wire `suggested_action` is
`Option<sc_observability_types::Remediation>`. `None` serialises as JSON
`null`. `Remediation::Recoverable` serialises as a JSON string equal to the
first recovery step when that step is non-empty. Later steps are not on the
wire. An empty step list, an empty first step, and
`Remediation::NotRecoverable` serialise `suggested_action` as JSON `null`,
never as `""`. A not-recoverable justification is copied into wire `details`
under `justification` and is not an instruction on `suggested_action`.
Deserialising a non-empty string builds a one-step `Recoverable`.
Deserialising `null` or `""` builds `None`. Multi-step lists and
`NotRecoverable` do not round-trip as the same Rust value. The tagged
`Remediation` object is not the wire form. `OpError` does not convert to or
from `sc_observability_types::Diagnostic`. `ErrorKind` is defined in
`sc-command`. `sc-observability-types` 1.5.0 has no `ErrorKind`, `OpError`,
or `suggested_action`.

`OpError::from_transport_strings` maps the plain strings `code`, `endpoint`,
`cause`, and `suggested_action` into this type without a dependency on
`sc-transport`. The code `DAEMON.NOT_RUNNING` selects
`ErrorKind::Unavailable` and the message `the daemon is not running`. Any
other code selects `ErrorKind::Internal` and the message
`transport failure {code} at {endpoint}: {cause}`. `suggested_action` is the
caller's already-rendered recovery text. `None`, or text that is empty after
trimming, leaves remediation unset. The crate does not emit a placeholder
application name. `endpoint` and `cause` are stored in `details`.

The CLI parses errors it receives from the daemon. REQ-CMD-0006 depends on details returning arbitrary JSON unchanged.

Example: an `OpError` built with the suggestion `run my-app daemon start` serialises with `"suggested_action": "run my-app daemon start"`.

The tested minimum is 1.5.0.

**MUST statements:**

- The crate `sc-command` MUST export the public type `OpError` from its crate root.
- `sc_command::OpError` MUST be available with default features.
- `OpError` MUST serialise to a JSON object that has every one of the five wire keys.
- `OpError` MUST serialise to a JSON object that has no key other than the five wire keys.
- `OpError` MUST implement `serde::Serialize`.
- `OpError` MUST implement `serde::Deserialize`.
- `ErrorKind` MUST implement `serde::Serialize`.
- `ErrorKind` MUST implement `serde::Deserialize`.
- `OpError` MUST implement `Debug`.
- `OpError` MUST implement `Clone`.
- `OpError` MUST implement `PartialEq`.
- `OpError` MUST implement `Display`.
- `ErrorKind` MUST implement `Debug`.
- `ErrorKind` MUST implement `Clone`.
- `ErrorKind` MUST implement `PartialEq`.
- `ErrorKind` MUST implement `Display`.
- `OpError` MUST implement `std::error::Error`.
- `code` MUST be the type `ErrorCode` from the published crate `sc-observability-types`.
- `crates/sc-command/Cargo.toml` MUST declare `sc-observability-types` with the registry requirement `">=1.5.0"`.
- When a suggestion is supplied, `suggested_action` MUST serialise as a JSON string equal to the text supplied.
- A remediation value that `OpError` holds or is built from MUST be of type `sc_observability_types::Remediation` or `sc_observability_types::RecoverableSteps`.
- The Rust type of the `details` field MUST be `Option<serde_json::Value>`.
- Serialising the `details` field after deserialising it as part of `OpError` MUST produce an equal JSON value.

**MUST NOT statements:**

- `sc-command` MUST NOT define its own error-code type.
- The `suggested_action` wire field MUST NOT serialise as a JSON object.
- `sc-command` MUST NOT define a parallel remediation type.
- The `sc-observability-types` requirement in `crates/sc-command/Cargo.toml` MUST NOT declare an upper bound.

### Rationale

`code` is what automation branches on. `suggested_action` is what lets an
agent recover without a human. `kind` lets the REST edge pick an HTTP status
without parsing codes. Taking the code type, and any structured remediation,
from `sc-observability-types`, the crate the SC logging stack already uses
for them, keeps an error code identical in an API response and in the log
line about the same failure, with one definition to maintain.
`suggested_action` is a plain string on the wire because that is the form
the CLI's `DAEMON.NOT_RUNNING` error is specified in
([REQ-RUN-0202](../requirements.md)) and the simplest form for an agent to
act on.

### Success Criteria

1. A unit test constructs an `OpError` with all five fields set (`details`
   holding at least one entry, and the suggestion
   `run my-app daemon start`), serialises it with `serde_json::to_value`,
   and asserts the
   object has exactly the keys `kind`, `code`, `message`, `details`,
   `suggested_action`, and that `suggested_action` is the JSON string
   `"run my-app daemon start"`.
2. The same test asserts `code` serialises as a JSON string equal to the
   code given at construction (use `DAEMON.NOT_RUNNING`) and `message` as a
   JSON string.
3. A round-trip test serialises that `OpError` to a string, deserialises it
   as `OpError`, and asserts equality with the original.
4. A compile-time check in a test assigns `err.code` (or the value returned
   by its accessor) to a binding of type
   `sc_observability_types::ErrorCode`.
5. Inspection of `crates/sc-command/Cargo.toml`: `sc-observability-types`
   is listed under `[dependencies]` with version requirement `">=1.5.0"`, has
   no upper bound, and is not `optional`.
6. Inspection of `crates/sc-command/src`: no type defined there duplicates
   `ErrorCode`, `Remediation` or `RecoverableSteps`.
7. A unit test constructs an `OpError` with no details and no suggestion,
   serialises it, and asserts: the keys `kind`, `code` and `message` are
   present; every key of the object is one of the five names; `details` is
   JSON `null`; `suggested_action` is JSON `null`.
8. A compile-time check in a test requires
   `OpError: Debug + Clone + PartialEq + Display + std::error::Error` and
   `ErrorKind: Debug + Clone + PartialEq + Display`.

### Related Documents

**Architecture Decisions:**

- ADR-RUN-0017 — 2026-10-31 amendment

---

## REQ-CMD-0003: `From<Result<T, OpError>>` for `Envelope<T>`

**Status:** Active  
**Version:** 1.0.0  
**Created:** 2026-09-19  
**Last Updated:** 2026-09-19  
**Owner:** randlee  

### Requirement Statement

Definitions: `sc-command` is the crate in `crates/sc-command`. `Envelope<T>` is the response envelope with JSON keys `version`, `ok`, `data` and `error` ([REQ-CMD-0001](requirements.md)). `OpError` is the typed error ([REQ-CMD-0002](requirements.md)). The conversion is the call `.into()` on a `Result<T, OpError>` value that yields an `Envelope<T>`. The envelope contract version is the JSON string `"1"` ([REQ-CMD-0001](requirements.md)). The serialised envelope is the JSON produced by `serde_json` from the `Envelope<T>` returned by the conversion.

**MUST statements:**

- `sc-command` built with default features MUST compile `impl<T> From<Result<T, OpError>> for Envelope<T>`.
- For input `Ok(value)`, the serialised envelope MUST have `ok` equal to `true`.
- For input `Ok(value)`, the serialised envelope MUST have `data` equal to the serialised `value`.
- For input `Ok(value)`, the serialised envelope MUST have `error` equal to JSON `null`.
- For input `Err(op_error)`, the serialised envelope MUST have `ok` equal to `false`.
- For input `Err(op_error)`, the serialised envelope MUST have `data` equal to JSON `null`.
- For input `Err(op_error)`, the serialised envelope MUST have `error` equal to the serialised `op_error`.
- The serialised envelope MUST have `version` equal to the envelope contract version.

**MUST NOT statements:**

- The conversion MUST NOT accept a `version` value from the caller.
- `impl<T> From<Result<T, OpError>> for Envelope<T>` MUST NOT require any trait bound on `T`.

### Rationale

Service functions in a generated project return `Result<T, OpError>`. With
this conversion an axum handler whose return type is `Envelope<Widget>` has a
one-line body, `service::create_widget(&s, input).await.into()`, and no
project writes its own success-or-failure wrapping code.

### Success Criteria

1. A unit test converts `Ok::<Sample, OpError>(sample)` with `.into()` into
   `Envelope<Sample>`, serialises it, and asserts `ok` is `true`, `data`
   equals the serialised `sample`, and `error` is JSON `null`.
2. A unit test converts `Err::<Sample, OpError>(op_error)` with `.into()`,
   serialises it, and asserts `ok` is `false`, `data` is JSON `null`, and
   `error` equals the serialised `op_error`.
3. A unit test asserts the `version` value is the same in the two envelopes
   above.
4. A test compiles `let _: Envelope<NotSerialize> = Ok(NotSerialize).into();`
   where `NotSerialize` implements no serde trait, proving the conversion
   has no bound on `T`.
5. `cargo test -p sc-command` (default features, without `server`) runs
   these tests and exits 0.

---

## REQ-CMD-0004: axum `IntoResponse` for `Envelope<T>` (`server` feature)

**Status:** Active  
**Version:** 1.0.0  
**Created:** 2026-09-19  
**Last Updated:** 2026-11-12
**Owner:** randlee  

### Requirement Statement

When the cargo feature `server` of `sc-command` is enabled, the crate MUST
provide

```rust
impl<T: serde::Serialize> axum::response::IntoResponse for Envelope<T>
```

so that an axum 0.8 handler can declare `Envelope<T>` as its return type.
`Envelope<T>` is the response envelope with JSON keys `version`, `ok`,
`data`, `error` ([REQ-CMD-0001](requirements.md)).

This rule is decided in this document; the design specifies that the conversion exists and does not specify the mapping.

| Envelope | HTTP status |
|---|---|
| success case (`ok` is `true`) | `200 OK` |
| failure, `ErrorKind::InvalidInput` | `400 Bad Request` (caller error) |
| failure, `ErrorKind::NotFound` | `404 Not Found` (caller error) |
| failure, `ErrorKind::Conflict` | `409 Conflict` (caller error) |
| failure, `ErrorKind::Unavailable` | `503 Service Unavailable` (internal error) |
| failure, `ErrorKind::Internal` | `500 Internal Server Error` (internal error) |

`error.kind` is the `ErrorKind` field of `OpError` ([REQ-CMD-0002](requirements.md)). The first three kinds are caller errors. `Unavailable` and `Internal` are internal errors.

HTTP means Hypertext Transfer Protocol. JSON means JavaScript Object Notation. The no-panic rule for this conversion is governed by [NFR-CMD-0003](requirements.md), the sc-command non-functional requirement for typed errors and panic prevention.

The crate references axum when its build depends on the `axum` crate or when its source names `axum` outside an item or module gated by `#[cfg(feature = "server")]`. In `crates/sc-command/Cargo.toml`, `axum` is an optional dependency enabled only by the `server` feature.

Definitions: `envelope` is the `Envelope<T>` value on which `into_response` is called. A serialisation failure is an error returned by the `Serialize` implementation of `T` while `into_response` serialises `envelope`. The serialisation failure envelope is a failure envelope ([REQ-CMD-0001](requirements.md)) whose `error.code` is `ENVELOPE.SERIALIZATION`.

**MUST statements:**

- When no serialisation failure occurs, the response body, parsed as JSON, MUST equal `serde_json::to_value(&envelope)`.
- The response MUST carry the header `Content-Type: application/json`.
- The HTTP status MUST be the one that the Envelope to HTTP status table in this record gives for the envelope.
- Two failure envelopes whose `error.kind` values are equal MUST get the same HTTP status, whatever their `error.code` values.
- When a serialisation failure occurs, the HTTP status of the response from `into_response` MUST be 500.
- When a serialisation failure occurs, the response body, parsed as JSON, MUST be the serialisation failure envelope.

**MUST NOT statements:**

- The `IntoResponse::into_response` implementation for `Envelope<T>` MUST NOT panic.
- When the `server` feature is disabled this implementation MUST NOT exist.
- When the `server` feature is disabled the crate MUST NOT reference axum.

### Rationale

REST clients, proxies and monitoring that look only at the HTTP status still
behave sensibly (retries on 5xx, no retries on 4xx), while the envelope in
the body stays the source of truth for agents. Putting the mapping in one
crate means every SC daemon maps the same error kind to the same status.

### Success Criteria

1. A test compiled with `--features server` calls `.into_response()` on a
   success envelope and asserts: status is `200`; the `content-type` header
   is `application/json`; the body, parsed as JSON, equals
   `serde_json::to_value` of the same envelope.
2. For every `ErrorKind` variant, a test builds a failure envelope with
   that kind, calls `.into_response()`, and asserts: the status equals the
   entry for that kind in the per-kind table; the body, parsed as JSON,
   equals `serde_json::to_value` of the envelope and has `ok` equal to
   `false`. The test iterates over all variants so that adding a variant
   without a table entry fails.
3. A test asserts every caller-error kind yields a status in `400..=499`
   and every internal-error kind a status in `500..=599`.
4. `cargo test -p sc-command --features server` exits 0.
5. `cargo build -p sc-command` (default features) exits 0, and
   `grep -rn "axum" crates/sc-command/src` shows every occurrence is inside
   an item or module gated by `#[cfg(feature = "server")]`.

---

## REQ-CMD-0005: `into_mcp()` to rmcp `CallToolResult` (`server` feature)

**Status:** Active  
**Version:** 1.0.0  
**Created:** 2026-09-19  
**Last Updated:** 2026-11-13
**Owner:** randlee  

### Requirement Statement

Definitions:

- MCP means Model Context Protocol; JSON means JavaScript Object Notation.
- `rmcp` is the Rust MCP crate; Phase C pins rmcp 3.8 (ADR-RUN-0203) and Phase D requires rmcp >=3.9 with no upper bound ([NFR-CMD-0001](requirements.md)).
- A server build is a build of the `sc-command` crate (`crates/sc-command`) with the Cargo feature `server` enabled.
- `OpError` is the service error type of `sc-command` ([REQ-CMD-0002](requirements.md)).
- `T` is any type that implements `serde::Serialize`.
- `CallToolResult` is the type `rmcp::model::CallToolResult`.
- `McpError` is the type `rmcp::ErrorData`, re-exported as `sc_command::McpError`.
- `is_error` is the `Option<bool>` field of `CallToolResult` that marks a failed tool call.
- `structured_content` is the JSON value field of `CallToolResult`.
- A text content item is an element of the `content` list of `CallToolResult` whose value is `rmcp::model::RawContent::Text`.
- The input result is the `Result<T, OpError>` value on which `into_mcp` is called.
- The input envelope is `Envelope::<T>::from(input result)`, where `Envelope<T>` is the response envelope of [REQ-CMD-0001](requirements.md).
- The envelope JSON is `serde_json::to_value` of the input envelope.
- The output is the `CallToolResult` that `into_mcp` returns inside `Ok`.
- The HTTP body is the body of `Envelope::from(input result).into_response()` ([REQ-CMD-0004](requirements.md)) parsed as a JSON value.
- The data code of an `McpError` is the string under the key `code` in the JSON object of its `data` field; the data code differs from the `code` field of the `McpError`.
- A server-gated item is an item or module marked `#[cfg(feature = "server")]`, or any code nested inside one.
- A panic is a Rust panic raised during the call, including one from `unwrap` or `expect`.
- A serialization failure is `serde_json` returning an error while serializing the input envelope.
- The required signature is `fn into_mcp(self) -> Result<CallToolResult, McpError>`.
- An rmcp path is a path in `crates/sc-command/src` whose first segment is `rmcp`.
- The serialization error is the `McpError` inside the `Err` that `into_mcp` returns on a serialization failure.
- The required impl is an `impl` item of `IntoMcp` for `Result<T, OpError>` whose only bound on `T` is `serde::Serialize`.
- A sealed supertrait is a trait declared in a private module of `crates/sc-command/src`, so code outside `sc-command` cannot name it.

**MUST statements:**

- In a server build, `sc_command::IntoMcp` MUST be a public trait.
- In `crates/sc-command/src`, the trait `IntoMcp` MUST declare exactly one method.
- The signature of `IntoMcp::into_mcp` in `crates/sc-command/src` MUST be the required signature.
- A file in `crates/sc-command/src` MUST define the required impl.
- In `crates/sc-command/src`, the supertraits of `IntoMcp` MUST include `Sized`.
- In `crates/sc-command/src`, the supertraits of `IntoMcp` MUST include a sealed supertrait.
- The body of `IntoMcp::into_mcp` in `crates/sc-command/src` MUST obtain the input envelope by calling `Envelope::<T>::from` on the input result.
- The output MUST contain exactly one text content item.
- The text of the output's text content item, parsed as JSON, MUST equal the envelope JSON.
- The `structured_content` field of the output MUST equal the envelope JSON.
- The `structured_content` field of the output MUST equal the HTTP body for the same input result.
- When the `ok` key of the envelope JSON is `false`, the `is_error` field of the output MUST be `Some(true)`.
- On a serialization failure, `into_mcp` MUST return `Err`.
- On a serialization failure, the `code` field of the serialization error MUST equal `rmcp::model::ErrorCode::INTERNAL_ERROR`.
- The data code of the serialization error MUST be `ENVELOPE.SERIALIZATION`.

- The definition of `IntoMcp` in `crates/sc-command/src` MUST lie in a server-gated item.
- Every rmcp path MUST lie in a server-gated item.
- In `crates/sc-command/Cargo.toml`, the `rmcp` dependency MUST be declared `optional = true`.

**MUST NOT statements:**

- When the `ok` key of the envelope JSON is `true`, the `is_error` field of the output MUST NOT be `Some(true)`.
- When the input result is `Err(OpError)` and no serialization failure occurs, `into_mcp` MUST NOT return `Err`.
- `IntoMcp::into_mcp` MUST NOT panic ([NFR-CMD-0003](requirements.md)).

### Rationale

Turning a service `OpError` into an MCP protocol error would hide the response envelope from the client.

An rmcp tool body in a generated project is one line,
`service::create_widget(&self.stores, input).await.into_mcp()`, mirroring the
one-line axum handler. An MCP client then sees exactly the envelope a REST
client sees, which the product requires of every surface
([REQ-RUN-0203](../requirements.md)), and MCP hosts that look only at
`is_error` still learn that the call failed.

The error type is rmcp's, not an `sc-command` error enum, because rmcp
dictates the signature of a `#[tool]` function. This is the recorded exception
to the rule that a fallible public function returns the crate's own typed
error enum ([NFR-RUN-0009](../requirements.md), restated in
[NFR-CMD-0003](requirements.md)). The design fixes only the method name
`into_mcp()`; the trait name `IntoMcp`, the sealing and the `is_error` rule are
decided here.

### Success Criteria

In criteria 1 to 3, "the envelope JSON extracted from the `CallToolResult`"
means the JSON value in `structured_content`. The text content item holds
the same JSON as a string.

1. A test compiled with `--features server` calls
   `Ok::<Sample, OpError>(sample).into_mcp()` and asserts: the result is
   `Ok`; `is_error` is `None` or `Some(false)`; the envelope JSON extracted
   from the `CallToolResult` content has `ok` equal to `true` and `data`
   equal to the serialised `sample`.
2. A test calls `Err::<Sample, OpError>(op_error).into_mcp()` and asserts:
   the result is `Ok`; `is_error` is `Some(true)`; the extracted envelope
   JSON has `ok` equal to `false`, `data` equal to `null`, and `error` equal
   to the serialised `op_error`.
3. For the success input and for the failure input, a test asserts the
   envelope JSON extracted from the `CallToolResult` equals, as a
   `serde_json::Value`, the parsed body of
   `Envelope::from(same_result).into_response()`.
4. The tests import `CallToolResult` and the error type from the `rmcp`
   crate itself, not local stand-ins.
5. `cargo test -p sc-command --features server` exits 0, and
   `grep -rn "rmcp" crates/sc-command/src` shows every occurrence is inside
   an item or module gated by `#[cfg(feature = "server")]`.

---

## REQ-CMD-0006: `Envelope` JSON round-trip is lossless for CLI `--json`

**Status:** Active  
**Version:** 1.0.0  
**Created:** 2026-09-19  
**Last Updated:** 2026-10-23  
**Owner:** randlee  

### Requirement Statement

Definitions: JSON means JavaScript Object Notation. T is the caller-chosen value type in Envelope<T>. OpError is the error value defined in [REQ-CMD-0002](requirements.md); its `details` field carries arbitrary JSON context, written error.details inside an envelope. The details round trip is deserialising a JSON value into the `details` field of `OpError` and serialising that field again; its input is that JSON value and its output is the JSON produced. An undeclared key is a JSON key that no Rust struct in the CLI or in `sc-command` declares. A null key is a key whose value is `null`; in a success envelope `error` is a null key, and in a failure envelope `data` is a null key. A typed struct is a Rust struct with named fields used as `T`, such as `Widget` below; it ignores unknown fields when deserialising drops keys it does not declare. The round trip is deserialising a JSON envelope into `sc_command::Envelope<T>` and serialising that value again; its input is the JSON envelope and its output is the JSON produced. Equality means `serde_json::Value` equality: same keys, same values, same `null`s; key order and whitespace are not compared. The envelope is the object with keys `version`, `ok`, `data`, `error` ([REQ-CMD-0001](requirements.md)).

This requirement does not cover a typed `T` that ignores unknown fields:
deserialising into `Envelope<Widget>` legitimately drops fields `Widget`
does not declare. A CLI that must print the response unchanged deserialises
into `Envelope<serde_json::Value>`.

**MUST statements:**

- The output of the round trip MUST equal its input.
- For `T = serde_json::Value`, the output of the round trip MUST equal its input for any JSON value inside `data`, including undeclared keys.
- `error.details` in the output of the round trip MUST equal `error.details` in its input, also when `T` is a typed struct that ignores unknown fields.
- The output of the details round trip MUST equal the JSON value the details round trip started from.
- Each null key of the input MUST be a null key of the output.
- The `version` value in the output MUST be the value read from the input, not the version constant of the `sc-command` build doing the printing.

**MUST NOT statements:**

- The round trip MUST NOT drop a null key of its input.

### Rationale

A CLI's `--json` flag prints the envelope it received from the daemon
unchanged. If the round-trip through `Envelope` lost or rewrote anything, the
CLI's output would differ from the API's for the same call, and an agent
could not treat the two as interchangeable. The risk is greatest when the
daemon is newer than the CLI and sends payload fields the CLI does not
know. Those fields are inside `data` and `details`. An unknown key of the
envelope object or of the error object is rejected by
[REQ-CMD-0001](requirements.md). Adding such a key is a new contract.

### Success Criteria

1. `crates/sc-command/tests/` contains a JSON fixture of a success envelope
   as a daemon would send it, whose `data` object includes at least one
   nested object and one array. A test parses the fixture to a
   `serde_json::Value`, deserialises that into
   `Envelope<serde_json::Value>`, serialises it with `serde_json::to_value`,
   and asserts the result equals the parsed fixture.
2. The same directory contains a JSON fixture of a failure envelope whose
   `error.details` holds nested JSON not modelled by any `sc-command` type.
   The same test procedure asserts equality.
3. The failure-fixture test is repeated with `T` set to a typed struct
   (`Envelope<Sample>`) and asserts `error.details` in the output equals
   `error.details` in the fixture.
4. A test asserts that the output for the success fixture contains the key
   `error` with value `null`, and the output for the failure fixture
   contains the key `data` with value `null`.
5. A test feeds an envelope whose `version` differs from the crate's own
   version value and asserts the output `version` equals the input
   `version`.

---

## NFR-CMD-0001: `sc-command` links axum and rmcp only behind `server`

**Status:** Active  
**Version:** 1.0.0  
**Created:** 2026-09-19  
**Last Updated:** 2026-10-23  
**Owner:** randlee  

### Requirement Statement

Definitions: MCP means Model Context Protocol, implemented by rmcp. The server feature enables server-side axum and rmcp conversions. Third-party dependencies are crates outside the public workspace package set in REQ-RUN-0002; workspace edges are governed by NFR-CMD-0002, including `sc-storage` for the shared command helpers. An upper bound is any version requirement that excludes a later version, including an exact `=` pin, a tilde requirement or an explicit `<` limit. `IntoResponse` is the axum trait that converts a value into an HTTP response. `IntoMcp` is the `sc-command` trait whose method `into_mcp` converts a value into an rmcp `CallToolResult` (REQ-CMD-0005).

Phase D dependency requirements: every registry dependency of `sc-command`
uses its tested floor with no upper bound (REQ-RUN-0002, ANC-037). The floors
are `sc-observability-types >=1.5.0`, `axum >=0.8.9` and `rmcp >=3.9`. The
Phase C `~3.8` rmcp allowlist, its frozen resolved identity 3.8.0 and the
historical 0.1.1 exact `=1.5.0` types pin do not govern Phase D. The crate's
own tests check compatibility at the selected versions. All
optional/server/defaults-off gating and remaining dependency, conversion and
error restrictions remain binding; this does not permit server dependencies in
a CLI's isolated normal graph. REQ-CMD-0005's nominal identity amendment still
applies.

The same c-1 contract also adds independent optional
`schema = ["dep:schemars", "dep:utoipa"]`, with compatible identities frozen
by ADR-RUN-0203. Schema-only adds those schema dependencies to the otherwise
unchanged default allowlist; schema+server uses the union of the two explicit
feature allowlists. Neither schema nor server enables the other or enters
defaults. C-1 owns compileable owner-local JsonSchema/ToSchema implementations
for Envelope<T>, OpError and ErrorKind in the narrow declaration modules before
c-3's later migration. Envelope's custom four-field wire representation and
OpError's five fields require schemas matching existing serialization exactly,
not a derive over internal enum variants. Foreign ErrorCode is represented by
its canonical serialized leaf shape inside the owned schema, without orphan
impls or duplicate error models. REQ-CMD-0001/0002 wire behavior is unchanged.

2026-10-31 amendment: ADR-RUN-0017 adds an unconditional neutral
`observability` contract module and non-default `test-util = []` doubles using
only std. Neither activates server/axum/rmcp, and no third-party dependency is
added.

`serde_json` is required because `details` holds arbitrary JSON.

This item is the owner of the list of third-party crates `sc-command` may
depend on. The repository-level boundary table
([REQ-RUN-0005](../requirements.md)) refers to this item for that list and
owns only the workspace-internal and forbidden edges, which are restated for
this crate in [NFR-CMD-0002](requirements.md).

Dev-dependencies are outside this rule: tests of the `server` feature may
use axum and rmcp.

**MUST statements:**

- In `crates/sc-command/Cargo.toml`, `axum` MUST be declared `optional = true`.
- In `crates/sc-command/Cargo.toml`, `rmcp` MUST be declared `optional = true`.
- In `crates/sc-command/Cargo.toml`, the set of `[features]` entries that list `dep:axum` MUST equal {`server`}.
- In `crates/sc-command/Cargo.toml`, the set of `[features]` entries that list `dep:rmcp` MUST equal {`server`}.
- The exit status of `cargo build -p sc-command` MUST be 0.
- In `crates/sc-command/Cargo.toml`, the set of non-optional third-party `[dependencies]` entries MUST equal {`serde`, `serde_json`, `sc-observability-types`}.
- In `crates/sc-command/Cargo.toml`, `sc-observability-types` MUST be declared with the registry requirement `">=1.5.0"`.
- In `crates/sc-command/Cargo.toml`, the set of crates named by `dep:` entries in the `server` feature list MUST equal {`axum`, `rmcp`}.
- In `crates/sc-command/Cargo.toml`, `axum` MUST be declared with the registry requirement `">=0.8.9"`.
- In `crates/sc-command/Cargo.toml`, `rmcp` MUST be declared with the registry requirement `">=3.9"`.
- In `crates/sc-command/Cargo.toml`, `axum` MUST be declared with `default-features = false`.
- The exit status of `cargo build -p sc-command --features server` MUST be 0.
- In `crates/sc-command/src/`, the `impl IntoResponse for Envelope<T>` block MUST have an enclosing `#[cfg(feature = "server")]` attribute.
- In `crates/sc-command/src/lib.rs`, the `pub use` line that exports `IntoMcp` MUST carry `#[cfg(feature = "server")]`.

**MUST NOT statements:**

- In `crates/sc-command/src/lib.rs`, the `pub use` line that exports `Envelope` MUST NOT carry a `#[cfg(feature = ...)]` attribute.
- In `crates/sc-command/src/lib.rs`, the `pub use` line that exports `OpError` MUST NOT carry a `#[cfg(feature = ...)]` attribute.
- In `crates/sc-command/src/lib.rs`, the `pub use` line that exports `ErrorKind` MUST NOT carry a `#[cfg(feature = ...)]` attribute.
- In `crates/sc-command/src/envelope.rs`, the `impl From<Result<T, OpError>> for Envelope<T>` block MUST NOT have an enclosing `#[cfg(feature = ...)]` attribute.
- In `crates/sc-command/Cargo.toml`, the `default` feature list MUST NOT contain `server`.
- In `crates/sc-command/src/`, code and comments outside `#[cfg(feature = "server")]` modules and items MUST NOT contain the text `axum`.
- In `crates/sc-command/src/`, code and comments outside `#[cfg(feature = "server")]` modules and items MUST NOT contain the text `rmcp`.
- In `crates/sc-command/Cargo.toml`, the `sc-observability-types` requirement MUST NOT declare an upper bound.
- In `crates/sc-command/Cargo.toml`, the `axum` requirement MUST NOT declare an upper bound.
- In `crates/sc-command/Cargo.toml`, the `rmcp` requirement MUST NOT declare an upper bound.

### Rationale

A CLI built on these crates uses `sc-command` to parse and print envelopes,
and a CLI binary must link none of axum, rmcp or sqlx
([NFR-RUN-0001](../requirements.md)): linking the daemon's server stack
would multiply the CLI's build time and binary size. Gating the two
server-side conversions behind a feature that is off by default lets the
daemon and the CLI share one crate.

### Success Criteria

1. `cargo tree -p sc-command -e normal --prefix none` (default features)
   prints no line beginning with `axum ` or `rmcp `.
2. `cargo tree -p sc-command -e normal --prefix none --features server`
   prints a line beginning with `axum ` and a line beginning with `rmcp `.
3. `cargo test -p sc-command` exits 0.
4. `cargo test -p sc-command --features server` exits 0.
5. Inspection of `crates/sc-command/Cargo.toml`: `axum` and `rmcp` each
   have `optional = true`; `[features]` has a `server` entry enabling both;
   `default` does not list `server`.
6. CI runs the commands in criteria 3 and 4 through the `just test` recipe,
   which runs both of them ([REQ-RUN-0004](../requirements.md)); CI does not
   call cargo directly.
7. `cargo tree -p sc-command -e normal --depth 1 --prefix none` (default
   features) prints, among the third-party dependency lines below the
   `sc-command` root line, exactly one line
   beginning with `serde `, one beginning with `serde_json `, and one
   beginning with `sc-observability-types `, and no other third-party line.
   Workspace dependencies are checked against NFR-CMD-0002; its allowed
   `sc-storage` edge is not rejected by this third-party check. The
   `sc-observability-types ` line is version 1.5.0 or later.
8. The same command with `--features server` prints the lines of criterion
   7 plus exactly one line beginning with `axum ` and one beginning with
   `rmcp `, and no other third-party line.
9. Inspection of `crates/sc-command/Cargo.toml`: the requirements are
   `sc-observability-types = ">=1.5.0"`, `axum = ">=0.8.9"` and
   `rmcp = ">=3.9"`, and none has an exact pin, caret, tilde or explicit upper bound.

### Related Documents

**Requirements:**

- REQ-CMD-0005 — Phase C frozen amendment
- REQ-CMD-0001 — Phase C frozen amendment

**Architecture Decisions:**

- ADR-RUN-0203 — Phase C frozen amendment
- ADR-RUN-0017 — 2026-10-31 amendment

---

## NFR-CMD-0002: `sc-command` dependency boundary

**Status:** Active  
**Version:** 1.0.0  
**Created:** 2026-09-19  
**Last Updated:** 2026-11-12
**Owner:** randlee  

### Requirement Statement

Definitions:

- A workspace crate is a package in the public workspace package set of [REQ-RUN-0002](../requirements.md).
- An upper bound is any version requirement that excludes a later version, including an exact `=` pin, a caret requirement, a tilde requirement or an explicit `<` limit.
- A production feature combination is any set of `sc-command` cargo features, including `server`, excluding dev-dependencies.
- The production dependency tree is every package, direct or transitive, that `cargo tree -p sc-command -e normal,build --prefix none` prints for any production feature combination.
- A manifest dependency of `sc-command` is an entry under `[dependencies]`, `[dev-dependencies]` or `[build-dependencies]` in `crates/sc-command/Cargo.toml`.
- `sc-observability` is the logging runtime crate; `sc-observability-otlp` is its OpenTelemetry export crate; `sc-observability-types` is the types-only crate of the same family.

Context: the historical 0.1.1 release used an exact `=1.5.0` pin under ADR-RUN-0017; Phase D replaces the pin with the tested floor `>=1.5.0` (REQ-RUN-0002, ANC-037). Under the 2026-10-31 amendment, `sc-cron` is a temporary dependent of `sc-command` for the observability port, with default features off. `allowed_dependents` in `boundaries/sc-command/sc-command.toml` lists `sc-runtime`, `sc-cron` (ADR-RUN-0017) and `sc-cron-commands`. The shared storage command helpers of ADR-RUN-0021 consume `sc-storage`. OPEN for owner decision: which feature selection carries the `sc-storage` edge, and which selection, if any, provides the envelope without a workspace dependency. REQ-RUN-0301 requires the generated service to depend on `sc-command` with default features; this record does not decide how the service obtains the shared storage helpers. The third-party crates `sc-command` may depend on are listed in [NFR-CMD-0001](requirements.md). sc-lint 0.8.0 rejects a forbidden edge whose target is not a workspace crate, so `forbidden_edges` in `boundaries/sc-command/sc-command.toml` lists `sc-config`, `sc-transport`, `sc-runtime` and `sc-runtime-db-sqlite` but not `sc-observability` or `sc-observability-otlp`; `cargo tree` checks those two crates and the feature-gated axum and rmcp edges ([REQ-RUN-0005](../requirements.md)). Sprint a-5 deleted `crates/sc-command/lint-edge/` when `crates/sc-runtime` landed.

**MUST statements:**

- The version requirement of the `sc-observability-types` entry under `[dependencies]` in `crates/sc-command/Cargo.toml` MUST be the string `">=1.5.0"`.

**MUST NOT statements:**

- A manifest dependency of `sc-command` MUST NOT name a workspace crate other than `sc-storage`.
- The production dependency tree MUST NOT contain `sc-observability`.
- The production dependency tree MUST NOT contain `sc-observability-otlp`.
- The `sc-observability-types` version requirement in `crates/sc-command/Cargo.toml` MUST NOT declare an upper bound.

### Rationale

Each library crate in this repository is an independent deliverable,
published to crates.io and usable without the others
([REQ-RUN-0002](../requirements.md)); an edge to a sibling crate would force
every user of the envelope to take that sibling too. The original standalone configuration, transport and envelope capabilities have no edges among them; the shared storage command path adds only its `sc-storage` edge
([ADR-RUN-0003](../architecture.md)). Observability is initialised directly
by each project and is not wrapped by these crates; the single permitted
observability dependency is the types-only crate, because the envelope's
error code and remediation types are defined there
([ADR-RUN-0004](../architecture.md)).

### Success Criteria

1. Inspection of `crates/sc-command/Cargo.toml`: none of `sc-config`,
   `sc-transport`, `sc-runtime`, `sc-observability`,
   `sc-observability-otlp` appears in `[dependencies]`,
   `[dev-dependencies]` or `[build-dependencies]`;
   `sc-observability-types` appears in `[dependencies]` with requirement
   `">=1.5.0"` and no exact `=` pin, caret, tilde or explicit upper bound.
2. `cargo tree -p sc-command -e normal --prefix none` (default features)
   prints no line beginning with `sc-config `, `sc-transport `,
   `sc-runtime `, `sc-observability ` or `sc-observability-otlp ` (each name
   followed by a space, so that `sc-observability-types` and the repository
   path do not match), and prints a line beginning with
   `sc-observability-types `.
3. `cargo tree -p sc-command -e normal --prefix none --features server`
   gives the same result as criterion 2.
4. This sprint's boundary check is `sc-lint lint sc-boundary --root .` on
   `boundaries/sc-command/sc-command.toml`. Root `just lint` checks all public-package boundary manifests under REQ-RUN-0005.

---

## NFR-CMD-0003: `sc-command` never panics; `into_mcp` error exception

**Status:** Active  
**Version:** 1.0.0  
**Created:** 2026-09-19  
**Last Updated:** 2026-11-12
**Owner:** randlee  

### Requirement Statement

Definitions: HTTP means Hypertext Transfer Protocol. JSON means JavaScript Object Notation. MCP means Model Context Protocol, implemented by the crate rmcp. API means application programming interface. `Envelope<T>` is the four-key response envelope with JSON keys `version`, `ok`, `data` and `error` defined by REQ-CMD-0001. ADR means architecture decision record. ENVELOPE.SERIALIZATION is the stable error-code string identifying envelope serialization failure. T is the value type being serialized by the conversion. `into_response` converts an `Envelope<T>` to an HTTP response; `into_mcp` converts a result to an MCP tool result. `Serialize` is the serde serialization trait. `McpError` is rmcp's `ErrorData`. An envelope serialization error is an error returned while `into_response` or `into_mcp` serializes an `Envelope<T>` to JSON, including an error returned by the `Serialize` implementation of `T`. The error code of an HTTP response is the `error.code` field of the JSON envelope in the response body. The `noop` expect is the single `expect` on `ServiceName::new("noop")` in `crates/sc-command/src/observability.rs` that ADR-RUN-0017 allows. A banned construct is any `unwrap`, `panic!`, `unreachable!`, `todo!`, `unimplemented!`, `[]` indexing on a slice, `Vec` or map, or `expect` other than the `noop` expect. Non-test code is code in `crates/sc-command/src` outside `#[cfg(test)]` modules.

2026-10-31 explicit exception: ADR-RUN-0017 allows only the fixed
`ServiceName::new("noop")` invariant under exact inspected types 1.5.0,
never caller/environment data. `CanonicalLog::emit` returns canonical foreign
EventError. The rest of this panic/error rule remains in force; moving an
existing expect into command without this amendment would violate it.

No public function, method or trait implementation of `sc-command` may panic
on any caller input. This covers constructors of `Envelope<T>` and
`OpError`, `From<Result<T, OpError>> for Envelope<T>`, the serde
implementations, `IntoResponse::into_response` for `Envelope<T>`, and
`IntoMcp::into_mcp`.

There is no allowance for a commented exception. Test code (`#[cfg(test)]` modules and `crates/sc-command/tests/`) is exempt. This item adds only the `sc-command`-specific points below.

The same repository rule requires a fallible public function to return
`Result<T, E>` where `E` is a typed error enum owned by the crate
(`ConfigError`, `TransportError` and `RuntimeError` in the other three
crates). It applies to errors a crate itself originates, and it is decided
in this document for `sc-command` (the design states it for `sc-config`
only). Two `sc-command` signatures are the recorded exception, because a
third-party crate dictates them: `sc_command::IntoMcp::into_mcp` returns
`Result<CallToolResult, McpError>` with rmcp's `ErrorData` (imported as
`McpError`), which is what an rmcp `#[tool]` function must return; and
axum's `IntoResponse::into_response` is infallible. `into_mcp` is the only
fallible public function this document requires of `sc-command`. v0.1 does
not add an `sc-command` error enum. No function required by
[REQ-CMD-0001](requirements.md) through [REQ-CMD-0006](requirements.md)
originates an error other than through serde or rmcp.

The repository-wide rule is owned by [NFR-RUN-0009](../requirements.md) and applies to `crates/sc-command/src` unchanged. Its banned constructs are `unwrap`, `expect`, `panic!`, `unreachable!`, `todo!`, `unimplemented!`, and panicking `[]` indexing.

**MUST statements:**

- When `into_response` meets an envelope serialization error, `into_response` MUST return a response with HTTP status 500.
- When `into_response` meets an envelope serialization error, the error code of the response returned by `into_response` MUST be `ENVELOPE.SERIALIZATION`.
- When `into_mcp` meets an envelope serialization error, `into_mcp` MUST return `Err(McpError)`.

**MUST NOT statements:**

- Non-test code MUST NOT contain a banned construct.

### Rationale

These conversions run inside the request handlers of a long-lived daemon. A
panic there aborts the request, or the whole process under some panic
settings, and the client receives no envelope it can parse. Library errors
in this repository are typed values, not panics
([ADR-RUN-0006](../architecture.md)). The exception for `into_mcp` and
`into_response` is recorded here so that a reviewer applying the typed-enum
rule does not reject the two signatures that rmcp and axum require.

### Success Criteria

1. `grep -rnE '\.unwrap\(|\.expect\(|panic!|unreachable!|todo!|unimplemented!' crates/sc-command/src`
   returns no match outside `#[cfg(test)]` modules, and no match is excused
   by a comment.
2. Source review of `crates/sc-command/src` finds no `[...]` indexing on a
   slice, `Vec` or map in non-test code.
3. A test compiled with `--features server` defines a type whose
   `Serialize` implementation always returns an error, wraps it in a
   success envelope, calls `.into_response()`, and completes without
   panicking.
4. A test calls `.into_mcp()` on `Ok(value)` of that same type and
   completes without panicking.
5. Inspection of the public API of `crates/sc-command/src`: apart from the
   `serde::Serialize` and `serde::Deserialize` implementations, whose error
   types serde dictates, the only public function returning a `Result` is
   `IntoMcp::into_mcp`, and its error type is rmcp's `McpError`.

### Related Documents

**Architecture Decisions:**

- ADR-RUN-0017 — 2026-10-31 explicit exception
