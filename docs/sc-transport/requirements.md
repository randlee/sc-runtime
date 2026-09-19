# `sc-transport` Requirements

Crate-level requirements for `sc-transport`. Repo-level requirements are in
[`../requirements.md`](../requirements.md); this crate's architecture and ADRs
are in [`architecture.md`](architecture.md). Extracted from
[`../sc-runtime-design.md`](../sc-runtime-design.md) ("decision N" refers to its
decision log). Entries follow `.claude/skills/plan-hardening/req-adr-format.md`:
the obligation, then **Why** and **Verified by**. Every id is binding and is
never reused.

## Purpose

`sc-transport` is the connection between any local daemon and its CLI:
HTTP over a Unix domain socket or TCP. The daemon uses it to bind a listener
for `axum::serve`; the CLI uses it to find the daemon, connect, and get a clear
typed error when the daemon is not running. It is useful to any daemon and CLI
pair, not only sc-runtime ones.

## Requirements

- `REQ-TRANSPORT-001` **One endpoint resolution order.**
  Endpoint resolution yields either a UDS path or a TCP address from, in order:
  an explicit `--endpoint` value, the `SC_ENDPOINT` environment variable, the
  endpoint in config, then the platform default. The default is UDS at
  `<instance-root>/daemon.sock` on macOS and Linux, and TCP on `127.0.0.1` on
  Windows with the port from config.
  **Why:** daemon and CLI must compute the same endpoint or they never meet;
  one resolver used by both makes disagreement impossible. UDS with file
  permissions is the default because it needs no port and no token (the same
  approach as dockerd); Windows gets TCP because reqwest's `unix_socket` is
  Unix only. Cross-host and frontend use is TCP with a port from config, ports
  being registered centrally in the synaptic-canvas repo.
  **Verified by:** unit tests for each precedence level and each platform
  default.

- `REQ-TRANSPORT-002` **Instance root.**
  The instance root is resolved per app name to a per-user directory, and can
  be supplied explicitly.
  **Why:** the socket and the daemon lock live there, so it identifies "this
  app's daemon for this user". Tests supply a tempdir so they never touch a
  developer's real daemon (`NFR-REPO-005`).
  **Verified by:** tests for default resolution and explicit override.

- `REQ-TRANSPORT-003` **Listener binding (`server` feature).**
  With the `server` feature, the crate binds a listener for a resolved endpoint
  that `axum::serve` accepts with no change to routes. A daemon may bind a UDS
  and a TCP listener at once.
  **Why:** `axum::serve` accepts a `TcpListener` or a `UnixListener`, so the
  listener is a configuration choice and routes never know. Binding both serves
  the case where a browser frontend needs TCP while the CLI stays on the
  socket.
  **Verified by:** tests serving one router over UDS, over TCP, and over both.

- `REQ-TRANSPORT-004` **Socket file hygiene.**
  A UDS socket file is created with access restricted to the owning user. A
  stale socket file left by a daemon that died is replaced on bind.
  **Why:** file permissions are the only access control on the default
  transport. A leftover socket file would otherwise make every restart after a
  crash fail with "address in use"; replacing it is safe because the
  `daemon.lock` singleton (owned by `sc-runtime`) has already proven no other
  daemon is alive.
  **Verified by:** a test asserting the file mode; a test binding over a stale
  file.

- `REQ-TRANSPORT-005` **Typed HTTP client.**
  `Client::connect(app, endpoint)` builds a `reqwest::Client`, using
  `ClientBuilder::unix_socket(path)` for UDS or a base URL for TCP, and offers
  typed `get` and `post` that serialise a request body and deserialise any
  `T: DeserializeOwned` response.
  **Why:** this is the whole client a CLI needs:
  `client.post("/ops/widget.create", &CreateWidget { name })`. Being generic
  over `T` keeps the crate independent of the envelope (ADR-TRANSPORT-003).
  **Verified by:** tests round-tripping a typed request and response over both
  transports.

- `REQ-TRANSPORT-006` **`DAEMON.NOT_RUNNING`.**
  A connection failure (no socket file, connection refused) maps to the typed
  `TransportError::DaemonNotRunning`, carrying the code `DAEMON.NOT_RUNNING`
  and `suggested_action: "run <app> daemon start"`. The client never starts the
  daemon. Other failures (timeout, HTTP error status, undecodable body) are
  distinct variants.
  **Why:** the daemon being down is the most common CLI failure and must be
  told apart from every other failure so an agent or a person knows the fix is
  to start it. Report-only is the design's default while auto-start is
  undecided (decision 24).
  **Verified by:** tests connecting to an absent socket and a closed port.

## Non-functional requirements

- `NFR-TRANSPORT-001` **axum only behind `server`.**
  axum is a dependency only behind the `server` feature, which is off by
  default. The default build depends on tokio and reqwest.
  **Why:** the CLI uses this crate and must not link Axum (`NFR-REPO-001`).
  **Verified by:** `cargo tree -p sc-transport` with default features shows no
  axum; the crate builds and tests with and without the feature.

- `NFR-TRANSPORT-002` **No custom connector.**
  No `hyperlocal`, custom hyper connector, or hand-written HTTP is used.
  **Why:** reqwest 0.13 provides `ClientBuilder::unix_socket`; the draft's
  assumption that UDS needs an extra crate was a fact correction in the design.
  **Verified by:** `Cargo.toml`.

- `NFR-TRANSPORT-003` **Standalone.**
  The crate depends on no other crate in this workspace.
  **Why:** `REQ-REPO-002` and ADR-004.
  **Verified by:** the boundary manifest.

- `NFR-TRANSPORT-004` **Errors are values.**
  No public function panics; errors are a typed `TransportError` enum.
  **Why:** ADR-011.
  **Verified by:** source review; tests for each variant.
