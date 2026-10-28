//! Host function `render_template` for a Monty cron feed.
//!
//! The feed stays inside Monty. This module spawns [`ComposeProgram::SC_COMPOSE`]
//! on a private temp root the hook cannot choose, then returns the child's
//! stdout. Tests pass another [`ComposeProgram`]. It does not start CPython.

use std::fmt;
use std::time::Instant;

use monty_types::{MontyObject, ObjectRef};
use serde_json::{Map, Number, Value};

/// Program name or path spawned for one `render_template` call.
///
/// Production uses [`Self::SC_COMPOSE`]. A test may pass another name. This is
/// not an embedder setting: [`crate::Scheduler`](crate::Scheduler) never
/// accepts a program.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct ComposeProgram<'a>(&'a str);

impl ComposeProgram<'static> {
    /// The program production feeds spawn on `PATH`.
    pub(crate) const SC_COMPOSE: Self = Self::new("sc-compose");
}

impl<'a> ComposeProgram<'a> {
    /// Wraps a program name or path. The hook source cannot supply this.
    #[must_use]
    pub(crate) const fn new(program: &'a str) -> Self {
        Self(program)
    }

    /// Returns the program name or path.
    #[must_use]
    pub(crate) const fn as_str(self) -> &'a str {
        self.0
    }
}

impl fmt::Display for ComposeProgram<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.0)
    }
}

/// How `render_template` failed.
///
/// [`Self::HostTimedOut`] is recorded as [`crate::RunOutcome::TimedOut`].
/// Every other variant is recorded as [`crate::RunOutcome::Failed`].
/// These variants are not [`crate::CronError`] variants.
#[derive(Debug)]
pub(crate) enum ComposeHostError {
    /// `compose_program` was not found on `PATH` (or the injected path).
    ProgramNotFound {
        /// Program name or path that could not be started.
        program: String,
    },
    /// The temp root could not be created.
    TempRootFailed {
        /// Program name or path that would have been spawned.
        program: String,
        /// Filesystem error from `tempfile`.
        source: std::io::Error,
    },
    /// Writing the template or the context file failed.
    StageFailed {
        /// Program name or path that would have been spawned.
        program: String,
        /// Filesystem error from the temp root.
        source: std::io::Error,
    },
    /// The context value could not be encoded as JSON.
    ContextEncodeFailed {
        /// Program name or path that would have been spawned.
        program: String,
        /// Serializer error.
        source: serde_json::Error,
    },
    /// The program was found but could not be executed.
    ProgramNotRunnable {
        /// Program name or path passed to the process spawn.
        program: String,
        /// Spawn error other than not-found.
        source: std::io::Error,
    },
    /// The child exited zero and stdout was not UTF-8.
    OutputNotUtf8 {
        /// Program name or path that produced the stdout.
        program: String,
    },
    /// The child exited zero and stdout exceeded [`crate::MAX_HOOK_OUTPUT_BYTES`].
    OutputTooLarge {
        /// Program name or path that produced the stdout.
        program: String,
    },
    /// The child ran and exited non-zero.
    NonZeroExit {
        /// Exit status, or `None` when the process was ended by a signal.
        status: Option<i32>,
        /// Capped stderr text from the child.
        stderr: String,
    },
    /// The host deadline passed while the compose child was still running.
    ///
    /// The child was killed and reaped. On Unix it is a process-group leader,
    /// so that signal also reaches grandchildren. This is not a machine-wide kill.
    HostTimedOut {
        /// Program name or path that was still running.
        program: String,
    },
}

