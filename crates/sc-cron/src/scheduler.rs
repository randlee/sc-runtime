//! The scheduler: starts and stops schedules and spawns the worker that runs them.
//!
//! [`Scheduler::start`] checks the payload size and that the expression has
//! a future occurrence before anything is written (name, expression and
//! timeout are already valid by type), draws the schedule's jitter, computes
//! its first run and stores it. [`Scheduler::spawn_worker`] starts the one
//! tokio task that ticks every active schedule. `start` and `stop` signal it
//! to reload, so a new schedule ticks without a restart.
//!
//! `start` and `stop` call the synchronous store directly. From async code
//! use [`Scheduler::start_async`] and [`Scheduler::stop_async`], which run
//! the same calls on tokio's blocking pool so a busy database never blocks
//! an executor thread.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::time::Duration;

use chrono::{DateTime, Utc};
use tokio::sync::{oneshot, watch};
use tokio::task::JoinHandle;

use crate::coord::RunCoord;
use crate::limits::{ScheduleLimits, encode_payload};
use crate::sqlite::SqliteScheduleStore;
use crate::worker::{DEFAULT_RELOAD_POLL, EXHAUSTED, MIN_RELOAD_POLL, Report, Worker};
use crate::{
    Clock, CronError, Generation, JitterSource, Job, NewSchedule, RunOutcome, Schedule, ScheduleId,
    ScheduleName, ScheduleSpec, ScheduleStore, StoreError, StoreErrorKind, Timestamp, jitter,
};
use sc_observability_types::{Level, SharedCanonicalLog};

mod run_due;

use run_due::record_due;

/// What the worker does with occurrences that came due while it could not run them.
///
/// Occurrences are missed while the worker is not running (process down,
/// before [`Scheduler::spawn_worker`]) or cannot wake (host suspended). An
/// occurrence counts as missed once the occurrence after it is also due.
/// A run that is merely late, with the following occurrence still in the
/// future, always runs. Either way the schedule then continues from its next
/// future occurrence, and a backlog is never run one by one.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum MissedTickPolicy {
    /// Replay all missed occurrences as one catch-up run as soon as the worker sees them.
    #[default]
    Coalesce,
    /// Drop missed occurrences; run again at the next future occurrence.
    Skip,
}

/// What `start`, `stop` and their async forms share, so the async forms can
/// move it onto the blocking pool.
struct Core<S, C, J> {
    store: Arc<S>,
    clock: Arc<C>,
    jitter: Arc<J>,
    reload: watch::Sender<u64>,
}

impl<S: ScheduleStore, C: Clock, J: JitterSource> Core<S, C, J> {
    /// The row `spec` describes, as of the clock's now: the payload checked,
    /// a future occurrence required, the jitter drawn and the next run
    /// computed. `start` and `replace_and_set_enabled` share it.
    fn new_schedule(&self, spec: ScheduleSpec) -> Result<(NewSchedule, DateTime<Utc>), CronError> {
        encode_payload(&spec.payload)?;
        let now = self.clock.now();
        spec.expr.require_future(now)?;
        let jitter = jitter::draw(&spec.expr, now, self.jitter.as_ref());
        let next = spec.expr.next_run(jitter, now).ok_or_else(|| {
            CronError::BadExpression("expression has no future occurrence".into())
        })?;
        let schedule = NewSchedule {
            spec,
            jitter,
            next_run_at: Timestamp::from_datetime(next),
        };
        Ok((schedule, now))
    }

    fn start(&self, spec: ScheduleSpec) -> Result<Schedule, CronError> {
        let (schedule, now) = self.new_schedule(spec)?;
        let created = self
            .store
            .create(&schedule, Timestamp::from_datetime(now))?;
        self.signal_reload();
        Ok(created)
    }

    fn stop(&self, id: ScheduleId, generation: Generation, reason: &str) -> Result<(), CronError> {
        let at = Timestamp::from_datetime(self.clock.now());
        if !self.store.stop(id, generation, at, reason)? {
            return Err(CronError::Fenced { id, generation });
        }
        self.signal_reload();
        Ok(())
    }

    fn set_enabled(&self, name: &ScheduleName, enabled: bool) -> Result<(), CronError> {
        if !self.store.set_enabled(name, enabled)? {
            return Err(CronError::NotFound(name.clone()));
        }
        self.signal_reload();
        Ok(())
    }

    fn set_enabled_many(&self, flags: &[(ScheduleName, bool)]) -> Result<(), CronError> {
        if !self.store.set_enabled_many(flags)? {
            return Err(self.first_missing(flags.iter().map(|(name, _)| name))?);
        }
        self.signal_reload();
        Ok(())
    }

