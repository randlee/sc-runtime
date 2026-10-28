# sc-compose from cron hooks

This note is the reference for rendering templates inside a schedule hook.
It matches what the **`sc_compose_hook`** and **`sc_compose_bench`** integration
tests exercise.

## Recommendation

**For compose-heavy hooks (render most ticks), use Monty + the host `sc-compose`
bridge.** Production resumes `render_template` inside `execute_feed` by spawning
the Rust **`sc-compose` CLI**. Benchmarks on this repo show that pattern
matches calling the CLI directly (~3 ms/run, ~6 MiB compose subprocess peak)
and is roughly **7× faster** with **~2× lower RSS** than a
**`HookTarget::Python`** hook that imports PyO3 `sc_compose` (~23 ms/run,
~14 MiB). Regenerate numbers with the bench command under
[Benchmarks](#benchmarks). Sprint `b-11` publishes campaign HTML under
`site/reports/benchmark-monty/`. This note does not link a report file.

**What ships:** production Monty feeds run through `monty_run::execute_feed`.
That function may call `MontyRepl::feed_start` internally and resume
`render_template` by spawning **`sc-compose`** on `PATH` against a temp root
the hook cannot choose. The tick does not start CPython.

**Program name:** production always spawns `sc-compose`. That name is not an
embedder setting. Install the Rust `sc-compose` binary on `PATH`, or symlink
that name onto `PATH`. A missing binary records `Failed` with
`COMPOSE_NOT_FOUND`. The blocking-pool thread stalls until that child exits.
After the child exits, a host `Instant` past the row budget records `TimedOut`.

The standalone lab
`MontyRepl::feed_start` loop in `sc_compose_bench` is not the production path.

Do not import PyO3 `sc_compose` inside the Monty VM; sc-cron does not use
`monty-pool` / `ExternalModule` workers. A tick that never calls
`render_template` still runs when `sc-compose` is absent.

Do not confuse this with **`pydantic_monty` + `ExternalModule` + PyO3
`sc_compose`**: that loads the extension inside a pool worker (full CPython in
the worker). sc-cron uses in-process **`monty` 1.4** without `monty-pool`.

## Hook JSON

Python and Monty both receive **`sc-cron.hook.v1`** on the argument object
(`protocol`, `event`, `schedule`, `payload`, `deadline`). Put template inputs
in **`payload`**, for example:

```json
{
  "template": "Hello {{ name }}",
  "context": { "name": "cron" }
}
```

For real `.j2` files, put paths and var files in `payload` and use
**`sc_compose.compose_file`** or **`sc-compose render --mode file`** in the
Python hook (same as `scripts/new_project.py`).

## Python hook (reference)

Fixture: `crates/sc-cron/tests/fixtures/sc_compose_tick.py`.

- **`cron.stop`**: return `{"action": "continue", "reason": "ok"}`.
- **`cron.tick`**: read hook JSON from stdin, call
  `sc_compose.render_template(payload["template"], payload["context"])`, exit 0.

Register the schedule with `HookTarget::python(module_path)` pointing at that
file. Requires **`sc_compose`** on the host (`pip install sc-compose`).

## Monty hook (with compose via host bridge)

In **`cron_tick`** or **`cron_stop`**, call host-provided
**`render_template(template, context)`** or **`sc_compose_run(subcommand, args)`**.
Production resumes those calls inside `execute_feed`, which spawns Rust
**`sc-compose`**, not `python3 -c import sc_compose`.

- **`render_template`** stages a temp-root string template and JSON context
  (fixed `sc-compose render --mode file` argv).
- **`sc_compose_run`** runs any **allowlisted** top-level subcommand with an
  optional list of argument strings. It returns a dict
  `{stdout, stderr, exit_code}`. Non-zero CLI exit does not fail the tick
  unless spawn, timeout, or output validation fails. Unknown subcommands record
  `COMPOSE_SUBCOMMAND`.

Pure in-VM logic (no compose) stays in Monty without calling either host
function; state can persist in the per-schedule REPL across ticks. The bench's
direct `MontyRepl::feed_start` loop is a lab excerpt, not the path a schedule
row runs.

## Benchmarks

Operator procedure: [`.claude/skills/benchmark-run/SKILL.md`](../../.claude/skills/benchmark-run/SKILL.md).

```bash
just --justfile scripts/benchmark/justfile benchmark-monty
just --justfile scripts/benchmark/justfile benchmark-monty-show
just --justfile scripts/benchmark/justfile benchmark-monty-publish
```

Published campaign HTML lives under
[`site/reports/benchmark-monty/`](../../site/reports/benchmark-monty/).
The harness writes gitignored sidecars next to this note
(`sc-compose-hook-comparison.report.html` and `.report.json`). There is no
checked-in copy of that HTML. Default CI does not run the benchmark.
`production_bridge` is the production `execute_feed` path via
`Scheduler::run_due` (the harness does not call `feed_start` itself). See
[`benchmark-monty.md`](benchmark-monty.md) for campaigns and the publish gate.

Optional direct harness invocation (same measurement, no site publish):

```bash
# Requires sc_compose importable and feature monty for the Monty lab case.
SC_CRON_BENCH=1 cargo test -p sc-cron --features monty --test sc_compose_bench -- --ignored --nocapture
```

They print wall time per run and, on macOS, approximate **subprocess peak RSS**
for the Python paths (one-shot `python3 -c` via `/usr/bin/time -l`; the hook
path uses the same render script—add interpreter + sc-cron child overhead on top):

| Case | Meaning |
|------|---------|
| `cpython_hook` | Full `PythonHook::invoke` path (subprocess + module import) |
| `python_render_only` | Same render work, no sc-cron wrapper |
| `sc_compose_cli_only` | Rust `sc-compose render` only (what the host bridge calls) |
| `monty_host_bridge + sc-compose CLI` | Lab `feed_start` loop in the bench. Production is `execute_feed`, which may call `feed_start` internally. |
| `production_bridge` | Production `monty_host` path through `execute_feed` calling `render_template`. Required for publish preflight `benchmark-gate`. |

The bench prints the numbers for your machine. Typical outcome (release, string
template): **Monty + host CLI ≈ bare CLI** (Monty overhead is small); **Python
hook ≈ 7× slower** and **~2× higher** subprocess RSS than compose-via-CLI paths.
Python remains appropriate when you need an **untrusted/isolated** child process.
The Monty host bridge ships in `execute_feed`, which spawns `sc-compose`.

## API notes (Gemini / pydantic_monty)

- **`sc_compose.render_template(template, context)`** is valid for **string**
  templates (low-level renderer).
- **`sc_compose.compose(ComposeRequest)`** is the structured file/profile API.
- **`pydantic_monty` + `ExternalModule`** is a **different** runtime
  (`monty-pool` workers). sc-cron uses in-process **`monty` 1.4**, not that pool.
