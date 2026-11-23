# sc-runtime v0.3.0 — A Layered Configuration Engine, One Toolchain, Current SQLite

**Released:** November 23, 2026 · **Tag:** `v0.3.0` · **Previous:** `v0.2.0` (November 6, 2026)

[Release notes](https://github.com/randlee/sc-runtime/releases/tag/v0.3.0) · [sc-config design](https://github.com/randlee/sc-runtime/tree/main/docs/sc-config) · [Generate a project](https://github.com/randlee/sc-runtime/blob/main/docs/generation.md)

---

## Rust Product Developer — configuration you can extend without forking the loader

**As a Rust product developer, I want a batteries-included runtime for CLI, MCP, SQL/SQLite, daemon and web-server plug-ins, so that I don't re-implement tokio setup, storage, shutdown and retry in every SC tool.**

v0.3.0 is a configuration release. The headline is that `sc-config` grew a
**layered engine that callers extend**, replacing the fixed loader that every
consumer had to work around. `Layers` stacks sources that implement the public,
unsealed `LayerSource` trait; a layer pushed later wins over earlier ones;
`Layers` resolves them into a `Resolved` and then into the caller's own type.
Three built-in sources cover the normal cases: `FileSource` (JSON, required or
optional, with an `accepts` list that rejects out-of-set keys as
`ConfigError::UnknownKey`), `EnvSource` (a literal prefix plus aliases) and
`ValueSource` (values supplied in code). A fourth source is just an impl of the
trait — that is the extensibility point, and it is deliberate that the trait is
not sealed.

Provenance is no longer guesswork. `Resolved::rows()` returns `ResolvedRow`
entries that record, per key, which layer supplied the value, and each `Source`
carries an open `SourceKind`. A `doctor`-style command can therefore print where
a value came from without re-deriving the stack. For the conventional case,
`StandardLayers` / `StandardContext` / `Slot` fix a seven-slot order —
`Defaults`, `System`, `User`, `Project`, `Local`, `Env`, `Flags` — and
`insert(Slot, source)` places a source in a slot. The user folder resolves as
`with_home(dir)`, then the `<FAMILY>_HOME` variable (`SC_AGENT_HOME` for family
`sc-agent`, renameable with `with_home_var`), then `<home>/.<family>/`.

The existing `Loader` is now a preset stack on that engine, not a parallel
pipeline: its observable behaviour is unchanged and its own tests pass unedited,
its dependencies are still only `serde` and `serde_json`, and loading is still
synchronous. There is one loading path, which is the point.

Two breaks come with the engine, both in `ConfigError` (#157). It is now
`#[non_exhaustive]` and gains `UnknownKey { path, key }` (`CFG.UNKNOWN_KEY`) and
`SourceFailed { origin, message }` (`CFG.SOURCE`); add a wildcard arm to every
exhaustive match, or branch on `ConfigError::code()`, which is stable.
`MissingDefault` now means "a required configuration file is absent" for any
required file rather than only `default.json`, and its `Display` and recovery
text changed — match the variant or `code()`, not the message. The code
`CFG.MISSING_DEFAULT` did not change.

`sc-runtime` itself restores one item that 0.2.0 dropped: `testing::instrument_router`
is back behind the `test-util` feature with its former five-argument signature
(#131), applying the current timeout and request-tracking/logging middleware so
downstream tests can choose a short HTTP deadline.

```toml
sc-config = "0.3.0"
sc-runtime = { version = "0.3.0", features = ["test-util"] }   # already-published 0.2.0 removed instrument_router
```

## Template / CI Integrator — a generated project on the toolchain the runtime uses

**As a team lead starting a new SC tool, I want a generated project skeleton that is already lint-clean and just-ready, so that "new repo" does not mean "new drift".**

Generated projects now pin the `sc-*` crates at `0.3.0` and declare
`rust-version = "1.98.1"`, and generated CI uses the same 1.98.1 toolchain. That
is the single-toolchain change: the workspace no longer straddles an MSRV for
most crates and a higher one for Monty, so the template cannot generate a
project that builds locally and fails in CI on a toolchain skew.

One template fix is worth knowing before it bites you: the daemon template's
`main.rs` adds `#![recursion_limit = "256"]`, which rustc 1.98 needs to lay out
the daemon future when the MySQL store is generated. If you have an existing
generated daemon and rustc 1.98 reports a query-depth overflow, add that
attribute to the daemon crate root — it is a one-line change, not a rebuild of
the project.

The generated consumer upgrades to sc-lint 0.8.6 with a synchronized bootstrap,
and the generated SQLite concurrency regression now uses two independent stores
rather than sharing one, so the test measures the contention it claims to.

## Plugin Author — the leaves are still leaves

**As a plugin author, I want stable, separately-published leaf contracts, so that I can depend on the transport or config layer without dragging the whole runtime into my crate.**

Nothing in v0.3.0 merges the leaf crates: `sc-config` still depends on only
`serde` and `serde_json`, keeps its synchronous loading, and stays independent of
the process model, so a component can adopt the new layered engine without
taking `sc-runtime`, Tokio or the plugin surface. `sc-transport` and `sc-command`
keep their contracts; adopting `0.3.0` is a version move, not an API migration.

The one thing a leaf consumer must do is the `ConfigError` migration above.
Because `sc-config`'s error type is `#[non_exhaustive]` you were always meant to
have a catch-all; v0.3.0 makes the compiler enforce it.

## Observability Consumer — unchanged, and the exporter is still absent

**As a developer instrumenting an SC component, I want logging built into the runtime and an opt-in path to OpenTelemetry, so that I get consistent logs by default and OTLP export only when my deployment has a collector.**

No change in this release. `sc-command`, `sc-runtime` and `sc-cron` continue to
emit through the injected logger against `sc-observability-types 1.5.0`, and the
bounded injection shim is untouched. **`sc-observability-otel` is still not
here** — no `otel`/`opentelemetry` dependency and no cargo feature gating one, so
a default build carries no exporter and no network egress. That remains an open
commitment, not a shipped one.

---

## Not this release

- **OTLP / OpenTelemetry export** — still open, still zero egress by default.
- **Fleet operator / SRE deployment surface** — daemon supervision and collector rollout remain sc-observability's and scterm's story.
- **Python bindings** — none. Monty hosts are driven from Rust.
- **Release binaries** — crates.io is the channel; the release page carries checksums and SDK assets only. No Homebrew, Scoop or installer path.

## Install

Six public crates at `0.3.0` on crates.io (the SQLite writer stays internal,
`publish = false`). All published crates share one version — update them
together, because mixing 0.2.x and 0.3.x `sc-*` crates gives you two copies of
the shared types.

```toml
sc-config        = "0.3.0"
sc-transport     = "0.3.0"
sc-command       = "0.3.0"
sc-runtime       = "0.3.0"
sc-cron          = "0.3.0"
sc-cron-commands = "0.3.0"   # defaults: DTOs + CronExecutor only
```

## Upgrade Notes

- **Build with rustc >= 1.98.1.** One toolchain for the whole workspace
  (`rustup toolchain install 1.98.1`); raise `rust-version` in your manifests if
  you declare one.
- **`monty` is 1.6** on the `sc-cron` feature and needs 1.98 — the reason for the
  toolchain bump. If you depend on `monty` directly alongside it, move to 1.6 so
  only one copy resolves.
- **`rusqlite` is 0.43 (`libsqlite3-sys` 0.40).** A consumer that also uses
  `sqlx` with SQLite needs `sqlx >= 0.9.1`, the first release that accepts
  `libsqlite3-sys` 0.40 (`>=0.30.1, <0.41.0`). Any direct `rusqlite` dependency
  must move to 0.43, because the native `links = "sqlite3"` library resolves only
  once. Anchor ANC-006 was amended to v4 with owner approval to state this rule.
- **`rmcp` is 3.9** (`url` 2.5.9) in `sc-command`, `sc-runtime` and the template
  daemon — RUSTSEC-2026-0462 and -0592.
- **`chrono` is still in the graph** (`cron` 0.17, `rmcp` 3.9 via `schemars`,
  `sc-cron` itself). RUSTSEC-2026-0543 is an unmaintained-crate advisory, not a
  vulnerability, and is ignored in `deny.toml`; a chrono-free release is tracked
  in #159 and does not exist yet.
- **`instrument_router` moves behind `test-util`.** If you consumed it at 0.2.0
  it was not there; enable the feature.