    fn replace_and_set_enabled(
        &self,
        spec: ScheduleSpec,
        enabled: bool,
        flags: &[(ScheduleName, bool)],
    ) -> Result<Schedule, CronError> {
        let (schedule, now) = self.new_schedule(spec)?;
        let Some(written) = self.store.replace_and_set_enabled(
            &schedule,
            enabled,
            Timestamp::from_datetime(now),
            flags,
        )?
        else {
            return Err(self.first_missing(flags.iter().map(|(name, _)| name))?);
        };
        self.signal_reload();
        Ok(written)
    }

    /// The invalid-spec error for the first of `names` with no active schedule, after
    /// a store call refused the list for one; a name gone by the time of this
    /// read is reported as the first name.
    fn first_missing<'n>(
        &self,
        mut names: impl Iterator<Item = &'n ScheduleName> + Clone,
    ) -> Result<CronError, CronError> {
        let active = self.store.list_active()?;
        let first = names.clone().next().cloned();
        let missing = names
            .find(|name| !active.iter().any(|s| s.spec.name == **name))
            .cloned()
            .or(first);
        Ok(match missing {
            Some(name) => CronError::NotFound(name),
            None => CronError::InvalidSpec("an empty list of schedules changed nothing".into()),
        })
    }

    fn signal_reload(&self) {
        self.reload
            .send_modify(|version| *version = version.wrapping_add(1));
    }
}

/// Starts, stops and runs schedules kept in a [`ScheduleStore`].
///
/// # Examples
///
/// ```
/// use std::sync::Arc;
/// use std::time::Duration;
///
/// use sc_cron::{
///     CronError, ScheduleLimits, ScheduleSpec, ScheduleStore, Scheduler, SqliteScheduleStore,
///     SystemClock, SystemJitter,
/// };
///
/// let limits = ScheduleLimits::default();
/// let scheduler = Scheduler::new(
///     Arc::new(SqliteScheduleStore::open_in_memory(limits)?),
///     Arc::new(SystemClock),
///     Arc::new(SystemJitter),
/// );
/// let spec = ScheduleSpec::new("sync", "*/30 * * * * *", Duration::from_secs(25), serde_json::json!({}))?;
/// let schedule = scheduler.start(spec.clone())?;
/// assert!(schedule.is_active());
///
/// // A second start of an active name is refused and stores nothing.
/// assert!(matches!(scheduler.start(spec), Err(CronError::AlreadyActive(_))));
/// assert_eq!(scheduler.store().list_active()?.len(), 1);
/// # Ok::<(), CronError>(())
/// ```
pub struct Scheduler<S: ScheduleStore, C: Clock, J: JitterSource> {
    core: Arc<Core<S, C, J>>,
    policy: MissedTickPolicy,
    reload_poll: Duration,
    worker_live: Arc<AtomicBool>,
    coord: Arc<RunCoord>,
    #[cfg(feature = "monty")]
    hosts: Arc<crate::MontyHostRegistry>,
}

impl<S, C, J> Scheduler<S, C, J>
where
    S: ScheduleStore,
    C: Clock,
    J: JitterSource,
{
    /// Returns another handle to the same store, clock, worker, and coordination state.
    #[must_use]
    pub fn share(&self) -> Self {
        Self::from_parts(
            Arc::clone(&self.core),
            self.policy,
            self.reload_poll,
            Arc::clone(&self.worker_live),
            Arc::clone(&self.coord),
            #[cfg(feature = "monty")]
            Arc::clone(&self.hosts),
        )
    }

    /// Subscribes to how many Monty feeds are still inside a run.
    ///
    /// The count changes when a feed starts and when it leaves. A test waits
    /// on it instead of sleeping while the interpreter runs.
    #[must_use]
    pub fn subscribe_monty_feeds(&self) -> watch::Receiver<usize> {
        self.coord.subscribe_feed_level()
    }

    /// Waits until a shutdown deadline has elapsed during a Monty feed more
    /// than `above` times.
    ///
    /// Returns false when the scheduler drops the watch.
    pub async fn wait_monty_shutdown_deadlines(&self, above: u64) -> bool {
        let mut seen = self.coord.subscribe_shutdown_deadlines();
        seen.wait_for(|value| *value > above).await.is_ok()
    }

    fn from_parts(
        core: Arc<Core<S, C, J>>,
        policy: MissedTickPolicy,
        reload_poll: Duration,
        worker_live: Arc<AtomicBool>,
        coord: Arc<RunCoord>,
        #[cfg(feature = "monty")] hosts: Arc<crate::MontyHostRegistry>,
    ) -> Self {
        Self {
            core,
            policy,
            reload_poll,
            worker_live,
            coord,
            #[cfg(feature = "monty")]
            hosts,
        }
    }
}

