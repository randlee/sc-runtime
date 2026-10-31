use super::*;

async fn run_policy(policy: HookEnvironment, host: HookEnvironmentHost) -> RunOutcome {
    let root = tempfile::tempdir().expect("root");
    let module = python_file(
        root.path(),
        "import sys, json\nhook=json.load(sys.stdin)\nif hook['event']=='cron.stop': print('{\"action\":\"continue\"}')\n",
    );
    let mut fixture = common::Fixture::new();
    fixture.scheduler = fixture.scheduler.with_hook_environment(host);
    fixture.start(spec(
        "environment",
        HookTarget::python(module).expect("hook"),
        policy,
    ));
    due(&fixture).await
}

#[tokio::test]
async fn file_authorization_precedes_absence_and_accepts_exact_files_or_roots() {
    let root = tempfile::tempdir().expect("root");
    let path = root.path().join("missing/nested.env");
    let optional = HookEnvironment::new()
        .dotenv(&path, false)
        .expect("optional");
    assert_code(
        &run_policy(optional.clone(), HookEnvironmentHost::new()).await,
        "ENV_FILE_DENIED",
    );
    let host = HookEnvironmentHost::new()
        .allow_dotenv_root(root.path())
        .expect("root allow");
    assert_eq!(run_policy(optional, host.clone()).await, RunOutcome::Ok);
    assert_code(
        &run_policy(
            HookEnvironment::new()
                .dotenv(&path, true)
                .expect("required"),
            host,
        )
        .await,
        "ENV_FILE_MISSING",
    );
    let path = root.path().join("present.env");
    std::fs::write(
        &path,
        "A=one\r\nexport B = 'two # literal'\nC=\"escaped\\nline\\t\\\\\\\"\" # comment\nEMPTY=\n",
    )
    .expect("file");
    let host = HookEnvironmentHost::new()
        .allow_dotenv_file(&path)
        .expect("exact");
    assert_eq!(
        run_policy(
            HookEnvironment::new().dotenv(&path, true).expect("source"),
            host
        )
        .await,
        RunOutcome::Ok
    );
    assert_code(
        &run_policy(
            HookEnvironment::new()
                .dotenv(root.path(), true)
                .expect("directory"),
            HookEnvironmentHost::new()
                .allow_dotenv_file(root.path())
                .expect("allow"),
        )
        .await,
        "ENV_FILE_INVALID",
    );
}

#[cfg(unix)]
#[tokio::test]
async fn canonical_symlinks_cannot_escape_file_or_root_policy() {
    let allowed = tempfile::tempdir().expect("allowed");
    let outside = tempfile::tempdir().expect("outside");
    let target = outside.path().join("secret.env");
    std::fs::write(&target, "A=canary").expect("target");
    let link = allowed.path().join("link.env");
    std::os::unix::fs::symlink(&target, &link).expect("link");
    assert_code(
        &run_policy(
            HookEnvironment::new().dotenv(&link, true).expect("source"),
            HookEnvironmentHost::new()
                .allow_dotenv_root(allowed.path())
                .expect("root"),
        )
        .await,
        "ENV_FILE_DENIED",
    );
    assert_eq!(
        run_policy(
            HookEnvironment::new().dotenv(&link, true).expect("source"),
            HookEnvironmentHost::new()
                .allow_dotenv_file(&target)
                .expect("exact target")
        )
        .await,
        RunOutcome::Ok
    );
    let missing = allowed.path().join("via-link");
    std::os::unix::fs::symlink(outside.path(), &missing).expect("directory symlink");
    assert_code(
        &run_policy(
            HookEnvironment::new()
                .dotenv(missing.join("absent.env"), false)
                .expect("source"),
            HookEnvironmentHost::new()
                .allow_dotenv_root(allowed.path())
                .expect("root"),
        )
        .await,
        "ENV_FILE_DENIED",
    );
}

