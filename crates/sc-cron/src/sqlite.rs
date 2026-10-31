//! [`ScheduleStore`] over SQLite (bundled), in its own database file.
//!
//! The store owns its file and its one connection (REQ-CRON-0009): it never
//! opens a second connection to another store's file. It is sc-cron's
//! standalone default; a caller may supply any other [`ScheduleStore`]. On open it sets the
//! busy timeout first, then WAL, then checks the schema version in
//! `PRAGMA user_version`. A fresh file (version 0, no tables) gets the
//! complete first-release schema and version [`SCHEMA_VERSION`]. Existing
//! databases must match its semantic columns, defaults, constraints and indexes.
//! Incompatible development databases are rejected without migration or data loss.
//!
//! `create` runs as one immediate transaction, so the active-name check, the
//! embedder row-limit check, and the write cannot interleave with another
//! writer; so do `set_enabled_many` and `replace_and_set_enabled`, which
//! change several rows at once (REQ-CRON-0017). Run-state writes are
//! guarded by `id`, `generation` and `ended_at IS NULL`. Table CHECKs keep
//! `next_run_at` set exactly while a schedule is active and `stop_reason`
//! set exactly once it has ended, and repeat the size limits. Timestamps
//! become text only here, in the fixed-width form [`Timestamp`] displays.

use std::collections::HashSet;
use std::num::NonZeroUsize;
use std::path::Path;
use std::sync::{Mutex, MutexGuard, PoisonError};
use std::time::Duration;

use rusqlite::{
    Connection, ErrorCode, OptionalExtension, Row, ToSql, Transaction, TransactionBehavior, params,
};

use crate::limits::{
    MAX_EXPR_LEN, MAX_HOOK_SOURCE_BYTES, MAX_NAME_LEN, MAX_PAYLOAD_BYTES, ScheduleLimits,
    encode_payload,
};
use crate::{
    CronError, CronExpr, Generation, HookTarget, NewSchedule, RunOutcome, RunTimeout, Schedule,
    ScheduleId, ScheduleName, ScheduleSpec, ScheduleStore, StoreError, StoreErrorKind, Timestamp,
};

use crate::MAX_ENV_DOCUMENT_BYTES;

const TABLE: &str = "sc_cron_schedule";

/// Complete first-release schema. Unreleased development schemas are not migrated.
pub const SCHEMA_VERSION: i64 = 1;

#[path = "sqlite_schema.rs"]
mod schema_check;

fn schema() -> String {
    format!(
        "CREATE TABLE sc_cron_schedule (
    id           INTEGER PRIMARY KEY AUTOINCREMENT,
    generation   INTEGER NOT NULL CHECK (generation BETWEEN 1 AND 4294967295),
    name         TEXT    NOT NULL UNIQUE
                 CHECK (length(CAST(name AS BLOB)) BETWEEN 1 AND {MAX_NAME_LEN}),
    expr         TEXT    NOT NULL CHECK (length(CAST(expr AS BLOB)) <= {MAX_EXPR_LEN}),
    timeout_ms   INTEGER NOT NULL CHECK (timeout_ms > 0),
    jitter_ms    INTEGER NOT NULL CHECK (jitter_ms >= 0),
    payload      TEXT    NOT NULL
                 CHECK (length(CAST(payload AS BLOB)) <= {MAX_PAYLOAD_BYTES}),
    created_at   TEXT    NOT NULL,
    next_run_at  TEXT,
    last_run_at  TEXT,
    last_outcome TEXT,
    ended_at     TEXT,
    stop_reason  TEXT,
    enabled      INTEGER NOT NULL DEFAULT 1 CHECK (enabled IN (0, 1)),
    hook_kind    TEXT    NOT NULL DEFAULT 'rust' CHECK (hook_kind IN ('rust', 'python', 'monty')),
    hook_module  TEXT    NOT NULL DEFAULT '',
    hook_source  TEXT    NOT NULL DEFAULT ''
                 CHECK (length(CAST(hook_source AS BLOB)) <= {MAX_HOOK_SOURCE_BYTES}),
    hook_environment TEXT NOT NULL DEFAULT '{{\"dotenv\":[],\"variables\":{{}}}}'
                 CHECK (length(CAST(hook_environment AS BLOB)) <= {MAX_ENV_DOCUMENT_BYTES}),
    CHECK ((ended_at IS NULL) = (next_run_at IS NOT NULL)),
    CHECK ((ended_at IS NULL) = (stop_reason IS NULL))
);"
    )
}

