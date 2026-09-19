# `sc-transport` Requirements

**ID Range:** REQ-TRN-0001 through REQ-TRN-0006; NFR-TRN-0001 through NFR-TRN-0004  
**Status:** Draft  
**Created:** 2026-09-19  
**Last Updated:** 2026-09-19  
**Owner:** sc-runtime maintainers  

---

Crate-level requirements for `sc-transport`. Repo-level requirements are in
[`../requirements.md`](../requirements.md); this crate's architecture and ADRs
are in [`architecture.md`](architecture.md). Extracted from
[`../sc-runtime-design.md`](../sc-runtime-design.md) ("decision N" refers to its
decision log). Entries follow the shared SC requirement and ADR templates:
the obligation, then **Why** and **Verified by**. Every id is binding and is
never reused.

## Purpose

`sc-transport` is the connection between any local daemon and its CLI:
HTTP over a Unix domain socket or TCP. The daemon uses it to bind a listener
for `axum::serve`; the CLI uses it to find the daemon, connect, and get a clear
typed error when the daemon is not running. It is useful to any daemon and CLI
pair, not only sc-runtime ones.

## ID Ranges

| Range | Area | Scope |
|---|---|---|
| REQ-TRN-0001 through REQ-TRN-0006 | Requirements | - |
| NFR-TRN-0001 through NFR-TRN-0004 | Non-functional requirements | - |

---

## REQ-TRN-0001: One endpoint resolution order

**Status:** Active  

### Requirement Statement

Endpoint resolution yields either a UDS path or a TCP address from, in order:
an explicit `--endpoint` value, the `SC_ENDPOINT` environment variable, the
endpoint in config, then the platform default. The default is UDS at
`<instance-root>/daemon.sock` on macOS and Linux, and TCP on `127.0.0.1` on
Windows with the port from config.

### Rationale

Daemon and CLI must compute the same endpoint or they never meet;
one resolver used by both makes disagreement impossible. UDS with file
permissions is the default because it needs no port and no token (the same
approach as dockerd); Windows gets TCP because reqwest's `unix_socket` is
Unix only. Cross-host and frontend use is TCP with a port from config, ports
being registered centrally in the synaptic-canvas repo.

### Success Criteria

Unit tests for each precedence level and each platform
default.

---

## REQ-TRN-0002: Instance root

**Status:** Active  

### Requirement Statement

The instance root is resolved per app name to a per-user directory, and can
be supplied explicitly.

### Rationale

The socket and the daemon lock live there, so it identifies "this
app's daemon for this user". Tests supply a tempdir so they never touch a
developer's real daemon ([NFR-RUN-0005](../requirements.md)).

### Success Criteria

Tests for default resolution and explicit override.

---

## REQ-TRN-0003: Listener binding (`server` feature)

**Status:** Active  

### Requirement Statement

With the `server` feature, the crate binds a listener for a resolved endpoint
that `axum::serve` accepts with no change to routes. A daemon may bind a UDS
and a TCP listener at once.

### Rationale

`axum::serve` accepts a `TcpListener` or a `UnixListener`, so the
listener is a configuration choice and routes never know. Binding both serves
the case where a browser frontend needs TCP while the CLI stays on the
socket.

### Success Criteria

Tests serving one router over UDS, over TCP, and over both.

---

## REQ-TRN-0004: Socket file hygiene

**Status:** Active  

### Requirement Statement

A UDS socket file is created with access restricted to the owning user. A
stale socket file left by a daemon that died is replaced on bind.

### Rationale

File permissions are the only access control on the default
transport. A leftover socket file would otherwise make every restart after a
crash fail with "address in use"; replacing it is safe because the
`daemon.lock` singleton (owned by `sc-runtime`) has already proven no other
daemon is alive.

### Success Criteria

A test asserting the file mode; a test binding over a stale
file.

---

## REQ-TRN-0005: Typed HTTP client

**Status:** Active  

### Requirement Statement

`Client::connect(app, endpoint)` builds a `reqwest::Client`, using
`ClientBuilder::unix_socket(path)` for UDS or a base URL for TCP, and offers
typed `get` and `post` that serialise a request body and deserialise any
`T: DeserializeOwned` response.

### Rationale

This is the whole client a CLI needs:
`client.post("/ops/widget.create", &CreateWidget { name })`. Being generic
over `T` keeps the crate independent of the envelope ([ADR-TRN-0003](architecture.md)).

### Success Criteria

Tests round-tripping a typed request and response over both
transports.

---

## REQ-TRN-0006: `DAEMON.NOT_RUNNING`

**Status:** Active  

### Requirement Statement

A connection failure (no socket file, connection refused) maps to the typed
`TransportError::DaemonNotRunning`, carrying the code `DAEMON.NOT_RUNNING`
and `suggested_action: "run <app> daemon start"`. The client never starts the
daemon. Other failures (timeout, HTTP error status, undecodable body) are
distinct variants.

### Rationale

The daemon being down is the most common CLI failure and must be
told apart from every other failure so an agent or a person knows the fix is
to start it. Report-only is the design's default while auto-start is
undecided (decision 24).

### Success Criteria

Tests connecting to an absent socket and a closed port.

---

## NFR-TRN-0001: axum only behind `server`

**Status:** Active  

### Requirement Statement

Axum is a dependency only behind the `server` feature, which is off by
default. The default build depends on tokio and reqwest.

### Rationale

The CLI uses this crate and must not link Axum ([NFR-RUN-0001](../requirements.md)).

### Success Criteria

`cargo tree -p sc-transport` with default features shows no
axum; the crate builds and tests with and without the feature.

---

## NFR-TRN-0002: No custom connector

**Status:** Active  

### Requirement Statement

No `hyperlocal`, custom hyper connector, or hand-written HTTP is used.

### Rationale

Reqwest 0.13 provides `ClientBuilder::unix_socket`; the draft's
assumption that UDS needs an extra crate was a fact correction in the design.

### Success Criteria

`Cargo.toml`.

---

## NFR-TRN-0003: Standalone

**Status:** Active  

### Requirement Statement

The crate depends on no other crate in this workspace.

### Rationale

[REQ-RUN-0002](../requirements.md) and [ADR-RUN-0003](../architecture.md).

### Success Criteria

The boundary manifest.

---

## NFR-TRN-0004: Errors are values

**Status:** Active  

### Requirement Statement

No public function panics; errors are a typed `TransportError` enum.

### Rationale

[ADR-RUN-0006](../architecture.md).

### Success Criteria

Source review; tests for each variant.
