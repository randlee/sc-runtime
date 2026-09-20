# `sc-transport` Architecture

**ID Range:** ADR-TRN-0001 through ADR-TRN-0006  
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
| endpoint-to-string conversion (name OPEN, [REQ-TRN-0001](requirements.md)) | default | renders a resolved endpoint in the form `--endpoint` and `SC_ENDPOINT` accept; parsing it back gives an equal endpoint |
| endpoint and instance-root configuration type (name OPEN, [REQ-TRN-0001](requirements.md)) | default | `serde::Deserialize` value a CLI config uses directly and `sc_runtime::DaemonConfig` embeds |
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
`hyperlocal`; `reqwest` at `0.13`); `cargo tree -p sc-transport -e normal
--prefix none`, with and without `--features server`, printing no line
beginning with `hyperlocal ` (the name followed by a space); the `arch-qa`
review agent checks the source for a custom connector or hand-written HTTP.

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
2. the value of the `SC_ENDPOINT` environment variable, passed in as an
   explicit input;
3. the endpoint from the caller's configuration, passed in;
4. the platform default: UDS at `<instance-root>/daemon.sock` on macOS and
   Linux; TCP on `127.0.0.1` with the port from configuration on Windows.

`<instance-root>` is the per-application, per-user directory resolved by
sc-transport, or an explicitly supplied path; its default location is
undecided ([REQ-TRN-0002](requirements.md)).

Further decision points:

5. The resolver accepts the `SC_ENDPOINT` value as an explicit input. A
   convenience form may read the variable itself, only with
   `std::env::var_os("SC_ENDPOINT")`, and then calls the explicit form.
   Decided in this document.
6. Who calls the resolver. In a CLI, the `cli` crate's own code calls it
   with the parsed `--endpoint` value, the `SC_ENDPOINT` value and its
   configuration ([REQ-RUN-0310](../requirements.md)). In a daemon,
   `sc_runtime`'s `run()` calls the instance-root function and the resolver
   and has no endpoint logic of its own; the daemon's `main.rs` puts the
   parsed `--endpoint` value, the configured endpoint and any explicit
   instance root into `DaemonConfig`, which carries them to `run()`
   ([REQ-RT-0008](../sc-runtime/requirements.md)). No other code in the
   daemon, the CLI or `sc-runtime` computes an endpoint.
7. The endpoint and instance-root portion of configuration is one public
   `serde::Deserialize` type exported by `sc-transport` with default
   features. A CLI cannot name `sc_runtime::DaemonConfig` because a CLI does
   not depend on `sc-runtime`, so the shared type lives here; `DaemonConfig`
   embeds it.
8. TCP ports come from project configuration (or from an explicit
   override). Port ranges are allocated per project in the central port
   registry kept in the synaptic-canvas repository. No TCP port is
   hard-coded in the crates of this workspace or in the template.

The order of sources 1 to 3 is decided in this document; the sc-runtime
design names the sources but not their precedence. The Windows address
`127.0.0.1` is stated by the design; that the Windows port comes from
configuration is decided in this document.

**OPEN:** The Windows default result when configuration supplies no port is
not decided ([REQ-TRN-0001](requirements.md)).

**OPEN:** The field names of `DaemonConfig` and of the shared configuration
type are not decided ([REQ-TRN-0001](requirements.md),
[REQ-RT-0001](../sc-runtime/requirements.md)).

**OPEN:** Whether v0.1 of the template renders a default TCP port into a
generated project's configuration is not decided
([REQ-RUN-0301](../requirements.md)).

**OPEN:** The resolver returns one endpoint. Whether
`sc_runtime::Daemon::builder()` binds a UDS and a TCP listener together in
v0.1, and where the second endpoint would come from, is not decided
([REQ-RT-0001](../sc-runtime/requirements.md)).

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

**Enforced by:** the precedence unit tests of
[REQ-TRN-0001](requirements.md); the end-to-end tests of
[REQ-RUN-0310](../requirements.md) and
[REQ-RT-0008](../sc-runtime/requirements.md), in which a client and a daemon
given the same inputs meet; and a text search, done by the `arch-qa` review
agent, of non-test Rust source under `crates/` and `template/`: the strings
`daemon.sock`, `SC_ENDPOINT` and `127.0.0.1` appear only inside
`crates/sc-transport`, and no literal TCP port number appears as a default
anywhere.

### Related Documents

- [REQ-TRN-0001](requirements.md)
- [REQ-TRN-0002](requirements.md)
- [REQ-RT-0008](../sc-runtime/requirements.md)
- [REQ-RUN-0310](../requirements.md)

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

