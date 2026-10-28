//! Schedule identity, validated input, stored state and run results.
//!
//! A schedule is identified by its [`ScheduleId`], stable for the life of
//! its name, and fenced by its [`Generation`], which a store raises each time
//! an ended schedule is started again. Every run-state write names both, so a
//! run that began under an older generation can never write over the
//! restarted schedule.
//!
//! Input is validated once, when its type is built: [`ScheduleName`],
//! [`CronExpr`](crate::CronExpr) and [`RunTimeout`] cannot hold an invalid
//! value, so the scheduler and stores never check them again.

use std::fmt;
use std::num::NonZeroU32;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::time::Duration;

/// Longest stored Python module path, in bytes.
const MAX_MODULE_LEN: usize = 1024;

use crate::limits::{MAX_HOOK_SOURCE_BYTES, MAX_NAME_LEN, MAX_TIMEOUT};
use crate::{CronError, CronExpr, Timestamp};

/// Stable row id of one schedule; kept when an ended name is started again.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ScheduleId(u64);

impl ScheduleId {
    /// Wraps a raw id, as read from a store.
    #[must_use]
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    /// Returns the raw id.
    #[must_use]
    pub const fn get(self) -> u64 {
        self.0
    }
}

impl fmt::Display for ScheduleId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Fencing generation of one schedule; a stale run cannot record.
///
/// Generations start at [`Generation::FIRST`] and rise by one each time an
/// ended schedule is started again under the same name.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Generation(NonZeroU32);

impl Generation {
    /// The generation of a newly created schedule.
    pub const FIRST: Self = Self(NonZeroU32::MIN);

    /// Wraps a raw generation; `None` for zero, which no schedule carries.
    #[must_use]
    pub const fn new(value: u32) -> Option<Self> {
        match NonZeroU32::new(value) {
            Some(value) => Some(Self(value)),
            None => None,
        }
    }

    /// Returns the raw generation.
    #[must_use]
    pub const fn get(self) -> u32 {
        self.0.get()
    }

    /// Returns the following generation, or `None` once `u32::MAX` is reached.
    #[must_use]
    pub fn next(self) -> Option<Self> {
        self.0.checked_add(1).map(Self)
    }
}

impl fmt::Display for Generation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// A valid schedule name.
///
/// A name is 1 to [`MAX_NAME_LEN`](crate::MAX_NAME_LEN) bytes, has no
/// leading or trailing whitespace and no control characters. This is the
/// one definition of a valid name; the scheduler and the store both rely on
/// it.
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ScheduleName(String);

impl ScheduleName {
    /// Validates `name`.
    ///
    /// # Errors
    ///
    /// [`CronError::InvalidSpec`] if `name` breaks a rule above.
    pub fn new(name: impl Into<String>) -> Result<Self, CronError> {
        let name = name.into();
        let invalid = |why: &str| CronError::InvalidSpec(format!("name {name:?} {why}"));
        if name.is_empty() {
            return Err(invalid("is empty"));
        }
        if name.len() > MAX_NAME_LEN {
            return Err(invalid(&format!("is longer than {MAX_NAME_LEN} bytes")));
        }
        if name.trim() != name {
            return Err(invalid("has leading or trailing whitespace"));
        }
        if name.chars().any(char::is_control) {
            return Err(invalid("contains a control character"));
        }
        Ok(Self(name))
    }

    /// Returns the name.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl FromStr for ScheduleName {
    type Err = CronError;

    fn from_str(name: &str) -> Result<Self, Self::Err> {
        Self::new(name)
    }
}

impl fmt::Display for ScheduleName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl AsRef<str> for ScheduleName {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

/// A valid run timeout: whole milliseconds, 1 ms to [`MAX_TIMEOUT`](crate::MAX_TIMEOUT).
///
/// The timeout bounds a stop check and its run together.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct RunTimeout(Duration);

impl RunTimeout {
    /// Validates `timeout`, dropping precision finer than a millisecond.
    ///
    /// # Errors
    ///
    /// [`CronError::InvalidSpec`] if `timeout` is under 1 ms or over
    /// [`MAX_TIMEOUT`](crate::MAX_TIMEOUT).
    pub fn new(timeout: Duration) -> Result<Self, CronError> {
        let millis = timeout.as_millis();
        if millis == 0 || timeout > MAX_TIMEOUT {
            return Err(CronError::InvalidSpec(format!(
                "timeout {timeout:?} is outside 1 ms..={} s",
                MAX_TIMEOUT.as_secs()
            )));
        }
        let millis = u64::try_from(millis).map_err(|_| {
            CronError::InvalidSpec(format!(
                "timeout {timeout:?} is outside 1 ms..={} s",
                MAX_TIMEOUT.as_secs()
            ))
        })?;
        Ok(Self(Duration::from_millis(millis)))
    }

