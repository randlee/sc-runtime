---
id: b-17
phase: b
status: planned
closure_type: boundary
target_boundary: sc-cron-hook-environment
track: hook-environment
wave: 9
branch: sprint/b-17-sc-cron-hook-environment
worktree: sprint/b-17-sc-cron-hook-environment
integration_branch: integrate/phase-b-observability
requirements:
  - REQ-CRON-0001
  - REQ-CRON-0003
  - REQ-CRON-0004
  - REQ-CRON-0005
  - REQ-CRON-0019
  - REQ-RUN-0301
  - REQ-RUN-0503
  - NFR-CRON-0001
  - NFR-CRON-0002
  - NFR-RUN-0005
  - NFR-RUN-0008
  - NFR-RUN-0009
adrs:
  - ADR-RUN-0002
  - ADR-RUN-0011
  - ADR-RUN-0013
  - ADR-RUN-0014
  - ADR-RUN-0016
owned_paths:
  - docs/architecture.md
  - docs/sc-cron/requirements.md
  - docs/sc-cron/architecture.md
  - boundaries/sc-cron/sc-cron.toml
  - crates/sc-cron/src/lib.rs
  - crates/sc-cron/src/types.rs
  - crates/sc-cron/src/sqlite.rs
  - crates/sc-cron/src/environment.rs
  - crates/sc-cron/src/hook.rs
  - crates/sc-cron/src/monty_host.rs
  - crates/sc-cron/src/monty_run.rs
  - crates/sc-cron/src/monty_compose.rs
  - crates/sc-cron/src/worker.rs
  - crates/sc-cron/src/worker/worker_runs.rs
  - crates/sc-cron/src/scheduler.rs
  - crates/sc-cron/src/scheduler/run_due.rs
  - crates/sc-cron/tests/hook_environment.rs
  - crates/sc-cron/tests/monty_host_bridge.rs
  - crates/sc-cron/tests/python_hook.rs
  - crates/sc-cron/tests/sqlite_store.rs
  - template/config/default.json
  - template/config/cron.schema.json
  - template/crates/daemon/src/cron.rs
  - template/examples/monty/cron.py
  - docs/sc-cron/monty-generated-example.md
  - tests/unit/generation/test_cron_environment.py
  - .github/workflows/crates.yml
must_follow: [b-15, b-16]
parallel_safe: []
recommended_agent: solar
recommended_model: sol
vertical_rationale: One boundary owns the persisted schedule policy, daemon resolver, Python subprocess environment, Monty OS-call bridge, generated configuration, and checked example so no layer invents a second precedence or redaction rule.
---

# Sprint b-17 — shared Python/Monty hook environment

## Goal and relations

Implement ANC-034 through ANC-042 and REQ-CRON-0019: one explicit environment
contract for Python and Monty, including source initialization and both CLI
adapters. Daemon inheritance is allowed by name; schedules can deliberately
persist set values, clear names, load absolute dotenv files, and resolve opaque
references. Schedule writes never capture ambient process values.

Consume b-15's worker/due-run paths and b-16's registry, adapters, features,
and example; b-18 consumes this policy.
sc-runtime remains a cargo-generate template plus published library crates;
sc-cron remains a general-purpose library for application-defined jobs.
MVP executes application-supplied Monty source without signature verification,
a separate approval check, or a consumer option enabling either.
Update `template/examples/monty/cron.py` to demonstrate this environment policy
through CLI adapters. ANC-032 applies: this example and its tests must work
without `render_template`, an optional helper rather than a required product API.
Rust jobs retain their application interface and process environment; nonempty
schedule environment on a Rust target is InvalidSpec.
ANC-031 applies to this work: all production release crates continue to exclude
sc-compose and ATM crate dependencies and Rust imports. Test and benchmark
references remain permitted; this sprint preserves b-16's boundary enforcement.

## Public model

Keep existing generics. Library entry points return `CronError`; resolver
callback errors map to `ENV_REFERENCE` through their separate protocol.