impl ComposeHostError {
    /// Stable token stored in the `RunOutcome::Failed` detail.
    #[must_use]
    pub(crate) const fn code(&self) -> &'static str {
        match self {
            Self::ProgramNotFound { .. } => "COMPOSE_NOT_FOUND",
            Self::TempRootFailed { .. } | Self::StageFailed { .. } => "COMPOSE_SPAWN",
            Self::ContextEncodeFailed { .. } => "COMPOSE_CONTEXT",
            Self::ProgramNotRunnable { .. } => "COMPOSE_NOT_RUNNABLE",
            Self::OutputNotUtf8 { .. } | Self::OutputTooLarge { .. } => "COMPOSE_OUTPUT",
            Self::NonZeroExit { .. } => "COMPOSE_EXIT",
            Self::HostTimedOut { .. } => "COMPOSE_HOST_TIMEOUT",
        }
    }

    /// Returns what an operator can do about this failure.
    #[must_use]
    pub(crate) const fn recovery(&self) -> &'static str {
        match self {
            Self::ProgramNotFound { .. } => {
                "Install `sc-compose` on PATH or fix the injected program name in tests."
            }
            Self::TempRootFailed { .. } | Self::StageFailed { .. } => {
                "Check permissions and disk space for temp roots under `tempfile::TempDir`."
            }
            Self::ContextEncodeFailed { .. } => {
                "Fix the template context; it could not be encoded as JSON."
            }
            Self::ProgramNotRunnable { .. } => {
                "Check that the compose program is executable and allowed to start."
            }
            Self::OutputNotUtf8 { .. } => "Fix the compose program so its stdout is UTF-8.",
            Self::OutputTooLarge { .. } => {
                "Shrink the rendered output; stdout is capped at the hook output limit."
            }
            Self::NonZeroExit { .. } => {
                "Fix template/context or run `sc-compose render` manually with the logged stderr."
            }
            Self::HostTimedOut { .. } => {
                "Reduce compose work or raise the row deadline; the compose child was stopped at the host budget."
            }
        }
    }
}

impl fmt::Display for ComposeHostError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ProgramNotFound { program } => {
                write!(formatter, "compose program not found: {program}")
            }
            Self::TempRootFailed { program, source } => {
                write!(
                    formatter,
                    "compose program {program} temp root failed: {source}"
                )
            }
            Self::StageFailed { program, source } => {
                write!(
                    formatter,
                    "compose program {program} failed to stage files: {source}"
                )
            }
            Self::ContextEncodeFailed { program, source } => {
                write!(
                    formatter,
                    "compose program {program} context could not be encoded: {source}"
                )
            }
            Self::ProgramNotRunnable { program, source } => {
                write!(
                    formatter,
                    "compose program {program} could not be executed: {source}"
                )
            }
            Self::OutputNotUtf8 { program } => {
                write!(formatter, "compose program {program} stdout was not utf-8")
            }
            Self::OutputTooLarge { program } => {
                write!(
                    formatter,
                    "compose program {program} stdout exceeded the hook output cap"
                )
            }
            Self::NonZeroExit { status, stderr } => match status {
                Some(status) => write!(formatter, "compose exited {status}: {stderr}"),
                None => write!(formatter, "compose exited from a signal: {stderr}"),
            },
            Self::HostTimedOut { program } => {
                write!(
                    formatter,
                    "compose program {program} exceeded the host row budget"
                )
            }
        }
    }
}

impl std::error::Error for ComposeHostError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::TempRootFailed { source, .. }
            | Self::StageFailed { source, .. }
            | Self::ProgramNotRunnable { source, .. } => Some(source),
            Self::ContextEncodeFailed { source, .. } => Some(source),
            Self::ProgramNotFound { .. }
            | Self::OutputNotUtf8 { .. }
            | Self::OutputTooLarge { .. }
            | Self::NonZeroExit { .. }
            | Self::HostTimedOut { .. } => None,
        }
    }
}

/// A Monty value `monty_value_to_json` will not represent.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct MontyJsonError;

impl fmt::Display for MontyJsonError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("monty value is not JSON")
    }
}

/// Whether `deadline` is already due on the host clock.
///
/// `None` means the row budget does not fit in [`Instant`] and is not exhausted.
#[must_use]
pub(crate) fn host_budget_exhausted(deadline: Option<Instant>) -> bool {
    deadline.is_some_and(|deadline| Instant::now() >= deadline)
}

