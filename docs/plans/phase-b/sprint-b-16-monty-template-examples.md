---
id: b-16
phase: b
status: planned
closure_type: boundary
target_boundary: monty-host-extensibility
track: monty-product
wave: 8
branch: sprint/b-16-monty-template-examples
worktree: sprint/b-16-monty-template-examples
integration_branch: integrate/phase-b-observability
requirements:
  - REQ-RUN-0003
  - REQ-RUN-0702
  - REQ-RUN-0309
  - REQ-RUN-0301
  - REQ-RUN-0005
  - REQ-CRON-0001
  - REQ-CRON-0003
  - REQ-CRON-0005
  - NFR-CRON-0001
  - NFR-RUN-0005
adrs:
  - ADR-RUN-0002
  - ADR-RUN-0013
  - ADR-RUN-0303
owned_paths:
  - docs/architecture.md
  - docs/requirements.md
  - boundaries/sc-config/sc-config.toml
  - boundaries/sc-transport/sc-transport.toml
  - boundaries/sc-command/sc-command.toml
  - boundaries/sc-runtime/sc-runtime.toml
  - sc-lint.toml
  - tests/unit/test_production_tool_boundary.py
  - docs/sc-cron/requirements.md
  - docs/sc-cron/architecture.md
  - docs/sc-cron/hooks-sc-compose.md
  - boundaries/sc-cron/sc-cron.toml
  - crates/sc-cron/README.md
  - crates/sc-cron/src/lib.rs
  - crates/sc-cron/src/scheduler.rs
  - crates/sc-cron/src/monty_host.rs
  - crates/sc-cron/src/monty_run.rs
  - crates/sc-cron/src/monty_compose.rs
  - crates/sc-cron/src/monty_compose_common.rs
  - crates/sc-cron/tests/monty_host_bridge.rs
  - crates/sc-cron/tests/sc_compose_bench.rs
  - scripts/benchmark/monty/campaign.py
  - template/Cargo.toml
  - template/workspace.template.toml
  - template/crates/daemon/Cargo.toml
  - template/examples/monty/cron.py
  - docs/sc-cron/monty-generated-example.md
  - tests/unit/generation/test_monty_example.py
must_follow: [b-10, b-13]
parallel_safe: [b-14, b-15]
recommended_agent: cipher
recommended_model: luna
vertical_rationale: One Monty product boundary replaces the pre-release fixed host list with an embedder extension seam, keeps compose as an optional adapter, and proves that seam in the generated example and benchmark; creating another host sprint would duplicate this ownership.
---

# Sprint b-16 — extensible Monty hosts and generated example

## Goal

Turn the landed Track B host bridge into a reusable `sc-cron` interface, then
expose it in generated projects through an opt-in Cargo feature and one checked
example. Applications can register their own Monty host calls;
`sc_compose_run` and `atm_run` are supplied adapters registered by the application, while
`render_template` is removed as product-level API.

## Relations

The baseline is `develop` after Track B Monty merged at `240e445`. b-10 proves
the production suspend/resume loop and the safe render-only compose adapter;
it does not freeze its pre-release fixed two-name registry as the general
library API. This sprint follows b-13 because b-13 owns the preceding edits to
the same crate and authority documents. It remains `parallel_safe` with b-14
and b-15 because their exact paths are disjoint. b-17 merges b-16 first and
adds the resolved environment snapshot to the completed host-call context.

## Public host-extension contract

Feature `monty` exports `MontyHostRegistry`, `MontyHostHandler`, and the owned
JSON-shaped `MontyHostCall`. `MontyHostHandler` is `Send + Sync` and has one
synchronous method returning `Result<serde_json::Value, CronError>`; a blanket
implementation makes a matching closure usable without a wrapper type. A call
contains its registered name, positional JSON arguments, keyword JSON
arguments, schedule name, hook phase, and row deadline. b-17 adds read-only
access to that run's resolved environment snapshot without changing handler
registration or allowing process-global reads.

