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

### Requirement Statement

The crate `sc-command` (directory `crates/sc-command`) MUST export a public
generic type `sc_command::Envelope<T>` from its crate root, available with
default features. It is the sc-ai-cli response envelope: the one JSON shape
every REST response, MCP tool result and CLI `--json` output uses.

`Envelope<T>` MUST implement `serde::Serialize` for any `T: Serialize` and
`serde::Deserialize` for any `T: DeserializeOwned`.

`Envelope<T>` MUST serialise to a JSON object with exactly these four keys
and no others:

| Key | JSON value in the success case | JSON value in the failure case |
|---|---|---|
| `version` | envelope contract version | envelope contract version |
| `ok` | `true` | `false` |
| `data` | the serialised `T` | `null` |
| `error` | `null` | the serialised `OpError` |

`error` is an `OpError`, the typed error with fields `kind`, `code`,
`message`, `details` and `suggested_action`
([REQ-CMD-0002](requirements.md)).

In the success case the `error` key MUST be present with the value `null`. In
the failure case the `data` key MUST be present with the value `null`. Neither
key is ever omitted. (Present-as-`null`, as opposed to omitted, is decided in
this document; the design gives only the key list.)

Key order in the serialised object is not significant. Tests MUST compare
parsed JSON values, not byte strings.

Deserialisation MUST discriminate on the value of `ok`. With `ok: true` the
`data` value, including a JSON `null`, MUST be deserialised as `T`, and the
result is the success case. With `ok: false` the `error` value MUST be
deserialised as `OpError`, and the result is the failure case. A `data` of
`null` therefore never by itself means "failure": for `T = ()` and for
`T = Option<U>` a success envelope legitimately carries `data: null`.
(Decided in this document; it follows from the round-trip obligation below,
which could not otherwise hold for those `T`.)

For any `T: Serialize + DeserializeOwned + PartialEq`, serialising an
`Envelope<T>` to JSON and deserialising it back MUST yield a value equal to
the original, in both the success case and the failure case. This includes
`T = ()` and `T = Option<U>`, for both `Some` and `None`.

`Envelope<T>` MUST implement `Debug` where `T: Debug`, `Clone` where
`T: Clone`, and `PartialEq` where `T: PartialEq`. (Decided in this document;
the round-trip tests below need `PartialEq` and `Debug`. The same traits are
required of `OpError` by [REQ-CMD-0002](requirements.md).)

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
`ENVELOPE.DISCRIMINANT`.

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

---

## REQ-CMD-0002: Typed error `OpError` and its five fields

**Status:** Active  

### Requirement Statement

The crate `sc-command` MUST export a public type `sc_command::OpError` from
its crate root, available with default features. `OpError` is the error half
of the response envelope `Envelope<T>` ([REQ-CMD-0001](requirements.md)) and
the error type service functions return in `Result<T, OpError>`. A generated
project's `service` crate therefore uses `sc-command` with default features;
the generated workspace's crate graph, including how `service` and `daemon`
obtain `sc-command`, is owned by [REQ-RUN-0301](../requirements.md).

`OpError` MUST carry exactly these five wire keys. It MUST serialise to a JSON
object whose keys are drawn from exactly these five names and no others. The
keys `kind`, `code`, `message`, `details`, and `suggested_action` are always
present. `details` and `suggested_action` are JSON `null` when the Rust field
is empty. This item is the owner of which `OpError` keys are present on the
wire; other items refer to "the keys defined by REQ-CMD-0002".

| Field | Meaning | Type |
|---|---|---|
| `kind` | coarse error category; the HTTP status is derived from it | `sc_command::ErrorKind`, a public enum exported from the crate root (the type name `ErrorKind` is decided in this document) |
| `code` | stable machine-readable code that automation branches on, for example `DAEMON.NOT_RUNNING` | `sc_observability_types::ErrorCode` |
| `message` | human-readable summary | `String` |
| `details` | structured machine-readable context | `Option<serde_json::Value>`; JSON `null` when absent |
| `suggested_action` | wire name for remediation | Rust field `remediation: Option<sc_observability_types::Remediation>`; on the wire a JSON string, or JSON `null` when absent |

