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
  - NFR-CRON-0001
  - NFR-CRON-0002
  - NFR-RUN-0005
  - NFR-RUN-0008
  - NFR-RUN-0009
adrs:
  - ADR-RUN-0011
  - ADR-RUN-0013
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

# Sprint b-17 — shared Python and Monty hook environment

## Goal

Give Python and Monty schedule rows one explicit, durable environment contract
without capturing the caller's ambient process environment or confusing it
with the daemon's environment. The daemon chooses, by name, which of its own
variables are globally available. Each schedule may load authorized absolute
`.env` files and then set, resolve, or clear individual variables. Rust jobs
remain trusted application code and keep their existing `Job` interface.

The rule is: **never capture or persist inherited host values accidentally**.
Explicit schedule `set` values are intentionally persisted. Reference and
`.env` values are resolved at execution and are not copied into SQLite.

## Relations

This sprint follows b-15 because it extends the final worker and `run_due`
paths after their event migration, and follows b-16 because it extends the
checked Monty example and documentation after their feature surface lands.
Both parents already follow b-13. b-14 remains independent. b-18 follows this
sprint and consumes the completed generated `cron` module and environment
configuration in its release proof.

The work is not observability injection and does not reopen b-10's compose
program or render-only subcommand decisions. It amends the host allowlist only
for Monty's existing `Getenv` and `GetEnviron` OS calls. Every other Monty
`OsCall` and every unregistered host function remains denied.

## Public model

`HookEnvironment` belongs to the schedule row and is shared by Python and
Monty targets. Existing constructors produce an empty environment. A non-empty
environment on `HookTarget::Rust` is rejected as `CronError::InvalidSpec` so
configuration is never accepted and silently ignored.

The public surface is equivalent to:

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

`EnvName` implements `Clone`, `Eq`, `Ord`, `Hash`, `Display`, and value-safe
`Debug`. `DotenvSource` and `HookEnvironment` implement `Clone`, `Eq`, and
custom value-free `Debug`; `HookEnvironment` also implements `Default` as the
empty environment. `EnvOperationRef` is `Copy + Clone + Eq` with value-free
`Debug`. `EnvironmentReferenceError` is `Copy + Clone + Eq` and implements
`Display` plus `std::error::Error` without provider text. `HookEnvironmentHost`
is `Clone + Default` (its resolver is an Arc and its default is the empty
policy) and has custom value-free `Debug`. The
resolver trait is object-safe, and a compile test constructs
`Arc<dyn EnvironmentReferenceResolver>`.

`HookEnvironment::len()` is the number of name-keyed variable operations, not
the number of dotenv sources; `is_empty()` is true only when both collections
are empty. `dotenv_sources().len()` exposes the other count without a second
ambiguous aggregate. `operation()` borrows and never clones a value or
reference locator.

The actual encoding is compact UTF-8 with no insignificant whitespace. Top
level keys are exactly `dotenv` then `variables`; each dotenv object is `path`
then `required` and retains declaration order; variable names sort by UTF-8
byte order; each operation object contains exactly one key, `set`, `reference`,
or `clear`. The clear form is exactly `{"clear":true}` and `false` is
rejected. The empty literal is exactly
`{"dotenv":[],"variables":{}}`. Decode rejects unknown or duplicate fields
instead of normalizing them silently.

Exact generic spelling may follow the crate's existing style, but these are
the only concepts. Do not add separate Python and Monty policy types, a second
environment builder on `MontyHostConfig`, or a Rust `JobContext` compatibility
layer. The stored operation enum is private so raw strings cannot bypass the
builders. Read access to deliberate set/reference configuration is explicit
through named accessors. `Debug` for `HookEnvironment`, `DotenvSource`,
`EnvOperationRef`, `EnvironmentReferenceRequest`, resolved maps, host
configuration, `ChildEnv`, `ChildSpec`, `PythonHook`, and `HookOutput` shows
names, source kinds, status, and byte counts only; it never
prints set values, resolved values, reference strings, stdout, or stderr.

This is a deliberate pre-release source break for environment types and
ambient-inheriting `PythonHook::new` behavior. The new validated constructors
and explicit `ChildEnv::Inherit` opt-in are the release contract.
`DotenvSource` fields remain private so callers cannot bypass absolute-path
and size validation. Existing public `ScheduleSpec` fields remain available;
construction, every store write/decode, and every execution revalidate the
hook/environment pairing so a caller cannot mutate a Python/Monty spec into an
invalid Rust spec after `with_environment`.