impl<S, C, J> Scheduler<S, C, J>
where
    S: ScheduleStore + 'static,
    C: Clock + 'static,
    J: JitterSource + 'static,
{
    /// Creates a scheduler over `store`, reading time from `clock`.
    ///
    /// The row ceiling is [`ScheduleStore::limits`]. The store enforces it
    /// inside the insert transaction. This constructor does not take a
    /// second copy. [`Scheduler::open`] is the production path: it opens
    /// the file with the embedder's [`ScheduleLimits`].
    /// [`ScheduleLimits::default`] does not cap the table. The missed-tick
    /// policy is [`MissedTickPolicy::Coalesce`].
    #[must_use]
    pub fn new(store: Arc<S>, clock: Arc<C>, jitter: Arc<J>) -> Self {
        Self::new_with_logger(store, clock, jitter, crate::noop_logger())
    }

    /// Creates a scheduler with a shared, best-effort canonical logger.
    ///
    /// The supplied store, clock and jitter remain unchanged and own their
    /// internal logging. Application code owns logger flush and shutdown.
    #[must_use]
    pub fn new_with_logger(
        store: Arc<S>,
        clock: Arc<C>,
        jitter: Arc<J>,
        logger: SharedCanonicalLog,
    ) -> Self {
        if let Some(event) = crate::component_event(
            logger.service_name().clone(),
            "cron.scheduler",
            "logger.attached",
            Level::Info,
            "success",
            None,
            serde_json::Map::new(),
        ) {
            let _ = logger.emit(event);
        }
        Self {
            core: Arc::new(Core {
                store,
                clock,
                jitter,
                reload: watch::channel(0).0,
            }),
            policy: MissedTickPolicy::default(),
            reload_poll: DEFAULT_RELOAD_POLL,
            worker_live: Arc::new(AtomicBool::new(false)),
            coord: Arc::new(RunCoord::with_logger(logger)),
            #[cfg(feature = "monty")]
            hosts: Arc::new(crate::MontyHostRegistry::new()),
        }
    }

    /// Returns the row ceiling the store enforces.
    ///
    /// This is [`ScheduleStore::limits`]. The scheduler does not keep another
    /// copy.
    pub fn limits(&self) -> ScheduleLimits {
        self.core.store.limits()
    }

    /// Sets the missed-tick policy for workers spawned after this call.
    #[must_use]
    pub fn with_missed_tick_policy(mut self, policy: MissedTickPolicy) -> Self {
        self.policy = policy;
        self
    }

    /// Sets the longest a worker spawned after this call goes between
    /// reloads of the active schedules (REQ-CRON-0014).
    ///
    /// A schedule started, restarted, enabled, disabled or stopped by another
    /// process or scheduler sends this scheduler's worker no signal, so the
    /// worker reloads at least once per `interval` on the scheduler's clock,
    /// whatever the rows are. Such a change is therefore noticed at most
    /// `interval` after it is stored (a reload that fails is retried with a
    /// backoff, so the bound holds while the store can be read). A disable or
    /// stop is also noticed earlier, at the schedule's next tick. Each reload
    /// is one `list_active` query, so the poll costs one query per interval.
    /// The default is [`DEFAULT_RELOAD_POLL`] (30 s); an interval under one
    /// second is raised to one second.
    #[must_use]
    pub fn with_reload_poll(mut self, interval: Duration) -> Self {
        self.reload_poll = interval.max(MIN_RELOAD_POLL);
        self
    }

    /// Installs one immutable registry for subsequent Monty feeds.
    ///
    /// The default registry is empty; CLI adapters and application handlers
    /// are registered explicitly by the embedding application.
    #[cfg(feature = "monty")]
    #[must_use]
    pub fn with_monty_hosts(mut self, registry: crate::MontyHostRegistry) -> Self {
        self.hosts = Arc::new(registry);
        self
    }

    /// Returns the store, for reading schedules and their run state.
    #[must_use]
    pub fn store(&self) -> &Arc<S> {
        &self.core.store
    }

    /// Stores a new schedule, checking it before any write.
    ///
    /// Restarting an ended name keeps its id and raises its generation, so
    /// any run still in flight from the old generation is fenced. This
    /// calls the store synchronously; from async code use
    /// [`Scheduler::start_async`].
    ///
    /// # Errors
    ///
    /// [`CronError::InvalidSpec`] for a payload over
    /// [`MAX_PAYLOAD_BYTES`](crate::MAX_PAYLOAD_BYTES), or when a new name
    /// would pass the embedder's schedule row limit, and
    /// [`CronError::BadExpression`] for an expression with no future
    /// occurrence, both before anything is stored; then the store's errors,
    /// including [`CronError::AlreadyActive`]. The row-limit refusal names
    /// the limit and the current count.
    pub fn start(&self, spec: ScheduleSpec) -> Result<Schedule, CronError> {
        self.core.start(spec)
    }

    /// [`Scheduler::start`] on tokio's blocking pool.
    ///
    /// # Errors
    ///
    /// As [`Scheduler::start`], or [`CronError::Store`] if the blocking task
    /// panics.
    pub async fn start_async(&self, spec: ScheduleSpec) -> Result<Schedule, CronError> {
        let core = Arc::clone(&self.core);
        blocking(move || core.start(spec)).await
    }

    /// Ends schedule `(id, generation)` with `reason`; its ticks stop.
    ///
    /// A run already in flight is cancelled and records nothing. This calls
    /// the store synchronously; from async code use [`Scheduler::stop_async`].
    ///
    /// # Errors
    ///
    /// [`CronError::Fenced`] if `(id, generation)` is not the active
    /// schedule, or the store's errors.
    pub fn stop(
        &self,
        id: ScheduleId,
        generation: Generation,
        reason: &str,
    ) -> Result<(), CronError> {
        self.core.stop(id, generation, reason)
    }

    /// [`Scheduler::stop`] on tokio's blocking pool.
    ///
    /// # Errors
    ///
    /// As [`Scheduler::stop`], or [`CronError::Store`] if the blocking task
    /// panics.
    pub async fn stop_async(
        &self,
        id: ScheduleId,
        generation: Generation,
        reason: &str,
    ) -> Result<(), CronError> {
        let core = Arc::clone(&self.core);
        let reason = reason.to_owned();
        blocking(move || core.stop(id, generation, &reason)).await
    }

    /// Enables or disables the active schedule `name`; only its `enabled`
    /// flag changes (REQ-CRON-0014).
    ///
    /// A disabled schedule never fires. It keeps its id, generation,
    /// expression, payload and stored next run, so enabling it again resumes
    /// it: a run that came due while it was disabled counts as missed and is
    /// handled per the [`MissedTickPolicy`]. A run in flight when it is
    /// disabled is cancelled and records nothing, as with
    /// [`Scheduler::stop`]. This scheduler's worker is told at once. A worker
    /// in another process or on another scheduler is not told: it notices the
    /// change at its next reload, at most [`Scheduler::with_reload_poll`]
    /// (default 30 s) after the write, or for a disable when its next tick
    /// finds the schedule disabled, if that is sooner. This calls the store synchronously; from
    /// async code use [`Scheduler::set_enabled_async`].
    ///
    /// # Errors
    ///
    /// [`CronError::NotFound`] if no active schedule has this name, or the
    /// store's errors.
    pub fn set_enabled(&self, name: &ScheduleName, enabled: bool) -> Result<(), CronError> {
        self.core.set_enabled(name, enabled)
    }

    /// [`Scheduler::set_enabled`] on tokio's blocking pool.
    ///
    /// # Errors
    ///
    /// As [`Scheduler::set_enabled`], or [`CronError::Store`] if the
    /// blocking task panics.
    pub async fn set_enabled_async(
        &self,
        name: &ScheduleName,
        enabled: bool,
    ) -> Result<(), CronError> {
        let core = Arc::clone(&self.core);
        let name = name.clone();
        blocking(move || core.set_enabled(&name, enabled)).await
    }

    /// Sets the `enabled` flag of every named active schedule in one store
    /// transaction (REQ-CRON-0017), and tells this scheduler's worker as
    /// [`Scheduler::set_enabled`] does (REQ-CRON-0014).
    ///
    /// Another connection sees every flag of the list change at once, or
    /// none. A worker in another process notices at its next reload. This
    /// calls the store synchronously; from async code use
    /// [`Scheduler::set_enabled_many_async`].
    ///
    /// # Errors
    ///
    /// [`CronError::NotFound`] naming a listed schedule that is not active,
    /// with nothing changed, or the store's errors.
    pub fn set_enabled_many(&self, flags: &[(ScheduleName, bool)]) -> Result<(), CronError> {
        self.core.set_enabled_many(flags)
    }

    /// [`Scheduler::set_enabled_many`] on tokio's blocking pool.
    ///
    /// # Errors
    ///
    /// As [`Scheduler::set_enabled_many`], or [`CronError::Store`] if the
    /// blocking task panics.
    pub async fn set_enabled_many_async(
        &self,
        flags: Vec<(ScheduleName, bool)>,
    ) -> Result<(), CronError> {
        let core = Arc::clone(&self.core);
        blocking(move || core.set_enabled_many(&flags)).await
    }

    /// Writes the schedule `spec` with its `enabled` flag and sets each
    /// listed other schedule's flag, in one store transaction (REQ-CRON-0017).
    ///
    /// The jitter, the next run and the creation time come from the injected
    /// sources, as [`Scheduler::start`] does, and the same checks run before
    /// anything is written. A new name is created; an ended name is restarted
    /// in place; an active name is replaced in place with its generation
    /// raised, so a run of the old generation is fenced. The worker is told as
    /// [`Scheduler::set_enabled`] does. This lets a caller keep "at most one
    /// of these schedules enabled" across a create or an update, which
    /// [`Scheduler::start`] (always enabled, `AlreadyActive` for an active
    /// name) cannot. This calls the store synchronously; from async code use
    /// [`Scheduler::replace_and_set_enabled_async`].
    ///
    /// # Errors
    ///
    /// As [`Scheduler::start`] for the spec; [`CronError::InvalidSpec`] if
    /// `flags` names `spec`'s schedule; [`CronError::NotFound`] naming a
    /// listed schedule that is not active, with nothing written; or the
    /// store's errors.
    pub fn replace_and_set_enabled(
        &self,
        spec: ScheduleSpec,
        enabled: bool,
        flags: &[(ScheduleName, bool)],
    ) -> Result<Schedule, CronError> {
        self.core.replace_and_set_enabled(spec, enabled, flags)
    }

    /// [`Scheduler::replace_and_set_enabled`] on tokio's blocking pool.
    ///
    /// # Errors
    ///
    /// As [`Scheduler::replace_and_set_enabled`], or [`CronError::Store`] if
    /// the blocking task panics.
    pub async fn replace_and_set_enabled_async(
        &self,
        spec: ScheduleSpec,
        enabled: bool,
        flags: Vec<(ScheduleName, bool)>,
    ) -> Result<Schedule, CronError> {
        let core = Arc::clone(&self.core);
        blocking(move || core.replace_and_set_enabled(spec, enabled, &flags)).await
    }

    /// Spawns the worker that runs each active schedule's stored hook.
    ///
    /// A [`HookTarget::Rust`](crate::HookTarget::Rust) row calls `job`. A
    /// [`HookTarget::Python`](crate::HookTarget::Python) row runs
    /// `python3 <module> cron.stop` and, when that continues,
    /// `python3 <module> cron.tick`, with one JSON object on stdin. The
    /// worker loads the active schedules from the store, then sleeps on the
    /// clock until the next run, run deadline or reload signal. A failed
    /// load is retried with a backoff of 1 s doubling to 60 s. It also
    /// reloads at least once per [`Scheduler::with_reload_poll`] (default
    /// 30 s), so a schedule started, restarted, enabled or stopped by another
    /// process is noticed within that interval.
    ///
    /// Stop the worker with [`WorkerHandle::shutdown`] and await it: that is
    /// the only way to know in-flight outcomes were recorded before the
    /// process exits. Dropping the handle only signals the worker, which
    /// then drains detached.
    ///
    /// Run timeouts cancel a run at its next `.await`. A job that blocks a
    /// runtime thread without awaiting cannot be cancelled, and on a
    /// current-thread runtime it would also stall the worker; use a
    /// multi-thread runtime and `spawn_blocking` for blocking work (see
    /// [`Job`]).
    ///
    /// # Panics
    ///
    /// Panics outside a tokio runtime, or if a worker from this scheduler is
    /// still running: two workers would run every schedule twice.
    #[must_use]
    pub fn spawn_worker(&self, job: Arc<dyn Job>) -> WorkerHandle {
        let claimed = self
            .worker_live
            .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
            .is_ok();
        assert!(
            claimed,
            "sc-cron: a worker for this scheduler is already running"
        );
        let live = LiveGuard(Arc::clone(&self.worker_live));
        let reload = self.core.reload.subscribe();
        let (shutdown, shutdown_rx) = oneshot::channel();
        let worker = Worker::new(
            Arc::clone(&self.core.store),
            Arc::clone(&self.core.clock),
            job,
            self.policy,
            Arc::clone(&self.coord),
            self.reload_poll,
            shutdown_rx,
            #[cfg(feature = "monty")]
            Arc::clone(&self.hosts),
        );
        let task = tokio::spawn(async move {
            let _live = live;
            worker.run(reload).await;
        });
        WorkerHandle {
            shutdown: Some(shutdown),
            task: Some(task),
            feeds: Arc::clone(&self.coord.feeds),
            logger: Arc::clone(&self.coord.logger),
        }
    }
}

