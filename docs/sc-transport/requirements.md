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
[`../sc-runtime-design.md`](../sc-runtime-design.md). Entries follow the
shared SC requirement and ADR templates: the obligation, then **Why** and **Verified by**. Every id is binding and is
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
| 2 | the `SC_ENDPOINT` environment variable | The value of the process environment variable named exactly `SC_ENDPOINT`. |
| 3 | the endpoint from the caller's configuration | The caller passes it as an argument. The crate MUST NOT depend on `sc-config`. |
| 4 | the platform default | Computed by the crate, see below. |

The platform default MUST be:

| Platform | Default endpoint |
|---|---|
| macOS and Linux | UDS at `<instance-root>/daemon.sock`, where `<instance-root>` is the per-app, per-user directory of [REQ-TRN-0002](requirements.md) |
| Windows | TCP on IP address `127.0.0.1`, with the port taken from the caller's configuration |

A source value that cannot be parsed as an endpoint MUST produce an `Err` of
the crate's error enum `TransportError`. The resolver MUST NOT panic and MUST
NOT silently fall through to the next source in that case.

**OPEN:** The names `Endpoint`, `Uds`, `Tcp`, `resolve_endpoint` and the
resolver's parameter list are illustrative until the contract sprint pins
them. This includes whether the resolver reads `SC_ENDPOINT` itself or
receives its value from the caller, and the name of the `TransportError`
variant for an unparseable value.

**OPEN:** The string syntax of an endpoint value in `--endpoint` and
`SC_ENDPOINT` (how a UDS path is told apart from a TCP `host:port`, and
whether an empty value counts as absent) is not decided.

**OPEN:** The type and shape of the configuration value (a full endpoint, a
port only, or both) is not decided, and neither is the Windows default result
when the configuration supplies no port.

**OPEN:** The behaviour when an override or configuration names a UDS path on
Windows is not decided.

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
else.

### Success Criteria

1. A unit test in `crates/sc-transport` supplies all of sources 1, 2 and 3
   with three different valid endpoints and asserts the result equals the
   source 1 value.
2. A unit test supplies sources 2 and 3 only and asserts the result equals
   the `SC_ENDPOINT` value.
3. A unit test supplies source 3 only and asserts the result equals the
   configuration value.
4. A unit test compiled only on Unix (`#[cfg(unix)]`) supplies no source,
   uses an explicit instance root `R` (a tempdir), and asserts the result is
   the UDS path `R/daemon.sock`.
5. A unit test compiled only on Windows (`#[cfg(windows)]`) supplies no
   override and a configuration port `P`, and asserts the result is the TCP
   address `127.0.0.1:P`.
6. A unit test supplies an unparseable source 1 value and asserts the result
   is `Err` with a `TransportError`, not a panic and not the source 2 value.
7. The tests above run correctly under the default parallel test runner;
   none depends on a process-global environment variable that another test
   in the same binary changes.
8. `crates/sc-transport/Cargo.toml` lists neither `clap` nor `sc-config`.

---

## REQ-TRN-0002: Instance root directory resolution in `sc-transport`

**Status:** Active  

### Requirement Statement

The instance root is the one directory that holds a daemon's run-time files
for one application and one operating-system user: the UDS socket file
`daemon.sock` and the singleton lock file `daemon.lock`.

The crate `sc-transport` MUST expose a public function that takes an
application name (a string such as `"my-app"`) and returns the default
instance root path for that application and the current user. In the crate's
public-surface sketch this is `instance_root(app)`.

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

**OPEN:** The exact per-user location of the default instance root on each
of macOS, Linux and Windows (which base directory, and how the application
name is combined with it) is not decided.

**OPEN:** Whether resolving the instance root also creates the directory, or
whether the caller creates it, is not decided.

**OPEN:** The name `instance_root`, its signature, the way an explicit root
is supplied, and the `TransportError` variant name are illustrative until
the contract sprint pins them.

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

---

## REQ-TRN-0003: UDS and TCP listener binding behind the `server` feature

**Status:** Active  

### Requirement Statement

When the cargo feature `server` of the crate `sc-transport` is enabled, the
crate MUST expose a public function that takes a resolved endpoint (a UDS
path or a TCP socket address) and returns a bound listener. In the crate's
public-surface sketch these are `bind(&Endpoint)` and `Listener`.

