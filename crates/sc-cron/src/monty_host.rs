//! Host function `render_template` for a Monty cron feed.
//!
//! The feed stays inside Monty. This module spawns the `sc-compose` program
//! (or a test-injected program name) on a private temp root the hook cannot
//! choose, then returns the child's stdout. It does not start CPython.

use std::fmt;
use std::io::ErrorKind;
use std::process::Command;
use std::time::Instant;

use monty_types::{MontyObject, ObjectRef};
use serde_json::{Map, Number, Value};

/// How `render_template` failed before the feed records [`crate::RunOutcome::Failed`].
#[derive(Debug)]
pub(crate) enum ComposeHostError {
    /// `compose_program` was not found on `PATH` (or the injected path).
    ProgramNotFound {
        /// Program name or path that could not be started.
        program: String,
    },
    /// The temp root or the child process could not be created.
    SpawnFailed {
        /// Program name or path passed to the process spawn.
        program: String,
        /// Operating-system or JSON detail.
        detail: String,
    },
    /// The child ran and exited non-zero.
    NonZeroExit {
        /// Exit status, or `-1` when the process was ended by a signal.
        status: i32,
        /// Capped stderr text from the child.
        stderr: String,
    },
}

impl ComposeHostError {
    /// Returns what an operator can do about this failure.
    #[must_use]
    pub(crate) const fn recovery(&self) -> &'static str {
        match self {
            Self::ProgramNotFound { .. } => {
                "Install `sc-compose` on PATH or fix the injected program name in tests."
            }
            Self::SpawnFailed { .. } => {
                "Check permissions and disk space for temp roots under `tempfile::TempDir`."
            }
            Self::NonZeroExit { .. } => {
                "Fix template/context or run `sc-compose render` manually with the logged stderr."
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
            Self::SpawnFailed { program, detail } => {
                write!(
                    formatter,
                    "compose program {program} failed to start: {detail}"
                )
            }
            Self::NonZeroExit { status, stderr } => {
                write!(formatter, "compose exited {status}: {stderr}")
            }
        }
    }
}

impl std::error::Error for ComposeHostError {}

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
/// function returns. The hook does not choose the root.
///
/// # Errors
///
/// [`ComposeHostError::ProgramNotFound`] when the program cannot be found,
/// [`ComposeHostError::SpawnFailed`] when the temp root or the process cannot
/// be created, and [`ComposeHostError::NonZeroExit`] when the child exits
/// non-zero.
pub(crate) fn render_template(
    template: &str,
    context: &Value,
    compose_program: &str,
) -> Result<String, ComposeHostError> {
    let dir = tempfile::TempDir::new().map_err(|error| ComposeHostError::SpawnFailed {
        program: compose_program.to_owned(),
        detail: error.to_string(),
    })?;
    let root = dir.path();
    std::fs::write(root.join("t.j2"), template).map_err(|error| ComposeHostError::SpawnFailed {
        program: compose_program.to_owned(),
        detail: error.to_string(),
    })?;
    let context_bytes =
        serde_json::to_vec(context).map_err(|error| ComposeHostError::SpawnFailed {
            program: compose_program.to_owned(),
            detail: error.to_string(),
        })?;
    std::fs::write(root.join("v.json"), context_bytes).map_err(|error| {
        ComposeHostError::SpawnFailed {
            program: compose_program.to_owned(),
            detail: error.to_string(),
        }
    })?;
    let output = Command::new(compose_program)
        .current_dir(root)
        .arg("render")
        .arg("--root")
        .arg(root)
        .args(["--mode", "file", "--file", "t.j2", "--var-file", "v.json"])
        .output()
        .map_err(|error| spawn_error(compose_program, error))?;
    if !output.status.success() {
        return Err(ComposeHostError::NonZeroExit {
            status: output.status.code().unwrap_or(-1),
            stderr: stderr_text(&output.stderr),
        });
    }
    String::from_utf8(output.stdout).map_err(|_| ComposeHostError::SpawnFailed {
        program: compose_program.to_owned(),
        detail: "stdout was not utf-8".to_owned(),
    })
}

fn spawn_error(program: &str, error: std::io::Error) -> ComposeHostError {
    if error.kind() == ErrorKind::NotFound {
        ComposeHostError::ProgramNotFound {
            program: program.to_owned(),
        }
    } else {
        ComposeHostError::SpawnFailed {
            program: program.to_owned(),
            detail: error.to_string(),
        }
    }
}

