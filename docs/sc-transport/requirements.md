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
**Version:** 1.0.0  
**Created:** 2026-09-19  
**Last Updated:** 2026-11-12
**Owner:** randlee  

### Requirement Statement

Definitions: CLI means command-line interface. UDS means Unix domain socket. TCP means Transmission Control Protocol. ADR means architecture decision record. The shared configuration type is TransportConfig, carrying endpoint and instance_root as described below. "In that case" refers to a source value that cannot be parsed as an endpoint. A daemon binary is an application server executable. The endpoint resolver determines the Unix domain socket path or TCP socket address at which that daemon listens. The explicit-input form is `resolve_endpoint`; the convenience form is `resolve_endpoint_from_env`. The source table is headed Order, Source, How the value reaches the resolver. The platform table is headed Platform, Default endpoint. An endpoint is an `Endpoint::Uds(PathBuf)` or `Endpoint::Tcp(SocketAddr)` value. A source supplies a value when the caller passes a value for it rather than "absent"; the platform default always supplies a value. The first supplying source is the lowest-numbered source in the source table that supplies a value.

In the crate's public-surface sketch these are `Endpoint::Uds(PathBuf)` and `Endpoint::Tcp(SocketAddr)`, produced by `resolve_endpoint(...)`.

This precedence order is decided in this document; the sc-runtime design lists the sources (`--endpoint`, `SC_ENDPOINT`, config, platform default) but not their order.

| Order | Source | How the value reaches the resolver |
|---|---|---|
| 1 | the `--endpoint` command-line flag | The calling binary parses the flag (the crate MUST NOT depend on `clap` or parse `argv`) and passes the value, or "absent", as an argument. |
| 2 | the `SC_ENDPOINT` environment variable | The value of the process environment variable named exactly `SC_ENDPOINT`, passed to the resolver as an explicit argument (the value, or "absent"). |
| 3 | the endpoint from the caller's configuration | The caller passes it as an argument. The crate MUST NOT depend on `sc-config`. |
| 4 | the platform default | Computed by the crate, see below. |

The crate MAY also offer a convenience form of the resolver that reads the variable itself. Both points are decided in this document; the sc-runtime design names `SC_ENDPOINT` as a source but does not say who reads it.

| Platform | Default endpoint |
|---|---|
| macOS and Linux | UDS at `<instance-root>/daemon.sock` |
| Windows | TCP on IP address `127.0.0.1`, with the port taken from the caller's configuration |

`<instance-root>` is the per-application, per-user directory resolved by
sc-transport, or an explicitly supplied path. On Unix the default base is
non-empty `XDG_RUNTIME_DIR`, otherwise `$HOME/.local/state`. On Windows it
is `LOCALAPPDATA`. The path is `<base>/<app>`
([REQ-TRN-0002](requirements.md)). `instance_root` does not create it.

The Windows address `127.0.0.1` is stated by the sc-runtime design. That the Windows port comes from the caller's configuration is decided in this document: the design states "port from config" for cross-host TCP and names no port for the Windows default.

`DaemonConfig` embeds this type ([REQ-RT-0001](../sc-runtime/requirements.md), [REQ-RT-0008](../sc-runtime/requirements.md)); a CLI's configuration uses it directly. The resolver takes source 3 and the explicit instance root from a value of this type or from its fields.

Any code that needs that string, including `sc_runtime::testing::DaemonFixture` ([REQ-RT-0006](../sc-runtime/requirements.md)), obtains it from this conversion and does not format it itself. This follows from REQ-RT-0006 and [REQ-RT-0008](../sc-runtime/requirements.md), which bar `sc-runtime` from holding endpoint logic. The conversion is `Display` and `FromStr` on `Endpoint`: `uds:<path>` and `tcp:<socket-addr>`. `tcp:` takes a literal `SocketAddr` and does not resolve DNS. An empty string is `TransportError::InvalidEndpoint` and does not fall through.

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
  when they are used (`TransportError::Request` and
  `TransportError::UnsupportedEndpoint`).

Whether the project template writes a default TCP port into generated
configuration stays with [REQ-RUN-0301](../requirements.md). This crate does
not decide that.

**MUST statements:**