impl<C, J> Scheduler<SqliteScheduleStore, C, J>
where
    C: Clock + 'static,
    J: JitterSource + 'static,
{
    /// Opens the schedule database at `path` and builds a scheduler.
    ///
    /// This is the production path. `limits` is stored on the
    /// [`SqliteScheduleStore`], which enforces it in the insert transaction.
    /// [`Scheduler::new`] reads that ceiling from the store and does not
    /// keep a second copy. This call does not compare `limits` with another
    /// writer's ceiling. Every writer of one schedule database must pass
    /// the same [`ScheduleLimits`]; the ceiling is not stored in the file.
    /// [`ScheduleLimits::default`] does not cap the table. The file is not
    /// the application's store-sqlite file.
    ///
    /// # Errors
    ///
    /// [`CronError::Store`] when the file cannot be opened or the schema does
    /// not match, as [`SqliteScheduleStore::open`]. [`CronError::InvalidSpec`]
    /// when `limits` does not match [`SqliteScheduleStore::limits`] on the
    /// store this call opened. That comparison is this process's store, not
    /// another writer's ceiling.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::num::NonZeroUsize;
    /// use std::sync::Arc;
    ///
    /// use sc_cron::{ScheduleLimits, Scheduler, SystemClock, SystemJitter};
    ///
    /// let dir = tempfile::tempdir().expect("tempdir");
    /// let limits = ScheduleLimits {
    ///     max_schedule_rows: NonZeroUsize::new(10),
    /// };
    /// let scheduler = Scheduler::open(
    ///     dir.path().join("cron.db"),
    ///     limits,
    ///     Arc::new(SystemClock),
    ///     Arc::new(SystemJitter),
    /// )?;
    /// assert_eq!(scheduler.limits(), limits);
    /// # Ok::<(), sc_cron::CronError>(())
    /// ```
    pub fn open(
        path: impl AsRef<std::path::Path>,
        limits: ScheduleLimits,
        clock: Arc<C>,
        jitter: Arc<J>,
    ) -> Result<Self, CronError> {
        let store = SqliteScheduleStore::open(path, limits)?;
        Self::from_opened_store(store, limits, clock, jitter)
    }

    /// Builds a scheduler after `store` was opened with `limits`.
    ///
    /// Compares `limits` with [`SqliteScheduleStore::limits`]. A mismatch is
    /// [`CronError::InvalidSpec`]. The ceiling is not read from the file.
    fn from_opened_store(
        store: SqliteScheduleStore,
        limits: ScheduleLimits,
        clock: Arc<C>,
        jitter: Arc<J>,
    ) -> Result<Self, CronError> {
        let stored = store.limits();
        if stored != limits {
            return Err(CronError::InvalidSpec(format!(
                "passed max_schedule_rows is {} but the store was opened with {}; every writer on one schedule database must open with the same ScheduleLimits",
                ceiling_label(limits),
                ceiling_label(stored),
            )));
        }
        Ok(Self::new(Arc::new(store), clock, jitter))
    }
}

