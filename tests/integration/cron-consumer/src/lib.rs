//! Consumer-owned schema example: legacy lifecycle, public store/query/fencing APIs.
use sc_cron::{
    CronError, Generation, NewSchedule, RunOutcome, Schedule, ScheduleId, ScheduleLimits,
    ScheduleName, SchedulePage, ScheduleQuery, ScheduleStore, SqliteScheduleStore, StoreError,
    StoreErrorKind, Timestamp,
};
use std::path::Path;
use std::sync::Mutex;

/// Adds a durable application tenant field without extending core Schedule/DTOs.
///
/// This provider deliberately uses legacy, caller-owned lifecycle: application metadata
/// uses a second application-owned database and is not in the bundled store transaction.
/// It does not advertise cooperative scope or cross-database atomicity.
pub struct ApplicationStore {
    schedules: SqliteScheduleStore,
    application: Mutex<rusqlite::Connection>,
    tenant: String,
}
impl ApplicationStore {
    /// Opens the bundled schedules and independent application-owned schema.
    pub fn open(schedules: &Path, application: &Path, tenant: &str) -> Result<Self, CronError> {
        let application = rusqlite::Connection::open(application).map_err(application_error)?;
        application
            .execute_batch(
                "CREATE TABLE IF NOT EXISTS application_schedule (
            schedule_id TEXT PRIMARY KEY, generation INTEGER NOT NULL, tenant TEXT NOT NULL
        )",
            )
            .map_err(application_error)?;
        Ok(Self {
            schedules: SqliteScheduleStore::open(schedules, ScheduleLimits::default())?,
            application: Mutex::new(application),
            tenant: tenant.to_owned(),
        })
    }
    fn remember(&self, row: &Schedule) -> Result<(), CronError> {
        self.application.lock().unwrap_or_else(std::sync::PoisonError::into_inner)
            .execute("INSERT INTO application_schedule VALUES (?1, ?2, ?3)
                ON CONFLICT(schedule_id) DO UPDATE SET generation=excluded.generation, tenant=excluded.tenant",
                rusqlite::params![row.id.to_string(), row.generation.get(), self.tenant]).map_err(application_error)?;
        Ok(())
    }
    /// Reads the application field for the exact persisted schedule generation.
    pub fn tenant(&self, id: ScheduleId, generation: Generation) -> Result<String, CronError> {
        self.application
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .query_row(
                "SELECT tenant FROM application_schedule WHERE schedule_id=?1 AND generation=?2",
                rusqlite::params![id.to_string(), generation.get()],
                |row| row.get(0),
            )
            .map_err(application_error)
    }
}
fn application_error(error: rusqlite::Error) -> CronError {
    StoreError::new(StoreErrorKind::Other, "application_metadata_failed")
        .with_source(error)
        .into()
}
impl ScheduleStore for ApplicationStore {
    // Deliberately inherit Legacy + unsupported cooperative scope defaults.
    fn query(&self, query: &ScheduleQuery) -> Result<SchedulePage, CronError> {
        self.schedules.query(query)
    }
    fn create(&self, schedule: &NewSchedule, at: Timestamp) -> Result<Schedule, CronError> {
        let row = self.schedules.create(schedule, at)?;
        self.remember(&row)?;
        Ok(row)
    }
    fn get(&self, id: ScheduleId) -> Result<Option<Schedule>, CronError> {
        self.schedules.get(id)
    }
    fn list_active(&self) -> Result<Vec<Schedule>, CronError> {
        self.schedules.list_active()
    }
    fn record_run(
        &self,
        id: ScheduleId,
        generation: Generation,
        at: Timestamp,
        outcome: &RunOutcome,
    ) -> Result<bool, CronError> {
        self.schedules.record_run(id, generation, at, outcome)
    }
    fn set_next(
        &self,
        id: ScheduleId,
        generation: Generation,
        next: Timestamp,
    ) -> Result<bool, CronError> {
        self.schedules.set_next(id, generation, next)
    }
    fn stop(
        &self,
        id: ScheduleId,
        generation: Generation,
        at: Timestamp,
        reason: &str,
    ) -> Result<bool, CronError> {
        self.schedules.stop(id, generation, at, reason)
    }
    fn set_enabled(&self, name: &ScheduleName, enabled: bool) -> Result<bool, CronError> {
        self.schedules.set_enabled(name, enabled)
    }
    fn disable(
        &self,
        id: ScheduleId,
        generation: Generation,
        at: Timestamp,
        reason: &str,
    ) -> Result<bool, CronError> {
        self.schedules.disable(id, generation, at, reason)
    }
    fn set_enabled_many(&self, flags: &[(ScheduleName, bool)]) -> Result<bool, CronError> {
        self.schedules.set_enabled_many(flags)
    }
    fn replace_and_set_enabled(
        &self,
        schedule: &NewSchedule,
        enabled: bool,
        at: Timestamp,
        flags: &[(ScheduleName, bool)],
    ) -> Result<Option<Schedule>, CronError> {
        let row = self
            .schedules
            .replace_and_set_enabled(schedule, enabled, at, flags)?;
        if let Some(row) = &row {
            self.remember(row)?;
        }
        Ok(row)
    }
}

