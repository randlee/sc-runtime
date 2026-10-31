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
    pub fn open_scheduler(self, config: &crate::cron::CronConfig, instance_root: &std::path::Path, logger: sc_observability_types::SharedCanonicalLog)
        -> Result<sc_cron::Scheduler<sc_cron::SqliteScheduleStore, sc_cron::SystemClock, sc_cron::SystemJitter>, sc_cron::CronError> {
        let mut hook_environment = config.hook_environment_host()?;
        if let Some(resolver) = self.resolver {
            hook_environment = hook_environment.with_reference_resolver(resolver);
        }
        crate::cron::open(config, instance_root, logger, hook_environment,
            #[cfg(feature = "monty")] self.hosts,
        )
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
                let scheduler = match cron_services.open_scheduler(
                    &cron_config, &instance_root, cron_logger,
                ) {
                    Ok(scheduler) => scheduler,
                    Err(error) => {
                        // Preserve the typed cron cause across the SDK's stores-error boundary.
                        let _ = worker_tx.send(Err(error));
                        return Err(sc_runtime::RuntimeError::stores_failed(
                            "DAEMON.CRON_OPEN", "cron startup failed", "fix cron configuration or database path and retry", None,
                        ));
                    }
                };
                let worker = scheduler.spawn_worker(std::sync::Arc::new(crate::cron::ApplicationJob));
                let _ = worker_tx.send(Ok(CronRuntime { scheduler, worker }));
            }
            Ok(stores)
        })
        .routes(crate::routes::router)
        {% if mcp %}.mcp(crate::mcp::service){% endif %}
        ;
    #[cfg(feature = "cron")]
    let result = result.run_with_cleanup(|| async {
        match worker_rx.await {
            Ok(Ok(CronRuntime { scheduler, worker })) => {
                let deadline = tokio::time::Instant::now() + std::time::Duration::from_secs(5);
                let _drained = worker.shutdown(deadline).await;
                drop(scheduler);
            }
            Ok(Err(error)) => cron_failure = Some(error),
            // The SDK drops unused startup closures before early-error cleanup.
            Err(_) => {}
        }
    }).await;
    #[cfg(feature = "cron")]
    if let Some(error) = cron_failure {
        return Err(cron_error(error));
    }
    #[cfg(not(feature = "cron"))]
    let result = result.run().await;
    result.map_err(|error| crate::agent_error::runtime_error(&error))
}

#[cfg(feature = "cron")]
fn cron_error(error: sc_cron::CronError) -> String {
    crate::agent_error::format_agent_error("DAEMON.CRON_OPEN", error, "fix cron configuration or database path and retry")
}

// Application-only ownership transferred from stores startup to awaited cleanup.
#[cfg(feature = "cron")]
struct CronRuntime {
    scheduler: sc_cron::Scheduler<sc_cron::SqliteScheduleStore, sc_cron::SystemClock, sc_cron::SystemJitter>,
    worker: sc_cron::WorkerHandle,
}