- The crate `sc-transport` (`crates/sc-transport`) MUST expose one public endpoint resolver.
- A daemon binary MUST call that resolver to learn where the daemon listens.
- A CLI binary MUST call that resolver to learn where the daemon listens.
- The resolver's result MUST be one of exactly two cases: a Unix domain socket (UDS) path, or a TCP socket address.
- The resolver MUST use the value of the first supplying source.
- The resolver MUST accept the `SC_ENDPOINT` value as an explicit input, so that a caller or a test can supply it without changing the process environment.
- The convenience form MUST read the variable with `std::env::var_os("SC_ENDPOINT")`.
- The convenience form MUST then call the explicit-input form.
- The platform default MUST be the default endpoint that the platform table gives for the running platform.
- The endpoint and instance-root portion of a project's configuration MUST be one public type exported by `sc-transport` with default features.
- The endpoint and instance-root configuration type that `sc-transport` exports MUST implement `serde::Deserialize`.
- A source value that cannot be parsed as an endpoint MUST produce an `Err` of the crate's error enum `TransportError`.
- The crate MUST also provide, with default features, the reverse conversion: from a resolved endpoint to the string form accepted by `--endpoint` and `SC_ENDPOINT`.
- Parsing the rendered string MUST yield an endpoint equal to the original.

**MUST NOT statements:**

- After a successful UDS bind, `bind` MUST NOT have changed the mode of any path other than the socket file to `0o600`.

- The convenience form MUST NOT use `std::env::var`.
- The convenience form MUST NOT use `std::env::vars`.
- The crate MUST NOT contain a literal TCP port number that it uses as a default; TCP ports come only from an override or from the caller's configuration ([ADR-TRN-0002](architecture.md)).
- This follows from two other binding facts: the CLI and the daemon must give the resolver the same configuration value, and a CLI MUST NOT depend on `sc-runtime`, so it cannot name `sc_runtime::DaemonConfig`.
- The resolver MUST NOT panic in that case.
- The resolver MUST NOT silently fall through to the next source in that case.

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
**Version:** 1.0.0  
**Created:** 2026-09-19  
**Last Updated:** 2026-09-19  
**Owner:** randlee  

### Requirement Statement

Definitions:

- UDS means Unix domain socket.
- The instance root is the one directory that holds a daemon's run-time files for one application and one operating-system user: the UDS socket file `daemon.sock` and the singleton lock file `daemon.lock`.
- An application name is an [`AppName`](architecture.md) value, such as `my-app`.
- `instance_root` is the public function `instance_root(&AppName) -> Result<PathBuf, TransportError>` of the crate `sc-transport` (`crates/sc-transport`).
- The per-user base is the directory that `instance_root` reads from the process environment at run time: on Unix non-empty `XDG_RUNTIME_DIR`, otherwise `$HOME/.local/state`; on Windows non-empty `LOCALAPPDATA`.
- The default instance root is the path `<base>/<app>`, where `<base>` is the per-user base and `<app>` is the application name.
- The path builder is the crate-private function `instance_root_from_base(&AppName, Option<&Path>)`; `instance_root` reads the per-user base and passes the per-user base to the path builder. The path builder is not a facade item.
- An explicit instance root is a directory path supplied by the caller, such as `TransportConfig.instance_root`.
- An instance-root consumer is a public function of `sc-transport` whose result depends on an instance root, such as `resolve_endpoint`.
- A fixed user-directory literal is a string literal that names a home-directory or temp-directory path; examples are paths starting with `/tmp`, `/home`, `/Users` or `C:\`.

Decided by sprint a-3:

- `instance_root` does not create the instance root directory; the caller creates the directory.
- An explicit instance root `R` is used unchanged as the Unix default endpoint's directory (`R/daemon.sock`), not as `R/<app>`.
- Failure is `TransportError::InstanceRootUnavailable`
  (`TRANSPORT.INSTANCE_ROOT_UNAVAILABLE`).

`AppName::new` accepts ASCII `A-Z`, `a-z`, `0-9`, `-`, and `_`, length
1 through 64 bytes. Rejection is `TransportError::InvalidName`
(`APP.INVALID_NAME`). `detail` says whether the name was empty, longer than
64 bytes, or which character was outside the accepted set. `AppName` exposes
`as_str` and `AsRef<str>` and does not implement `Deref`. Tests load
`tests/fixtures/app-name-vectors.json` and do not embed a second list.

**MUST statements:**

- The crate `sc-transport` MUST export the public function `instance_root`.
- `instance_root` MUST return the default instance root for the given application name and the current operating-system user.
- For one application name, `instance_root` MUST return different paths when called by two different operating-system users.
- For one operating-system user, `instance_root` MUST return different paths for two different application names.
- Two calls of `instance_root` with the same application name by the same operating-system user MUST return equal paths.
- `instance_root` MUST obtain the per-user base by calling `std::env::var_os`.
- Every instance-root consumer MUST have a parameter that carries an explicit instance root.
- `resolve_endpoint` MUST receive an explicit instance root in the field `TransportConfig.instance_root`.
- A public function of `sc-transport` that receives an explicit instance root `R` MUST use `R` unchanged in place of the default instance root.
- When no per-user base is available, `instance_root` MUST return `Err(TransportError::InstanceRootUnavailable)`.
- When the per-user base argument is `None`, the path builder MUST return `Err(TransportError::InstanceRootUnavailable)`.
- The source of `sc-transport` MUST build the default instance root only in the path builder.
- The path builder MUST receive the per-user base as an argument.

**MUST NOT statements:**

- Files under `crates/sc-transport/src` MUST NOT contain a fixed user-directory literal.
- `instance_root` MUST NOT panic.

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
**Version:** 1.0.0  
**Created:** 2026-09-19  
**Last Updated:** 2026-11-13
**Owner:** randlee  

### Requirement Statement

Definitions: CLI means command-line interface.
HTTP means Hypertext Transfer Protocol.
UDS means Unix domain socket.
TCP means Transmission Control Protocol.
A resolved endpoint is an Endpoint value containing a UDS path or TCP socket address.
bind is the public asynchronous sc-transport function taking &Endpoint and returning Result<Listener, TransportError>.
Listener is the sc-transport type owning one bound socket.
The router is the axum::Router passed to axum::serve.
A handler is an application function attached to a router route.
A transport switch is choosing a TCP listener instead of a UDS listener, or a UDS listener instead of a TCP listener.
Transport-dependent means varying according to whether the listener is UDS or TCP.
A wrapper is an additional Rust type or function adapting a router or handler.
Unmodified means no transport-dependent change to the application router or its handlers.
The server feature gate excludes the bind function and Listener type from builds without server.
A server build enables the sc-transport Cargo feature server.

The six router/handler portability constraints are checked by source inspection of the shared router-building function, its handlers and the two `axum::serve` call sites in the dual-listener conformance test.

`Listener` implements `axum::serve::Listener`. `Listener::local_endpoint()` reports the bound `Endpoint`, including the operating-system-assigned TCP port for a request using port `0`; failure to read the TCP address returns `TransportError::Bind`.

Bind errors, including an occupied address or missing parent directory, use `TransportError::Bind` (`TRANSPORT.BIND`). An already-held endpoint lock returns `TransportError::EndpointBusy` (`TRANSPORT.ENDPOINT_BUSY`). A UDS bind on non-Unix returns `TransportError::UnsupportedEndpoint` (`TRANSPORT.UNSUPPORTED_ENDPOINT`), with recovery directing the caller to a `tcp:` endpoint. Binding does not create the parent directory.

v0.1 `TransportConfig` describes exactly one endpoint and has no `additional_endpoints` field or reserved dual-bind value. A later dual-bind extension adds a field on `TransportConfig`. Calling `bind` twice is valid. Runtime use of two listeners belongs to [REQ-RT-0001](../sc-runtime/requirements.md).

**MUST statements:**

- When `server` is enabled, `sc-transport` MUST publicly export `bind`.
- When `axum::serve` (axum 0.8) serves a `Listener`, HTTP requests MUST reach the supplied unmodified `axum::Router`.
- On Unix, `bind` for a UDS endpoint MUST bind the supplied socket path.
- `bind` for a TCP endpoint MUST bind the supplied socket address.
- For a requested TCP port of `0`, `Listener::local_endpoint()` MUST return the address assigned by the operating system.
- One process MUST support simultaneous ownership of a UDS listener and a TCP listener.
- One process MUST be able to serve the same router concurrently on a UDS listener and a TCP listener.
- When socket binding fails, `sc_transport::bind` MUST return `Err(TransportError::Bind { .. })`.
- The `bind` export in `crates/sc-transport/src/lib.rs` MUST be guarded by `#[cfg(feature = "server")]`.
- The `Listener` export in `crates/sc-transport/src/lib.rs` MUST be guarded by `#[cfg(feature = "server")]`.

**MUST NOT statements:**

- The router MUST NOT require transport-dependent source changes.
- The router MUST NOT require a transport-dependent generic type parameter.
- The router MUST NOT require a transport-dependent wrapper.
- The handler MUST NOT require transport-dependent source changes.
- The handler MUST NOT require a transport-dependent generic type parameter.
- The handler MUST NOT require a transport-dependent wrapper.
- `sc_transport::bind` MUST NOT panic.

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
**Version:** 1.0.0  
**Created:** 2026-09-19  
**Last Updated:** 2026-11-12
**Owner:** randlee  

### Requirement Statement

