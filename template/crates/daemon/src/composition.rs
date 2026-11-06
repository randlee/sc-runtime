//! Application-owned assembly shared by production main and composition tests.

#[cfg(feature = "cron")]
use std::sync::Arc;

/// Application extension points for hook reference resolution and Monty hosts.
///
/// Defaults resolve no references and register no hosts. Applications supply
/// their resolver and registry before scheduler construction.
#[cfg(feature = "cron")]
#[derive(Default)]
pub struct CronServices {
    resolver: Option<Arc<dyn sc_cron::EnvironmentReferenceResolver>>,
    job: Option<Arc<dyn sc_cron::Job>>,
    #[cfg(feature = "cron-commands")]
    bindings: Option<Arc<dyn sc_cron_commands::ApplicationTargetBindings>>,
    #[cfg(feature = "cron-commands")]
    executor: Option<Arc<dyn sc_cron_commands::CronExecutor>>,
    capacity: Option<u32>,
    cleanup: Option<ApplicationCleanup>,
    #[cfg(feature = "monty")]
    hosts: sc_cron::MontyHostRegistry,
}

#[cfg(feature = "cron")]
impl CronServices {
    /// Installs the application-owned reference resolver without changing policy.
    #[must_use]
    pub fn with_reference_resolver(mut self, resolver: Arc<dyn sc_cron::EnvironmentReferenceResolver>) -> Self {
        self.resolver = Some(resolver);
        self
    }

    /// Replaces the application Rust job dispatcher before worker startup.
    #[must_use]
    pub fn with_job(mut self, job: Arc<dyn sc_cron::Job>) -> Self {
        self.job = Some(job);
        self
    }

    /// Replaces target authorization; command DTOs and shared native admission remain unchanged.
    #[cfg(feature = "cron-commands")]
    #[must_use]
    pub fn with_target_bindings(mut self, bindings: Arc<dyn sc_cron_commands::ApplicationTargetBindings>) -> Self {
        self.bindings = Some(bindings);
        self
    }

    /// Replaces command dispatch policy. The consumer owns its authorization and physical work.
    #[cfg(feature = "cron-commands")]
    #[must_use]
    pub fn with_executor(mut self, executor: Arc<dyn sc_cron_commands::CronExecutor>) -> Self {
        self.executor = Some(executor);
        self
    }

    /// Installs the owner's physical cleanup for custom command operations/executors.
    /// Runs after UI intake closes and native stop is requested, before native physical join.
    /// The callback must retain and join all its work even when reporting an error.
    #[must_use]
    pub fn with_command_cleanup<F, Fut>(mut self, cleanup: F) -> Self
    where F: FnOnce() -> Fut + Send + 'static,
          Fut: std::future::Future<Output = Result<(), sc_cron::CronError>> + Send + 'static,
    {
        self.cleanup = Some(Box::new(move || Box::pin(cleanup())));
        self
    }

    /// Sets the finite shared scheduler capacity; invalid values fail before worker startup.
    #[must_use]
    pub fn with_command_capacity(mut self, capacity: u32) -> Self {
        self.capacity = Some(capacity);
        self
    }

    /// Installs the completed application host registry; no CLI is implicit.
    #[cfg(feature = "monty")]
    #[must_use]
    pub fn with_monty_hosts(mut self, hosts: sc_cron::MontyHostRegistry) -> Self {
        self.hosts = hosts;
        self
    }

    /// Builds configured policy, attaches application extensions, and opens cron.
    ///
    /// This is the production assembly path called by [`run`].
    ///
    /// # Errors
    /// Returns typed policy, database-path validation, or scheduler-open failures.
    pub fn open_scheduler(self, config: &crate::cron::CronConfig, instance_root: &std::path::Path, logger: sc_command::observability::SharedCanonicalLog)
        -> Result<sc_cron::Scheduler<sc_cron::SqliteScheduleStore, sc_cron::SystemClock, sc_cron::SystemJitter>, sc_cron::CronError> {
        let mut hook_environment = config.hook_environment_host()?;
        if let Some(resolver) = self.resolver {
            hook_environment = hook_environment.with_reference_resolver(resolver);
        }
        let scheduler = crate::cron::open(config, instance_root, logger, hook_environment,
            #[cfg(feature = "monty")] self.hosts,
        )?;
        match self.capacity {
            Some(capacity) => scheduler.with_command_capacity(capacity),
            None => Ok(scheduler),
        }
    }
}