`EnvName` accepts `[A-Za-z_][A-Za-z0-9_]*`. Limits are
`MAX_ENV_ENTRIES` entries in every intermediate and final environment,
`MAX_ENV_DOTENV_SOURCES` `.env` sources, `MAX_ENV_NAME_BYTES` bytes per name,
16 KiB per stored or resolved value, 4 KiB per reference locator, 4 KiB per
path, 256 KiB for the canonical stored environment document, 1 MiB per `.env`
file, and 32,000 UTF-8 bytes for the encoded final block. The block size is the
sum of `NAME=VALUE` plus one terminator per entry and one final terminator; this
single portable ceiling stays below Windows' 32,767-character process block
and is also used by Monty and compose on every platform. Names, paths, set
values, resolved values, and references use
UTF-8; NUL is invalid everywhere. Empty set values are valid. Empty references
and relative paths are invalid. One name-keyed map makes `set`, `reference`,
and `clear` mutually exclusive. Persisted/config inspection retains spelling,
but ASCII
case-fold collisions such as `Path` and `PATH` are rejected on every platform,
so the same map is representable by a Windows child and by Monty. A
non-Unicode or oversized value found for an allowlisted daemon name fails
closed as `ENV_PROCESS_INVALID`; it is not treated as missing. An oversized
dotenv value is `ENV_FILE_INVALID`, an oversized resolved reference value is
`ENV_REFERENCE`, and an entry-count or encoded-block overflow is
`ENV_SNAPSHOT_LIMIT`; all details are value-free. These limits are checked
after every resolution layer and before Python, Monty, or compose executes.

Repeated builder calls are deterministic. `set`, `reference`, or `clear` for
the same exact name replaces that name's prior operation; a different spelling
with the same ASCII case-fold is rejected. Repeating the same dotenv source is
invalid: identity is the normalized path regardless of the `required` flag,
so one file cannot be listed twice with contradictory flags. Host allowlists
accumulate; an exact duplicate is idempotent and a case-fold name collision is
rejected. Exact file/root duplicates are idempotent.

## Daemon and schedule scopes

`HookEnvironmentHost` is daemon scope. Its default process allowlist, file
allowlist, roots, and reference resolver are empty. `allow_process` stores
names only and resolves them from the daemon process at execution. It never
reads or copies the process environment during schedule creation. The library
has no concept of an invoking agent process and never infers values from one.

The generated daemon reads this application configuration and passes the
typed policy into sc-cron:

```json
{
  "cron": {
    "environment": {
      "allow_process": ["JEV_API_KEY"],
      "allow_dotenv_files": ["/var/lib/widget-app/shared.env"],
      "allow_dotenv_roots": ["/var/lib/widget-app/agents"]
    }
  }
}
```

The generated default is three empty arrays. sc-cron still does not depend on
`sc-config` and never opens the application configuration file itself. An
embedder may build the same policy without using the template.

The persisted per-schedule document is canonical JSON equivalent to:

```json
{
  "dotenv": [
    {
      "path": "/var/lib/widget-app/agents/planner-7/.env",
      "required": true
    }
  ],
  "variables": {
    "ATM_IDENTITY": { "set": "planner-7" },
    "ATM_TEAM": { "set": "runtime" },
    "OTHER_API_KEY": {
      "reference": "keychain://agents/planner-7/api-key"
    },
    "OPENAI_API_KEY": { "clear": true }
  }
}
```

The registry may therefore contain deliberate non-secret or secret `set`
values. Documentation states plainly that they are plaintext SQLite data.
Applications that do not accept that storage use a reference or `.env` file.
Reference strings are also persisted plaintext and must be non-secret locator
metadata, never bearer tokens or embedded credentials. Resolved reference
values and file contents are never copied into an outcome, diagnostic,
canonical event, or `Debug` output.

## Deterministic resolution

The resolver constructs one immutable snapshot for an entire due occurrence,
before `cron.stop`; `cron.tick` receives the same snapshot. Resolution order is:

1. present daemon-process values whose names are in `allow_process`;
2. authorized `.env` files in schedule declaration order;
3. schedule variable operations, where `Set` and resolved `Reference` replace
   an earlier value and `Clear` removes it.