Definitions: UDS is a Unix domain socket.
`bind` is the `sc-transport` listener-binding function for a UDS endpoint, enabled by the `server` Cargo feature.
The endpoint lock is a nonblocking operating-system exclusive lock on the file named by appending `.lock` to the complete socket path ([ADR-TRN-0006](architecture.md)).
`Listener` owns the bound socket and its endpoint lock.
A typed bind error is an `Err(TransportError)` returned by `bind`.
The process owner is the operating-system user running the binding process.
The endpoint path is the filesystem path selected for the UDS socket.
Socket access is permission to use the UDS socket through the filesystem.
A process check queries whether a process is alive.
Listener liveness is whether another listener is serving the existing socket.
Address in use is the operating-system error for a socket address already in use.
`TRANSPORT.NOT_A_SOCKET` is the stable code for `TransportError::NotASocket` in the transport error inventory.
Cleanup associated with a failed competing bind is cleanup of the failed bind attempt or result.
A stale socket is an existing socket file at the endpoint path after `bind` acquires the endpoint lock.
A competing bind is a bind attempt while another listener holds the endpoint lock.
The instance root is the per-application, per-user directory resolved or explicitly supplied through `sc-transport` ([REQ-TRN-0002](requirements.md)).
The default root is `<base>/<app>`, where Unix uses non-empty `XDG_RUNTIME_DIR` or `$HOME/.local/state` and Windows uses `LOCALAPPDATA`.
`instance_root` does not create the directory.

The endpoint lock is separate from `<instance-root>/daemon.lock`. The runtime instance lock serializes one application instance before stores open. Socket replacement and permission changes occur once at bind time, outside the request path ([NFR-RUN-0002](../requirements.md)). File permissions are the UDS listener's only access control ([ADR-TRN-0005](architecture.md)).

**MUST statements:**

- `bind` MUST leave the instance-root directory permission bits unchanged.
- During socket removal by `Listener` cleanup, that `Listener` MUST hold the endpoint lock.

- After a successful UDS bind, the socket file mode MUST be `0o600`.
- After a successful UDS bind, the socket file's group and other permission bits MUST be zero (`mode & 0o077 == 0`).
- Before inspecting an existing UDS socket file, `bind` MUST acquire the endpoint lock.
- Before removing an existing UDS socket file, `bind` MUST acquire the endpoint lock.
- Until the returned `Listener` is dropped, a second nonblocking exclusive lock attempt on the endpoint-lock file MUST fail.
- When a stale socket file exists at the endpoint path, `bind` MUST remove the stale socket file.
- After removing a stale socket, `bind` MUST bind the replacement at the same endpoint path.
- During stale-socket replacement, `bind` MUST retain the endpoint lock.
- The UDS socket file MUST be removed when its `Listener` is dropped.
- If endpoint-lock acquisition fails, `bind` MUST return `Err(TransportError)`.
- If an existing endpoint path is not a socket, `bind` MUST return `TransportError::NotASocket`.
- The `code()` method of `TransportError::NotASocket` MUST return `TRANSPORT.NOT_A_SOCKET`.
- If an existing endpoint path is not a socket, `bind` MUST leave the path in place.

**MUST NOT statements:**

- After a successful UDS `bind`, the socket file MUST NOT allow access by an operating-system user other than the process owner.

- When `bind` fails to acquire the endpoint lock, it MUST NOT inspect the socket file.
- After endpoint-lock acquisition fails, `bind` MUST NOT unlink the socket file.
- If endpoint-lock acquisition fails, `bind` MUST NOT replace the existing socket file.
- `bind` MUST NOT return an address-in-use error solely because an existing socket file has no endpoint-lock holder.
- The listener MUST NOT connect to an existing UDS socket to determine listener liveness.
- The listener MUST NOT use a process check to determine listener liveness.
- A failed competing bind MUST NOT remove another listener's socket.
- Cleanup associated with a failed competing bind MUST NOT remove another listener's socket.
- `sc-transport` MUST NOT delete the endpoint-lock file.


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
**Version:** 1.0.0  
**Created:** 2026-09-19  
**Last Updated:** 2026-11-12
**Owner:** randlee  

### Requirement Statement

Definitions: CLI means command-line interface. HTTP means Hypertext Transfer Protocol. UDS means Unix domain socket. TCP means Transmission Control Protocol. JSON means JavaScript Object Notation. ADR means architecture decision record. SC_ENDPOINT is the environment variable carrying endpoint text. An instance root is the per-application, per-user directory for daemon run-time files. Configuration means application settings from configuration files or environment variables; the caller resolves the endpoint before passing it to `Client::new`. `Client` is the public HTTP client type of `sc-transport`. The client's endpoint is the endpoint given to `Client::new`. A request to `path` is a request sent to the client's endpoint with `path` as its request path. The default build is a build with no cargo features. `Endpoint` is the endpoint type of [REQ-TRN-0001](requirements.md), either a UDS path or a TCP address. The client source is `crates/sc-transport/src`.

In the design sketch it is used as:

```rust
let client = sc_transport::Client::new(endpoint)?;
let widget: Envelope<Widget> =
    client.post("/ops/widget.create", &CreateWidget { name }).await?;
```

1. It does not take an application name. `DaemonNotRunning` carries no suggested action ([REQ-TRN-0006](requirements.md)). The endpoint argument is the value returned by the endpoint resolver of [REQ-TRN-0001](requirements.md).
2. The UDS base URL is `http://localhost`.
5. Deserialisation runs for every HTTP status, including non-2xx. A body that is `T` is `Ok(T)`.
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