/// Runs the application and joins cron cleanup while the instance lock is held.
///
/// # Errors
/// Returns the exact structured library failure with its original recovery.
pub async fn run(
    config: &crate::AppConfig,
    transport_name: &sc_transport::AppName,
    observability: &crate::observability::Observability,
    #[cfg(feature = "cron")] cron_services: CronServices,
) -> Result<(), String> {
    let targets = crate::store_targets(config, transport_name)
        .map_err(|error| crate::agent_error::transport_error(&error))?;
    #[cfg(feature = "cron")]
    let instance_root = crate::instance_root(config, transport_name)
        .map_err(|error| crate::agent_error::transport_error(&error))?;
    #[cfg(feature = "cron")]
    let cron_config = config.cron.clone();
    #[cfg(feature = "cron")]
    let cron_logger = observability.logger();
    #[cfg(feature = "cron")]
    let (worker_tx, worker_rx) = tokio::sync::oneshot::channel::<Result<CronRuntime, sc_cron::CronError>>();
    #[cfg(feature = "cron")]
    let mut cron_failure = None;
    let result = sc_runtime::Daemon::builder(transport_name, &config.daemon)
        .logger(observability.logger())
        .stores(move || async move {
            let stores = crate::open_stores(targets).await?;
            #[cfg(feature = "cron")]
            {
                // Endpoint validation and daemon.lock precede every database open.
                let runtime = match cron_services.start(&cron_config, &instance_root, cron_logger) {
                    Ok(runtime) => runtime,
                    Err(error) => {
                        let failure = sc_runtime::RuntimeError::stores_failed(
                            error.code(), cron_message(&error), error.recovery(), None,
                        );
                        let _ = worker_tx.send(Err(error));
                        return Err(failure);
                    }
                };
                #[cfg(feature = "cron-commands")]
                let executor = runtime.executor();
                // Cleanup owns the only worker handle even when UI requests are dropped.
                let _ = worker_tx.send(Ok(runtime));
                #[cfg(feature = "cron-commands")]
                return Ok(RuntimeStores { stores, executor });
            }
            #[cfg(not(feature = "cron-commands"))]
            Ok(RuntimeStores { stores })
        })
        .routes(|state: RuntimeStores| {
            let router = crate::routes::router(state.stores);
            #[cfg(feature = "cron-commands")]
            let router = router.merge(crate::cron_commands::router(state.executor));
            router
        })
        {% if mcp %}.mcp(crate::mcp::runtime_service){% endif %}
        ;
    #[cfg(feature = "cron")]
    let result = result.run_with_cleanup(|| async {
        match worker_rx.await {
            Ok(Ok(runtime)) => {
                if let Err(error) = runtime.shutdown().await {
                    cron_failure = Some(error);
                }
            }
            // Startup already returned the native, redacted RuntimeError. It
            // is not a second cleanup failure and no worker was constructed.
            Ok(Err(_)) => {}
            // The SDK drops unused startup closures before early-error cleanup.
            Err(_) => {}
        }
    }).await;
    #[cfg(feature = "cron")]
    return finish_cron_runtime(result, cron_failure, |error| {
        report_cron_failure(&observability.logger(), error);
    });
    #[cfg(not(feature = "cron"))]
    {
        result.run().await.map_err(|error| crate::agent_error::runtime_error(&error))
    }
}

#[cfg(feature = "cron")]
fn cron_error(error: &sc_cron::CronError) -> String {
    crate::agent_error::format_agent_error(error.code(), cron_message(error), error.recovery())
}

