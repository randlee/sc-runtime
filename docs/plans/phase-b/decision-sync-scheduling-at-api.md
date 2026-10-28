# Decision: sync scheduling stays at the bd-cache API; schedule rows are capped by the embedder

**Status:** Settled. Amended 2026-10-27 (owner/lead). The 2026-10-26 ruling "no schedule slot restriction" is **superseded** by [Owner ruling: embedder-supplied row limit](#owner-ruling-embedder-supplied-row-limit-2026-10-27). The no-two-row-mutex and no-global-lock parts of the 2026-10-26 ruling stand.  
**Date:** 2026-10-26 (amended 2026-10-27)  
**Bead:** `run-zif.2` (parent `run-zif`)  
**Owner intent:** 2026-10-26, amended 2026-10-27

This record is the decision for `run-zif.2`. It names no change to PR #68. Row-limit **implementation** is [sprint b-12](sprint-b-12-sc-cron-schedule-row-limit.md) on `develop` ([ADR-RUN-0014](../../architecture.md)).

## Settled

1. **REQ-BDC-0720's two-row ceiling is temporary beads-cache behavior, not lasting scheduling policy.** Today the bd-cache CLI still enforces at most one built-in sync row and one Python sync row, at most one enabled (`What the API enforces today`, below). That shape is **not** the permanent limit on the schedule table; bead **`run-zif.2`** amends beads-cache requirements and CLI text toward the [row-limit ruling](#owner-ruling-embedder-supplied-row-limit-2026-10-27). **`REQ-BDC-0722`** (`tick_lease`) stays: one sync run at a time across the cache file is cross-process concurrency control, not a row cap.
2. **BDC sync and the lease are beads-cache only.** `REQ-BDC-0720`, `REQ-BDC-0722`, `REQ-BDC-0724`, the `tick_lease` row, `sync use-python`, and `sync use-builtin` are bd-cache product behavior over the bd-cache file. sc-cron has no sync concept, no lease, and no beads type. sc-cron provides **generic primitives only** (see [What sc-cron provides](#what-sc-cron-provides)); bd-cache composes them.
3. **sc-cron stays a general scheduler.** Many schedule rows up to the embedder's limit. Rows are added and replaced at runtime. A table may mix hook targets. Two different schedule names may run at the same time. The only skip inside the scheduler is a tick that comes due while that same schedule name is still in flight (beads-cache `REQ-CRON-0006`). sc-cron does not grow a two-row mutex, a global in-flight lock, or a beads-specific schedule limit. Its only row ceiling is the **caller-supplied** limit below; sc-cron ships no default of its own.
4. **Monty is out of scope for beads-cache.** beads-cache sync stays a Rust built-in job or a consumer CPython script. beads-cache `REQ-CRON-0016` keeps that crate's target set at Rust or Python. An in-process Monty hook is an sc-runtime follow-on ([sprint-b-9-sc-cron-monty-hooks.md](sprint-b-9-sc-cron-monty-hooks.md)), after the b-3 port merges.

The reading that "replace Rust with Python" limits sc-cron itself to those two sync rows is rejected. That sentence is the bd-cache switch between its two designated rows.

## What the API enforces today

**Current beads-cache behavior**, subordinate to the [row-limit ruling](#owner-ruling-embedder-supplied-row-limit-2026-10-27). Quoted scope from beads-cache `docs/requirements.md` (approved 2026-10-26). This planning sprint does not edit that text.

`REQ-BDC-0720` — at most one built-in sync row and one Python sync row; at most one enabled. `schedule enable` while the Python row is enabled is `SCHEDULE_INVALID` (`details.reason: "python_sync_enabled"`). `schedule set` writes through `replace_and_set_enabled` so a row written enabled disables the Python row in the same transaction. While the Python row is enabled, `bdc cache tick` and `Cache.tick()` are refused with `PRECONDITION` (`details.reason: "python_sync_enabled"`).

`REQ-BDC-0722` — one `tick_lease` row in the cache file, taken under `BEGIN IMMEDIATE`, expiring at the run's deadline plus 10 seconds, released on every exit path. A scheduled or on-demand run that finds a live lease replies `skipped: "in_flight"`. `sync use-python` fails with `TICK_IN_FLIGHT`. `sync use-builtin` takes no lease.

`REQ-BDC-0724` — `sync use-python` takes the lease, then in one transaction writes the one Python row enabled and disables the built-in row. There is never a second Python sync row. `sync use-builtin` makes the built-in row the enabled one in one transaction.

All three are beads-cache requirements over the bd-cache file. None is an sc-cron rule.

## What sc-cron provides

The scheduler primitives those commands already use (generic; no sync, lease, or beads semantics):

- `ScheduleStore::set_enabled_many` and `Scheduler::set_enabled_many` (beads-cache `REQ-CRON-0017`)
- `ScheduleStore::replace_and_set_enabled` and `Scheduler::replace_and_set_enabled` (`REQ-CRON-0017`)
- one run in flight per schedule name (`REQ-CRON-0006`)
- generation fencing so a replaced row's old run cannot write (`REQ-CRON-0007`)
- optional row limit the embedder passes in ([sprint b-12](sprint-b-12-sc-cron-schedule-row-limit.md))

`REQ-CRON-0006` is per schedule name, inside one scheduler: if a tick is due while that name's previous run is still in flight, sc-cron skips the new tick. `REQ-BDC-0722` is one lease for every sync run in the cache file, across processes, whichever row is enabled. They are different mechanisms (in-process per-name skip vs cross-process sync lease); do not merge them into one requirement. The REQ-CRON-0006 vs REQ-BDC-0722 distinction is also tabulated in [sc-cron-req-crosswalk.md](sc-cron-req-crosswalk.md#per-schedule-in-flight-skip-and-the-tick-lease).

## Owner ruling: embedder-supplied row limit (2026-10-27)

Replaces the 2026-10-26 text "the scheduler must not cap how many schedule rows a consumer may hold or enable".

1. **Default limit: about 10 schedule rows**, counting **enabled and disabled** rows together (every row in the app's `sc_cron_schedule` table). "About 10" is the per-app default the consuming application ships; it is not a constant inside sc-cron.
2. **Where the limit is configured.** The limit lives in the **consuming application's sc-config JSON** (the application owns the key name and its own default; per-app default is configurable). **sc-cron never reads sc-config** and takes no sc-config dependency (`REQ-CRON-0001`: no other workspace crate). The embedder reads its own config and **passes the limit into the sc-cron API** when it builds or opens the scheduler/store.
3. **Library default is unlimited.** If the embedder passes no limit (`None`), sc-cron enforces none. A bd-cache, ATM, or other consumer that wants about 10 sets it in its own config and passes it.
4. **Enforcement point ([sprint b-12](sprint-b-12-sc-cron-schedule-row-limit.md)).** The check runs inside the same store transaction as the insert, so two processes sharing one file cannot both pass it. Adding a row whose name is new and would exceed the limit is rejected with the existing `CronError::InvalidSpec` (detail names the limit and current count); `replace` of an existing name does not change the count. No new `CronError` variant (`REQ-CRON-0005` stays seven variants). Lowering the limit below the current row count never deletes rows: existing rows keep running; new names are rejected until the count is under the limit. Deleting a row frees a slot immediately.
5. **Relationship to blocking-pool stalls.** The b-9 text that many names may each hold a blocking-pool thread is bounded for any consumer that sets the limit. sc-cron still has no global in-flight lock and no two-row mutex.

Illustrative shape only (not a frozen contract):

```rust
// embedder side: value comes from the app's own sc-config JSON.
let limits = ScheduleLimits { max_rows: Some(NonZeroUsize::new(10).unwrap()) };
let scheduler = Scheduler::open(path, limits /* , ... */)?;
```

Sync fallback (Rust built-in vs consumer Python) stays **commands and row semantics** in bd-cache: named rows, enable flags, and transactions (`replace_and_set_enabled`, `set_enabled_many`). The sync pipeline's mutual exclusion is bd-cache's (`REQ-BDC-0722`), not sc-cron's.

Follow-up in the **beads-cache** repo: amend **REQ-BDC-0720** (and related CLI text) toward "any rows up to the app's `max_rows`" and wire bd-cache's own config key to the sc-cron limit; link from bead `run-zif.2`.

Binding record: [ADR-RUN-0014](../../architecture.md) (Active, owner 2026-10-27). [ADR-RUN-0013](../../architecture.md) consequences are amended on this branch accordingly.

## Out of scope

- PR #68 and bead `run-9ny`. Row-limit code is **b-12**, not this decision PR alone.
- A Monty hook target. beads-cache does not take one.
- Implementing a beads-cache patch from this repo.
- Any sc-config reader inside sc-cron.

## Consequences

No sc-cron code moves because of this decision alone. The b-3 port copies a general scheduler; the planned row limit is an optional embedder-supplied parameter on top of it. beads-cache must align its requirements and API with the row-limit ruling in a separate beads-cache change set.
