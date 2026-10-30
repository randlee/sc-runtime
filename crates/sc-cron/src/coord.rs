//! Shared Monty coordination for the worker and [`crate::Scheduler::run_due`].
//!
//! The feed count and the per-name mark are atomics and a mutex held only for
//! an insert or a remove. Neither is held across an `.await`. Dropping a guard
//! clears its claim, including when the owner panics.

#[cfg(feature = "monty")]
use std::collections::HashMap;
use std::collections::HashSet;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use tokio::sync::watch;

use crate::ScheduleName;

/// Claims shared by a scheduler, its worker, and `run_due`.
pub(crate) struct RunCoord {
    pub logger: sc_observability_types::SharedCanonicalLog,
    /// Monty joins whose feed has not yet finished its outcome write.
    pub feeds: Arc<AtomicUsize>,
    /// Serializes the atomic update with [`Self::feed_level`] so a waiter cannot observe a stale count.
    feed_lock: Mutex<()>,
    /// Outstanding feed count after each enter or drop. Tests wait on this.
    feed_level: watch::Sender<usize>,
    /// Times a shutdown deadline elapsed while a Monty feed was still in flight.
    shutdown_deadlines: watch::Sender<u64>,
    /// Schedule names whose `run_due` feed has not finished `finish_due`.
    pub names: Arc<Mutex<HashSet<ScheduleName>>>,
    /// Repls that are not inside a feed, keyed by schedule name.
    #[cfg(feature = "monty")]
    pub(crate) repls: Mutex<HashMap<ScheduleName, crate::monty_run::MontySession>>,
}

impl RunCoord {
    #[cfg(test)]
    pub(crate) fn new() -> Self {
        Self::with_logger(crate::noop_logger())
    }

    pub(crate) fn with_logger(logger: sc_observability_types::SharedCanonicalLog) -> Self {
        Self {
            logger,
            feeds: Arc::new(AtomicUsize::new(0)),
            feed_lock: Mutex::new(()),
            feed_level: watch::channel(0).0,
            shutdown_deadlines: watch::channel(0).0,
            names: Arc::new(Mutex::new(HashSet::new())),
            #[cfg(feature = "monty")]
            repls: Mutex::new(HashMap::new()),
        }
    }

    #[cfg(feature = "monty")]
    pub(crate) fn take_repl(&self, name: &ScheduleName) -> Option<crate::monty_run::MontySession> {
        self.repls
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .remove(name)
    }

    #[cfg(feature = "monty")]
    pub(crate) fn store_repl(&self, name: ScheduleName, session: crate::monty_run::MontySession) {
        self.repls
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .insert(name, session);
    }

    #[cfg(feature = "monty")]
    pub(crate) fn drop_repl(&self, name: &ScheduleName) {
        self.repls
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .remove(name);
    }

    pub(crate) fn name_in_flight(&self, name: &ScheduleName) -> bool {
        self.names
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .contains(name)
    }

    /// Subscribes to the outstanding Monty feed count.
    pub(crate) fn subscribe_feed_level(&self) -> watch::Receiver<usize> {
        self.feed_level.subscribe()
    }

    /// Subscribes to shutdown-deadline observations.
    pub(crate) fn subscribe_shutdown_deadlines(&self) -> watch::Receiver<u64> {
        self.shutdown_deadlines.subscribe()
    }

    /// Records that a shutdown deadline elapsed while a Monty feed was in flight.
    pub(crate) fn note_shutdown_deadline(&self) {
        self.shutdown_deadlines
            .send_modify(|seen| *seen = seen.wrapping_add(1));
    }

    fn lock_feeds(&self) -> std::sync::MutexGuard<'_, ()> {
        self.feed_lock
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    fn publish_feed_level(&self, count: usize) {
        self.feed_level.send_replace(count);
    }
}

/// Increments the outstanding-feed count until dropped.
pub(crate) struct MontyFeedGuard {
    coord: Arc<RunCoord>,
}

impl MontyFeedGuard {
    #[cfg_attr(not(any(test, feature = "monty")), allow(dead_code))]
    pub(crate) fn enter(coord: &Arc<RunCoord>) -> Self {
        {
            let _lock = coord.lock_feeds();
            let count = coord.feeds.fetch_add(1, Ordering::AcqRel) + 1;
            coord.publish_feed_level(count);
        }
        Self {
            coord: Arc::clone(coord),
        }
    }
}

impl Drop for MontyFeedGuard {
    fn drop(&mut self) {
        let _lock = self.coord.lock_feeds();
        let previous = self.coord.feeds.fetch_sub(1, Ordering::AcqRel);
        self.coord.publish_feed_level(previous.saturating_sub(1));
    }
}

/// Removes `name` from the in-flight set when dropped.
pub(crate) struct NameInFlightGuard {
    names: Arc<Mutex<HashSet<ScheduleName>>>,
    name: ScheduleName,
}

impl NameInFlightGuard {
    /// Inserts `name` if it is absent.
    ///
    /// The mutex is released before this returns. `None` means the name is
    /// already marked.
    pub(crate) fn try_insert(
        names: &Arc<Mutex<HashSet<ScheduleName>>>,
        name: &ScheduleName,
    ) -> Option<Self> {
        let mut set = names
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if !set.insert(name.clone()) {
            return None;
        }
        drop(set);
        Some(Self {
            names: Arc::clone(names),
            name: name.clone(),
        })
    }
}

impl Drop for NameInFlightGuard {
    fn drop(&mut self) {
        self.names
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .remove(&self.name);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_panicking_feed_guard_still_drops_the_count() {
        let coord = Arc::new(RunCoord::new());
        let watched = Arc::clone(&coord);
        let panicked = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let _guard = MontyFeedGuard::enter(&watched);
            assert_eq!(watched.feeds.load(Ordering::Acquire), 1);
            assert_eq!(*watched.feed_level.borrow(), 1);
            panic!("feed");
        }));
        assert!(panicked.is_err());
        assert_eq!(coord.feeds.load(Ordering::Acquire), 0);
        assert_eq!(*coord.feed_level.borrow(), 0);
    }

    #[test]
    fn a_panicking_name_guard_still_clears_the_mark() {
        let names = Arc::new(Mutex::new(HashSet::new()));
        let watched = Arc::clone(&names);
        let name = ScheduleName::new("sync").expect("name");
        let panicked = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let _guard = NameInFlightGuard::try_insert(&watched, &name).expect("inserted");
            panic!("mark");
        }));
        assert!(panicked.is_err());
        assert!(
            !names
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .contains(&name)
        );
    }
}
