//! Semantic first-release table checks, independent of SQL spelling/order.
use crate::{CronError, StoreError, StoreErrorKind};
use rusqlite::{Connection, OptionalExtension};

fn mismatch() -> CronError {
    StoreError::new(StoreErrorKind::SchemaMismatch, "incompatible first-release sc-cron table; recreate this development database after preserving needed data").into()
}

pub(super) fn validate(connection: &Connection) -> Result<(), CronError> {
    if !columns_compatible(connection).map_err(|_| mismatch())?
        || !unique_name_index(connection).map_err(|_| mismatch())?
    {
        return Err(mismatch());
    }
    // Probe constraints inside a savepoint. Rollback restores both table data
    // and AUTOINCREMENT state, including when any probe unexpectedly succeeds.
    connection
        .execute_batch("SAVEPOINT sc_cron_schema_probe")
        .map_err(|_| mismatch())?;
    let result = probe_constraints(connection);
    let rollback =
        connection.execute_batch("ROLLBACK TO sc_cron_schema_probe; RELEASE sc_cron_schema_probe");
    if result.is_err() || rollback.is_err() {
        return Err(mismatch());
    }
    Ok(())
}

const REQUIRED_COLUMNS: &[(&str, &str, bool, Option<&str>)] = &[
    ("id", "INTEGER", false, None),
    ("generation", "INTEGER", true, None),
    ("name", "TEXT", true, None),
    ("expr", "TEXT", true, None),
    ("timeout_ms", "INTEGER", true, None),
    ("jitter_ms", "INTEGER", true, None),
    ("payload", "TEXT", true, None),
    ("created_at", "TEXT", true, None),
    ("next_run_at", "TEXT", false, None),
    ("last_run_at", "TEXT", false, None),
    ("last_outcome", "TEXT", false, None),
    ("ended_at", "TEXT", false, None),
    ("stop_reason", "TEXT", false, None),
    ("enabled", "INTEGER", true, Some("1")),
    ("hook_kind", "TEXT", true, Some("rust")),
    ("hook_module", "TEXT", true, Some("")),
    ("hook_source", "TEXT", true, Some("")),
    (
        "hook_environment",
        "TEXT",
        true,
        Some("{\"dotenv\":[],\"variables\":{}}"),
    ),
];

fn columns_compatible(connection: &Connection) -> rusqlite::Result<bool> {
    let mut statement = connection.prepare("PRAGMA table_info(sc_cron_schedule)")?;
    let columns = statement
        .query_map([], |row| {
            Ok((
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, bool>(3)?,
                row.get::<_, Option<String>>(4)?,
                row.get::<_, i64>(5)?,
            ))
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    for (name, ty, required, default) in REQUIRED_COLUMNS.iter().copied() {
        let Some(column) = columns.iter().find(|c| c.0 == name) else {
            return Ok(false);
        };
        if !column.1.eq_ignore_ascii_case(ty)
            || (required && !column.2)
            || (name == "id" && column.4 != 1)
        {
            return Ok(false);
        }
        if let Some(default) = default {
            let Some(expression) = column.3.as_deref() else {
                return Ok(false);
            };
            let actual: String =
                connection.query_row(&format!("SELECT CAST(({expression}) AS TEXT)"), [], |r| {
                    r.get(0)
                })?;
            if actual != default {
                return Ok(false);
            }
        }
    }
    Ok(true)
}
fn unique_name_index(connection: &Connection) -> rusqlite::Result<bool> {
    let mut indexes = connection.prepare("SELECT name FROM pragma_index_list('sc_cron_schedule') WHERE \"unique\" = 1 AND partial = 0")?;
    let names = indexes
        .query_map([], |r| r.get::<_, String>(0))?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    for name in names {
        let mut statement =
            connection.prepare("SELECT name FROM pragma_index_info(?1) ORDER BY seqno")?;
        let keys = statement
            .query_map([name], |r| r.get::<_, String>(0))?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        if keys == ["name"] {
            return Ok(true);
        }
    }
    Ok(false)
}

fn probe_constraints(connection: &Connection) -> Result<(), CronError> {
    let mut name = "__sc_cron_schema_probe".to_owned();
    while connection
        .query_row(
            "SELECT 1 FROM sc_cron_schedule WHERE name = ?1",
            [&name],
            |_| Ok(()),
        )
        .optional()
        .map_err(|_| mismatch())?
        .is_some()
    {
        name.push('_');
        if name.len() > crate::MAX_NAME_LEN {
            return Err(mismatch());
        }
    }
    connection.execute("INSERT INTO sc_cron_schedule (generation,name,expr,timeout_ms,jitter_ms,payload,created_at,next_run_at) VALUES (1,?1,'* * * * *',1000,0,'{}','2026-01-01T00:00:00Z','2026-01-01T00:01:00Z')", [&name]).map_err(|_| mismatch())?;
    let id = connection.last_insert_rowid();
    let bad = [
        "generation = 0",
        "generation = 4294967296",
        "name = ''",
        "name = replace(hex(zeroblob(65)), '0', 'x')",
        "expr = hex(zeroblob(129))",
        "timeout_ms = 0",
        "jitter_ms = -1",
        "payload = hex(zeroblob(32769))",
        "enabled = 2",
        "hook_kind = 'invalid'",
        "hook_source = hex(zeroblob(32769))",
        "hook_environment = hex(zeroblob(131073))",
        "next_run_at = NULL",
        "stop_reason = 'unexpected'",
    ];
    for update in bad {
        if connection
            .execute(
                &format!("UPDATE sc_cron_schedule SET {update} WHERE id = ?1"),
                [id],
            )
            .is_ok()
        {
            return Err(mismatch());
        }
    }
    Ok(())
}