```rust
pub const MAX_ENV_ENTRIES: usize = 128;
pub const MAX_ENV_DOTENV_SOURCES: usize = 16;
pub const MAX_ENV_NAME_BYTES: usize = 256;
pub const MAX_ENV_VALUE_BYTES: usize = 16 * 1024;
pub const MAX_ENV_REFERENCE_BYTES: usize = 4 * 1024;
pub const MAX_ENV_PATH_BYTES: usize = 4 * 1024;
pub const MAX_ENV_SNAPSHOT_BYTES: usize = 32_000;
pub const MAX_ENV_DOCUMENT_BYTES: usize = 256 * 1024;
pub const MAX_ENV_FILE_BYTES: usize = 1024 * 1024;

pub struct EnvName { /* validated ASCII identifier */ }

impl EnvName {
    pub fn new(name: impl AsRef<str>) -> Result<Self, CronError>;
    pub fn as_str(&self) -> &str;
}

pub struct DotenvSource { /* private validated absolute path + required flag */ }

impl DotenvSource {
    pub fn new(path: impl Into<PathBuf>, required: bool) -> Result<Self, CronError>;
    pub fn path(&self) -> &Path;
    pub fn required(&self) -> bool;
}

pub struct HookEnvironment { /* ordered dotenv sources + name-keyed operations */ }

#[non_exhaustive]
pub enum EnvOperationRef<'a> {
    Set(&'a str),
    Reference(&'a str),
    Clear,
}

impl HookEnvironment {
    pub fn new() -> Self;
    pub fn dotenv(self, path: impl Into<PathBuf>, required: bool) -> Result<Self, CronError>;
    pub fn set(self, name: impl AsRef<str>, value: impl Into<String>) -> Result<Self, CronError>;
    pub fn reference(self, name: impl AsRef<str>, reference: impl Into<String>) -> Result<Self, CronError>;
    pub fn clear(self, name: impl AsRef<str>) -> Result<Self, CronError>;
    pub fn is_empty(&self) -> bool;
    pub fn len(&self) -> usize;
    pub fn dotenv_sources(&self) -> impl ExactSizeIterator<Item = &DotenvSource>;
    pub fn variable_names(&self) -> impl ExactSizeIterator<Item = &EnvName>;
    pub fn operation(&self, name: &EnvName) -> Option<EnvOperationRef<'_>>;
}

impl ScheduleSpec {
    pub fn with_environment(self, environment: HookEnvironment) -> Result<Self, CronError>;
}

pub struct EnvironmentReferenceRequest<'a> { /* redacted Debug */ }

impl EnvironmentReferenceRequest<'_> {
    pub fn schedule(&self) -> &ScheduleName;
    pub fn variable(&self) -> &EnvName;
    pub fn reference(&self) -> &str;
    pub fn deadline(&self) -> Instant;
}

#[non_exhaustive]
pub enum EnvironmentReferenceError { Denied, Unavailable }

#[async_trait::async_trait]
pub trait EnvironmentReferenceResolver: Send + Sync {
    async fn resolve(
        &self,
        request: EnvironmentReferenceRequest<'_>,
    ) -> Result<Option<String>, EnvironmentReferenceError>;
}

pub struct HookEnvironmentHost { /* daemon policy and resolver */ }

impl HookEnvironmentHost {
    pub fn new() -> Self;
    pub fn allow_process(self, names: impl IntoIterator<Item = impl AsRef<str>>)
        -> Result<Self, CronError>;
    pub fn allow_dotenv_file(self, path: impl Into<PathBuf>) -> Result<Self, CronError>;
    pub fn allow_dotenv_root(self, path: impl Into<PathBuf>) -> Result<Self, CronError>;
    pub fn with_reference_resolver(self, resolver: Arc<dyn EnvironmentReferenceResolver>) -> Self;
    pub fn process_names(&self) -> impl ExactSizeIterator<Item = &EnvName>;
    pub fn dotenv_files(&self) -> impl ExactSizeIterator<Item = &Path>;
    pub fn dotenv_roots(&self) -> impl ExactSizeIterator<Item = &Path>;
    pub fn has_reference_resolver(&self) -> bool;
}

impl<S, C, J> Scheduler<S, C, J> {
    pub fn with_hook_environment(self, host: HookEnvironmentHost) -> Self;
}
```

EnvName is Clone/Eq/Ord/Hash with safe Display/Debug. DotenvSource and
HookEnvironment are Clone/Eq with value-free Debug; HookEnvironment defaults
empty. EnvOperationRef is Copy/Clone/Eq with value-free Debug. Resolver errors
are Copy/Clone/Eq and implement Display/Error without provider prose. The
object-safe resolver is Arc-owned; HookEnvironmentHost is Clone/Default with
empty policy and value-free Debug.

