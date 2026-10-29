---
id: b-11
phase: b
status: complete
closure_type: integration
target_boundary: benchmark
track: cron-monty
wave: 5
branch: sprint/b-11-benchmark-monty-reports
worktree: sprint/b-11-benchmark-monty-reports
integration_branch: integrate/phase-b-monty
requirements:
  - REQ-CRON-0001
  - REQ-CRON-0003
  - REQ-CRON-0005
  - REQ-RUN-0005
  - NFR-RUN-0005
adrs:
  - ADR-RUN-0002
  - ADR-RUN-0013
  - ADR-RUN-0011
  - ADR-RUN-0015
owned_paths:
  - .claude/skills/benchmark-run/
  - scripts/benchmark/
  - site/reports/benchmark-monty/
  - crates/sc-cron/tests/sc_compose_bench.rs
  - docs/sc-cron/benchmark-monty.md
  - release/sc-publish-consumer-input.json
  - release/publish-preflight.toml
must_follow: [b-10]
parallel_safe: [b-8]
recommended_agent: cipher
recommended_model: luna
---

# Sprint b-11 — Benchmark platform and Monty reports

Plan only. Implementation targets `integrate/phase-b-monty` after `b-10`.
The PR for this sprint targets that branch, not `develop`.

The sprint delivers a **multi-subsystem benchmark platform** and its first
subsystem, `monty` (the compose-hook comparison). Later subsystems add a
directory pair; they do not add a second runner or a second publish path.

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

An operator or agent can run a subsystem's benchmark campaign (automated
procedure below), review it, and publish one immutable campaign under
`site/reports/benchmark-<subsystem>/`. The first subsystem is `monty`; its
published campaign includes `production_bridge` once `b-10` has landed. A
**binary publish is blocked** unless every registered subsystem's benchmark
passes in publish preflight on the **controlled host** ([ADR-RUN-0015](../../architecture.md)).
Default PR CI does not run campaigns or produce timing regressions.

## This sprint does not close

- Implementing `render_template` (`b-10`).
- `docs/sc-cron/hooks-sc-compose.md` (`b-10` owns that note).
- Benchmark subsystems other than `monty`. The platform admits them; this
  sprint registers only `monty`.
- A per-PR regression CI gate. The gate runs in publish preflight only.
- Phase close of waves 1–3.

## Multi-subsystem layout

A **subsystem** is a short lowercase name (`monty` here). Every subsystem owns
exactly two trees, and nothing else:

| Path | Holds |
|---|---|
| `scripts/benchmark/<subsystem>/` | the campaign runner, `baselines.json`, and the subsystem's fixtures |
| `site/reports/benchmark-<subsystem>/` | `*.campaign.json`, the campaign HTML beside each JSON, and `index.html` |

Shared pieces live in `scripts/benchmark/` itself: `justfile` (recipes below),
`check_gate.py` (the preflight gate), and `render_report.py` (JSON → HTML, used
by every subsystem so code examples and layout are uniform). A subsystem is
registered by having a `scripts/benchmark/<subsystem>/campaign.py` that
implements `run`, `--index-only`, and `--gate`; `scripts/benchmark/justfile`
discovers subsystems by that file. There is no separate registry file.

For this sprint the only subsystem is `monty`:
`scripts/benchmark/monty/campaign.py` (replaces the earlier single
`monty_compose_campaign.py`) and `site/reports/benchmark-monty/`.

## Controlled environment

Authoritative benchmark timings come only from a **controlled environment**
([ADR-RUN-0015](../../architecture.md)). The reference host is **`atmbench@rand-m5`**
(SSH). Operators coordinate access to the `atmbench` account; agents run the
skill on that machine (or SSH there and execute the same recipes).

- Campaign JSON and HTML must record **git SHA**, **rustc** version, **host
  label**, and every case result. `SC_BENCH_HOST_LABEL` overrides the default
  hostname when set.
- `baselines.json` ceilings apply to campaigns from approved host labels
  (default allowlist includes `rand-m5` and the label recorded when baselines
  were first accepted). GitHub Actions workflow duration is **not** a baseline
  source.