const COLUMNS: &str = "id, generation, name, expr, timeout_ms, jitter_ms, payload, created_at, \
     next_run_at, last_run_at, last_outcome, ended_at, stop_reason, enabled, \
     hook_kind, hook_module, hook_source, hook_environment";

/// How long a writer waits for another connection's lock on the same file.
const BUSY_TIMEOUT: Duration = Duration::from_secs(5);

/// SQLite-backed schedule store on its own database file.
///
/// # Examples
///
/// ```
/// use sc_cron::{ScheduleLimits, ScheduleStore, SqliteScheduleStore};
///
/// let store = SqliteScheduleStore::open_in_memory(ScheduleLimits::default())?;
/// assert!(store.list_active()?.is_empty());
/// # Ok::<(), sc_cron::CronError>(())
/// ```
pub struct SqliteScheduleStore {
    logger: sc_observability_types::SharedCanonicalLog,
    // A rusqlite `Connection` is `Send` but not `Sync`, and the port's
    // methods take `&self` from any thread, so the one connection sits
    // behind a mutex. Calls are short single statements or one transaction.
    connection: Mutex<Connection>,
    // Row ids already warned about as corrupt, so a bad row is reported once
    // per store rather than on every worker reload.
    warned: Mutex<HashSet<i64>>,
    /// Embedder row ceiling. `None` does not enforce a count (ADR-RUN-0014).
    limits: ScheduleLimits,
}

impl std::fmt::Debug for SqliteScheduleStore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SqliteScheduleStore")
            .field("limits", &self.limits)
            .field("service", self.logger.service_name())
            .finish_non_exhaustive()
    }
}

impl SqliteScheduleStore {
    /// Opens or creates the sc-cron database at `path`.
    ///
    /// `limits` is enforced in the same transaction as an insert of a new
    /// name. [`ScheduleLimits::default`] does not cap the table. The value
    /// stays in this process and is not written to the database. Every
    /// writer of this file must pass the same `limits`.
    ///
    /// # Errors
    ///
    /// [`CronError::Store`]: [`StoreErrorKind::Unavailable`] if the file
    /// cannot be opened, [`StoreErrorKind::SchemaMismatch`] if it holds an
    /// unknown schema version or another store's tables.
    pub fn open(path: impl AsRef<Path>, limits: ScheduleLimits) -> Result<Self, CronError> {
        Self::open_with_logger(path, limits, crate::noop_logger())
    }

    /// Opens a schedule database with a best-effort canonical logger.
    ///
    /// # Errors
    /// Returns the same store errors as `open`.
    pub fn open_with_logger(
        path: impl AsRef<Path>,
        limits: ScheduleLimits,
        logger: sc_observability_types::SharedCanonicalLog,
    ) -> Result<Self, CronError> {
        let connection = Connection::open(path.as_ref())
            .map_err(|error| store_error("failed to open database", error))?;
        connection
            .busy_timeout(BUSY_TIMEOUT)
            .map_err(|error| store_error("failed to set busy timeout", error))?;
        connection
            .pragma_update(None, "journal_mode", "WAL")
            .map_err(|error| store_error("failed to enable WAL", error))?;
        Self::with_connection(connection, limits, logger)
    }

    /// Opens a private in-memory database, for tests and ephemeral use.
    ///
    /// `limits` is the same ceiling [`SqliteScheduleStore::open`] stores.
    ///
    /// # Errors
    ///
    /// [`CronError::Store`] if SQLite cannot allocate the database.
    pub fn open_in_memory(limits: ScheduleLimits) -> Result<Self, CronError> {
        let connection = Connection::open_in_memory()
            .map_err(|error| store_error("failed to open database", error))?;
        connection
            .busy_timeout(BUSY_TIMEOUT)
            .map_err(|error| store_error("failed to set busy timeout", error))?;
        Self::with_connection(connection, limits, crate::noop_logger())
    }

    /// Returns the row ceiling this store was opened with.
    pub fn limits(&self) -> ScheduleLimits {
        self.limits
    }

    fn with_connection(
        mut connection: Connection,
        limits: ScheduleLimits,
        logger: sc_observability_types::SharedCanonicalLog,
    ) -> Result<Self, CronError> {
        init_schema(&mut connection)?;
        Ok(Self {
            connection: Mutex::new(connection),
            logger,
            warned: Mutex::new(HashSet::new()),
            limits,
        })
    }