All schedule operations use one name-keyed map. A repeated set/reference/clear
for the exact same name replaces its previous operation. Repeated normalized
dotenv paths are invalid,
regardless of required flag. Host allowlists accumulate; exact duplicates are
idempotent. len counts variable operations; is_empty checks operations and
dotenv sources. Accessors borrow values rather than cloning them. Stored
operation fields and DotenvSource fields remain private to enforce validation.

Names match [A-Za-z_][A-Za-z0-9_]*. Names, paths, values, and references are UTF-8
and NUL-free. Empty set values are valid; empty references and relative dotenv
paths are invalid. The constants above limit names, sources, entries, values,
references, paths, canonical document, files, and final snapshot. Check the
entry count and snapshot block after every resolution layer. Encoded block
bytes are the sum of NAME=VALUE plus one terminator per entry and one final
terminator, using the same portable ceiling for Python, Monty, and CLI children.
An invalid/oversize allowlisted process value is ENV_PROCESS_INVALID; an
oversize file value is ENV_FILE_INVALID; an oversize reference result is
ENV_REFERENCE; aggregate overflow is ENV_SNAPSHOT_LIMIT.

Construction, store writes/decodes, and execution all validate hook/environment
pairing, including ScheduleSpec instances modified through existing public
fields. No separate Python/Monty policy types, per-adapter environment builder,
or Rust JobContext is added.

## Persisted and daemon configuration

Canonical schedule JSON has exactly dotenv then variables as top-level keys.
Each source has path then required; source order is preserved. Variable names
sort by UTF-8 bytes. Each operation has exactly one set/reference/clear key;
clear must be true. Reject unknown and duplicate fields. The empty encoding
is exactly {"dotenv":[],"variables":{}}.

Example schedule data:

```json
{
  "dotenv": [{"path": "/var/lib/widget-app/agents/planner-7/.env", "required": true}],
  "variables": {
    "ATM_IDENTITY": {"set": "planner-7"},
    "ATM_TEAM": {"set": "runtime"},
    "OTHER_API_KEY": {"reference": "keychain://agents/planner-7/api-key"},
    "OPENAI_API_KEY": {"clear": true}
  }
}
```

Explicit set values are deliberate plaintext SQLite configuration, including
values the application chooses to treat as secrets. Reference locators are
non-secret plaintext metadata; resolved reference/file/process values are
never copied into the registry. Applications choose references or files when
plaintext set storage is unsuitable.

The generated daemon config has cron.environment.allow_process,
allow_dotenv_files, and allow_dotenv_roots, all empty arrays by default.
For example allow_process may name JEV_API_KEY for all hooks. The library
receives typed HookEnvironmentHost and never reads sc-config or infers a caller
environment. Allow_process stores names, resolves daemon values at execution,
and never captures values during schedule creation or update.

Generated CronConfig::hook_environment_host() builds the configured policy.
Attach the application's resolver before `cron::open`; retain that object.

## Resolution and authorization

One immutable snapshot is resolved before cron.stop and reused for cron.tick:
present allowlisted daemon values, then dotenv files in declaration order,
then schedule operations. Set and resolved Reference replace; Clear removes,
including an absent name. A missing daemon value is absent. Missing references,
required-file absence, denied paths, invalid data, and provider failures fail
before source execution.

EnvironmentReferenceRequest carries schedule, variable, opaque reference, and
row deadline. The application resolver authorizes that request; the default
resolves nothing. Denied, Unavailable, and missing are one fixed value-free
ENV_REFERENCE class. Never format/chain/log provider prose. Bound the resolver
future and complete resolution by the row deadline; expiration is TimedOut.
File I/O runs off the async hot path with no connection or synchronous lock
held across await. After blocking work returns, check the deadline before
executing source.

Dotenv files accept LF/CRLF, optional final newline, blank/comment lines,
optional export plus whitespace, and NAME=VALUE with horizontal whitespace
around =. Unquoted values trim horizontal whitespace; single quotes are literal;
double quotes support only backslash, quote, newline, carriage-return, and tab
escapes. After a closing quote accept whitespace/comment only. A # starts a
comment outside quotes. Empty values are valid. Reject multiline values,
duplicate names, malformed escapes, interpolation ($NAME or ${...}), command
substitution, non-UTF-8 input, and NUL. The parser never evaluates shell or
consults process environment.