**Enforced by:** the TOML boundary manifest in `boundaries/sc-transport/`,
whose `forbidden_edges` list holds `"sc-transport -> sc-config"`,
`"sc-transport -> sc-command"`, `"sc-transport -> sc-runtime"`,
`"sc-transport -> sc-observability"` and
`"sc-transport -> sc-observability-otlp"`
([NFR-TRN-0003](requirements.md)); `sc-lint-boundary`, which checks
inter-crate edges and the public facade list, runs it under `just lint`.
That the source never names `Envelope` or `OpError` is checked by the
inspection criterion of [REQ-TRN-0005](requirements.md).

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
compile `sc-transport` with `server`, and a CLI built in that same build is
linked against the `server`-enabled crate. The property "a CLI links no
axum" is therefore defined for a CLI built as its own selection
(`cargo build -p cli`), and evidence must come from `cargo tree` on the CLI
or on this crate alone, not from a workspace build.

**OPEN:** Whether workspace-wide builds are exempt from the no-axum
property, or the crate must instead be split into a client crate and a
server crate, is not decided; the spike must measure it. The open point is
owned by [NFR-RUN-0001](../requirements.md) and
[ADR-RUN-0003](../architecture.md).

### Alternatives Considered

- Separate client and server crates (for example `sc-transport-client` and
  `sc-transport-server`). Not chosen because it doubles the crates to
  publish and version, and splits the shared endpoint resolver across crates
  or forces a third one. It is not equivalent to the feature: a crate split
  keeps axum out of a CLI even in a workspace-wide build, which the feature
  does not. It is reconsidered if the OPEN above is resolved against the
  feature.

### Implementation

**Enforced by:** `cargo tree -p sc-transport -e normal --prefix none`
printing no line beginning with `axum ` (the name followed by a space), and
the same command with `--features server` printing one; `just test` running
`cargo test -p sc-transport` and `cargo test -p sc-transport --features
server` ([REQ-RUN-0004](../requirements.md)). The boundary manifest format
has no cargo-feature key, so `sc-lint-boundary` does not enforce this
decision; it enforces only the inter-crate edges of
[NFR-TRN-0003](requirements.md).

### Related Documents

- [NFR-TRN-0001](requirements.md)
- [REQ-TRN-0003](requirements.md)

---

## ADR-TRN-0005: Transport access control is UDS file permissions only

**Status:** Active  
**Decision Date:** 2026-09-19  
**Source:** sc-runtime design, 2026-09-19  

### Context

A daemon built on `sc-transport` serves every operation of an application
over HTTP, so anything that can open a connection to its listener can run
those operations. On macOS and Linux the default listener is a Unix domain
socket (UDS) file at `<instance-root>/daemon.sock`; on Windows, for
cross-host use and for browser frontends it is TCP. `<instance-root>` is the
per-application, per-user directory resolved by sc-transport, or an
explicitly supplied path; its default location is undecided
([REQ-TRN-0002](requirements.md)). A UDS file can be protected by ordinary
file permissions, the approach dockerd uses. A TCP port has no such
protection: any local process, and for a non-loopback address any host that
can route to it, can connect. The sc-runtime design raises bearer tokens for
TCP listeners, and `Origin` and `Host` header checks on the `/mcp` route, as
ideas only and leaves them out of the build plan. Without a recorded
decision a reader cannot tell whether a TCP listener is meant to be
protected.

### Decision

1. On a UDS listener the only access control is the permission bits of the
   socket file. After `sc-transport` binds it, the file has no access for
   group and no access for others (`mode & 0o077 == 0`,
   [REQ-TRN-0004](requirements.md)). There is no token, no peer-credential
   check and no header check.
2. A TCP listener bound by `sc-transport` is unauthenticated in v0.1. Any
   process that can reach the address can call the daemon.
3. The only TCP address `sc-transport` computes by default is the loopback
   address `127.0.0.1` (the Windows platform default). Any other TCP address
   is used only when an override (`--endpoint`, `SC_ENDPOINT`) or the
   caller's configuration names it.
4. Bearer tokens for TCP listeners, and `Origin` and `Host` checks on
   `/mcp`, are deferred. v0.1 of `sc-transport` and `sc-runtime` contains no
   such check. Adding one needs a later ADR that names this one.

**OPEN:** The exact permission mode of the socket file (for example `0600`
or `0700`), and whether it is set on the socket file, on the instance-root
directory or on both, is not decided. Only "no group or other access" is
binding ([REQ-TRN-0004](requirements.md)).

### Consequences

The default transport on macOS and Linux needs no port, no token and no
secret storage, and is private to the operating-system user. On Windows, and
wherever a project configures TCP, every local process of every user can
call the daemon; a project that configures a non-loopback address exposes
the daemon to the network with no authentication, and that is the
project's own decision and risk. A browser page can send requests to a
loopback TCP listener, because no `Origin` or `Host` check exists. These
limits are accepted for v0.1.

### Alternatives Considered

- A bearer token on TCP listeners: the daemon writes a secret to a
  user-only file and the client sends it in a header. Deferred because it
  needs token creation, storage and rotation code on both sides, and the
  default transport does not need it.