    /// Warns about a skipped corrupt row, once per row id for this store.
    fn warn_corrupt(&self, id: Option<i64>, name: Option<&str>, error: &CronError) {
        if let Some(id) = id {
            let mut warned = self.warned.lock().unwrap_or_else(PoisonError::into_inner);
            if !warned.insert(id) {
                return;
            }
        }
        let mut fields = serde_json::Map::new();
        fields.insert("error".into(), error.to_string().into());
        if let Some(id) = id {
            fields.insert("schedule_id".into(), id.into());
        }
        if let Some(name) = name {
            fields.insert("schedule_name".into(), name.into());
        }
        if let Some(event) = crate::component_event(
            self.logger.service_name().clone(),
            "cron.store",
            "row.corrupt",
            sc_observability_types::Level::Warn,
            "failed",
            Some("CORRUPT_SCHEDULE_ROW"),
            fields,
        ) {
            let _ = self.logger.emit(event);
        }
    }

    fn lock(&self) -> MutexGuard<'_, Connection> {
        // A panic while the lock was held cannot leave a transaction open:
        // rusqlite rolls back on drop, so the connection is still sound.
        self.connection
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
    }

    /// Runs one guarded `UPDATE` and reports whether it changed the row.
    fn guarded_update(
        &self,
        sql: &str,
        id: ScheduleId,
        generation: Generation,
        values: &[&dyn ToSql],
    ) -> Result<bool, CronError> {
        let Ok(id) = i64::try_from(id.get()) else {
            // No stored row can carry an id above i64::MAX.
            return Ok(false);
        };
        let generation = generation.get();
        let mut all: Vec<&dyn ToSql> = vec![&id, &generation];
        all.extend_from_slice(values);
        let changed = self
            .lock()
            .execute(sql, all.as_slice())
            .map_err(|error| store_error(&format!("failed to update {TABLE} id {id}"), error))?;
        Ok(changed == 1)
    }
}

/// Applies or checks the schema in one immediate transaction.
fn init_schema(connection: &mut Connection) -> Result<(), CronError> {
    let transaction = connection
        .transaction_with_behavior(TransactionBehavior::Immediate)
        .map_err(|error| store_error("failed to begin schema transaction", error))?;
    let version: i64 = transaction
        .query_row("PRAGMA user_version", [], |row| row.get(0))
        .map_err(|error| store_error("failed to read schema version", error))?;
    match version {
        0 => {
            let tables: i64 = transaction
                .query_row(
                    "SELECT count(*) FROM sqlite_master
                     WHERE type = 'table' AND name NOT LIKE 'sqlite_%'",
                    [],
                    |row| row.get(0),
                )
                .map_err(|error| store_error("failed to inspect database", error))?;
            if tables > 0 {
                return Err(StoreError::new(
                    StoreErrorKind::SchemaMismatch,
                    format!(
                        "database holds {tables} other table(s); sc-cron needs its own database file"
                    ),
                )
                .into());
            }
            transaction
                .execute_batch(&schema())
                .map_err(|error| store_error("failed to create schema", error))?;
            transaction
                .pragma_update(None, "user_version", SCHEMA_VERSION)
                .map_err(|error| store_error("failed to set schema version", error))?;
        }
        SCHEMA_VERSION => schema_check::validate(&transaction)?,
        other => {
            return Err(StoreError::new(
                StoreErrorKind::SchemaMismatch,
                format!("schema version {other} is not {SCHEMA_VERSION}"),
            )
            .into());
        }
    }
    transaction
        .commit()
        .map_err(|error| store_error("failed to commit schema", error))
}

impl ScheduleStore for SqliteScheduleStore {
    fn limits(&self) -> ScheduleLimits {
        self.limits
    }

    fn create(&self, schedule: &NewSchedule, created_at: Timestamp) -> Result<Schedule, CronError> {
        let columns = Columns::new(schedule, created_at)?;
        let mut connection = self.lock();
        let transaction = connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(|error| store_error("failed to begin transaction", error))?;
        let existing = transaction
            .query_row(
                "SELECT id, generation, ended_at IS NULL FROM sc_cron_schedule WHERE name = ?1",
                [schedule.spec.name.as_str()],
                |row| {
                    Ok((
                        row.get::<_, i64>(0)?,
                        row.get::<_, i64>(1)?,
                        row.get::<_, bool>(2)?,
                    ))
                },
            )
            .optional()
            .map_err(|error| store_error(&format!("failed to read {TABLE}"), error))?;
        let id = match existing {
            Some((_, _, true)) => {
                return Err(CronError::AlreadyActive(schedule.spec.name.clone()));
            }
            Some((id, generation, false)) => {
                reserve_new_row(self.limits, &transaction)?;
                columns.rewrite(&transaction, id, generation, true)?;
                id
            }
            None => {
                reserve_new_row(self.limits, &transaction)?;
                columns.insert(&transaction, schedule.spec.name.as_str(), true)?
            }
        };
        let row = select_one(&transaction, id)?.ok_or_else(|| {
            StoreError::new(
                StoreErrorKind::Other,
                format!("{TABLE} id {id}: missing after create"),
            )
        })?;
        transaction
            .commit()
            .map_err(|error| store_error("failed to commit", error))?;
        Ok(row)
    }

