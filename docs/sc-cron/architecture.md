# sc-cron Architecture

**Status:** Active as of 2026-10-25. Binding decision: [ADR-RUN-0013](../architecture.md). Sprint `b-3` ports beads-cache `crates/sc-cron` and applies the expression rule below.

## What this crate is

A SQLite-backed cron scheduler. The app supplies a `Job`. The crate stores schedules and calls that hook. It has no HTTP, MCP, or CLI surface.

## Schedule strings

A normal string has five fields, minute first, in UTC: `minute hour day-of-month month day-of-week`. Example: `*/5 * * * *`.

`@yearly`, `@monthly`, `@weekly`, `@daily`, and `@hourly` are valid.

A six-field string is seconds first: `second minute hour day-of-month month day-of-week`. Example: `0 */5 * * * *`. A five-field string is never read as seconds.

## Registration

```rust
pub struct Scheduler<S, C, J> { /* store, clock, jitter */ }

impl Scheduler<SqliteScheduleStore, SystemClock, SystemJitter> {
    pub async fn start_async(&self, spec: ScheduleSpec) -> Result<Schedule, CronError>;
    pub async fn stop_async(&self, id: ScheduleId, generation: Generation, reason: &str) -> Result<(), CronError>;
    pub fn spawn_worker(&self, job: std::sync::Arc<dyn Job>) -> WorkerHandle;
}

pub struct ScheduleSpec { /* name, cron string, timeout, JSON payload */ }

#[async_trait::async_trait]
pub trait Job: Send + Sync {
    async fn run(&self, payload: &serde_json::Value, deadline: std::time::Instant) -> RunOutcome;
    async fn should_stop(&self, payload: &serde_json::Value) -> StopDecision;
}
```

`SqliteScheduleStore::open` takes the path of its own database file. That connection is not `SqliteWriter`.

## Error

`CronError` is `#[non_exhaustive]` and implements `Display` and `std::error::Error`. Variants: `BadExpression(String)`, `InvalidSpec(String)`, `AlreadyActive(ScheduleName)`, `Store(StoreError)`, `Fenced { id, generation }`. Codes: `BAD_EXPRESSION`, `INVALID_SPEC`, `ALREADY_ACTIVE`, `STORE_FAILED`, `FENCED`. The `BadExpression` recovery text tells the caller to use a five-field string, a six-field seconds-first string, or `@hourly` through `@yearly`.

## What this crate is not

It does not open Postgres, call beads, or know whether the hook copies a cache or syncs through the beads CLI. beads-cache's hook does that second job. The app decides when a schedule exists by calling `start_async`.