An absent daemon variable is omitted. `Clear` of an absent name succeeds.
Missing required files, invalid files, denied paths, missing references, and
resolver failures fail the occurrence before hook source runs. A missing
optional file contributes no values. Stable recorded failure prefixes are
`ENV_PROCESS_INVALID`, `ENV_FILE_DENIED`, `ENV_FILE_MISSING`,
`ENV_FILE_INVALID`, `ENV_REFERENCE`, and `ENV_SNAPSHOT_LIMIT`; they use the established
`CODE: detail; recovery` form and never contain a value or reference URI.

Reference resolution receives the schedule name, variable name, opaque
reference, and row deadline so the application can authorize schedule-scoped
access. `Denied`, unavailable, and missing results become the same fixed,
value-free `ENV_REFERENCE` failure class. sc-cron never formats, chains, or
logs provider prose. The resolver future is bounded with the row deadline;
expiration cancels the future and records `RunOutcome::TimedOut`. File parsing
and all other resolution share that deadline; after blocking file work returns,
an expired deadline records `TimedOut` before hook source starts.

The `.env` grammar accepts LF or CRLF and an optional final newline. A line is
blank, a comment after optional ASCII horizontal whitespace, or optional
`export` plus at least one space followed by `NAME=VALUE`; horizontal
whitespace is allowed around `=`. An unquoted value trims surrounding
horizontal whitespace. Single quotes are literal. Double quotes support only
`\\`, `\"`, `\n`, `\r`, and `\t`; any other escape is invalid. After a quoted
value only whitespace or a comment is allowed. `#` begins a comment only
outside quotes. `NAME=`, `NAME=''`, and `NAME=""` are valid empty values.
Multiline values, duplicate names, malformed escapes, interpolation syntax
(`${...}` or `$NAME`), command substitution, non-UTF-8 input, and NUL are
rejected. The parser does not consult the daemon environment. This is a
data-file parser, not a shell evaluator.

Every `.env` path is absolute, lexically normalized, and contains no `..`.
Authorization happens before missing-file classification so an unauthorized
path cannot be used as an existence probe. Host policy canonicalizes the
nearest existing ancestor of each configured exact file/root and appends its
normalized missing suffix to form an expected canonical path. At execution:

1. an existing target is canonicalized, must be a regular file, and must equal
   an expected exact-file path or remain below an allowed canonical root;
2. an absent target is authorized only when its expected canonical path equals
   an exact-file policy entry or its nearest existing canonical ancestor is
   below an allowed root;
3. an authorized absent optional source contributes nothing, while an
   authorized absent required source is `ENV_FILE_MISSING`; an unauthorized
   source is always `ENV_FILE_DENIED`.

This ordering follows symlinks before applying the policy, so a lexically
allowed path cannot escape through a link. If a previously absent path becomes
a symlink whose canonical target differs, it is denied. The application remains
responsible for authorizing which agent may select which permitted path.
Allowed files and roots are administrator policy and must not be concurrently
replaced by an adversarial principal during resolution; hostile concurrent
filesystem mutation is outside this library boundary. The implementation
opens the authorized canonical target, verifies it is still a regular file
before and after the bounded read, and never reopens the original lexical path.

Reference strings are opaque to sc-cron. `keychain://`, `vault://`, and other
schemes work only when the embedder's resolver supports and authorizes them for
the supplied schedule and variable. The default resolver resolves nothing and
causes a reference to fail closed.

Environment lookup and enumeration preserve configured spelling on Unix. On
Windows runtime snapshot keys are uppercased for both the Python child and
Monty `GetEnviron`, matching standard CPython behavior; Monty `Getenv`
uppercases its query for case-insensitive lookup. Persisted/config inspection
still shows the caller's original spelling. Case-fold collisions are rejected
on all platforms so normalization never has two answers.

## Failure redaction boundary

Hooks can print or raise with an environment value, so redaction is applied to
behavior, not merely to configuration formatting. For every hook/provider-
controlled detail or stop reason, sc-cron replaces each distinct non-empty
value in the final resolved snapshot with `[REDACTED_ENV]`, longest values
first, and then applies existing detail byte caps. Only after that does it
assemble fixed `CODE: detail; recovery`, diagnostic, trace, event, outcome, or
persisted-reason fields. Substitution never runs over stable codes, recovery
text, schedule metadata, or the canonical `HookEnvironment` configuration
write; deliberate `Set` values remain plaintext schedule data. This covers
Python stdout and stderr, stop-reply errors/reasons, Monty compile/runtime
exceptions and return reasons, compose stderr, and every derived `RunOutcome`.
Resolver and parser failures never include provider prose or source lines in
the first place.