The returned listener MUST be accepted by `axum::serve` (axum 0.8) together
with an ordinary `axum::Router`. The router and its handlers MUST NOT need
any change, type parameter or wrapper that depends on whether the listener
is UDS or TCP.

For a UDS endpoint the function MUST bind a Unix domain socket at the given
path. For a TCP endpoint the function MUST bind the given socket address.
When the TCP port is `0` the listener MUST let the caller read the actual
bound address.

One process MUST be able to hold a UDS listener and a TCP listener from this
function at the same time and serve the same router on both.

A bind failure (for example the address is in use, or the directory does not
exist) MUST be returned as an `Err` of the crate's error enum
`TransportError`. The function MUST NOT panic.

The function and the listener type MUST NOT exist in a build without the
`server` feature.

**OPEN:** The names `bind` and `Listener`, whether `Listener` is one type
covering both transports or two types, the accessor for the bound address,
and the `TransportError` variant name for a bind failure are illustrative
until the contract sprint pins them.

**OPEN:** The behaviour of binding a UDS endpoint on Windows is not decided.

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
   for group and for others MUST all be zero (`mode & 0o077 == 0`).
2. When a socket file already exists at the path, left behind by a daemon
   process that died without removing it, the function MUST remove that file
   and bind a new socket at the same path. It MUST NOT fail with "address in
   use" for that reason.
3. The function MUST NOT try to decide whether another daemon is alive. The
   caller MUST guarantee that before calling. In `sc-runtime` the daemon
   takes an OS exclusive lock on `<instance-root>/daemon.lock` before it
   binds, which proves no other daemon for that instance root is running.

**OPEN:** The exact permission mode of the socket file (for example `0600`
or `0700`), and whether it is set on the socket file, on the instance root
directory, or on both, is not decided. Only the "no group or other access"
property above is binding.

**OPEN:** The behaviour when the existing path is not a socket (a regular
file or a directory) is not decided.

### Rationale

File permissions are the only access control on the default transport, so a
socket that other users can open would expose the daemon to them. A leftover
socket file would make every restart after a crash fail with "address in
use". Replacing it is safe only because the caller already holds the
`daemon.lock` singleton; without that guarantee, removing the file would cut
off a live daemon.

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

---

## REQ-TRN-0005: Typed HTTP client `Client` built on reqwest

**Status:** Active  

### Requirement Statement

The crate `sc-transport` MUST expose, in its default build (no cargo
features), a public HTTP client type. In the design sketch it is used as:

```rust
// names are illustrative
let client = sc_transport::Client::connect("my-app", &cfg.endpoint).await?;
let widget: Envelope<Widget> =
    client.post("/ops/widget.create", &CreateWidget { name }).await?;
```

1. `Client::connect(app, endpoint)` MUST take the application name (used in
   the `DAEMON.NOT_RUNNING` suggested action,
   [REQ-TRN-0006](requirements.md)) and an endpoint (a UDS path or a TCP
   address).
2. It MUST build a `reqwest::Client` (reqwest 0.13). For a UDS endpoint it
   MUST configure it with `reqwest::ClientBuilder::unix_socket(path)`. For a
   TCP endpoint it MUST use the base URL `http://<ip>:<port>`.
3. `get(path)` MUST send an HTTP `GET` to `path` on that endpoint.
4. `post(path, &body)` MUST send an HTTP `POST` to `path` with `body`
   serialised as JSON, for any `B: serde::Serialize`.
5. Both methods MUST be `async`, MUST deserialise the JSON response body
   into the caller-chosen type `T`, for any `T: serde::de::DeserializeOwned`,
   and MUST return `Result<T, TransportError>`.
6. The crate MUST NOT name or depend on any concrete response type such as
   `Envelope<T>`; the caller chooses `T`.
7. No method may panic; every failure is a `TransportError` value.

**OPEN:** The names `Client`, `connect`, `get`, `post` and their exact
signatures are illustrative until the contract sprint pins them.

**OPEN:** Whether `connect` itself contacts the daemon (and with what
request) or only builds the client, so that an unreachable daemon is first
reported by `get` or `post`, is not decided. The design sketch comments
`connect` with "Err = DAEMON.NOT_RUNNING" but does not say how.

**OPEN:** Client timeouts (connect timeout, whole-request timeout, and
whether the caller can set them) are not decided.

