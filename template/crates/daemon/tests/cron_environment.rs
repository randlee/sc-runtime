//! Generated application policy, resolver and host registry composition proof.

#![cfg(feature = "monty")]

use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

use daemon::cron::{self, CronConfig};
use sc_cron::{
    Clock, EnvironmentReferenceError, EnvironmentReferenceRequest, EnvironmentReferenceResolver,
    HookEnvironment, HookTarget, MontyHostCall, MontyHostRegistry, NewSchedule, RunOutcome,
    ScheduleSpec, ScheduleStore, SystemClock, Timestamp,
};
use serde_json::{Value, json};

struct RecordingResolver(AtomicUsize);

#[async_trait::async_trait]
impl EnvironmentReferenceResolver for RecordingResolver {
    async fn resolve(
        &self,
        request: EnvironmentReferenceRequest<'_>,
    ) -> Result<Option<String>, EnvironmentReferenceError> {
        assert_eq!(request.variable().as_str(), "REFERENCE");
        assert_eq!(request.reference(), "recording://application/value");
        self.0.fetch_add(1, Ordering::SeqCst);
        Ok(Some("resolved-canary-value".to_owned()))
    }
}

const PYTHON: &str = r#"import os,sys,json
hook=json.load(sys.stdin)
keys=['IDENTITY','TEAM','FILE_ONLY','REFERENCE','CLEAR_ME','UNLISTED']
values={key:os.getenv(key) for key in keys}
if hook['event']=='cron.stop':
    print(json.dumps({'action':'continue'}))
else:
    with open(hook['payload']['output'],'w') as output:
        json.dump(values,output)
"#;

const MONTY: &str = r#"import os
initial=os.getenv('REFERENCE')
def cron_stop(hook):
    assert initial=='resolved-canary-value'
    assert os.getenv('CLEAR_ME') is None
    assert os.getenv('UNLISTED') is None
    return {'action':'continue'}
def cron_tick(hook):
    application_snapshot_v7({'IDENTITY':os.getenv('IDENTITY'),'TEAM':os.getenv('TEAM'),'FILE_ONLY':os.getenv('FILE_ONLY'),'REFERENCE':os.getenv('REFERENCE'),'CLEAR_ME':os.getenv('CLEAR_ME'),'UNLISTED':os.getenv('UNLISTED')})
"#;

fn policy(path: &std::path::Path) -> HookEnvironment {
    HookEnvironment::new()
        .dotenv(path, true)
        .expect("dotenv")
        .set("IDENTITY", "schedule-identity")
        .expect("identity")
        .set("TEAM", "deliberate-plaintext-value")
        .expect("team")
        .clear("CLEAR_ME")
        .expect("clear")
        .reference("REFERENCE", "recording://application/value")
        .expect("reference")
}

fn due(
    scheduler: &sc_cron::Scheduler<
        sc_cron::SqliteScheduleStore,
        SystemClock,
        sc_cron::SystemJitter,
    >,
    name: &str,
    target: HookTarget,
    policy: HookEnvironment,
    payload: Value,
) {
    let now = Timestamp::from_datetime(SystemClock.now());
    let spec = ScheduleSpec::with_hook(name, "* * * * *", Duration::from_secs(10), payload, target)
        .expect("spec")
        .with_environment(policy)
        .expect("environment");
    scheduler
        .store()
        .create(
            &NewSchedule {
                spec,
                jitter: Duration::ZERO,
                next_run_at: now,
            },
            now,
        )
        .expect("due row");
}