#[tokio::test]
async fn dotenv_never_evaluates_and_rejects_malformed_duplicate_or_oversize_data() {
    let root = tempfile::tempdir().expect("root");
    let path = root.path().join("input.env");
    let cases: Vec<Vec<u8>> = vec![
        b"A=x\nA=y".to_vec(),
        b"A=x\na=y".to_vec(),
        b"A=$HOME".to_vec(),
        b"A=${HOME}".to_vec(),
        b"A=$(id)".to_vec(),
        b"A=`id`".to_vec(),
        b"A='unterminated".to_vec(),
        b"A=\"bad\\x\"".to_vec(),
        b"A=\"a\" tail".to_vec(),
        b"A='multi\nline'".to_vec(),
        b"1A=x".to_vec(),
        b"A=x\0".to_vec(),
        vec![b'A', b'=', 255],
        format!("A={}", "x".repeat(MAX_ENV_VALUE_BYTES + 1)).into_bytes(),
    ];
    for input in cases {
        std::fs::write(&path, input).expect("input");
        let outcome = run_policy(
            HookEnvironment::new().dotenv(&path, true).expect("policy"),
            HookEnvironmentHost::new()
                .allow_dotenv_file(&path)
                .expect("host"),
        )
        .await;
        assert_code(&outcome, "ENV_FILE_INVALID");
    }
    let exact = format!("#{}", "x".repeat(MAX_ENV_FILE_BYTES - 1));
    std::fs::write(&path, &exact).expect("exact file");
    assert_eq!(
        run_policy(
            HookEnvironment::new().dotenv(&path, true).expect("policy"),
            HookEnvironmentHost::new()
                .allow_dotenv_file(&path)
                .expect("host")
        )
        .await,
        RunOutcome::Ok
    );
    std::fs::write(&path, format!("{exact}x")).expect("over file");
    assert_code(
        &run_policy(
            HookEnvironment::new().dotenv(&path, true).expect("policy"),
            HookEnvironmentHost::new()
                .allow_dotenv_file(&path)
                .expect("host"),
        )
        .await,
        "ENV_FILE_INVALID",
    );
}

#[tokio::test]
async fn resolver_success_missing_denied_unavailable_and_invalid_results_are_value_free() {
    for reference in ["missing", "denied", "unavailable", "oversize"] {
        let resolver = Arc::new(RecordingResolver::default());
        let host = HookEnvironmentHost::new().with_reference_resolver(resolver.clone());
        let outcome = run_policy(
            HookEnvironment::new()
                .reference("KEY", reference)
                .expect("reference"),
            host,
        )
        .await;
        assert_code(&outcome, "ENV_REFERENCE");
        assert_eq!(resolver.calls.lock().expect("calls").len(), 1);
        assert!(!outcome.to_string().contains(reference));
    }
    assert_code(
        &run_policy(
            HookEnvironment::new()
                .reference("KEY", "opaque-canary-locator")
                .expect("reference"),
            HookEnvironmentHost::new(),
        )
        .await,
        "ENV_REFERENCE",
    );
    assert_eq!(
        run_policy(
            HookEnvironment::new()
                .reference("KEY", "at-limit")
                .expect("reference"),
            HookEnvironmentHost::new()
                .with_reference_resolver(Arc::new(RecordingResolver::default()))
        )
        .await,
        RunOutcome::Ok
    );
}

struct DeadlineResolver(tokio::sync::Notify);
#[async_trait::async_trait]
impl EnvironmentReferenceResolver for DeadlineResolver {
    async fn resolve(
        &self,
        _: EnvironmentReferenceRequest<'_>,
    ) -> Result<Option<String>, EnvironmentReferenceError> {
        self.0.notify_one();
        std::future::pending().await
    }
}
#[tokio::test]
async fn pending_resolver_is_cancelled_at_the_row_deadline_before_source_execution() {
    let resolver = Arc::new(DeadlineResolver(tokio::sync::Notify::new()));
    let mut fixture = common::Fixture::new();
    fixture.scheduler = fixture.scheduler.with_hook_environment(
        HookEnvironmentHost::new().with_reference_resolver(resolver.clone()),
    );
    fixture.start(spec(
        "deadline",
        HookTarget::python("does-not-exist.py").expect("hook"),
        HookEnvironment::new()
            .reference("KEY", "opaque")
            .expect("reference"),
    ));
    fixture.clock.advance(chrono::Duration::seconds(60));
    let run = fixture.scheduler.run_due(common::idle_job());
    tokio::pin!(run);
    tokio::select! {_=resolver.0.notified()=>{}, result=&mut run=>panic!("resolver should remain pending: {result:?}")}
    fixture.clock.advance(chrono::Duration::seconds(21));
    let rows = tokio::time::timeout(common::WAIT, run)
        .await
        .expect("bounded")
        .expect("due");
    assert_eq!(rows[0].outcome, RunOutcome::TimedOut);
}