fn stderr_text(bytes: &[u8]) -> String {
    const CAP: usize = 1024;
    let text = String::from_utf8_lossy(bytes);
    let trimmed = text.trim();
    if trimmed.len() <= CAP {
        return trimmed.to_owned();
    }
    let mut end = CAP;
    while !trimmed.is_char_boundary(end) {
        end -= 1;
    }
    format!("{}…", &trimmed[..end])
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
    use std::time::{Duration, Instant};

    use serde_json::{Value, json};

    use super::{
        ComposeHostError, host_budget_exhausted, json_to_monty, monty_value_to_json,
        render_template,
    };
    use crate::monty_run::{FeedResult, MontyPhase};
    use crate::{Generation, RunOutcome};

    fn hook() -> Value {
        json!({})
    }

    fn run_tick(source: &str) -> crate::monty_run::FeedDone {
        let generation = Generation::new(1).expect("generation");
        let budget = Duration::from_secs(20);
        let payload = hook();
        let stop = crate::monty_run::execute_feed(
            None,
            source,
            MontyPhase::Stop,
            generation,
            budget,
            &payload,
        );
        assert!(
            matches!(stop.result, FeedResult::Continue),
            "stop: {:?}",
            stop.result
        );
        crate::monty_run::execute_feed(
            stop.session,
            source,
            MontyPhase::Tick,
            generation,
            budget,
            &payload,
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
        assert!(missing.to_string().contains("sc-cron-missing-compose-b10"));
        assert_eq!(
            ComposeHostError::SpawnFailed {
                program: "sc-compose".to_owned(),
                detail: "disk".to_owned(),
            }
            .recovery(),
            "Check permissions and disk space for temp roots under `tempfile::TempDir`."
        );
        assert_eq!(
            ComposeHostError::NonZeroExit {
                status: 2,
                stderr: "syntax".to_owned(),
            }
            .recovery(),
            "Fix template/context or run `sc-compose render` manually with the logged stderr."
        );
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
        let error = render_template("Hello", &json!({}), "sc-cron-missing-compose-b10")
            .expect_err("missing");
        assert!(
            matches!(error, ComposeHostError::ProgramNotFound { ref program } if program == "sc-cron-missing-compose-b10")
        );

        let source = r#"
# sc-cron-test-compose-program: sc-cron-missing-should-not-run
def cron_stop(hook):
    return {"action": "continue", "reason": "ok"}

def cron_tick(hook):
    render_template("only-one-arg")
    return None
"#;
        let done = run_tick(source);
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
# sc-cron-test-host-fn: compose_file
def cron_stop(hook):
    return {"action": "continue", "reason": "ok"}

def cron_tick(hook):
    compose_file("t.j2", {})
    return None
"#;
        let detail = failed_detail(run_tick(other));
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
# sc-cron-test-compose-program: sc-cron-missing-compose-b10
def cron_stop(hook):
    return {"action": "continue", "reason": "ok"}

def cron_tick(hook):
    return None
"#;
        let done = run_tick(source);
        assert!(matches!(done.result, FeedResult::Outcome(RunOutcome::Ok)));
        assert!(done.session.is_some());
    }

    #[test]
    fn a_missing_program_on_the_feed_drops_the_repl() {
        let source = r#"
# sc-cron-test-compose-program: sc-cron-missing-compose-b10
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
        let first = run_tick(source);
        assert!(first.session.is_none());
        let detail = failed_detail(first);
        assert!(detail.contains("sc-cron-missing-compose-b10"), "{detail}");
        assert!(!detail.contains("HOOK_FAILED"), "{detail}");
        assert!(!detail.contains("repl kept"), "{detail}");

        let generation = Generation::new(1).expect("generation");
        let payload = hook();
        let second = crate::monty_run::execute_feed(
            None,
            source,
            MontyPhase::Tick,
            generation,
            Duration::from_secs(20),
            &payload,
        );
        let detail = failed_detail(second);
        assert!(detail.contains("sc-cron-missing-compose-b10"), "{detail}");
        assert!(!detail.contains("repl kept"), "{detail}");
    }

    #[test]
    fn sc_compose_renders_the_template_stdout() {
        let rendered = render_template("Hello {{ name }}", &json!({"name": "cron"}), "sc-compose")
            .expect("sc-compose");
        assert!(
            rendered == "Hello cron\n" || rendered == "Hello cron",
            "{rendered:?}"
        );
        assert!(!rendered.contains("INFO"), "{rendered:?}");
    }

    #[cfg(unix)]
    fn executable_script(dir: &std::path::Path, name: &str, body: &str) -> std::path::PathBuf {
        use std::os::unix::fs::PermissionsExt;
        let path = dir.join(name);
        std::fs::write(&path, body).expect("script");
        let mut permissions = std::fs::metadata(&path).expect("meta").permissions();
        permissions.set_mode(0o755);
        std::fs::set_permissions(&path, permissions).expect("chmod");
        path
    }

    #[cfg(unix)]
    #[test]
    fn an_injected_program_sees_the_render_argv_and_can_resume_twice() {
        let dir = tempfile::TempDir::new().expect("dir");
        let log = dir.path().join("argv.txt");
        let log_quoted = format!("'{}'", log.display().to_string().replace('\'', "'\\''"));
        let script = executable_script(
            dir.path(),
            "compose-stub",
            &format!("#!/bin/sh\nprintf '%s\\n' \"$@\" > {log_quoted}\nprintf 'rendered'\n"),
        );
        let program = script.to_str().expect("utf8 path");
        let rendered = render_template("ignored", &json!({"n": 1}), program).expect("stub");
        assert_eq!(rendered, "rendered");
        let argv = std::fs::read_to_string(&log).expect("argv");
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

        let source = format!(
            r#"
# sc-cron-test-compose-program: {program}
def cron_stop(hook):
    return {{"action": "continue", "reason": "ok"}}

def cron_tick(hook):
    a = render_template("one", {{"n": 1}})
    b = render_template("two", {{"n": 2}})
    if a != "rendered" or b != "rendered":
        raise RuntimeError(a + "|" + b)
    return None
"#
        );
        let done = run_tick(&source);
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
        let script = executable_script(
            dir.path(),
            "fail-stub",
            "#!/bin/sh\nprintf 'no-template\\n' >&2\nexit 7\n",
        );
        let program = script.to_str().expect("utf8 path");
        let error = render_template("t", &json!({}), program).expect_err("exit");
        match error {
            ComposeHostError::NonZeroExit { status, stderr } => {
                assert_eq!(status, 7);
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
        let error = render_template("t", &json!({}), program).expect_err("spawn");
        assert!(
            matches!(error, ComposeHostError::SpawnFailed { .. }),
            "{error}"
        );
    }
}