impl<C: Clock + 'static> Scheduler<SqliteScheduleStore, C, crate::SystemJitter> {
    /// Opens built-in components sharing one canonical logger.
    ///
    /// # Errors
    /// Returns `CronError` when the store cannot be opened or validated.
    pub fn open_with_logger(
        path: impl AsRef<std::path::Path>,
        limits: ScheduleLimits,
        clock: Arc<C>,
        logger: SharedCanonicalLog,
    ) -> Result<Self, CronError> {
        let store = SqliteScheduleStore::open_with_logger(path, limits, Arc::clone(&logger))?;
        let jitter = crate::SystemJitter::with_logger(Arc::clone(&logger));
        Ok(Self::new_with_logger(
            Arc::new(store),
            clock,
            Arc::new(jitter),
            logger,
        ))
    }
}

fn ceiling_label(limits: ScheduleLimits) -> String {
    match limits.max_schedule_rows {
        Some(limit) => limit.to_string(),
        None => "unlimited".to_owned(),
    }
}

/// One run started by [`Scheduler::run_due`] and how it ended.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DueRun {
    /// The schedule that ran.
    pub name: ScheduleName,
    /// Its outcome; a stop check that disabled or ended the schedule is
    /// [`RunOutcome::Refused`] with its reason.
    pub outcome: RunOutcome,
}

