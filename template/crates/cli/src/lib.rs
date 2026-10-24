//! Report-only CLI. `daemon start` is the one command that execs the daemon.

use std::ffi::OsStr;
use std::path::PathBuf;
use std::process::ExitCode;

use sc_command::{Envelope, OpError};
use sc_transport::{Client, Endpoint, TransportConfig, TransportError};
use serde::Deserialize;
use serde_json::{Value, json};

/// Generated application name. cargo-generate fills this from `--name`.
pub const APP_NAME: &str = "{{project-name}}";

/// Parsed argv, excluding the program name.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Command {
    /// Exec the daemon binary. Report-only commands never produce this.
    DaemonStart,
    /// `widget create`.
    WidgetCreate {
        /// Widget name.
        name: String,
        /// `--endpoint` text, when the flag was present.
        endpoint: Option<String>,
        /// `--json` was present.
        json: bool,
    },
    /// `widget get`.
    WidgetGet {
        /// Widget id.
        id: String,
        /// `--endpoint` text, when the flag was present.
        endpoint: Option<String>,
        /// `--json` was present.
        json: bool,
    },
}

/// Loads the endpoint fields and ignores the daemon-only keys.
#[derive(Debug, Deserialize)]
struct CliConfig {
    #[serde(flatten)]
    transport: TransportConfig,
}

/// Parses CLI arguments.
///
/// # Errors
///
/// Returns a usage string when the arguments do not match a command.
pub fn parse(args: &[String]) -> Result<Command, String> {
    let mut endpoint = None;
    let mut json = false;
    let mut rest = Vec::new();
    let mut iter = args.iter();
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--json" => json = true,
            "--endpoint" => {
                let value = iter
                    .next()
                    .ok_or_else(|| "missing value for --endpoint".to_owned())?;
                endpoint = Some(value.clone());
            }
            "--name" | "--id" => {
                let value = iter
                    .next()
                    .ok_or_else(|| format!("missing value for {arg}"))?;
                rest.push((*arg).clone());
                rest.push(value.clone());
            }
            "--" => {
                rest.extend(iter.cloned());
                break;
            }
            other if other.starts_with("--") => {
                return Err(format!("unknown flag {other}"));
            }
            other => rest.push(other.to_owned()),
        }
    }
    let words: Vec<&str> = rest.iter().map(String::as_str).collect();
    match words.as_slice() {
        ["daemon", "start"] => Ok(Command::DaemonStart),
        ["widget", "create", "--name", name] => Ok(Command::WidgetCreate {
            name: (*name).to_owned(),
            endpoint,
            json,
        }),
        ["widget", "get", "--id", id] => Ok(Command::WidgetGet {
            id: (*id).to_owned(),
            endpoint,
            json,
        }),
        _ => Err("usage: widget create --name NAME | widget get --id ID | daemon start".into()),
    }
}

/// Maps one transport failure into the CLI envelope error.
///
/// This function owns `suggested_action` for every [`TransportError`].
/// `DAEMON.NOT_RUNNING` uses `run <app> daemon start` from this application's
/// name. Transport does not carry that string.
#[must_use]
pub fn map_transport(error: &TransportError) -> OpError {
    let code = error.code();
    let (endpoint, cause) = endpoint_and_cause(error);
    let action = if code == sc_transport::CODE_DAEMON_NOT_RUNNING {
        Some(format!("run {APP_NAME} daemon start"))
    } else {
        None
    };
    OpError::from_transport_strings(code, &endpoint, &cause, action.as_deref())
}

