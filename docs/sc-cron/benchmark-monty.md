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
| `scripts/benchmark/check_gate.py` | publish preflight entry (`benchmark-gate`) |
| `scripts/benchmark/render_report.py` | validates `examples.hook_source`, `examples.resume_loop`, and `examples.cli_argv`; `campaign.py --index-only` regenerates index and campaign HTML from those keys and exits non-zero when one is missing |
| `site/reports/benchmark-monty/` | immutable `*.campaign.json`, HTML, `index.html` |
| `crates/sc-cron/tests/sc_compose_bench.rs` | harness (ignored unless `SC_CRON_BENCH=1`) |

Hook interpretation: [`hooks-sc-compose.md`](hooks-sc-compose.md).

## Publish preflight

Release publish runs `benchmark-gate` from
[`release/publish-preflight.toml`](../../release/publish-preflight.toml) on the
controlled host **`atmbench@rand-m5`**. A skipped **`production_bridge`** case
fails the gate. The gate also rejects a host label that is not `rand-m5` and
not listed in `scripts/benchmark/monty/baselines.json` (`allowed_hosts`),
unless `SC_BENCH_GATE_ALLOW_ANY_HOST=1` (local development only). Floors are
read from that baselines file. `site/reports/benchmark-monty/` does not carry
a second copy.