impl<S, C, J> Scheduler<S, C, J>
where
    S: ScheduleStore + 'static,
    C: Clock + 'static,
    J: JitterSource + 'static,
{
    /// Runs every enabled schedule that is due now, once, and waits for them.
    ///
    /// A schedule is due when its stored next run is at or before the
    /// clock's now and that occurrence was not already recorded. Each due
    /// run uses the row's stored hook and `job` for a Rust row, stop-checked
    /// and run exactly as the worker does, under its own deadline (the
    /// timeout from its start, on the clock's monotonic timeline; a run past
    /// it is [`RunOutcome::TimedOut`]). The following occurrence is stored
    /// before the hook runs. A finished run is recorded with its generation,
    /// and a schedule with no further occurrence is ended after a run that
    /// was not a stop-check disable, as the worker does. A stop check that
    /// disables the row leaves it active and disabled and does not end it.
    /// A final run whose ending write did not commit is ended on a later
    /// call, without running the hook again; a re-enabled stop-check disable
    /// is left as it is. A schedule that is not due is left untouched. This
    /// is for a caller
    /// that drives the table from an outside scheduler instead of a worker:
    /// never run it beside a worker on the same rows. Dropping the returned
    /// future aborts Rust and Python runs it started; a Monty feed keeps
    /// running on the blocking pool until it returns and the spawned task
    /// records the outcome.
    ///
    /// # Errors
    ///
    /// The store's errors on the first read; later store failures are logged
    /// and do not stop the other runs.
    pub async fn run_due(&self, job: Arc<dyn Job>) -> Result<Vec<DueRun>, CronError> {
        run_due::drive(self, job).await
    }

    /// Records one run of [`Scheduler::run_due`].
    ///
    /// The following occurrence, when there is one, was stored before the
    /// hook ran. A stop-check disable only disables. A run with no further
    /// occurrence ends the schedule. A refused follow-up write after a
    /// successful record is a partial write: the outcome stays recorded.
    pub(super) async fn finish_due(
        &self,
        row: &Schedule,
        report: Report,
        next: Option<DateTime<Utc>>,
        correlation: crate::worker::RunCorrelation,
    ) -> DueRun {
        record_due(
            Arc::clone(&self.core),
            Arc::clone(&self.coord.logger),
            row,
            report,
            next,
            correlation,
        )
        .await
    }
}