/// Renders `template` with `context` by spawning `compose_program`.
///
/// The child is `compose_program render --root <temp> --mode file --file t.j2 --var-file v.json`.
/// Stdout is the rendered string. The temp directory is removed when this
/// function returns. The hook does not choose the root or the program.
///
/// # Errors
///
/// [`ComposeHostError::ProgramNotFound`] when the program cannot be found,
/// [`ComposeHostError::TempRootFailed`] or [`ComposeHostError::StageFailed`]
/// when the temp root cannot be prepared, [`ComposeHostError::ContextEncodeFailed`]
/// when `context` cannot be encoded, [`ComposeHostError::ProgramNotRunnable`]
/// when the process cannot be started, [`ComposeHostError::OutputNotUtf8`] when
/// stdout is not UTF-8, [`ComposeHostError::OutputTooLarge`] when stdout
/// exceeds the hook output cap, [`ComposeHostError::HostTimedOut`] when the
/// child is killed at `host_deadline`, and [`ComposeHostError::NonZeroExit`]
/// when the child exits non-zero.
pub(crate) fn render_template(
    template: &str,
    context: &Value,
    compose_program: ComposeProgram<'_>,
    host_deadline: Option<Instant>,
) -> Result<String, ComposeHostError> {
    let program = compose_program.as_str();
    let dir = tempfile::TempDir::new().map_err(|source| ComposeHostError::TempRootFailed {
        program: program.to_owned(),
        source,
    })?;
    let root = dir.path();
    std::fs::write(root.join("t.j2"), template).map_err(|source| stage_failed(program, source))?;
    let context_bytes =
        serde_json::to_vec(context).map_err(|source| ComposeHostError::ContextEncodeFailed {
            program: program.to_owned(),
            source,
        })?;
    std::fs::write(root.join("v.json"), context_bytes)
        .map_err(|source| stage_failed(program, source))?;
    crate::monty_compose::run_compose_child(program, root, host_deadline)
}

fn stage_failed(program: &str, source: std::io::Error) -> ComposeHostError {
    ComposeHostError::StageFailed {
        program: program.to_owned(),
        source,
    }
}

pub(crate) fn stderr_text(bytes: &[u8]) -> String {
    let text = String::from_utf8_lossy(bytes);
    crate::hook::stderr_in_message(&text).to_owned()
}

/// Converts a JSON value into the Monty object passed as `__hook` or a context.
#[must_use]
pub(crate) fn json_to_monty(value: &Value) -> MontyObject {
    match value {
        Value::Null => MontyObject::none(),
        Value::Bool(item) => MontyObject::bool(*item),
        Value::Number(number) => number.as_i64().map_or_else(
            || MontyObject::float(number.as_f64().unwrap_or(0.0)),
            MontyObject::int,
        ),
        Value::String(text) => MontyObject::string(text.clone()),
        Value::Array(items) => MontyObject::list(items.iter().map(json_to_monty)),
        Value::Object(map) => MontyObject::dict(
            map.iter()
                .map(|(key, item)| (MontyObject::string(key.clone()), json_to_monty(item))),
        ),
    }
}

/// Converts a Monty value that mirrors [`json_to_monty`].
///
/// Null, bool, int, float, string, list, and dict (string keys) are accepted.
/// Any other Monty type, a non-string dict key, an int that does not fit in
/// `i64`, or a non-finite float is [`MontyJsonError`].
///
/// # Errors
///
/// [`MontyJsonError`] when `value` is not one of those JSON shapes.
pub(crate) fn monty_value_to_json(value: ObjectRef<'_>) -> Result<Value, MontyJsonError> {
    match value.type_name() {
        "NoneType" => Ok(Value::Null),
        "bool" => Ok(Value::Bool(value.as_bool().ok_or(MontyJsonError)?)),
        "int" => Ok(Value::Number(value.as_int().ok_or(MontyJsonError)?.into())),
        "float" => {
            let number = value.as_float().ok_or(MontyJsonError)?;
            Ok(Value::Number(
                Number::from_f64(number).ok_or(MontyJsonError)?,
            ))
        }
        "str" => Ok(Value::String(
            value.as_str().ok_or(MontyJsonError)?.to_owned(),
        )),
        "list" => {
            let items = value.items().ok_or(MontyJsonError)?;
            let mut out = Vec::with_capacity(items.len());
            for item in items {
                out.push(monty_value_to_json(item)?);
            }
            Ok(Value::Array(out))
        }
        "dict" => {
            let pairs = value.pairs().ok_or(MontyJsonError)?;
            let mut map = Map::new();
            for (key, item) in pairs {
                let key = key.as_str().ok_or(MontyJsonError)?.to_owned();
                map.insert(key, monty_value_to_json(item)?);
            }
            Ok(Value::Object(map))
        }
        _ => Err(MontyJsonError),
    }
}

