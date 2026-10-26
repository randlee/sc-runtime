# `sc-transport` Requirements

**ID Range:** REQ-TRN-0001 through REQ-TRN-0006; NFR-TRN-0001 through NFR-TRN-0004  
**Status:** Draft  
**Created:** 2026-09-19  
**Last Updated:** 2026-10-23
**Owner:** sc-runtime maintainers  

---

Crate-level requirements for `sc-transport`. Repo-level requirements are in
[`../requirements.md`](../requirements.md); this crate's architecture and ADRs
are in [`architecture.md`](architecture.md). Extracted from
[`../sc-runtime-design.md`](../sc-runtime-design.md). Entries follow the
shared SC requirement template: each entry has the three sections
**Requirement Statement**, **Rationale** and **Success Criteria**, in that
order. Every id is binding and is never reused.

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

## REQ-TRN-0001: Endpoint resolution order in `sc-transport`

**Status:** Active  

### Requirement Statement

The crate `sc-transport` (`crates/sc-transport`) MUST expose one public
endpoint resolver that both a daemon binary and a CLI binary call to learn
where the daemon listens. The resolver's result MUST be one of exactly two
cases: a Unix domain socket (UDS) path, or a TCP socket address. In the
crate's public-surface sketch these are `Endpoint::Uds(PathBuf)` and
`Endpoint::Tcp(SocketAddr)`, produced by `resolve_endpoint(...)`.

The resolver MUST pick the first source below that supplies a value. This
precedence order is decided in this document; the sc-runtime design lists the
sources (`--endpoint`, `SC_ENDPOINT`, config, platform default) but not their
order.

| Order | Source | How the value reaches the resolver |
|---|---|---|
| 1 | the `--endpoint` command-line flag | The calling binary parses the flag (the crate MUST NOT depend on `clap` or parse `argv`) and passes the value, or "absent", as an argument. |
| 2 | the `SC_ENDPOINT` environment variable | The value of the process environment variable named exactly `SC_ENDPOINT`, passed to the resolver as an explicit argument (the value, or "absent"). |
| 3 | the endpoint from the caller's configuration | The caller passes it as an argument. The crate MUST NOT depend on `sc-config`. |
| 4 | the platform default | Computed by the crate, see below. |

The resolver MUST accept the `SC_ENDPOINT` value as an explicit input, so
that a caller or a test can supply it without changing the process
environment. The crate MAY also offer a convenience form of the resolver
that reads the variable itself. That form MUST read it with
`std::env::var_os("SC_ENDPOINT")`, MUST NOT use `std::env::var` or
`std::env::vars`, and MUST then call the explicit-input form. Both points
are decided in this document; the sc-runtime design names `SC_ENDPOINT` as a
source but does not say who reads it.

The platform default MUST be:

| Platform | Default endpoint |
|---|---|
| macOS and Linux | UDS at `<instance-root>/daemon.sock` |
| Windows | TCP on IP address `127.0.0.1`, with the port taken from the caller's configuration |

`<instance-root>` is the per-application, per-user directory resolved by
sc-transport, or an explicitly supplied path. On Unix the default base is
non-empty `XDG_RUNTIME_DIR`, otherwise `$HOME/.local/state`. On Windows it
is `LOCALAPPDATA`. The path is `<base>/<app>`
([REQ-TRN-0002](requirements.md)). `instance_root` does not create it.

The Windows address `127.0.0.1` is stated by the sc-runtime design. That the
Windows port comes from the caller's configuration is decided in this
document: the design states "port from config" for cross-host TCP and names
no port for the Windows default. The crate MUST NOT contain a literal TCP
port number that it uses as a default; TCP ports come only from an override
or from the caller's configuration ([ADR-TRN-0002](architecture.md)).

The endpoint and instance-root portion of a project's configuration MUST be
one public type exported by `sc-transport` with default features, and that
type MUST implement `serde::Deserialize`. This follows from two other binding
facts: the CLI and the daemon must give the resolver the same configuration
value, and a CLI MUST NOT depend on `sc-runtime`, so it cannot name
`sc_runtime::DaemonConfig`. `DaemonConfig` embeds this type
([REQ-RT-0001](../sc-runtime/requirements.md),
[REQ-RT-0008](../sc-runtime/requirements.md)); a CLI's configuration uses it
directly. The resolver takes source 3 and the explicit instance root from a
value of this type or from its fields.

A source value that cannot be parsed as an endpoint MUST produce an `Err` of
the crate's error enum `TransportError`. The resolver MUST NOT panic and MUST
NOT silently fall through to the next source in that case.

The crate MUST also provide, with default features, the reverse conversion:
from a resolved endpoint to the string form accepted by `--endpoint` and
`SC_ENDPOINT`. Parsing the rendered string MUST yield an endpoint equal to
the original. Any code that needs that string, including
`sc_runtime::testing::DaemonFixture`
([REQ-RT-0006](../sc-runtime/requirements.md)), obtains it from this
conversion and does not format it itself. This follows from REQ-RT-0006 and
[REQ-RT-0008](../sc-runtime/requirements.md), which bar `sc-runtime` from
holding endpoint logic. The conversion is `Display` and `FromStr` on
`Endpoint`: `uds:<path>` and `tcp:<socket-addr>`. `tcp:` takes a literal
`SocketAddr` and does not resolve DNS. An empty string is
`TransportError::InvalidEndpoint` and does not fall through.