/// Runs `operation` on the blocking pool, mapping a panic to a store error.
pub(super) async fn blocking<T: Send + 'static>(
    operation: impl FnOnce() -> Result<T, CronError> + Send + 'static,
) -> Result<T, CronError> {
    tokio::task::spawn_blocking(operation)
        .await
        .map_err(|error| {
            CronError::Store(
                StoreError::new(StoreErrorKind::Other, "blocking store task failed")
                    .with_source(error),
            )
        })?
}

impl<S: ScheduleStore, C: Clock, J: JitterSource> std::fmt::Debug for Scheduler<S, C, J> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut dbg = f.debug_struct("Scheduler");
        dbg.field("service", self.coord.logger.service_name())
            .field("policy", &self.policy)
            .field(
                "max_schedule_rows",
                &self.core.store.limits().max_schedule_rows,
            )
            .field("worker_live", &self.worker_live.load(Ordering::Acquire));
        #[cfg(feature = "monty")]
        dbg.field("monty_hosts", &*self.hosts);
        dbg.finish_non_exhaustive()
    }
}

/// Clears the scheduler's one-worker flag when the worker task ends or is dropped.
struct LiveGuard(Arc<AtomicBool>);

impl Drop for LiveGuard {
    fn drop(&mut self) {
        self.0.store(false, Ordering::Release);
    }
}

/// Owner of a running worker; await [`WorkerHandle::shutdown`] to stop it.
///
/// Call [`WorkerHandle::shutdown`] before the handle is dropped. Dropping
/// the handle signals the worker to stop and does not wait. The worker then
/// drains its in-flight runs detached, and their outcomes are lost if the
/// process exits first.
pub struct WorkerHandle {
    shutdown: Option<oneshot::Sender<Option<tokio::time::Instant>>>,
    task: Option<JoinHandle<()>>,
    feeds: Arc<AtomicUsize>,
    logger: SharedCanonicalLog,
}