    fn get(&self, id: ScheduleId) -> Result<Option<Schedule>, CronError> {
        let Ok(id) = i64::try_from(id.get()) else {
            return Ok(None);
        };
        select_one(&self.lock(), id)
    }

    fn list_active(&self) -> Result<Vec<Schedule>, CronError> {
        let connection = self.lock();
        let mut statement = connection
            .prepare(&format!(
                "SELECT {COLUMNS} FROM sc_cron_schedule WHERE ended_at IS NULL
                 ORDER BY created_at, id"
            ))
            .map_err(|error| store_error(&format!("failed to read {TABLE}"), error))?;
        let rows = statement
            .query_map([], |row| {
                // Keep the id even when the rest of the row cannot be read.
                Ok(RawRow::read(row).map_err(|error| (row.get::<_, i64>(0).ok(), error)))
            })
            .and_then(Iterator::collect::<Result<Vec<_>, _>>)
            .map_err(|error| store_error(&format!("failed to read {TABLE}"), error))?;
        let mut schedules = Vec::with_capacity(rows.len());
        for row in rows {
            let (id, name, error) = match row {
                Ok(raw) => {
                    let (id, name) = (raw.id, raw.name.clone());
                    match raw.decode() {
                        Ok(schedule) => {
                            schedules.push(schedule);
                            continue;
                        }
                        Err(error) => (Some(id), Some(name), error),
                    }
                }
                Err((id, error)) => (
                    id,
                    None,
                    store_error(&format!("failed to read {TABLE}"), error),
                ),
            };
            // A row that cannot be decoded must not stop the others: skip it.
            // Any other failure (busy, I/O) still fails the whole call.
            match &error {
                CronError::Store(store) if store.kind() == StoreErrorKind::Corrupt => {
                    self.warn_corrupt(id, name.as_deref(), &error);
                }
                _ => return Err(error),
            }
        }
        Ok(schedules)
    }

    fn record_run(
        &self,
        id: ScheduleId,
        generation: Generation,
        at: Timestamp,
        outcome: &RunOutcome,
    ) -> Result<bool, CronError> {
        self.guarded_update(
            "UPDATE sc_cron_schedule SET last_run_at = ?3, last_outcome = ?4
             WHERE id = ?1 AND generation = ?2 AND ended_at IS NULL",
            id,
            generation,
            &[&at.to_string(), &outcome.to_string()],
        )
    }

    fn set_next(
        &self,
        id: ScheduleId,
        generation: Generation,
        next_run_at: Timestamp,
    ) -> Result<bool, CronError> {
        self.guarded_update(
            "UPDATE sc_cron_schedule SET next_run_at = ?3
             WHERE id = ?1 AND generation = ?2 AND ended_at IS NULL AND enabled = 1",
            id,
            generation,
            &[&next_run_at.to_string()],
        )
    }

    fn stop(
        &self,
        id: ScheduleId,
        generation: Generation,
        at: Timestamp,
        reason: &str,
    ) -> Result<bool, CronError> {
        self.guarded_update(
            "UPDATE sc_cron_schedule SET ended_at = ?3, stop_reason = ?4, next_run_at = NULL
             WHERE id = ?1 AND generation = ?2 AND ended_at IS NULL",
            id,
            generation,
            &[&at.to_string(), &reason],
        )
    }

    fn set_enabled(&self, name: &ScheduleName, enabled: bool) -> Result<bool, CronError> {
        let changed = self
            .lock()
            .execute(
                "UPDATE sc_cron_schedule SET enabled = ?2
                 WHERE name = ?1 AND ended_at IS NULL",
                params![name.as_str(), enabled],
            )
            .map_err(|error| store_error(&format!("failed to update {TABLE} {name}"), error))?;
        Ok(changed == 1)
    }

    fn disable(
        &self,
        id: ScheduleId,
        generation: Generation,
        at: Timestamp,
        reason: &str,
    ) -> Result<bool, CronError> {
        self.guarded_update(
            "UPDATE sc_cron_schedule SET enabled = 0, last_run_at = ?3, last_outcome = ?4
             WHERE id = ?1 AND generation = ?2 AND ended_at IS NULL",
            id,
            generation,
            &[
                &at.to_string(),
                &RunOutcome::Refused(reason.to_owned()).to_string(),
            ],
        )
    }

