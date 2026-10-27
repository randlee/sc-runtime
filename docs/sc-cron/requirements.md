# sc-cron Requirements

**Status:** Active as of 2026-10-25. The contract is [architecture.md](architecture.md) and [ADR-RUN-0013](../architecture.md). Sprint `b-3` implements these items.

## REQ-CRON-0001: Workspace member

`crates/sc-cron` is a workspace member, ported from beads-cache `crates/sc-cron` at `develop`. It depends on no other workspace crate. The template does not name it. `sc-runtime` does not name it.

## REQ-CRON-0002: Normal crontab strings

A schedule uses a normal crontab string of five fields: minute, hour, day of month, month, and day of week. `*/5 * * * *` means every fifth minute.

## REQ-CRON-0003: Rust, Python, or Monty hook

A schedule row names the in-process `Job`, a Python module, or a third target, `Monty { source }`, when the crate is built with feature `monty`. Python and Monty receive the `sc-cron.hook.v1` object with `protocol`, `event`, `schedule`, `payload`, and `deadline`; it has no beads, bd-cache, or ATM field. The crate exposes no HTTP route, MCP tool, or binary. A consumer may build one that inserts rows.

## REQ-CRON-0004: One configuration table

The crate creates one schedule table. A test writes a row without constructing `SqliteWriter`. The table is the configuration. The crate does not migrate the consumer's other tables.

## REQ-CRON-0005: One error enum

Fallible public functions return `CronError`. The variants are `BadExpression`, `InvalidSpec`, `AlreadyActive`, `NotFound`, `HookFailed`, `Store`, and `Fenced`, with codes `BAD_EXPRESSION`, `INVALID_SPEC`, `ALREADY_ACTIVE`, `NOT_FOUND`, `HOOK_FAILED`, `STORE_FAILED`, and `FENCED`.
