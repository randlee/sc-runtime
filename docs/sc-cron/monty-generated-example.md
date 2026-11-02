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
`MontyHostCall` owns the validated `MontyHostName` (borrowed with `as_str()`), positional/keyword JSON arguments,
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
can escape group cleanup. Both adapters receive the same resolved immutable environment snapshot as Python and Monty, with no implicit full-daemon inheritance.

The normal generation path uses the initial release shim directly:

```sh
python3 scripts/new_project.py generated/monty-dev --var-file wizard/fixtures/sqlite-mcp.json
```

Generated application source contains its private adapter over the real
registry logger/types `=1.5.0`; the five SDK checkout patches are used only for
prepublication development. `--release` omits those patches and is proven after
all five publications. No separate observability mode or local facade/adapter
package is required. Setup and later recipes never edit Rust. The command-owned
neutral contract and temporary cron-command edge are removed by capability-proven
upstream cleanup in [issue 97](https://github.com/randlee/sc-runtime/issues/97).

The generated `main` opens `cron.database_path` (default `cron.db`) relative
to the instance root, starts the worker, then joins it after runtime drain
before dropping the scheduler and consuming the logger owner. Absolute and parent-traversal database paths are rejected before any database
is opened; configured paths remain below the instance root. `ApplicationJob` refuses Rust jobs
until the application provides its own implementation; stored Python/Monty
hooks need no Rust workload implementation.

The shared environment is built independently of the registry. Empty administrator
allowlists are generated under `cron.environment.allow_process`,
`allow_dotenv_files`, and `allow_dotenv_roots`. The following recording example
uses an invented value, with no real credential or implicit daemon capture:

```rust,ignore
use std::sync::{Arc, Mutex};
use sc_cron::{EnvironmentReferenceError, EnvironmentReferenceRequest,
    EnvironmentReferenceResolver, HookEnvironment, HookTarget, ScheduleSpec};

struct RecordingResolver(Mutex<Vec<String>>);
#[async_trait::async_trait]
impl EnvironmentReferenceResolver for RecordingResolver {
    async fn resolve(&self, request: EnvironmentReferenceRequest<'_>)
        -> Result<Option<String>, EnvironmentReferenceError>
    {
        // Application authorization checks schedule + variable + locator here.
        if request.schedule().as_str() != "planner" ||
            request.variable().as_str() != "OTHER_API_KEY" ||
            request.reference() != "recording://planner/key" {
            return Err(EnvironmentReferenceError::Denied);
        }
        self.0.lock().expect("recording lock").push(request.variable().to_string());
        Ok(Some("recording-value".to_owned()))
    }
}
let resolver = Arc::new(RecordingResolver(Mutex::new(Vec::new())));
// Use the same application-owned production composition path as main.
let services = daemon::composition::CronServices::default()
    .with_reference_resolver(resolver.clone())
    .with_monty_hosts(registry);
let scheduler = services.open_scheduler(
    &config.cron, std::path::Path::new("/var/lib/widget-app"), logger,
)?;
let environment = HookEnvironment::new()
    .dotenv("/var/lib/widget-app/agents/planner-7/.env", true)?
    .set("ATM_IDENTITY", "planner-7")?
    .set("ATM_TEAM", "runtime")?
    .clear("OPENAI_API_KEY")?
    .reference("OTHER_API_KEY", "recording://planner/key")?;
let spec = ScheduleSpec::with_hook("planner", "* * * * *",
    std::time::Duration::from_secs(20),
    serde_json::json!({"identity": "planner-7", "team": "runtime",
        "compose_args": ["--root", "/app/templates", "--file", "note.j2"]}),
    HookTarget::monty(include_str!("examples/monty/cron.py"))?)?
    .with_environment(environment)?;
scheduler.start(spec)?;
```

The administrator separately authorizes the exact dotenv file or its parent
root. The example file contains `AGENT_MODE=recording`. Resolution happens once
per occurrence: allowlisted daemon values, then files in declaration order,
then schedule Set/Reference/Clear operations. Both hooks and both CLI children
share that immutable map. Schedule identity/team Set values replace daemon
values, and Clear removes names. The checked example needs no `render_template`.

Python defaults to an empty `ChildEnv::Only`; trusted direct callers may choose
`ChildEnv::Inherit` explicitly. Executables resolve against daemon launch
PATH/PATHEXT and captured cwd before clearing the child environment. Monty source
initialization may read `os.getenv`/`os.environ`, but application host invocations
are denied during initialization. A retained generation initializes once;
module globals retain that value while function-time reads use each occurrence.
All other suspended OS operations remain `HOST_OS` denied. Windows snapshots
uppercase names and perform case-insensitive getenv lookup; stored spelling is
preserved and ASCII case collisions are rejected everywhere.

Set values are deliberate plaintext SQLite configuration. Process/file/resolved
reference values are never copied into registry data. Runtime details and stop
reasons replace distinct nonempty snapshot values with `[REDACTED_ENV]`, longest
first, before detail caps and fixed diagnostic codes/recovery are assembled.
Debug output exposes names/kinds/counts only. Trusted direct `HookOutput` fields
remain raw. Exact replacement reduces accidental propagation; application source
can transform or transmit values, so this contract is not hostile-code isolation.
