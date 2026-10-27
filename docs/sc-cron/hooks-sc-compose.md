# sc-compose from cron hooks

This note is the reference for rendering templates inside a schedule hook.
It matches what the **`sc_compose_hook`** and **`sc_compose_bench`** integration
tests exercise.

## Recommendation

**For compose-heavy hooks (render most ticks), prefer Monty + a host `sc-compose`
bridge** once sc-cron implements it. Benchmarks on this repo show that pattern
matches calling the Rust **`sc-compose` CLI** directly (~3 ms/run, ~6 MiB
compose subprocess peak) and is roughly **7× faster** with **~2× lower RSS**
than a **`HookTarget::Python`** hook that imports PyO3 `sc_compose` (~23 ms/run,
~14 MiB). See
[sc-compose-hook-comparison.report.html](sc-compose-hook-comparison.report.html).

**What ships today:** production Monty feeds use `MontyRepl::call_function` for
`cron_stop` / `cron_tick` only—they do **not** resume a host `render_template`
yet. Until the bridge lands, use **`HookTarget::Python`** (reference fixture
below) or pre-render in the embedding **`Job`**.

**Target bridge (lab in `sc_compose_bench`):** Monty hook code calls a host
ext fn; the host runs **`sc-compose render`** (or an in-process compose
engine)—**no CPython per tick**. Do not import PyO3 `sc_compose` inside the
Monty VM; sc-cron does not use `monty-pool` / `ExternalModule` workers.

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

In **`cron_tick`**, call host-provided **`render_template(template, context)`**
(ext fn). The host implementation should invoke Rust **`sc-compose`**, not
`python3 -c import sc_compose`. Pure in-VM logic (no compose) stays in Monty
without a bridge; state can persist in the per-schedule REPL across ticks.

Until the bridge is productized, pass pre-rendered text in **`payload`** or use
the Python reference hook.

## Benchmarks

Latest numbers (regenerated each bench run):
[sc-compose-hook-comparison.report.html](sc-compose-hook-comparison.report.html).

Optional tests (not run in default CI):

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
| `sc_compose_cli_only` | Rust `sc-compose render` only (what the host bridge should call) |
| `monty_host_bridge + sc-compose CLI` | Monty VM + host resume → CLI (target pattern; not wired in sc-cron yet) |

Read the generated report for your machine. Typical outcome (release, string
template): **Monty + host CLI ≈ bare CLI** (Monty overhead is small); **Python
hook ≈ 7× slower** and **~2× higher** subprocess RSS than compose-via-CLI paths.
Python remains appropriate when you need an **untrusted/isolated** child process
today, or until the Monty host bridge ships.

## API notes (Gemini / pydantic_monty)

- **`sc_compose.render_template(template, context)`** is valid for **string**
  templates (low-level renderer).
- **`sc_compose.compose(ComposeRequest)`** is the structured file/profile API.
- **`pydantic_monty` + `ExternalModule`** is a **different** runtime
  (`monty-pool` workers). sc-cron uses in-process **`monty` 1.4**, not that pool.