#[cfg(all(test, feature = "monty"))]
mod tests {
    use std::error::Error;
    use std::time::{Duration, Instant};

    use serde_json::{Value, json};

    use super::{
        ComposeHostError, ComposeProgram, host_budget_exhausted, json_to_monty,
        monty_value_to_json, render_template,
    };
    use crate::monty_run::{ComposeInjection, FeedResult, MontyPhase};
    use crate::{Generation, RunOutcome};

    fn hook() -> Value {
        json!({})
    }

    fn run_tick(source: &str) -> crate::monty_run::FeedDone {
        run_injected(
            source,
            ComposeProgram::SC_COMPOSE,
            &[],
            Duration::from_secs(20),
        )
    }

    fn run_injected(
        source: &str,
        program: ComposeProgram<'_>,
        extra_host_fns: &[&str],
        budget: Duration,
    ) -> crate::monty_run::FeedDone {
        let generation = Generation::new(1).expect("generation");
        let payload = hook();
        let stop = crate::monty_run::execute_feed_with_compose_program(
            None,
            source,
            MontyPhase::Stop,
            generation,
            Duration::from_secs(20),
            &payload,
            ComposeInjection::production(),
        );
        assert!(
            matches!(stop.result, FeedResult::Continue),
            "stop: {:?}",
            stop.result
        );
        crate::monty_run::execute_feed_with_compose_program(
            stop.session,
            source,
            MontyPhase::Tick,
            generation,
            budget,
            &payload,
            ComposeInjection {
                program,
                extra_host_fns,
            },
        )
    }

    fn failed_detail(done: crate::monty_run::FeedDone) -> String {
        match done.result {
            FeedResult::Outcome(RunOutcome::Failed(detail)) => detail,
            other => panic!("expected Failed, got {other:?}"),
        }
    }

    #[test]
    fn recovery_text_matches_the_error_table() {
        let missing = ComposeHostError::ProgramNotFound {
            program: "sc-cron-missing-compose-b10".to_owned(),
        };
        assert_eq!(
            missing.recovery(),
            "Install `sc-compose` on PATH or fix the injected program name in tests."
        );
        assert_eq!(missing.code(), "COMPOSE_NOT_FOUND");
        assert!(missing.source().is_none());
        assert!(missing.to_string().contains("sc-cron-missing-compose-b10"));
        let root = ComposeHostError::TempRootFailed {
            program: "sc-compose".to_owned(),
            source: std::io::Error::other("disk"),
        };
        assert_eq!(
            root.recovery(),
            "Check permissions and disk space for temp roots under `tempfile::TempDir`."
        );
        assert_eq!(root.code(), "COMPOSE_SPAWN");
        assert_eq!(
            root.source().map(ToString::to_string).as_deref(),
            Some("disk")
        );
        assert_eq!(
            ComposeHostError::StageFailed {
                program: "sc-compose".to_owned(),
                source: std::io::Error::other("disk"),
            }
            .recovery(),
            root.recovery()
        );
        let encode = ComposeHostError::ContextEncodeFailed {
            program: "sc-compose".to_owned(),
            source: serde_json::from_str::<Value>("{").expect_err("json"),
        };
        assert_eq!(
            encode.recovery(),
            "Fix the template context; it could not be encoded as JSON."
        );
        assert!(encode.source().is_some());
        assert_eq!(encode.code(), "COMPOSE_CONTEXT");
        let runnable = ComposeHostError::ProgramNotRunnable {
            program: "sc-compose".to_owned(),
            source: std::io::Error::other("permission"),
        };
        assert_eq!(
            runnable.recovery(),
            "Check that the compose program is executable and allowed to start."
        );
        assert!(runnable.source().is_some());
        assert!(!runnable.recovery().contains("disk"));
        let utf8 = ComposeHostError::OutputNotUtf8 {
            program: "sc-compose".to_owned(),
        };
        assert_eq!(
            utf8.recovery(),
            "Fix the compose program so its stdout is UTF-8."
        );
        assert!(utf8.source().is_none());
        assert!(!utf8.recovery().contains("disk"));
        assert_eq!(
            ComposeHostError::NonZeroExit {
                status: Some(2),
                stderr: "syntax".to_owned(),
            }
            .recovery(),
            "Fix template/context or run `sc-compose render` manually with the logged stderr."
        );
        assert!(
            ComposeHostError::NonZeroExit {
                status: None,
                stderr: String::new(),
            }
            .to_string()
            .contains("signal")
        );
    }