The caller chooses `T`.

**MUST statements:**

- The crate `sc-transport` MUST expose `Client`.
- In the client source, the parameter list of `Client::new` MUST be exactly `(endpoint: Endpoint)`.
- `Client::new` MUST build a `reqwest::Client` (reqwest 0.13).
- For a UDS endpoint `Client::new` MUST configure the `reqwest::Client` with `reqwest::ClientBuilder::unix_socket(path)`.
- For a TCP endpoint, each request that `Client` sends MUST go to the URL `http://<ip>:<port>` followed by the request path, where `<ip>:<port>` is the endpoint's TCP address.
- `get(path)` MUST send its request to `path`.
- The request that `get(path)` sends MUST use the HTTP method `GET`.
- `post(path, &body)` MUST send its request to `path`.
- The request that `post(path, &body)` sends MUST use the HTTP method `POST`.
- The body of the request that `post(path, &body)` sends MUST be `body` serialised as JSON.
- In the client source, the signature of `post` MUST accept every `B: serde::Serialize` as the body type.
- `get(path)` MUST be `async`.
- In the client source, the signature of `get` MUST accept every `T: serde::de::DeserializeOwned` as the response type.
- `get(path)` MUST deserialise the JSON response body into `T`.
- In the client source, the return type of `get` MUST be `Result<T, TransportError>`.
- `post(path, &body)` MUST be `async`.
- In the client source, the signature of `post` MUST accept every `T: serde::de::DeserializeOwned` as the response type.
- `post(path, &body)` MUST deserialise the JSON response body into `T`.
- In the client source, the return type of `post` MUST be `Result<T, TransportError>`.

**MUST NOT statements:**

- In the client source, `Client` MUST NOT be behind any `#[cfg(feature = ...)]` attribute.
- `Client::new` MUST NOT perform any endpoint resolution of its own.
- `Client::new` MUST NOT read `SC_ENDPOINT`.
- `Client::new` MUST NOT read configuration.
- `Client::new` MUST NOT read the instance root.
- The client source MUST NOT contain the name of any concrete response type such as `Envelope<T>`.
- The crate `sc-transport` MUST NOT depend on any concrete response type such as `Envelope<T>`.

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
**Version:** 1.0.0  
**Created:** 2026-09-19  
**Last Updated:** 2026-11-12
**Owner:** randlee  

### Requirement Statement

Definitions: CLI means command-line interface. HTTP means Hypertext Transfer Protocol. UDS means Unix domain socket. TCP means Transmission Control Protocol. ADR means architecture decision record. An endpoint is the resolved Unix domain socket path or TCP socket address represented by `Endpoint` and passed to `Client::new` (REQ-TRN-0001 and REQ-TRN-0005). A connection failure is any one of three results of opening a connection to the endpoint: the UDS socket file does not exist; the UDS socket file exists and the connection is refused because nothing is listening; the TCP connection is refused. The connection error of a connection failure is the underlying error that prevented opening the endpoint. A connection attempt is one try to open a connection to the endpoint. The deserialisation error is the error returned when a response body fails to deserialise into the requested type. The body preview of a response body is its first 512 bytes, or the whole body when shorter, decoded as lossy UTF-8. The `Client` is the HTTP client of [REQ-TRN-0005](requirements.md), constructed by `Client::new` and called through `get` and `post`.

2. `code()` returns `DAEMON.NOT_RUNNING`. The variant has no `suggested_action`, no `AppName`, no `sc_observability_types::ErrorCode`, and no `Remediation`. This crate does not name `daemon start`. a-6 maps the string to `ErrorCode` and owns `suggested_action` for every `TransportError`.
3. Phase A's generated CLI is report-only. Deferred [REQ-RUN-0206](../requirements.md) preserves a possible later optional auto-start layer outside this transport boundary.
4. A body that deserialises as `T` is `Ok(T)` for every HTTP status.
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

**MUST statements:**

