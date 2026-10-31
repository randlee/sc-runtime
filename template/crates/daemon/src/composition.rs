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
    let scheduler = {
        let instance_root = crate::instance_root(config, transport_name)
            .map_err(|error| crate::agent_error::transport_error(&error))?;
        std::sync::Arc::new(cron_services.open_scheduler(
            &config.cron, &instance_root, observability.logger(),
        ).map_err(cron_error)?)
    };
    #[cfg(feature = "cron")]
    let scheduler_for_stores = std::sync::Arc::clone(&scheduler);
    #[cfg(feature = "cron")]
    let (worker_tx, worker_rx) = tokio::sync::oneshot::channel();
    let result = sc_runtime::Daemon::builder(transport_name, &config.daemon)
        .logger(observability.logger())
        .stores(move || async move {
            let stores = crate::open_stores(targets).await?;
            #[cfg(feature = "cron")]
            {
                // The runtime holds daemon.lock before invoking this closure.
                let worker = scheduler_for_stores.spawn_worker(std::sync::Arc::new(crate::cron::ApplicationJob));
                let _ = worker_tx.send(worker);
            }
            Ok(stores)
        })
        .routes(crate::routes::router)
        {% if mcp %}.mcp(crate::mcp::service){% endif %}
        ;
    #[cfg(feature = "cron")]
    let result = result.run_with_cleanup(move || async move {
        let deadline = tokio::time::Instant::now() + std::time::Duration::from_secs(5);
        if let Ok(worker) = worker_rx.await {
            let _drained = worker.shutdown(deadline).await;
        }
        drop(scheduler);
    }).await;
    #[cfg(not(feature = "cron"))]
    let result = result.run().await;
    result.map_err(|error| crate::agent_error::runtime_error(&error))
}

#[cfg(feature = "cron")]
fn cron_error(error: sc_cron::CronError) -> String {
    crate::agent_error::format_agent_error("DAEMON.CRON_OPEN", error, "fix cron configuration or database path and retry")
}