Pinned by sprint a-3:

- The endpoint type is `Endpoint::{Uds(PathBuf), Tcp(SocketAddr)}`.
- The resolver is
  `resolve_endpoint(&AppName, flag: Option<&OsStr>, environment: Option<&OsStr>, &TransportConfig) -> Result<Endpoint, TransportError>`.
  `resolve_endpoint_from_env` is the convenience form and reads only
  `std::env::var_os("SC_ENDPOINT")`.
- An unparseable value is `TransportError::InvalidEndpoint`
  (`TRANSPORT.INVALID_ENDPOINT`).
- The configuration type is `TransportConfig`. It is `#[non_exhaustive]`
  and `serde::Deserialize`. Its fields are `endpoint: Option<String>`,
  `instance_root: Option<PathBuf>`, and `tcp_port: Option<u16>`. It has no
  `additional_endpoints` field. v0.1 binds exactly one listener per config.
  A later dual-bind adds a field on this type. v0.1 does not ignore a
  reserved value. `DaemonConfig` field names stay with
  [REQ-RT-0001](../sc-runtime/requirements.md); that type embeds
  `TransportConfig`.
- On Windows, a missing `tcp_port` with no override is
  `TransportError::MissingTcpPort` (`TRANSPORT.MISSING_TCP_PORT`). `detail`
  says to set `TransportConfig.tcp_port` or pass a `tcp:` override. This
  crate has no default TCP port.
- An override or `TransportConfig.endpoint` that names `uds:<path>` is a
  UDS endpoint on every platform, including Windows. The resolver returns
  it. `Client::new` and `bind` reject a UDS endpoint on non-Unix targets
  when they are used (`TransportError::Request` and `TransportError::Bind`).

Whether the project template writes a default TCP port into generated
configuration stays with [REQ-RUN-0301](../requirements.md). This crate does
not decide that.

### Rationale

A daemon and its CLI are separate binaries with no run-time coordination. If
they compute the endpoint differently they never meet, so one resolver used
by both makes disagreement impossible. UDS is the default on macOS and Linux
because access is controlled by file permissions and it needs no port and no
token, the same approach dockerd uses. Windows defaults to TCP because
reqwest's `ClientBuilder::unix_socket` exists only on Unix. Cross-host use
and browser frontends use TCP with a port from configuration; port ranges are
registered centrally in the synaptic-canvas repository. The override sources
exist so tests and multi-instance setups can point both binaries somewhere
else. `SC_ENDPOINT` is an explicit input because tests run in parallel in one
process, and a test that sets a process-wide environment variable breaks the
others. The configuration type lives here because this is the only crate
that both a CLI and a daemon depend on for endpoints. The callers are the
generated `cli` and `daemon`, which each accept `--endpoint`
([REQ-RUN-0310](../requirements.md)), and `sc-runtime`'s `run()`, which
resolves the daemon's bind endpoint only through this resolver
([REQ-RT-0008](../sc-runtime/requirements.md)).

### Success Criteria

1. A unit test in `crates/sc-transport` supplies all of sources 1, 2 and 3
   with three different valid endpoints and asserts the result equals the
   source 1 value.
2. A unit test supplies sources 2 and 3 only, both as explicit arguments,
   and asserts the result equals the supplied `SC_ENDPOINT` value.
3. A unit test supplies source 3 only and asserts the result equals the
   configuration value.
4. A unit test compiled only on Unix (`#[cfg(unix)]`) supplies no source,
   uses an explicit instance root `R` (a tempdir), and asserts the result is
   the UDS path `R/daemon.sock`.
5. A unit test compiled only on Windows (`#[cfg(windows)]`) supplies no
   override and a configuration port `P`, and asserts the result is the TCP
   address `127.0.0.1:P`.
6. A unit test supplies an unparseable source 1 value and asserts the result
   is `Err` with `TransportError::InvalidEndpoint`. The source 2 value is
   not returned.
7. Inspection of `crates/sc-transport` (source and tests) finds no call to
   `std::env::set_var` or `std::env::remove_var`; tests 1 to 6 pass
   `SC_ENDPOINT` as an argument and pass under the default parallel test
   runner.
8. `crates/sc-transport/Cargo.toml` lists neither `clap` nor `sc-config`.
9. Inspection of `crates/sc-transport/src` finds that the string
   `SC_ENDPOINT` is read, if at all, only through `std::env::var_os`, and
   finds no call to `std::env::var(` or `std::env::vars`.
10. `cargo doc -p sc-transport` with default features lists the public
    configuration type, and a test containing
    `fn assert_de<T: serde::de::DeserializeOwned>() {}` instantiated with
    that type compiles.
11. Inspection of non-test code in `crates/sc-transport/src` finds no
    literal TCP port number used as a default endpoint.
12. A unit test renders one UDS endpoint and one TCP endpoint to the string
    form, parses each string back as an explicit `--endpoint` value, and
    asserts the result equals the original endpoint, and asserts the literal
    strings `uds:<path>` and `tcp:<socket-addr>`.

---

## REQ-TRN-0002: Instance root directory resolution in `sc-transport`

**Status:** Active  

### Requirement Statement

The instance root is the one directory that holds a daemon's run-time files
for one application and one operating-system user: the UDS socket file
`daemon.sock` and the singleton lock file `daemon.lock`.