#[tokio::test]
async fn final_snapshot_block_ceiling_and_resolved_entry_limit_are_portable() {
    // A=...\0B=...\0\0 is exactly the portable encoded ceiling.
    let a = "a".repeat(MAX_ENV_VALUE_BYTES);
    let b = "b".repeat(MAX_ENV_SNAPSHOT_BYTES - a.len() - 7);
    let exact = HookEnvironment::new()
        .set("A", &a)
        .expect("A")
        .set("B", &b)
        .expect("B");
    assert_eq!(
        run_policy(exact, HookEnvironmentHost::new()).await,
        RunOutcome::Ok
    );
    let overflow = HookEnvironment::new()
        .set("A", &a)
        .expect("A")
        .set("B", format!("{b}b"))
        .expect("B");
    assert_code(
        &run_policy(overflow, HookEnvironmentHost::new()).await,
        "ENV_SNAPSHOT_LIMIT",
    );
    let root = tempfile::tempdir().expect("root");
    let path = root.path().join("entries.env");
    let mut text = String::new();
    for n in 0..MAX_ENV_ENTRIES {
        text.push_str(&format!("N{n}=x\n"));
    }
    std::fs::write(&path, &text).expect("entries");
    assert_eq!(
        run_policy(
            HookEnvironment::new().dotenv(&path, true).expect("source"),
            HookEnvironmentHost::new()
                .allow_dotenv_file(&path)
                .expect("host")
        )
        .await,
        RunOutcome::Ok
    );
    text.push_str("EXTRA=x\n");
    std::fs::write(&path, text).expect("over entries");
    assert_code(
        &run_policy(
            HookEnvironment::new().dotenv(&path, true).expect("source"),
            HookEnvironmentHost::new()
                .allow_dotenv_file(&path)
                .expect("host"),
        )
        .await,
        "ENV_SNAPSHOT_LIMIT",
    );
}

#[test]
fn daemon_environment_is_read_only_at_execution_and_python_never_inherits_unlisted_values() {
    let output = std::process::Command::new(std::env::current_exe().expect("self"))
        .args([
            "--exact",
            "resolution::process_environment_child",
            "--nocapture",
        ])
        .env("SC_CRON_ENV_CHILD", "python")
        .env("SC_ENV_INHERITED", "daemon-value")
        .env("SC_ENV_UNLISTED", "unlisted-secret")
        .env("ATM_IDENTITY", "daemon-identity")
        .env("ATM_TEAM", "daemon-team")
        .output()
        .expect("child");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}