`OpError` and `ErrorKind` MUST implement `serde::Serialize` and
`serde::Deserialize`, because the CLI parses errors it receives from the
daemon. `OpError` and `ErrorKind` MUST also implement `Debug`, `Clone` and
`PartialEq`. (The three extra traits are decided in this document; the
round-trip tests need them, and `sc_observability_types::ErrorCode` already
implements all three.)

`code` MUST be the type `ErrorCode` from the published crate
`sc-observability-types` 1.5.0, the pin sprint a-1 recorded. `sc-command` MUST NOT define its own
error-code type. `ErrorCode` is a newtype over a string and serialises as a
JSON string, so `code` appears on the wire as, for example,
`"DAEMON.NOT_RUNNING"`.

When a suggestion is supplied, `suggested_action` MUST serialise as a JSON
string equal to the text supplied: an `OpError` built with the suggestion
`run my-app daemon start` serialises with
`"suggested_action": "run my-app daemon start"`. It MUST NOT serialise as a
JSON object. (The design's only example shows a plain string.)

`sc-command` MUST NOT define a parallel remediation type. If `OpError`
carries, or is built from, structured remediation, that structure MUST be
`sc_observability_types::Remediation` or
`sc_observability_types::RecoverableSteps`.

Whatever type is chosen for `details` MUST be able to hold arbitrary JSON and
return it unchanged when re-serialised
([REQ-CMD-0006](requirements.md) depends on this).

`ErrorKind` variants and their wire spellings are `invalid_input`,
`not_found`, `conflict`, `unavailable`, and `internal`. HTTP status is 400,
404, 409, 503, and 500 in that order. `details` is
`Option<serde_json::Value>` and is JSON `null` when absent. The Rust field
behind wire `suggested_action` is
`Option<sc_observability_types::Remediation>`. `None` serialises as JSON
`null`. A present value serialises as a JSON string: the first recovery step,
or the not-recoverable justification. The tagged `Remediation` object is not
the wire form. `OpError` does not convert to or from
`sc_observability_types::Diagnostic`. `ErrorKind` is defined in `sc-command`.
`sc-observability-types` 1.5.0 has no `ErrorKind`, `OpError`, or
`suggested_action`.

`OpError::from_transport_strings` maps the plain strings `code`, `endpoint`,
and `cause` into this type without a dependency on `sc-transport`. The code
`DAEMON.NOT_RUNNING` selects `ErrorKind::Unavailable` and the suggestion
`run <app> daemon start`. Any other code selects `ErrorKind::Internal` and
no suggestion.

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
   is listed under `[dependencies]` and is not `optional`.
6. Inspection of `crates/sc-command/src`: no type defined there duplicates
   `ErrorCode`, `Remediation` or `RecoverableSteps`.
7. A unit test constructs an `OpError` with no details and no suggestion,
   serialises it, and asserts: the keys `kind`, `code` and `message` are
   present; every key of the object is one of the five names; `details` is
   JSON `null`; `suggested_action` is JSON `null`.
8. A compile-time check in a test requires
   `OpError: Debug + Clone + PartialEq` and
   `ErrorKind: Debug + Clone + PartialEq`.

---

## REQ-CMD-0003: `From<Result<T, OpError>>` for `Envelope<T>`

**Status:** Active  

### Requirement Statement

The crate `sc-command` MUST provide, with default features, the
implementation

```rust
impl<T> From<Result<T, OpError>> for Envelope<T>
```

so that a service result converts with `.into()`. `Envelope<T>` is the
response envelope with JSON keys `version`, `ok`, `data`, `error`
([REQ-CMD-0001](requirements.md)); `OpError` is the typed error
([REQ-CMD-0002](requirements.md)).

The conversion MUST behave as follows:

| Input | Resulting envelope |
|---|---|
| `Ok(value)` | `ok` is `true`, `data` is `value`, `error` is null |
| `Err(op_error)` | `ok` is `false`, `data` is null, `error` is `op_error` |

In both cases `version` MUST be the crate's single envelope contract version
(its type and value are an open item recorded in
[REQ-CMD-0001](requirements.md)); the caller does not supply it.

The implementation MUST NOT require any trait bound on `T`. The conversion
MUST NOT alter `value` or `op_error`.

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

### Requirement Statement

When the cargo feature `server` of `sc-command` is enabled, the crate MUST
provide

```rust
impl<T: serde::Serialize> axum::response::IntoResponse for Envelope<T>
```

so that an axum 0.8 handler can declare `Envelope<T>` as its return type.
`Envelope<T>` is the response envelope with JSON keys `version`, `ok`,
`data`, `error` ([REQ-CMD-0001](requirements.md)).

The response body MUST be the envelope serialised as JSON, identical as a
JSON value to `serde_json::to_value(&envelope)`. The response MUST carry the
header `Content-Type: application/json`.

The HTTP status MUST be chosen as follows. This rule is decided in this
document; the design specifies that the conversion exists and does not
specify the mapping.

| Envelope | HTTP status |
|---|---|
| success case (`ok` is `true`) | `200 OK` |
| failure, `ErrorKind::InvalidInput` | `400 Bad Request` (caller error) |
| failure, `ErrorKind::NotFound` | `404 Not Found` (caller error) |
| failure, `ErrorKind::Conflict` | `409 Conflict` (caller error) |
| failure, `ErrorKind::Unavailable` | `503 Service Unavailable` (internal error) |
| failure, `ErrorKind::Internal` | `500 Internal Server Error` (internal error) |

`error.kind` is the `ErrorKind` field of `OpError`
([REQ-CMD-0002](requirements.md)). The status MUST be a pure function of
`kind`: two errors with the same `kind` get the same status whatever their
`code`. The first three kinds are caller errors. `Unavailable` and
`Internal` are internal errors.

When serialising `T` fails, `into_response` MUST return HTTP 500 with a JSON
envelope whose `error.code` is `ENVELOPE.SERIALIZATION`. It MUST NOT panic
([NFR-CMD-0003](requirements.md)).

When the `server` feature is disabled this implementation MUST NOT exist and
the crate MUST NOT reference axum.

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

### Requirement Statement

When the cargo feature `server` of `sc-command` is enabled, the crate MUST
export a public extension trait `sc_command::IntoMcp` with one method,
implemented for `Result<T, OpError>` where `T: serde::Serialize`:

```rust
pub trait IntoMcp {
    fn into_mcp(self) -> Result<CallToolResult, McpError>;
}
```

`CallToolResult` is `rmcp::model::CallToolResult` from rmcp 3.7. `McpError`
is `rmcp::ErrorData`, which tool handlers import under that name and which
`sc-command` re-exports as `sc_command::McpError`. The error flag is
`is_error`. A tool declared `-> Result<CallToolResult, McpError>` can return
`into_mcp()` directly. The trait name `IntoMcp` is decided in this document;
the design fixes only the method name `into_mcp()` and its use on a
`Result<T, OpError>`.

`IntoMcp` is sealed by a private supertrait and also requires `Sized`. The
only implementation is `Result<T, OpError>`. `into_mcp` takes `self` by
value. Dispatch is static. `dyn IntoMcp` is not a supported API. A downstream
crate can call the method and cannot implement the trait.

The error type of this signature is rmcp's, not an error enum of
`sc-command`, because rmcp dictates the signature of a `#[tool]` function.
This is the recorded exception to the repository rule that a fallible public
function returns the crate's own typed error enum
([NFR-RUN-0009](../requirements.md), restated for this crate in
[NFR-CMD-0003](requirements.md)).

`into_mcp()` MUST first form the same envelope that
`Envelope::<T>::from(result)` produces (JSON keys `version`, `ok`, `data`,
`error`; [REQ-CMD-0001](requirements.md) and
[REQ-CMD-0003](requirements.md)) and MUST place that envelope, as JSON, in
the content of the returned `CallToolResult`.

| Input | Returned value |
|---|---|
| `Ok(value)` | `Ok(CallToolResult)` whose content is the success envelope and whose `is_error` is not `Some(true)` |
| `Err(op_error)` | `Ok(CallToolResult)` whose content is the failure envelope and whose `is_error` is `Some(true)` |

Setting `is_error` to `Some(true)` exactly when the envelope's `ok` is
`false` is decided in this document. An `OpError` from the service MUST NOT
be turned into `Err(McpError)`: an MCP protocol error would hide the
envelope from the client.

The envelope JSON delivered through MCP MUST equal, as a JSON value, the
body the axum conversion produces for the same `Result<T, OpError>`
([REQ-CMD-0004](requirements.md)).

The envelope is carried as both one text content item holding the JSON
string and as `structured_content`. Success uses `CallToolResult::structured`,
so `is_error` is `Some(false)`. Failure uses
`CallToolResult::structured_error`, so `is_error` is `Some(true)`.

`into_mcp()` returns `Err(McpError)` only when serialising the envelope
fails. That error is `ErrorData::internal_error`, and its data carries the
code `ENVELOPE.SERIALIZATION`. An `OpError` is not this `Err`. The function
MUST NOT panic ([NFR-CMD-0003](requirements.md)).

When the `server` feature is disabled the trait MUST NOT exist and the crate
MUST NOT reference rmcp.

### Rationale

An rmcp tool body in a generated project is one line,
`service::create_widget(&self.stores, input).await.into_mcp()`, mirroring the
one-line axum handler. An MCP client then sees exactly the envelope a REST
client sees, which the product requires of every surface
([REQ-RUN-0203](../requirements.md)), and MCP hosts that look only at
`is_error` still learn that the call failed.

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

### Requirement Statement

Deserialising a JSON response envelope into `sc_command::Envelope<T>` and
serialising that value again MUST produce a JSON value equal to the input.
Equality means `serde_json::Value` equality: same keys, same values, same
`null`s; key order and whitespace are not compared. The envelope is the
object with keys `version`, `ok`, `data`, `error`
([REQ-CMD-0001](requirements.md)).

This MUST hold for `T = serde_json::Value` with arbitrary JSON inside `data`,
including fields the CLI has no struct for.

This MUST hold for arbitrary JSON inside `error.details`, whatever `T` is.
`details` is the structured-context field of `OpError`
([REQ-CMD-0002](requirements.md)); its type MUST therefore preserve
arbitrary JSON.

A key whose value is `null` in the input (`error` in the success case,
`data` in the failure case) MUST be present with the value `null` in the
output; it MUST NOT be dropped.

The `version` value in the output MUST be the value read from the input, not
the version constant of the `sc-command` build doing the printing.

This requirement does not cover a typed `T` that ignores unknown fields:
deserialising into `Envelope<Widget>` legitimately drops fields `Widget`
does not declare. A CLI that must print the response unchanged deserialises
into `Envelope<serde_json::Value>`.

### Rationale

A CLI's `--json` flag prints the envelope it received from the daemon
unchanged. If the round-trip through `Envelope` lost or rewrote anything, the
CLI's output would differ from the API's for the same call, and an agent
could not treat the two as interchangeable. The risk is greatest when the
daemon is newer than the CLI and sends fields the CLI does not know.

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

### Requirement Statement

In `crates/sc-command/Cargo.toml` the dependencies `axum` and `rmcp` MUST be
declared `optional = true`. They MUST be enabled only by a cargo feature
named `server`. The `server` feature MUST NOT be in the `default` feature
list.

With default features the crate MUST compile and MUST provide `Envelope<T>`,
`OpError`, `ErrorKind` and `From<Result<T, OpError>> for Envelope<T>`. With
default features the crate's direct non-dev dependencies MUST be `serde`,
`serde_json`, and `sc-observability-types` 1.5.0. `serde_json` is required
because `details` holds arbitrary JSON. With `--features server` the direct
non-dev dependencies MUST be those plus `axum` 0.8.9 and `rmcp` `~3.7`, and
nothing else.

This item is the owner of the list of third-party crates `sc-command` may
depend on. The repository-level boundary table
([REQ-RUN-0005](../requirements.md)) refers to this item for that list and
owns only the workspace-internal and forbidden edges, which are restated for
this crate in [NFR-CMD-0002](requirements.md).

With `--features server` the crate MUST additionally provide
`impl IntoResponse for Envelope<T>` and the `IntoMcp::into_mcp` conversion.

Every `use`, type or function that names axum or rmcp MUST be inside code
gated by `#[cfg(feature = "server")]`.

Dev-dependencies are outside this rule: tests of the `server` feature may
use axum and rmcp.

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
   features) prints, below the `sc-command` root line, exactly one line
   beginning with `serde `, one beginning with `serde_json `, and one
   beginning with `sc-observability-types `, and no other line. The
   `sc-observability-types ` line is version 1.5.0.