    fn set_enabled_many(&self, flags: &[(ScheduleName, bool)]) -> Result<bool, CronError> {
        let mut connection = self.lock();
        let transaction = connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(|error| store_error("failed to begin transaction", error))?;
        // A missing name returns before the commit: the transaction rolls
        // back on drop, so no flag of the list changes (REQ-CRON-0017).
        if !set_flags(&transaction, flags)? {
            return Ok(false);
        }
        transaction
            .commit()
            .map_err(|error| store_error("failed to commit", error))?;
        Ok(true)
    }

    fn replace_and_set_enabled(
        &self,
        schedule: &NewSchedule,
        enabled: bool,
        created_at: Timestamp,
        flags: &[(ScheduleName, bool)],
    ) -> Result<Option<Schedule>, CronError> {
        let name = &schedule.spec.name;
        if flags.iter().any(|(listed, _)| listed == name) {
            return Err(CronError::InvalidSpec(format!(
                "the flags to set name the written schedule {name}"
            )));
        }
        let columns = Columns::new(schedule, created_at)?;
        let mut connection = self.lock();
        let transaction = connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(|error| store_error("failed to begin transaction", error))?;
        let existing = transaction
            .query_row(
                "SELECT id, generation, ended_at IS NULL FROM sc_cron_schedule WHERE name = ?1",
                [name.as_str()],
                |row| {
                    Ok((
                        row.get::<_, i64>(0)?,
                        row.get::<_, i64>(1)?,
                        row.get::<_, bool>(2)?,
                    ))
                },
            )
            .optional()
            .map_err(|error| store_error(&format!("failed to read {TABLE}"), error))?;
        // An active name is rewritten in place and does not change the row
        // count. An ended name or a new name claims a slot. No write sets
        // another flag first.
        let id = match existing {
            Some((id, generation, true)) => {
                columns.rewrite(&transaction, id, generation, enabled)?;
                id
            }
            Some((id, generation, false)) => {
                reserve_new_row(self.limits, &transaction)?;
                columns.rewrite(&transaction, id, generation, enabled)?;
                id
            }
            None => {
                reserve_new_row(self.limits, &transaction)?;
                columns.insert(&transaction, name.as_str(), enabled)?
            }
        };
        if !set_flags(&transaction, flags)? {
            return Ok(None);
        }
        let row = select_one(&transaction, id)?.ok_or_else(|| {
            StoreError::new(
                StoreErrorKind::Other,
                format!("{TABLE} id {id}: missing after write"),
            )
        })?;
        transaction
            .commit()
            .map_err(|error| store_error("failed to commit", error))?;
        Ok(Some(row))
    }
}

/// Rejects a write that would add a non-ended row past `limits`.
///
/// Enabled and disabled rows both count. An ended row does not. The check
/// runs in the caller's immediate transaction, before the insert, so two
/// writers that enforce the same ceiling cannot both pass it (ADR-RUN-0014).
/// The ceiling is the [`ScheduleLimits`] this process opened with. It is not
/// stored in the database. A finite cap is always compared; a cap above
/// [`i64::MAX`] is not treated as unlimited.
fn reserve_new_row(limits: ScheduleLimits, transaction: &Transaction<'_>) -> Result<(), CronError> {
    let Some(limit) = limits.max_schedule_rows else {
        return Ok(());
    };
    let count: i64 = transaction
        .query_row(
            "SELECT COUNT(*) FROM sc_cron_schedule WHERE ended_at IS NULL",
            [],
            |row| row.get(0),
        )
        .map_err(|error| store_error(&format!("failed to count {TABLE}"), error))?;
    if row_count_reached(count, limit) {
        return Err(CronError::InvalidSpec(format!(
            "schedule row limit is {limit}; current row count is {count}"
        )));
    }
    Ok(())
}

/// Whether `count` is at or above the finite cap `limit`.
///
/// `count` is SQLite `COUNT(*)`, a signed integer. It is widened to `u64`
/// and then compared as `usize` against `limit`. A cap that does not fit in
/// `i64` stays a cap. A count that does not fit in `u64` or `usize` is
/// already past every finite cap this process can name.
fn row_count_reached(count: i64, limit: NonZeroUsize) -> bool {
    let Ok(count) = u64::try_from(count) else {
        return true;
    };
    match usize::try_from(count) {
        Ok(count) => count >= limit.get(),
        Err(_) => true,
    }
}

