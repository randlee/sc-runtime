# sc-runtime-db-sqlite Requirements

**Amended 2026-11-11.** See the note at the top of [architecture.md](architecture.md): PRD-STO-3 puts this writer behind `Storage<T>` with an async input.

**Status:** Active as of 2026-10-25. The contract is [architecture.md](architecture.md) and [ADR-RUN-0012](../architecture.md). Sprint `b-1` implements these items.

## REQ-DBSQ-0001: Workspace member

**Amended 2026-11-11 (PRD-STO-3).** The async input needs an async channel, so `rusqlite` will not be the only dependency.

`crates/sc-runtime-db-sqlite` is a workspace member. This item owns its third-party dependencies: `rusqlite` with the `bundled` feature, and no other dependency. It depends on no other workspace crate. [REQ-RUN-0005](../requirements.md) point 4 points here.

## REQ-DBSQ-0002: Public items

**Amended 2026-11-11 (PRD-STO-1, PRD-STO-3).** The public surface becomes the SQLite implementation of `Storage<T>` with an async input. The blocking `SqliteWriter` signatures are the code as it is, not the target.

The public items are `SqliteWriter`, `WriterConn`, `SqlValue`, `SqliteWriterError`, `CHANNEL_CAPACITY`, `BATCH_TIME_BUDGET`, `WRITE_DEADLINE`, and `SHUTDOWN_JOIN`, with the signatures in the architecture file.

## REQ-DBSQ-0003: Reply after commit

The writer thread is the only caller of rusqlite. `execute` replies with the number of rows affected only after the outer commit.

## REQ-DBSQ-0004: One commit order

A test of two concurrent `execute` calls observes one commit order: the second reply is not sent before the first statement has committed or failed.

## REQ-DBSQ-0005: No runtime or template edge

**Status:** Superseded by PRD-STO-3 (2026-11-11)  
The template's SQLite store and the runtime use this writer. The text below binds nothing.

Template `store-sqlite` does not name this crate. `sc-runtime` does not name it.
