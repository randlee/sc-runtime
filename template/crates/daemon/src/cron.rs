//! Schedule row limit read from the application's sc-config.
//!
//! Build administrator policy with `CronConfig::hook_environment_host`, attach
//! the application resolver, then pass that same prebuilt object to `open`.
//! JSON null/zero row limits remain unlimited; absent limits default to ten.

use std::num::NonZeroUsize;

use serde::Deserialize;

/// `cron` object in the application configuration file.
#[derive(Debug, Deserialize)]
pub struct CronConfig {
    /// Relative schedule database path below the instance root; no parent traversal.
    #[serde(default = "default_database_path")]
    pub database_path: std::path::PathBuf,
    /// Maximum schedule rows, enabled and disabled.
    ///
    /// [`None`] is JSON `null` and means unlimited. `Some(0)` is also
    /// unlimited. `Some(n)` for `n > 0` caps the table at `n`. When the key
    /// is absent the value is `Some(10)`.
    #[serde(default = "default_max_schedule_rows")]
    pub max_schedule_rows: Option<u64>,
    /// Daemon policy, empty by default; values are never captured here.
    #[serde(default)]
    pub environment: CronEnvironmentConfig,
}

fn default_database_path() -> std::path::PathBuf {
    "cron.db".into()
}

fn default_max_schedule_rows() -> Option<u64> {
    Some(10)
}

impl Default for CronConfig {
    fn default() -> Self {
        Self {
            database_path: default_database_path(),
            max_schedule_rows: default_max_schedule_rows(),
            environment: CronEnvironmentConfig::default(),
        }
    }
}

impl CronConfig {
    /// Maps `max_schedule_rows` to the `ScheduleLimits` field of the same name.
    ///
    /// JSON `null` and `0` are [`None`]. Any other value that fits in
    /// [`usize`] is [`Some`]. A value that does not fit becomes
    /// [`usize::MAX`], which still caps the table. This is the only mapping
    /// from the sc-config integer. `sc-cron` receives the resulting
    /// `ScheduleLimits` and does not read sc-config.
    pub fn row_limit(&self) -> Option<NonZeroUsize> {
        let rows = self.max_schedule_rows?;
        let width = usize::try_from(rows).unwrap_or(usize::MAX);
        NonZeroUsize::new(width)
    }

    /// Values to pass into `sc_cron::ScheduleLimits::with_max_schedule_rows`.
    ///
    /// Keeps sc-config mapping in this module so layouts do not duplicate `0` / JSON `null` rules.
    /// This module does not depend on `sc-cron`, so those paths are not doc links.
    #[must_use]
    pub fn schedule_limits_fields(&self) -> CronScheduleLimitsFields {
        CronScheduleLimitsFields {
            max_schedule_rows: self.row_limit(),
        }
    }
}

/// Administrator-owned environment authorization, independent of schedule data.
#[derive(Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CronEnvironmentConfig {
    /// Names inherited from the daemon only at occurrence execution.
    #[serde(default)]
    pub allow_process: Vec<String>,
    /// Individually authorized absolute dotenv files.
    #[serde(default, deserialize_with = "absolute_paths")]
    pub allow_dotenv_files: Vec<std::path::PathBuf>,
    /// Absolute dotenv directory roots, authorizing their descendants.
    #[serde(default, deserialize_with = "absolute_paths")]
    pub allow_dotenv_roots: Vec<std::path::PathBuf>,
}
impl std::fmt::Debug for CronEnvironmentConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CronEnvironmentConfig")
            .field("allow_process", &self.allow_process)
            .field("file_count", &self.allow_dotenv_files.len())
            .field("root_count", &self.allow_dotenv_roots.len())
            .finish()
    }
}
fn absolute_paths<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<Vec<std::path::PathBuf>, D::Error> {
    let paths = Vec::<String>::deserialize(deserializer)?;
    paths
        .into_iter()
        .map(|text| {
            let path = std::path::PathBuf::from(&text);
            if !path.is_absolute()
                || text.contains('\0')
                || text.len() > 4096
                || path
                    .components()
                    .any(|c| c == std::path::Component::ParentDir)
            {
                return Err(serde::de::Error::custom(
                    "dotenv authorization requires an absolute normalized path",
                ));
            }
            Ok(path)
        })
        .collect()
}

