//! Fixed event taxonomy kept as data, independent of emission.
use super::EventRow;
use sc_observability_types::Level;

pub(super) const TARGET_RESOLVE_FAILED: EventRow = (
    "cron.worker",
    "target.resolve_failed",
    Level::Warn,
    Some("failed"),
    Some("TARGET_RESOLVE_FAILED"),
);

pub(super) const SCHEDULE_RELOAD_FAILED: EventRow = (
    "cron.worker",
    "schedule.reload_failed",
    Level::Error,
    Some("failed"),
    Some("SCHEDULE_RELOAD_FAILED"),
);

pub(super) const SCHEDULE_CANCELLED: EventRow = (
    "cron.worker",
    "schedule.cancelled",
    Level::Info,
    Some("stopped"),
    None,
);

pub(super) const RUN_TIMED_OUT: EventRow = (
    "cron.worker",
    "run.timed_out",
    Level::Warn,
    Some("timeout"),
    Some("RUN_TIMEOUT"),
);

pub(super) const RUN_SKIPPED_IN_FLIGHT: EventRow = (
    "cron.worker",
    "run.skipped_in_flight",
    Level::Info,
    None,
    None,
);

pub(super) const RUN_MISSED_REPLAYED: EventRow = (
    "cron.worker",
    "run.missed_replayed",
    Level::Info,
    None,
    None,
);

pub(super) const RUN_LAUNCHED: EventRow = ("cron.worker", "run.launched", Level::Info, None, None);

pub(super) const RUN_FINISHED: EventRow = ("cron.worker", "run.finished", Level::Info, None, None);

pub(super) const RUN_RECORD_FAILED: EventRow = (
    "cron.worker",
    "run.record_failed",
    Level::Error,
    Some("failed"),
    Some("RUN_RECORD_FAILED"),
);

pub(super) const SCHEDULE_NEXT_FAILED: EventRow = (
    "cron.worker",
    "schedule.next_failed",
    Level::Error,
    Some("failed"),
    Some("SCHEDULE_NEXT_FAILED"),
);

pub(super) const SCHEDULE_STOPPED: EventRow = (
    "cron.worker",
    "schedule.stopped",
    Level::Info,
    Some("stopped"),
    None,
);

pub(super) const SCHEDULE_STOP_FAILED: EventRow = (
    "cron.worker",
    "schedule.stop_failed",
    Level::Error,
    Some("failed"),
    Some("SCHEDULE_STOP_FAILED"),
);

pub(super) const SCHEDULE_DISABLED: EventRow = (
    "cron.worker",
    "schedule.disabled",
    Level::Info,
    Some("stopped"),
    None,
);

pub(super) const SCHEDULE_DISABLE_FAILED: EventRow = (
    "cron.worker",
    "schedule.disable_failed",
    Level::Error,
    Some("failed"),
    Some("SCHEDULE_DISABLE_FAILED"),
);

pub(super) const RUN_FENCED: EventRow = (
    "cron.worker",
    "run.fenced",
    Level::Warn,
    Some("fenced"),
    Some("SCHEDULE_FENCED"),
);

pub(super) const DUE_LAUNCHED: EventRow =
    ("cron.scheduler", "due.launched", Level::Info, None, None);

pub(super) const DUE_FINISHED: EventRow =
    ("cron.scheduler", "due.finished", Level::Info, None, None);

pub(super) const DUE_FENCED: EventRow = (
    "cron.scheduler",
    "due.fenced",
    Level::Warn,
    Some("fenced"),
    Some("SCHEDULE_FENCED"),
);

pub(super) const DUE_PARTIAL_WRITE: EventRow = (
    "cron.scheduler",
    "due.partial_write",
    Level::Warn,
    Some("partial"),
    Some("FOLLOW_UP_FENCED"),
);

pub(super) const DUE_FOLLOW_UP_FAILED: EventRow = (
    "cron.scheduler",
    "due.follow_up_failed",
    Level::Error,
    Some("partial"),
    Some("FOLLOW_UP_WRITE_FAILED"),
);

pub(super) const DUE_RECORD_FAILED: EventRow = (
    "cron.scheduler",
    "due.record_failed",
    Level::Error,
    Some("failed"),
    Some("RUN_RECORD_FAILED"),
);

pub(super) const DUE_STOPPED: EventRow = (
    "cron.scheduler",
    "due.stopped",
    Level::Info,
    Some("stopped"),
    None,
);

pub(super) const DUE_STOP_FAILED: EventRow = (
    "cron.scheduler",
    "due.stop_failed",
    Level::Error,
    Some("failed"),
    Some("SCHEDULE_STOP_FAILED"),
);

pub(super) const DUE_NEXT_FAILED: EventRow = (
    "cron.scheduler",
    "due.next_failed",
    Level::Error,
    Some("failed"),
    Some("SCHEDULE_NEXT_FAILED"),
);

pub(super) const JOB_JOIN_FAILED: EventRow = (
    "cron.worker",
    "job.join_failed",
    Level::Error,
    Some("failed"),
    Some("JOB_JOIN_FAILED"),
);

#[cfg(feature = "monty")]
pub(super) const DUE_JOIN_FAILED: EventRow = (
    "cron.scheduler",
    "due.join_failed",
    Level::Error,
    Some("failed"),
    Some("DUE_JOIN_FAILED"),
);

pub(super) const DUE_TASK_FAILED: EventRow = (
    "cron.scheduler",
    "due.task_failed",
    Level::Error,
    Some("failed"),
    Some("RUN_TASK_FAILED"),
);

#[cfg(feature = "monty")]
pub(super) const MONTY_RESOLVE_FAILED: EventRow = (
    "cron.scheduler",
    "monty.resolve_failed",
    Level::Error,
    Some("failed"),
    Some("MONTY_RESOLVE"),
);

#[cfg(feature = "monty")]
pub(super) const MONTY_TICK_FAILED: EventRow = (
    "cron.scheduler",
    "monty.tick_failed",
    Level::Error,
    Some("failed"),
    Some("MONTY_TICK"),
);

#[cfg(feature = "monty")]
pub(super) const FEED_JOIN_FAILED: EventRow = (
    "cron.monty",
    "feed.join_failed",
    Level::Error,
    Some("failed"),
    Some("FEED_JOIN_FAILED"),
);

pub(super) const RUN_MISSED_SKIPPED: EventRow =
    ("cron.worker", "run.missed_skipped", Level::Info, None, None);