- `check_gate.py` rejects `--gate` when the host label is not in the allowlist
  unless `SC_BENCH_GATE_ALLOW_ANY_HOST=1` (local dev only; never in publish
  preflight).

## Operator and agent flow

`.claude/skills/benchmark-run/SKILL.md` is the procedure agents invoke
(`/benchmark-run`): preflight on the controlled host, run, review, publish.
First consumer is the `monty` campaign. The skill is subsystem-parameterized
(`<subsystem>` argument) and lists subsystems by directory name; it does not add
a second runner. Steps name the SSH target (`atmbench@rand-m5`) when the session
is not already on that host.

Recipes live in `scripts/benchmark/justfile` (owned here). The root `Justfile`
stays with sprint `b-2`. Invoke with `just --justfile scripts/benchmark/justfile …`.

| Recipe | Does |
|---|---|
| `benchmark <subsystem>` | runs `scripts/benchmark/<subsystem>/campaign.py`; for `monty` that runs `SC_CRON_BENCH=1 cargo test -p sc-cron --features monty --test sc_compose_bench --release -- --ignored --nocapture` and writes `{timestamp}-{host}.campaign.json` plus its HTML |
| `benchmark-show <subsystem>` | prints the latest campaign HTML path and the index path |
| `benchmark-publish <subsystem>` | refreshes `site/reports/benchmark-<subsystem>/index.html` from `*.campaign.json` and leaves that directory as the only publish staging area |
| `benchmark-gate` | runs every registered subsystem's `campaign.py --gate` (publish preflight entry point, below) |

`SC_BENCH_HOST_LABEL` overrides the host label. Default is the machine
hostname. The runner records git SHA, `rustc` version, case id, ms/run, and
RSS notes into `{timestamp}-{host}.campaign.json`.

## Publish preflight gate

**Replaces** the earlier "campaign is manual only / no regression gate" rule.
Publishing a reviewed campaign to `site/reports/` (`benchmark-publish`) stays
an operator or agent step after review. The **pass/fail gate** is automated on
the controlled host: a binary publish cannot proceed unless it passes.

- `release/publish-preflight.toml` declares a check `benchmark-pass` with
  `run = "just --justfile scripts/benchmark/justfile benchmark-gate"`.
  `.github/scripts/run-consumer-preflight.sh` runs every `[[checks]]` entry
  from `release-preflight.yml` and fails closed before release dispatch, so a
  non-zero `benchmark-gate` blocks binary publish.
- **Where the gate runs:** publish preflight invokes the skill or an SSH step
  that executes `benchmark-gate` **on `atmbench@rand-m5`**, not on a GitHub
  hosted runner. The release workflow may SSH to rand-m5 or fail with a clear
  message when the controlled host is unreachable. Timing regressions are never
  inferred from Actions job duration.
- `release/publish-preflight.toml` is rendered from
  `release/publish-preflight.toml.j2`. The check is added through the
  `preflight_checks` list in `release/sc-publish-consumer-input.json` and the
  toml is re-rendered, not hand-edited.
- `benchmark-gate` does a **fresh** run per registered subsystem (a stale
  committed campaign never passes) with `--gate` on the controlled host. A
  subsystem passes only when:
  1. every required case ran (none skipped; `monty` requires
     `production_bridge`);
  2. each case's rendered output equals the expected string (a wrong render is
     a failure, not a fast sample);
  3. each case is within its `baselines.json` ceiling
     (`floor_ms * max_regression_ratio`, recorded from the first accepted
     campaign on the controlled host);
  4. the subsystem wrote its campaign JSON and HTML with an allowed host
     label.
- The gate host must provide what the gate needs (`sc-compose` on `PATH`,
  Rust 1.96.1 for `--features monty`). A missing prerequisite **fails** the
  gate; it is never a skip.
- Default PR CI and root `just test` do not run `benchmark-gate`.

## Campaign cases (`monty`)