#[cfg(feature = "cron")]
impl CronConfig {
    /// Builds typed policy while keeping application reference resolution separate.
    ///
    /// # Errors
    /// Invalid configured names or paths return sc-cron's value-free InvalidSpec.
    pub fn hook_environment_host(
        &self,
    ) -> Result<sc_cron::HookEnvironmentHost, sc_cron::CronError> {
        let mut host =
            sc_cron::HookEnvironmentHost::new().allow_process(&self.environment.allow_process)?;
        for path in &self.environment.allow_dotenv_files {
            host = host.allow_dotenv_file(path)?;
        }
        for path in &self.environment.allow_dotenv_roots {
            host = host.allow_dotenv_root(path)?;
        }
        Ok(host)
    }
}

/// Opens a scheduler with the exact prebuilt administrator policy/resolver.
///
/// # Errors
/// Returns schedule database open or configuration errors from sc-cron.
#[cfg(feature = "cron")]
pub fn open(
    config: &CronConfig,
    instance_root: &std::path::Path,
    logger: sc_observability_types::SharedCanonicalLog,
    hook_environment: sc_cron::HookEnvironmentHost,
    #[cfg(feature = "monty")] monty_hosts: sc_cron::MontyHostRegistry,
) -> Result<
    sc_cron::Scheduler<sc_cron::SqliteScheduleStore, sc_cron::SystemClock, sc_cron::SystemJitter>,
    sc_cron::CronError,
> {
    // Validate before creating the instance directory or opening any database.
    let configured = &config.database_path;
    if configured.as_os_str().is_empty()
        || configured
            .components()
            .any(|component| !matches!(component, std::path::Component::Normal(_)))
        || configured.to_string_lossy().contains(['\\', ':'])
    {
        return Err(sc_cron::CronError::InvalidSpec(
            "cron.database_path must be a relative path without parent traversal".to_owned(),
        ));
    }
    std::fs::create_dir_all(instance_root).map_err(|error| {
        sc_cron::CronError::Store(
            sc_cron::StoreError::new(
                sc_cron::StoreErrorKind::Unavailable,
                "failed to prepare cron instance root",
            )
            .with_source(error),
        )
    })?;
    let database_path = instance_root.join(configured);
    let mut component_path = instance_root.to_path_buf();
    for component in configured.components() {
        component_path.push(component.as_os_str());
        if std::fs::symlink_metadata(&component_path)
            .is_ok_and(|metadata| metadata.file_type().is_symlink())
        {
            return Err(sc_cron::CronError::InvalidSpec(
                "cron.database_path must not traverse symlinks".to_owned(),
            ));
        }
    }
    let limits = sc_cron::ScheduleLimits::with_max_schedule_rows(config.row_limit());
    let scheduler = sc_cron::Scheduler::open_with_logger(
        database_path,
        limits,
        std::sync::Arc::new(sc_cron::SystemClock),
        logger,
    )?
    .with_hook_environment(hook_environment);
    #[cfg(feature = "monty")]
    let scheduler = scheduler.with_monty_hosts(monty_hosts);
    Ok(scheduler)
}

/// Application Rust-job extension point; Python and Monty use their stored hooks.
///
/// Replace this implementation when the application registers Rust schedules.
#[cfg(feature = "cron")]
pub struct ApplicationJob;

#[cfg(feature = "cron")]
#[async_trait::async_trait]
impl sc_cron::Job for ApplicationJob {
    async fn run(
        &self,
        _payload: &serde_json::Value,
        _deadline: std::time::Instant,
    ) -> sc_cron::RunOutcome {
        sc_cron::RunOutcome::Refused("no application Rust job is registered".to_owned())
    }

    async fn should_stop(&self, _payload: &serde_json::Value) -> sc_cron::StopDecision {
        sc_cron::StopDecision::Continue
    }
}

/// Row-cap fields for `sc_cron::ScheduleLimits` (see [`CronConfig::schedule_limits_fields`]).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CronScheduleLimitsFields {
    /// Same meaning as `sc_cron::ScheduleLimits::max_schedule_rows`.
    pub max_schedule_rows: Option<NonZeroUsize>,
}

#[cfg(test)]
mod tests {
    use super::CronConfig;

