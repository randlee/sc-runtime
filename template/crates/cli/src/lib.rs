//! Report-only CLI. `daemon start` is the one command that execs the daemon.

use std::ffi::OsStr;
use std::fmt;
use std::path::{Path, PathBuf};
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

/// Stable code when `daemon start` cannot exec the sibling daemon binary.
pub const CODE_DAEMON_EXEC: &str = "DAEMON.EXEC_FAILED";

/// Recovery line for [`CODE_DAEMON_EXEC`].
const DAEMON_EXEC_RECOVERY: &str = "build the sibling daemon binary and run daemon start again";

/// Envelope message for [`CODE_DAEMON_EXEC`].
const DAEMON_EXEC_MESSAGE: &str = "the daemon binary could not be started";

/// Parsed argv, excluding the program name.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Command {
    /// Exec the daemon binary. Report-only commands never produce this.
    DaemonStart {
        /// `--json` was present.
        json: bool,
    },
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
        } => Command::DaemonStart { json },
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
        Command::DaemonStart { .. } => {
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

/// Text a failed `daemon start` writes. Stdout is empty unless `--json` is set.
#[derive(Debug, Clone, PartialEq)]
pub struct DaemonStartReport {
    /// Process status. Failed exec is 1. A Windows wait that exits 0 is 0.
    pub code: ExitCode,
    /// Coded stderr line, without a trailing newline.
    pub stderr: String,
    /// JSON envelope when `--json` is set. Empty otherwise.
    pub stdout: String,
}

/// Replaces this process with the daemon binary.
///
/// A missing sibling binary, or any exec that returns, sets
/// [`DaemonStartReport::stderr`] to a [`CODE_DAEMON_EXEC`] line. `json` also
/// fills [`DaemonStartReport::stdout`] with the failure envelope
/// `{version, ok, data, error}`. The caller writes those strings.
pub fn exec_daemon(json: bool) -> DaemonStartReport {
    let path = match prepare_daemon_path() {
        Ok(path) => path,
        Err(error) => return error.report(json),
    };
    let mut command = std::process::Command::new(&path);
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        let io_error = command.exec();
        DaemonExecError::io(&path, &io_error).report(json)
    }
    #[cfg(not(unix))]
    {
        match command.status() {
            Ok(status) if status.success() => DaemonStartReport {
                code: ExitCode::SUCCESS,
                stderr: String::new(),
                stdout: String::new(),
            },
            Ok(status) => DaemonExecError::status(&path, &status).report(json),
            Err(error) => DaemonExecError::io(&path, &error).report(json),
        }
    }
}

/// Coded failure when `daemon start` does not replace this process.
#[derive(Debug, Clone, PartialEq, Eq)]
struct DaemonExecError {
    code: &'static str,
    cause: String,
    recovery: &'static str,
    path: String,
}

impl fmt::Display for DaemonExecError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{} {} — {}",
            self.code, self.cause, self.recovery
        )
    }
}

impl DaemonExecError {
    fn unresolved() -> Self {
        Self {
            code: CODE_DAEMON_EXEC,
            cause: "could not locate this executable or its directory".to_owned(),
            recovery: DAEMON_EXEC_RECOVERY,
            path: String::new(),
        }
    }

    fn missing(path: &Path) -> Self {
        Self {
            code: CODE_DAEMON_EXEC,
            cause: format!("daemon binary {} is missing", path.display()),
            recovery: DAEMON_EXEC_RECOVERY,
            path: path.display().to_string(),
        }
    }

    fn io(path: &Path, error: &std::io::Error) -> Self {
        Self {
            code: CODE_DAEMON_EXEC,
            cause: format!("could not exec {}: {error}", path.display()),
            recovery: DAEMON_EXEC_RECOVERY,
            path: path.display().to_string(),
        }
    }

    #[cfg(not(unix))]
    fn status(path: &Path, status: &std::process::ExitStatus) -> Self {
        Self {
            code: CODE_DAEMON_EXEC,
            cause: format!("daemon binary {} exited {status}", path.display()),
            recovery: DAEMON_EXEC_RECOVERY,
            path: path.display().to_string(),
        }
    }