    #[test]
    fn stderr_text_uses_the_hook_message_cap() {
        let mut bytes = vec![b'a'; 511];
        bytes.extend("é".as_bytes());
        bytes.extend(b"zzzz");
        let text = super::stderr_text(&bytes);
        assert_eq!(text, "a".repeat(511));
        assert!(!text.contains('é'), "{text}");
        assert!(text.len() <= 512, "{text}");
    }

    #[test]
    fn a_passed_deadline_is_exhausted_and_an_open_one_is_not() {
        let earlier = Instant::now()
            .checked_sub(Duration::from_secs(1))
            .expect("earlier");
        assert!(host_budget_exhausted(Some(earlier)));
        assert!(!host_budget_exhausted(Some(
            Instant::now() + Duration::from_secs(60)
        )));
        assert!(!host_budget_exhausted(None));
    }

    #[test]
    fn json_round_trip_accepts_the_mirrored_shapes_and_denies_the_rest() {
        let value = json!({
            "n": 1,
            "x": 1.5,
            "ok": true,
            "s": "a",
            "z": null,
            "xs": [1, "b"],
            "nested": {"k": false}
        });
        let back = monty_value_to_json(json_to_monty(&value).as_ref()).expect("json");
        assert_eq!(back, value);
        assert!(
            monty_value_to_json(
                monty_types::MontyObject::tuple([monty_types::MontyObject::int(1)]).as_ref()
            )
            .is_err()
        );
        assert!(
            monty_value_to_json(monty_types::MontyObject::function("f", None).as_ref()).is_err()
        );
        assert!(monty_value_to_json(monty_types::MontyObject::float(f64::NAN).as_ref()).is_err());
    }

    #[test]
    fn a_missing_program_names_itself_and_does_not_spawn_on_a_bad_call() {
        let error = render_template(
            "Hello",
            &json!({}),
            ComposeProgram::new("sc-cron-missing-compose-b10"),
            None,
        )
        .expect_err("missing");
        assert!(
            matches!(error, ComposeHostError::ProgramNotFound { ref program } if program == "sc-cron-missing-compose-b10")
        );

        let source = r#"
def cron_stop(hook):
    return {"action": "continue", "reason": "ok"}

def cron_tick(hook):
    render_template("only-one-arg")
    return None
"#;
        let done = run_injected(
            source,
            ComposeProgram::new("sc-cron-missing-should-not-run"),
            &[],
            Duration::from_secs(20),
        );
        assert!(done.session.is_none());
        let detail = failed_detail(done);
        assert!(detail.contains("allowlisted"), "{detail}");
        assert!(
            !detail.contains("sc-cron-missing-should-not-run"),
            "{detail}"
        );
    }

    #[test]
    fn a_non_string_template_or_a_non_json_context_is_denied() {
        let template = r#"
def cron_stop(hook):
    return {"action": "continue", "reason": "ok"}

def cron_tick(hook):
    render_template(1, {})
    return None
"#;
        let detail = failed_detail(run_tick(template));
        assert!(detail.contains("template must be a string"), "{detail}");

        let context = r#"
def cron_stop(hook):
    return {"action": "continue", "reason": "ok"}

def cron_tick(hook):
    render_template("Hello", (1, 2))
    return None
"#;
        let detail = failed_detail(run_tick(context));
        assert!(detail.contains("context is not a JSON value"), "{detail}");
    }