This exact-value replacement reduces accidental disclosure through ordinary
error propagation. It is not a sandbox or an adversarial exfiltration control:
trusted hook source can split, transform, encode, or transmit a value. Builder,
decode, and execution validation errors omit rejected values and reference
locators entirely.

Raw `HookOutput.stdout` and `.stderr` remain deliberately available to a
direct `PythonHook` caller, which is already inside the trusted embedding API;
their custom `Debug` omits both strings. The scheduler path never persists or
emits raw `HookOutput` and must use the snapshot-aware redacted conversion.

## Runtime delivery

Python and Monty receive the same resolved key/value map:

- The scheduled Python subprocess uses `ChildEnv::Only` semantics: clear the
  child environment and install exactly the resolved snapshot. Interpreter
  lookup is host launch policy: before clearing the child environment, sc-cron
  resolves a configured basename against the daemon's launch `PATH` (including
  Windows `PATHEXT`), retains an absolute program, or makes a relative path
  with components absolute against the daemon current directory captured when
  the hook/host configuration is built. This preserves prior executable
  selection without relying on the cleared child's `PATH`.
  The child receives that absolute executable and does not receive the daemon's
  `PATH`. `PythonHook::new` no longer inherits the daemon environment by
  default; direct callers must explicitly select `ChildEnv::Inherit` if they
  accept that legacy behavior.
- Monty resumes `OsFunctionCall::Getenv` and `GetEnviron` in Rust.
  Allowed/present names return values; missing, cleared, or unavailable names
  behave as absent, so `os.getenv(name, default)` returns its default and
  `os.environ` contains only the snapshot. Every other `OsCall` retains
  `HOST_OS` denial.
- Monty source initialization moves from the non-resumable `feed_run` path to
  a stage-specific resume driver that answers only `Getenv` and `GetEnviron`.
  Every function call, including `render_template` and `sc_compose_run`, is
  denied during module initialization, so b-10's capability is not expanded.
  Top-level `os.getenv` works before `cron_stop` and `cron_tick` are invoked.
  A retained REPL follows ordinary module semantics: a value copied into a
  module global at initialization remains that generation's value, while a
  later function-time `os.getenv` uses the current occurrence snapshot.
- A compose child launched by `render_template` or render-only
  `sc_compose_run` receives the same resolved snapshot and does not inherit
  the daemon's complete environment. An embedder-selected basename is likewise
  resolved to an absolute program through daemon launch policy before the
  child's environment is cleared; absolute and relative-with-components paths
  follow the same rule above. `MontyHostConfig` remains the sole program
  selector.

Rust `Job` remains trusted in-process application code. This sprint does not
change the `Job` trait, pretend to sandbox `std::env`, or persist an
environment for a Rust row. Rust code may use whatever daemon environment the
application intentionally provides.

The schedule-level operations are the supported way to set or clear values
before Python or Monty source executes. Python-compatible write-through
mutation of `os.environ` inside a running Monty program is a non-goal until
Monty exposes set/unset OS calls; sc-cron never calls `std::env::set_var`.

## Pre-release schema baseline

No sc-runtime binary or `sc-cron` package has been released. This sprint does
not add a ceremonial schema version 5. It replaces the development migration
ladder with one release baseline:

- fresh databases create the complete schedule table at `SCHEMA_VERSION = 1`,
  including canonical `hook_environment` text defaulting to the exact empty
  literal above;
- v1-v4 development migrations and their fixtures are removed from the public
  contract;
- opening an earlier development database whose columns or constraints do not
  match the release-v1 fingerprint fails as `SchemaMismatch` with recovery to
  recreate the unreleased development database;
- the table remains the only sc-cron table and no consumer table is inspected
  or migrated.

The schema fingerprint is `user_version = 1`; exact declared column order,
name, type text, nullability, default, hidden flag, and primary-key flag from
`PRAGMA table_xinfo`; byte-for-byte equality with the one canonical
`sqlite_schema.sql` table string produced by the release-v1 `CREATE TABLE`
constant (no SQL normalization); and exact unique/index name, column order,
and partial flag from `index_list` and `index_xinfo`. An existing non-empty
file is first opened read-only and checked
before any busy-timeout or WAL pragma. On mismatch sc-cron closes it without
changing the main file or creating/modifying `-journal`, `-wal`, or `-shm`
sidecars. Only a valid file is reopened read-write; a missing/empty file is
created as release v1. This is required because an earlier development
database may also carry `user_version = 1`; accepting the number without the
shape would misidentify it as the release schema. Backward compatibility
begins with the first released schema, not with discarded development cuts.

