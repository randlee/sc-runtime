//! Daemon process. Loads config, starts logging, then builds the daemon.
//!
//! Early failures print one stderr line, `<code> <cause> — <recovery>`, before
//! logging exists. `/health` does not check the widget database.

use std::path::PathBuf;
use std::process::ExitCode;

use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = daemon::APP_NAME)]
struct Args {
    /// Listen endpoint override (`uds:<path>` or `tcp:<socket-addr>`).
    #[arg(long)]
    endpoint: Option<String>,
}

/// Observability failed to start. The cause is the builder error text.
struct ObservabilityStart {
    cause: String,
}

impl std::fmt::Display for ObservabilityStart {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.cause)
    }
}

#[tokio::main]
async fn main() -> ExitCode {
    let args = match Args::try_parse() {
        Ok(args) => args,
        Err(err) => return usage(err),
    };
    let config_name = match sc_config::AppName::new(daemon::APP_NAME) {
        Ok(name) => name,
        Err(err) => {
            return startup_exit(
                "APP.INVALID_NAME",
                err,
                "fix the generated application name",
            );
        }
    };
    let mut config = match sc_config::load::<daemon::AppConfig>(config_name) {
        Ok(config) => config,
        Err(err) => {
            return startup_exit("DAEMON.CONFIG", err, "fix the configuration file and retry");
        }
    };
    if args.endpoint.is_some() {
        config.daemon.endpoint_flag = args.endpoint;
    }
    let transport_name = match sc_transport::AppName::new(daemon::APP_NAME) {
        Ok(name) => name,
        Err(err) => {
            return startup_exit(err.code(), err, "fix the generated application name");
        }
    };
    let log_root = match log_root(&config, &transport_name) {
        Ok(path) => path,
        Err(err) => {
            return startup_exit(
                err.code(),
                err,
                "set logging.log_root or a writable instance root",
            );
        }
    };
    let logger = match init_observability(&config.logging, log_root) {
        Ok(logger) => logger,
        Err(err) => {
            return startup_exit(
                "DAEMON.OBSERVABILITY",
                err,
                "check log_root and the logging settings",
            );
        }
    };
    let database = match daemon::database_path(&config, &transport_name) {
        Ok(path) => path,
        Err(err) => {
            return startup_exit(
                err.code(),
                err,
                "set an absolute database path or a writable instance root",
            );
        }
    };
    let result = sc_runtime::Daemon::builder(&transport_name, &config.daemon)
        .stores(move || async move {
            service::Stores::open(database)
                .await
                .map_err(daemon::map_store_open)
        })
        .routes(daemon::routes::router)
        {% if mcp %}.mcp(daemon::mcp::service){% endif %}
        .run()
        .await;
    drop(logger);
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => startup_exit(err.code(), err, "read the cause and retry"),
    }
}

fn log_root(
    config: &daemon::AppConfig,
    app: &sc_transport::AppName,
) -> Result<PathBuf, sc_transport::TransportError> {
    if let Some(root) = &config.logging.log_root {
        return Ok(root.clone());
    }
    daemon::instance_root(config, app)
}

fn init_observability(
    logging: &sc_observability::LogSettings,
    log_root: PathBuf,
) -> Result<sc_observability::v2::Logger, ObservabilityStart> {
    let service = sc_observability::ServiceName::new(daemon::APP_NAME).map_err(observe)?;
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
        .map_err(observe)?
        .build()
        .map_err(observe)
}

fn observe(err: impl std::fmt::Display) -> ObservabilityStart {
    ObservabilityStart {
        cause: err.to_string(),
    }
}

fn startup_exit(code: &str, cause: impl std::fmt::Display, recovery: &str) -> ExitCode {
    eprintln!("{code} {cause} — {recovery}");
    ExitCode::from(1)
}

fn usage(err: clap::Error) -> ExitCode {
    if matches!(
        err.kind(),
        clap::error::ErrorKind::DisplayHelp | clap::error::ErrorKind::DisplayVersion
    ) {
        print!("{err}");
        return ExitCode::from(u8::try_from(err.exit_code()).unwrap_or(0));
    }
    let rendered = err.to_string();
    let cause = rendered
        .lines()
        .find(|line| !line.is_empty())
        .unwrap_or("invalid arguments");
    eprintln!("CLI.USAGE {cause} — run with --help");
    ExitCode::from(2)
}
