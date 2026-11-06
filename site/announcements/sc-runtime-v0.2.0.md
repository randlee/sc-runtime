# sc-runtime v0.2.0 — Standard Cron Commands, Cooperative Cron Lifecycle, Lint-Clean Generation

**Released:** November 6, 2026 · **Tag:** `v0.2.0` · **Previous:** `v0.1.1` (November 2, 2026)

[Release notes](https://github.com/randlee/sc-runtime/releases/tag/v0.2.0) · [Requirements](https://github.com/randlee/sc-runtime/blob/main/docs/sc-cron-commands/requirements.md) · [Generate a project](https://github.com/randlee/sc-runtime/blob/main/docs/generation.md)

---

## Rust Product Developer — a cron surface that is a contract, not an ad-hoc loop

**As a Rust product developer, I want a batteries-included runtime for CLI, MCP, SQL/SQLite, daemon and web-server plug-ins, so that I don't re-implement tokio setup, storage, shutdown and retry in every SC tool.**

v0.2.0 closes the last gap between "the runtime has a scheduler" (that was
v0.1.1) and "the runtime has a scheduler a product can expose." The new
**`sc-cron-commands`** crate defines `cron.start`, `cron.end` and `cron.list` as
one version-1 JSON contract — a single `CronRequest` and
`sc_command::Envelope<CronResponse>` projected identically over CLI, HTTP and
MCP. A handler kind selects Rust, Python, or explicitly enabled Monty; the verbs
and the JSON semantics do not change with the projection. Nothing starts a job
automatically: an application decides what its targets are before any schedule is
accepted. `cron.start` takes a five-field expression (or a supported alias), a
bounded timeout and an opaque payload/environment; `cron.end` requires name *and*
generation; `cron.list` pages deterministically over active/ended/all with
id/generation/state views. Validation happens before the write — unknown or
duplicate fields, bad version/generation/limits/cursor, overflow and unsupported
targets all fail ahead of persistence, and diagnostics never leak raw SQL,
environment values, source or payload.

Underneath it, `sc-cron` gained the lifecycle that makes embedding honest in
**C-5**: `Scheduler::spawn_worker_cooperative(Arc<dyn Job>)` returns a
`WorkerHandle` the application retains for the physical join, and
`worker.stop_control()` yields a cloneable `WorkerStopControl` for the operation
context. That is the difference between a scheduler that drops its worker on a
daemon restart and one whose shutdown actually waits. `StandardCronExecutor`
derives its operation context as a child of the caller's; it does not invent a
second timeout or ownership policy inside the adapter. Applications that need
different shutdown behaviour can implement `CronExecutor` or the underlying
`CronOperations` — the standard cooperative guarantee is documented as belonging
to the standard implementation, not inherited by every custom one.

The rest of the release is ecosystem currency rather than new surface: the
workspace moves to edition 2024 and `rust-version = 1.94.1`, with `axum 0.8.9`,
`utoipa`/`utoipa-axum` on the 6.0 line and `rmcp ~3.8` for the MCP server
transport. Public crates now inherit the shared release version rather than
restating it per manifest, which is what removes the class of bug where one crate
in a published set drifts from its siblings.

```toml
sc-cron-commands = { version = "0.2.0", default-features = false, features = ["scheduler"] }
sc-cron = { version = "0.2.0", default-features = false }
```

## Template / CI Integrator — the generated project is finally lint-clean

**As a team lead starting a new SC tool, I want a generated project skeleton that is already lint-clean and just-ready, so that "new repo" does not mean "new drift".**

This is the commitment v0.1.0 and v0.1.1 both deferred, and it is delivered. A
generated project's `sc-lint.toml` now carries the full **three-profile** pass in
both its `full` and `default` lint profiles: `fmt`, `clippy` (`-D warnings
-D clippy::await_holding_lock`), `sc-boundary`, `sc-portability` and
`sc-runtime`, plus the workspace test step. A new component is therefore measured
against the same gates as everything else from its first commit instead of
inheriting drift and discovering it months later. The manifest sits at
`minimum_version = "0.8.4"`, so the generated repo fails loudly rather than
silently on an older sc-lint.

Generation grew two options to match the runtime work: `cron` (default `false`)
and `cron_commands` (default `false`, and selecting commands **requires** cron
and does not enable MCP on its own). Selecting both emits a project whose
application root opens the SQLite schedule store with its configured limits,
builds the scheduler with its clock and jitter, configures bounded command
capacity, explicitly starts the cooperative worker, retains the handle for the
physical join, and hands the same `StandardCronExecutor` to each selected UI.
The generated daemon joins the cron lifecycle into its shutdown path, so a
stopped daemon stops its worker rather than leaking the task. The `db` answer
still selects one layout out of `none`, `sqlite`, `postgres`, `mysql`,
`sqlite-postgres`, `sqlite-mysql`, and the answers schema remains version 2 with
one schema-validated path for `--var-file`, `--prefill` and interactive runs.

## Plugin Author — leaf crates that stay leaves

**As a plugin author, I want stable, separately-published leaf contracts, so that I can depend on the transport or config layer without dragging the whole runtime into my crate.**

`sc-cron-commands` defaults contain **only** the DTOs and the object-safe
`CronExecutor` port; the default dependency graph pulls serde, serde_json and a
defaults-off `sc-command`, and no cron, SQLite, Tokio, Monty, clap, Axum or RMCP.
Three independent features add surface on demand: `schema` (Schemars/OpenAPI on
the same DTO family), `scheduler` (`StandardCronExecutor` over native
`sc_cron::CronOperations` plus target bindings) and `monty` (explicitly enabling
the interpreter, and requiring Rust 1.96.1 — the other builds keep the workspace
MSRV of 1.94.1). `schema` and `scheduler` are independent of each other, so a CLI
client can share the wire DTOs without linking a scheduler or a server. There is
no `sc-runtime` → cron edge and no `sc-command` → cron edge; the boundary holds.

`sc-config`, `sc-transport` and `sc-command` keep their contracts. Adopting the
new crates is additive: you can take the DTO contract for a client, or the
scheduler feature for a daemon, without renegotiating the transport or config
layer you already ship against.

## Observability Consumer — same canonical vocabulary, still no exporter

**As a developer instrumenting an SC component, I want logging built into the runtime and an opt-in path to OpenTelemetry, so that I get consistent logs by default and OTLP export only when my deployment has a collector.**

No change in substance at v0.2.0: the runtime and `sc-cron` emit through the
injected logger against `sc-observability-types 1.5.0`, so a component's events
land in the same vocabulary as the rest of the SC logging stack. The bounded
injection shim added in v0.1.1 is unchanged and still carries the contract and
the values, not the exporter surface. **`sc-observability-otel` is still not
here** — there is no `otel`/`opentelemetry` reference in the crates or their
manifests and no cargo feature gating one. A default build carries no exporter,
no background retry loop and no network egress; that remains an open commitment,
and calling it shipped would be wrong.

---

## Not this release

- **OTLP / OpenTelemetry export** — see above. Still open, still zero egress by default.
- **Fleet operator / SRE deployment surface** — daemon supervision and collector rollout remain sc-observability's and scterm's story. sc-runtime carries no deployment surface.
- **Python bindings** — none. Monty hosts are driven from Rust; there is no Python API to document.
- **Release binaries** — the v0.2.0 release page carries `checksums.txt` and SDK assets only. crates.io is the distribution channel; there is no Homebrew, Scoop or installer path.

## Install

Six public crates at `0.2.0` on crates.io (the SQLite writer stays internal,
`publish = false`):

```toml
sc-config        = "0.2.0"
sc-transport     = "0.2.0"
sc-command       = "0.2.0"
sc-runtime       = "0.2.0"
sc-cron          = "0.2.0"
sc-cron-commands = "0.2.0"   # defaults: DTOs + CronExecutor only
```

## Upgrade Notes

- **MSRV is now Rust 1.94.1** and the workspace is on edition 2024. The `monty`
  feature on `sc-cron-commands` requires 1.96.1; every other build keeps 1.94.1.
- **`sc-cron-commands` defaults are intentionally inert.** If you expect a
  running scheduler from the default feature set you will not get one — enable
  `scheduler`, and start the worker from your application root. That is by
  design: the adapter does not install a daemon or migrate a consumer.
- **Workspace versions are inherited**, so depend on `0.2.0` consistently across
  the six crates rather than mixing published versions.