    /// Returns the timeout.
    #[must_use]
    pub const fn get(self) -> Duration {
        self.0
    }
}

/// What a schedule row runs.
///
/// `Rust` calls the [`Job`](crate::Job) passed to
/// [`Scheduler::spawn_worker`](crate::Scheduler::spawn_worker). `Python`
/// and [`HookTarget::Monty`] run `cron.stop` and `cron.tick` with the
/// [`sc-cron.hook.v1`](crate::hook::HOOK_PROTOCOL) JSON object (required
/// `protocol`, `event`, `schedule`, `payload`, and `deadline`). Python
/// writes it to the child's stdin; Monty passes it as the hook argument.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum HookTarget {
    /// Call the worker's in-process job.
    Rust,
    /// A Python file. `module` is the path given to `python3`.
    Python {
        /// Path of the Python file.
        module: PathBuf,
    },
    /// Trusted source executed by the optional in-process Monty interpreter.
    ///
    /// The source is program text, not a filesystem path. It must define
    /// `cron_stop(hook)` and `cron_tick(hook)` when the feature is enabled.
    Monty {
        /// Trusted Python-compatible hook program text.
        source: String,
    },
}

impl HookTarget {
    /// Builds a Python target from `module`.
    ///
    /// The path is non-empty, UTF-8, not an option (it does not start with
    /// `-`), and at most 1024 bytes. It is stored as given. The runner makes
    /// a relative path absolute when the hook starts, so the temporary
    /// working directory does not change what the path means.
    ///
    /// # Errors
    ///
    /// [`CronError::InvalidSpec`] if `module` breaks a rule above.
    pub fn python(module: impl AsRef<Path>) -> Result<Self, CronError> {
        let module = module.as_ref();
        let Some(text) = module.to_str() else {
            return Err(CronError::InvalidSpec(
                "module path is not UTF-8".to_owned(),
            ));
        };
        if text.is_empty() {
            return Err(CronError::InvalidSpec("module path is empty".to_owned()));
        }
        if text.starts_with('-') || text.contains('\0') {
            return Err(CronError::InvalidSpec(format!(
                "module {text:?} is not a path python3 can be given"
            )));
        }
        if text.len() > MAX_MODULE_LEN {
            return Err(CronError::InvalidSpec(format!(
                "module path is longer than {MAX_MODULE_LEN} bytes"
            )));
        }
        Ok(Self::Python {
            module: PathBuf::from(text),
        })
    }

    /// Builds a trusted in-process Monty target from source text.
    ///
    /// # Errors
    ///
    /// [`CronError::InvalidSpec`] if `source` exceeds
    /// [`MAX_HOOK_SOURCE_BYTES`](crate::MAX_HOOK_SOURCE_BYTES).
    pub fn monty(source: impl Into<String>) -> Result<Self, CronError> {
        let source = source.into();
        if source.len() > MAX_HOOK_SOURCE_BYTES {
            return Err(CronError::InvalidSpec(format!(
                "Monty source is {} bytes; the limit is {MAX_HOOK_SOURCE_BYTES}",
                source.len()
            )));
        }
        Ok(Self::Monty { source })
    }

    /// The stored kind, module text and source text.
    pub(crate) fn stored(&self) -> (&'static str, &str, &str) {
        match self {
            Self::Rust => ("rust", "", ""),
            Self::Python { module } => ("python", module.to_str().unwrap_or(""), ""),
            Self::Monty { source } => ("monty", "", source),
        }
    }

