# `sc-transport` Architecture

**ID Range:** ADR-TRN-0001 through ADR-TRN-0004  
**Status:** Draft  
**Created:** 2026-09-19  
**Last Updated:** 2026-09-19  
**Owner:** sc-runtime maintainers  

---

Crate-level architecture and ADRs for `sc-transport`. Repo-level architecture is in
[`../architecture.md`](../architecture.md); this crate's requirements are in
[`requirements.md`](requirements.md); its boundary manifest is
`boundaries/sc-transport/`. ADRs follow
the shared SC requirement and ADR templates. Every ADR here is Active
and binding; an ADR changes only through a later ADR that names it.

## Role

A leaf crate with two halves. Endpoint resolution and the client are always
built; listener binding is behind the `server` feature. The crate moves HTTP
over a socket and reports transport facts. It knows nothing about envelopes,
operations or stores.

## Public surface

| Item | Feature | Purpose |
|---|---|---|
| `Endpoint` (`Uds(PathBuf)` or `Tcp(SocketAddr)`) | default | a resolved address |
| `resolve_endpoint(...)`, `instance_root(app)` | default | the one resolver both sides call |
| `Client`, `Client::connect`, `get`, `post` | default | typed HTTP client over reqwest |
| `TransportError` | default | typed errors, including `DaemonNotRunning` |
| `bind(&Endpoint)`, `Listener` | `server` | a listener `axum::serve` accepts |

---

## ADR-TRN-0001: reqwest `unix_socket`, no custom connector

**Status:** Active  
**Decision Date:** 2026-09-19  
**Source:** design "Transport and clients"; "Changes from the reviewed draft" §3  

### Context

The draft assumed HTTP over a Unix socket needs `hyperlocal` or a
custom hyper connector. As of reqwest 0.13, `ClientBuilder::unix_socket(path)`
does it.

### Decision

The client is a plain `reqwest::Client`. No extra crate and no
hand-written connector.

### Consequences

One well-known HTTP client for both transports; Windows has
no UDS client, which is why its default is TCP.

### Alternatives Considered

`hyperlocal`; a bespoke hyper connector.

### Implementation

**Enforced by:** `Cargo.toml`; `arch-qa`.

### Related Documents

- [REQ-TRN-0005](requirements.md)
- [NFR-TRN-0002](requirements.md)

---

## ADR-TRN-0002: One resolution order with platform defaults

**Status:** Active  
**Decision Date:** 2026-09-19  
**Source:** design "Transport and clients" table  

### Context

Daemon and CLI are separate binaries that must agree on an address
with no coordination at run time.

### Decision

Both call the same resolver with the same order: `--endpoint`,
then `SC_ENDPOINT`, then config, then the platform default (UDS under the
instance root on Unix, TCP loopback on Windows).

### Consequences

Agreement is structural. Overrides work identically on both
sides, which is what tests and multi-instance setups rely on.

### Alternatives Considered

Discovery files or port scanning; separate client and server
defaults.

### Implementation

**Enforced by:** Both generated binaries call `resolve_endpoint`; `req-qa`.

### Related Documents

- [REQ-TRN-0001](requirements.md)
- [REQ-TRN-0002](requirements.md)

---

## ADR-TRN-0003: Response types belong to the caller

**Status:** Active  
**Decision Date:** 2026-09-19  
**Source:** decided here; applies repo [ADR-RUN-0003](../architecture.md)  

### Context

The CLI sketch in the design returns `Envelope<Widget>` from
`client.post(...)`, and `DAEMON.NOT_RUNNING` is an sc-ai-cli error code. Both
suggest `sc-transport` should know `sc-command`'s types, which would add an
edge between two crates that must stay standalone.

### Decision

`Client` is generic over `T: DeserializeOwned`; the caller names
`Envelope<Widget>`. `DaemonNotRunning` is this crate's own typed error carrying
the code and suggested action as plain strings. Mapping it into an `OpError`
happens in the project's CLI, which depends on both crates.

### Consequences

No edge among the standalone crates. The mapping is a few
lines of template code a project can change.

### Alternatives Considered

`sc-transport` depending on `sc-command`; a shared types crate.

### Implementation

**Enforced by:** `boundaries/sc-transport/` `forbidden_edges`.

### Related Documents

- [REQ-TRN-0005](requirements.md)
- [REQ-TRN-0006](requirements.md)
- [NFR-TRN-0003](requirements.md)

---

## ADR-TRN-0004: The `server` feature gates axum

**Status:** Active  
**Decision Date:** 2026-09-19  
**Source:** design "Repository layout"; applies repo [ADR-RUN-0003](../architecture.md)  

### Context

Listener binding exists to feed `axum::serve`; the client half is
used by a CLI that must not link Axum.

### Decision

`bind` and `Listener` sit behind a `server` feature that is off
by default. Only `sc-runtime` enables it.

### Consequences

CI builds and tests the crate both ways.

### Alternatives Considered

Separate client and server crates.

### Implementation

**Enforced by:** `cargo tree` evidence; the boundary manifest.

### Related Documents

- [NFR-TRN-0001](requirements.md)
- [REQ-TRN-0003](requirements.md)
