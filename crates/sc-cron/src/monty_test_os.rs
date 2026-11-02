//! Host replies for Monty `OsCall` suspensions in `monty_hook` handshakes.
//!
//! Enabled only with feature `monty-test-handshake`. Production embedders leave
//! every `OsCall` denied; handshake sources may touch files under the temp dir.

use std::fs::OpenOptions;
use std::path::{Path, PathBuf};
use std::time::Instant;

use monty::{ReplOsCall, ReplProgress};
use monty_types::{
    ExcType, ExtFunctionResult, MontyException, MontyObject, OpenCallArgs, OsFunctionCall,
    PrintWriter,
};

use crate::RunOutcome;
use crate::monty_host::host_budget_exhausted;
use crate::monty_run::{FeedDone, FeedResult};

/// When `monty-test-handshake` is on, OS calls outside the temp dir still use
/// production-style denial so unit tests in `monty_host` stay unchanged.
pub(crate) fn os_call_allowed_in_tests(call: &ReplOsCall) -> bool {
    let path = match &call.function_call {
        OsFunctionCall::Exists(path) | OsFunctionCall::Open(OpenCallArgs { path, .. }) => {
            path.as_str()
        }
        OsFunctionCall::WriteText(args) | OsFunctionCall::AppendText(args) => args.path.as_str(),
        _ => return false,
    };
    allowed_path(path)
}

fn allowed_path(path: &str) -> bool {
    let Ok(temp) = std::env::temp_dir().canonicalize() else {
        return false;
    };
    let path = PathBuf::from(path);
    if path.starts_with(&temp) {
        return true;
    }
    if let Ok(canonical) = path.canonicalize() {
        return canonical.starts_with(&temp);
    }
    // macOS often hands Monty `/var/...` while `temp_dir().canonicalize()` is `/private/var/...`.
    let temp_text = temp.to_string_lossy();
    let path_text = path.to_string_lossy();
    if let Some(suffix) = temp_text.strip_prefix("/private") {
        return path_text.starts_with(suffix);
    }
    false
}

fn file_not_found(path: &str) -> MontyException {
    MontyException::new(
        ExcType::FileNotFoundError,
        Some(format!("No such file or directory: {path:?}")),
    )
}

fn os_exception(exc_type: ExcType, message: impl Into<String>) -> MontyException {
    MontyException::new(exc_type, Some(message.into()))
}

fn write_text(args: monty_types::PathStringDataArgs) -> ExtFunctionResult {
    if !allowed_path(args.path.as_str()) {
        return ExtFunctionResult::Error(os_exception(
            ExcType::PermissionError,
            "path outside test temp dir",
        ));
    }
    let len = args.data.len();
    match std::fs::write(args.path.as_str(), &args.data) {
        Ok(()) => ExtFunctionResult::Return(MontyObject::int(
            i64::try_from(len).expect("write length fits in i64"),
        )),
        Err(error) => ExtFunctionResult::Error(os_exception(ExcType::OSError, error.to_string())),
    }
}

fn dispatch(call: OsFunctionCall) -> ExtFunctionResult {
    match call {
        OsFunctionCall::Exists(path) => {
            if !allowed_path(path.as_str()) {
                return ExtFunctionResult::Error(os_exception(
                    ExcType::PermissionError,
                    "path outside test temp dir",
                ));
            }
            ExtFunctionResult::Return(MontyObject::bool(Path::new(path.as_str()).exists()))
        }
        OsFunctionCall::WriteText(args) | OsFunctionCall::AppendText(args) => write_text(args),
        OsFunctionCall::Open(args) => open(args),
        other => ExtFunctionResult::Error(os_exception(
            ExcType::PermissionError,
            format!("test os denied {}", other.name()),
        )),
    }
}

fn open(args: OpenCallArgs) -> ExtFunctionResult {
    if !allowed_path(args.path.as_str()) {
        return ExtFunctionResult::Error(os_exception(
            ExcType::PermissionError,
            "path outside test temp dir",
        ));
    }
    let path = args.path.as_str();
    if args.mode.readable() && !args.mode.writable() && !Path::new(path).exists() {
        return ExtFunctionResult::Error(file_not_found(path));
    }
    if args.mode.create() {
        if let Some(parent) = Path::new(path).parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let mut options = OpenOptions::new();
        if args.mode.writable() {
            options.write(true);
        }
        if args.mode.create() {
            options.create(true);
        }
        if args.mode.truncate() {
            options.truncate(true);
        }
        if args.mode.readable() {
            options.read(true);
        }
        match options.open(path) {
            Ok(file) => {
                if args.mode.truncate() {
                    let _ = file.set_len(0);
                }
            }
            Err(error) => {
                return ExtFunctionResult::Error(os_exception(ExcType::OSError, error.to_string()));
            }
        }
    }
    ExtFunctionResult::Return(MontyObject::file_handle(monty_types::MontyFileHandle {
        path: path.to_owned(),
        mode: args.mode,
        position: 0,
    }))
}

pub(crate) fn resume_test_os(
    call: ReplOsCall,
    host_deadline: Option<Instant>,
) -> Result<ReplProgress, FeedDone> {
    if host_budget_exhausted(host_deadline) {
        return Err(FeedDone {
            session: None,
            result: FeedResult::Outcome(RunOutcome::TimedOut),
        });
    }
    call.resume_with(PrintWriter::Disabled, dispatch)
        .map_err(|error| FeedDone {
            session: None,
            result: FeedResult::Outcome(RunOutcome::Failed(format!(
                "monty test os resume failed: {error:?}"
            ))),
        })
}

#[cfg(test)]
mod tests {
    use std::time::{Duration, Instant};

    use crate::Generation;
    use crate::monty_run::{FeedResult, MontyPhase, execute_feed};

    const HOLD_SNIPPET: &str = r#"
def cron_stop(hook):
    payload = hook["payload"]
    open(payload["hold"], "w").write("stop")
    while True:
        try:
            open(payload["release"]).close()
            break
        except OSError:
            pass
    return {"action": "continue", "reason": "ok"}

def cron_tick(hook):
    return None
"#;

    #[test]
    fn hold_handshake_writes_the_hold_file() {
        let dir = tempfile::TempDir::new().expect("dir");
        let hold = dir.path().join("hold");
        let release = dir.path().join("release");
        let hook = serde_json::json!({
            "protocol": "sc-cron.hook.v1",
            "event": "cron.stop",
            "schedule": "sync",
            "payload": {
                "hold": hold.display().to_string(),
                "release": release.display().to_string(),
            },
            "deadline": "2026-10-19T00:01:00Z",
        });
        let deadline = Instant::now() + Duration::from_secs(5);
        std::fs::write(&release, b"go").expect("release");
        let done = execute_feed(
            None,
            HOLD_SNIPPET,
            MontyPhase::Stop,
            Generation::new(1).expect("gen"),
            Duration::from_secs(5),
            &hook,
            deadline,
        );
        assert!(
            matches!(done.result, FeedResult::Continue),
            "{:?}",
            done.result
        );
        assert!(hold.is_file(), "hold file missing");
    }
}