    /// Reads a stored kind, module text and source text.
    pub(crate) fn from_stored(kind: &str, module: &str, source: &str) -> Result<Self, String> {
        match HookKind::parse(kind)? {
            HookKind::Rust if module.is_empty() && source.is_empty() => Ok(Self::Rust),
            HookKind::Rust => Err("rust hook has module or source text".to_owned()),
            HookKind::Python if module.is_empty() => {
                Err("python hook has an empty module".to_owned())
            }
            HookKind::Python if !source.is_empty() => Err("python hook has source text".to_owned()),
            HookKind::Python if module.len() > MAX_MODULE_LEN => {
                Err(format!("module path is longer than {MAX_MODULE_LEN} bytes"))
            }
            HookKind::Python => Ok(Self::Python {
                module: PathBuf::from(module),
            }),
            HookKind::Monty if !module.is_empty() => Err("monty hook has a module path".to_owned()),
            HookKind::Monty if source.len() > MAX_HOOK_SOURCE_BYTES => Err(format!(
                "Monty source is {} bytes; the limit is {MAX_HOOK_SOURCE_BYTES}",
                source.len()
            )),
            HookKind::Monty => Ok(Self::Monty {
                source: source.to_owned(),
            }),
        }
    }
}

/// Stored hook kind. The database column stays text; this is the only parser.
enum HookKind {
    Rust,
    Python,
    Monty,
}

impl HookKind {
    fn parse(kind: &str) -> Result<Self, String> {
        match kind {
            "rust" => Ok(Self::Rust),
            "python" => Ok(Self::Python),
            "monty" => Ok(Self::Monty),
            other => Err(format!("unknown hook kind {other:?}")),
        }
    }
}

/// What a caller asks the scheduler to run.
///
/// `payload` is opaque to the scheduler and handed to the [`Job`](crate::Job)
/// on every stop check and run. Its compact JSON encoding is limited to
/// [`MAX_PAYLOAD_BYTES`](crate::MAX_PAYLOAD_BYTES), checked by
/// [`ScheduleSpec::new`], by [`Scheduler::start`](crate::Scheduler::start)
/// and by the store, with one rule. `hook` says whether the row calls that
/// job or a Python file.
#[derive(Clone, Debug, PartialEq)]
pub struct ScheduleSpec {
    /// Unique name; an active schedule's name cannot be started again.
    pub name: ScheduleName,
    /// Cron expression.
    pub expr: CronExpr,
    /// Budget for one stop check plus run.
    pub timeout: RunTimeout,
    /// Opaque job input.
    pub payload: serde_json::Value,
    /// Rust job or Python file for this row.
    pub hook: HookTarget,
}

impl ScheduleSpec {
    /// Validates every field and builds a spec.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::time::Duration;
    ///
    /// let spec = sc_cron::ScheduleSpec::new(
    ///     "sync",
    ///     "*/5 * * * *",
    ///     Duration::from_secs(25),
    ///     serde_json::json!({"n": 1}),
    /// )?;
    /// assert_eq!(spec.name.as_str(), "sync");
    /// assert!(matches!(spec.hook, sc_cron::HookTarget::Rust));
    /// # Ok::<(), sc_cron::CronError>(())
    /// ```
    ///
    /// # Errors
    ///
    /// [`CronError::InvalidSpec`] for an invalid name, timeout, over-long
    /// expression or over-size payload; [`CronError::BadExpression`] for an
    /// expression that does not parse.
    pub fn new(
        name: &str,
        expr: &str,
        timeout: Duration,
        payload: serde_json::Value,
    ) -> Result<Self, CronError> {
        Self::with_hook(name, expr, timeout, payload, HookTarget::Rust)
    }

    /// As [`ScheduleSpec::new`], with `hook` instead of [`HookTarget::Rust`].
    ///
    /// # Errors
    ///
    /// [`CronError::InvalidSpec`] for an invalid name, timeout, over-long
    /// expression or over-size payload; [`CronError::BadExpression`] for an
    /// expression that does not parse.
    pub fn with_hook(
        name: &str,
        expr: &str,
        timeout: Duration,
        payload: serde_json::Value,
        hook: HookTarget,
    ) -> Result<Self, CronError> {
        if let HookTarget::Monty { source } = &hook
            && source.len() > MAX_HOOK_SOURCE_BYTES
        {
            return Err(CronError::InvalidSpec(format!(
                "Monty source is {} bytes; the limit is {MAX_HOOK_SOURCE_BYTES}",
                source.len()
            )));
        }
        let spec = Self {
            name: ScheduleName::new(name)?,
            expr: CronExpr::parse(expr)?,
            timeout: RunTimeout::new(timeout)?,
            payload,
            hook,
        };
        crate::limits::encode_payload(&spec.payload)?;
        Ok(spec)
    }
}

