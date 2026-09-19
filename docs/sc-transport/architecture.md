# `sc-transport` Architecture

Crate-level architecture and ADRs for `sc-transport`. Repo-level architecture is in
[`../architecture.md`](../architecture.md); this crate's requirements are in
[`requirements.md`](requirements.md); its boundary manifest is
`boundaries/sc-transport/`. ADRs follow
`.claude/skills/plan-hardening/req-adr-format.md`. Every ADR here is `accepted`
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

## ADR

### ADR-TRANSPORT-001 reqwest `unix_socket`, no custom connector

| Field | Value |
|---|---|
| Status | accepted |
| Date | 2026-09-19 |
| Source | design "Transport and clients"; "Changes from the reviewed draft" §3 |
| Relates to | `REQ-TRANSPORT-005`, `NFR-TRANSPORT-002` |

**Context.** The draft assumed HTTP over a Unix socket needs `hyperlocal` or a
custom hyper connector. As of reqwest 0.13, `ClientBuilder::unix_socket(path)`
does it.

**Decision.** The client is a plain `reqwest::Client`. No extra crate and no
hand-written connector.

**Consequences.** One well-known HTTP client for both transports; Windows has
no UDS client, which is why its default is TCP.

**Rejected.** `hyperlocal`; a bespoke hyper connector.

**Enforced by.** `Cargo.toml`; `arch-qa`.

### ADR-TRANSPORT-002 One resolution order with platform defaults

| Field | Value |
|---|---|
| Status | accepted |
| Date | 2026-09-19 |
| Source | design "Transport and clients" table |
| Relates to | `REQ-TRANSPORT-001`, `REQ-TRANSPORT-002` |

**Context.** Daemon and CLI are separate binaries that must agree on an address
with no coordination at run time.

**Decision.** Both call the same resolver with the same order: `--endpoint`,
then `SC_ENDPOINT`, then config, then the platform default (UDS under the
instance root on Unix, TCP loopback on Windows).

**Consequences.** Agreement is structural. Overrides work identically on both
sides, which is what tests and multi-instance setups rely on.

**Rejected.** Discovery files or port scanning; separate client and server
defaults.

**Enforced by.** Both generated binaries call `resolve_endpoint`; `req-qa`.

### ADR-TRANSPORT-003 Response types belong to the caller

| Field | Value |
|---|---|
| Status | accepted |
| Date | 2026-09-19 |
| Source | decided here; applies repo ADR-004 |
| Relates to | `REQ-TRANSPORT-005`, `REQ-TRANSPORT-006`, `NFR-TRANSPORT-003` |

**Context.** The CLI sketch in the design returns `Envelope<Widget>` from
`client.post(...)`, and `DAEMON.NOT_RUNNING` is an sc-ai-cli error code. Both
suggest `sc-transport` should know `sc-command`'s types, which would add an
edge between two crates that must stay standalone.

**Decision.** `Client` is generic over `T: DeserializeOwned`; the caller names
`Envelope<Widget>`. `DaemonNotRunning` is this crate's own typed error carrying
the code and suggested action as plain strings. Mapping it into an `OpError`
happens in the project's CLI, which depends on both crates.

**Consequences.** No edge among the standalone crates. The mapping is a few
lines of template code a project can change.

**Rejected.** `sc-transport` depending on `sc-command`; a shared types crate.

**Enforced by.** `boundaries/sc-transport/` `forbidden_edges`.

### ADR-TRANSPORT-004 The `server` feature gates axum

| Field | Value |
|---|---|
| Status | accepted |
| Date | 2026-09-19 |
| Source | design "Repository layout"; applies repo ADR-004 |
| Relates to | `NFR-TRANSPORT-001`, `REQ-TRANSPORT-003` |

**Context.** Listener binding exists to feed `axum::serve`; the client half is
used by a CLI that must not link Axum.

**Decision.** `bind` and `Listener` sit behind a `server` feature that is off
by default. Only `sc-runtime` enables it.

**Consequences.** CI builds and tests the crate both ways.

**Rejected.** Separate client and server crates.

**Enforced by.** `cargo tree` evidence; the boundary manifest.
