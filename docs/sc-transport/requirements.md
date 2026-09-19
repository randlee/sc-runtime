# `sc-transport` Requirements

Crate-level requirements. Repo-level requirements: [`../requirements.md`](../requirements.md).
Source: [`../sc-runtime-design.md`](../sc-runtime-design.md). Every id is binding and is never reused.

## Purpose

The listener and client connector for any local daemon and CLI pair: HTTP over
a Unix domain socket or TCP.

## Requirements

- `REQ-TRANSPORT-001` Endpoint resolution yields a UDS path or a TCP address
  from, in order: an explicit `--endpoint` value, the `SC_ENDPOINT`
  environment variable, the configured endpoint, then the platform default.
  The default is UDS at `<instance-root>/daemon.sock` on macOS and Linux and
  TCP on `127.0.0.1` on Windows.
- `REQ-TRANSPORT-002` The instance root is resolved per app name and can be
  supplied explicitly so tests use a tempdir.
- `REQ-TRANSPORT-003` With the `server` feature, the crate binds a listener
  for a resolved endpoint that is served by `axum::serve` with no change to
  routes. A daemon may bind a UDS and a TCP listener at once.
- `REQ-TRANSPORT-004` A UDS socket file is created with access restricted to
  the owning user, and a stale socket file left by a dead daemon is replaced.
- `REQ-TRANSPORT-005` `Client::connect(app, endpoint)` builds a
  `reqwest::Client` using `ClientBuilder::unix_socket` for UDS or a base URL
  for TCP, and offers typed `get` and `post` that serialise a request body and
  deserialise any `T: DeserializeOwned` response.
- `REQ-TRANSPORT-006` A connection failure maps to the typed
  `DaemonNotRunning` error carrying code `DAEMON.NOT_RUNNING` and
  `suggested_action: "run <app> daemon start"`. The client never starts the
  daemon.

## Non-functional requirements

- `NFR-TRANSPORT-001` axum is a dependency only behind the `server` feature,
  which is off by default. The default build depends on tokio and reqwest.
- `NFR-TRANSPORT-002` No `hyperlocal`, custom connector, or hand-written HTTP
  is used.
- `NFR-TRANSPORT-003` The crate depends on no other crate in this workspace.
- `NFR-TRANSPORT-004` No public function panics; errors are a typed
  `TransportError` enum.
