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

use crate::ScheduleName;

/// Claims shared by a scheduler, its worker, and `run_due`.
pub(crate) struct RunCoord {
    /// Monty joins whose feed has not yet finished its outcome write.
    pub feeds: Arc<AtomicUsize>,
    /// Schedule names whose `run_due` feed has not finished `finish_due`.
    pub names: Arc<Mutex<HashSet<ScheduleName>>>,
    /// Repls that are not inside a feed, keyed by schedule name.
    #[cfg(feature = "monty")]
    pub(crate) repls: Mutex<HashMap<ScheduleName, crate::monty_run::MontySession>>,
}

impl RunCoord {
    pub(crate) fn new() -> Self {
        Self {
            feeds: Arc::new(AtomicUsize::new(0)),
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
}

/// Increments the outstanding-feed count until dropped.
pub(crate) struct MontyFeedGuard {
    feeds: Arc<AtomicUsize>,
}

impl MontyFeedGuard {
    #[cfg_attr(not(any(test, feature = "monty")), allow(dead_code))]
    pub(crate) fn enter(feeds: &Arc<AtomicUsize>) -> Self {
        feeds.fetch_add(1, Ordering::AcqRel);
        Self {
            feeds: Arc::clone(feeds),
        }
    }
}

impl Drop for MontyFeedGuard {
    fn drop(&mut self) {
        self.feeds.fetch_sub(1, Ordering::AcqRel);
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
        let feeds = Arc::new(AtomicUsize::new(0));
        let watched = Arc::clone(&feeds);
        let panicked = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let _guard = MontyFeedGuard::enter(&watched);
            assert_eq!(watched.load(Ordering::Acquire), 1);
            panic!("feed");
        }));
        assert!(panicked.is_err());
        assert_eq!(feeds.load(Ordering::Acquire), 0);
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