#[cfg(feature = "cron")]
fn cron_message(error: &sc_cron::CronError) -> String {
    use sc_cron::CronError;
    match error {
        CronError::BadExpression(_) => "invalid cron expression".to_owned(),
        CronError::InvalidSpec(_) => "invalid cron specification".to_owned(),
        CronError::AlreadyActive(_) => "cron schedule is already active".to_owned(),
        CronError::NotFound(_) => "cron schedule was not found".to_owned(),
        CronError::HookFailed(_) => "cron hook failed".to_owned(),
        CronError::Fenced { .. } => "cron schedule generation is stale".to_owned(),
        CronError::Store(error) => {
            let mut message = format!("cron store operation failed ({})", error.kind().as_str());
            if let Some(outcome) = error.transaction_outcome() {
                // Typed enum/bool facts only; never the provider context or source.
                message.push_str(&format!("; {outcome:?}"));
            }
            message
        }
        _ => "cron operation failed".to_owned(),
    }
}

#[cfg(feature = "cron")]
fn finish_cron_runtime(
    primary: Result<(), sc_runtime::RuntimeError>,
    cleanup: Option<sc_cron::CronError>,
    report_secondary: impl FnOnce(&sc_cron::CronError),
) -> Result<(), String> {
    match (primary, cleanup) {
        (Err(primary), Some(secondary)) => {
            report_secondary(&secondary);
            Err(crate::agent_error::runtime_error(&primary))
        }
        (Err(primary), None) => Err(crate::agent_error::runtime_error(&primary)),
        (Ok(()), Some(error)) => Err(cron_error(&error)),
        (Ok(()), None) => Ok(()),
    }
}

#[cfg(feature = "cron")]
fn report_cron_failure(logger: &sc_command::observability::SharedCanonicalLog, error: &sc_cron::CronError) {
    use sc_observability_types::{ActionName, Level, LogEvent, OBSERVATION_SCHEMA_VERSION,
        ProcessIdentity, TargetCategory, Timestamp};
    let (Ok(target), Ok(action)) = (TargetCategory::new("cron.lifecycle"), ActionName::new("cleanup.error")) else {
        return;
    };
    let event = LogEvent {
        version: (*OBSERVATION_SCHEMA_VERSION).clone(),
        timestamp: Timestamp::now_utc(),
        level: Level::Error,
        service: logger.service_name().clone(),
        target,
        action,
        message: Some(cron_error(error)),
        identity: ProcessIdentity::default(),
        trace: None,
        request_id: None,
        correlation_id: None,
        outcome: None,
        diagnostic: None,
        state_transition: None,
        fields: Default::default(),
    };
    // Diagnostic admission must not replace the primary runtime failure.
    if logger.emit(event).is_err() {
        eprintln!("{}", cron_error(error));
    }
}

/// Application-owned worker lifetime transferred from stores startup to awaited cleanup.
/// Call shutdown after closing intake; dropping this handle is not a physical join.
#[cfg(feature = "cron")]
pub struct CronRuntime<
    S: sc_cron::ScheduleStore = sc_cron::SqliteScheduleStore,
    C: sc_cron::Clock = sc_cron::SystemClock,
    J: sc_cron::JitterSource = sc_cron::SystemJitter,
> {
    scheduler: Arc<sc_cron::Scheduler<S, C, J>>,
    worker: sc_cron::WorkerHandle,
    cleanup: Option<ApplicationCleanup>,
    #[cfg(feature = "cron-commands")]
    executor: Arc<dyn sc_cron_commands::CronExecutor>,
}

// The SDK receives one cloneable root, shared by all selected UI projections.
#[derive(Clone)]
pub(crate) struct RuntimeStores {
    pub(crate) stores: service::Stores,
    #[cfg(feature = "cron-commands")]
    pub(crate) executor: Arc<dyn sc_cron_commands::CronExecutor>,
}

#[cfg(feature = "cron")]
impl CronServices {
    /// Starts the real scheduler, native worker and selected command root.
    /// Call only after acquiring the instance lock. Startup is explicit; no jobs are scheduled.
    ///
    /// # Errors
    /// Returns configuration, open, capacity or cooperative worker startup errors.
    pub fn start(mut self, config: &crate::cron::CronConfig, instance_root: &std::path::Path,
                 logger: sc_command::observability::SharedCanonicalLog) -> Result<CronRuntime, sc_cron::CronError> {
        let opening = Self {
            resolver: self.resolver.take(),
            #[cfg(feature = "monty")]
            hosts: std::mem::take(&mut self.hosts),
            ..Self::default()
        };
        let scheduler = opening.open_scheduler(config, instance_root, logger)?;
        self.start_with_scheduler(scheduler)
    }