**OPEN:** How a response with a non-2xx HTTP status is treated is not
decided: see [REQ-TRN-0006](requirements.md), which carries the same open
point.

**OPEN:** The behaviour of `connect` with a UDS endpoint on Windows, where
`unix_socket` does not exist, is not decided.

### Rationale

This is the whole client a CLI needs: one call to connect and one call per
operation. Building on `reqwest` gives one well-known HTTP client for both
transports with no extra crate. Being generic over `T` keeps `sc-transport`
free of any dependency on `sc-command`, where the response envelope lives;
the two crates must stay independent so either can be used alone
([ADR-TRN-0003](architecture.md)).

### Success Criteria

1. A test on Unix starts an HTTP server on a UDS path in a tempdir with a
   `POST` route that echoes a JSON struct, calls `connect` with that path,
   calls `post` with a test struct `Req { name: String }`, and asserts the
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
   build lists the client type with its `connect`, `get` and `post` methods.

---

## REQ-TRN-0006: Typed error `DaemonNotRunning` (`DAEMON.NOT_RUNNING`)

**Status:** Active  

### Requirement Statement

The error enum `TransportError` of the crate `sc-transport` MUST have a
variant `DaemonNotRunning`.

1. The client ([REQ-TRN-0005](requirements.md): `Client::connect`, `get`,
   `post`) MUST return `TransportError::DaemonNotRunning` when it cannot open
   a connection to the endpoint for one of these reasons: the UDS socket
   file does not exist; the UDS socket file exists but nothing is listening
   (connection refused); the TCP connection is refused.
2. `DaemonNotRunning` MUST carry two plain strings: the code, exactly
   `DAEMON.NOT_RUNNING`, and the suggested action, exactly
   `run <app> daemon start` with `<app>` replaced by the application name
   given to `Client::connect` (for `"my-app"`: `run my-app daemon start`).
3. The client MUST NOT start the daemon, spawn any process, or retry in
   order to wait for a daemon.
4. A request that times out MUST produce a `TransportError` variant other
   than `DaemonNotRunning`.
5. A response body that cannot be deserialised into the requested type MUST
   produce a `TransportError` variant other than `DaemonNotRunning`, and
   different from the timeout variant.

**OPEN:** How the client treats a response with a non-2xx HTTP status is not
decided. The earlier text of this requirement made "HTTP error status" its
own error variant. That conflicts with two other facts: the daemon returns a
failed operation as a JSON envelope with `ok: false` under a 4xx or 5xx
status (`sc-command`), and the CLI is meant to receive and print that
envelope. Returning an error variant without the body would hide the
envelope from the CLI. The choice (always deserialise the body into `T`
whatever the status; or an error variant that carries status and body) must
be made in the contract sprint.

**OPEN:** The names of the non-`DaemonNotRunning` variants, the field or
accessor names for the code and suggested action, and whether other
connection errors (for example permission denied on the socket file) map to
`DaemonNotRunning` are not decided.

### Rationale

The daemon being down is the most common CLI failure. It must be told apart
from every other failure so that an agent or a person knows the fix is to
start the daemon, and automation can branch on the stable code
`DAEMON.NOT_RUNNING`. The client only reports: whether a CLI should start
the daemon automatically is an undecided question, and until it is decided
the default is report-only. The code and action are plain strings because
this crate must not depend on `sc-command`, where `OpError` lives; the
project's CLI maps this variant into an `OpError`.

### Success Criteria

1. A test on Unix calls the client against a UDS path in a tempdir where no
   file exists and asserts the first `Err` returned by `connect` or the
   following `get` is `TransportError::DaemonNotRunning`.
2. A test on Unix does the same against a stale socket file (a listener
   bound and dropped without removing the file) and asserts
   `DaemonNotRunning`.
3. A test binds `127.0.0.1:0`, records the port, closes the listener, calls
   the client against that port, and asserts `DaemonNotRunning`.
4. Each of tests 1 to 3 uses the application name `"my-app"` and asserts the
   code equals `DAEMON.NOT_RUNNING` and the suggested action equals
   `run my-app daemon start`.
5. A test asserts that an undecodable response body gives a variant that is
   not `DaemonNotRunning`.
6. Inspection of `crates/sc-transport/src` finds no use of
   `std::process::Command` or `tokio::process`.

---

## NFR-TRN-0001: `sc-transport` links axum only behind `server`

**Status:** Active  

### Requirement Statement

