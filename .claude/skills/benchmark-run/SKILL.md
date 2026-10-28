---
name: benchmark-run
description: Run subsystem benchmark campaigns on the controlled bench host (atmbench@rand-m5), review results, publish to site/reports, and execute benchmark-gate for release preflight. Use for /benchmark-run.
---

# Benchmark run

Governed by [ADR-RUN-0015](../../docs/architecture.md) and
[sprint-b-11](../../docs/plans/phase-b/sprint-b-11-benchmark-monty-reports.md).

Timing data is authoritative only from the **controlled environment**. Primary
host: **`atmbench@rand-m5`** (SSH). GitHub Actions is not a benchmark host.

## Step 1 — Host preflight

1. Confirm the session is on `rand-m5` or SSH there:
   `ssh -o BatchMode=yes atmbench@rand-m5`.
2. Confirm repo checkout matches the commit under test.
3. Confirm `sc-compose` is on `PATH` when running the `monty` subsystem.
4. Confirm Rust **1.96.1** and `cargo test -p sc-cron --features monty` builds.

Set `SC_BENCH_HOST_LABEL` when the recorded label must differ from hostname.

## Step 2 — Run campaign

From the repository root on the controlled host:

```bash
just --justfile scripts/benchmark/justfile benchmark <subsystem>
```

`<subsystem>` is a directory name under `scripts/benchmark/` (for example
`monty`). The runner writes
`site/reports/benchmark-<subsystem>/{timestamp}-{host}.campaign.json` and HTML.

## Step 3 — Review

```bash
just --justfile scripts/benchmark/justfile benchmark-show <subsystem>
```

Open the latest campaign HTML. Confirm required cases ran (for `monty`,
`production_bridge` must not be skipped). Confirm host label, git SHA, and
rustc version in the JSON.

## Step 4 — Publish index

After acceptance:

```bash
just --justfile scripts/benchmark/justfile benchmark-publish <subsystem>
```

## Step 5 — Gate (release preflight)

On the **controlled host only**:

```bash
just --justfile scripts/benchmark/justfile benchmark-gate
```

Publish preflight must invoke this step on `atmbench@rand-m5`, not on a GitHub
hosted runner. Fail closed if the host is wrong or prerequisites are missing.

Do not set `SC_BENCH_GATE_ALLOW_ANY_HOST=1` for release gates.