`MontyHostRegistry::new()` is empty. Its consuming `register(name, handler)`
builder returns `CronError::InvalidSpec` for a non-identifier, `__hook`,
`cron_stop`, `cron_tick`, or a duplicate name. Registration order does not
change behavior. `Scheduler::with_monty_hosts(registry)` installs one immutable
registry for subsequent runs. Hook source may invoke a registered name but
cannot register, replace, or discover a handler object. A source reference to
an unregistered name fails as `HOST_UNDEFINED`; a registered handler error is
recorded as `HOST_CALL_FAILED` after b-17 secret redaction. The handler runs on
the existing blocking feed thread: the call includes the row deadline and
sc-cron checks it before and after the handler, but cannot preempt arbitrary
synchronous application code. This is the same trusted-code boundary as a
Rust `Job`, not a sandbox or an async executor.

`ScComposeHostConfig` and `AtmHostConfig` are the supplied opt-in adapters.
Adding either to a registry registers `sc_compose_run(subcommand, args)` or
`atm_run(subcommand, args)` respectively. Both configs accept an
embedder-selected non-empty program path; the respective defaults are
`sc-compose` and `atm` on `PATH`. Enabling Monty registers neither adapter.
Both adapters forward any non-empty command/subcommand and its ordered string
arguments directly to the configured executable using argv, never a shell.
No subcommand allowlist, forced render arguments, ignored flags, or mandatory
temp-root staging narrows either integration. The embedding application sets
the executable and working directory; script arguments are forwarded unchanged.
The working directory defaults to the daemon directory captured at adapter
configuration. Both adapters share one process supervisor, return
`{stdout, stderr, exit_code}` for completed children (including nonzero exits),
and use the row deadline, process-group termination, and per-stream output cap.
Spawn/capture errors fail the hook; deadline expiration records `TimedOut`.
The call shape is one command/subcommand string plus an optional list of
string arguments; keyword arguments and other arities/types fail as
`HOST_DENIED` before spawn. A command such as `atm agent list` is represented
as `atm_run("agent", ["list"])`; the registry does not parse or restrict the
tool's command grammar. The script decides how to handle a nonzero exit
returned in the result object.
The generic registry remains usable with neither tool installed. b-17
supplies the same resolved environment to both children. `render_template`
is absent from the required product API and production host registration.
The production-bridge benchmark and generated example invoke
`sc_compose_run("render", ...)` through the same optional adapter. Internal
render helpers, lab benchmark code, and historical reports may retain that
name; their presence does not create a product requirement.

The template Cargo files are owned only by this sprint. b-17 consumes the
resulting registry and `cron`/`monty` feature definitions and does not edit
those manifests.

The generated daemon feature contract is fixed:

```toml
[features]
cron = ["dep:sc-cron"]
monty = ["cron", "sc-cron/monty"]

[dependencies]
sc-cron = { workspace = true, optional = true }
sc-observability-types.workspace = true
```

`template/Cargo.toml` is the authoritative checkout patch inventory and
`template/workspace.template.toml` is the rendered registry dependency
inventory. Both record the internal `0.1.1` cut (`sc-config`, `sc-transport`,
`sc-command`, `sc-runtime`, and `sc-cron`); the workspace template additionally
uses caret-compatible `1.5.1` requirements for the published
`sc-observability` / `sc-observability-types` pair. Neither feature joins
`default`. A normal checkout render therefore
patches `sc-cron` to the local workspace just like the other internal crates,
while `scripts/new_project.py --release` suppresses every checkout patch.

## Required work and deliverables

1. REQ-CRON-0003 / REQ-CRON-0005 / ADR-RUN-0013 / NFR-CRON-0001: amend the ADR, requirements,
   component architecture, boundary exports, and crate API in lockstep to
   implement the public host-extension contract above. Replace
   `MontyHostConfig` / `Scheduler::with_monty_host` with
   `ScComposeHostConfig`, `AtmHostConfig`, `MontyHostRegistry`, and
   `Scheduler::with_monty_hosts`. There is no compatibility alias because no
   binary or crate carrying the b-10 development API has been released.
