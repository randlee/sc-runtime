---
name: benchmark-run
description: Run, review, and publish sc-runtime benchmark campaigns (Monty compose first).
---

# Benchmark run (sc-runtime)

This is the operator procedure for sc-runtime benchmark campaigns. It follows
the atm-core `benchmark-run` shape: preflight, run, review, publish. Paths and
recipes are sc-runtime specific. Sprint documents keep the historical decisions;
this skill is how an operator produces the next campaign.

## Benchmarks catalog

| Campaign | Command | Published under |
|----------|---------|-----------------|
| Monty compose hooks | `just --justfile scripts/benchmark/justfile benchmark-monty` | `site/reports/benchmark-monty/` |

Underlying harness: `crates/sc-cron/tests/sc_compose_bench.rs`. Interpretation
notes: `docs/sc-cron/hooks-sc-compose.md`.

The harness writes HTML and JSON under `target/sc-compose-bench/`. The publisher
turns those into immutable `{campaign-id}.campaign.json` and `{campaign-id}.html`
files and regenerates `index.html` from every `*.campaign.json`. Default CI
does not run this campaign.

## 1. Preflight

Authoritative timings come from the controlled host **`atmbench@rand-m5`**
([ADR-RUN-0015](../../../docs/architecture.md)). When this session is not
already on that host, SSH there and continue in a checkout of the revision
you intend to measure:

```sh
ssh atmbench@rand-m5
```

On `atmbench@rand-m5`, the tree must be clean before you publish. Install the
Monty toolchain (Rust 1.96.1) and have `sc-compose` on `PATH`. Python cases
also need `import sc_compose`.

```sh
git status --short
command -v sc-compose
python3 -c "import sc_compose" || true
rustup run 1.96.1 rustc --version
hostname -s
```

`hostname -s` on the controlled host is `rand-m5`. `SC_BENCH_HOST_LABEL`
overrides the label recorded in the campaign. The publish gate allowlist is
`rand-m5` plus every label in `scripts/benchmark/<subsystem>/baselines.json`
(`allowed_hosts`). `SC_BENCH_GATE_ALLOW_ANY_HOST=1` bypasses that check for
local development only. Do not set it for publish preflight.

Keep the host quiet. A campaign is a measurement of this checkout, this
toolchain, and this machine. Do not run it on a shared developer session that
is also compiling the workspace.

## 2. Run

Run the recipe on `atmbench@rand-m5` (the SSH session from preflight when you
were not already on that host):

```sh
just --justfile scripts/benchmark/justfile benchmark monty
```

`benchmark-monty` is the same Monty campaign. The recipe runs release
`sc_compose_bench` with `SC_CRON_BENCH=1` on Rust 1.96.1, then writes the
campaign under `site/reports/benchmark-monty/`. A campaign id looks like
`20261027T180000Z-rand-m5`. The runner refuses to overwrite an existing
campaign file.

`scripts/benchmark/monty/baselines.json` holds optional `max_ms_per_run`
floors. A null floor does not fail a case. When a floor is set and a measured
case is slower, the campaign status is `FAIL`. The result is still written.
`production_bridge` must run through `Scheduler::run_due` → `drive_feed` →
`execute_feed_with_compose_program` (not cfg(test) `execute_feed`). A skip is not a
performance result and fails `benchmark-gate`.

`just --justfile scripts/benchmark/justfile benchmark-gate` and
`scripts/benchmark/monty/campaign.py --gate` reject a host label outside the
allowlist before they measure anything, unless
`SC_BENCH_GATE_ALLOW_ANY_HOST=1`.

A harness or environment error before measurement (missing toolchain, missing
`sc-compose`, test failure before harness output exists) exits non-zero and writes
no campaign. Fix the cause and run again.

## 3. Review

```sh
just --justfile scripts/benchmark/justfile benchmark-monty-show
```

The command rebuilds `index.html` from the campaign JSON, prints the newest
campaign HTML, and opens it when a desktop opener exists. Confirm:

- host label and git SHA
- cargo profile `release`
- every expected case is present (`cli`, `py_render`, `py_hook`, `monty`,
  `production_bridge`)
- status is `PASS`, `MEASURED`, `FAIL`, or `INCOMPLETE`, and you know why

`MEASURED` means the run finished and no floor was configured for at least one
measured case. It is not a pass. Compare the HTML with
`baselines.json` before treating a number as a regression.

## 4. Publish

Publish every measured campaign after review, including `FAIL` and
`INCOMPLETE`. The helper stages only `site/reports/benchmark-monty/` and
refuses to run when the index already has other staged files. It does not
commit.

```sh
just --justfile scripts/benchmark/justfile benchmark-monty-publish
git commit -m "evidence(benchmark-monty): <campaign-id> <host-label>"
git push
```

`just --justfile scripts/benchmark/justfile benchmark-monty-publish` regenerates the index and checks that the
checked-in `index.html` matches the campaign JSON. Do not amend or delete an
older campaign to make a result look better. Changing a floor in
`baselines.json` requires the new floor, the campaigns that justify it, and a
rationale in the same commit. Existing campaign JSON stays immutable.

## 5. Failure policy

- Harness or environment error before measurement: not a campaign. Fix it and
  rerun. Do not publish a partial tree.
- Measured `FAIL`: publish it, then root-cause the regression on a product
  branch.
- `INCOMPLETE`: publish it. The reason is in the campaign JSON. Run a new
  campaign after the harness or environment is fixed.
- A stale `index.html` is unsafe to push. Stop and rerun
  `just --justfile scripts/benchmark/justfile benchmark-monty-publish` so the index matches the JSON.

## Extending

Add a catalog row, a `just benchmark-<name>` recipe, a runner under
`scripts/benchmark/`, and a subdirectory under `site/reports/`. Keep default
CI free of benchmark workloads. Campaign JSON stays the source the index is
regenerated from.