#[test]
fn process_environment_child() {
    if std::env::var("SC_CRON_ENV_CHILD").as_deref() != Ok("python") {
        return;
    }
    tokio::runtime::Runtime::new().expect("runtime").block_on(async {
        let root=tempfile::tempdir().expect("root");let first=root.path().join("first.env");let second=root.path().join("second.env");
        std::fs::write(&first,"SC_ENV_INHERITED=file-one\nCLEAR_ME=file\nORDER=first\n").expect("first");
        std::fs::write(&second,"ORDER=second\n").expect("second");
        let source=format!("import os,json,sys\nhook=json.load(sys.stdin)\nassert os.getenv('ATM_IDENTITY')=='planner-7'\nassert os.getenv('ATM_TEAM')=='runtime'\nassert os.getenv('SC_ENV_INHERITED')=='file-one'\nassert os.getenv('ORDER')=='second'\nassert os.getenv('REFERENCE')=='reference-canary'\nassert os.getenv('CLEAR_ME') is None\nassert os.getenv('SC_ENV_UNLISTED') is None\nassert os.getenv('SC_CRON_ENV_CHILD') is None\nassert os.getenv('PATH') is None\nif hook['event']=='cron.stop':\n    open({},'w').write('ORDER=changed\\n')\n    print('{{\"action\":\"continue\"}}')\n",serde_json::to_string(&second.to_string_lossy()).expect("path"));
        let module=python_file(root.path(),&source);
        let resolver=Arc::new(RecordingResolver::default());
        let host=HookEnvironmentHost::new().allow_process(["SC_ENV_INHERITED","ATM_IDENTITY","ATM_TEAM"]).expect("allow").allow_dotenv_root(root.path()).expect("root").with_reference_resolver(resolver.clone());
        let policy=HookEnvironment::new().dotenv(&first,true).expect("first").dotenv(&second,true).expect("second").set("ATM_IDENTITY","planner-7").expect("identity").set("ATM_TEAM","runtime").expect("team").clear("CLEAR_ME").expect("clear").reference("REFERENCE","value:reference-canary").expect("reference");
        let mut fixture=common::Fixture::new();fixture.scheduler=fixture.scheduler.with_hook_environment(host);
        let row=fixture.start(spec("python-parity",HookTarget::python(module).expect("hook"),policy.clone()));
        assert_eq!(due(&fixture).await,RunOutcome::Ok);
        assert_eq!(resolver.calls.lock().expect("calls").len(),1,"stop and tick share one resolution");
        assert_eq!(fixture.row(&row).spec.environment,policy,"resolution never writes back values");
    });
}