    #[test]
    fn another_host_function_and_an_os_call_are_denied() {
        let other = r#"
def cron_stop(hook):
    return {"action": "continue", "reason": "ok"}

def cron_tick(hook):
    compose_file("t.j2", {})
    return None
"#;
        let detail = failed_detail(run_injected(
            other,
            ComposeProgram::SC_COMPOSE,
            &["compose_file"],
            Duration::from_secs(20),
        ));
        assert!(detail.contains("compose_file"), "{detail}");
        assert!(detail.contains("allowlisted"), "{detail}");

        let os_call = r#"
def cron_stop(hook):
    return {"action": "continue", "reason": "ok"}

def cron_tick(hook):
    open("t.j2")
    return None
"#;
        let detail = failed_detail(run_tick(os_call));
        assert!(detail.contains("os call"), "{detail}");
    }

    #[test]
    fn a_tick_that_never_renders_still_runs_when_the_program_is_missing() {
        let source = r#"
def cron_stop(hook):
    return {"action": "continue", "reason": "ok"}

def cron_tick(hook):
    return None
"#;
        let done = run_injected(
            source,
            ComposeProgram::new("sc-cron-missing-compose-b10"),
            &[],
            Duration::from_secs(20),
        );
        assert!(matches!(done.result, FeedResult::Outcome(RunOutcome::Ok)));
        assert!(done.session.is_some());
    }

    #[test]
    fn a_missing_program_on_the_feed_drops_the_repl() {
        let source = r#"
n = 0

def cron_stop(hook):
    return {"action": "continue", "reason": "ok"}

def cron_tick(hook):
    global n
    n = n + 1
    if n > 1:
        raise RuntimeError("repl kept")
    render_template("Hello {{ name }}", {"name": "cron"})
    return None
"#;
        let program = ComposeProgram::new("sc-cron-missing-compose-b10");
        let first = run_injected(source, program, &[], Duration::from_secs(20));
        assert!(first.session.is_none());
        let detail = failed_detail(first);
        assert!(detail.contains("COMPOSE_NOT_FOUND"), "{detail}");
        assert!(detail.contains("sc-cron-missing-compose-b10"), "{detail}");
        assert!(!detail.contains("HOOK_FAILED"), "{detail}");
        assert!(!detail.contains("repl kept"), "{detail}");

        let generation = Generation::new(1).expect("generation");
        let payload = hook();
        let second = crate::monty_run::execute_feed_with_compose_program(
            None,
            source,
            MontyPhase::Tick,
            generation,
            Duration::from_secs(20),
            &payload,
            ComposeInjection {
                program,
                extra_host_fns: &[],
            },
        );
        let detail = failed_detail(second);
        assert!(detail.contains("sc-cron-missing-compose-b10"), "{detail}");
        assert!(!detail.contains("repl kept"), "{detail}");
    }

    #[test]
    fn sc_compose_renders_the_template_stdout() {
        let rendered = render_template(
            "Hello {{ name }}",
            &json!({"name": "cron"}),
            ComposeProgram::SC_COMPOSE,
            None,
        )
        .expect("sc-compose");
        assert!(
            rendered == "Hello cron\n" || rendered == "Hello cron",
            "{rendered:?}"
        );
        assert!(!rendered.contains("INFO"), "{rendered:?}");
    }

    #[cfg(unix)]
    fn slow_compose_script(dir: &std::path::Path) -> std::path::PathBuf {
        let path = dir.join("slow-stub");
        let body = "#!/bin/sh\n\
dir=$(dirname \"$0\")\n\
printf '%s\\n' \"$$\" > \"$dir/leader.pid\"\n\
sleep 15 &\n\
printf '%s\\n' \"$!\" > \"$dir/child.pid\"\n\
touch \"$dir/started\"\n\
wait\n\
touch \"$dir/finished\"\n\
printf 'rendered\\n'\n";
        std::fs::write(&path, body).expect("script");
        use std::os::unix::fs::PermissionsExt;
        let mut permissions = std::fs::metadata(&path).expect("meta").permissions();
        permissions.set_mode(0o755);
        std::fs::set_permissions(&path, permissions).expect("chmod");
        path
    }