Dotenv paths are absolute and normalized without .. components. Canonicalize
before policy comparison so symlinks cannot escape exact-file/root authorization.
Authorize before classifying absence: an unauthorized path is ENV_FILE_DENIED,
an authorized missing required file is ENV_FILE_MISSING, and an authorized
missing optional file contributes nothing. For missing suffixes, canonicalize
the nearest existing ancestor and append the normalized suffix; compare that
expected canonical path against the exact-file/root policy. For existing files,
require a regular canonical target equal to an allowed file or below an allowed
canonical root. Open that canonical target, verify regular-file status before
and after the bounded read, and do not reopen the original lexical path.
Administrator-authorized roots must not be replaced concurrently by an
adversarial principal; agent-level file authorization belongs to the application.

Unix lookup/enumeration preserves configured spelling. Windows snapshot keys
are uppercased for both Python and Monty, and Getenv lookup is case-insensitive;
persisted configuration retains original spelling. Reject ASCII case-fold
collisions on all platforms. A non-Unicode allowlisted daemon value fails
ENV_PROCESS_INVALID rather than silently becoming absent.

## Runtime delivery and redaction

Python uses ChildEnv::Only: clear the child environment and install exactly
the snapshot. PythonHook::new uses this default; direct callers can explicitly
choose ChildEnv::Inherit. Python and both adapter executables resolve before
PATH clearing: basename through daemon launch PATH/PATHEXT, absolute paths
unchanged, component-relative paths against the captured daemon cwd.
Configured CLI child working directory remains b-16's independent setting.

Monty answers only Getenv/GetEnviron from the snapshot with normal absent and
default behavior. Source initialization uses a resumable environment-read
driver: top-level os.getenv works before hooks. Applications can register
additional host functions without editing sc-cron; their invocation is denied
during source initialization and enabled during hooks. Other OS calls stay
HOST_OS-denied in all stages.
A retained REPL initializes once per generation: copied module globals retain
that initialization value; later function-time getenv reads the current
occurrence snapshot. This does not add mutable os.environ, putenv, or unsetenv.

sc_compose_run and atm_run children receive the same snapshot as Python/Monty,
with no implicit full-daemon inheritance. Both adapters forward any command or
subcommand and its arguments unchanged, with no command allowlist. They retain
the common row deadline, process-group kill, and output limits.
sc-cron never mutates the daemon process environment with set_var/remove_var.

Before scheduled persistence/logging, replace each distinct nonempty snapshot
value in hook/provider-controlled detail or stop reason with [REDACTED_ENV],
longest first, then apply detail byte caps and assemble fixed codes/recovery.
Cover Python stdout/stderr, stop reasons/reply errors, Monty compile/runtime
exceptions/return reasons, both CLI streams, and derived outcomes. Do not
substitute stable codes/recovery, metadata, or canonical configuration writes.
Provider/parser failures omit prose, source lines, rejected values, and locators.

Debug for configuration, operations, requests, resolved maps, host policy,
ChildEnv, ChildSpec, PythonHook, and HookOutput exposes only names/kinds/status/
byte counts, never values, locators, stdout, or stderr. Trusted direct callers
may access raw HookOutput fields; scheduled conversion must redact them.
Exact-value replacement mitigates accidental propagation; source can transform
or transmit values, so it is not a hostile-code isolation mechanism.

## First-release schema

No package or binary has been released. Define SCHEMA_VERSION = 1 with one
complete schedule table including canonical hook_environment and its empty
default. Remove unreleased migration ladders and fixtures; no development-cut
compatibility obligation exists. Opening an existing database checks version
and the columns/defaults/constraints/indexes required to use that table,
not exact SQL text or column order. Reject incompatible data with recreate
guidance without overwriting it. The crate does not inspect or migrate
consumer tables or create a second environment/secret table.

## Required work and deliverables

1. REQ-CRON-0001 / REQ-CRON-0003 / REQ-CRON-0004 / REQ-CRON-0005 /
   REQ-CRON-0019 / ADR-RUN-0013 / ADR-RUN-0016 / NFR-CRON-0002:
   implement the model, resolution, runtime, redaction, and schema above; align
   authority docs. Skip corrupt rows through the existing bounded decode path.
2. NFR-CRON-0001 / NFR-RUN-0009: export the types and boundary facade;
   compile-test constructors/accessors and Arc<dyn EnvironmentReferenceResolver>.
3. REQ-CRON-0019 / NFR-RUN-0005: deliver the same snapshot through worker,
   run_due, initialization, and both CLI paths, retaining process supervision
   and sc-cron.hook.v1 input.