    /// Uses a consumer-owned store, clock and jitter through the same selected root.
    /// The consumer configures schema, environment authority, validation and persistence truth.
    /// The store must support cooperative scopes; legacy stores fail explicitly before startup.
    ///
    /// # Errors
    /// Returns invalid capacity or unsupported/cooperative worker startup errors.
    pub fn start_with_scheduler<S, C, J>(mut self, scheduler: sc_cron::Scheduler<S, C, J>)
        -> Result<CronRuntime<S, C, J>, sc_cron::CronError>
    where S: sc_cron::ScheduleStore + 'static, C: sc_cron::Clock + 'static,
          J: sc_cron::JitterSource + 'static,
    {
        let job = self.job.take().unwrap_or_else(|| {
            #[cfg(feature = "cron-commands")]
            { Arc::new(crate::cron_commands::HeartbeatJob) }
            #[cfg(not(feature = "cron-commands"))]
            { Arc::new(crate::cron::ApplicationJob) }
        });
        #[cfg(feature = "cron-commands")]
        let bindings = self.bindings.take().unwrap_or_else(|| Arc::new(crate::cron_commands::RegisteredTargets::default()));
        #[cfg(feature = "cron-commands")]
        let executor_override = self.executor.take();
        let scheduler = match self.capacity {
            Some(capacity) => scheduler.with_command_capacity(capacity)?,
            None => scheduler,
        };
        let scheduler = Arc::new(scheduler);
        let worker = scheduler.spawn_worker_cooperative(job)?;
        #[cfg(feature = "cron-commands")]
        let executor = executor_override.unwrap_or_else(|| Arc::new(sc_cron_commands::StandardCronExecutor::new(
            Arc::clone(&scheduler), bindings, worker.stop_control().operation_context(),
        )));
        Ok(CronRuntime { scheduler, worker, cleanup: self.cleanup, #[cfg(feature = "cron-commands")] executor })
    }
}

#[cfg(feature = "cron")]
impl<S, C, J> CronRuntime<S, C, J>
where S: sc_cron::ScheduleStore + 'static, C: sc_cron::Clock + 'static,
      J: sc_cron::JitterSource + 'static,
{
    /// Shares the native scheduler for application extensions. Scoped command calls
    /// retain this scheduler's one admission controller; legacy methods do not acquire it.
    #[must_use]
    pub fn scheduler(&self) -> Arc<sc_cron::Scheduler<S, C, J>> {
        Arc::clone(&self.scheduler)
    }

    /// Independent stop control; shutdown retains physical join ownership.
    #[must_use]
    pub fn stop_control(&self) -> sc_cron::WorkerStopControl {
        self.worker.stop_control()
    }

    /// The single selected root shared by HTTP, MCP and CLI transport calls.
    #[cfg(feature = "cron-commands")]
    #[must_use]
    pub fn executor(&self) -> Arc<dyn sc_cron_commands::CronExecutor> {
        Arc::clone(&self.executor)
    }

    /// Restricts new admission, then physically joins worker and accepted command work.
    /// Call after stopping UI intake. Already admitted native work can outlive the cutoffs.
    ///
    /// # Errors
    /// Returns native stop or worker join failure; never treats a cutoff as physical completion.
    pub async fn shutdown(self) -> Result<sc_cron::ShutdownReport, sc_cron::CronError> {
        let now = std::time::Instant::now();
        let budget = sc_cron::ShutdownBudget::new(
            now + std::time::Duration::from_secs(5), now + std::time::Duration::from_secs(10),
        )?;
        self.shutdown_with_budget(budget).await
    }

    /// Uses the consumer's checked shutdown cutoffs instead of the starter's five/ten seconds.
    /// Stop intake first. A later request may shorten but cannot extend existing cutoffs.
    /// Every admitted native call and custom cleanup still remains owned until completion.
    ///
    /// # Errors
    /// Returns native stop, custom cleanup or physical worker join errors.
    pub async fn shutdown_with_budget(self, budget: sc_cron::ShutdownBudget)
        -> Result<sc_cron::ShutdownReport, sc_cron::CronError> {
        let stop = self.worker.stop_control().request_stop(budget);
        // Retain both scheduler and worker until actual native completion, even on stop error.
        let custom = match self.cleanup {
            Some(cleanup) => cleanup().await,
            None => Ok(()),
        };
        let joined = self.worker.join_owned().await;
        drop(self.scheduler);
        stop?;
        custom?;
        joined
    }
}

#[cfg(feature = "cron")]
type ApplicationCleanup = Box<dyn FnOnce() -> std::pin::Pin<Box<
    dyn std::future::Future<Output = Result<(), sc_cron::CronError>> + Send,
>> + Send>;

#[cfg(all(test, feature = "cron"))]
mod failure_tests {
    use super::{cron_error, finish_cron_runtime, report_cron_failure};
    use sc_command::observability::{CanonicalLog, SharedCanonicalLog};
    use sc_observability_types::{LogEvent, ServiceName, v2::EventError};
    use std::sync::{Arc, Mutex};

    struct Capture {
        service: ServiceName,
        events: Mutex<Vec<LogEvent>>,
    }

    impl CanonicalLog for Capture {
        fn service_name(&self) -> &ServiceName { &self.service }
        fn emit(&self, event: LogEvent) -> Result<(), EventError> {
            self.events.lock().unwrap().push(event);
            Ok(())
        }
    }

    fn primary() -> sc_runtime::RuntimeError {
        sc_runtime::RuntimeError::stores_failed(
            "STORE.OPEN", "application startup failed", "repair application stores", None,
        )
    }

    fn secondary() -> sc_cron::CronError {
        sc_cron::StoreError::new(sc_cron::StoreErrorKind::Indeterminate, "PRIVATE_PROVIDER_CONTEXT")
            .with_source(std::io::Error::other("PRIVATE_SOURCE_PAYLOAD")).into()
    }

    #[test]
    fn cleanup_keeps_primary_runtime_failure_and_emits_safe_secondary() {
        let capture = Arc::new(Capture {
            service: ServiceName::new("test-cron").unwrap(), events: Mutex::default(),
        });
        let logger: SharedCanonicalLog = capture.clone();
        let expected = crate::agent_error::runtime_error(&primary());
        let finish = |primary, cleanup| finish_cron_runtime(primary, cleanup,
            |error| report_cron_failure(&logger, error));
        assert_eq!(finish(Err(primary()), None), Err(expected.clone()));
        assert!(capture.events.lock().unwrap().is_empty());
        assert_eq!(finish(Err(primary()), Some(secondary())), Err(expected));
        let events = capture.events.lock().unwrap();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].action.as_str(), "cleanup.error");
        assert_eq!(events[0].message.as_deref(), Some(cron_error(&secondary()).as_str()));
        drop(events);
        assert_eq!(finish(Ok(()), Some(secondary())), Err(cron_error(&secondary())));
        assert_eq!(finish(Ok(()), None), Ok(()));
        assert_eq!(capture.events.lock().unwrap().len(), 1);
    }

    #[test]
    fn native_cron_failure_preserves_reconciliation_and_redacts_provider_text() {
        let error = secondary();
        let rendered = cron_error(&error);
        assert!(rendered.starts_with("STORE_FAILED cron store operation failed (indeterminate)"));
        assert!(rendered.ends_with(error.recovery()));
        assert!(rendered.contains("separately authorized operation"));
        assert!(rendered.contains("never replay automatically"));
        assert!(!rendered.contains("PRIVATE_"));
        let error = sc_cron::CronError::InvalidSpec("PRIVATE_ENVIRONMENT".into());
        let rendered = cron_error(&error);
        assert!(rendered.starts_with("INVALID_SPEC "));
        assert!(rendered.ends_with(error.recovery()));
        assert!(!rendered.contains("PRIVATE_"));
    }
}