8. The same command with `--features server` prints the lines of criterion
   7 plus exactly one line beginning with `axum ` and one beginning with
   `rmcp `, and no other line.

---

## NFR-CMD-0002: `sc-command` dependency boundary

**Status:** Active  

### Requirement Statement

`sc-command` MUST NOT depend on any other crate of this workspace:
`sc-config`, `sc-transport` or `sc-runtime`.

`sc-command` MUST depend on the published crate `sc-observability-types`
1.5.0.

`sc-command` MUST NOT depend, directly or transitively, on the crate
`sc-observability` (the logging runtime) or on its OTel export crate
`sc-observability-otlp`.

These rules apply to every feature combination, including `server`.

`cargo tree` enforces the logging crates and the sibling crates.
`sc-observability-types` is an allowed dependency and is not a forbidden
edge. The boundary manifest's `forbidden_edges` names the workspace-member
anchors `sc-observability` and `sc-observability-otlp` only. sc-lint 0.8.0
rejects a forbidden edge whose target is not a workspace member, so those
two names are `publish = false` packages under
`crates/sc-command/lint-edge/`. `sc-command` does not depend on them.
`sc-config` and `sc-transport` are not workspace members in this sprint, so
they are not `forbidden_edges` entries; Cargo.toml inspection and
`cargo tree` enforce them. `allowed_dependents` is `sc-runtime` only. The
`lint-edge/sc-runtime` package exists so that dependent is not stale, and
sprint a-5 deletes it when `crates/sc-runtime` lands. The third-party crates
`sc-command` may depend on are listed in [NFR-CMD-0001](requirements.md).
`sc-lint lint sc-boundary` checks inter-crate edges and the public facade
list only; it has no cargo-feature key, so the feature-conditional
dependencies on axum and rmcp are checked with `cargo tree`.