- The error enum `TransportError` of the crate `sc-transport` MUST have a variant `DaemonNotRunning`.
- A `get` or `post` call on the `Client` that meets a connection failure MUST return `Err(TransportError::DaemonNotRunning)`.
- The type of the field `DaemonNotRunning.code` declared in `crates/sc-transport/src` MUST be `&'static str`.
- When a `get` or `post` call returns `DaemonNotRunning`, `DaemonNotRunning.code` MUST equal `DAEMON.NOT_RUNNING`.
- When a `get` or `post` call returns `DaemonNotRunning`, `DaemonNotRunning.endpoint` MUST equal the endpoint value passed to `Client::new`.
- When a `get` or `post` call returns `DaemonNotRunning`, `DaemonNotRunning.cause` MUST hold the connection error of the connection failure.
- A `get` or `post` call whose response body cannot be deserialised into the requested type MUST return `Err(TransportError::ResponseDecode)`.
- When a `get` or `post` call returns `ResponseDecode`, `ResponseDecode.code()` MUST return `TRANSPORT.RESPONSE_DECODE`.
- When a `get` or `post` call returns `ResponseDecode`, `ResponseDecode.status` MUST equal the HTTP status of the response.
- When a `get` or `post` call returns `ResponseDecode`, `ResponseDecode.cause` MUST hold the deserialisation error.
- When a `get` or `post` call returns `ResponseDecode`, `ResponseDecode.body_preview` MUST equal the body preview of the response body.

**MUST NOT statements:**

- The `Client` MUST NOT start the daemon.
- The `Client` MUST NOT spawn any process.
- A `get` or `post` call MUST NOT make another connection attempt after a connection failure.

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
3. A test gives the production client's connection-failure handling a controlled connection-refused result for a TCP endpoint and asserts `TransportError::DaemonNotRunning`. The test does not connect to a port after releasing its listener. How the result is supplied is not bound here and adds no hyper or tower connector (NFR-TRN-0002).
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
8. Drive the production client's timeout handling with a test-controlled pending request and clock. Advance that clock to the configured request deadline and assert `TransportError::TimedOut`. Do not use wall-clock sleeps, load or event-loop ordering to trigger the deadline.

---

## NFR-TRN-0001: `sc-transport` links axum only behind `server`

**Status:** Active  
**Version:** 1.0.0  
**Created:** 2026-09-19  
**Last Updated:** 2026-09-19  
**Owner:** randlee  

### Requirement Statement

Definitions:

- The manifest is the file `crates/sc-transport/Cargo.toml`.
- The default dependency set of `sc-transport` is the set of crate names on the lines after the first line of the output of `cargo tree -p sc-transport -e normal --depth 1 --prefix none`.
- The server dependency set of `sc-transport` is the set of crate names on the lines after the first line of the output of `cargo tree -p sc-transport -e normal --depth 1 --prefix none --features server`.
- The default dependency tree of `sc-transport` is every package that `cargo tree -p sc-transport -e normal --prefix none` prints.
- Listener-binding code is the Rust code in `crates/sc-transport/src` that binds a UDS or TCP listener ([REQ-TRN-0003](requirements.md)).
- An item is gated by the `server` feature when the item, or a module that contains the item, carries `#[cfg(feature = "server")]`.
- A command succeeds when the command exits with status 0.
- `just test` is the test recipe of the repository `justfile` ([REQ-RUN-0004](../requirements.md)).

Context: this item owns the list of third-party crates `sc-transport` may depend on; [REQ-RUN-0005](../requirements.md) owns the workspace-internal and forbidden edges and refers here for the third-party list. `tokio` and `reqwest` are stated by the sc-runtime design. `serde` follows from the crate's API: the client methods are bounded by `serde::Serialize` and `serde::de::DeserializeOwned` ([REQ-TRN-0005](requirements.md)), and the configuration type implements `serde::Deserialize` ([REQ-TRN-0001](requirements.md)). `serde_json` decodes a buffered body so `ResponseDecode` can keep `body_preview`. `sc-observability-types` is not in the list and not in `allowed_dependencies`. `cargo test --workspace` alone builds `sc-transport` only with the `server` feature enabled.

**MUST statements:**

- The `[features]` table of the manifest MUST contain a key named `server`.
- The `axum` entry in the `[dependencies]` table of the manifest MUST contain `optional = true`.
- In the `[features]` table of the manifest, `dep:axum` MUST appear only in the `server` feature list.
- The default dependency set of `sc-transport` MUST equal the set {`reqwest`, `serde`, `serde_json`, `tokio`}.
- The server dependency set of `sc-transport` MUST equal the set {`axum`, `reqwest`, `serde`, `serde_json`, `tokio`}.
- Every item of listener-binding code MUST be gated by the `server` feature.
- `cargo build -p sc-transport --features server` MUST succeed.
- `cargo build -p sc-transport` MUST succeed.
- `cargo test -p sc-transport --features server` MUST succeed.
- `cargo test -p sc-transport` MUST succeed.
- `just test` MUST run the command `cargo test -p sc-transport`.
- `just test` MUST run the command `cargo test -p sc-transport --features server`.

**MUST NOT statements:**

- The `default` list in the `[features]` table of the manifest MUST NOT contain `server`.
- The default dependency tree of `sc-transport` MUST NOT contain a line beginning `axum ` (the name followed by a space).

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
**Version:** 1.0.0  
**Created:** 2026-09-19  
**Last Updated:** 2026-11-13
**Owner:** randlee  