The crate `sc-transport` MUST expose a public function that takes an
application name (an [`AppName`](architecture.md), such as `"my-app"`) and
returns the default instance root path for that application and the current
user. The function is `instance_root(&AppName) -> Result<PathBuf, TransportError>`.
The crate-private function `instance_root_from_base(&AppName, Option<&Path>)`
is the path builder that receives the per-user base. It is not a facade item.

The default instance root MUST be a per-user location: two different
operating-system users MUST get different paths for the same application
name. Two different application names MUST give different paths for the same
user. The same application name and user MUST give the same path on every
call. The function MUST compute the path at run time from the operating
system or environment. The source MUST NOT contain a literal home-directory
or temp-directory path such as `/tmp` or `/Users/...`.

Every public function of the crate that uses an instance root (endpoint
resolution in particular) MUST also accept an explicitly supplied instance
root path, and MUST use that path unchanged in place of the default when it
is supplied.

Failure to determine the default location MUST be returned as an `Err` of
the crate's error enum `TransportError`. The function MUST NOT panic.

The default path MUST be built by one function inside the crate that
receives the per-user base location as an input (a path, or "absent"), so
that the per-user property and the failure case can be tested without
changing the process environment. The public function obtains the base
location from the operating system or environment and calls that function.
This structure is decided in this document.

The default base is decided by sprint a-3:

- Unix: non-empty `XDG_RUNTIME_DIR`, otherwise `$HOME/.local/state`.
- Windows: non-empty `LOCALAPPDATA`.
- The instance root is `<base>/<app>`. `instance_root` does not create the
  directory. The caller creates it.
- An explicit `TransportConfig.instance_root` is used unchanged as the Unix
  default's directory (`<explicit>/daemon.sock`), not as `<explicit>/<app>`.
- Failure is `TransportError::InstanceRootUnavailable`
  (`TRANSPORT.INSTANCE_ROOT_UNAVAILABLE`).

`AppName::new` accepts ASCII `A-Z`, `a-z`, `0-9`, `-`, and `_`, length
1 through 64 bytes. Rejection is `TransportError::InvalidName`
(`APP.INVALID_NAME`). `detail` says whether the name was empty, longer than
64 bytes, or which character was outside that set. The type exposes `as_str`
and `AsRef<str>`. It does not implement `Deref`. Tests load
`tests/fixtures/app-name-vectors.json` and do not embed a second list.

### Rationale

The socket and the daemon lock live in the instance root, so the directory
is what identifies "this application's daemon for this user". Daemon and CLI
must derive it the same way or the CLI looks for the socket in the wrong
place. The explicit override exists for tests: every test that touches the
filesystem or starts a daemon must use its own tempdir as instance root so
that tests run in parallel and never touch a developer's real daemon
([NFR-RUN-0005](../requirements.md)).

### Success Criteria

1. A unit test calls the default resolution twice with the application name
   `"app-a"` and asserts both results are equal.
2. A unit test calls the default resolution with `"app-a"` and `"app-b"` and
   asserts the two paths differ.
3. A unit test supplies an explicit instance root `R` (a tempdir) and asserts
   that endpoint resolution with no other source returns `R/daemon.sock` on
   Unix, that is, `R` is used unchanged and the default is not consulted.