### Rationale

Each library crate in this repository is an independent deliverable,
published to crates.io and usable without the others
([REQ-RUN-0002](../requirements.md)); an edge to a sibling crate would force
every user of the envelope to take that sibling too. The three standalone
crates have no edges among them and only `sc-runtime` knows the others
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
   `sc-observability-types` appears in `[dependencies]`.
2. `cargo tree -p sc-command -e normal --prefix none` (default features)
   prints no line beginning with `sc-config `, `sc-transport `,
   `sc-runtime `, `sc-observability ` or `sc-observability-otlp ` (each name
   followed by a space, so that `sc-observability-types` and the repository
   path do not match), and prints a line beginning with
   `sc-observability-types `.
3. `cargo tree -p sc-command -e normal --prefix none --features server`
   gives the same result as criterion 2.
4. This sprint's boundary check is `sc-lint lint sc-boundary --root .` on
   `boundaries/sc-command/sc-command.toml`. Root `just lint` across all four
   manifests stays with sprint a-5.

---

## NFR-CMD-0003: `sc-command` never panics; `into_mcp` error exception

**Status:** Active  

### Requirement Statement

No public function, method or trait implementation of `sc-command` may panic
on any caller input. This covers constructors of `Envelope<T>` and
`OpError`, `From<Result<T, OpError>> for Envelope<T>`, the serde
implementations, `IntoResponse::into_response` for `Envelope<T>`, and
`IntoMcp::into_mcp`.

The repository-wide rule is owned by [NFR-RUN-0009](../requirements.md) and
applies to `crates/sc-command/src` unchanged: non-test code MUST NOT contain
`unwrap`, `expect`, `panic!`, `unreachable!`, `todo!`, `unimplemented!`, or
panicking `[]` indexing, reachable from a public function. There is no
allowance for a commented exception. Test code (`#[cfg(test)]` modules and
`crates/sc-command/tests/`) is exempt. This item adds only the
`sc-command`-specific points below.

A failure inside a conversion, in particular a `T` whose `Serialize`
implementation returns an error, MUST surface as a returned value: HTTP 500
with code `ENVELOPE.SERIALIZATION` from `into_response`, and
`Err(McpError)` from `into_mcp`.

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