### Requirement Statement

Definitions: HTTP means Hypertext Transfer Protocol. UDS means Unix domain socket. TCP means Transmission Control Protocol. The crate is `sc-transport`. `reqwest` is the HTTP client library declared with version requirement `>=0.13`. Hyper or tower connectors include types implementing `tower::Service<Uri>` or hyper-util's `Connect`. A raw stream is a `UnixStream` or `TcpStream` used directly to read or write HTTP messages.

**MUST statements:**

- The manifest `crates/sc-transport/Cargo.toml` MUST declare `reqwest` with version requirement `>=0.13`.

- Every client HTTP request that `sc-transport` sends over UDS MUST use `reqwest`.
- The crate `sc-transport` MUST make every client HTTP request over TCP through `reqwest`.
- The crate `sc-transport` MUST configure its UDS client with `reqwest::ClientBuilder::unix_socket(path)`.

**MUST NOT statements:**

- The `reqwest` requirement in `crates/sc-transport/Cargo.toml` MUST NOT declare an upper bound.

- The crate MUST NOT depend on `hyperlocal`.
- The crate `sc-transport` MUST NOT implement its own hyper connector.
- The crate `sc-transport` MUST NOT implement its own tower connector.
- The crate MUST NOT write HTTP messages on a raw `UnixStream`.
- The crate MUST NOT write HTTP messages on a raw `TcpStream`.
- The crate MUST NOT parse HTTP messages on a raw `UnixStream`.
- The crate MUST NOT parse HTTP messages on a raw `TcpStream`.

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
   requirement `>=0.13`.
3. Inspection of `crates/sc-transport/src` finds a call to `unix_socket(`,
   no `impl` of a hyper or tower connector trait, and no code that writes an
   HTTP request line (such as `"GET "` or `"HTTP/1.1"`) to a stream.

---

## NFR-TRN-0003: `sc-transport` depends on no other workspace crate

**Status:** Active  
**Version:** 1.0.0  
**Created:** 2026-09-19  
**Last Updated:** 2026-11-13  
**Owner:** randlee  

### Requirement Statement

Definitions: The forbidden crate table is the table in this Requirement Statement whose first column is headed Forbidden crate. A forbidden crate is a crate name in the first column of the forbidden crate table. A dependency table is the `[dependencies]`, `[dev-dependencies]` or `[build-dependencies]` table, or any `[target.*]` dependency table, of `crates/sc-transport/Cargo.toml`. A dependency table entry names a crate when the entry key or the entry `package` value is that crate name. The boundary manifest is the TOML file `boundaries/sc-transport/sc-transport.toml` read by `sc-lint-boundary`. The lint tree is the workspace that `sc-lint lint sc-boundary --root crates/sc-transport` reads. The `sc-transport` row of the `forbidden_edges` column is the cell in the `sc-transport` row and the column headed Forbidden edges (`forbidden_edges`) of the crate graph table in item 3 of REQ-RUN-0005; that cell is a list of crate names. A forbidden edge from `sc-transport` is a dependency `sc-transport -> <crate>` that REQ-RUN-0005 forbids, and `<crate>` is its target crate.

| Forbidden crate | What it is |
|---|---|
| `sc-config` | library crate of this workspace |
| `sc-command` | library crate of this workspace |
| `sc-runtime` | library crate of this workspace |
| `sc-runtime-db-sqlite` | rusqlite write actor |
| `sc-cron` | cron policy over application storage |
| `sc-cron-commands` | standard cron command adapter |
| `sc-storage` | shared typed storage contract |
| `sc-monty` | shared Monty interpreter |
| `sc-observability` | the SC logging and telemetry crate |
| `sc-observability-otlp` | the OpenTelemetry export crate of `sc-observability` |
| `sc-observability-types` | the error-code crate of `sc-observability` |

sc-lint 0.8.0 exits 3 for a `forbidden_edges` target absent from the lint tree ([REQ-RUN-0005](../requirements.md)), so the boundary manifest lists only forbidden crates that are lint-tree members; `cargo tree` checks every forbidden crate. `sc-lint-boundary` checks inter-crate edges and the public facade list only; [NFR-TRN-0001](requirements.md) checks `axum` with `cargo tree`.

**MUST statements:**

- The forbidden crate table MUST contain the same set of crate names as the `sc-transport` row of the `forbidden_edges` column in [REQ-RUN-0005](../requirements.md), where each name is the target crate of a forbidden edge from `sc-transport`.

**MUST NOT statements:**

- `crates/sc-transport/Cargo.toml` MUST NOT name a forbidden crate in `[dependencies]`, `[dev-dependencies]`, `[build-dependencies]` or any `[target.*]` dependency table.

### Rationale