| Case id | What it measures |
|---|---|
| `cpython_hook` | `HookTarget::Python` invoke path |
| `python_render_only` | same render, no sc-cron wrapper |
| `sc_compose_cli_only` | `sc-compose render` with no Monty |
| `monty_host_bridge` | lab `MontyRepl::feed_start` plus host resume to the CLI |
| `production_bridge` | `cron_tick` through `execute_feed` calling `render_template` (the bench must not call `feed_start` itself; `execute_feed` may use `feed_start` internally) |

`production_bridge` is skipped, and the campaign JSON records the skip, when
the `b-10` path is absent, **in a local run**. Under `--gate` that skip is a
failure. A publish that claims a `production_bridge` sample has to have run
that case.

## HTML reports must show code, not only timings

Every campaign HTML page and every `index.html` entry for a campaign must
include **code examples** beside the timing table. A table of ms/run alone does
not satisfy this sprint. Required example blocks, per campaign, rendered as
`<pre><code>`:

1. **Hook snippet:** the exact Monty hook source the Monty cases run (a
   `cron_tick` that calls `render_template(template, context)`), and the
   Python fixture body for `cpython_hook`.
2. **`feed_start` / resume loop:** the host loop that takes `ReplProgress`,
   matches the `render_template` `FunctionCall`, runs the CLI, and calls
   `resume` until `Complete`. For `monty_host_bridge` it is the lab's loop; for
   `production_bridge` it is the `execute_feed` excerpt, labelled as the
   production path.
3. **`sc-compose` CLI argv:** the exact argv the host spawns
   (`sc-compose render --root <temp> --mode file --file t.j2 --var-file v.json`),
   as run, with `<temp>` shown as a placeholder.

The examples are **data in the campaign JSON** (`examples.hook_source`,
`examples.resume_loop`, `examples.cli_argv`), captured from the constants the
benchmark actually executes, not retyped prose. `render_report.py` renders
them, so `--index-only` regenerates the same blocks from stored JSON without
running cargo. A campaign JSON without all three `examples` keys fails
`--gate` and `--index-only`.

Starting point: the bench's current HTML generator already embeds static
excerpt constants (`EXAMPLE_MONTY_HOOK`, `EXAMPLE_RESUME_LOOP`,
`EXAMPLE_CLI_ARGV`) in a "Code examples" section. This sprint replaces them
with values captured into the campaign JSON.

## Site tree

`site/reports/benchmark-<subsystem>/` (here `benchmark-monty/`):

- `index.html`, regenerated from `*.campaign.json` by the publish recipe.
- one campaign HTML per `*.campaign.json`, written beside it.
- `baselines.json` lives in `scripts/benchmark/<subsystem>/` and is read by the
  index and the gate; the index copies the floor it displays into the page.

`baselines.json` consumer: the gate, and a person reading the index.
Capability: the gate fails a host-bridge regression automatically, and the
index shows each case against the last accepted floor so a regression is
visible without re-reading old HTML. Observed defect: an earlier lab resumed
`render_template` by spawning `python3 -c import sc_compose` and published
that as the Monty result. The floor is the CLI-backed sample, not that run.

The bench writes campaign HTML beside the JSON under
`site/reports/benchmark-monty/`. Harness HTML and JSON sidecars live under
`target/sc-compose-bench/` (`sc-compose-hook-comparison.report.html` and
`.report.json`), not under `docs/sc-cron/`. The bench's temp root
is a `tempfile::TempDir`, not `std::env::temp_dir().join("sc_cron_sc_compose_bench")`
(`NFR-RUN-0005`).

`docs/sc-cron/benchmark-monty.md` points at the skill, the `monty` index, and
the preflight gate. It does not edit `hooks-sc-compose.md`.

## Deliverables

