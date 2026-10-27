---
id: b-11
phase: b
status: planned
closure_type: integration
target_boundary: benchmark-monty
track: cron-monty
wave: 5
branch: sprint/b-11-benchmark-monty-reports
worktree: sprint/b-11-benchmark-monty-reports
integration_branch: integrate/phase-b-monty
requirements:
  - REQ-CRON-0003
  - NFR-RUN-0005
adrs:
  - ADR-RUN-0013
  - ADR-RUN-0011
owned_paths:
  - .claude/skills/benchmark-run/
  - site/reports/benchmark-monty/
  - scripts/benchmark/
  - Justfile
  - crates/sc-cron/tests/sc_compose_bench.rs
  - docs/sc-cron/benchmark-monty.md
must_follow: [b-10]
parallel_safe: [b-8]
recommended_agent: cipher
recommended_model: luna
---

# Sprint b-11 — Monty benchmark reports

Plan only. Implementation targets `integrate/phase-b-monty` after `b-10`.
The PR for this sprint targets that branch, not `develop`.

## Relations

`must_follow` `b-10`. The artifact is `monty_host::render_template`, resumed
inside `monty_run::execute_feed`. The campaign case `production_bridge` is a
Monty `cron_tick` that calls that function and records the CLI render. The
lab case `monty_host_bridge` is a standalone `MontyRepl::feed_start` in the
bench and is not that artifact.

`owned_paths` do not intersect `b-10`. The serial edge is the ext fn, not a
shared file. Publishing only the lab cases could share a wave with `b-10`.
That publisher would be a thin sprint: **#73** already runs the lab, and the
acceptance below still waits on `production_bridge`. The plan keeps one
integration sprint. Extension width stays 1. See
[phase-b-integration-close.md](phase-b-integration-close.md).

`parallel_safe` with `b-8` and with phase-end review on `integrate/phase-b`.

## Goal

An operator can run the compose-hook comparison, review it, and publish one
immutable campaign under `site/reports/benchmark-monty/`. The published
campaign includes `production_bridge` once `b-10` has landed. Default CI does
not run it.

## This sprint does not close

- Implementing `render_template` (`b-10`).
- `docs/sc-cron/hooks-sc-compose.md` (`b-10` owns that note).
- A regression CI gate. The campaign is manual.
- Phase close of waves 1–3.

## Operator flow

`.claude/skills/benchmark-run/SKILL.md` is the procedure: preflight, run,
review, publish. First consumer is this campaign. The skill lists later
campaigns by directory name; it does not add a second runner in this sprint.

| Recipe | Does |
|---|---|
| `just benchmark-monty` | `scripts/benchmark/monty_compose_campaign.py` runs `SC_CRON_BENCH=1 cargo test -p sc-cron --features monty --test sc_compose_bench --release -- --ignored --nocapture` |
| `just benchmark-monty-show` | prints the latest campaign HTML path and the index path |
| `just benchmark-monty-publish` | refreshes `site/reports/benchmark-monty/index.html` from `*.campaign.json` and leaves that directory as the only publish staging area |

`SC_BENCH_HOST_LABEL` overrides the host label. Default is the machine
hostname. The runner records git SHA, `rustc` version, case id, ms/run, and
RSS notes into `{timestamp}-{host}.campaign.json`.

## Campaign cases

| Case id | What it measures |
|---|---|
| `cpython_hook` | `HookTarget::Python` invoke path |
| `python_render_only` | same render, no sc-cron wrapper |
| `sc_compose_cli_only` | `sc-compose render` with no Monty |
| `monty_host_bridge` | lab `MontyRepl::feed_start` plus host resume to the CLI |
| `production_bridge` | `cron_tick` through `execute_feed` calling `render_template` |

`production_bridge` is skipped, and the campaign JSON records the skip, when
the `b-10` path is absent. A publish that claims a `production_bridge` sample
has to have run that case.

## Site tree

`site/reports/benchmark-monty/`:

- `index.html`, regenerated from `*.campaign.json` by the publish recipe.
- `baselines.json`, the floor the index compares against.

`baselines.json` consumer: a person reading the index. Capability: the index
shows each case against the last accepted floor so a host-bridge regression is
visible without re-reading old HTML. Observed defect: an earlier lab resumed
`render_template` by spawning `python3 -c import sc_compose` and published
that as the Monty result. The floor is the CLI-backed sample, not that run.
The file stays while this report directory is published. It is not a CI gate.

The bench writes campaign HTML beside the JSON under
`site/reports/benchmark-monty/`. It stops writing
`docs/sc-cron/sc-compose-hook-comparison.report.html`. The bench's temp root
is a `tempfile::TempDir`, not `std::env::temp_dir().join("sc_cron_sc_compose_bench")`
(`NFR-RUN-0005`).

`docs/sc-cron/benchmark-monty.md` points at the skill and the index. It does
not edit `hooks-sc-compose.md`.

## Deliverables

1. `REQ-CRON-0003` / ADR-RUN-0013 (the `b-10` host-bridge sentence): the `production_bridge` case in `sc_compose_bench` and in the campaign JSON.
2. `NFR-RUN-0005`: TempDir in the harness; default `cargo test -p sc-cron` does not run the ignored bench; the bench does not set process environment variables except reading `SC_CRON_BENCH` and `SC_BENCH_HOST_LABEL`.
3. The skill, the three `just` recipes, the runner, `index.html`, and `baselines.json`.
4. `docs/sc-cron/benchmark-monty.md`.

## Acceptance criteria

- Root `adr:ADR-RUN-0013`: after `b-10`, `just benchmark-monty` writes a campaign JSON whose `production_bridge` case ran through `execute_feed` (not `feed_start`) and whose render path is `sc-compose` with no `python3` child. The JSON records git SHA and rustc version.
- Root `req:NFR-RUN-0005`: the harness uses `tempfile::TempDir`; `cargo test -p sc-cron` exits 0 without `SC_CRON_BENCH`; the ignored test is not part of default CI.
- Root `req:NFR-RUN-0005`: `just benchmark-monty-publish` rewrites only `site/reports/benchmark-monty/index.html` from the campaign files, and the index links each campaign and shows the `baselines.json` floor.
- The skill's four steps name these recipes and the output directory. `docs/sc-cron/benchmark-monty.md` links the skill and the index.

## Required validation

- `cargo test -p sc-cron` exits 0 (bench stays ignored; default features do not link `monty`).
- `python3 scripts/benchmark/monty_compose_campaign.py --index-only` exits 0 against a fixture `*.campaign.json` and refreshes the index without running cargo. The script grows that flag in this sprint.
- `just benchmark-monty` is not invoked by `just test` or by CI.
- `sc-lint lint sc-boundary --root . --config sc-lint.toml` exits 0. This sprint does not edit `boundaries/`.

## Non-goals

No new `CronError` variant. No boundary-manifest edit. No edit to
`monty_run.rs`, `monty_host.rs`, `hooks-sc-compose.md`, or `docs/architecture.md`.
No default-CI benchmark. No second campaign runner.