    #[cfg(unix)]
    fn assert_reaped(dir: &std::path::Path) {
        for name in ["leader.pid", "child.pid"] {
            let text = std::fs::read_to_string(dir.join(name)).expect(name);
            let pid: u32 = text.trim().parse().expect("pid");
            let alive = std::process::Command::new("/bin/kill")
                .args(["-0", &pid.to_string()])
                .stdin(std::process::Stdio::null())
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .status()
                .is_ok_and(|status| status.success());
            assert!(!alive, "{name} {pid} still alive");
        }
    }

    /// A checked-in fixture under `tests/fixtures/`. Mode 0755 in git.
    #[cfg(unix)]
    fn fixture_path(name: &str) -> std::path::PathBuf {
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures")
            .join(name)
    }

    /// Symlink `fixture` into `dir` as `link_name`.
    ///
    /// The exec path is the link, so the fixture can write side files beside
    /// `$0`. The fixture inode is not written by the test.
    #[cfg(unix)]
    fn linked_fixture(dir: &std::path::Path, link_name: &str, fixture: &str) -> std::path::PathBuf {
        let link = dir.join(link_name);
        std::os::unix::fs::symlink(fixture_path(fixture), &link).expect("link fixture");
        link
    }

    #[cfg(unix)]
    #[test]
    fn an_injected_program_sees_the_render_argv_and_can_resume_twice() {
        let dir = tempfile::TempDir::new().expect("dir");
        let script = linked_fixture(dir.path(), "compose-stub", "compose_record_argv.sh");
        let program = script.to_str().expect("utf8 path");
        let rendered = render_template(
            "ignored",
            &json!({"n": 1}),
            ComposeProgram::new(program),
            None,
        )
        .expect("stub");
        assert_eq!(rendered, "rendered");
        let argv = std::fs::read_to_string(dir.path().join("argv.txt")).expect("argv");
        for flag in [
            "render",
            "--root",
            "--mode",
            "file",
            "--file",
            "t.j2",
            "--var-file",
            "v.json",
        ] {
            assert!(argv.contains(flag), "{argv}");
        }
        assert!(!argv.contains("--output"), "{argv}");

        let source = r#"
def cron_stop(hook):
    return {"action": "continue", "reason": "ok"}

def cron_tick(hook):
    a = render_template("one", {"n": 1})
    b = render_template("two", {"n": 2})
    if a != "rendered" or b != "rendered":
        raise RuntimeError(a + "|" + b)
    return None
"#;
        let done = run_injected(
            source,
            ComposeProgram::new(program),
            &[],
            Duration::from_secs(20),
        );
        assert!(
            matches!(done.result, FeedResult::Outcome(RunOutcome::Ok)),
            "{:?}",
            done.result
        );
        assert!(done.session.is_some());
    }