4. Inspection of `crates/sc-transport/src` finds no string literal that
   starts with `/tmp`, `/home`, `/Users` or `C:\`.
5. No test in `crates/sc-transport` creates a file under the default
   instance root; every test that writes files uses a tempdir.
6. A unit test calls the path-building function with the application name
   `"app-a"` and two different per-user base locations `B1` and `B2` (two
   tempdirs) and asserts the two results differ, the first starts with `B1`
   and the second starts with `B2`.
7. A unit test calls the path-building function with the base location
   absent and asserts the result is `Err` with a `TransportError`, with no
   panic.
8. Inspection of `instance_root` finds that the base location is
   `XDG_RUNTIME_DIR` or `HOME` on Unix and `LOCALAPPDATA` on Windows, read
   at run time with `std::env::var_os`.

---

## REQ-TRN-0003: UDS and TCP listener binding behind the `server` feature

**Status:** Active  

### Requirement Statement

When the cargo feature `server` of the crate `sc-transport` is enabled, the
crate MUST expose a public function that takes a resolved endpoint (a UDS
path or a TCP socket address) and returns a bound listener. In the crate's
public-surface sketch these are `bind(&Endpoint) -> Result<Listener, TransportError>`
and `Listener`. `Listener` is one type for both transports. `local_endpoint`
returns the bound `Endpoint` (`TransportError::Bind` if the TCP address
cannot be read). A TCP port of `0` is reported as the port the operating
system assigned. `Listener` implements `axum::serve::Listener`.

The returned listener MUST be accepted by `axum::serve` (axum 0.8) together
with an ordinary `axum::Router`. The router and its handlers MUST NOT need
any change, type parameter or wrapper that depends on whether the listener
is UDS or TCP.

For a UDS endpoint the function MUST bind a Unix domain socket at the given
path. For a TCP endpoint the function MUST bind the given socket address.
When the TCP port is `0` the listener MUST let the caller read the actual
bound address.

One process MUST be able to hold a UDS listener and a TCP listener from this
same time and serve the same router on both. v0.1 `TransportConfig` describes
exactly one endpoint. It has no `additional_endpoints` field. A later
dual-bind adds a field on that type. v0.1 does not ignore a reserved value.
Calling `bind` twice remains valid. Whether `sc_runtime::Daemon::builder()`
uses two listeners is owned by
[REQ-RT-0001](../sc-runtime/requirements.md); a-5 copies the one-listener
decision into the root documents. This crate's dual-bind question is closed.

A bind failure (for example the address is in use, or the directory does not
exist) MUST be returned as an `Err` of the crate's error enum
`TransportError`. The function MUST NOT panic.

The function and the listener type MUST NOT exist in a build without the
`server` feature.

A bind failure is `TransportError::Bind` (`TRANSPORT.BIND`). The endpoint
lock already being held is `TransportError::EndpointBusy`
(`TRANSPORT.ENDPOINT_BUSY`). On a non-Unix target, `bind` of a UDS endpoint
returns `TransportError::Bind`. The parent directory is not created.

### Rationale

`axum::serve` accepts a `tokio::net::TcpListener` or a
`tokio::net::UnixListener` with no change to routes, so the transport is a
configuration choice that route code never sees. Binding both at once serves
the case where a browser frontend needs TCP while the CLI stays on the
socket. The code is behind `server` because it needs axum, and a CLI that
uses this crate must not link axum.

### Success Criteria

1. A test compiled with `--features server` on Unix binds a UDS endpoint in
   a tempdir, serves a router with one `GET` route through `axum::serve`,
   requests that route over the socket, and asserts status 200 and the
   expected body.
2. A test compiled with `--features server` binds the TCP endpoint
   `127.0.0.1:0`, reads the bound address, serves the same router function,
   requests the route over TCP, and asserts status 200 and the expected
   body.
3. A test compiled with `--features server` on Unix binds one UDS and one
   TCP listener in the same process, serves the same router on both, and
   asserts the route answers on both.
4. The router-building function used by tests 1 to 3 is one function with no
   generic parameter or argument that names the transport.
5. A test binds a TCP address that is already bound and asserts the result
   is `Err` with a `TransportError`.
6. `cargo doc -p sc-transport` without the feature shows neither the bind
   function nor the listener type.

---

## REQ-TRN-0004: UDS socket file permissions and stale-file replacement

**Status:** Active  

### Requirement Statement

This requirement applies to the listener-binding function of the crate
`sc-transport` (sketched as `bind(&Endpoint)`, behind the `server` cargo
feature) when the endpoint is a UDS path.

1. After a successful bind the socket file MUST be accessible only to the
   operating-system user that owns the process: the file's permission bits
   for group and for others MUST all be zero (`mode & 0o077 == 0`). These
   permissions are the only access control on a UDS listener
   ([ADR-TRN-0005](architecture.md)).
2. Before inspecting or removing an existing UDS socket file, `bind` MUST
   acquire a nonblocking OS-exclusive endpoint lock at the sibling path
   `<socket-path>.lock`. The returned `Listener` MUST retain that lock for
   its lifetime. Failure to acquire it is a typed bind error and MUST NOT
   inspect, remove, or replace the socket file. The lock path is formed by
   appending the literal `.lock` to the full socket path, not by replacing
   its extension.
3. Once it holds the endpoint lock, `bind` MUST remove an existing stale
   socket file and bind the new socket at the same path. It MUST NOT fail
   with "address in use" merely because a socket file was left by a process
   that no longer holds the endpoint lock. It MUST NOT use a connect probe
   or process check to infer liveness.
4. Cleanup MUST remove the socket only while the same `Listener` still holds
   the endpoint lock. A failed competing bind or its cleanup MUST NOT remove
   another listener's socket. This endpoint lock is distinct from
   `<instance-root>/daemon.lock`: the runtime lock serializes one application
   instance before stores open, while the endpoint lock protects an arbitrary
   overridden UDS path shared by otherwise different instance roots. The
   endpoint-lock file is never deleted; releasing the OS lock is sufficient
   and avoids a remove/recreate inode race.

Points 2, 3 and 4 are decided in this document
([ADR-TRN-0006](architecture.md)); the sc-runtime design states only that
file permissions restrict access to the user.

`<instance-root>` is the per-application, per-user directory resolved by
sc-transport, or an explicitly supplied path. On Unix the default base is
non-empty `XDG_RUNTIME_DIR`, otherwise `$HOME/.local/state`. On Windows it
is `LOCALAPPDATA`. The path is `<base>/<app>`
([REQ-TRN-0002](requirements.md)). `instance_root` does not create it.

Removing the old file and setting the permissions happen once, at bind time.
They are outside the request path, so they do not conflict with the rule
that request handling never blocks ([NFR-RUN-0002](../requirements.md)).

The socket file mode is `0o600`. The mode is set on the socket file only.
The instance-root directory is left unchanged. When the existing path is not
a socket, `bind` returns `TransportError::NotASocket`
(`TRANSPORT.NOT_A_SOCKET`) and leaves the path in place.

### Rationale

File permissions are the only access control on the default transport, so a
socket that other users can open would expose the daemon to them. A leftover
socket file would make every restart after a crash fail with "address in
use". Replacing it is safe only after acquiring ownership scoped to the
endpoint itself. An instance-root lock is insufficient because two distinct
instance roots may resolve to the same overridden socket path.

### Success Criteria

1. A test (`#[cfg(unix)]`, `--features server`) binds a UDS endpoint in a
   tempdir, reads the socket file's metadata, and asserts
   `mode & 0o077 == 0`.
