use super::*;

async fn run(
    source: &str,
    environment: HookEnvironment,
    host: HookEnvironmentHost,
    registry: MontyHostRegistry,
) -> RunOutcome {
    let mut fixture = common::Fixture::new();
    fixture.scheduler = fixture
        .scheduler
        .with_hook_environment(host)
        .with_monty_hosts(registry);
    fixture.start(spec(
        "monty-environment",
        HookTarget::monty(source).expect("source"),
        environment,
    ));
    due(&fixture).await
}

#[tokio::test]
async fn monty_initialization_and_hook_getenv_getenviron_absent_default_and_clear() {
    let source = "import os\ninitial=os.getenv('KEY')\ndef cron_stop(hook):\n    assert initial=='value'\n    assert os.getenv('ABSENT','fallback')=='fallback'\n    assert os.getenv('EMPTY')==''\n    assert os.getenv('CLEAR_ME') is None\n    assert os.environ=={'KEY':'value','EMPTY':''}\n    return {'action':'continue'}\ndef cron_tick(hook):\n    assert os.getenv('KEY')==initial\n";
    let environment = HookEnvironment::new()
        .set("KEY", "value")
        .expect("key")
        .set("EMPTY", "")
        .expect("empty")
        .clear("CLEAR_ME")
        .expect("clear");
    assert_eq!(
        run(
            source,
            environment,
            HookEnvironmentHost::new(),
            MontyHostRegistry::new()
        )
        .await,
        RunOutcome::Ok
    );
    for source in [
        "import os\na=os.path.exists('/blocked')\ndef cron_stop(h): return {'action':'continue'}\ndef cron_tick(h): return None",
        "import os\ndef cron_stop(h): return {'action':'continue'}\ndef cron_tick(h): os.path.exists('/blocked')",
    ] {
        assert_code(
            &run(
                source,
                HookEnvironment::new(),
                HookEnvironmentHost::new(),
                MontyHostRegistry::new(),
            )
            .await,
            "HOST_OS",
        );
    }
    let called = Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let count = called.clone();
    let registry = MontyHostRegistry::new()
        .register("application_call", move |_: &MontyHostCall| {
            count.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            Ok(json!(true))
        })
        .expect("registry");
    let source = "a=application_call()\ndef cron_stop(h): return {'action':'continue'}\ndef cron_tick(h): return None";
    assert_code(
        &run(
            source,
            HookEnvironment::new(),
            HookEnvironmentHost::new(),
            registry,
        )
        .await,
        "HOST_DENIED",
    );
    assert_eq!(called.load(std::sync::atomic::Ordering::SeqCst), 0);
}

struct ChangingResolver(std::sync::atomic::AtomicUsize);
#[async_trait::async_trait]
impl EnvironmentReferenceResolver for ChangingResolver {
    async fn resolve(
        &self,
        _: EnvironmentReferenceRequest<'_>,
    ) -> Result<Option<String>, EnvironmentReferenceError> {
        Ok(Some(format!(
            "value{}",
            self.0.fetch_add(1, std::sync::atomic::Ordering::SeqCst) + 1
        )))
    }
}
#[tokio::test]
async fn retained_module_globals_initialize_once_but_function_reads_current_occurrence() {
    let resolver = Arc::new(ChangingResolver(std::sync::atomic::AtomicUsize::new(0)));
    let observations = Arc::new(std::sync::Mutex::new(Vec::new()));
    let record = observations.clone();
    let registry = MontyHostRegistry::new()
        .register("record", move |call: &MontyHostCall| {
            record.lock().expect("observations").push(call.args.clone());
            Ok(json!(null))
        })
        .expect("record");
    let mut fixture = common::Fixture::new();
    fixture.scheduler = fixture
        .scheduler
        .with_hook_environment(HookEnvironmentHost::new().with_reference_resolver(resolver.clone()))
        .with_monty_hosts(registry);
    fixture.start(spec("retained",HookTarget::monty("import os\ninitial=os.getenv('KEY')\ndef cron_stop(hook):\n    record(initial,os.getenv('KEY'))\n    return {'action':'continue'}\ndef cron_tick(hook):\n    record(initial,os.getenv('KEY'))\n").expect("source"),HookEnvironment::new().reference("KEY","changing").expect("reference")));
    assert_eq!(due(&fixture).await, RunOutcome::Ok);
    assert_eq!(due(&fixture).await, RunOutcome::Ok);
    assert_eq!(
        resolver.0.load(std::sync::atomic::Ordering::SeqCst),
        2,
        "one snapshot per occurrence"
    );
    assert_eq!(
        *observations.lock().expect("observations"),
        vec![
            vec![json!("value1"), json!("value1")],
            vec![json!("value1"), json!("value1")],
            vec![json!("value1"), json!("value2")],
            vec![json!("value1"), json!("value2")]
        ]
    );
}

