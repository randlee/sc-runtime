# sc-cron Requirements

**Status:** Active as of 2026-10-25. The contract is [architecture.md](architecture.md) and [ADR-RUN-0013](../architecture.md). Sprint `b-3` implements these items.

## REQ-CRON-0001: Workspace member

`crates/sc-cron` is a workspace member, ported from beads-cache `crates/sc-cron` at `develop`. It depends on no other workspace crate. The template does not name it. `sc-runtime` does not name it.

## REQ-CRON-0002: Normal crontab strings

`CronExpr::parse("*/5 * * * *")` succeeds and means every fifth minute. `CronExpr::parse("0 */5 * * * *")` succeeds and means second 0 of every fifth minute. `CronExpr::parse("@hourly")` succeeds.

## REQ-CRON-0003: The app registers the hook

`Scheduler::start_async` stores a schedule. `spawn_worker` calls the app's `Job`. `stop_async` stops a schedule. The crate exposes no HTTP route, MCP tool, or binary.

## REQ-CRON-0004: Own schedule file

`SqliteScheduleStore` opens its own file. A test can show a schedule write without constructing `SqliteWriter`.

## REQ-CRON-0005: One error enum

Fallible public functions return `CronError`. The variant set and codes are the ones in the architecture file.
