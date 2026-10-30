# Generated Monty example

`examples/monty/cron.py` is copied into a generated project. Enable the generated
daemon's `monty` feature to enable `cron` and `sc-cron/monty`. Both features are
off by default. Only the daemon has the optional `sc-cron` dependency; package
names retain `<project>-<role>`, including `<project>-daemon`.

The application supplies an immutable registry; enabling Monty alone registers
nothing and requires neither CLI. For the checked CLI example:

```rust,ignore
use sc_cron::{AtmHostConfig, MontyHostRegistry, ScComposeHostConfig};
let registry = MontyHostRegistry::new()
    .with_sc_compose(ScComposeHostConfig::default())?
    .with_atm(AtmHostConfig::default())?;
let scheduler = scheduler.with_monty_hosts(registry);
```

Each configuration accepts a non-empty executable path with `new(program)` and
an optional `.with_working_directory(path)?`. The default cwd is captured at
configuration. The executable is resolved before clearing the child environment.
The script receives the regular `sc-cron.hook.v1` object: protocol, event,
schedule, payload, and deadline. Its payload supplies `compose_args`, such as
`["--root", "/app/templates", "--mode", "file", "--file", "note.j2"]`.
`sc_compose_run("render", args)` and `atm_run("agent", ["list"])` return
`{stdout, stderr, exit_code}`. Any nonempty command/subcommand and all ordered
string arguments are forwarded unchanged as argv. The driver never invokes a
shell, rewrites flags, or forces template staging. Completed nonzero exits are
ordinary results; this example chooses to fail its hook when either exit is
nonzero. `cron_stop` returns the current continue decision.

An application can also register a host with neither CLI installed:

```rust,ignore
use sc_cron::{MontyHostCall, MontyHostRegistry};
let registry = MontyHostRegistry::new().register("jev_call", |call: &MontyHostCall| {
    Ok(serde_json::json!({"schedule": call.schedule.as_str(), "args": call.args}))
})?;
let scheduler = scheduler.with_monty_hosts(registry);
```

The open object-safe `MontyHostHandler: Send + Sync` supports matching closures.
`MontyHostCall` owns the registered name, positional/keyword JSON arguments,
schedule name, hook phase, and row deadline. Invalid identifiers, duplicate
names, and reserved `__hook`, `cron_stop`, and `cron_tick` names return
`CronError::InvalidSpec`. Source can invoke a registered function but cannot
obtain or replace its handler object. Unknown names fail with `HOST_UNDEFINED`;
conversion and CLI shape failures use `HOST_DENIED`; custom handler errors,
including returned `InvalidSpec`, use `HOST_CALL_FAILED`. CLI spawn and capture
failures use `HOST_SPAWN` and `HOST_OUTPUT`. Deadline expiration is `TimedOut`.
`render_template` is not a product API or default registration; private or lab
helpers may retain that name.

Hooks and handlers are application-supplied in-process code. MVP has no signing
or separate script approval gate and no consumer option enabling either.
Registration and environment policy are independent. Synchronous application
handlers run on the blocking feed thread; deadlines are checked before and
after callbacks but cannot preempt arbitrary Rust. Stack overflow, allocator
abort, panic, and non-cooperative deadline risks remain those of a Rust Job.
CLI children have row deadlines, process-group/job termination, and
`MAX_HOOK_OUTPUT_BYTES` per stream. A descendant that leaves its process group
can escape group cleanup. Both adapters start with an explicitly empty child
environment here; b-17 supplies its resolved environment snapshot and redaction.

For an explicitly requested development checkout before upstream 1.5.1 is
available, generate with:

```sh
python3 scripts/new_project.py generated/monty-dev --var-file wizard/fixtures/sqlite-mcp.json --development-observability
```

This generation-time mode selects the real registry logger `=1.5.0`, local
`1.5.1` types facade, and real logger adapter `0.0.0`, alongside the five local
internal crate patches. The generated source stays unedited through setup,
lint, test, build-cli, and build-daemon. The flag conflicts with `--release`
and release-mode environment selection before output is created. Canonical
template manifests retain the matching `1.5.1` requirements. Development builds
and benchmarks are permitted; the publication guard remains active. b-18 owns
removal of development paths/adapter and the real registry-only release proof.