#[tokio::test]
async fn worker_shares_the_snapshot_between_stop_and_tick() {
    let resolver = Arc::new(ChangingResolver(std::sync::atomic::AtomicUsize::new(0)));
    let mut fixture = common::Fixture::new();
    fixture.scheduler = fixture.scheduler.with_hook_environment(
        HookEnvironmentHost::new().with_reference_resolver(resolver.clone()),
    );
    let row=fixture.start(spec("worker",HookTarget::monty("import os\ninitial=os.getenv('KEY')\ndef cron_stop(h):\n    assert os.getenv('KEY')==initial\n    return {'action':'continue'}\ndef cron_tick(h):\n    assert os.getenv('KEY')==initial\n").expect("source"),HookEnvironment::new().reference("KEY","changing").expect("reference")));
    let worker = fixture.worker(common::idle_job()).await;
    fixture.step(60).await;
    fixture.store.wait_recorded(&row, RunOutcome::Ok).await;
    assert_eq!(resolver.0.load(std::sync::atomic::Ordering::SeqCst), 1);
    common::shutdown(worker).await;
}

#[test]
fn python_monty_and_both_cli_children_use_one_allowlisted_map() {
    let output = std::process::Command::new(std::env::current_exe().expect("self"))
        .args(["--exact", "monty::cli_environment_child", "--nocapture"])
        .env("SC_CRON_ENV_CHILD", "monty")
        .env("SC_ENV_INHERITED", "allowed-value")
        .env("SC_ENV_UNLISTED", "unlisted-secret")
        .env("ATM_IDENTITY", "daemon-identity")
        .env("ATM_TEAM", "daemon-team")
        .output()
        .expect("child");
    assert!(
        output.status.success(),
        "{} {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}
#[test]
fn cli_environment_child() {
    if std::env::var("SC_CRON_ENV_CHILD").as_deref() != Ok("monty") {
        return;
    }
    tokio::runtime::Runtime::new().expect("runtime").block_on(async{
        let root=tempfile::tempdir().expect("root");let selected=root.path().join("selected");std::fs::create_dir(&selected).expect("cwd");
        let registry=MontyHostRegistry::new().with_sc_compose(ScComposeHostConfig::new(python_program()).expect("config").with_working_directory(&selected).expect("cwd")).expect("compose").with_atm(AtmHostConfig::new(python_program()).expect("config").with_working_directory(&selected).expect("cwd")).expect("atm");
        let script="import os,sys,json; print(json.dumps({'env':dict(os.environ),'args':sys.argv[1:],'cwd':os.getcwd()}))";
        let script=serde_json::to_string(script).expect("script string");let canonical = selected.canonicalize().expect("canonical cwd");
        let text = canonical.to_string_lossy();
        let text = text.strip_prefix(r"\\?\").unwrap_or(&text);
        let cwd = serde_json::to_string(text).expect("cwd string");
        let source=format!("import os\nimport json\ninitial=os.getenv('ATM_IDENTITY')\ndef cron_stop(hook):\n    assert initial=='planner-7'\n    assert os.getenv('SC_ENV_UNLISTED') is None\n    return {{'action':'continue'}}\ndef cron_tick(hook):\n    a=sc_compose_run('-c',[{script},'arbitrary command','--flag=with space','tail'])\n    b=atm_run('-c',[{script},'arbitrary command','--flag=with space','tail'])\n    for item in [a,b]:\n        assert item['exit_code']==0\n        data=json.loads(item['stdout'])\n        env=data['env']\n        assert env['ATM_IDENTITY']=='planner-7'\n        assert env['ATM_TEAM']=='runtime'\n        assert env['SC_ENV_INHERITED']=='allowed-value'\n        assert env['REFERENCE']=='reference-value'\n        assert 'SC_ENV_UNLISTED' not in env\n        assert 'SC_CRON_ENV_CHILD' not in env\n        assert 'CLEAR_ME' not in env\n        assert data['args']==['arbitrary command','--flag=with space','tail']\n        assert data['cwd']=={cwd}\n        assert os.getenv('SC_ENV_INHERITED')==env['SC_ENV_INHERITED']\n");
        let host=HookEnvironmentHost::new().allow_process(["SC_ENV_INHERITED","ATM_IDENTITY","ATM_TEAM"]).expect("allow").with_reference_resolver(Arc::new(RecordingResolver::default()));
        let policy=HookEnvironment::new().set("ATM_IDENTITY","planner-7").expect("identity").set("ATM_TEAM","runtime").expect("team").clear("CLEAR_ME").expect("clear").reference("REFERENCE","value:reference-value").expect("reference");
        assert_eq!(run(&source,policy,host,registry).await,RunOutcome::Ok);
    });
}

#[tokio::test]
async fn initialization_runtime_return_and_both_cli_streams_redact_without_touching_codes() {
    for value in ["ENV", "HOST", ":", "1", "canary-value"] {
        let policy = HookEnvironment::new().set("KEY", value).expect("set");
        let sources = [
            "import os\nraise RuntimeError(os.getenv('KEY'))\ndef cron_stop(h): return {'action':'continue'}\ndef cron_tick(h): return None",
            "import os\ndef cron_stop(h): return {'action':'continue'}\ndef cron_tick(h): raise RuntimeError(os.getenv('KEY'))",
            "import os\ndef cron_stop(h): return {'action':'disable','reason':os.getenv('KEY')}\ndef cron_tick(h): return None",
        ];
        for (index, source) in sources.iter().enumerate() {
            let outcome = run(
                source,
                policy.clone(),
                HookEnvironmentHost::new(),
                MontyHostRegistry::new(),
            )
            .await;
            assert!(
                outcome.to_string().contains("[REDACTED_ENV]"),
                "{outcome:?}"
            );
            match index {
                0 => assert_code(&outcome, "HOOK_FAILED"),
                1 => assert_code(&outcome, "HOST_RAISED"),
                _ => assert_eq!(outcome, RunOutcome::Refused("[REDACTED_ENV]".into())),
            }
        }
        let message = value.to_owned();
        let custom = MontyHostRegistry::new()
            .register("fail", move |_: &MontyHostCall| {
                Err(CronError::InvalidSpec(message.clone()))
            })
            .expect("custom");
        let outcome = run(
            "def cron_stop(h): return {'action':'continue'}\ndef cron_tick(h): fail()",
            policy.clone(),
            HookEnvironmentHost::new(),
            custom,
        )
        .await;
        assert_code(&outcome, "HOST_CALL_FAILED");
        assert!(outcome.to_string().contains("[REDACTED_ENV]"));
        let outcome = run("import os\ndef cron_stop(h): return {'action':'bad-'+os.getenv('KEY')}\ndef cron_tick(h): return None", policy.clone(), HookEnvironmentHost::new(), MontyHostRegistry::new()).await;
        assert_code(&outcome, "HOST_REPLY");
        assert!(outcome.to_string().contains("[REDACTED_ENV]"));
        let registry = MontyHostRegistry::new()
            .with_sc_compose(ScComposeHostConfig::new(python_program()).expect("compose"))
            .expect("registry")
            .with_atm(AtmHostConfig::new(python_program()).expect("atm"))
            .expect("registry");
        let script = serde_json::to_string(
            "import os,sys; print(os.getenv('KEY')); print(os.getenv('KEY'),file=sys.stderr)",
        )
        .expect("script");
        for adapter in ["sc_compose_run", "atm_run"] {
            for stream in ["stdout", "stderr"] {
                let source = format!(
                    "def cron_stop(h): return {{'action':'continue'}}\ndef cron_tick(h):\n    reply={adapter}('-c',[{script}])\n    raise RuntimeError(reply['{stream}'])\n"
                );
                let outcome = run(
                    &source,
                    policy.clone(),
                    HookEnvironmentHost::new(),
                    registry.clone(),
                )
                .await;
                assert_code(&outcome, "HOST_RAISED");
                assert!(
                    outcome.to_string().contains("[REDACTED_ENV]"),
                    "{outcome:?}"
                );
            }
        }
    }
}

#[cfg(windows)]
#[test]
fn windows_path_spelling_is_uppercase_for_python_and_monty_and_lookup_ignores_case() {
    let output = std::process::Command::new(std::env::current_exe().expect("self"))
        .args(["--exact", "monty::windows_path_child", "--nocapture"])
        .env("SC_CRON_ENV_CHILD", "windows-path")
        .output()
        .expect("child");
    assert!(
        output.status.success(),
        "{} {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}
#[cfg(windows)]
#[test]
fn windows_path_child() {
    if std::env::var("SC_CRON_ENV_CHILD").as_deref() != Ok("windows-path") {
        return;
    }
    tokio::runtime::Runtime::new().expect("runtime").block_on(async{
        let host=HookEnvironmentHost::new().allow_process(["Path"]).expect("allow Path");
        let source="import os\ndef cron_stop(h):\n    assert 'PATH' in os.environ\n    assert 'Path' not in os.environ\n    assert os.getenv('path')==os.environ['PATH']\n    return {'action':'continue'}\ndef cron_tick(h): return None";
        assert_eq!(run(source,HookEnvironment::new(),host.clone(),MontyHostRegistry::new()).await,RunOutcome::Ok);
        let root=tempfile::tempdir().expect("root");let module=python_file(root.path(),"import os,json,sys\nhook=json.load(sys.stdin)\nassert 'PATH' in os.environ\nassert 'Path' not in list(os.environ)\nif hook['event']=='cron.stop': print('{\"action\":\"continue\"}')");
        let mut fixture=common::Fixture::new();fixture.scheduler=fixture.scheduler.with_hook_environment(host);fixture.start(spec("path",HookTarget::python(module).expect("hook"),HookEnvironment::new()));
        assert_eq!(due(&fixture).await,RunOutcome::Ok);
    });
}
