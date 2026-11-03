# sc-runtime v0.1.1 — Scheduling, Storage Backends, and Canonical Observability

**Released:** November 2, 2026 · **Tag:** `v0.1.1` · **Previous:** `v0.1.0` (October 25, 2026)

[Release notes](https://github.com/randlee/sc-runtime/releases/tag/v0.1.1) · [Architecture](https://github.com/randlee/sc-runtime/blob/main/docs/architecture.md) · [Generate a project](https://github.com/randlee/sc-runtime/blob/main/docs/generation.md)

---

## Rust Product Developer — a runtime that now schedules and stores

**As a Rust product developer, I want the runtime to cover scheduled work and real database backends, so that a new SC tool does not have to bolt on its own cron loop or storage layer.**

v0.1.1 grows the workspace from four crates to six. The two additions are the ones that were missing for anything with background work:

- **`sc-cron`** — a SQLite-backed cron scheduler for tokio. Schedules live in a `ScheduleStore` as opaque JSON payloads; `Scheduler::start` validates the expression and proves a future occurrence before writing anything. Each schedule gets a fixed jitter, and missed ticks follow a policy (`Coalesce` by default, or `Skip`). Every run-state write is generation-fenced, so a stale run from before a restart cannot overwrite a newer schedule.
- **`sc-runtime-db-sqlite`** — one rusqlite connection on one writer thread. Callers enqueue `execute` and `transaction` work and block until the commit is acknowledged; the reply arrives only after the commit. The crate sets `foreign_keys = ON`, a 5-second busy timeout and WAL mode up front. It is an internal crate (`publish = false`) and is not a separate crates.io dependency.

Behind those, the storage story expanded into the generated template. A shared schema trait pairs the SQLite, Postgres and MySQL engine stores, so the same template can be generated against `sqlite`, `postgres` or `mysql` from the answers file's `db` value, with one database layout emitted per choice. The template store adaptor keeps the generated project on that shared trait instead of hand-written per-backend glue. This is still a 0.x contract and the backend surfaces should be treated as stabilising, not frozen.

Everything from v0.1.0 is still here: the JSON CLI, HTTP daemon, SQLite store and optional MCP surface generated in one command, plus the four original crates. The practical change is that "add a schedule" and "point at Postgres" are now first-class paths rather than projects you write yourself.

```bash
just new <dest> --var-file wizard/fixtures/sqlite-mcp.json --release
```

## Observability Consumer — canonical events and a bounded logging shim

**As a developer instrumenting an SC component, I want the runtime and the scheduler to emit the same canonical events through an injected logger, so that a dashboard or collector sees one vocabulary instead of per-tool dialects.**

v0.1.1 is the first release where observability is a wired contract rather than a type dependency. Three layers landed:

1. **A bounded shim.** The initial-release shim bridges the registry `sc-observability-types 1.5.0` values to a bounded injection contract, so runtime and cron emit through one logger interface without a sixth published crate and without a hard edge to the full `sc-observability`, OTLP or sc-compose stack. The shim is deliberately bounded: it carries the contract and the values, not the whole exporter surface.
2. **Canonical HTTP lifecycle events.** `sc-runtime` emits canonical events for the request lifecycle it owns, so daemon request outcomes arrive in the same shape as the rest of the SC logging stack.
3. **Canonical worker and due-run events.** `sc-cron` emits canonical events for worker state and for each due run, including the outcomes the scheduler already distinguishes — continued, disabled, deleted, timed out.

The generated daemon is composed with the shared logger and a joined cron lifecycle, so a generated project starts on the canonical path rather than wiring logging by hand. One honest caveat, same as v0.1.0: `sc-observability-otel` / OTLP export is **not** in this release. There is no opt-in exporter feature yet, and a default build carries no exporter and no network egress. That remains a 0.2.0 commitment.

For teams on v0.1.0: the four original crates keep their published contracts. The additions are the injection contract and the event vocabulary, so an existing integration keeps working and can adopt canonical events incrementally.

## Template / CI Integrator — lint-clean generation with the cron and store layouts

**As a team lead starting a new SC tool, I want the generated project to include the scheduling, storage and logging wiring already measured against the same gates, so that "new repo" does not mean "new drift".**

The generator now emits a project whose `db` answer selects the database layout, whose daemon is composed with the shared logger, and whose cron lifecycle is joined into the daemon's shutdown path instead of leaking tasks at exit. The host lint profile and the ADR-RUN-0011 boundary grep are part of the tree, so a generated project starts with the boundary rules already applied. The Monty example ships as the production tool-boundary example rather than a stub. The full three-profile sc-lint pass — runtime, cross-platform and boundary — is still the 0.2.0 delta; the generated `sc-lint.toml` continues to cover `fmt` and `clippy` at this tag.

## Plugin Author — additive leaf crates, unchanged existing contracts

**As a plugin author, I want to consume the new scheduling and storage pieces without renegotiating the transport or config contracts I already depend on.**

`sc-config`, `sc-transport` and `sc-command` are unchanged in contract at v0.1.1. The additions are additive leaves: `sc-cron` (with its sole workspace dependency being `sc-observability-types`) and the internal `sc-runtime-db-sqlite` writer. A plugin can adopt the scheduler without pulling in the daemon assembly, and can keep using `sc-transport` or `sc-config` independently as before.

## Not this release

- **OTLP / OpenTelemetry export** — the bounded shim is not an exporter. `sc-observability-otel` behind a cargo feature is a 0.2.0 item; this release deliberately ships no network egress.
- **Fleet operator / SRE deployment surface** — daemon supervision and collector rollout remain sc-observability's and scterm's story. sc-runtime carries no deployment surface.
- **Python bindings** — none. Monty hosts are driven from Rust; there is no Python API to document.
- **Release binaries** — no Homebrew, Scoop or installer path. The five published crates are the distribution channel.

## Install

All five public crates are on crates.io at `0.1.1`:

```toml
sc-config    = "0.1.1"
sc-transport = "0.1.1"
sc-command   = "0.1.1"
sc-runtime   = "0.1.1"
sc-cron      = "0.1.1"
```

`sc-command`'s Axum and rmcp conversions stay behind the `server` feature. `sc-runtime-db-sqlite` is workspace-internal and not published; use `sc-runtime`'s storage path or the generated template's store adaptor instead. The workspace `rust-version` is 1.94.1.

## Upgrading from v0.1.0

Patch a `0.1.0` dependency to `0.1.1`; the original four crates keep their published contracts. If you generated a project at v0.1.0, regenerate or merge the new cron/store/logging wiring to pick up the joined shutdown path and canonical events. Release binaries and installers did not exist at v0.1.0, so there is no binary upgrade path to reconcile.
