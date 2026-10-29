---
id: b-16
phase: b
status: planned
closure_type: integration
target_boundary: generated-daemon-monty
track: monty-product
wave: 7
branch: sprint/b-16-monty-template-examples
worktree: sprint/b-16-monty-template-examples
integration_branch: integrate/phase-b-observability
requirements:
  - REQ-RUN-0003
  - REQ-RUN-0702
  - REQ-RUN-0309
  - REQ-CRON-0003
  - NFR-CRON-0001
  - NFR-RUN-0005
adrs:
  - ADR-RUN-0002
  - ADR-RUN-0013
owned_paths:
  - template/Cargo.toml
  - template/workspace.template.toml
  - template/crates/daemon/Cargo.toml
  - template/examples/monty/cron.py
  - docs/sc-cron/monty-generated-example.md
  - tests/unit/generation/test_monty_example.py
must_follow: [b-10]
parallel_safe: [b-13, b-14, b-15]
recommended_agent: cipher
recommended_model: luna
vertical_rationale: One generated-product boundary owns the Cargo feature, checked example, documentation, and render assertion; splitting it would create pass-through sprints without parallel work.
---

# Sprint b-16 — generated Monty feature and checked examples

## Goal

Expose the already-implemented PR #84 Monty host contract in generated
projects through an opt-in Cargo feature and one checked example. This is the
remaining Monty product-surface work; it does not reopen b-10 through b-12.

## Relations

PR #84 being merged is a phase entry prerequisite. The retained `must_follow`
b-10 edge names the concrete `render_template` and render-only
`sc_compose_run` host artifact that this example consumes. This sprint is
`parallel_safe` with b-13 through b-15 because it alone owns the shared
generated Cargo files and its example/docs/test paths. b-17 must merge b-16
first because it consumes the `cron`/`monty` feature artifact.

## Consumed artifact

PR #84 must be merged. The example consumes exactly `render_template` and the
render-only `sc_compose_run`. It cannot select a root or another compose
subcommand. The template Cargo files are owned only by this sprint. b-17
consumes the resulting `cron`/`monty` feature definitions and does not edit
those files.

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
pins the published `sc-observability` / `sc-observability-types` pair to
`1.5.1`. Neither feature joins `default`. A normal checkout render therefore
patches `sc-cron` to the local workspace just like the other internal crates,
while `scripts/new_project.py --release` suppresses every checkout patch.

## Required work and deliverables

Every numbered item must land production-ready for the generated Monty
product boundary: rendered Cargo features, example, documentation, and
generation assertions ship together with no placeholder hook body.

1. REQ-CRON-0003: add optional daemon dependency `sc-cron`, direct
   `sc-observability-types`, and features equivalent to
   `cron = ["dep:sc-cron"]` and
   `monty = ["cron", "sc-cron/monty"]`; preserve the default feature set.
2. ADR-RUN-0002 / REQ-RUN-0003 / REQ-RUN-0702: update both template dependency
   inventories for the `0.1.1` internal cut, including `sc-cron`, and set the
   published `sc-observability` / `sc-observability-types` pins to `1.5.1`.
   `template/Cargo.toml` must contain the same five internal crate names used
   by checkout patch generation; `template/workspace.template.toml` must
   contain their exact registry versions. These shared Cargo artifacts are
   consumed by b-17 and owned only here.
3. REQ-CRON-0003: add `template/examples/monty/cron.py` with `cron_stop` and `cron_tick`.
   The example calls both allowed hosts, treats `sc_compose_run` as
   render-only, and returns the current stop decision.
4. ADR-RUN-0013 / NFR-CRON-0001: document generated path, feature enablement, input/output shape, and
   security limits. Do not imply that Monty source is trusted.
5. NFR-RUN-0005: add a generation test that renders the example and compiles the generated
   daemon with and without `monty`. Assert that a checkout render contains the
   local `sc-cron` patch and a `--release` render contains no
   `[patch.crates-io]` or path dependency; b-17 owns the registry build after
   publication.

## Acceptance criteria

1. `REQ-CRON-0003`: a rendered project has the opt-in Monty feature and the
   checked example; default generation does not link Monty.
2. ADR-RUN-0013: the example contains no root/program override and invokes no
   host other than the two allowed functions.
3. `NFR-RUN-0005`: generation assertions are deterministic and use no sleep,
   network, globally fixed project path, or external compose process.
4. b-17 can consume the exact feature without reopening a Cargo manifest.
5. REQ-RUN-0309: a checkout-rendered Cargo tree uses the recorded matching
   observability pair and locally patched `sc-cron` at the exact future
   registry version; the default feature tree does not enable
   `sc-cron/monty`. Registry resolution itself belongs to b-17 after publish.
6. REQ-RUN-0702: the two template manifests agree on the complete five-crate
   internal inventory at `0.1.1`; checkout generation patches `sc-cron`
   locally, while release-mode generation emits no checkout patch.

## Required validation

```sh
python3 -m unittest tests.unit.generation.test_monty_example
just new generated/test-monty --var-file wizard/fixtures/sqlite-mcp.json
cargo check --manifest-path generated/test-monty/Cargo.toml -p widget-app-daemon
cargo +1.96.1 check --manifest-path generated/test-monty/Cargo.toml -p widget-app-daemon --features monty
cargo tree --manifest-path generated/test-monty/Cargo.toml -p widget-app-daemon -e features
rg -n 'render_template|sc_compose_run|cron_stop|cron_tick' generated/test-monty/examples/monty/cron.py
rg -n 'sc-(config|transport|command|runtime|cron).*0\.1\.1|sc-observability(-types)?.*1\.5\.1' template/Cargo.toml template/workspace.template.toml
just lint gates
```

The generation test owns cleanup of its temporary output; the literal
`generated/test-monty` commands are the operator reproduction form and require
that destination to be absent before the first command.

## This sprint does not close

- Scheduler construction, injected logging, a running cron job, or any new
  Monty host capability.