2. REQ-CRON-0003 / ADR-RUN-0013: retain `sc_compose_run` only as the opt-in
   adapter described above. Remove the fixed `render_template` host registration, migrate
   `monty_host_bridge`, `sc_compose_bench`, and the campaign source to
   `sc_compose_run("render", ...)`, and keep the b-10 process deadline,
   output cap and child-environment protections. Implement `atm_run` in this
   same sprint through the identical registry and supervisor. Remove the old
   render-only/temp-root argument rewriting from the general CLI adapters.
3. REQ-RUN-0301 / REQ-CRON-0001 / ADR-RUN-0303: add optional daemon dependency `sc-cron`, direct
   `sc-observability-types`, and features equivalent to
   `cron = ["dep:sc-cron"]` and
   `monty = ["cron", "sc-cron/monty"]`; preserve the default feature set.
4. ADR-RUN-0002 / REQ-RUN-0003 / REQ-RUN-0309 / REQ-RUN-0702: update both template dependency
   inventories for the `0.1.1` internal cut, including `sc-cron`, and set the
   published `sc-observability` / `sc-observability-types` requirements to
   caret-compatible `1.5.1`; generated lockfiles prove exact resolution.
   `template/Cargo.toml` must contain the same five internal crate names used
   by checkout patch generation; `template/workspace.template.toml` must
   contain their `0.1.1` registry requirements. These shared Cargo artifacts are
   consumed by b-17 and owned only here.
5. REQ-CRON-0003: add `template/examples/monty/cron.py` with `cron_stop` and
   `cron_tick`. Examples demonstrate opt-in `sc_compose_run` and `atm_run`
   adapters and return the current stop decision. Component
   documentation includes a second minimal scheduler snippet that registers
   an application closure such as `jev_call`, proving the library is not
   compose-specific.
6. ADR-RUN-0013 / NFR-CRON-0001: create
   `docs/sc-cron/monty-generated-example.md` documenting the generated path,
   feature enablement, input/output shape, and security limits. Monty source
   is application-supplied in-process source. MVP has no signing, separate
   script approval mechanism, or consumer option enabling one. Host registration
   and environment policy are enforced independently. Registered synchronous
   handlers retain stack-overflow, allocator-abort, panic, and non-cooperative
   deadline risks.
7. REQ-RUN-0005 / NFR-CRON-0001: enforce the tool-crate exclusion on all
   production release crates (`sc-config`, `sc-transport`, `sc-command`,
   `sc-runtime`, and `sc-cron`). Boundary lint checks normal/build dependencies,
   renamed packages, Rust imports, and production feature combinations; a
   cargo-tree companion gate rejects transitive tool-crate linkage where the
   boundary tool cannot inspect external packages. Test/benchmark-only references
   remain permitted. `test_production_tool_boundary` includes rejected
   production and accepted dev-only fixture manifests. Resolve package identity
   rather than matching dependency aliases: known tool packages include
   `sc-compose`, `sc-compose-*`, `agent-team-mail`, `agent-team-mail-core`, and
   `atm-*`; the ATM Rust library name `atm_core` is also covered by import
   checks. Renamed dependency fixtures must fail as production edges.
8. NFR-RUN-0005: add a generation test that renders the example and compiles the generated
   daemon with and without `monty`. Assert that a checkout render contains the
   local `sc-cron` patch and a `--release` render contains no
   `[patch.crates-io]` or path dependency; b-18 owns the registry build after
   publication.

## Acceptance criteria

1. `REQ-CRON-0001`: `test_monty_example` and the two generated `cargo check`
   commands pass with the generated daemon as the only generated crate with
   the optional `sc-cron` edge; the feature-off build does not link it.
2. `REQ-CRON-0003`: `monty_host_bridge` passes empty-default, custom
   closure call/result, handler error, multiple-call, invalid/duplicate/reserved
   registration, unregistered-name denial, and both opt-in CLI adapter cases.
   Named cases `custom_host_without_cli`, `compose_non_render_argv`,
   `atm_argv`, `cli_nonzero_result`, `cli_spawn_failure`, `cli_deadline`, and
   `cli_output_cap` prove these behaviors with temporary executable fixtures.
   Spawn, output, and handler-error cases assert `HOST_SPAWN`, `HOST_OUTPUT`,
   and `HOST_CALL_FAILED` respectively; nonzero child exit remains an ordinary
   returned CLI result rather than being rewritten as a render failure.
   The production registration assertion derives names from the configured
   registry rather than requiring a fixed global list.
