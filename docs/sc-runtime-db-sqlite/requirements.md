# sc-runtime-db-sqlite Requirements

**Amended 2026-11-11.** See the note at the top of [architecture.md](architecture.md): PRD-STO-3 puts this writer behind `Storage<T>` with an async input.

**Status:** Active as of 2026-10-25. The contract is [architecture.md](architecture.md) and [ADR-RUN-0012](../architecture.md). Sprint `b-1` implements these items.

## REQ-DBSQ-0001: Workspace member

**Status:** Active  
**Version:** 1.0.0  
**Created:** 2026-10-25  
**Last Updated:** 2026-11-12
**Owner:** randlee  

### Requirement Statement

The SQLite backend implements the shared `sc-storage` contract with asynchronous input. [REQ-RUN-0005](../requirements.md) point 4 refers to this item.


**MUST statements:**


- `crates/sc-runtime-db-sqlite` MUST be a workspace member.

- The SQLite backend MUST use `rusqlite` with the `bundled` feature.
- The generic SQLite backend MUST use only `sc-storage`, `rusqlite` and `tokio` as its normal dependencies.



**MUST NOT statements:**


- The SQLite backend MUST NOT depend on `sc-cron`.

- The SQLite backend MUST NOT depend on `sqlx`.

- The SQLite backend MUST NOT contain consumer table schemas or SQL.

### Rationale

ADR-RUN-0020 places the copied writer behind the shared storage contract while keeping consumer schemas outside the engine. The `rusqlite` `bundled` feature avoids a system SQLite dependency.
### Success Criteria

**Acceptance Criteria:**

- [ ] The crate's boundary manifest permits the shared `sc-storage` dependency and forbids `sc-cron`.
- [ ] `cargo tree -p sc-runtime-db-sqlite -e normal --prefix none` prints no line `sqlx`.

## REQ-DBSQ-0002: Public items

**Version:** 1.0.0  
**Created:** 2026-10-25  
**Last Updated:** 2026-11-12
**Owner:** randlee  
**Status:** Active  

### Requirement Statement

Definitions: `SqliteOpenOptions` names `queue_capacity` and `reader_count` with nonzero types.

The public surface is the SQLite implementation of the shared asynchronous `Storage<T>` contract.


**MUST statements:**


- `SqliteDatabase` MUST expose an inherent `open` operation.

- `SqliteDatabase` MUST expose an inherent `storage` operation.

- `SqliteDatabase` MUST expose an inherent `shutdown` operation.

- `SqliteDatabase` MUST expose an inherent `wait_closed` operation.

- `SqliteDatabase::open` MUST use `SqliteOpenOptions`.

- The backend MUST expose the typed `SqliteStore<T>` implementation of `Storage<T>`.



**MUST NOT statements:**


- The backend MUST NOT expose raw connection or SQL machinery publicly.

- The backend MUST NOT expose the legacy `SqliteWriter`, `WriterConn`, `SqlValue` or `SqliteWriterError` facade in version 0.3.0.

### Rationale

ADR-RUN-0020 and the Phase D storage contract retain the proven writer machinery behind typed generic storage and retire the raw SQL facade.
### Success Criteria

**Acceptance Criteria:**

- [ ] `sc-lint lint sc-boundary` exits 0 with the crate's boundary manifest matching the implemented typed public facade.

## REQ-DBSQ-0003: Reply after commit

**Status:** Active  
**Version:** 1.0.0  
**Created:** 2026-10-25  
**Last Updated:** 2026-11-12
**Owner:** randlee  

### Requirement Statement

**MUST statements:**


- The writer thread MUST exclusively own the write-mode SQLite connection.

- The SQLite writer MUST send successful replies only after the outer transaction commits.
### Rationale

ADR-RUN-0020 preserves the measured atm-core writer path: a bounded channel, one write-mode connection on the writer thread, and successful replies only after the outer commit. Reader workers use read-only connections.
### Success Criteria

**Acceptance Criteria:**

- [ ] `cargo test -p sc-runtime-db-sqlite` exits 0, covering successful write replies after the outer commit.

## REQ-DBSQ-0004: One commit order

**Status:** Active
**Version:** 1.0.0  
**Created:** 2026-10-25  
**Last Updated:** 2026-11-12
**Owner:** randlee  

### Requirement Statement

Definitions: An admitted write operation is a write accepted into the SQLite writer queue. The first and second operations are ordered by their admission to that queue.


**MUST NOT statements:**


- For two admitted write operations, the second reply MUST NOT be sent before the first operation has committed or failed.

### Rationale

ADR-RUN-0020 requires batched SQLite writes and successful replies after the outer transaction commits.
### Success Criteria

**Acceptance Criteria:**

- [ ] A test controls admission and commit outcomes for two writes and checks their replies against the required order.