    #[cfg(feature = "cron")]
    #[test]
    fn absolute_and_parent_database_paths_are_rejected_before_side_effects() {
        use std::sync::Arc;
        let root = tempfile::tempdir().expect("root");
        let outside = root.path().join("outside.db");
        for database_path in [
            outside.clone(),
            "../outside.db".into(),
            "nested/../../outside.db".into(),
            "C:\\outside.db".into(),
        ] {
            let instance = root.path().join("absent-instance");
            let config = CronConfig {
                database_path,
                ..CronConfig::default()
            };
            let result = super::open(
                &config,
                &instance,
                Arc::new(sc_observability_types::NoopCanonicalLog::new(
                    sc_observability_types::ServiceName::new("path-proof").expect("service"),
                )),
                config.hook_environment_host().expect("policy"),
                #[cfg(feature = "monty")]
                sc_cron::MontyHostRegistry::new(),
            );
            assert!(matches!(result, Err(sc_cron::CronError::InvalidSpec(_))));
            assert!(
                !instance.exists(),
                "invalid path must not create the instance root"
            );
            assert!(
                !outside.exists(),
                "invalid path must not create an outside database"
            );
        }
    }

    #[cfg(feature = "cron")]
    #[test]
    fn configured_database_and_row_limits_reach_store() {
        use std::sync::Arc;
        use std::time::Duration;

        for (value, capped) in [
            (serde_json::json!(null), false),
            (serde_json::json!(0), false),
            (serde_json::json!(1), true),
        ] {
            let root = tempfile::tempdir().expect("instance root");
            let config: CronConfig = serde_json::from_value(serde_json::json!({
                "database_path": "configured.db", "max_schedule_rows": value,
            }))
            .expect("config");
            let scheduler = super::open(
                &config,
                root.path(),
                Arc::new(sc_observability_types::NoopCanonicalLog::new(
                    sc_observability_types::ServiceName::new("limit-proof").expect("service"),
                )),
                config.hook_environment_host().expect("policy"),
                #[cfg(feature = "monty")]
                sc_cron::MontyHostRegistry::new(),
            )
            .expect("open");
            assert!(root.path().join("configured.db").is_file());
            let spec = |name| {
                sc_cron::ScheduleSpec::new(
                    name,
                    "* * * * *",
                    Duration::from_secs(1),
                    serde_json::json!({}),
                )
                .expect("spec")
            };
            scheduler.start(spec("first")).expect("first row");
            let second = scheduler.start(spec("second"));
            if capped {
                assert!(matches!(second, Err(sc_cron::CronError::InvalidSpec(_))));
            } else {
                second.expect("unlimited row");
            }
        }
    }

