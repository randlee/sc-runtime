# `sc-runtime-core` Requirements

These requirements define the `sc-runtime-core` crate surface and constraints.

## Purpose

`sc-runtime-core` is the foundation crate of the sc-runtime workspace. Its
requirements describe what this crate must provide, what it must not depend on,
and how its types must behave. All other crate-level requirement documents
reference this one when they depend on core-provided types.

## Authoritative Requirement Sources

- [../../docs/prd/sc-runtime-prd.md](../../docs/prd/sc-runtime-prd.md) —
  Sections 6.1, 9, 11, 12 (FR-12, FR-14, FR-15, NF-01, NF-05)
- [architecture.md](./architecture.md)

## Scope Rules

The following belong in `sc-runtime-core`:

- `Plugin` trait and object-safety machinery
- `PluginMetadata`, `PluginContext`, `PluginError`
- `ScRuntimeHome` and its `SC_RUNTIME_HOME` resolution
- `ScRuntimeError`, `ErrorCode`, `Remediation`
- `PluginName` and other workspace-wide newtypes with no layer affiliation

The following do not belong in `sc-runtime-core`:

- CLI argument parsing or envelope types (belong in `sc-runtime-cli`)
- Storage trait or backend types (belong in `sc-runtime-db`)
- Daemon lifecycle, PID files, signal handling (belong in `sc-runtime-daemon`)
- Any type whose definition requires a dependency on a domain crate
- Logger construction (`LoggerBuilder` calls belong in application code)

## Functional Requirements

### FR-CORE-01 — Plugin Trait Must Be Object-Safe

The `Plugin` trait must be usable as `Box<dyn Plugin>` without any
`where Self: Sized` escape hatch on the lifecycle methods.

The lifecycle methods (`init`, `run`, `shutdown`) must use
`BoxFuture<'a, Result<(), PluginError>>` return types — not bare `async fn` —
to maintain object safety. Bare `async fn` in a trait is not compatible with
`dyn Trait` in stable Rust (as of 1.94) because each impl produces a distinct,
unnameable future type that cannot be placed in a vtable.

A compile-time test asserting `Box<dyn Plugin>` compiles must exist in the
`sc-runtime-core` test suite:

```rust
#[test]
fn plugin_trait_is_object_safe() {
    let _: Box<dyn Plugin>;
}
```

This test need not execute — it only needs to compile. If it fails to compile,
the PR must not merge.

**Rationale:** The daemon plugin registry stores heterogeneous plugin
collections. Monomorphization across an open set of consumer plugin types is
not viable. Verifying object safety at compile time via a test catches
regressions that design-time review may miss.

### FR-CORE-02 — `PluginContext` Must Carry an Injected `Logger<Running>`

`PluginContext` must carry a `Logger<Running>` as `Arc<Logger<Running>>`.
The logger must be injected by the application via the sc-runtime builder
before `run()` is called. `sc-runtime-core` must never call `LoggerBuilder`
or construct a `Logger` internally.

**Rationale:** Tools own their log root, service name, sinks, and retention
policy. sc-runtime making opinionated decisions about logger construction
would force a single log configuration on all consumers (ADR-07).

### FR-CORE-03 — `SC_RUNTIME_HOME` Override Must Work for Test Isolation

`ScRuntimeHome` must read the `SC_RUNTIME_HOME` environment variable at
construction time. When the variable is set to a temporary directory, all
sc-runtime artifact paths (PID files, sockets, database files) must resolve
under that directory. Tests must be able to achieve full artifact isolation by
setting this variable without modifying any other configuration.

**Rationale:** Parallel test execution requires isolated artifact roots.
A global mutable path would cause test interference.

### FR-CORE-04 — `ScRuntimeError` Must Carry Stable Code and Remediation on Every Variant

Every variant of `ScRuntimeError` must carry:

- a stable `ErrorCode` using the `SC_RUNTIME.{SUBSYSTEM}.{NAME}` convention
- a human-readable `message: &'static str`
- an optional `cause: Option<Box<dyn Error + Send + Sync>>` for error chaining
- an optional `Remediation` containing a `suggested_action` for
  operator-recoverable conditions

No variant may carry only an opaque `String` or omit the `ErrorCode` field.
Removing or renaming an `ErrorCode` value is a semver-breaking change.

**Rationale:** RBP-001 (Error Context + Recovery). The CLI and MCP layers
surface `suggested_action` to operators; machine consumers key on `ErrorCode`
for programmatic handling.

### FR-CORE-05 — Zero SC Domain Crate Dependencies

`sc-runtime-core` must have zero compile-time dependencies on any SC domain
crate. This is a hard boundary enforced by the `sc-lint-boundary` CI gate
via the definitions in `boundaries/sc-runtime-core/`. A dependency violation
must fail CI.