1. `REQ-CRON-0003` / ADR-RUN-0013 (the `b-10` host-bridge sentence): the `production_bridge` case in `sc_compose_bench` and in the campaign JSON.
2. `NFR-RUN-0005`: TempDir in the harness; default `cargo test -p sc-cron` does not run the ignored bench; the bench does not set process environment variables except reading `SC_CRON_BENCH` and `SC_BENCH_HOST_LABEL`.
3. The multi-subsystem layout: `scripts/benchmark/justfile` (four recipes above), `render_report.py`, `check_gate.py` (if the gate logic is not inside each `campaign.py`), and the `monty` subsystem (`campaign.py`, `baselines.json`, `site/reports/benchmark-monty/index.html`).
4. The HTML code-example requirement: `examples.*` captured in campaign JSON and rendered in campaign HTML and `index.html`.
5. The publish preflight gate: `benchmark-pass` check in `release/sc-publish-consumer-input.json` → re-rendered `release/publish-preflight.toml`.
6. The subsystem-parameterized `.claude/skills/benchmark-run/SKILL.md` and `docs/sc-cron/benchmark-monty.md`.

## Acceptance criteria

- Root `adr:ADR-RUN-0013`: after `b-10`, `just --justfile scripts/benchmark/justfile benchmark monty` writes a campaign JSON whose `production_bridge` case calls **`execute_feed` only** (the bench source must not call `feed_start`; `feed_start` inside `execute_feed` is allowed) and whose render path is `sc-compose` with no `python3` child. The JSON records git SHA and rustc version.
- Root `req:NFR-RUN-0005`: the harness uses `tempfile::TempDir`; `cargo test -p sc-cron` exits 0 without `SC_CRON_BENCH`; the ignored test is not part of default CI.
- Root `req:NFR-RUN-0005`: `just --justfile scripts/benchmark/justfile benchmark-publish monty` rewrites only `site/reports/benchmark-monty/index.html` from the campaign files, and the index links each campaign and shows the `baselines.json` floor.
- Root `adr:ADR-RUN-0013`: the skill's four steps name `scripts/benchmark/justfile` recipes and `site/reports/benchmark-<subsystem>/`. `docs/sc-cron/benchmark-monty.md` links the skill, the index, and the gate.
- Layout: no benchmark script or report path outside `scripts/benchmark/<subsystem>/`, `scripts/benchmark/` shared files, and `site/reports/benchmark-<subsystem>/`; a second subsystem directory pair is picked up by `benchmark-gate` and `benchmark-publish` with no edit to the justfile.
- Code examples: the generated `monty` campaign HTML and `index.html` each contain a `<pre><code>` block with the hook source (`cron_tick`), one with the resume loop (`feed_start` and `resume`), and one with the CLI argv (`sc-compose render --root`). A campaign JSON missing any `examples.*` key makes `render_report.py` exit non-zero.
- Gate: `just --justfile scripts/benchmark/justfile benchmark-gate` exits 0 on a host with prerequisites and a passing campaign, and non-zero when `production_bridge` is skipped, a render mismatches, or a case exceeds its ceiling. `release/publish-preflight.toml` contains the `benchmark-pass` check, and `.github/scripts/run-consumer-preflight.sh` fails when the gate exits non-zero.

## Required validation

- `cargo test -p sc-cron` exits 0 (bench stays ignored; default features do not link `monty`).
- `python3 scripts/benchmark/monty/campaign.py --index-only` exits 0 against a fixture `*.campaign.json` (with all three `examples` keys) and refreshes the index and HTML without running cargo; it exits non-zero against a fixture missing `examples.hook_source`. The script grows that flag in this sprint.
- `python3 scripts/benchmark/monty/campaign.py --gate` exits non-zero when run against a fixture campaign that skipped `production_bridge`.
- Root `just test` and default PR CI do not invoke `benchmark` or `benchmark-gate`. `release-preflight.yml` does, through `release/publish-preflight.toml`.
- `sc-lint lint sc-boundary --root . --config sc-lint.toml` exits 0. This sprint does not edit `boundaries/`.

## Non-goals

No new `CronError` variant (`REQ-CRON-0005`). No workspace crate dependency on
sc-cron (`REQ-CRON-0001`). No boundary-manifest edit (`REQ-RUN-0005`). No edit to
`monty_run.rs`, `monty_host.rs`, `hooks-sc-compose.md`, or `docs/architecture.md`.
No per-PR benchmark CI. No second campaign runner per subsystem beyond that
subsystem's `campaign.py`. No benchmark subsystem other than `monty`.