impl std::fmt::Debug for WorkerHandle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WorkerHandle")
            .field("service", self.logger.service_name())
            .field("feeds", &self.feeds)
            .finish_non_exhaustive()
    }
}

impl WorkerHandle {
    /// Constructs a gated worker owner to exercise shutdown deadline branches.
    #[cfg(feature = "test-utils")]
    #[doc(hidden)]
    pub fn shutdown_probe(
        logger: SharedCanonicalLog,
        outstanding_feeds: usize,
    ) -> (Self, tokio::sync::oneshot::Sender<()>) {
        let (release, wait) = tokio::sync::oneshot::channel();
        let task = tokio::spawn(async move {
            let _ = wait.await;
        });
        (
            Self {
                shutdown: None,
                task: Some(task),
                feeds: Arc::new(AtomicUsize::new(outstanding_feeds)),
                logger,
            },
            release,
        )
    }
    /// Stops the worker, waiting for in-flight runs until `deadline`.
    ///
    /// The worker stops launching runs, then waits for each in-flight run
    /// until that run's own deadline, recording its outcome. If `deadline`
    /// passes while no Monty feed is outstanding, the worker task is aborted
    /// and records nothing. While a Monty feed is outstanding the worker is
    /// not aborted: Rust and Python siblings are recorded `TimedOut`, and the
    /// Monty feed runs until it returns. Returns whether the worker finished
    /// before `deadline`.
    pub async fn shutdown(mut self, deadline: tokio::time::Instant) -> bool {
        if let Some(shutdown) = self.shutdown.take() {
            // The worker may already have exited; nothing to signal then.
            let _ = shutdown.send(Some(deadline));
        }
        let Some(mut task) = self.task.take() else {
            return true;
        };
        if tokio::time::timeout_at(deadline, &mut task).await.is_ok() {
            return true;
        }
        if self.feeds.load(Ordering::Acquire) == 0 {
            if let Some(event) = crate::component_event(
                self.logger.service_name().clone(),
                "cron.scheduler",
                "shutdown.worker_timeout",
                Level::Error,
                "timeout",
                Some("WORKER_SHUTDOWN_TIMEOUT"),
                serde_json::json!({"deadline_remaining_ms": 0, "worker_count": 1})
                    .as_object()
                    .cloned()
                    .unwrap_or_default(),
            ) {
                let _ = self.logger.emit(event);
            }
            task.abort();
            let _ = task.await;
            return false;
        }
        if let Some(event) = crate::component_event(
            self.logger.service_name().clone(),
            "cron.scheduler",
            "shutdown.deadline_exceeded",
            Level::Warn,
            "timeout",
            Some("SHUTDOWN_DEADLINE_EXCEEDED"),
            serde_json::json!({"deadline_remaining_ms": 0, "worker_count": 1,
                "feed_count": self.feeds.load(Ordering::Acquire)})
            .as_object()
            .cloned()
            .unwrap_or_default(),
        ) {
            let _ = self.logger.emit(event);
        }
        let _ = task.await;
        false
    }

    /// Returns whether the worker task has ended.
    #[must_use]
    pub fn is_finished(&self) -> bool {
        self.task.as_ref().is_none_or(JoinHandle::is_finished)
    }
}

impl Drop for WorkerHandle {
    fn drop(&mut self) {
        if let Some(shutdown) = self.shutdown.take() {
            // The worker may already have exited; nothing to signal then.
            let _ = shutdown.send(None);
        }
    }
}

#[cfg(test)]
mod tests {
    use std::num::NonZeroUsize;
    use std::sync::Arc;

    use super::Scheduler;
    use crate::{CronError, ScheduleLimits, SqliteScheduleStore, SystemClock, SystemJitter};

    #[test]
    fn open_rejects_limits_that_differ_from_the_store() {
        let store = SqliteScheduleStore::open_in_memory(ScheduleLimits::default()).expect("store");
        let passed = ScheduleLimits {
            max_schedule_rows: NonZeroUsize::new(4),
        };
        let error = Scheduler::from_opened_store(
            store,
            passed,
            Arc::new(SystemClock),
            Arc::new(SystemJitter),
        )
        .expect_err("mismatch");
        let CronError::InvalidSpec(detail) = error else {
            panic!("expected InvalidSpec, got {error}");
        };
        assert!(detail.contains("4"), "{detail}");
        assert!(detail.contains("unlimited"), "{detail}");
        assert!(detail.contains("same ScheduleLimits"), "{detail}");
    }
}