#[test]
fn invalid_nonunicode_or_oversized_allowlisted_process_values_fail_without_capture() {
    let mut cases = vec![std::ffi::OsString::from(
        "x".repeat(MAX_ENV_VALUE_BYTES + 1),
    )];
    #[cfg(unix)]
    {
        use std::os::unix::ffi::OsStringExt;
        cases.push(std::ffi::OsString::from_vec(vec![255]));
    }
    for value in cases {
        let output = std::process::Command::new(std::env::current_exe().expect("self"))
            .args(["--exact", "resolution::invalid_process_child"])
            .env("SC_CRON_ENV_CHILD", "invalid")
            .env("SC_ENV_INVALID", value)
            .output()
            .expect("child");
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
}
#[test]
fn invalid_process_child() {
    if std::env::var("SC_CRON_ENV_CHILD").as_deref() != Ok("invalid") {
        return;
    }
    tokio::runtime::Runtime::new()
        .expect("runtime")
        .block_on(async {
            let host = HookEnvironmentHost::new()
                .allow_process(["SC_ENV_INVALID"])
                .expect("allow");
            assert_code(
                &run_policy(HookEnvironment::new(), host).await,
                "ENV_PROCESS_INVALID",
            );
        });
}

#[tokio::test]
async fn python_default_only_explicit_inherit_raw_output_and_debug_contract() {
    let root = tempfile::tempdir().expect("root");
    let module = python_file(
        root.path(),
        "import os\nprint(os.getenv('PATH','absent'))\n",
    );
    let hook = PythonHook::new(&module).expect("hook");
    assert!(
        matches!(hook.env(),process::ChildEnv::Only{keep,set} if keep.is_empty()&&set.is_empty())
    );
    let name = ScheduleName::new("direct").expect("name");
    let timeout = RunTimeout::new(BUDGET).expect("timeout");
    let payload = json!({});
    let call = HookCall {
        event: HookEvent::Tick,
        schedule: &name,
        payload: &payload,
        timeout,
        deadline: Instant::now() + BUDGET,
    };
    let output = hook.invoke(&SystemClock, &call).await.expect("invoke");
    assert_eq!(output.stdout.trim(), "absent");
    let inherited = hook
        .with_env(process::ChildEnv::Inherit)
        .invoke(&SystemClock, &call)
        .await
        .expect("inherit");
    assert_ne!(inherited.stdout.trim(), "absent");
    let raw = "direct-secret-canary";
    let env = process::ChildEnv::Only {
        keep: vec![],
        set: vec![("KEY".into(), raw.into())],
    };
    let module = python_file(
        root.path(),
        "import os,sys\nprint(os.getenv('KEY'))\nprint(os.getenv('KEY'),file=sys.stderr)\n",
    );
    let hook = PythonHook::new(module).expect("hook").with_env(env.clone());
    let output = hook.invoke(&SystemClock, &call).await.expect("raw");
    assert!(output.stdout.contains(raw) && output.stderr.contains(raw));
    let spec = process::ChildSpec {
        program: std::path::Path::new(python_program()),
        args: vec![raw.into()],
        cwd: root.path(),
        input: raw.as_bytes(),
        env,
    };
    for debug in [
        format!("{output:?}"),
        format!("{hook:?}"),
        format!("{spec:?}"),
    ] {
        assert!(!debug.contains(raw));
    }
}

#[tokio::test]
async fn scheduled_python_details_reasons_and_events_redact_values_before_fixed_codes() {
    for value in ["ENV", "HOST", ":", "1", "long-canary"] {
        for stop in [false, true] {
            let root = tempfile::tempdir().expect("root");
            let source = if stop {
                "import os,json,sys\nhook=json.load(sys.stdin)\nprint(json.dumps({'action':'disable','reason':os.getenv('KEY')}))\n"
            } else {
                "import os,json,sys\nhook=json.load(sys.stdin)\nif hook['event']=='cron.stop': print('{\"action\":\"continue\"}')\nelse:\n print(os.getenv('KEY'),file=sys.stderr)\n sys.exit(1)\n"
            };
            let module = python_file(root.path(), source);
            let recording = Arc::new(sc_observability_types::RecordingCanonicalLog::new(
                sc_observability_types::ServiceName::new("canary").expect("service"),
            ));
            let store = Arc::new(
                SqliteScheduleStore::open_in_memory(ScheduleLimits::default()).expect("store"),
            );
            let clock = Arc::new(ManualClock::new(common::at("2026-10-19T00:00:00Z")));
            let scheduler = Scheduler::new_with_logger(
                store.clone(),
                clock.clone(),
                Arc::new(SeqJitter::default()),
                recording.clone(),
            );
            let row = scheduler
                .start(spec(
                    "canary",
                    HookTarget::python(module).expect("hook"),
                    HookEnvironment::new().set("KEY", value).expect("set"),
                ))
                .expect("start");
            clock.advance(chrono::Duration::seconds(60));
            let runs = scheduler.run_due(common::idle_job()).await.expect("due");
            let detail = runs[0].outcome.to_string();
            assert!(detail.contains("[REDACTED_ENV]"), "{detail}");
            if stop {
                assert_eq!(
                    runs[0].outcome,
                    RunOutcome::Refused("[REDACTED_ENV]".into())
                );
                assert_eq!(
                    store.get(row.id).expect("get").expect("row").last_outcome,
                    Some(runs[0].outcome.clone())
                );
            }
            let terminal = recording
                .events()
                .into_iter()
                .find(|e| e.action.as_str() == "due.finished")
                .expect("terminal");
            assert_eq!(
                terminal.diagnostic.as_ref().map(|d| d.code.as_str()),
                if stop { None } else { Some("JOB_FAILED") }
            );
            assert_eq!(terminal.fields["schedule"], row.id.to_string());
        }
    }
}

#[tokio::test]
async fn cross_layer_ascii_case_collisions_are_rejected_for_set_reference_and_clear() {
    let root = tempfile::tempdir().expect("root");
    let path = root.path().join("case.env");
    std::fs::write(&path, "Name=value\n").expect("file");
    let host = HookEnvironmentHost::new()
        .allow_dotenv_file(&path)
        .expect("file policy")
        .with_reference_resolver(Arc::new(RecordingResolver::default()));
    for policy in [
        HookEnvironment::new().set("NAME", "new").expect("set"),
        HookEnvironment::new()
            .reference("NAME", "value:result")
            .expect("reference"),
        HookEnvironment::new().clear("NAME").expect("clear"),
    ] {
        assert!(matches!(
            run_policy(policy.dotenv(&path, true).expect("source"), host.clone()).await,
            RunOutcome::Failed(_)
        ));
    }
}
