//! Monty host functions for the `sc-compose` CLI.
//!
//! Production registers [`render_template`] (string-template sugar) and
//! [`sc_compose_run`] (any allowlisted subcommand). Tests pass another
//! [`ComposeProgram`]. The feed does not start CPython.

use std::fmt;
use std::time::Instant;

use monty_types::{MontyObject, ObjectRef};
use serde_json::{Map, Number, Value};

pub(crate) use crate::monty_compose_common::ComposeHostError;

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
/// host budget expires while waiting on the compose group, and
/// [`ComposeHostError::NonZeroExit`] when the child exits non-zero.
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

/// Subcommands exposed through [`sc_compose_run`]. Kept in sync with `sc-compose --help`.
pub(crate) const SC_COMPOSE_SUBCOMMANDS: &[&str] = &[
    "bead",
    "examples",
    "extract",
    "frontmatter-init",
    "help",
    "init",
    "lint",
    "observability-health",
    "render",
    "reports",
    "resolve",
    "template-init",
    "templates",
    "validate",
    "verify",
];

#[must_use]
pub(crate) fn sc_compose_subcommand_allowed(subcommand: &str) -> bool {
    SC_COMPOSE_SUBCOMMANDS.contains(&subcommand)
}

/// Runs `sc-compose <subcommand> …` with hook-supplied argv tail.
///
/// Non-zero exit is returned in [`crate::monty_compose::ComposeRunResult`].
pub(crate) fn sc_compose_run(
    subcommand: &str,
    args: &[String],
    compose_program: ComposeProgram<'_>,
    host_deadline: Option<Instant>,
) -> Result<crate::monty_compose::ComposeRunResult, ComposeHostError> {
    if !sc_compose_subcommand_allowed(subcommand) {
        return Err(ComposeHostError::SubcommandDenied {
            subcommand: subcommand.to_owned(),
        });
    }
    let program = compose_program.as_str();
    let mut argv = Vec::with_capacity(1 + args.len());
    argv.push(subcommand.to_owned());
    argv.extend_from_slice(args);
    crate::monty_compose::run_compose_argv(program, None, &argv, host_deadline)
}

#[must_use]
pub(crate) fn compose_run_to_monty(result: &crate::monty_compose::ComposeRunResult) -> MontyObject {
    let mut map = Map::new();
    map.insert("stdout".to_owned(), Value::String(result.stdout.clone()));
    map.insert("stderr".to_owned(), Value::String(result.stderr.clone()));
    map.insert(
        "exit_code".to_owned(),
        result
            .exit_code
            .map(|code| Value::Number(Number::from(code)))
            .unwrap_or(Value::Null),
    );
    json_to_monty(&Value::Object(map))
}

fn stage_failed(program: &str, source: std::io::Error) -> ComposeHostError {
    ComposeHostError::StageFailed {
        program: program.to_owned(),
        source,
    }
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
    use std::sync::mpsc;
    use std::thread;
    use std::time::{Duration, Instant};

    use serde_json::{Value, json};

    use super::{
        ComposeHostError, ComposeProgram, host_budget_exhausted, json_to_monty,
        monty_value_to_json, render_template,
    };
    use crate::monty_compose_common::stderr_text;
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

    const STOP_BUDGET: Duration = Duration::from_secs(20);
    const INJECTED_GUARD_SLACK: Duration = Duration::from_secs(5);

    fn run_injected(
        source: &str,
        program: ComposeProgram<'_>,
        extra_host_fns: &[&str],
        budget: Duration,
    ) -> crate::monty_run::FeedDone {
        let outer = STOP_BUDGET
            .saturating_add(budget)
            .saturating_add(INJECTED_GUARD_SLACK);
        let (tx, rx) = mpsc::sync_channel(1);
        thread::scope(|scope| {
            scope.spawn(|| {
                let _ = tx.send(run_injected_on_thread(
                    source,
                    program,
                    extra_host_fns,
                    budget,
                ));
            });
            rx.recv_timeout(outer).unwrap_or_else(|_| {
                panic!("run_injected did not finish within {outer:?} (stop+tick guard)");
            })
        })
    }

    fn run_injected_on_thread(
        source: &str,
        program: ComposeProgram<'_>,
        extra_host_fns: &[&str],
        budget: Duration,
    ) -> crate::monty_run::FeedDone {
        let generation = Generation::new(1).expect("generation");
        let payload = hook();
        let stop_deadline = Instant::now()
            .checked_add(STOP_BUDGET)
            .expect("stop deadline");
        let stop = crate::monty_run::execute_feed_with_compose_program(
            None,
            source,
            MontyPhase::Stop,
            generation,
            STOP_BUDGET,
            &payload,
            ComposeInjection::production(),
            stop_deadline,
        );
        assert!(
            matches!(stop.result, FeedResult::Continue),
            "stop: {:?}",
            stop.result
        );
        let tick_deadline = Instant::now().checked_add(budget).expect("tick deadline");
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
            tick_deadline,
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
        let text = stderr_text(&bytes);
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
        let run_deadline = Instant::now()
            .checked_add(Duration::from_secs(20))
            .expect("run deadline");
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
            run_deadline,
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

    #[test]
    fn sc_compose_run_denies_an_unknown_subcommand() {
        let source = r#"
def cron_stop(hook):
    return {"action": "continue", "reason": "ok"}

def cron_tick(hook):
    sc_compose_run("not-an-allowlisted-subcommand", [])
    return None
"#;
        let detail = failed_detail(run_tick(source));
        assert!(detail.contains("COMPOSE_SUBCOMMAND"), "{detail}");
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
        let started = dir.path().join("started");
        let script = linked_fixture(dir.path(), "slow-stub", "compose_slow_touch.sh");
        let program = script.to_str().expect("utf8 path");
        let source = r#"
def cron_stop(hook):
    return {"action": "continue", "reason": "ok"}

def cron_tick(hook):
    render_template("Hello", {"n": 1})
    return None
"#;
        let done = run_injected(
            source,
            ComposeProgram::new(program),
            &[],
            Duration::from_secs(2),
        );
        assert!(started.is_file(), "stub did not start");
        assert!(done.session.is_none());
        assert!(
            matches!(done.result, FeedResult::Outcome(RunOutcome::TimedOut)),
            "{:?}",
            done.result
        );
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
        let stop_deadline = Instant::now()
            .checked_add(Duration::from_secs(20))
            .expect("stop deadline");
        let stop = crate::monty_run::execute_feed(
            None,
            source,
            MontyPhase::Stop,
            generation,
            Duration::from_secs(20),
            &payload,
            stop_deadline,
        );
        assert!(matches!(stop.result, FeedResult::Continue));
        let tick_deadline = Instant::now()
            .checked_add(Duration::from_secs(20))
            .expect("tick deadline");
        let done = crate::monty_run::execute_feed(
            stop.session,
            source,
            MontyPhase::Tick,
            generation,
            Duration::from_secs(20),
            &payload,
            tick_deadline,
        );
        let detail = failed_detail(done);
        assert!(detail.contains("not_defined"), "{detail}");
        assert!(detail.contains("HOST_UNDEFINED"), "{detail}");
        assert!(!detail.contains("host function"), "{detail}");
        assert!(!detail.contains("sc-cron-missing-compose-b10"), "{detail}");
        assert!(!detail.contains("allowlisted"), "{detail}");
    }
}