Each public crate must build and test when selected on its own. `sc-transport`
remains usable without the other SDK crates or the template
([REQ-RUN-0002](../requirements.md)), so programs that are not sc-runtime
daemons can use it and it can move to its own repository later. The three standalone crates `sc-config`,
`sc-transport` and `sc-command` have no dependency edges among them; the required assembly and shared-storage edges are owned by REQ-RUN-0005 ([ADR-RUN-0003](../architecture.md)).
`sc-observability` is forbidden because observability is not wrapped: a
project's `main.rs` initialises it directly and no library crate of this
workspace depends on it ([ADR-RUN-0004](../architecture.md)).

### Success Criteria

1. `cargo tree -p sc-transport -e normal --prefix none` and the same command
   with `--features server` each print no line beginning with `sc-config `,
   `sc-command `, `sc-runtime `, `sc-runtime-db-sqlite `, `sc-cron `,
   `sc-cron-commands `, `sc-storage `, `sc-monty `, `sc-observability `, `sc-observability-types `, or
   `sc-observability-otlp ` (each name followed by a space).
2. Inspection of `crates/sc-transport/Cargo.toml` finds none of those names
   as a key in `[dependencies]`, `[dev-dependencies]`,
   `[build-dependencies]`, or a `[target.*]` dependency table.
3. A TOML manifest file exists under `boundaries/sc-transport/`. Its
   `forbidden_edges` list records the workspace-member edges
   [REQ-RUN-0005](../requirements.md) requires for `sc-transport` that are
   members of the lint tree, including `sc-transport -> sc-runtime-db-sqlite`.
   All other public-package workspace targets are included when present in
   the lint tree; historical incomplete trees omit absent targets because
   sc-lint 0.8.0 exits 3 for a target absent from the workspace. Its `allowed_dependents`
   list is `sc-runtime`. Its `allowed_dependencies` are `tokio`, `reqwest`,
   `serde`, and `serde_json`.
4. `sc-lint lint sc-boundary --root crates/sc-transport` exits 0. Root
   `just lint` checks the public-package manifests under REQ-RUN-0005.
5. `cargo test -p sc-transport` passes from a clean checkout.

---

## NFR-TRN-0004: `sc-transport` returns `TransportError`, never panics

**Status:** Active  
**Version:** 1.0.0  
**Created:** 2026-09-19  
**Last Updated:** 2026-09-19  
**Owner:** randlee  

### Requirement Statement

Definitions: `TransportError` is the public `enum` of `sc-transport` whose variants, stable codes, cause payloads and recoveries are the error-inventory table in [ADR-TRN-0001](architecture.md): `InvalidName`, `InvalidEndpoint`, `MissingTcpPort`, `InstanceRootUnavailable`, `DaemonNotRunning`, `ResponseDecode`, `TimedOut`, `Request`, `Bind`, `EndpointBusy` and `NotASocket`. `TransportCause` is the type of the `cause` field of `TransportError` and of the value `Error::source` returns. A public function is a `pub fn` or `pub` method reachable from the `sc-transport` crate root. The error type of a function is `E` in its return type `Result<T, E>`. Reachable code is every function body in `crates/sc-transport/src`, outside items marked `#[cfg(test)]`, that a public function calls directly or through other functions. An opaque error type is an error type whose variants a caller cannot match on, including `anyhow::Error` and `Box<dyn std::error::Error>`. The arguments and environment states under which a public function cannot return its `Ok` value include an unparseable endpoint string, a missing path, a missing directory, a refused connection and an unreadable environment variable.

The rule is owned by [NFR-RUN-0009](../requirements.md), which applies to every library crate of the workspace; this record applies the rule to `sc-transport` with no exception. The sc-runtime design states the rule for `sc-config` only; this document decides its application to `sc-transport`. A comment beside a call or expression does not exempt the call or expression from a MUST NOT statement.

**MUST statements:**

- In `crates/sc-transport/src`, the count of public `enum` definitions named `TransportError` MUST equal 1.
- The error type of each public function of `sc-transport` MUST be `TransportError`.
- When a public function of `sc-transport` is called with an argument or in an environment state under which it cannot return its `Ok` value, the call MUST return `Err`.

**MUST NOT statements:**

- Reachable code of `sc-transport` MUST NOT call `unwrap`.
- Reachable code of `sc-transport` MUST NOT call `expect`.
- Reachable code of `sc-transport` MUST NOT invoke `panic!`.
- Reachable code of `sc-transport` MUST NOT invoke `unreachable!`.
- Reachable code of `sc-transport` MUST NOT invoke `todo!`.
- Reachable code of `sc-transport` MUST NOT invoke `unimplemented!`.
- Reachable code of `sc-transport` MUST NOT use panicking `[]` indexing.
- In `crates/sc-transport/src`, the signature of a public function MUST NOT contain an opaque error type.

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
