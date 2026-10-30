# External CLI calls from cron hooks

This note defines the Phase B application-extension examples. sc-cron is a
general-purpose scheduler; neither CLI is a required dependency or privileged
class of job. Binding authority is [ADR-RUN-0013](../architecture.md),
[REQ-CRON-0003](requirements.md), and approved ANC-027 through ANC-042.

## Host registration

The embedding application builds a `MontyHostRegistry` and passes it to
`Scheduler::with_monty_hosts`. The default registry is empty. Applications
can register their own `Send + Sync` handler closures without editing sc-cron,
using JSON arguments/results and the existing `CronError` contract.

Phase B provides `ScComposeHostConfig` and `AtmHostConfig` to register
`sc_compose_run(subcommand, args)` and `atm_run(subcommand, args)` through
that same interface. Enabling feature `monty` does not register them.
Executable paths default to `sc-compose` and `atm`; the embedder may select
other non-empty paths. The working directory is embedder configuration,
defaulting to the daemon directory captured when the adapter is built.
Basenames resolve through daemon launch PATH/PATHEXT before environment
clearing. Neither script source nor a schedule row replaces the registered
executable or handler.

Both functions accept one non-empty CLI command/subcommand string and an
optional list of string arguments, with no keyword arguments. The host executes argv directly, never a shell. It
preserves argument order and contents, including flags and paths. There is no
render-only subcommand filter, forced render argv, discarded `--root`, or
mandatory template staging. Any restrictions imposed by a consumer belong to
its own registered handler and are not the library's default CLI contract.

MVP executes application-supplied source without signatures, separate script
approval, or a consumer option enabling approval. Host-function registration
and environment policy are enforced independently. In-process Monty and
custom synchronous Rust handlers retain the same resource/panic/abort risks
as other application code; arbitrary Rust handlers cannot be forcibly
cancelled by the interpreter's execution budget.

## Hook example

Python and Monty receive `sc-cron.hook.v1` with `protocol`, `event`,
`schedule`, `payload`, and `deadline`. Application-specific data is payload.

```python
def cron_stop(hook):
    return {"action": "continue", "reason": "scheduled"}

def cron_tick(hook):
    payload = hook["payload"]
    composed = sc_compose_run(payload["compose_command"], payload["compose_args"])
    if composed["exit_code"] != 0:
        raise ValueError("compose command failed")
    called = atm_run(payload["atm_command"], payload["atm_args"])
    if called["exit_code"] != 0:
        raise ValueError("ATM command failed")
```

The application chooses which registrations this example receives. Unknown
host names fail with `HOST_UNDEFINED`; source cannot register or replace
handlers. Registration rejects invalid identifiers, duplicate names, and
reserved runtime bindings.

## Process result and environment

Completed children return `{stdout, stderr, exit_code}`, including nonzero
exit codes. The script decides whether a nonzero result fails its hook.
Spawn/capture failures fail the hook with the stable codes in
[REQ-CRON-0005](requirements.md#req-cron-0005-one-error-enum).
Row deadlines, process-group termination, and per-stream output caps apply to
both adapters through the same supervisor.

Python, Monty environment reads, and both adapter children receive the same
resolved occurrence snapshot under [ADR-RUN-0016](../architecture.md).
Children never inherit the full daemon environment implicitly.
Configured values echoed into hook failures, stderr, stop reasons, or
scheduled events are redacted before persistence/logging.
Top-level Monty getenv works before hooks run; registered host calls execute
only in cron_stop/cron_tick, not during source initialization.

## Production boundary and benchmark proof

No production release crate links or imports sc-compose or ATM crates.
Boundary lint and production dependency-tree checks enforce that rule;
test/benchmark-only references are permitted. Executable names and subprocess
calls are configuration/integration references, not Rust crate linkage.

`render_template` is not a product requirement. The current Monty benchmark
and generated examples use `sc_compose_run` through the same production
registry/dispatcher exercised by application-defined hosts.
Closed b-10/b-11 records and previously published reports retain their
historical benchmark descriptions; they do not freeze today's product API.

Operator procedure: [benchmark-run](../../.claude/skills/benchmark-run/SKILL.md).
Campaign documentation: [benchmark-monty.md](benchmark-monty.md).
Reports: [benchmark-monty](../../site/reports/benchmark-monty/).

```bash
just --justfile scripts/benchmark/justfile benchmark-monty
just --justfile scripts/benchmark/justfile benchmark-monty-show
just --justfile scripts/benchmark/justfile benchmark-monty-publish
SC_CRON_BENCH=1 cargo test -p sc-cron --features monty --test sc_compose_bench -- --ignored --nocapture
```

The `production_bridge` case drives a real schedule through
`Scheduler::run_due` and `drive_feed`; a lab feed_start microbenchmark is
separate. Test executable fixtures verify non-render compose argv, ATM argv,
nonzero results, missing programs, deadlines, and capture caps without relying
on live external services. Historical PyO3 renderer comparisons are test-only
measurements and do not authorize production tool-crate dependencies.