**Rationale:** sc-runtime is consumed by domain crates. A reverse dependency
would create a cycle. The boundary also protects `sc-runtime-core` from
accumulating domain-specific concepts that belong in consumer code.

### FR-CORE-06 — `PluginMetadata` Accessors Must Be Infallible

`PluginMetadata::name()` and `PluginMetadata::version()` must return
`&'static str` with no `Result` wrapper. Plugin identity and version are
known at compile time and must not require error handling at call sites.

**Rationale:** RBP-007 (Infallible). Wrapping a static string return in
`Result` adds noise at every call site with no benefit.

### FR-CORE-07 — `ErrorCode` Must Be a `&'static str` Newtype

`ErrorCode` must be defined as a newtype wrapping `&'static str`. It must
expose a `const fn new(code: &'static str) -> Self` constructor and an
`as_str(&self) -> &'static str` accessor. All predefined codes must be
`const` values so that pattern matching and equality checks are zero-cost.

No variant of `ScRuntimeError` or `PluginError` may carry an error code
that is a runtime-constructed `String` — all codes must be `&'static str`
values traceable to a declaration in source.

**Rationale:** FR-CORE-04. Machine consumers key on `ErrorCode` for
programmatic handling. A dynamically constructed string code cannot be
matched exhaustively and is not stable across releases.

### FR-CORE-08 — All `PluginError` Variants Must Be Enumerated and Stable

The `PluginError` enum must enumerate all possible plugin lifecycle failure
modes as named variants. The following variants are required and stable:

- `InitFailed` — plugin `init()` returned an error
- `RunFailed` — plugin `run()` returned an error
- `ShutdownFailed` — plugin `shutdown()` returned an error
- `Panic` — plugin task panicked; `plugin_name` must be `String` (not
  `&'static str`) because a panic may corrupt static memory
- `Cancelled` — plugin was cancelled before completing

No `Other(String)` or open-ended catch-all variant is permitted. Adding a
new variant is a semver-breaking change and requires a PRD update. Removing
or renaming a variant is also semver-breaking.

`PluginError` must derive `Debug` and implement `thiserror::Error`. Every
variant that represents an operator-recoverable condition should carry
`remediation: Option<Remediation>`.

**Rationale:** RBP-003 (Exhaustive Enums). The daemon's error handling and
the CLI's exit code mapping depend on matching `PluginError` exhaustively.
An open-ended variant undermines both.

## Non-Functional Requirements

### NF-CORE-01 — `unsafe_code = "forbid"`

The `sc-runtime-core` crate manifest must set `#![forbid(unsafe_code)]`.
No unsafe code is permitted without a documented, justified override approved
through the normal PR process.

### NF-CORE-02 — No Std Runtime Assumptions

`sc-runtime-core` must not assume a specific async runtime. `tokio` must not
be a direct dependency. `CancellationToken` from `tokio-util` is permitted
because it carries no tokio runtime requirement at the type level.

**Rationale:** Keeps `sc-runtime-core` usable in sync-only contexts and
avoids coupling the foundation layer to a specific tokio feature set.

### NF-CORE-03 — Pedantic Clippy at Deny Level

The crate must compile without warnings under `clippy::pedantic` and
`clippy::nursery` at the deny level, consistent with the sc-lint workspace
Clippy policy.

## Dependency Boundary Rules

| Category | Rule |
|----------|------|
| Permitted workspace dependencies | **none** — zero `sc-runtime-*` dependencies |
| Permitted external dependencies | `sc-observability`, `tokio-util` (CancellationToken only), `futures` (BoxFuture), `thiserror` |
| Forbidden | any `sc-runtime-*` crate; any SC domain crate (`atm-core`, `continuity`, etc.) |
| Boundary file | `boundaries/sc-runtime-core/Boundary.toml` |

`sc-runtime-core` is the foundation layer. Depending on any peer `sc-runtime-*`
crate would create a cycle. Depending on a domain crate would allow
domain-specific concepts to accumulate in the foundation, undermining the
zero-domain-dependency invariant.

## Boundary Governing Document

The boundary rules for `sc-runtime-core` are defined in:

```
boundaries/sc-runtime-core/
```

These TOML files are read by `sc-lint-boundary` during CI. Any crate that
adds a dependency on `sc-runtime-core` that violates the boundary definition
will fail the lint gate.

## Related Docs

- [architecture.md](./architecture.md)
- [../sc-runtime-cli/requirements.md](../sc-runtime-cli/requirements.md)
- [../../boundaries/sc-runtime-core/](../../boundaries/sc-runtime-core/)
- [../../docs/prd/sc-runtime-prd.md](../../docs/prd/sc-runtime-prd.md)
