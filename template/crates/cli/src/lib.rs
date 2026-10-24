//! Report-only CLI. `daemon start` is the one command that execs the daemon.

use std::ffi::OsStr;
use std::fmt;
use std::path::PathBuf;
use std::process::ExitCode;

use api_types::{CreateWidget, GetWidget, WidgetId, WidgetName};
use clap::{Parser, Subcommand};
use sc_command::{Envelope, OpError};
use sc_transport::{Client, Endpoint, TransportConfig, TransportError};
use serde::Serialize;
use serde_json::Value;

/// Generated application name. cargo-generate fills this from `--name`.
pub const APP_NAME: &str = "{{project-name}}";

/// Stable code for argv that does not match a command.
pub const CODE_USAGE: &str = "CLI.USAGE";

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

/// Structured parse failure. `--help` carries clap's rendered text.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseError {
    /// Stable code. Usage failures use [`CODE_USAGE`].
    pub code: &'static str,
    /// What was wrong with the arguments.
    pub cause: String,
    /// How to recover.
    pub recovery: &'static str,
    /// Process status. Help exits 0. Usage exits 2.
    pub exit_code: u8,
    /// Rendered help or version text, when clap asked to display it.
    pub help: Option<String>,
}

impl fmt::Display for ParseError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{} {} — {}",
            self.code, self.cause, self.recovery
        )
    }
}

impl ParseError {
    fn usage(cause: impl Into<String>) -> Self {
        Self {
            code: CODE_USAGE,
            cause: cause.into(),
            recovery: "run with --help",
            exit_code: 2,
            help: None,
        }
    }

    fn from_clap(err: clap::Error) -> Self {
        let help = matches!(
            err.kind(),
            clap::error::ErrorKind::DisplayHelp | clap::error::ErrorKind::DisplayVersion
        );
        if help {
            let exit_code = u8::try_from(err.exit_code()).unwrap_or(0);
            return Self {
                code: CODE_USAGE,
                cause: "help requested".to_owned(),
                recovery: "see the command help",
                exit_code,
                help: Some(err.to_string()),
            };
        }
        let cause = err
            .to_string()
            .lines()
            .find(|line| !line.is_empty())
            .unwrap_or("invalid arguments")
            .to_owned();
        Self::usage(cause)
    }
}

#[derive(Parser, Debug)]
#[command(name = APP_NAME)]
struct CliArgs {
    /// Endpoint override (`uds:<path>` or `tcp:<socket-addr>`).
    #[arg(long, global = true)]
    endpoint: Option<String>,

    /// Print the JSON envelope.
    #[arg(long, global = true)]
    json: bool,

    #[command(subcommand)]
    command: CliCommand,
}

#[derive(Subcommand, Debug)]
enum CliCommand {
    /// Daemon process commands.
    Daemon {
        #[command(subcommand)]
        action: DaemonAction,
    },
    /// Widget commands.
    Widget {
        #[command(subcommand)]
        action: WidgetAction,
    },
}

#[derive(Subcommand, Debug)]
enum DaemonAction {
    /// Replace this process with the daemon binary.
    Start,
}

#[derive(Subcommand, Debug)]
enum WidgetAction {
    /// Create one widget.
    Create {
        /// Widget name.
        #[arg(long)]
        name: String,
    },
    /// Get one widget by id.
    Get {
        /// Widget id.
        #[arg(long)]
        id: String,
    },
}

/// Loads the endpoint fields and ignores the daemon-only keys.
#[derive(Debug, serde::Deserialize)]
struct CliConfig {
    #[serde(flatten)]
    transport: TransportConfig,
}

/// Parses CLI arguments.
///
/// # Errors
///
/// Returns [`ParseError`] with code [`CODE_USAGE`] when the arguments do not
/// match a command.
pub fn parse(args: &[String]) -> Result<Command, ParseError> {
    let parsed =
        CliArgs::try_parse_from(std::iter::once(APP_NAME.to_owned()).chain(args.iter().cloned()))
            .map_err(ParseError::from_clap)?;
    let endpoint = parsed.endpoint;
    let json = parsed.json;
    let command = match parsed.command {
        CliCommand::Daemon {
            action: DaemonAction::Start,
        } => Command::DaemonStart,
        CliCommand::Widget {
            action: WidgetAction::Create { name },
        } => Command::WidgetCreate {
            name,
            endpoint,
            json,
        },
        CliCommand::Widget {
            action: WidgetAction::Get { id },
        } => Command::WidgetGet { id, endpoint, json },
    };
    Ok(command)
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
        } => {
            let Ok(name) = WidgetName::parse(&name) else {
                return usage_result("widget name is longer than 256 bytes");
            };
            (
                endpoint,
                json,
                api_types::WIDGET_CREATE_PATH,
                WidgetBody::Create(CreateWidget { name }),
            )
        }
        Command::WidgetGet { id, endpoint, json } => {
            let Ok(id) = WidgetId::parse(&id) else {
                return usage_result(
                    "widget id must be 1-64 ascii letters, digits, hyphens, or underscores",
                );
            };
            (
                endpoint,
                json,
                api_types::WIDGET_GET_PATH,
                WidgetBody::Get(GetWidget { id }),
            )
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

enum WidgetBody {
    Create(CreateWidget),
    Get(GetWidget),
}

impl Serialize for WidgetBody {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Self::Create(body) => body.serialize(serializer),
            Self::Get(body) => body.serialize(serializer),
        }
    }
}

fn usage_result(cause: &str) -> (ExitCode, String) {
    (ExitCode::from(2), ParseError::usage(cause).to_string())
}

async fn post_widget(
    endpoint_flag: Option<&str>,
    path: &str,
    body: &impl Serialize,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unknown_flag_uses_the_usage_code() {
        let err = parse(&["--nope".into()]).expect_err("flag");
        assert_eq!(err.code, CODE_USAGE);
        assert_eq!(err.exit_code, 2);
        let text = err.to_string();
        assert!(text.starts_with("CLI.USAGE "));
        assert!(text.contains('—'));
    }
}
