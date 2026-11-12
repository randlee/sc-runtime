# sc-runtime-db-sqlite Requirements

**Status:** Active

Active as of 2026-10-25. The contract is [architecture.md](architecture.md) and [ADR-RUN-0012](../architecture.md). Sprint `b-1` implements these items.

## REQ-DBSQ-0001: Workspace member

**Status:** Active  
**Version:** 1.0.0  
**Created:** 2026-10-25  
**Last Updated:** 2026-10-25  
**Owner:** randlee  

### Requirement Statement

`crates/sc-runtime-db-sqlite` is a workspace member. This item owns its third-party dependencies: `rusqlite` with the `bundled` feature, and no other dependency. It depends on no other workspace crate.

### Rationale

Owner, 2026-10-25 (ADR-RUN-0012): copy the atm-core writer machine into a library crate; do not put it behind a SQL trait shared with sqlx. The rusqlite `bundled` feature gives zero system SQLite dependency (PRD NF-04).

### Success Criteria

**Acceptance Criteria:**

- [ ] The crate's boundary manifest forbids every other workspace crate.
- [ ] `cargo tree -p sc-runtime-db-sqlite -e normal --prefix none` prints no line `sqlx`.

### Related Documents

**Requirements:**

- [REQ-RUN-0005](../requirements.md) — point 4 points here

## REQ-DBSQ-0002: Public items

**Status:** Active  
**Version:** 1.0.0  
**Created:** 2026-10-25  
**Last Updated:** 2026-10-25  
**Owner:** randlee  

### Requirement Statement

The public items are `SqliteWriter`, `WriterConn`, `SqlValue`, `SqliteWriterError`, `CHANNEL_CAPACITY`, `BATCH_TIME_BUDGET`, `WRITE_DEADLINE`, and `SHUTDOWN_JOIN`, with the signatures in the architecture file.

### Rationale

Sprint b-1 copies the atm-core writer with these types substituted for its `WriteOp`, `WriteOpResult`, `AtmError`, and `SharedDbTarget`.

### Success Criteria

**Acceptance Criteria:**

- [ ] `sc-lint lint sc-boundary` exits 0 with the crate's boundary manifest, whose `[public] facade` lists these public types.

## REQ-DBSQ-0003: Reply after commit

**Status:** Active  
**Version:** 1.0.0  
**Created:** 2026-10-25  
**Last Updated:** 2026-10-25  
**Owner:** randlee  

### Requirement Statement

The writer thread is the only caller of rusqlite. `execute` replies with the number of rows affected only after the outer commit.

### Rationale

ADR-RUN-0012 copies the measured atm-core path: one rusqlite connection on one thread, a bounded channel, `BEGIN IMMEDIATE`, and a reply only after commit.

### Success Criteria

**Acceptance Criteria:**

- [ ] `cargo test -p sc-runtime-db-sqlite` exits 0, covering one `execute` round-trip and two concurrent `execute` calls with reply-after-commit.

## REQ-DBSQ-0004: One commit order

**Status:** Active  
**Version:** 1.0.0  
**Created:** 2026-10-25  
**Last Updated:** 2026-10-25  
**Owner:** randlee  

### Requirement Statement

A test of two concurrent `execute` calls observes one commit order: the second reply is not sent before the first statement has committed or failed.

### Rationale

ADR-RUN-0012 point 3: contiguous executes share one savepoint inside one `BEGIN IMMEDIATE` transaction, and each caller is answered only after that transaction commits.

### Success Criteria

**Acceptance Criteria:**

- [ ] A test of two concurrent `execute` calls observes one commit order: the second reply is not sent before the first statement has committed or failed.

## REQ-DBSQ-0005: No runtime or template edge

**Status:** Active  
**Version:** 1.0.0  
**Created:** 2026-10-25  
**Last Updated:** 2026-10-25  
**Owner:** randlee  

### Requirement Statement

Template `store-sqlite` does not name this crate. `sc-runtime` does not name it.

### Rationale

ADR-RUN-0301 binds `template/crates/store-sqlite` to sqlx 0.9 with no write actor; a `StorageBackend` that runs the same SQL on rusqlite and on Postgres is the portable query layer that ADR rejects. `sc-runtime` does not open the writer (ADR-RUN-0012).

### Success Criteria

**Acceptance Criteria:**

- [ ] `rg sc-runtime-db-sqlite template` prints nothing.
- [ ] `sc-lint lint sc-boundary` exits 0 with `sc-runtime-db-sqlite` in the forbidden edges of `sc-runtime`.