2. A test (`#[cfg(unix)]`, `--features server`) creates a stale socket file
   by binding a `std::os::unix::net::UnixListener` at a tempdir path and
   dropping it without removing the file, then calls the crate's bind on the
   same path and asserts `Ok`.
3. The same test then serves a router on the new listener and asserts a
   request over the socket gets status 200.
4. A first listener binds a UDS path. A second bind of that same path fails
   without unlinking it; a request still reaches the first listener. Dropping
   the failed result cannot remove the first listener's socket.
5. `cargo doc -p sc-transport --features server` states the endpoint-lock
   path, retention lifetime, stale-file replacement, and cleanup rule.
6. Inspection finds no connect probe or process check before stale-file
   removal, and proves the endpoint lock is acquired first and retained by
   `Listener`; no path deletes the endpoint-lock file.

---

## REQ-TRN-0005: Typed HTTP client `Client` built on reqwest

**Status:** Active  

### Requirement Statement

The crate `sc-transport` MUST expose, in its default build (no cargo
features), a public HTTP client type. In the design sketch it is used as:

```rust
let client = sc_transport::Client::new(endpoint)?;
let widget: Envelope<Widget> =
    client.post("/ops/widget.create", &CreateWidget { name }).await?;
```

1. `Client::new(endpoint)` MUST take an endpoint (a UDS path or a TCP
   address) and nothing else. It does not take an application name.
   `DaemonNotRunning` carries no suggested action
   ([REQ-TRN-0006](requirements.md)). The endpoint argument is the value
   returned by the endpoint resolver of [REQ-TRN-0001](requirements.md).
   `Client::new` MUST NOT perform any endpoint resolution of its own: it
   MUST NOT read `SC_ENDPOINT`, configuration or the instance root.
2. It MUST build a `reqwest::Client` (reqwest 0.13). For a UDS endpoint it
   MUST configure it with `reqwest::ClientBuilder::unix_socket(path)`. For a
   TCP endpoint it MUST use the base URL `http://<ip>:<port>`. The UDS base
   URL is `http://localhost`.
3. `get(path)` MUST send an HTTP `GET` to `path` on that endpoint.
4. `post(path, &body)` MUST send an HTTP `POST` to `path` with `body`
   serialised as JSON, for any `B: serde::Serialize`.
5. Both methods MUST be `async`, MUST deserialise the JSON response body
   into the caller-chosen type `T`, for any `T: serde::de::DeserializeOwned`,
   and MUST return `Result<T, TransportError>`. Deserialisation runs for
   every HTTP status, including non-2xx. A body that is `T` is `Ok(T)`.
6. The crate MUST NOT name or depend on any concrete response type such as
   `Envelope<T>`; the caller chooses `T`.
7. No method may panic; every failure is a `TransportError` value.

The constructor signature is
`Client::new(endpoint: Endpoint) -> Result<Client, TransportError>`.
It stores the endpoint and builds the reqwest client. It does not contact
the daemon. An unreachable daemon is first reported by `get` or `post`.
Connect timeout and request timeout are each two seconds. The caller cannot
set them in v0.1. [ADR-TRN-0001](architecture.md) reserves a future
`#[non_exhaustive] ClientOptions` and `Client::with_options` for timeouts
and default headers. v0.1 does not define that type. `Client::new` stays the
handoff constructor. On a non-Unix target, `Client::new` with a UDS endpoint
returns `TransportError::Request`. The resolver may still return that UDS
endpoint.

### Rationale

This is the whole client a CLI needs: one constructor call and one call per
operation. Building on `reqwest` gives one well-known HTTP client for both
transports with no extra crate. Being generic over `T` keeps `sc-transport`
free of any dependency on `sc-command`, where the response envelope lives;
the two crates must stay independent so either can be used alone
([ADR-TRN-0003](architecture.md)).

### Success Criteria

1. A test on Unix starts an HTTP server on a UDS path in a tempdir with a
   `POST` route that echoes a JSON struct, calls `Client::new` with that
   endpoint, calls `post` with a test struct
   `Req { name: String }`, and asserts the
   returned test struct `Resp` has the expected field values.
2. The same test over TCP on `127.0.0.1` with an ephemeral port.
3. A test calls `get` on a route that returns a fixed JSON document and
   asserts the deserialised value, over UDS and over TCP.
4. A test calls `get::<Resp>` on a route that returns the body `not json`
   and asserts an `Err` whose variant is not `DaemonNotRunning`.
5. Inspection of `crates/sc-transport/src` finds a call to
   `ClientBuilder::unix_socket` and no mention of `Envelope` or `OpError`.
6. The client compiles with default features: `cargo build -p sc-transport`
   (no `--features`) succeeds, and `cargo doc -p sc-transport` for that
   build lists the client type with its `new`, `get` and `post` methods.

---

## REQ-TRN-0006: Typed error `DaemonNotRunning` (`DAEMON.NOT_RUNNING`)

**Status:** Active  

### Requirement Statement

The error enum `TransportError` of the crate `sc-transport` MUST have a
variant `DaemonNotRunning`.