#[tokio::test]
async fn generated_policy_resolver_and_custom_registry_preserve_python_monty_snapshots() {
    for exact_file in [true, false] {
        let root = tempfile::tempdir().expect("root");
        let dotenv = root.path().join("application.env");
        let module = root.path().join("snapshot.py");
        let output = root.path().join("python-snapshot.json");
        std::fs::write(
            &dotenv,
            "IDENTITY=file-identity\nFILE_ONLY=file-canary-value\nCLEAR_ME=file-clear\n",
        )
        .expect("dotenv");
        std::fs::write(&module, PYTHON).expect("python");
        let environment = if exact_file {
            json!({"allow_dotenv_files": [dotenv]})
        } else {
            json!({"allow_dotenv_roots": [root.path()]})
        };
        let config: CronConfig =
            serde_json::from_value(json!({"max_schedule_rows": null, "environment": environment}))
                .expect("config");
        let empty = CronConfig::default()
            .hook_environment_host()
            .expect("defaults");
        assert_eq!(empty.process_names().count(), 0);
        assert_eq!(empty.dotenv_files().count(), 0);
        assert_eq!(empty.dotenv_roots().count(), 0);
        let resolver = Arc::new(RecordingResolver(AtomicUsize::new(0)));
        let host = config
            .hook_environment_host()
            .expect("typed policy")
            .with_reference_resolver(resolver.clone());
        let observations = Arc::new(std::sync::Mutex::new(Vec::new()));
        let recorded = Arc::clone(&observations);
        let registry = MontyHostRegistry::new()
            .register("application_snapshot_v7", move |call: &MontyHostCall| {
                recorded
                    .lock()
                    .expect("observations")
                    .push(call.args.clone());
                Ok(Value::Null)
            })
            .expect("custom application registry");
        let scheduler = cron::open(
            &config,
            root.path(),
            Arc::new(sc_observability_types::NoopCanonicalLog::new(
                sc_observability_types::ServiceName::new("generated-environment").expect("service"),
            )),
            host,
            registry,
        )
        .expect("generated open");
        due(
            &scheduler,
            "python",
            HookTarget::python(&module).expect("python"),
            policy(&dotenv),
            json!({"output": output}),
        );
        due(
            &scheduler,
            "monty",
            HookTarget::monty(MONTY).expect("monty"),
            policy(&dotenv),
            json!({}),
        );
        let runs = scheduler
            .run_due(Arc::new(cron::ApplicationJob))
            .await
            .expect("due runs");
        assert_eq!(runs.len(), 2);
        assert!(
            runs.iter().all(|run| run.outcome == RunOutcome::Ok),
            "{runs:?}"
        );
        assert_eq!(
            resolver.0.load(Ordering::SeqCst),
            2,
            "one resolution per occurrence, shared between stop and tick"
        );
        let expected = json!({"IDENTITY":"schedule-identity","TEAM":"deliberate-plaintext-value","FILE_ONLY":"file-canary-value","REFERENCE":"resolved-canary-value","CLEAR_ME":null,"UNLISTED":null});
        let python: Value =
            serde_json::from_str(&std::fs::read_to_string(output).expect("Python output"))
                .expect("snapshot");
        assert_eq!(python, expected);
        assert_eq!(
            *observations.lock().expect("recorded"),
            vec![vec![expected]]
        );
        let persisted = serde_json::to_string(
            &scheduler
                .store()
                .list_active()
                .expect("rows")
                .iter()
                .map(|row| &row.spec.environment)
                .collect::<Vec<_>>(),
        )
        .expect("stored policy");
        assert!(persisted.contains("deliberate-plaintext-value"));
        assert!(persisted.contains("recording://application/value"));
        assert!(!persisted.contains("file-canary-value"));
        assert!(!persisted.contains("resolved-canary-value"));
        let denied_root = tempfile::tempdir().expect("unauthorized root");
        let denied = denied_root.path().join("denied.env");
        std::fs::write(&denied, "FILE_ONLY=denied-canary").expect("denied file");
        due(
            &scheduler,
            "denied",
            HookTarget::monty(MONTY).expect("monty"),
            policy(&denied),
            json!({}),
        );
        let runs = scheduler
            .run_due(Arc::new(cron::ApplicationJob))
            .await
            .expect("denied run");
        assert_eq!(runs.len(), 1);
        assert!(runs[0].outcome.to_string().contains("ENV_FILE_DENIED"));
        let redacted = "import os\ndef cron_stop(h): return {'action':'continue'}\ndef cron_tick(h): raise RuntimeError(os.getenv('REFERENCE'))";
        due(
            &scheduler,
            "redaction",
            HookTarget::monty(redacted).expect("monty"),
            policy(&dotenv),
            json!({}),
        );
        let runs = scheduler
            .run_due(Arc::new(cron::ApplicationJob))
            .await
            .expect("redaction run");
        assert_eq!(runs.len(), 1);
        let detail = runs[0].outcome.to_string();
        assert!(detail.contains("[REDACTED_ENV]"), "{detail}");
        assert!(!detail.contains("resolved-canary-value"), "{detail}");
    }
}