/// Sets each listed active schedule's flag inside `transaction`; `false` as
/// soon as a name has no active schedule (the caller then rolls back).
fn set_flags(
    transaction: &Transaction<'_>,
    flags: &[(ScheduleName, bool)],
) -> Result<bool, CronError> {
    for (name, enabled) in flags {
        let changed = transaction
            .execute(
                "UPDATE sc_cron_schedule SET enabled = ?2
                 WHERE name = ?1 AND ended_at IS NULL",
                params![name.as_str(), enabled],
            )
            .map_err(|error| store_error(&format!("failed to update {TABLE} {name}"), error))?;
        if changed != 1 {
            return Ok(false);
        }
    }
    Ok(true)
}

/// One schedule's column values, converted before any write.
///
/// Name, expression and timeout are valid by type; only the payload size
/// and the integer range of the durations remain to check.
struct Columns {
    expr: String,
    timeout_ms: i64,
    jitter_ms: i64,
    payload: String,
    created_at: String,
    next_run_at: String,
    hook_kind: String,
    hook_module: String,
    hook_source: String,
    hook_environment: String,
}

impl Columns {
    fn new(schedule: &NewSchedule, created_at: Timestamp) -> Result<Self, CronError> {
        let spec = &schedule.spec;
        spec.validate_environment()?;
        let (hook_kind, hook_module, hook_source) = spec.hook.stored();
        Ok(Self {
            expr: spec.expr.as_str().to_owned(),
            timeout_ms: millis("timeout", spec.timeout.get())?,
            jitter_ms: millis("jitter", schedule.jitter)?,
            payload: encode_payload(&spec.payload)?,
            created_at: created_at.to_string(),
            next_run_at: schedule.next_run_at.to_string(),
            hook_kind: hook_kind.to_owned(),
            hook_module: hook_module.to_owned(),
            hook_source: hook_source.to_owned(),
            hook_environment: spec.environment.encode()?,
        })
    }