    #[test]
    fn zero_is_unlimited_and_the_default_is_ten() {
        let missing: CronConfig = serde_json::from_str("{}").expect("json");
        assert_eq!(missing.max_schedule_rows, Some(10));
        assert_eq!(
            missing.row_limit().map(std::num::NonZeroUsize::get),
            Some(10)
        );

        let zero: CronConfig = serde_json::from_str(r#"{"max_schedule_rows":0}"#).expect("json");
        assert_eq!(zero.max_schedule_rows, Some(0));
        assert!(zero.row_limit().is_none());

        let raised: CronConfig = serde_json::from_str(r#"{"max_schedule_rows":25}"#).expect("json");
        assert_eq!(
            raised.row_limit().map(std::num::NonZeroUsize::get),
            Some(25)
        );
    }

    #[test]
    fn json_null_is_unlimited() {
        let unlimited: CronConfig =
            serde_json::from_str(r#"{"max_schedule_rows": null}"#).expect("json");
        assert!(unlimited.max_schedule_rows.is_none());
        assert!(unlimited.row_limit().is_none());
    }

    #[test]
    fn schedule_limits_fields_match_row_limit() {
        let config: CronConfig = serde_json::from_str(r#"{"max_schedule_rows":25}"#).expect("json");
        let fields = config.schedule_limits_fields();
        assert_eq!(
            fields.max_schedule_rows.map(std::num::NonZeroUsize::get),
            Some(25)
        );
    }
}

#[cfg(all(test, feature = "cron"))]
mod environment_tests {
    use super::*;
    use sc_cron::{
        EnvironmentReferenceError, EnvironmentReferenceRequest, EnvironmentReferenceResolver,
        ScheduleStore,
    };
    use std::sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    };

    struct RecordingResolver(AtomicUsize);
    #[async_trait::async_trait]
    impl EnvironmentReferenceResolver for RecordingResolver {
        async fn resolve(
            &self,
            request: EnvironmentReferenceRequest<'_>,
        ) -> Result<Option<String>, EnvironmentReferenceError> {
            assert_eq!(request.schedule().as_str(), "resolver-preserved");
            assert_eq!(request.variable().as_str(), "OTHER_API_KEY");
            assert_eq!(request.reference(), "recording://example");
            self.0.fetch_add(1, Ordering::SeqCst);
            Err(EnvironmentReferenceError::Denied)
        }
    }
    struct Idle;
    #[async_trait::async_trait]
    impl sc_cron::Job for Idle {
        async fn should_stop(&self, _: &serde_json::Value) -> sc_cron::StopDecision {
            sc_cron::StopDecision::Continue
        }
        async fn run(&self, _: &serde_json::Value, _: std::time::Instant) -> sc_cron::RunOutcome {
            sc_cron::RunOutcome::Ok
        }
    }

    #[tokio::test]
    async fn prebuilt_policy_retains_application_reference_resolver_through_open() {
        let config = CronConfig::default();
        let host = config
            .hook_environment_host()
            .expect("empty administrator policy");
        assert_eq!(host.process_names().len(), 0);
        assert_eq!(host.dotenv_files().len(), 0);
        assert_eq!(host.dotenv_roots().len(), 0);
        let resolver = Arc::new(RecordingResolver(AtomicUsize::new(0)));
        let host = host.with_reference_resolver(resolver.clone());
        assert!(host.has_reference_resolver());
        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("time")
            .as_nanos();
        let path =
            std::env::temp_dir().join(format!("cron-policy-{}-{nonce}.db", std::process::id()));
        let logger = Arc::new(sc_observability_types::NoopCanonicalLog::new(
            sc_observability_types::ServiceName::new("cron-policy-test").expect("service"),
        ));
        let mut config = config;
        config.database_path = path.file_name().expect("file name").into();
        let scheduler = open(
            &config,
            &std::env::temp_dir(),
            logger,
            host,
            #[cfg(feature = "monty")]
            sc_cron::MontyHostRegistry::new(),
        )
        .expect("open");
        let now = sc_cron::Timestamp::from_datetime(sc_cron::Clock::now(&sc_cron::SystemClock));
        let spec = sc_cron::ScheduleSpec::with_hook(
            "resolver-preserved",
            "* * * * *",
            std::time::Duration::from_secs(20),
            serde_json::json!({}),
            sc_cron::HookTarget::python("unexecuted.py").expect("target"),
        )
        .expect("spec")
        .with_environment(
            sc_cron::HookEnvironment::new()
                .reference("OTHER_API_KEY", "recording://example")
                .expect("reference"),
        )
        .expect("environment");
        scheduler
            .store()
            .create(
                &sc_cron::NewSchedule {
                    spec,
                    jitter: std::time::Duration::ZERO,
                    next_run_at: now,
                },
                now,
            )
            .expect("due row");
        let runs = scheduler.run_due(Arc::new(Idle)).await.expect("run due");
        assert_eq!(
            runs[0].outcome.failure_parts().and_then(|p| p.code),
            Some("ENV_REFERENCE")
        );
        assert_eq!(resolver.0.load(Ordering::SeqCst), 1);
        drop(scheduler);
        std::fs::remove_file(path).expect("remove test database");
    }

    #[test]
    fn exact_file_and_root_policy_remain_separate_and_paths_are_absolute() {
        let root = std::env::temp_dir();
        let file = root.join("agent.env");
        let config: CronConfig = serde_json::from_value(serde_json::json!({"environment": {"allow_process": ["JEV_API_KEY"], "allow_dotenv_files": [file], "allow_dotenv_roots": [root]}})).expect("config");
        let host = config.hook_environment_host().expect("host");
        assert_eq!(
            host.dotenv_files().collect::<Vec<_>>(),
            vec![file.as_path()]
        );
        assert_eq!(
            host.dotenv_roots().collect::<Vec<_>>(),
            vec![root.as_path()]
        );
        assert!(
            serde_json::from_value::<CronConfig>(
                serde_json::json!({"environment": {"allow_dotenv_files": ["relative.env"]}})
            )
            .is_err()
        );
    }
}