In `crates/sc-transport/Cargo.toml`:

1. There MUST be a cargo feature named `server`.
2. `server` MUST NOT be in the `default` feature list.
3. `axum` MUST be declared `optional = true` and MUST be enabled only by the
   `server` feature.
4. The default build MUST depend on `tokio` and `reqwest` (and `serde` for
   the `Serialize` and `DeserializeOwned` bounds of the client) and MUST NOT
   have `axum` anywhere in its normal dependency tree.
5. All listener-binding code MUST be compiled only under
   `#[cfg(feature = "server")]`.

The crate MUST build and pass its tests both with and without the feature.

### Rationale

The CLI of a generated project depends on this crate for its client, and a
CLI binary must link none of axum, rmcp or sqlx
([NFR-RUN-0001](../requirements.md)): linking the daemon's stack would
multiply its build time and binary size. Listener binding needs axum, so it
is opt-in.

### Success Criteria

1. `cargo tree -p sc-transport -e normal` prints no line containing `axum`.
2. `cargo tree -p sc-transport -e normal --features server` prints a line
   containing `axum`.
3. `cargo test -p sc-transport` passes.
4. `cargo test -p sc-transport --features server` passes.
5. `Cargo.toml` inspection: `default` does not list `server`; `axum` has
   `optional = true`.

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

1. `cargo tree -p sc-transport --all-features` prints no line containing
   `hyperlocal`.
2. `crates/sc-transport/Cargo.toml` declares `reqwest` with version
   requirement `0.13`.
3. Inspection of `crates/sc-transport/src` finds a call to `unix_socket(`,
   no `impl` of a hyper or tower connector trait, and no code that writes an
   HTTP request line (such as `"GET "` or `"HTTP/1.1"`) to a stream.

---

## NFR-TRN-0003: `sc-transport` depends on no other workspace crate

**Status:** Active  

### Requirement Statement

`crates/sc-transport/Cargo.toml` MUST NOT list `sc-config`, `sc-command` or
`sc-runtime` (the other three library crates of the `sc-runtime` workspace)
under `[dependencies]`, `[dev-dependencies]`, `[build-dependencies]` or any
target-specific dependency table, with any feature set.

The crate's boundary manifest under the repository directory
`boundaries/sc-transport/` MUST record `sc-config`, `sc-command` and
`sc-runtime` as forbidden edges.

### Rationale

Each of the four crates must build and test on its own and be usable without
the other three or the template ([REQ-RUN-0002](../requirements.md)), so any
of them can be used by programs that are not sc-runtime daemons or move to
its own repository later. The three standalone crates `sc-config`,
`sc-transport` and `sc-command` have no dependency edges among them; only
`sc-runtime` knows the others ([ADR-RUN-0003](../architecture.md)).

### Success Criteria

1. `cargo tree -p sc-transport --all-features -e all` prints no line
   containing `sc-config`, `sc-command` or `sc-runtime`.
2. A manifest file exists under `boundaries/sc-transport/` and names those
   three crates as forbidden edges.
3. `just lint`, which runs `sc-lint-boundary`, exits 0.
4. `cargo test -p sc-transport` passes from a clean checkout.

---

## NFR-TRN-0004: `sc-transport` returns `TransportError`, never panics

**Status:** Active  

### Requirement Statement

Every public function and method of the crate `sc-transport` that can fail
MUST return `Result<T, TransportError>`, where `TransportError` is one public
`enum` defined in this crate. It is the crate's only public error type.

No public function or method may panic because of caller input (for example
an unparseable endpoint string or a missing path) or because of environment
state (for example a missing directory, a refused connection or an
unreadable environment variable).

Public signatures MUST NOT use opaque error types such as `anyhow::Error` or
`Box<dyn std::error::Error>`.

**OPEN:** The full list of `TransportError` variants and their names is not
decided, except `DaemonNotRunning`
([REQ-TRN-0006](requirements.md)), which is fixed.

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
2. Inspection of non-test code in `crates/sc-transport/src` finds no
   `unwrap()`, `expect(`, `panic!`, `unreachable!`, `todo!` or
   `unimplemented!` on a path reachable from a public function.
3. For each `TransportError` variant there is at least one test that
   triggers it and matches on that variant.
4. A test passes an unparseable endpoint value to the resolver and asserts
   `Err`; a test binds in a directory that does not exist and asserts `Err`.
