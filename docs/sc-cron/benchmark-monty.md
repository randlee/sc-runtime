# Monty compose benchmark (sc-cron)

Operator doc for the **`monty`** benchmark subsystem. Sprint plan:
[`sprint-b-11-benchmark-monty-reports.md`](../plans/phase-b/sprint-b-11-benchmark-monty-reports.md).

## Procedure

Follow [`.claude/skills/benchmark-run/SKILL.md`](../../.claude/skills/benchmark-run/SKILL.md)
(`/benchmark-run monty`). Run on a controlled host when timings matter
([ADR-RUN-0015](../architecture.md)).

## Recipes

From repo root:

```bash
just --justfile scripts/benchmark/justfile benchmark monty
just --justfile scripts/benchmark/justfile benchmark-show monty
just --justfile scripts/benchmark/justfile benchmark-publish monty
just --justfile scripts/benchmark/justfile benchmark-gate
```

Legacy names `benchmark-monty*` call the same Monty campaign.

## Layout

| Path | Role |
|------|------|
| `scripts/benchmark/monty/campaign.py` | runs release `sc_compose_bench`, packages campaigns |
| `scripts/benchmark/monty/baselines.json` | optional ms/run floors per case id |
| `scripts/benchmark/check_gate.py` | fresh controlled-host execution (`benchmark-gate`) |
| `scripts/benchmark/check_release_evidence.py` | hosted preflight verification of committed reports and measured-source equivalence |
| `scripts/benchmark/render_report.py` | validates `examples.hook_source`, `examples.resume_loop`, and `examples.cli_argv`; `campaign.py --index-only` regenerates index and campaign HTML from those keys and exits non-zero when one is missing |
| `site/reports/benchmark-monty/` | immutable `*.campaign.json`, HTML, `index.html` |
| `crates/sc-cron/tests/sc_compose_bench.rs` | harness (ignored unless `SC_CRON_BENCH=1`) |
| `target/sc-compose-bench/` | harness HTML and JSON that `campaign.py` packages |

Hook interpretation: [`hooks-sc-compose.md`](hooks-sc-compose.md).

## Publish preflight

A developer runs a fresh campaign from clean, pushed workspace source on
**`atmbench@rand-m5`**, using the procedure above or the optional workstation
SSH producer `scripts/benchmark/run_remote_gate.py`. Submit the immutable
JSON/HTML and regenerated index under `site/reports/benchmark-monty/`, then
have quality manager independently review the campaign and measured build.
First-release measurement does not require an installed prerelease archive or
already-published SDK.

Hosted publish preflight runs `scripts/benchmark/check_release_evidence.py`
through [`release/publish-preflight.toml`](../../release/publish-preflight.toml).
It never connects to the controlled host. It requires committed canonical
report pairs/index, matching HTML measurements, five complete cases without
skips, and current baseline semantics. It selects the newest campaign matching
the release source before accepting its status: a later applicable failure or
invalid result cannot be hidden by an older success. Source changes beyond
added report pairs/index updates invalidate evidence; identical source across
a squash merge remains eligible. Existing quality review and release-candidate
tag policy apply, without a new signing or approval-record framework.

Ceilings are read from `scripts/benchmark/monty/baselines.json`. Null ceilings
produce `MEASURED`: complete execution evidence, not a numerical performance
PASS. A skipped case, FAIL, INCOMPLETE, malformed report, or stale index blocks
preflight. Publish all measured attempts unchanged, including failures.
Historical Mac campaigns remain labeled `Mac`; they do not establish
controlled-host release evidence. PR CI does not run fresh benchmark campaigns.
