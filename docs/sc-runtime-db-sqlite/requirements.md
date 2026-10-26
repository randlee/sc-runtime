# sc-runtime-db-sqlite Requirements

**Status:** Active as of 2026-10-25. The contract is [architecture.md](architecture.md) and [ADR-RUN-0012](../architecture.md). Sprint `b-1` implements these items.

## REQ-DBSQ-0001: Workspace member

`crates/sc-runtime-db-sqlite` is a workspace member. It depends on `rusqlite` with the `bundled` feature and on no other workspace crate.

## REQ-DBSQ-0002: Public items

The public items are `SqliteWriter`, `WriterConn`, `SqlValue`, `SqliteWriterError`, `CHANNEL_CAPACITY`, `BATCH_TIME_BUDGET`, `WRITE_DEADLINE`, and `SHUTDOWN_JOIN`, with the signatures in the architecture file.

## REQ-DBSQ-0003: Reply after commit

The writer thread is the only caller of rusqlite. `execute` replies with the number of rows affected only after the outer commit.

## REQ-DBSQ-0004: One commit order

A test of two concurrent `execute` calls observes one commit order: the second reply is not sent before the first statement has committed or failed.

## REQ-DBSQ-0005: No runtime or template edge

Template `store-sqlite` does not name this crate. `sc-runtime` does not name it.