4. REQ-RUN-0301 / REQ-CRON-0019 / ADR-RUN-0002 / ADR-RUN-0014: generate the specified empty
   policy defaults, config/schema, prebuilt-policy wiring, and identity/team,
   clear, dotenv, and recording-resolver example. Preserve schedule limits and
   b-16 Cargo features.
5. NFR-RUN-0005 / NFR-RUN-0008 / ADR-RUN-0011: add the parity/canary tests.
   Changed production Rust files stay below 1000 source lines. Production adds
   no sleeps/block_on and holds no std::sync::Mutex guard across await;
   condition/deadline test coordination is permitted.

## Acceptance criteria

1. The hook_environment suite proves exact canonical round-trip/default,
   set/reference/clear replacement, empty values, duplicate/case collisions,
   malformed names/data, and every stored/resolved limit at and over its bound.
   Ordered daemon/file/operation cases prove precedence and one stop/tick map.
2. Daemon identity/team cannot override schedule Set values. Caller-only and
   nonallowlisted daemon canaries are absent from Python, Monty, and both CLI
   children. Deliberate Set persists; process/file/reference results do not.
3. File cases cover optional/required absence, malformed dotenv, symlink escape,
   exact-file/root policy, and authorization before existence classification.
   Resolver cases cover success, missing, Denied, Unavailable, deadline, and
   canary-bearing provider errors without leaking prose or locators.
4. Python preserves hook protocol, process deadlines/kill/caps, explicit Inherit
   opt-in, raw direct HookOutput access, and redacted scheduled results.
   Monty host tests cover top-level/current-occurrence getenv, retained-global
   semantics, GetEnviron parity, absent/default/clear, and non-environment OS/
   initialization-host denial.
5. Both CLI fixtures echo allowed canaries and omit unlisted secrets while
   preserving arbitrary argv and configured working directories. Unix and
   windows-latest cases prove executable lookup before child PATH clearing;
   Windows Path config appears as PATH in both Python and Monty snapshots.
6. Canary cases include ENV, HOST, :, and 1: redaction must not alter fixed
   codes/recovery or metadata. All scheduled diagnostics/outcomes/events/reasons
   and Debug strings omit values; direct raw HookOutput fields are the stated
   exception. Initialization/runtime exceptions and both CLI streams are covered.
7. SQLite cases prove fresh schema creation, canonical round-trip, empty
   defaults, corrupt-row handling, and non-destructive rejection of an
   incompatible table. No historical migration fixture is required.
8. test_cron_environment proves empty administrator defaults, absolute paths,
   separately wired exact-file/root policy, prebuilt resolver preservation,
   and checked-example set/clear/reference/dotenv behavior without a real secret.
   A freshly generated project passes REQ-RUN-0503's setup, lint,
   test, build-cli, and build-daemon sequence with no generated source edits.
   Default feature builds exclude Monty and boundary lint permits only the
   declared types/dependencies.

## Required validation

```sh
cargo test -p sc-cron --test hook_environment
cargo test -p sc-cron
cargo +1.96.1 test -p sc-cron --features monty --test hook_environment
cargo +1.96.1 test -p sc-cron --features monty --test monty_host_bridge
cargo +1.96.1 test -p sc-cron --features monty
cargo +1.96.1 test -p sc-cron --features monty-test-handshake --test monty_hook
cargo clippy -p sc-cron --all-targets --all-features -- -D warnings
cargo fmt --all --check
just test sc-cron-tree-default
just test sc-cron-tree-monty
python3 -m unittest tests.unit.generation.test_cron_environment
python3 -m unittest tests.unit.generation.test_monty_example
just new generated/test-cron-env --var-file wizard/fixtures/sqlite-mcp.json
just --justfile generated/test-cron-env/justfile setup
just --justfile generated/test-cron-env/justfile lint
just --justfile generated/test-cron-env/justfile test
just --justfile generated/test-cron-env/justfile build-cli
just --justfile generated/test-cron-env/justfile build-daemon
! rg -n 'std::env::set_var|std::env::remove_var' crates/sc-cron/src
sc-lint lint sc-boundary --root . --config sc-lint.toml
just lint
```

The windows-latest crates.yml job executes the same named hook_environment and
monty_host_bridge roots; compilation or skipped parity cases do not count as
Windows execution evidence.

## This sprint does not close

Excluded: Rust JobContext/isolation, writable Monty os.environ, caller capture,
built-in secret providers, agent-to-agent file authorization, new CLI/network
adapters, and signing/approval mechanisms.