    fn report(&self, json: bool) -> DaemonStartReport {
        let stderr = self.to_string();
        let stdout = if json {
            self.envelope_json()
        } else {
            String::new()
        };
        DaemonStartReport {
            code: ExitCode::from(1),
            stderr,
            stdout,
        }
    }

    fn envelope_json(&self) -> String {
        let operation = exec_operation(self);
        let envelope: Envelope<Value> = Envelope::failure(operation);
        serde_json::to_string(&envelope).unwrap_or_else(|_| "{}".to_owned())
    }
}

fn prepare_daemon_path() -> Result<PathBuf, DaemonExecError> {
    let Some(path) = daemon_binary() else {
        return Err(DaemonExecError::unresolved());
    };
    if path.is_file() {
        Ok(path)
    } else {
        Err(DaemonExecError::missing(&path))
    }
}

fn exec_operation(error: &DaemonExecError) -> OpError {
    let path = if error.path.is_empty() {
        None
    } else {
        Some(error.path.as_str())
    };
    let value = serde_json::json!({
        "kind": "unavailable",
        "code": error.code,
        "message": DAEMON_EXEC_MESSAGE,
        "details": {
            "path": path,
            "cause": error.cause,
        },
        "suggested_action": error.recovery,
    });
    serde_json::from_value(value).unwrap_or_else(|_| {
        OpError::from_transport_strings(error.code, &error.path, &error.cause, Some(error.recovery))
    })
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

    #[test]
    fn daemon_start_records_json() {
        let plain = parse(&["daemon".into(), "start".into()]).expect("parse");
        assert_eq!(plain, Command::DaemonStart { json: false });
        let json = parse(&["--json".into(), "daemon".into(), "start".into()]).expect("json");
        assert_eq!(json, Command::DaemonStart { json: true });
    }

    #[test]
    fn missing_sibling_daemon_is_coded() {
        let path = PathBuf::from(format!("{APP_NAME}-daemon"));
        let error = DaemonExecError::missing(&path);
        let report = error.report(true);
        assert_eq!(report.code, ExitCode::from(1));
        assert!(report.stderr.starts_with(&format!("{CODE_DAEMON_EXEC} ")));
        assert!(report.stderr.contains('—'));
        assert!(report.stderr.contains(DAEMON_EXEC_RECOVERY));
        assert!(report.stderr.contains(&path.display().to_string()));
        let body: Value = serde_json::from_str(&report.stdout).expect("envelope");
        assert_eq!(body["ok"], false);
        assert!(body["data"].is_null());
        assert_eq!(body["version"], "1");
        assert_eq!(body["error"]["code"], CODE_DAEMON_EXEC);
        assert_eq!(body["error"]["kind"], "unavailable");
        assert_eq!(body["error"]["message"], DAEMON_EXEC_MESSAGE);
        assert_eq!(body["error"]["suggested_action"], DAEMON_EXEC_RECOVERY);
        assert_eq!(body["error"]["details"]["path"], path.display().to_string());
        let plain = DaemonExecError::missing(&path).report(false);
        assert!(plain.stdout.is_empty());
        assert_eq!(plain.stderr, report.stderr);
    }

    #[test]
    fn unresolved_daemon_path_has_null_path() {
        let report = DaemonExecError::unresolved().report(true);
        let body: Value = serde_json::from_str(&report.stdout).expect("envelope");
        assert!(body["error"]["details"]["path"].is_null());
        assert!(report.stderr.contains("could not locate this executable"));
    }

    #[test]
    fn this_process_has_no_sibling_daemon() {
        let error = prepare_daemon_path().expect_err("sibling daemon");
        assert_eq!(error.code, CODE_DAEMON_EXEC);
        assert!(error.cause.contains("is missing"));
    }

    #[test]
    fn exec_io_error_names_the_path() {
        let path = PathBuf::from("missing-daemon");
        let io_error =
            std::io::Error::new(std::io::ErrorKind::PermissionDenied, "permission denied");
        let report = DaemonExecError::io(&path, &io_error).report(false);
        assert!(report.stderr.contains("permission denied"));
        assert!(report.stderr.contains("missing-daemon"));
        assert!(report.stdout.is_empty());
    }
}