## Required work and deliverables

1. REQ-CRON-0001 / REQ-CRON-0003 / REQ-CRON-0004 / REQ-CRON-0005:
   add ADR-RUN-0016, REQ-CRON-0019, and NFR-CRON-0002 with the public model,
   resolution order, redaction, file authorization, runtime delivery, Rust
   non-goal, and pre-release schema ruling above. Amend ADR-RUN-0013 point 3
   for the release-v1 row shape and point 4 so Monty allows only `Getenv` and
   `GetEnviron` in addition to its two existing host functions. Do not shorten
   the landed compose program, render-only, timeout, or recovery text.
2. NFR-CRON-0001 / NFR-RUN-0009: implement the validated environment types,
   private resolved snapshot, host
   policy, reference-resolver seam, and boundary exports. Existing schedule
   constructors default to empty. Reject non-empty Rust environments and
   custom `Debug` output that reveals values, including through `ChildEnv`,
   `ChildSpec`, and `PythonHook`. Compile-test the documented
   constructors, accessors, resolver request, and redacted formatting surface.
   Tests fix replacement/idempotence/collision behavior for every builder.
3. Store canonical environment JSON on Python and Monty rows and consolidate
   SQLite into the release-v1 schema. Validate at construction, store write,
   and row decode. A corrupt row is skipped through the existing store path;
   no unbounded allocation occurs before size checks.
4. REQ-CRON-0005 / NFR-RUN-0005: resolve one snapshot per occurrence and carry it through worker and
   `run_due` stop/tick paths. Implement exact daemon, `.env`, reference,
   set/clear precedence and stable failures. Bound reference futures and the
   complete resolution phase by the row deadline. Resolution performs file
   I/O on a blocking path and never holds a store connection or lock across
   await. Enforce per-value, entry-count, and encoded-snapshot limits after
   every layer with source-specific value-free failures.
5. Deliver the snapshot to Python with an empty child base; change ambient
   inheritance to explicit opt-in for direct `PythonHook` callers. Preserve
   process-group kill, deadline, output cap, and stdin hook protocol. Resolve
   Python and compose programs to absolute paths through daemon launch policy
   before clearing each child environment.
6. Resume Monty `Getenv`/`GetEnviron` during source initialization and hook
   feeds, preserve absence semantics, deny every function call during source
   initialization and every other OS call in all stages, and pass the same
   snapshot to compose children without changing allowed subcommands.
7. NFR-RUN-0005: add generated daemon configuration with empty `allow_process`,
   `allow_dotenv_files`, and `allow_dotenv_roots` defaults, wire the typed
   host policy in `cron.rs`, and expose a generated
   `CronConfig::hook_environment_host()` builder so application composition can
   attach an `EnvironmentReferenceResolver` before scheduler open. Extend the
   checked example with
   `ATM_IDENTITY`, `ATM_TEAM`, a cleared global variable, one reference, and
   one absolute `.env` source. The test uses temporary files and a recording
   resolver; it never depends on the developer's real environment or keychain.
8. Add one snapshot-aware redaction boundary for Python, Monty, compose, stop
   reasons, outcomes, logging, and persistence. Preserve raw public
   `HookOutput` access for trusted direct callers while making its `Debug`
   output content-free.
9. NFR-RUN-0008 / ADR-RUN-0011: keep changed production Rust sources below
   the line-count gate, add no production sleep or `block_on`, call no
   `std::env::set_var`, and hold no synchronous lock across await.
10. Extend crate CI with a `windows-latest` acceptance job for the
    platform-gated environment tests. It installs Rust 1.96.1 and Python and
    runs the named hook-environment and Monty bridge suites; Linux-only green
    results are not evidence for Windows case-folding or `PATHEXT` behavior.

## Acceptance criteria

1. `hook_environment` passes a table covering present/missing daemon names,
   multiple `.env` precedence, set/reference replacement, clear, empty values,
   separately authorized exact files and roots, required/optional files,
   invalid grammar, symlink escape, resolver denial/
   failure/timeout, non-UTF-8 daemon input, exact-Unix and uppercased/
   case-insensitive-Windows lookup plus enumeration parity, builder
   replacement/duplicate behavior, value-free environment/supervisor debug formatting,
   and every size/name limit. Process, dotenv, and resolver oversize values plus
   entry-count/encoded-total overflow each fail value-free before execution.
   A Windows-gated case stores `Path` in config but observes only `PATH` in
   both Python `os.environ` and Monty `GetEnviron`.
   That case must pass on `windows-latest`; compiling or skipping it on Linux
   is not acceptance evidence.