/// A cooperative forwarding provider with no additional unscoped persistence.
pub struct CooperativeForwarder(pub SqliteScheduleStore);
impl ScheduleStore for CooperativeForwarder {
    fn operation_support(&self) -> sc_cron::StoreOperationSupport {
        self.0.operation_support()
    }
    fn with_operation_context<T>(
        &self,
        context: &sc_cron::StoreOperationContext,
        operation: impl FnOnce(&Self) -> Result<T, CronError>,
    ) -> Result<T, CronError> {
        self.0.with_operation_context(context, |_| operation(self))
    }
    fn query(&self, query: &ScheduleQuery) -> Result<SchedulePage, CronError> {
        self.0.query(query)
    }
    fn create(&self, schedule: &NewSchedule, at: Timestamp) -> Result<Schedule, CronError> {
        self.0.create(schedule, at)
    }
    fn get(&self, id: ScheduleId) -> Result<Option<Schedule>, CronError> {
        self.0.get(id)
    }
    fn list_active(&self) -> Result<Vec<Schedule>, CronError> {
        self.0.list_active()
    }
    fn record_run(
        &self,
        id: ScheduleId,
        generation: Generation,
        at: Timestamp,
        outcome: &RunOutcome,
    ) -> Result<bool, CronError> {
        self.0.record_run(id, generation, at, outcome)
    }
    fn set_next(
        &self,
        id: ScheduleId,
        generation: Generation,
        next: Timestamp,
    ) -> Result<bool, CronError> {
        self.0.set_next(id, generation, next)
    }
    fn stop(
        &self,
        id: ScheduleId,
        generation: Generation,
        at: Timestamp,
        reason: &str,
    ) -> Result<bool, CronError> {
        self.0.stop(id, generation, at, reason)
    }
    fn set_enabled(&self, name: &ScheduleName, enabled: bool) -> Result<bool, CronError> {
        self.0.set_enabled(name, enabled)
    }
    fn disable(
        &self,
        id: ScheduleId,
        generation: Generation,
        at: Timestamp,
        reason: &str,
    ) -> Result<bool, CronError> {
        self.0.disable(id, generation, at, reason)
    }
    fn set_enabled_many(&self, flags: &[(ScheduleName, bool)]) -> Result<bool, CronError> {
        self.0.set_enabled_many(flags)
    }
    fn replace_and_set_enabled(
        &self,
        schedule: &NewSchedule,
        enabled: bool,
        at: Timestamp,
        flags: &[(ScheduleName, bool)],
    ) -> Result<Option<Schedule>, CronError> {
        self.0.replace_and_set_enabled(schedule, enabled, at, flags)
    }
}