/// A schedule as the scheduler asks a store to create it.
///
/// The scheduler fills `jitter` and `next_run_at` from its injected jitter
/// source and clock, so the store needs neither. A new or restarted schedule
/// is enabled.
#[derive(Clone, Debug, PartialEq)]
pub struct NewSchedule {
    /// The caller's specification.
    pub spec: ScheduleSpec,
    /// Fixed offset added to every occurrence, in whole milliseconds.
    pub jitter: Duration,
    /// First run time.
    pub next_run_at: Timestamp,
}

/// One stored schedule with its run state.
///
/// A schedule is active while `ended_at` is `None`; an active schedule
/// always has a `next_run_at`, an ended one never does. An active schedule
/// that is not `enabled` is kept, with its generation, expression and
/// payload, but never fires (REQ-CRON-0014).
#[derive(Clone, Debug, PartialEq)]
pub struct Schedule {
    /// Stable id.
    pub id: ScheduleId,
    /// Current fencing generation.
    pub generation: Generation,
    /// The specification it was (re)started with.
    pub spec: ScheduleSpec,
    /// Fixed jitter added to each occurrence.
    pub jitter: Duration,
    /// When it was (re)started.
    pub created_at: Timestamp,
    /// When it next runs; `None` once ended.
    pub next_run_at: Option<Timestamp>,
    /// When the last recorded run finished.
    pub last_run_at: Option<Timestamp>,
    /// The last recorded run's result.
    pub last_outcome: Option<RunOutcome>,
    /// When it ended; `None` while active.
    pub ended_at: Option<Timestamp>,
    /// Why it ended, when it has.
    pub stop_reason: Option<String>,
    /// Whether the worker fires it; a disabled schedule stays active.
    pub enabled: bool,
}

impl Schedule {
    /// Returns whether the schedule is still active.
    #[must_use]
    pub fn is_active(&self) -> bool {
        self.ended_at.is_none()
    }

    /// The first jittered run strictly after `after`, as the worker computes
    /// it (REQ-CRON-0002); `None` when the expression has no further occurrence.
    #[must_use]
    pub fn next_run_after(&self, after: Timestamp) -> Option<Timestamp> {
        self.spec
            .expr
            .next_run(self.jitter, after.as_datetime())
            .map(Timestamp::from_datetime)
    }

    /// The gap between the schedule's first two runs strictly after `after`.
    ///
    /// For a regular expression this is its interval; for an irregular one
    /// (`0 0,5 * * * *`) it is the gap that follows `after`, so it varies with
    /// `after`. `None` when fewer than two runs follow.
    ///
    /// # Examples
    ///
    /// ```
    /// # use std::time::Duration;
    /// # use sc_cron::{Generation, Schedule, ScheduleId, ScheduleSpec, Timestamp};
    /// let spec = ScheduleSpec::new("s", "0 0,5 * * * *", Duration::from_secs(1), serde_json::json!({}))?;
    /// let at: Timestamp = "2026-10-26T10:01:00Z".parse()?;
    /// let row = Schedule {
    ///     id: ScheduleId::new(1), generation: Generation::FIRST, spec, jitter: Duration::ZERO,
    ///     created_at: at, next_run_at: None, last_run_at: None, last_outcome: None,
    ///     ended_at: None, stop_reason: None, enabled: true,
    /// };
    /// // The runs after 10:01 are 10:05 and 11:00: a 55-minute gap.
    /// assert_eq!(row.period_after(at), Some(Duration::from_secs(55 * 60)));
    /// # Ok::<(), sc_cron::CronError>(())
    /// ```
    #[must_use]
    pub fn period_after(&self, after: Timestamp) -> Option<Duration> {
        let first = self.next_run_after(after)?;
        let second = self.next_run_after(first)?;
        (second.as_datetime() - first.as_datetime()).to_std().ok()
    }
}

/// The result of one run, as recorded by the store.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RunOutcome {
    /// The job completed its work.
    Ok,
    /// The job declined this run; the schedule stays active.
    Refused(String),
    /// The job failed or panicked; the schedule stays active.
    Failed(String),
    /// The stop check and run together exceeded the schedule's timeout.
    TimedOut,
}

impl RunOutcome {
    /// The text stored in `last_outcome`. [`Display`] uses the same text.
    #[must_use]
    pub(crate) fn to_stored(&self) -> String {
        match self {
            Self::Ok => "ok".to_owned(),
            Self::Refused(message) => format!("refused: {message}"),
            Self::Failed(message) => format!("failed: {message}"),
            Self::TimedOut => "timed_out".to_owned(),
        }
    }