    fn insert(
        &self,
        transaction: &Transaction<'_>,
        name: &str,
        enabled: bool,
    ) -> Result<i64, CronError> {
        transaction
            .execute(
                "INSERT INTO sc_cron_schedule
                    (generation, name, expr, timeout_ms, jitter_ms, payload,
                     created_at, next_run_at, enabled, hook_kind, hook_module, hook_source, hook_environment)
                 VALUES (1, ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
                params![
                    name,
                    self.expr,
                    self.timeout_ms,
                    self.jitter_ms,
                    self.payload,
                    self.created_at,
                    self.next_run_at,
                    enabled,
                    self.hook_kind,
                    self.hook_module,
                    self.hook_source,
                    self.hook_environment,
                ],
            )
            .map_err(|error| store_error(&format!("failed to insert {TABLE}"), error))?;
        Ok(transaction.last_insert_rowid())
    }

    /// Rewrites row `id` in place: generation plus one, every field from the
    /// schedule, run and end state cleared, and `enabled` as given. A restart
    /// of an ended row and a replace of an active one are this same write.
    fn rewrite(
        &self,
        transaction: &Transaction<'_>,
        id: i64,
        generation: i64,
        enabled: bool,
    ) -> Result<(), CronError> {
        let next = u32::try_from(generation)
            .ok()
            .and_then(Generation::new)
            .and_then(Generation::next)
            .ok_or_else(|| {
                StoreError::new(
                    StoreErrorKind::Other,
                    format!("{TABLE} id {id}: generation exhausted"),
                )
            })?;
        transaction
            .execute(
                "UPDATE sc_cron_schedule SET generation = ?2, expr = ?3, timeout_ms = ?4,
                    jitter_ms = ?5, payload = ?6, created_at = ?7, next_run_at = ?8,
                    last_run_at = NULL, last_outcome = NULL, ended_at = NULL,
                    stop_reason = NULL, enabled = ?9, hook_kind = ?10, hook_module = ?11,
                    hook_source = ?12, hook_environment = ?13
                 WHERE id = ?1",
                params![
                    id,
                    next.get(),
                    self.expr,
                    self.timeout_ms,
                    self.jitter_ms,
                    self.payload,
                    self.created_at,
                    self.next_run_at,
                    enabled,
                    self.hook_kind,
                    self.hook_module,
                    self.hook_source,
                    self.hook_environment,
                ],
            )
            .map_err(|error| store_error(&format!("failed to restart {TABLE} id {id}"), error))?;
        Ok(())
    }
}

fn millis(field: &str, value: Duration) -> Result<i64, CronError> {
    i64::try_from(value.as_millis())
        .map_err(|_overflow| CronError::InvalidSpec(format!("{field} is too large to store")))
}

/// Classifies a rusqlite error so callers can tell busy from corrupt.
fn store_error(context: &str, error: rusqlite::Error) -> CronError {
    let kind = match error.sqlite_error_code() {
        Some(ErrorCode::DatabaseBusy | ErrorCode::DatabaseLocked) => StoreErrorKind::Busy,
        Some(ErrorCode::DatabaseCorrupt | ErrorCode::NotADatabase) => StoreErrorKind::Corrupt,
        Some(
            ErrorCode::CannotOpen
            | ErrorCode::SystemIoFailure
            | ErrorCode::DiskFull
            | ErrorCode::ReadOnly
            | ErrorCode::PermissionDenied,
        ) => StoreErrorKind::Unavailable,
        _ => match error {
            rusqlite::Error::FromSqlConversionFailure(..)
            | rusqlite::Error::InvalidColumnType(..)
            | rusqlite::Error::IntegralValueOutOfRange(..) => StoreErrorKind::Corrupt,
            _ => StoreErrorKind::Other,
        },
    };
    StoreError::new(kind, context).with_source(error).into()
}

fn select_one(connection: &Connection, id: i64) -> Result<Option<Schedule>, CronError> {
    connection
        .query_row(
            &format!("SELECT {COLUMNS} FROM sc_cron_schedule WHERE id = ?1"),
            [id],
            RawRow::read,
        )
        .optional()
        .map_err(|error| store_error(&format!("failed to read {TABLE} id {id}"), error))?
        .map(RawRow::decode)
        .transpose()
}

/// Decodes `last_outcome` text written by [`RunOutcome::to_stored`].
fn decode_outcome(text: &str) -> Option<RunOutcome> {
    RunOutcome::from_stored(text)
}

/// One row as stored, before typed decoding.
struct RawRow {
    id: i64,
    generation: i64,
    name: String,
    expr: String,
    timeout_ms: i64,
    jitter_ms: i64,
    payload: String,
    created_at: String,
    next_run_at: Option<String>,
    last_run_at: Option<String>,
    last_outcome: Option<String>,
    ended_at: Option<String>,
    stop_reason: Option<String>,
    enabled: i64,
    hook_kind: String,
    hook_module: String,
    hook_source: String,
    hook_environment: String,
}

impl RawRow {
    fn read(row: &Row<'_>) -> rusqlite::Result<Self> {
        Ok(Self {
            id: row.get(0)?,
            generation: row.get(1)?,
            name: row.get(2)?,
            expr: row.get(3)?,
            timeout_ms: row.get(4)?,
            jitter_ms: row.get(5)?,
            payload: row.get(6)?,
            created_at: row.get(7)?,
            next_run_at: row.get(8)?,
            last_run_at: row.get(9)?,
            last_outcome: row.get(10)?,
            ended_at: row.get(11)?,
            stop_reason: row.get(12)?,
            enabled: row.get(13)?,
            hook_kind: row.get(14)?,
            hook_module: row.get(15)?,
            hook_source: row.get(16)?,
            hook_environment: row.get(17)?,
        })
    }