3. `NFR-CRON-0001`: the required `cargo tree` command passes inspection with
   no Monty crate in the default tree and no `monty-pool`, `monty-fs`, or
   `monty-runtime` in the Monty tree.
4. `NFR-RUN-0005`: `test_monty_example` passes using a temporary output and
   no sleep, network, globally fixed project path, or external compose process.
5. ADR-RUN-0013: `monty_host_bridge` and `test_monty_example` pass only when
   the default registry is empty, the example opts into `sc_compose_run`, no
   source can install or replace a handler, and docs label source and handlers
   application-supplied in-process code with no MVP signing/approval gate.
6. ADR-RUN-0303: the required `cargo tree` and manifest `rg` commands pass
   with the recorded matching observability pair and locally patched
   `sc-cron`; the default feature tree does not enable `sc-cron/monty`.
   Registry resolution itself belongs to b-18 after publish.
7. ADR-RUN-0002: the manifest `rg` check and `test_monty_example` pass only
   when the two template manifests agree on the complete five-crate
   internal inventory at `0.1.1`; checkout generation patches `sc-cron`
   locally, while release-mode generation emits no checkout patch.
8. `test_monty_example` passes with b-17 able to consume the exact feature
   names without reopening a Cargo manifest.
9. `monty_host_bridge::render_helper_not_registered` passes only when default
   registration has no `render_template` binding and a configured CLI registry
   exposes only its registered names. `test_monty_example` proves the checked
   example and production-bridge benchmark call `sc_compose_run`; it does not
   ban internal helper names or explanatory historical documentation.
10. REQ-RUN-0005: `test_production_tool_boundary` and the production
    dependency-tree gates pass only when every release crate excludes the
    compose/ATM crate families from normal/build dependencies, including
    renamed and transitive packages, for its supported production features.
    Fixture cases prove a production Rust import is rejected and a test-only
    dependency/import is accepted. Executable names in adapter configuration
    and documentation are not Rust crate references.

## Required validation

```sh
python3 -m unittest tests.unit.generation.test_monty_example tests.unit.test_production_tool_boundary
cargo +1.96.1 test -p sc-cron --features monty --test monty_host_bridge
cargo +1.96.1 test -p sc-cron --features monty --test sc_compose_bench
just new generated/test-monty --var-file wizard/fixtures/sqlite-mcp.json
cargo check --manifest-path generated/test-monty/Cargo.toml -p widget-app-daemon
cargo +1.96.1 check --manifest-path generated/test-monty/Cargo.toml -p widget-app-daemon --features monty
cargo tree --manifest-path generated/test-monty/Cargo.toml -p widget-app-daemon -e features
cargo tree --manifest-path generated/test-monty/Cargo.toml -p widget-app-daemon -e features --features monty
rg -n 'sc_compose_run|cron_stop|cron_tick' generated/test-monty/examples/monty/cron.py
rg -n 'MontyHostRegistry|MontyHostHandler|MontyHostCall|ScComposeHostConfig|AtmHostConfig|with_monty_hosts' crates/sc-cron/src/lib.rs crates/sc-cron/src/scheduler.rs crates/sc-cron/src/monty_host.rs boundaries/sc-cron/sc-cron.toml docs/sc-cron
rg -n 'sc-(config|transport|command|runtime|cron).*0\.1\.1|sc-observability(-types)?.*1\.5\.1' template/Cargo.toml template/workspace.template.toml
just lint gates
```

The generation test owns cleanup of its temporary output; the literal
`generated/test-monty` commands are the operator reproduction form and require
that destination to be absent before the first command.

## This sprint does not close

- Async host handlers, per-schedule handler registration, script signing,
  separate script approval, or a claim that arbitrary synchronous application code can be
  forcibly cancelled at the row deadline.