1. The client ([REQ-TRN-0005](requirements.md): `Client::new`, `get`,
   `post`) MUST return `TransportError::DaemonNotRunning` from the first
   `get` or `post` when it cannot open
   a connection to the endpoint for one of these reasons: the UDS socket
   file does not exist; the UDS socket file exists but nothing is listening
   (connection refused); the TCP connection is refused.
2. `DaemonNotRunning` MUST carry `code: &'static str`, exactly
   `DAEMON.NOT_RUNNING`, the endpoint, and the cause. `code()` returns
   `DAEMON.NOT_RUNNING`. The variant has no `suggested_action`, no
   `AppName`, no `sc_observability_types::ErrorCode`, and no `Remediation`.
   This crate does not name `daemon start`. a-6 maps the string to
   `ErrorCode` and owns `suggested_action` for every `TransportError`.
3. The `Client` MUST NOT start the daemon, spawn any process, or retry in
   order to wait for a daemon. Phase A's generated CLI is report-only.
   Deferred [REQ-RUN-0206](../requirements.md) preserves a possible later
   optional auto-start layer outside this transport boundary.
4. A response body that cannot be deserialised into the requested type MUST
   produce `TransportError::ResponseDecode` (`TRANSPORT.RESPONSE_DECODE`),
   which carries the HTTP status, the cause, and `body_preview` (a lossy
   UTF-8 prefix of at most 512 bytes of the body). A body that deserialises
   as `T` is `Ok(T)` for every HTTP status.
5. Connect timeout and request timeout are each two seconds and are not
   caller-configurable in v0.1. The extension point is `ClientOptions` in
   [ADR-TRN-0001](architecture.md). A timeout is `TransportError::TimedOut`
   (`TRANSPORT.TIMED_OUT`).
6. A client failure that is neither a missing socket, nor connection
   refused, nor a timeout, nor a decode failure is `TransportError::Request`
   (`TRANSPORT.REQUEST`). A UDS path that is a directory is `Request`.

The inventory of every `TransportError` variant is the error-inventory
table in [ADR-TRN-0001](architecture.md). [ADR-TRN-0003](architecture.md)
records the same table as the decision for `DaemonNotRunning`.

### Rationale

The daemon being down is the most common CLI failure. It must be told apart
from every other failure so that an agent or a person can branch on the
stable code `DAEMON.NOT_RUNNING`. The `Client` only reports; any later
optional auto-start layer under deferred
[REQ-RUN-0206](../requirements.md) would act outside this boundary. The
code is a `&'static str` because this crate must not depend on
`sc-observability-types` or on `sc-command`, where `ErrorCode` and `OpError`
live. a-6 maps the string and chooses `suggested_action`, including any
`daemon start` text. Which envelope a CLI prints for the other
`TransportError` variants is recorded in
[REQ-RUN-0202](../requirements.md).

### Success Criteria

1. A test on Unix constructs the client against a UDS path in a tempdir where
   no file exists and asserts the first `Err` returned by `get` is
   `TransportError::DaemonNotRunning`.
2. A test on Unix does the same against a stale socket file (a listener
   bound and dropped without removing the file) and asserts
   `DaemonNotRunning`.
3. A test binds `127.0.0.1:0`, records the port, closes the listener, calls
   the client against that port, and asserts `DaemonNotRunning`.
4. Each of tests 1 to 3 asserts the code equals `DAEMON.NOT_RUNNING`, the
   endpoint equals the endpoint passed to `Client::new`, and the cause is
   present. The matched fields are `code`, `endpoint`, and `cause`.
5. A same-crate unit test matches `DaemonNotRunning` exhaustively on
   `code`, `endpoint`, and `cause`. The error carries no application name.
   `Client::new` takes an `Endpoint` only.
6. A test asserts that an undecodable response body gives
   `ResponseDecode` whose `body_preview` contains that body, and that a
   non-2xx body that deserialises as `T` is `Ok(T)`.
7. Inspection of `crates/sc-transport/src` finds no use of
   `std::process::Command` or `tokio::process`.
8. A test calls the client against a route that never responds and asserts
   `TimedOut`.

---

## NFR-TRN-0001: `sc-transport` links axum only behind `server`

**Status:** Active  

### Requirement Statement

In `crates/sc-transport/Cargo.toml`:

1. There MUST be a cargo feature named `server`.
2. `server` MUST NOT be in the `default` feature list.
3. `axum` MUST be declared `optional = true` and MUST be enabled only by the
   `server` feature.
4. With default features the direct non-dev dependencies MUST be exactly
   `tokio`, `reqwest`, `serde`, and `serde_json`. `tokio` and `reqwest` are
   stated by the sc-runtime design. `serde` follows from the crate's own API:
   the client methods are bounded by `serde::Serialize` and
   `serde::de::DeserializeOwned` ([REQ-TRN-0005](requirements.md)) and the
   configuration type implements `serde::Deserialize`
   ([REQ-TRN-0001](requirements.md)). `serde_json` decodes a buffered body so
   `ResponseDecode` can keep `body_preview`. The list does not name
   `sc-observability-types`. `allowed_dependencies` does not include it.
   `server` still adds only `axum`.
5. With the `server` feature the direct non-dev dependencies MUST be exactly
   those four plus `axum`.
6. The default build MUST NOT have `axum` anywhere in its normal dependency
   tree.
