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

## ADR-TRN-0001: Client uses reqwest `unix_socket`, no custom connector

**Status:** Active  
**Decision Date:** 2026-09-19  
**Source:** sc-runtime design, 2026-09-19  

### Context

The client half of the crate `sc-transport` (`crates/sc-transport`) sends
HTTP requests from a CLI to a local daemon. On macOS and Linux the default
transport is a Unix domain socket (UDS); on Windows and for cross-host use
it is TCP. Most Rust HTTP clients historically spoke only TCP, so an early
draft of this project assumed HTTP over a Unix socket needed the
`hyperlocal` crate or a hand-written hyper connector. That is no longer
true: reqwest 0.13 provides `ClientBuilder::unix_socket(path)`. The project
prefers standard crates used the documented way over code it must own.

### Decision

The client in `sc-transport` is a plain `reqwest::Client` from reqwest 0.13.
For a UDS endpoint it is built with
`reqwest::ClientBuilder::unix_socket(path)`. For a TCP endpoint it is built
normally and requests go to the base URL `http://<ip>:<port>`.
`crates/sc-transport/Cargo.toml` does not depend on `hyperlocal`. The crate
contains no type that implements a hyper or tower connector trait and no
code that writes or parses HTTP on a raw stream.

### Consequences

One well-known HTTP client serves both transports, and the UDS path adds no
code beyond one builder call. `unix_socket` exists only on Unix, so there is
no UDS client on Windows; that is why the Windows default endpoint is TCP on
`127.0.0.1`. The crate is tied to reqwest 0.13 or later.

### Alternatives Considered

- `hyperlocal`: a crate that adds a Unix-socket connector to hyper.
  Rejected because it is an extra dependency and a lower-level client API
  for something reqwest now does itself.
- A bespoke hyper connector: a type written in this crate that opens a
  `UnixStream` for hyper. Rejected because it is custom code to write, test
  and maintain where a standard tool already does the job.

### Implementation

**Enforced by:** inspection of `crates/sc-transport/Cargo.toml` (no
`hyperlocal`; `reqwest` at `0.13`) and `cargo tree -p sc-transport
--all-features`; the `arch-qa` review agent checks the source for a custom
connector or hand-written HTTP.

### Related Documents

- [REQ-TRN-0005](requirements.md)
- [NFR-TRN-0002](requirements.md)

---

## ADR-TRN-0002: One endpoint resolver shared by daemon and CLI

**Status:** Active  
**Decision Date:** 2026-09-19  
**Source:** sc-runtime design, 2026-09-19  

### Context

A daemon and its CLI are separate binaries. The CLI must find the address
the daemon listens on with no coordination at run time: there is no
registry process to ask. The address can come from several places: a
`--endpoint` command-line flag, the `SC_ENDPOINT` environment variable, the
project's configuration, or a platform default. If each binary combined
those sources with its own code, they could disagree and never meet. Tests
and multi-instance setups also need to move both binaries to another
address with one setting.

### Decision

The crate `sc-transport` exposes one public resolver function (sketched as
`resolve_endpoint(...)`; the name and signature are illustrative until the
contract sprint pins them). The daemon binary and the CLI binary both call
it. It returns a UDS path or a TCP socket address, taken from the first of
these sources that supplies a value:

1. the `--endpoint` flag value, parsed by the calling binary and passed in;
2. the `SC_ENDPOINT` environment variable;
3. the endpoint from the caller's configuration, passed in;
4. the platform default: UDS at `<instance-root>/daemon.sock` on macOS and
   Linux; TCP on `127.0.0.1` with the port from configuration on Windows.

`<instance-root>` is the per-app, per-user directory that the same crate
resolves from the application name, or an explicitly supplied path. No other
code in the daemon, the CLI or `sc-runtime` computes an endpoint. The order
of sources 1 to 3 is decided in this document; the sc-runtime design names
the sources but not their precedence.

### Consequences

Agreement between daemon and CLI is structural, not a convention to
remember. An override works identically on both sides, which tests and
multi-instance setups rely on. The crate does not parse command lines or
load configuration, so it needs neither `clap` nor `sc-config`; the calling
binary passes those values in. A user who sets `SC_ENDPOINT` for the CLI but
not for the daemon will still get a mismatch; the resolver cannot prevent
different inputs.

### Alternatives Considered

- Discovery files: the daemon writes its address to a file that the CLI
  reads. Rejected because it adds state that goes stale after a crash, and
  the file's own location would still need a shared resolver.
- Port scanning: the CLI probes a port range for the daemon. Rejected
  because it is slow, unreliable, and does not apply to UDS.
- Separate client and server defaults: each binary holds its own default
  logic. Rejected because the two copies can drift apart, which is the exact
  failure this decision removes.

### Implementation

**Enforced by:** both generated binaries (`daemon` through `sc-runtime`, and
`cli`) obtain their endpoint only from the `sc-transport` resolver; the
`req-qa` review agent checks the precedence tests of
[REQ-TRN-0001](requirements.md).

### Related Documents

- [REQ-TRN-0001](requirements.md)
- [REQ-TRN-0002](requirements.md)

---

## ADR-TRN-0003: Response types and `OpError` mapping belong to the caller