    #[cfg(unix)]
    #[test]
    fn a_non_zero_stub_and_a_program_that_cannot_start_map_to_the_error_table() {
        let dir = tempfile::TempDir::new().expect("dir");
        let script = fixture_path("compose_fail.sh");
        let program = script.to_str().expect("utf8 path");
        let error =
            render_template("t", &json!({}), ComposeProgram::new(program), None).expect_err("exit");
        match error {
            ComposeHostError::NonZeroExit { status, stderr } => {
                assert_eq!(status, Some(7));
                assert!(stderr.contains("no-template"), "{stderr}");
            }
            other => panic!("expected non-zero, got {other}"),
        }

        let blocked = dir.path().join("not-exec");
        std::fs::write(&blocked, b"nope").expect("write");
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut permissions = std::fs::metadata(&blocked).expect("meta").permissions();
            permissions.set_mode(0o644);
            std::fs::set_permissions(&blocked, permissions).expect("chmod");
        }
        let program = blocked.to_str().expect("utf8 path");
        let error = render_template("t", &json!({}), ComposeProgram::new(program), None)
            .expect_err("spawn");
        assert!(
            matches!(error, ComposeHostError::ProgramNotRunnable { .. }),
            "{error}"
        );
        assert!(error.source().is_some());
        assert_eq!(error.code(), "COMPOSE_NOT_RUNNABLE");
    }

    #[cfg(unix)]
    #[test]
    fn stdout_past_the_hook_cap_is_compose_output() {
        let script = fixture_path("compose_flood.sh");
        let program = script.to_str().expect("utf8 path");
        let error = render_template("t", &json!({}), ComposeProgram::new(program), None)
            .expect_err("flood");
        assert!(
            matches!(error, ComposeHostError::OutputTooLarge { .. }),
            "{error}"
        );
        assert_eq!(error.code(), "COMPOSE_OUTPUT");
    }

    #[cfg(unix)]
    #[test]
    fn a_stub_that_outlasts_the_row_budget_is_timed_out_and_drops_the_session() {
        let dir = tempfile::TempDir::new().expect("dir");
        let script = slow_compose_script(dir.path());
        let program = script.to_str().expect("utf8 path");
        let source = r#"
def cron_stop(hook):
    return {"action": "continue", "reason": "ok"}

def cron_tick(hook):
    render_template("Hello", {"n": 1})
    return None
"#;
        let started_at = Instant::now();
        let done = run_injected(
            source,
            ComposeProgram::new(program),
            &[],
            Duration::from_millis(800),
        );
        assert!(
            started_at.elapsed() < Duration::from_secs(4),
            "host waited for the stub"
        );
        assert!(dir.path().join("started").is_file(), "stub did not start");
        assert!(
            !dir.path().join("finished").exists(),
            "stub ran past the deadline"
        );
        assert_reaped(dir.path());
        assert!(done.session.is_none());
        assert!(
            matches!(done.result, FeedResult::Outcome(RunOutcome::TimedOut)),
            "{:?}",
            done.result
        );
    }

    #[cfg(unix)]
    #[test]
    fn a_slow_compose_child_is_killed_at_the_host_deadline_only() {
        let dir = tempfile::TempDir::new().expect("dir");
        let script = slow_compose_script(dir.path());
        let program = script.to_str().expect("utf8 path");
        let started_at = Instant::now();
        let error = render_template(
            "t",
            &json!({}),
            ComposeProgram::new(program),
            Some(started_at + Duration::from_millis(1500)),
        )
        .expect_err("timeout");
        assert!(
            started_at.elapsed() < Duration::from_secs(3),
            "host waited for the stub"
        );
        assert!(
            matches!(error, ComposeHostError::HostTimedOut { .. }),
            "{error}"
        );
        assert!(dir.path().join("started").is_file(), "stub did not start");
        assert!(
            !dir.path().join("finished").exists(),
            "stub ran past the deadline"
        );
        assert_reaped(dir.path());
    }

    #[test]
    fn a_missing_program_is_failed_not_timed_out() {
        let source = r#"
def cron_stop(hook):
    return {"action": "continue", "reason": "ok"}

def cron_tick(hook):
    render_template("Hello", {"n": 1})
    return None
"#;
        let done = run_injected(
            source,
            ComposeProgram::new("sc-cron-missing-compose-b10"),
            &[],
            Duration::from_secs(5),
        );
        assert!(done.session.is_none());
        let detail = failed_detail(done);
        assert!(detail.contains("COMPOSE_NOT_FOUND"), "{detail}");
        assert!(detail.contains("sc-cron-missing-compose-b10"), "{detail}");
        assert!(!detail.contains("HOOK_FAILED"), "{detail}");
    }

    #[test]
    fn hook_source_cannot_select_the_program_or_an_extra_host_function() {
        let source = r#"
def cron_stop(hook):
    return {"action": "continue", "reason": "ok"}

def cron_tick(hook):
    x = not_defined
    return x
"#;
        let generation = Generation::new(1).expect("generation");
        let payload = hook();
        let stop = crate::monty_run::execute_feed(
            None,
            source,
            MontyPhase::Stop,
            generation,
            Duration::from_secs(20),
            &payload,
        );
        assert!(matches!(stop.result, FeedResult::Continue));
        let done = crate::monty_run::execute_feed(
            stop.session,
            source,
            MontyPhase::Tick,
            generation,
            Duration::from_secs(20),
            &payload,
        );
        let detail = failed_detail(done);
        assert!(detail.contains("not_defined"), "{detail}");
        assert!(detail.contains("HOST_UNDEFINED"), "{detail}");
        assert!(!detail.contains("host function"), "{detail}");
        assert!(!detail.contains("sc-cron-missing-compose-b10"), "{detail}");
        assert!(!detail.contains("allowlisted"), "{detail}");
    }
}