7. All listener-binding code MUST be compiled only under
   `#[cfg(feature = "server")]`.

This item is the owner of the list of third-party crates `sc-transport` may
depend on. [REQ-RUN-0005](../requirements.md) owns the workspace-internal
and forbidden edges and refers here for the third-party list.

The crate MUST build and pass its tests both with and without the feature.
`just test` MUST run both `cargo test -p sc-transport` and
`cargo test -p sc-transport --features server`
([REQ-RUN-0004](../requirements.md)), because `cargo test --workspace`
alone builds this crate only with `server` enabled.

### Rationale

The CLI of a generated project depends on this crate for its client, and a
CLI binary must link none of axum, rmcp or sqlx
([NFR-RUN-0001](../requirements.md)): linking the daemon's stack would
multiply its build time and binary size. Listener binding needs axum, so it
is opt-in. Cargo unifies features across one build, so under
`cargo build --workspace` this crate is compiled with `server` for every
member; whether such builds are exempt or the crate must be split is an
OPEN owned by [NFR-RUN-0001](../requirements.md). The criteria below are
therefore all run with `-p sc-transport`.

### Success Criteria

1. `cargo tree -p sc-transport -e normal --prefix none` prints no line
   beginning with `axum ` (the name followed by a space).
2. `cargo tree -p sc-transport -e normal --prefix none --features server`
   prints a line beginning with `axum `.
3. `cargo test -p sc-transport` passes, run by `just test`.
4. `cargo test -p sc-transport --features server` passes, run by
   `just test`.
5. `Cargo.toml` inspection: `default` does not list `server`; `axum` has
   `optional = true`.
6. `cargo tree -p sc-transport -e normal --depth 1 --prefix none` prints,
   after the first line (the crate itself), exactly four lines, beginning
   with `reqwest `, `serde `, `serde_json `, and `tokio `.
7. The same command with `--features server` prints, after the first line,
   exactly five lines, beginning with `axum `, `reqwest `, `serde `,
   `serde_json `, and `tokio `.

---

## NFR-TRN-0002: No `hyperlocal`, custom connector or hand-written HTTP

**Status:** Active  

### Requirement Statement

The crate `sc-transport` MUST make every client HTTP request, over UDS and
over TCP, through `reqwest` 0.13. For UDS it MUST use
`reqwest::ClientBuilder::unix_socket(path)`.