/// Runs a report-only command. `DaemonStart` is rejected here so the process
/// image is replaced only by [`exec_daemon`].
///
/// # Errors
///
/// The returned status is non-zero when the command fails. The string is the
/// JSON envelope when `--json` is set.
pub async fn execute(command: Command) -> (ExitCode, String) {
    let (endpoint_flag, json, path, body) = match command {
        Command::DaemonStart => {
            return (
                ExitCode::from(1),
                String::from("daemon start is not a report-only command"),
            );
        }
        Command::WidgetCreate {
            name,
            endpoint,
            json,
        } => (
            endpoint,
            json,
            "/ops/widget.create",
            json!({ "name": name }),
        ),
        Command::WidgetGet { id, endpoint, json } => {
            (endpoint, json, "/ops/widget.get", json!({ "id": id }))
        }
    };
    match post_widget(endpoint_flag.as_deref(), path, &body).await {
        Ok(value) => {
            let failed = value.get("ok").and_then(Value::as_bool) != Some(true);
            let text = if json {
                serde_json::to_string(&value).unwrap_or_else(|_| "{}".to_owned())
            } else {
                String::new()
            };
            let code = if failed {
                ExitCode::from(1)
            } else {
                ExitCode::SUCCESS
            };
            (code, text)
        }
        Err(error) => {
            let envelope: Envelope<Value> = Envelope::failure(map_transport(&error));
            let text = if json {
                serde_json::to_string(&envelope).unwrap_or_else(|_| "{}".to_owned())
            } else {
                error.to_string()
            };
            (ExitCode::from(1), text)
        }
    }
}

/// Example choice: the daemon binary is `{app}-daemon` beside this executable.
#[must_use]
pub fn daemon_binary() -> Option<PathBuf> {
    let current = std::env::current_exe().ok()?;
    let parent = current.parent()?;
    Some(parent.join(format!("{APP_NAME}-daemon")))
}

/// Replaces this process with the daemon binary.
pub fn exec_daemon() -> ExitCode {
    let Some(path) = daemon_binary() else {
        return ExitCode::from(1);
    };
    let mut command = std::process::Command::new(path);
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        let _ = command.exec();
        ExitCode::from(1)
    }
    #[cfg(not(unix))]
    {
        match command.status() {
            Ok(status) if status.success() => ExitCode::SUCCESS,
            _ => ExitCode::from(1),
        }
    }
}

async fn post_widget(
    endpoint_flag: Option<&str>,
    path: &str,
    body: &Value,
) -> Result<Value, TransportError> {
    let endpoint = resolve(endpoint_flag)?;
    let client = Client::new(endpoint)?;
    client.post(path, body).await
}

fn resolve(endpoint_flag: Option<&str>) -> Result<Endpoint, TransportError> {
    let app = sc_transport::AppName::new(APP_NAME)?;
    let transport = load_transport();
    let flag = endpoint_flag.map(OsStr::new);
    sc_transport::resolve_endpoint_from_env(&app, flag, &transport)
}

fn load_transport() -> TransportConfig {
    let Ok(config_name) = sc_config::AppName::new(APP_NAME) else {
        return TransportConfig::default();
    };
    sc_config::load::<CliConfig>(config_name)
        .map(|config| config.transport)
        .unwrap_or_default()
}

fn endpoint_and_cause(error: &TransportError) -> (String, String) {
    match error {
        TransportError::InvalidName { name, detail, .. } => (name.clone(), detail.clone()),
        TransportError::InvalidEndpoint { source, detail, .. } => (source.clone(), detail.clone()),
        TransportError::MissingTcpPort { detail, .. } => (String::new(), (*detail).to_owned()),
        TransportError::InstanceRootUnavailable { detail, .. } => (String::new(), detail.clone()),
        TransportError::ResponseDecode {
            status,
            cause,
            body_preview,
            ..
        } => (status.to_string(), format!("{cause}: {body_preview}")),
        TransportError::DaemonNotRunning {
            endpoint, cause, ..
        }
        | TransportError::TimedOut {
            endpoint, cause, ..
        }
        | TransportError::Request {
            endpoint, cause, ..
        }
        | TransportError::Bind {
            endpoint, cause, ..
        }
        | TransportError::EndpointBusy {
            endpoint, cause, ..
        }
        | TransportError::NotASocket {
            endpoint, cause, ..
        } => (endpoint.to_string(), cause.as_str().to_owned()),
        _ => (String::new(), error.to_string()),
    }
}
