//! One immutable environment snapshot per occurrence, shared by every adapter.
use std::ffi::OsString;

use super::*;
use crate::{Clock, RunOutcome, Schedule};

#[derive(Clone, Copy, Debug)]
pub(super) enum EnvironmentFailure {
    ProcessInvalid,
    FileInvalid,
    FileMissing,
    FileDenied,
    Reference,
    SnapshotLimit,
    TimedOut,
}
impl EnvironmentFailure {
    fn outcome(self) -> RunOutcome {
        let code = match self {
            Self::TimedOut => return RunOutcome::TimedOut,
            Self::ProcessInvalid => "ENV_PROCESS_INVALID",
            Self::FileInvalid => "ENV_FILE_INVALID",
            Self::FileMissing => "ENV_FILE_MISSING",
            Self::FileDenied => "ENV_FILE_DENIED",
            Self::Reference => "ENV_REFERENCE",
            Self::SnapshotLimit => "ENV_SNAPSHOT_LIMIT",
        };
        RunOutcome::Failed(format!(
            "{code}: hook environment could not be resolved; Check daemon environment policy and schedule configuration."
        ))
    }
}

pub(super) fn check_block<'a>(
    entries: impl Iterator<Item = (&'a str, &'a str)>,
) -> Result<(), EnvironmentFailure> {
    let mut count = 0;
    let mut bytes = 1usize; // final block terminator in addition to each entry
    for (name, value) in entries {
        count += 1;
        bytes = bytes.saturating_add(name.len() + value.len() + 2);
        if count > MAX_ENV_ENTRIES || bytes > MAX_ENV_SNAPSHOT_BYTES {
            return Err(EnvironmentFailure::SnapshotLimit);
        }
    }
    Ok(())
}

/// Values are deliberately accessible only inside runtime delivery code.
#[derive(Clone, Default)]
pub(crate) struct EnvironmentSnapshot {
    entries: BTreeMap<String, String>,
    spellings: BTreeMap<String, String>,
}
impl fmt::Debug for EnvironmentSnapshot {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("EnvironmentSnapshot")
            .field("names", &self.entries.keys())
            .finish()
    }
}
impl EnvironmentSnapshot {
    pub(crate) fn child_env(&self) -> crate::process::ChildEnv {
        crate::process::ChildEnv::Only {
            keep: vec![],
            set: self
                .entries
                .iter()
                .map(|(n, v)| (OsString::from(n), OsString::from(v)))
                .collect(),
        }
    }
    #[cfg(feature = "monty")]
    pub(crate) fn get(&self, name: &str) -> Option<&str> {
        #[cfg(windows)]
        let name = &name.to_ascii_uppercase();
        self.entries.get(name).map(String::as_str)
    }
    #[cfg(feature = "monty")]
    pub(crate) fn iter(&self) -> impl Iterator<Item = (&str, &str)> {
        self.entries.iter().map(|(n, v)| (n.as_str(), v.as_str()))
    }
    pub(crate) fn redact(&self, detail: &str) -> String {
        let mut values: Vec<_> = self
            .entries
            .values()
            .filter(|value| !value.is_empty())
            .map(String::as_str)
            .collect();
        values.sort_unstable_by(|a, b| b.len().cmp(&a.len()).then_with(|| a.cmp(b)));
        values.dedup();
        // Scan the original once so short values cannot corrupt a replacement
        // marker inserted while replacing a longer value.
        let mut out = String::new();
        let mut at = 0;
        while at < detail.len() {
            if let Some(value) = values
                .iter()
                .find(|value| detail[at..].starts_with(**value))
            {
                out.push_str("[REDACTED_ENV]");
                at += value.len();
            } else if let Some(ch) = detail[at..].chars().next() {
                out.push(ch);
                at += ch.len_utf8();
            }
        }
        if out.len() > crate::MAX_HOOK_OUTPUT_BYTES {
            let mut end = crate::MAX_HOOK_OUTPUT_BYTES;
            while !out.is_char_boundary(end) {
                end -= 1;
            }
            out.truncate(end);
        }
        out
    }
    fn check_spelling(
        &mut self,
        name: &EnvName,
        invalid: EnvironmentFailure,
    ) -> Result<(), EnvironmentFailure> {
        let folded = name.0.to_ascii_uppercase();
        if self
            .spellings
            .get(&folded)
            .is_some_and(|old| old != name.as_str())
        {
            return Err(invalid);
        }
        self.spellings.insert(folded, name.0.clone());
        Ok(())
    }
    fn insert(
        &mut self,
        name: &EnvName,
        value: String,
        invalid: EnvironmentFailure,
    ) -> Result<(), EnvironmentFailure> {
        if !text_valid(&value, MAX_ENV_VALUE_BYTES) {
            return Err(invalid);
        }
        #[cfg(windows)]
        let key = name.0.to_ascii_uppercase();
        #[cfg(not(windows))]
        let key = name.0.clone();
        // Cross-layer spelling changes are also rejected on Unix, where they
        // would otherwise create a different map than on Windows.
        self.check_spelling(name, invalid)?;
        self.entries.insert(key, value);
        Ok(())
    }
}