    /// Inverse of [`RunOutcome::to_stored`].
    #[must_use]
    pub(crate) fn from_stored(text: &str) -> Option<Self> {
        match text {
            "ok" => Some(Self::Ok),
            "timed_out" => Some(Self::TimedOut),
            _ => text
                .strip_prefix("refused: ")
                .map(|message| Self::Refused(message.to_owned()))
                .or_else(|| {
                    text.strip_prefix("failed: ")
                        .map(|message| Self::Failed(message.to_owned()))
                }),
        }
    }
}

impl fmt::Display for RunOutcome {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.to_stored())
    }
}

/// The answer of a job's stop check, asked before every run (REQ-CRON-0005).
///
/// `Disable` and `Delete` both skip this tick. A stop check that cannot
/// answer (a panic, a timeout, or for a Python hook a failure or a malformed
/// reply) is not a decision: the run is skipped, the outcome is recorded, and
/// the schedule stays active and enabled.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum StopDecision {
    /// Run this tick.
    Continue,
    /// Keep the schedule but stop firing it, with this reason; it can be
    /// enabled again with [`Scheduler::set_enabled`](crate::Scheduler::set_enabled).
    /// The reason is kept as the schedule's last outcome, [`RunOutcome::Refused`].
    Disable(String),
    /// End the schedule with this reason (ADR-CRON-0007). Its row is kept as
    /// the ended record, and the name can be started again.
    Delete(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generation_rejects_zero_and_counts_up() {
        assert_eq!(Generation::new(0), None);
        assert_eq!(Generation::FIRST.get(), 1);
        assert_eq!(Generation::FIRST.next().map(Generation::get), Some(2));
        assert_eq!(Generation::new(u32::MAX).and_then(Generation::next), None);
    }

    #[test]
    fn schedule_id_reads_back_and_displays_the_integer() {
        let id = ScheduleId::new(42);
        assert_eq!(id.get(), 42);
        assert_eq!(id.to_string(), "42");
    }

    #[test]
    fn schedule_name_has_one_validation_rule() {
        assert!(ScheduleName::new("sync-job").is_ok());
        let longest = "n".repeat(MAX_NAME_LEN);
        assert!(ScheduleName::new(longest.as_str()).is_ok());
        for bad in [
            String::new(),
            "  ".into(),
            " sync".into(),
            "sync ".into(),
            "a\nb".into(),
            "n".repeat(MAX_NAME_LEN + 1),
        ] {
            assert!(
                matches!(
                    ScheduleName::new(bad.as_str()),
                    Err(CronError::InvalidSpec(_))
                ),
                "{bad:?}"
            );
        }
    }

    #[test]
    fn run_timeout_bounds_and_truncates_to_milliseconds() {
        assert_eq!(
            RunTimeout::new(Duration::from_micros(1500))
                .map(RunTimeout::get)
                .ok(),
            Some(Duration::from_millis(1))
        );
        assert!(RunTimeout::new(MAX_TIMEOUT).is_ok());
        for bad in [
            Duration::ZERO,
            Duration::from_micros(999),
            MAX_TIMEOUT + Duration::from_millis(1),
        ] {
            assert!(matches!(
                RunTimeout::new(bad),
                Err(CronError::InvalidSpec(_))
            ));
        }
    }

    #[test]
    fn run_outcome_display_names_each_class() {
        assert_eq!(RunOutcome::Ok.to_string(), "ok");
        assert_eq!(
            RunOutcome::Refused("quota".into()).to_string(),
            "refused: quota"
        );
        assert_eq!(
            RunOutcome::Failed("boom".into()).to_string(),
            "failed: boom"
        );
        assert_eq!(RunOutcome::TimedOut.to_string(), "timed_out");
    }

    #[test]
    fn monty_rejects_source_over_the_cap_and_the_variant_stays_matchable() {
        let over = "x".repeat(crate::MAX_HOOK_SOURCE_BYTES + 1);
        assert!(matches!(
            HookTarget::monty(over),
            Err(CronError::InvalidSpec(_))
        ));
        let hook = HookTarget::monty("pass").expect("short source");
        assert!(matches!(hook, HookTarget::Monty { ref source } if source == "pass"));
        // `HookTarget` is `#[non_exhaustive]`; a wildcard keeps this match compiling
        // when a later variant is added.
        match hook {
            HookTarget::Rust | HookTarget::Python { .. } | HookTarget::Monty { .. } => {}
        }
    }
}
