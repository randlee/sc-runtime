# `sc-runtime` Requirements

---

## Scope

`sc-runtime` is the facade crate and typestate builder. It is responsible for:

- The `ScRuntimeBuilder<State>` typestate builder
- The `ScRuntime` assembled instance and its `run()` entry point
- Re-exporting the public API of all sub-crates for single-dependency consumers
- Enforcing legal configurations at compile time via typestate

`sc-runtime` is not responsible for:

- Implementing any capability — CLI, storage, daemon, web are implemented in sub-crates
- Domain logic of any kind
- Log construction — the consumer owns the `Logger<Running>` and injects it
- Backend selection — the consumer selects and constructs the storage backend

---

## Functional Requirements

### Typestate Builder

**FR-RT-01** — `ScRuntimeBuilder<NoCli>.cli(register)` must return `ScRuntimeBuilder<HasCli>`. Calling any method other than `.name()` and `.cli()` on `ScRuntimeBuilder<NoCli>` must be a compile error.

**FR-RT-02** — `ScRuntimeBuilder<HasCli>.daemon()` must return `ScRuntimeBuilder<HasDaemon>`.

**FR-RT-03** — `mcp_http()` must only be callable on `ScRuntimeBuilder<HasDaemon>`. Calling `mcp_http()` on `ScRuntimeBuilder<HasCli>` or `ScRuntimeBuilder<NoCli>` must be a compile error. The method must not exist on those types.

**FR-RT-04** — `.web(config)` must be callable on both `ScRuntimeBuilder<HasCli>` (foreground mode) and `ScRuntimeBuilder<HasDaemon>` (supervised mode). The mode is determined by the state type, not by a runtime flag.

**FR-RT-05** — `.plugin(plugin)` must be callable on both `ScRuntimeBuilder<HasCli>` and `ScRuntimeBuilder<HasDaemon>`.

### Logger Injection

**FR-RT-06** — `.logger(logger)` must accept `Logger<Running>` from `sc-observability`. The `Running` typestate guarantees the logger is fully initialized before injection. `sc-runtime` must not construct a `Logger` internally, at any point, in any code path.

### Storage Backend

**FR-RT-07** — `.db(backend)` must accept `Arc<dyn StorageBackend>`, not a concrete backend type. The builder must hold a trait object. Consumers select and construct the implementation; the builder accepts any valid `StorageBackend`.

### Entry Point

**FR-RT-08** — `ScRuntime::run()` must return `Result<(), ScRuntimeError>`. It must never panic. It must never have return type `!`. All error conditions in the runtime lifecycle must be represented as `Err(ScRuntimeError)`.

### Re-exports

**FR-RT-09** — All sub-crate public APIs must be re-exported from `sc-runtime` so that consumers can declare a single `sc-runtime` dependency. A consumer must be able to write `use sc_runtime::Plugin` instead of `use sc_runtime_core::Plugin`.

**FR-RT-10** — axum types must never be re-exported from `sc-runtime`. The HTTP framework remains a private implementation detail of `sc-runtime-web`.

**FR-RT-11** — `Logger<Running>` and `LoggerBuilder` from `sc-observability` must be re-exported from `sc-runtime`. Consumers need these types to construct and inject the logger.

---

## Non-Functional Requirements

**NF-RT-01** — No runtime panics. All invalid states are compile errors enforced by the typestate builder. `ScRuntime::run()` returns `Result`, never panics. This is the core invariant of the `sc-runtime` crate.

**NF-RT-02** — RBP-002 (typestate) must be applied: the `State` type parameter on `ScRuntimeBuilder<State>` encodes configuration state. Method availability is enforced by `impl` blocks scoped to specific state types.

**NF-RT-03** — RBP-010 (PhantomData) must be applied: `ScRuntimeBuilder<State>` uses `PhantomData<State>` to mark the state parameter without storing a state value at runtime.

**NF-RT-04** — The `sc-runtime` crate must compile with no warnings under pedantic + nursery Clippy lints at deny level.

**NF-RT-05** — `unsafe_code = "forbid"` applies to `sc-runtime`. The facade crate contains no unsafe code.

**NF-RT-06** — `sc-runtime` must not introduce any dependency that is not already present in one of its sub-crates. It is a composition layer, not an implementation layer.
