//! Daemon process. Loads config, starts logging, then builds the daemon.

use std::path::PathBuf;
use std::process::ExitCode;

{% if mcp %}mod mcp;{% endif %}

const APP_NAME: &str = "{{project-name}}";

#[tokio::main]
async fn main() -> ExitCode {
    let config_name = match sc_config::AppName::new(APP_NAME) {
        Ok(name) => name,
        Err(_) => return ExitCode::from(1),
    };
    let config = match sc_config::load::<daemon::AppConfig>(config_name) {
        Ok(config) => config,
        Err(_) => return ExitCode::from(1),
    };
    let transport_name = match sc_transport::AppName::new(APP_NAME) {
        Ok(name) => name,
        Err(_) => return ExitCode::from(1),
    };
    let log_root = match log_root(&config, &transport_name) {
        Ok(path) => path,
        Err(_) => return ExitCode::from(1),
    };
    let logger = match init_observability(&config.logging, log_root) {
        Ok(logger) => logger,
        Err(_) => return ExitCode::from(1),
    };
    let database = match daemon::database_path(&config, &transport_name) {
        Ok(path) => path,
        Err(_) => return ExitCode::from(1),
    };
    let result = sc_runtime::Daemon::builder(&transport_name, &config.daemon)
        .stores(move || service::open_stores(database))
        .routes(daemon::routes::router)
        {% if mcp %}.mcp(mcp::service){% endif %}
        .run()
        .await;
    drop(logger);
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(_) => ExitCode::from(1),
    }
}

fn log_root(config: &daemon::AppConfig, app: &sc_transport::AppName) -> Result<PathBuf, String> {
    if let Some(root) = &config.logging.log_root {
        return Ok(root.clone());
    }
    if let Some(root) = &config.daemon.transport.instance_root {
        return Ok(root.clone());
    }
    sc_transport::instance_root(app).map_err(|err| err.to_string())
}

fn init_observability(
    logging: &sc_observability::LogSettings,
    log_root: PathBuf,
) -> Result<sc_observability::v2::Logger, String> {
    let service = sc_observability::ServiceName::new(APP_NAME).map_err(|err| err.to_string())?;
    let mut settings = sc_observability::LoggerConfig::default_for(service, log_root);
    if let Some(level) = logging.level {
        settings.level = level;
    }
    if let Some(enabled) = logging.enable_file_sink {
        settings.enable_file_sink = enabled;
    }
    if let Some(enabled) = logging.enable_console_sink {
        settings.enable_console_sink = enabled;
    }
    sc_observability::v2::LoggerBuilder::new(settings)
        .map_err(|err| err.to_string())?
        .build()
        .map_err(|err| err.to_string())
}