The crate MUST NOT depend on `hyperlocal`. It MUST NOT implement its own
hyper or tower connector (for example a type implementing
`tower::Service<Uri>` or hyper-util's `Connect`). It MUST NOT write or parse
HTTP messages on a raw `UnixStream` or `TcpStream`.

### Rationale

Since reqwest 0.13, `ClientBuilder::unix_socket` sends HTTP over a Unix
socket with no extra crate. An earlier draft of the project assumed
`hyperlocal` or a custom connector was needed; that was factually wrong.
Standard crates used the documented way mean less code to own and review.

### Success Criteria

1. `cargo tree -p sc-transport -e normal --prefix none` and the same command
   with `--features server` each print no line beginning with `hyperlocal `
   (the name followed by a space).
2. `crates/sc-transport/Cargo.toml` declares `reqwest` with version
   requirement `0.13`.
3. Inspection of `crates/sc-transport/src` finds a call to `unix_socket(`,
   no `impl` of a hyper or tower connector trait, and no code that writes an
   HTTP request line (such as `"GET "` or `"HTTP/1.1"`) to a stream.

---

## NFR-TRN-0003: `sc-transport` depends on no other workspace crate

**Status:** Active  

### Requirement Statement

`crates/sc-transport/Cargo.toml` MUST NOT list any of these crates
under `[dependencies]`, `[dev-dependencies]`, `[build-dependencies]` or any
target-specific dependency table, with any feature set:

| Forbidden crate | What it is |
|---|---|
| `sc-config` | library crate of this workspace |
| `sc-command` | library crate of this workspace |
| `sc-runtime` | library crate of this workspace |
| `sc-runtime-db-sqlite` | rusqlite write actor |
| `sc-cron` | SQLite-backed cron |
| `sc-observability` | the SC logging and telemetry crate |
| `sc-observability-otlp` | its OpenTelemetry export crate |
| `sc-observability-types` | its error-code crate |

The list is owned by [REQ-RUN-0005](../requirements.md), which states the
workspace-internal and forbidden edges of all four crates; this item applies
it to `sc-transport` and MUST stay identical to it.

The crate's boundary manifest under the repository directory
`boundaries/sc-transport/` is a TOML file. Sprint a-3 lints it as a
single-crate workspace (`sc-lint lint sc-boundary --root crates/sc-transport`).
sc-lint 0.8.0 lists a `forbidden_edges` target only when that target is a
workspace member of the lint tree, and it exits 3 for a name absent from
the workspace and from the cargo resolve
([REQ-RUN-0005](../requirements.md)). The root workspace includes
`sc-runtime-db-sqlite`, so `boundaries/sc-transport/sc-transport.toml` lists
`sc-transport -> sc-runtime-db-sqlite`. Other workspace-member edges from the
table above are listed when those targets are members of the lint tree.
`sc-cron` is not a member during sprint `b-1`, so it is not a
`forbidden_edges` entry yet. Crates.io names stay on the `cargo tree` check:
no line begins with `sc-config `, `sc-command `, `sc-runtime `,
`sc-runtime-db-sqlite `, `sc-cron `, `sc-observability `,
`sc-observability-types `, or `sc-observability-otlp ` (each name followed
by a space).

`allowed_dependents` is `sc-runtime`. a-5 added that name once the
four-crate workspace existed. Naming it before it was a workspace member
would make sc-lint 0.8.0 exit 3.

`allowed_dependencies` for the default build is `tokio`, `reqwest`, `serde`,
and `serde_json`. `axum` is absent from that list because the manifest has no
feature key; [NFR-TRN-0001](requirements.md) checks it with `cargo tree`.
`sc-lint-boundary` checks inter-crate edges and the public facade list only.

### Rationale

Each of the four crates must build and test on its own and be usable without
the other three or the template ([REQ-RUN-0002](../requirements.md)), so any
of them can be used by programs that are not sc-runtime daemons or move to
its own repository later. The three standalone crates `sc-config`,
`sc-transport` and `sc-command` have no dependency edges among them; only
`sc-runtime` knows the others ([ADR-RUN-0003](../architecture.md)).
`sc-observability` is forbidden because observability is not wrapped: a
project's `main.rs` initialises it directly and no library crate of this
workspace depends on it ([ADR-RUN-0004](../architecture.md)).

### Success Criteria

1. `cargo tree -p sc-transport -e normal --prefix none` and the same command
   with `--features server` each print no line beginning with `sc-config `,
   `sc-command `, `sc-runtime `, `sc-runtime-db-sqlite `, `sc-cron `,
   `sc-observability `, `sc-observability-types `, or
   `sc-observability-otlp ` (each name followed by a space).
2. Inspection of `crates/sc-transport/Cargo.toml` finds none of those names
   as a key in `[dependencies]`, `[dev-dependencies]`,
   `[build-dependencies]`, or a `[target.*]` dependency table.
3. A TOML manifest file exists under `boundaries/sc-transport/`. Its
   `forbidden_edges` list records the workspace-member edges
   [REQ-RUN-0005](../requirements.md) requires for `sc-transport` that are
   members of the lint tree, including `sc-transport -> sc-runtime-db-sqlite`.
   `sc-cron` is omitted until it is a workspace member, because sc-lint 0.8.0
   exits 3 for a target absent from the workspace. Its `allowed_dependents`
   list is `sc-runtime`. Its `allowed_dependencies` are `tokio`, `reqwest`,
   `serde`, and `serde_json`.
4. `sc-lint lint sc-boundary --root crates/sc-transport` exits 0. Root
   `just lint` across all four manifests stays with a-5.
5. `cargo test -p sc-transport` passes from a clean checkout.

---

## NFR-TRN-0004: `sc-transport` returns `TransportError`, never panics

**Status:** Active  

### Requirement Statement

Every public function and method of the crate `sc-transport` that can fail
MUST return `Result<T, TransportError>`, where `TransportError` is one public
`enum` defined in this crate. Public functions return that enum only.
`TransportCause` is the `cause` payload and the `Error::source` value. It
is not a function's `Err` type.

The rule is owned by [NFR-RUN-0009](../requirements.md), which applies to
every library crate of the workspace; this item applies it to
`sc-transport` and adds no exception. The sc-runtime design states the rule
for `sc-config` only; applying it to `sc-transport` is decided in this
document. Non-test code of `sc-transport` MUST
NOT use any of the following where it is reachable from a public function:
`unwrap`, `expect`, `panic!`, `unreachable!`, `todo!`, `unimplemented!`,
panicking `[]` indexing. A comment does not make such a use acceptable.

No public function or method may panic because of caller input (for example
an unparseable endpoint string or a missing path) or because of environment
state (for example a missing directory, a refused connection or an
unreadable environment variable).

Public signatures MUST NOT use opaque error types such as `anyhow::Error` or
`Box<dyn std::error::Error>`.

Every variant, its stable code, its cause payload, and its recovery are the
error-inventory table in [ADR-TRN-0001](architecture.md). The variants are
`InvalidName`, `InvalidEndpoint`, `MissingTcpPort`,
`InstanceRootUnavailable`, `DaemonNotRunning`, `ResponseDecode`, `TimedOut`,
`Request`, `Bind`, `EndpointBusy`, and `NotASocket`.

### Rationale

The crate runs inside long-lived daemons and inside CLIs driven by agents. A
panic there is an outage, or a failure no caller can parse. A typed enum lets
callers branch on the variant, and lets a generated `main.rs` turn errors
into exit codes. The same rule applies to every library crate in the
workspace: public APIs return `Result<T, E>` with one typed error enum per
crate ([ADR-RUN-0006](../architecture.md)).

### Success Criteria

1. Inspection of the public API (`cargo doc -p sc-transport --all-features`)
   shows every fallible function returns `Result<_, TransportError>` and no
   signature mentions `anyhow` or `Box<dyn Error>`.
2. Inspection of non-test code in `crates/sc-transport/src` finds none of
   `unwrap`, `expect`, `panic!`, `unreachable!`, `todo!`, `unimplemented!`,
   panicking `[]` indexing, reachable from a public function.
3. For each `TransportError` variant there is at least one test that
   triggers it and matches on that variant.
4. A test passes an unparseable endpoint value to the resolver and asserts
   `Err`; a test binds in a directory that does not exist and asserts `Err`.
