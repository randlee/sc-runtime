# `sc-transport` Architecture

Crate-level architecture and ADRs. Repo-level: [`../architecture.md`](../architecture.md).
Requirements: [`requirements.md`](requirements.md). Boundary manifest: `boundaries/sc-transport/`.
All ADRs are `accepted` unless stated; an ADR changes only through a later ADR that names it.

## Role

A leaf crate with two halves: endpoint resolution and the client (always
built), and listener binding (`server` feature). It moves HTTP over a socket
and reports transport facts; it knows nothing about envelopes or operations.

## Public surface

`Endpoint`, `resolve_endpoint`, `instance_root`, `Client`, `TransportError`;
behind `server`: `bind`, `Listener`.

## ADR

### ADR-TRANSPORT-001 reqwest `unix_socket`, no custom connector

The client is a plain `reqwest::Client` (reqwest 0.13
`ClientBuilder::unix_socket`). No extra crate and no hand-written connector.

### ADR-TRANSPORT-002 One resolution order, platform defaults

`--endpoint`, then `SC_ENDPOINT`, then config, then the platform default (UDS
on Unix, TCP loopback on Windows). Daemon and CLI call the same resolver so
they cannot disagree.

### ADR-TRANSPORT-003 Response types are the caller's

`Client` is generic over `T: DeserializeOwned`; it does not know `Envelope`
or `OpError`. The mapping from `TransportError::DaemonNotRunning` to an
`OpError` happens in the project's CLI. Applies repo ADR-004.

### ADR-TRANSPORT-004 `server` feature gates axum

Listener binding sits behind `server`, off by default, so a CLI never links
axum. Applies repo ADR-004.
