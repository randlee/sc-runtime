---
id: b-16
phase: b
status: planned
closure_type: integration
target_boundary: generated-daemon-monty
track: monty-product
wave: 8
branch: sprint/b-16-monty-template-examples
worktree: sprint/b-16-monty-template-examples
integration_branch: integrate/phase-b-observability
requirements:
  - REQ-RUN-0003
  - REQ-RUN-0702
  - REQ-RUN-0309
  - REQ-CRON-0001
  - REQ-CRON-0003
  - NFR-CRON-0001
  - NFR-RUN-0005
adrs:
  - ADR-RUN-0002
  - ADR-RUN-0013
  - ADR-RUN-0303
owned_paths:
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
vertical_rationale: One generated-product boundary owns the Cargo feature, checked example, documentation, and render assertion; splitting it would create pass-through sprints without parallel work.
---

# Sprint b-16 — generated Monty feature and checked examples

## Goal

Expose the landed Track B Monty host contract in generated projects through
an opt-in Cargo feature and one checked example. This is the remaining Monty
product-surface work; it does not reopen b-10 through b-12.

## Relations

The baseline is `develop` after Track B Monty merged at `240e445`. The
retained `must_follow` b-10 edge names the concrete `render_template` and render-only
`sc_compose_run` host artifact that this example consumes. It also follows
b-13 for amended REQ-CRON-0001 and ADR-RUN-0303 graph authority. This sprint is
`parallel_safe` with b-14 and b-15 because it alone owns the shared
generated Cargo files and its example/docs/test paths. b-17 must merge b-16
first because it consumes the `cron`/`monty` feature artifact.

## Consumed artifact

The example consumes exactly `render_template` and the render-only
`sc_compose_run`. It cannot select a root, program, or another compose
subcommand. Program selection remains embedder policy through
`MontyHostConfig`: the default is the basename `sc-compose` on `PATH`, and an
application may select any other non-empty program path implementing the same
host contract. `sc-compose` is this repository's reference CLI for tests,
benchmarks, and examples, not a required dependency of every application.
The template Cargo files are owned only by this sprint. b-17
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
uses caret-compatible `1.5.1` requirements for the published
`sc-observability` / `sc-observability-types` pair. Neither feature joins
`default`. A normal checkout render therefore
patches `sc-cron` to the local workspace just like the other internal crates,
while `scripts/new_project.py --release` suppresses every checkout patch.

## Required work and deliverables

1. REQ-CRON-0001 / REQ-CRON-0003 / ADR-RUN-0303: add optional daemon dependency `sc-cron`, direct
   `sc-observability-types`, and features equivalent to
   `cron = ["dep:sc-cron"]` and
   `monty = ["cron", "sc-cron/monty"]`; preserve the default feature set.
2. ADR-RUN-0002 / REQ-RUN-0003 / REQ-RUN-0702: update both template dependency
   inventories for the `0.1.1` internal cut, including `sc-cron`, and set the
   published `sc-observability` / `sc-observability-types` requirements to
   caret-compatible `1.5.1`; generated lockfiles prove exact resolution.
   `template/Cargo.toml` must contain the same five internal crate names used
   by checkout patch generation; `template/workspace.template.toml` must
   contain their `0.1.1` registry requirements. These shared Cargo artifacts are
   consumed by b-17 and owned only here.
3. REQ-CRON-0003: add `template/examples/monty/cron.py` with `cron_stop` and `cron_tick`.
   The example calls both allowed hosts, treats `sc_compose_run` as
   render-only, and returns the current stop decision.
4. ADR-RUN-0013 / NFR-CRON-0001: create
   `docs/sc-cron/monty-generated-example.md` documenting the generated path,
   feature enablement, input/output shape, and security limits. Monty source
   is trusted, embedding-chosen in-process source, not a sandbox:
   applications may provide trusted Monty source, untrusted source is
   forbidden, and the render-only host allowlist does not remove
   stack-overflow or allocator-abort risk. Do not shorten or replace the
   landed ADR-RUN-0013 section 4 or REQ-CRON-0003 host contract.
5. NFR-RUN-0005: add a generation test that renders the example and compiles the generated
   daemon with and without `monty`. Assert that a checkout render contains the
   local `sc-cron` patch and a `--release` render contains no
   `[patch.crates-io]` or path dependency; b-17 owns the registry build after
   publication.

## Acceptance criteria

1. `REQ-CRON-0001`: `test_monty_example` and the two generated `cargo check`
   commands pass with the generated daemon as the only generated crate with
   the optional `sc-cron` edge; the feature-off build does not link it.
2. `REQ-CRON-0003`: `test_monty_example` passes with the opt-in Monty feature
   and checked example present; default generation does not link Monty.
3. `NFR-CRON-0001`: the required `cargo tree` command passes inspection with
   no Monty crate in the default tree and no `monty-pool`, `monty-fs`, or
   `monty-runtime` in the Monty tree.
4. `NFR-RUN-0005`: `test_monty_example` passes using a temporary output and
   no sleep, network, globally fixed project path, or external compose process.
5. ADR-RUN-0013: `test_monty_example` passes only when the example contains no
   root/program override, invokes no host other than the two allowed
   functions, and labels its source trusted in-process code with accepted
   abort risk rather than a sandbox.
6. ADR-RUN-0303: the required `cargo tree` and manifest `rg` commands pass
   with the recorded matching observability pair and locally patched
   `sc-cron`; the default feature tree does not enable `sc-cron/monty`.
   Registry resolution itself belongs to b-17 after publish.
7. ADR-RUN-0002: the manifest `rg` check and `test_monty_example` pass only
   when the two template manifests agree on the complete five-crate
   internal inventory at `0.1.1`; checkout generation patches `sc-cron`
   locally, while release-mode generation emits no checkout patch.
8. `test_monty_example` passes with b-17 able to consume the exact feature
   names without reopening a Cargo manifest.

## Required validation

```sh
python3 -m unittest tests.unit.generation.test_monty_example
just new generated/test-monty --var-file wizard/fixtures/sqlite-mcp.json
cargo check --manifest-path generated/test-monty/Cargo.toml -p widget-app-daemon
cargo +1.96.1 check --manifest-path generated/test-monty/Cargo.toml -p widget-app-daemon --features monty
cargo tree --manifest-path generated/test-monty/Cargo.toml -p widget-app-daemon -e features
cargo tree --manifest-path generated/test-monty/Cargo.toml -p widget-app-daemon -e features --features monty
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
