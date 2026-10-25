# sc-cron Requirements

**Status:** Active as of 2026-10-25. The contract is [architecture.md](architecture.md) and [ADR-RUN-0013](../architecture.md). Sprint `b-3` implements these items.

## REQ-CRON-0001: Workspace member

`crates/sc-cron` is a workspace member, ported from beads-cache `crates/sc-cron` at `develop`. It depends on no other workspace crate. The template does not name it. `sc-runtime` does not name it.

## REQ-CRON-0002: Normal crontab strings

`CronExpr::parse("*/5 * * * *")` succeeds and means every fifth minute. `CronExpr::parse("0 */5 * * * *")` succeeds and means second 0 of every fifth minute. `CronExpr::parse("@hourly")` succeeds.

## REQ-CRON-0003: Rust or Python hook

A schedule row names either the in-process `Job` or a Python module. For Python, the crate runs `python3 <module> cron.tick` or `python3 <module> cron.stop` and writes one JSON object to stdin. The object includes the schedule name and the row payload. The crate exposes no HTTP route, MCP tool, or binary. A consumer may build one that inserts rows.

## REQ-CRON-0004: One configuration table

The crate creates one schedule table. A test writes a row without constructing `SqliteWriter`. The table is the configuration. The crate does not migrate the consumer's other tables.

## REQ-CRON-0005: One error enum

Fallible public functions return `CronError`. The variant set and codes are the ones in the architecture file.