    /// Decodes every column; any malformed value is a `Corrupt` error naming the row.
    fn decode(self) -> Result<Schedule, CronError> {
        let id = self.id;
        let corrupt = |what: String| -> CronError {
            StoreError::new(StoreErrorKind::Corrupt, format!("{TABLE} id {id}: {what}")).into()
        };
        let field = |name: &str, error: CronError| corrupt(format!("{name}: {error}"));
        let stamp = |name: &str, text: &str| -> Result<Timestamp, CronError> {
            text.parse().map_err(|error| field(name, error))
        };
        let optional_stamp =
            |name: &str, text: Option<&str>| text.map(|text| stamp(name, text)).transpose();
        let duration = |value: i64, name: &str| {
            u64::try_from(value)
                .map(Duration::from_millis)
                .map_err(|_negative| corrupt(format!("{name} is negative")))
        };
        let payload = serde_json::from_str(&self.payload)
            .map_err(|error| corrupt(format!("payload is not JSON: {error}")))?;
        let last_outcome = self
            .last_outcome
            .as_deref()
            .map(|text| {
                decode_outcome(text).ok_or_else(|| corrupt(format!("unknown outcome {text:?}")))
            })
            .transpose()?;
        let timeout = duration(self.timeout_ms, "timeout_ms")?;
        let schedule = Schedule {
            id: ScheduleId::new(
                u64::try_from(id).map_err(|_negative| corrupt("id is negative".into()))?,
            ),
            generation: u32::try_from(self.generation)
                .ok()
                .and_then(Generation::new)
                .ok_or_else(|| corrupt("generation is out of range".into()))?,
            spec: ScheduleSpec {
                name: ScheduleName::new(self.name).map_err(|error| field("name", error))?,
                expr: CronExpr::parse(&self.expr).map_err(|error| field("expr", error))?,
                timeout: RunTimeout::new(timeout).map_err(|error| field("timeout_ms", error))?,
                payload,
                environment: crate::HookEnvironment::decode(&self.hook_environment)
                    .map_err(|_| corrupt("invalid hook environment".into()))?,
                hook: HookTarget::from_stored(
                    &self.hook_kind,
                    &self.hook_module,
                    &self.hook_source,
                )
                .map_err(|error| corrupt(format!("hook: {error}")))?,
            },
            jitter: duration(self.jitter_ms, "jitter_ms")?,
            created_at: stamp("created_at", &self.created_at)?,
            next_run_at: optional_stamp("next_run_at", self.next_run_at.as_deref())?,
            last_run_at: optional_stamp("last_run_at", self.last_run_at.as_deref())?,
            last_outcome,
            ended_at: optional_stamp("ended_at", self.ended_at.as_deref())?,
            stop_reason: self.stop_reason,
            enabled: match self.enabled {
                0 => false,
                1 => true,
                other => return Err(corrupt(format!("enabled is {other}, not 0 or 1"))),
            },
        };
        schedule
            .spec
            .validate_environment()
            .map_err(|_| corrupt("invalid hook environment pairing".into()))?;
        Ok(schedule)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn outcomes_round_trip_through_their_stored_text() {
        for outcome in [
            RunOutcome::Ok,
            RunOutcome::TimedOut,
            RunOutcome::Refused("quota: full".into()),
            RunOutcome::Failed(String::new()),
        ] {
            assert_eq!(decode_outcome(&outcome.to_string()), Some(outcome));
        }
        assert_eq!(decode_outcome("exit"), None);
    }

    fn kind_of(code: std::ffi::c_int) -> StoreErrorKind {
        let error = rusqlite::Error::SqliteFailure(rusqlite::ffi::Error::new(code), None);
        match store_error("op", error) {
            CronError::Store(store) => store.kind(),
            other => panic!("expected a store error, got {other:?}"),
        }
    }

    #[test]
    fn rusqlite_errors_are_classified_so_busy_is_distinguishable() {
        use rusqlite::ffi;
        assert_eq!(kind_of(ffi::SQLITE_BUSY), StoreErrorKind::Busy);
        assert_eq!(kind_of(ffi::SQLITE_LOCKED), StoreErrorKind::Busy);
        assert_eq!(kind_of(ffi::SQLITE_CORRUPT), StoreErrorKind::Corrupt);
        assert_eq!(kind_of(ffi::SQLITE_NOTADB), StoreErrorKind::Corrupt);
        assert_eq!(kind_of(ffi::SQLITE_FULL), StoreErrorKind::Unavailable);
        assert_eq!(kind_of(ffi::SQLITE_READONLY), StoreErrorKind::Unavailable);
        assert_eq!(kind_of(ffi::SQLITE_CONSTRAINT), StoreErrorKind::Other);
    }

    #[test]
    fn a_finite_cap_above_i64_max_is_compared() {
        let above = u64::try_from(i64::MAX).expect("i64::MAX") + 1;
        let Some(cap) = usize::try_from(above).ok().and_then(NonZeroUsize::new) else {
            // A 32-bit `usize` cannot name a cap above `i64::MAX`. A count
            // that does not fit in `usize` is still at every finite cap.
            let cap = NonZeroUsize::new(1).expect("one");
            assert!(row_count_reached(i64::MAX, cap));
            return;
        };
        assert!(
            !row_count_reached(i64::MAX, cap),
            "i64::MAX rows are under a larger finite cap"
        );
        assert!(
            row_count_reached(-1, cap),
            "a count that does not fit in u64 is not an unlimited bypass"
        );
        let ten = NonZeroUsize::new(10).expect("ten");
        assert!(row_count_reached(10, ten));
        assert!(!row_count_reached(9, ten));
    }

    #[test]
    fn a_store_error_keeps_the_rusqlite_error_as_its_source() {
        use std::error::Error;
        let error = store_error("op", rusqlite::Error::QueryReturnedNoRows);
        let store = error.source().expect("store error");
        assert!(store.source().is_some());
    }
}