**Status:** Active  
**Decision Date:** 2026-09-19  
**Source:** decided in this document; applies repo [ADR-RUN-0003](../architecture.md)  

### Context

Two facts pull `sc-transport` toward knowing the types of the crate
`sc-command`. First, the CLI example in the sc-runtime design receives an
`Envelope<Widget>` from `client.post(...)`, and `Envelope<T>` is defined in
`sc-command`. Second, `DAEMON.NOT_RUNNING` is an error code in the sc-ai-cli
error convention, whose error type `OpError` is also defined in
`sc-command`. But the repository rule is that `sc-config`, `sc-transport`
and `sc-command` are standalone crates with no dependency edges among them,
so each can be published and used alone
([ADR-RUN-0003](../architecture.md)). A dependency from `sc-transport` on
`sc-command` would break that rule.

### Decision

1. The `sc-transport` client methods `get` and `post` are generic over the
   response type `T: serde::de::DeserializeOwned`. The caller names the
   type, for example `Envelope<Widget>`. `sc-transport` never names
   `Envelope` or `OpError`.
2. `TransportError::DaemonNotRunning` is `sc-transport`'s own typed error
   variant. It carries the code `DAEMON.NOT_RUNNING` and the suggested
   action `run <app> daemon start` as plain strings.
3. Converting `DaemonNotRunning` into an `OpError` is done in the project's
   CLI crate (template code in the generated `cli` crate), which depends on
   both `sc-transport` and `sc-command`.
4. `crates/sc-transport/Cargo.toml` does not list `sc-command`.

The method and type names are illustrative until the contract sprint pins
them. This decision was made in this document; the sc-runtime design does
not state where the mapping lives.

### Consequences

There is no dependency edge among the standalone crates, and `sc-transport`
is usable by a daemon and CLI pair that does not use the sc-ai-cli envelope
at all. The cost is a few lines of mapping code in each generated CLI, which
the project owns and can change. The strings `DAEMON.NOT_RUNNING` and the
`OpError` field layout are defined in two crates that cannot check each
other at compile time.

### Alternatives Considered

- `sc-transport` depends on `sc-command` and returns `OpError` directly.
  Rejected because it adds an edge between two crates that must stay
  standalone, and forces every user of the transport to take the envelope.
- A shared types crate that both depend on. Rejected because it is a fifth
  crate to publish and version for the sake of two strings.

### Implementation

**Enforced by:** the boundary manifest in `boundaries/sc-transport/`, whose
`forbidden_edges` list names `sc-command`, `sc-config` and `sc-runtime`;
`sc-lint-boundary` checks it when `just lint` runs.

### Related Documents

- [REQ-TRN-0005](requirements.md)
- [REQ-TRN-0006](requirements.md)
- [NFR-TRN-0003](requirements.md)

---

## ADR-TRN-0004: The `server` cargo feature gates axum and listener binding

**Status:** Active  
**Decision Date:** 2026-09-19  
**Source:** sc-runtime design, 2026-09-19; applies repo [ADR-RUN-0003](../architecture.md)  

### Context

The crate `sc-transport` has two halves with different users. Listener
binding exists to feed `axum::serve` in a daemon, so it needs axum. Endpoint
resolution and the HTTP client are used by a CLI, and a CLI binary must link
none of axum, rmcp or sqlx: they would multiply its build time and binary
size. Both halves must agree on endpoints, so they belong in one crate. The
repository rule is that server-side code in the standalone crates sits
behind a `server` cargo feature that is off by default
([ADR-RUN-0003](../architecture.md)).

### Decision

In `crates/sc-transport`, the listener-binding function and the listener
type (sketched as `bind(&Endpoint)` and `Listener`; names illustrative until
the contract sprint pins them) are compiled only under
`#[cfg(feature = "server")]`. In `Cargo.toml` the feature `server` is not in
`default`, and `axum` is `optional = true`, enabled only by `server`.
Endpoint resolution, the client and `TransportError` are in the default
build. Within this workspace only the crate `sc-runtime` enables
`sc-transport/server`; a generated `cli` crate depends on `sc-transport`
with default features.

### Consequences

A CLI that depends on `sc-transport` links tokio and reqwest but not axum.
CI must build and test the crate twice, with and without `server`, because
code behind a feature is otherwise not compiled. Cargo unifies features
across one build, so a workspace build that includes `sc-runtime` will
compile `sc-transport` with `server`; evidence that the CLI is clean must
come from `cargo tree` on the CLI or the crate alone, not from a workspace
build.

### Alternatives Considered

- Separate client and server crates (for example `sc-transport-client` and
  `sc-transport-server`). Rejected because it doubles the crates to publish
  and version for the same effect, and splits the shared endpoint resolver
  across crates or forces a third one.

### Implementation

**Enforced by:** `cargo tree -p sc-transport -e normal` showing no `axum`
and `cargo tree -p sc-transport -e normal --features server` showing it; CI
running `cargo test -p sc-transport` with and without `--features server`;
the boundary manifest in `boundaries/sc-transport/`.

### Related Documents

- [NFR-TRN-0001](requirements.md)
- [REQ-TRN-0003](requirements.md)