impl HookEnvironmentHost {
    pub(crate) async fn snapshot<C: Clock + ?Sized>(
        &self,
        clock: &C,
        schedule: &Schedule,
        deadline: Instant,
    ) -> Result<Arc<EnvironmentSnapshot>, RunOutcome> {
        if let Err(error) = schedule.spec.validate_environment() {
            return Err(RunOutcome::Failed(error.recorded()));
        }
        self.resolve_snapshot(clock, schedule, deadline)
            .await
            .map(Arc::new)
            .map_err(EnvironmentFailure::outcome)
    }
    async fn resolve_snapshot<C: Clock + ?Sized>(
        &self,
        clock: &C,
        schedule: &Schedule,
        deadline: Instant,
    ) -> Result<EnvironmentSnapshot, EnvironmentFailure> {
        let check_deadline = || {
            if clock.monotonic_now() >= deadline {
                Err(EnvironmentFailure::TimedOut)
            } else {
                Ok(())
            }
        };
        check_deadline()?;
        let mut snapshot = EnvironmentSnapshot::default();
        for name in &self.process {
            match std::env::var_os(name.as_str()) {
                Some(value) => snapshot.insert(
                    name,
                    value
                        .into_string()
                        .map_err(|_| EnvironmentFailure::ProcessInvalid)?,
                    EnvironmentFailure::ProcessInvalid,
                )?,
                None => continue,
            }
        }
        check_block(
            snapshot
                .entries
                .iter()
                .map(|(n, v)| (n.as_str(), v.as_str())),
        )?;
        for source in &schedule.spec.environment.sources {
            let source = source.clone();
            let host = self.clone();
            let work = tokio::task::spawn_blocking(move || dotenv::read(&source, &host));
            let entries = tokio::select! {
                entries = work => entries.map_err(|_| EnvironmentFailure::FileInvalid)??,
                () = crate::clock::sleep_until_monotonic(clock, deadline) => return Err(EnvironmentFailure::TimedOut),
            };
            check_deadline()?;
            for (name, value) in entries {
                snapshot.insert(&name, value, EnvironmentFailure::FileInvalid)?;
            }
            check_block(
                snapshot
                    .entries
                    .iter()
                    .map(|(n, v)| (n.as_str(), v.as_str())),
            )?;
        }
        for (name, operation) in &schedule.spec.environment.variables {
            check_deadline()?;
            match operation {
                Operation::Set(value) => {
                    snapshot.insert(name, value.clone(), EnvironmentFailure::SnapshotLimit)?
                }
                Operation::Clear => {
                    snapshot.check_spelling(name, EnvironmentFailure::SnapshotLimit)?;
                    #[cfg(windows)]
                    let key = name.0.to_ascii_uppercase();
                    #[cfg(not(windows))]
                    let key = name.0.clone();
                    snapshot.entries.remove(&key);
                }
                Operation::Reference(reference) => {
                    let resolver = self
                        .resolver
                        .as_ref()
                        .ok_or(EnvironmentFailure::Reference)?;
                    let request = EnvironmentReferenceRequest {
                        schedule: &schedule.spec.name,
                        variable: name,
                        reference,
                        deadline,
                    };
                    let result = tokio::select! {
                        result = resolver.resolve(request) => result.map_err(|_| EnvironmentFailure::Reference)?.ok_or(EnvironmentFailure::Reference)?,
                        () = crate::clock::sleep_until_monotonic(clock, deadline) => return Err(EnvironmentFailure::TimedOut),
                    };
                    check_deadline()?;
                    snapshot.insert(name, result, EnvironmentFailure::Reference)?;
                }
            }
        }
        check_block(
            snapshot
                .entries
                .iter()
                .map(|(n, v)| (n.as_str(), v.as_str())),
        )?;
        check_deadline()?;
        Ok(snapshot)
    }
}