2. `hook_environment` passes only when caller-only variables and non-allowlisted
   daemon variables are absent from both Python and Monty. A daemon
   `ATM_IDENTITY`/`ATM_TEAM` cannot override schedule `Set` values.
3. The Python cases pass only when the child starts from the resolved snapshot,
   receives the unchanged `sc-cron.hook.v1` object, and retains kill/deadline/
   output behavior. The test proves full ambient inheritance occurs only after
   an explicit direct-call `ChildEnv::Inherit` selection. A hook deliberately
   prints each resolved canary; raw `HookOutput` remains accessible to the
   direct caller, while the scheduler's outcome, events, and store contain no
   canary. Short canaries such as `ENV`, `HOST`, `:`, and `1` prove redaction
   cannot alter stable failure codes or recovery text. Platform-gated tests
   prove basename/PATHEXT resolution occurs before `PATH` is cleared.
4. `monty_host_bridge` passes top-level and function-time `os.getenv`, filtered
   `os.environ`, absent/default behavior, stop/tick snapshot identity, and
   retained denial of a non-environment `OsCall` and unknown host function.
   Top-level `os.getenv` succeeds, while top-level `render_template` and
   `sc_compose_run` are denied. Runtime exceptions, returned stop reasons, and
   compose stderr deliberately echo canaries and prove redaction before store
   or event emission.
5. The compose-child case passes only when an allowed canary is present and a
   non-allowlisted daemon secret is absent; render-only policy and configured
   program selection remain unchanged. Basename resolution works with no
   `PATH` in the child on Unix and Windows.
6. SQLite tests pass for a fresh release-v1 database, canonical round-trip,
   empty defaults, maximum-size rejection, malformed/corrupt row handling,
   and refusal of each old development fixture shape without changing its
   bytes or creating/modifying SQLite sidecars.
   No test or active document calls the release schema version 5.
7. `test_cron_environment` passes only when all three generated administrator
   arrays are empty by default, absolute paths are required, exact-file and
   root authorization are wired separately, the daemon builds the exact typed host
   policy, and its temporary example passes a recording resolver through that
   public builder and proves set/clear/reference/`.env` behavior without
   reading a real secret.
8. `cargo test -p sc-cron` and the Rust 1.96.1 Monty suite pass; feature-off
   builds contain no Monty dependency. Boundary lint accepts only the new
   sc-cron public types and no new workspace dependency.

## Required validation

```sh
cargo test -p sc-cron --test hook_environment
cargo test -p sc-cron
cargo +1.96.1 test -p sc-cron --features monty --test monty_host_bridge
cargo +1.96.1 test -p sc-cron --features monty
cargo +1.96.1 test -p sc-cron --features monty-test-handshake --test monty_hook
# The crates.yml windows-latest job runs these same named parity roots:
cargo +1.96.1 test -p sc-cron --features monty --test hook_environment
cargo +1.96.1 test -p sc-cron --features monty --test monty_host_bridge
cargo clippy -p sc-cron --all-targets --all-features -- -D warnings
cargo fmt --all --check
just test sc-cron-tree-default
just test sc-cron-tree-monty
python3 -m unittest tests.unit.generation.test_cron_environment
python3 -m unittest tests.unit.generation.test_monty_example
! rg -n 'std::env::set_var|std::env::remove_var' crates/sc-cron/src
sc-lint lint sc-boundary --root . --config sc-lint.toml
test -f docs/plans/phase-b/sprint-b-18-template-observability-integration.md
test ! -e docs/plans/phase-b/sprint-b-17-template-observability-integration.md
! rg -n 'sprint-b-17-template-observability-integration' docs .sprints --glob '!docs/plans/phase-b/sprint-b-17-sc-cron-hook-environment.md'
just lint
```

## This sprint does not close

- Rust hook isolation or a new Rust `Job` context.
- Monty `os.environ` write-through mutation, `putenv`, or `unsetenv`.
- Automatic caller-process capture, secret-manager implementation, or an
  application API for authorizing one agent's access to another agent's file.
- A new compose host function, arbitrary compose subcommands, networking, or
  an untrusted-code sandbox.
