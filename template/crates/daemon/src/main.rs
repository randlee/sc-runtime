//! Daemon process. Loads config, starts logging, then builds the daemon.
//!
//! Early failures print one stderr line, `<code> <cause> — <recovery>`, before
//! logging exists. `/health` does not check the widget database. `GET /ready`
//! does.

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

#[tokio::main]
async fn main() -> ExitCode {
    let args = match Args::try_parse() {
        Ok(args) => args,
        Err(err) => return usage(err),
    };
    let config_name = match sc_config::AppName::new(daemon::APP_NAME) {
        Ok(name) => name,
        Err(err) => return config_exit(&err),
    };
    let mut config = match sc_config::load::<daemon::AppConfig>(config_name) {
        Ok(config) => config,
        Err(err) => return config_exit(&err),
    };
    if args.endpoint.is_some() {
        config.daemon.endpoint_flag = args.endpoint;
    }
    let transport_name = match sc_transport::AppName::new(daemon::APP_NAME) {
        Ok(name) => name,
        Err(err) => {
            return transport_exit(&err);
        }
    };
    let log_root = match log_root(&config, &transport_name) {
        Ok(path) => path,
        Err(err) => {
            return transport_exit(&err);
        }
    };
    let observability = match daemon::observability::Observability::init(
        daemon::APP_NAME,
        &config.logging,
        log_root,
    ) {
        Ok(observability) => observability,
        Err(err) => {
            return startup_exit(
                "DAEMON.OBSERVABILITY",
                err,
                "fix logging configuration or sink paths and retry",
            );
        }
    };
    let result = daemon::composition::run(
        &config, &transport_name, &observability,
        #[cfg(feature = "cron")] daemon::composition::CronServices::default(),
    ).await;
    // `run` has joined both library users and dropped every injected handle.
    match daemon::agent_error::finish(result, observability.shutdown()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            ExitCode::from(1)
        }
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

fn transport_exit(error: &sc_transport::TransportError) -> ExitCode {
    eprintln!("{}", daemon::agent_error::transport_error(error));
    ExitCode::from(1)
}

fn startup_exit(code: &str, cause: impl std::fmt::Display, recovery: &str) -> ExitCode {
    eprintln!("{}", daemon::agent_error::format_agent_error(code, cause, recovery));
    ExitCode::from(1)
}

// `ConfigError`'s `Display` already includes `code()` and `recovery()`.
// Pass those to `startup_exit` once so the stderr line does not repeat them.
fn config_exit(err: &sc_config::ConfigError) -> ExitCode {
    startup_exit(err.code(), config_cause(err), err.recovery())
}

fn config_cause(err: &sc_config::ConfigError) -> String {
    let rendered = err.to_string();
    let prefix = format!("{}: ", err.code());
    let suffix = format!("; {}", err.recovery());
    rendered
        .strip_prefix(&prefix)
        .and_then(|body| body.strip_suffix(&suffix))
        .unwrap_or(&rendered)
        .to_owned()
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
    eprintln!("{}", daemon::agent_error::format_agent_error("CLI.USAGE", cause, "run with --help"));
    ExitCode::from(2)
}

#[cfg(test)]
mod tests {
    use std::ffi::OsString;
    use std::path::Path;

    use sc_config::ConfigError;

    use super::config_cause;

    fn startup_line(err: &ConfigError) -> String {
        daemon::agent_error::format_agent_error(err.code(), config_cause(err), err.recovery())
    }

    fn assert_library_line(err: &ConfigError) {
        let line = startup_line(err);
        let code = err.code();
        let recovery = err.recovery();
        assert!(line.starts_with(&format!("{code} ")), "{line}");
        let (head, tail) = line
            .split_once(" — ")
            .unwrap_or_else(|| panic!("missing recovery separator: {line}"));
        assert_eq!(tail, recovery, "{line}");
        assert!(head.starts_with(code), "{line}");
        assert!(!head[code.len()..].contains(code), "{line}");
        assert!(!head.contains(recovery), "{line}");
        assert!(!line.contains("DAEMON.CONFIG"), "{line}");
        assert!(!config_cause(err).is_empty(), "{line}");
    }

    #[cfg(feature = "cron")]
    #[tokio::test]
    async fn runtime_failure_before_stores_releases_handles_without_waiting_for_worker() {
        let root = tempfile::tempdir().expect("instance root");
        let mut values: serde_json::Value = serde_json::from_str(include_str!("../../../config/default.json")).expect("default config");
        values["instance_root"] = serde_json::json!(root.path());
        let mut config: daemon::AppConfig = serde_json::from_value(values).expect("config");
        config.daemon.endpoint_flag = Some("invalid-endpoint".to_owned());
        let observability = daemon::observability::Observability::init(
            daemon::APP_NAME, &config.logging, root.path().to_path_buf(),
        ).unwrap_or_else(|error| panic!("logger: {error}"));
        let weak = std::sync::Arc::downgrade(&observability.logger());
        let app = sc_transport::AppName::new(daemon::APP_NAME).expect("app");
        let error = tokio::time::timeout(
            std::time::Duration::from_secs(5), daemon::composition::run(&config, &app, &observability, daemon::composition::CronServices::default()),
        ).await.expect("no worker handle exists before the stores closure runs").expect_err("endpoint rejected");
        assert_eq!(error, "RUNTIME.ENDPOINT_FAILED the endpoint could not be resolved: TRANSPORT.INVALID_ENDPOINT: source=invalid-endpoint: endpoint must start with uds: or tcp: — Use uds:<path> or tcp:<socket-addr>.");
        assert_eq!(weak.strong_count(), 1, "only owner retains logger handle");
        observability.shutdown().expect("consuming shutdown");
        assert!(weak.upgrade().is_none());
    }

    #[test]
    fn config_startup_uses_each_library_code_once() {
        let errors = [
            ConfigError::MissingDefault {
                path: Path::new("config/default.json").to_path_buf(),
            },
            ConfigError::Read {
                path: Path::new("config/default.json").to_path_buf(),
                source: std::io::Error::other("denied"),
            },
            ConfigError::Parse {
                path: Path::new("config/default.json").to_path_buf(),
                line: 2,
                column: 4,
                message: "unexpected ; token".to_owned(),
            },
            ConfigError::Override {
                variable: "MY_APP__A__B".to_owned(),
            },
            ConfigError::Deserialize {
                message: "type".to_owned(),
                files: vec![Path::new("config/default.json").to_path_buf()],
                variables: vec!["MY_APP__A".to_owned()],
            },
            ConfigError::InvalidName {
                name: "my app".to_owned(),
            },
            ConfigError::NonUnicode {
                variable: OsString::from("MY_APP__NAME"),
            },
        ];
        for err in &errors {
            assert_library_line(err);
        }
    }
}