- `Origin` and `Host` header checks on `/mcp`: reject requests that come
  from a browser page or a rebound DNS name. Deferred for the same reason;
  it matters only when a daemon listens on TCP.
- UDS peer-credential checks (`SO_PEERCRED` and similar): the daemon asks
  the kernel for the connecting user id. Rejected because file permissions
  already restrict who can connect, and the call differs per platform.
- No TCP support until authentication exists. Rejected because Windows has
  no UDS client in reqwest, so Windows would have no transport at all.

### Implementation

**Enforced by:** the permission test of [REQ-TRN-0004](requirements.md)
(criterion 1: `mode & 0o077 == 0` after bind); the Windows default test of
[REQ-TRN-0001](requirements.md) (criterion 5: the address is `127.0.0.1`);
the `arch-qa` review agent checks that `crates/sc-transport/src` and
`crates/sc-runtime/src` contain no token, `Origin` or `Host` check and no
default TCP address other than `127.0.0.1`.

### Related Documents

- [REQ-TRN-0004](requirements.md)
- [REQ-TRN-0001](requirements.md)
- [REQ-TRN-0003](requirements.md)

---

## ADR-TRN-0006: `bind` unlinks an existing socket file; caller guarantees

**Status:** Active  
**Decision Date:** 2026-09-19  
**Source:** decided in this document; the sc-runtime design does not state it  

### Context

A Unix domain socket (UDS) listener is a file. When a daemon process dies
without cleaning up (a crash, `kill -9`, power loss) the file stays, and the
next attempt to bind the same path fails with "address in use". A daemon
that cannot restart after a crash without manual cleanup is not acceptable.
But a socket file that exists may also belong to a daemon that is alive, and
removing it cuts that daemon off from every new client. The listener-binding
function of `sc-transport` (sketched as `bind(&Endpoint)`, behind the
`server` cargo feature) cannot tell the two cases apart reliably on its own.
In `sc-runtime` the question is already answered before bind is called:
`run()` takes an OS exclusive lock on `<instance-root>/daemon.lock` first,
and holding that lock proves no other daemon for that instance root is
alive. `<instance-root>` is the per-application, per-user directory resolved
by sc-transport, or an explicitly supplied path; its default location is
undecided ([REQ-TRN-0002](requirements.md)). `sc-transport` is also
published for use without `sc-runtime`, where no such lock exists unless the
caller provides one.

### Decision

1. When the endpoint is a UDS path and a socket file already exists there,
   the bind function removes (unlinks) the file and binds a new socket at
   the same path. It does not return "address in use" for that reason.
2. The bind function does not try to find out whether another process is
   listening: no connect probe, no process check, no lock of its own.
3. The caller must guarantee, before calling, that no live process is
   listening on the path. `sc-runtime` meets this by holding the
   `daemon.lock` exclusive lock before it binds. A standalone caller must
   provide an equivalent guarantee.
4. The rustdoc of the bind function states points 1 and 3, because the
   precondition is otherwise invisible to a standalone user.
5. This decision is made in this document; the sc-runtime design states
   only that file permissions restrict access to the socket.

**OPEN:** The behaviour when the existing path is not a socket (a regular
file or a directory) is not decided ([REQ-TRN-0004](requirements.md)).

### Consequences

A daemon restarts cleanly after a crash with no manual step. `sc-transport`
needs no locking dependency and keeps a single job. The cost is a sharp
edge: a standalone caller that ignores the precondition and binds a path a
live daemon is using silently disconnects that daemon from new clients. The
unlink and the permission change are blocking file operations done once at
bind time, outside the request path, so they do not conflict with the
async-end-to-end rule ([NFR-RUN-0002](../requirements.md)).

### Alternatives Considered

- Connect probe: try to connect to the existing socket and unlink only if
  the connection is refused. Rejected because there is a race between the
  probe and the unlink, a live but overloaded daemon can look dead, and it
  duplicates a guarantee the singleton lock already gives.
- Fail with "address in use" and leave cleanup to the caller or the user.
  Rejected because every restart after a crash would then need a manual
  step or the same unlink code copied into every caller.
- Take a lock file inside `bind`. Rejected because it would add a locking
  dependency to `sc-transport` and duplicate `sc-runtime`'s `daemon.lock`,
  which must be taken earlier than bind, before the stores are opened.

### Implementation

**Enforced by:** the stale-socket test, the rustdoc criterion and the
no-probe inspection of [REQ-TRN-0004](requirements.md) (criteria 2 to 5);
on the caller side, the start-up order of `sc_runtime`'s `run()`, which
takes `daemon.lock` before it binds
([REQ-RT-0001](../sc-runtime/requirements.md)).

### Related Documents

- [REQ-TRN-0004](requirements.md)
- [REQ-TRN-0003](requirements.md)
- [ADR-TRN-0005](architecture.md)
