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
        let (code, recovery) = match self {
            Self::TimedOut => return RunOutcome::TimedOut,
            Self::ProcessInvalid => (
                "ENV_PROCESS_INVALID",
                "Restart the daemon with a bounded UTF-8 value or remove the name from the allowlist.",
            ),
            Self::FileInvalid => (
                "ENV_FILE_INVALID",
                "Fix the bounded data-file syntax. Shell expansion is not supported.",
            ),
            Self::FileMissing => (
                "ENV_FILE_MISSING",
                "Restore the absolute file or mark the source optional.",
            ),
            Self::FileDenied => (
                "ENV_FILE_DENIED",
                "Authorize the canonical file/root or remove the schedule source.",
            ),
            Self::Reference => (
                "ENV_REFERENCE",
                "Configure the daemon resolver and restore the referenced value.",
            ),
            Self::SnapshotLimit => (
                "ENV_SNAPSHOT_LIMIT",
                "Remove variables or reduce values before retrying.",
            ),
        };
        RunOutcome::Failed(format!(
            "{code}: hook environment could not be resolved; {recovery}"
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
    fn check_limit(&self) -> Result<(), EnvironmentFailure> {
        check_block(self.entries.iter().map(|(n, v)| (n.as_str(), v.as_str())))
    }
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
        check_deadline(clock, deadline)?;
        let mut snapshot = self.process_snapshot()?;
        self.apply_files(clock, &schedule.spec.environment, deadline, &mut snapshot)
            .await?;
        self.apply_operations(clock, schedule, deadline, &mut snapshot)
            .await?;
        check_deadline(clock, deadline)?;
        Ok(snapshot)
    }

    fn process_snapshot(&self) -> Result<EnvironmentSnapshot, EnvironmentFailure> {
        let mut snapshot = EnvironmentSnapshot::default();
        for name in &self.process {
            if let Some(value) = std::env::var_os(name.as_str()) {
                snapshot.insert(
                    name,
                    value
                        .into_string()
                        .map_err(|_| EnvironmentFailure::ProcessInvalid)?,
                    EnvironmentFailure::ProcessInvalid,
                )?;
            }
        }
        snapshot.check_limit()?;
        Ok(snapshot)
    }

    async fn apply_files<C: Clock + ?Sized>(
        &self,
        clock: &C,
        policy: &HookEnvironment,
        deadline: Instant,
        snapshot: &mut EnvironmentSnapshot,
    ) -> Result<(), EnvironmentFailure> {
        for source in &policy.sources {
            let source = source.clone();
            let host = self.clone();
            let work = tokio::task::spawn_blocking(move || dotenv::read(&source, &host));
            let result = tokio::select! {
                entries = work => entries,
                () = crate::clock::sleep_until_monotonic(clock, deadline) => return Err(EnvironmentFailure::TimedOut),
            };
            check_deadline(clock, deadline)?;
            let entries = result.map_err(|_| EnvironmentFailure::FileInvalid)??;
            for (name, value) in entries {
                snapshot.insert(&name, value, EnvironmentFailure::FileInvalid)?;
            }
            snapshot.check_limit()?;
        }
        Ok(())
    }

    async fn apply_operations<C: Clock + ?Sized>(
        &self,
        clock: &C,
        schedule: &Schedule,
        deadline: Instant,
        snapshot: &mut EnvironmentSnapshot,
    ) -> Result<(), EnvironmentFailure> {
        for (name, operation) in &schedule.spec.environment.variables {
            check_deadline(clock, deadline)?;
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
                    let result = self
                        .reference_value(clock, &schedule.spec.name, name, reference, deadline)
                        .await?;
                    snapshot.insert(name, result, EnvironmentFailure::Reference)?;
                }
            }
        }
        snapshot.check_limit()
    }

    async fn reference_value<C: Clock + ?Sized>(
        &self,
        clock: &C,
        schedule: &ScheduleName,
        variable: &EnvName,
        reference: &str,
        deadline: Instant,
    ) -> Result<String, EnvironmentFailure> {
        let resolver = self
            .resolver
            .as_ref()
            .ok_or(EnvironmentFailure::Reference)?;
        let request = EnvironmentReferenceRequest {
            schedule,
            variable,
            reference,
            deadline,
        };
        let result = tokio::select! {
            result = resolver.resolve(request) => result,
            () = crate::clock::sleep_until_monotonic(clock, deadline) => return Err(EnvironmentFailure::TimedOut),
        };
        check_deadline(clock, deadline)?;
        result
            .map_err(|_| EnvironmentFailure::Reference)?
            .ok_or(EnvironmentFailure::Reference)
    }
}
fn check_deadline<C: Clock + ?Sized>(
    clock: &C,
    deadline: Instant,
) -> Result<(), EnvironmentFailure> {
    if clock.monotonic_now() >= deadline {
        Err(EnvironmentFailure::TimedOut)
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolution_failure_recovery_matches_the_documented_operator_action() {
        for (failure, code, recovery) in [
            (
                EnvironmentFailure::ProcessInvalid,
                "ENV_PROCESS_INVALID",
                "Restart the daemon with a bounded UTF-8 value or remove the name from the allowlist.",
            ),
            (
                EnvironmentFailure::FileDenied,
                "ENV_FILE_DENIED",
                "Authorize the canonical file/root or remove the schedule source.",
            ),
            (
                EnvironmentFailure::FileMissing,
                "ENV_FILE_MISSING",
                "Restore the absolute file or mark the source optional.",
            ),
            (
                EnvironmentFailure::FileInvalid,
                "ENV_FILE_INVALID",
                "Fix the bounded data-file syntax. Shell expansion is not supported.",
            ),
            (
                EnvironmentFailure::Reference,
                "ENV_REFERENCE",
                "Configure the daemon resolver and restore the referenced value.",
            ),
            (
                EnvironmentFailure::SnapshotLimit,
                "ENV_SNAPSHOT_LIMIT",
                "Remove variables or reduce values before retrying.",
            ),
        ] {
            let outcome = failure.outcome();
            let parts = outcome.failure_parts().expect("structured failure");
            assert_eq!(parts.code, Some(code));
            assert_eq!(parts.recovery, Some(recovery));
        }
        assert_eq!(EnvironmentFailure::TimedOut.outcome(), RunOutcome::TimedOut);
    }
}
