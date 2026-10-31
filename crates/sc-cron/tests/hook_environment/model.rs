use super::*;

#[test]
fn public_model_replacement_borrowed_accessors_and_exact_canonical_form() {
    let empty = HookEnvironment::new();
    assert!(empty.is_empty());
    assert_eq!(
        serde_json::to_string(&empty).expect("encode"),
        r#"{"dotenv":[],"variables":{}}"#
    );
    let policy = empty
        .set("Z", "deliberate-plaintext")
        .expect("set")
        .clear("Z")
        .expect("clear")
        .reference("Z", "opaque-locator")
        .expect("reference")
        .set("A", "")
        .expect("empty");
    assert_eq!(policy.len(), 2);
    assert_eq!(
        policy
            .variable_names()
            .map(EnvName::as_str)
            .collect::<Vec<_>>(),
        ["A", "Z"]
    );
    assert_eq!(
        policy.operation(&EnvName::new("Z").expect("name")),
        Some(EnvOperationRef::Reference("opaque-locator"))
    );
    let encoded = serde_json::to_string(&policy).expect("encode");
    assert_eq!(
        encoded,
        r#"{"dotenv":[],"variables":{"A":{"set":""},"Z":{"reference":"opaque-locator"}}}"#
    );
    assert_eq!(
        serde_json::from_str::<HookEnvironment>(&encoded).expect("decode"),
        policy
    );
    let debug = format!(
        "{policy:?} {:?}",
        EnvOperationRef::Set("deliberate-plaintext")
    );
    assert!(!debug.contains("opaque-locator") && !debug.contains("deliberate-plaintext"));
    let host = HookEnvironmentHost::new()
        .allow_process(["A"])
        .expect("allow")
        .allow_process(["A"])
        .expect("idempotent");
    assert_eq!(host.process_names().count(), 1);
    assert!(!host.has_reference_resolver());
    let resolver: Arc<dyn EnvironmentReferenceResolver> = Arc::new(RecordingResolver::default());
    assert!(
        host.with_reference_resolver(resolver)
            .has_reference_resolver()
    );
}

#[test]
fn malformed_duplicate_unknown_casefold_and_null_documents_are_rejected() {
    let bad = [
        r#"{}"#,
        r#"{"dotenv":[],"dotenv":[],"variables":{}}"#,
        r#"{"dotenv":[],"variables":{},"unknown":1}"#,
        r#"{"dotenv":[],"variables":{"A":{"set":"one"},"A":{"set":"two"}}}"#,
        r#"{"dotenv":[],"variables":{"A":{"set":"one"},"a":{"clear":true}}}"#,
        r#"{"dotenv":[],"variables":{"A":{"set":"one","set":"two"}}}"#,
        r#"{"dotenv":[],"variables":{"A":{"set":"one","clear":true}}}"#,
        r#"{"dotenv":[],"variables":{"A":{"set":null}}}"#,
        r#"{"dotenv":[],"variables":{"A":{"set":null,"clear":true}}}"#,
        r#"{"dotenv":[],"variables":{"A":{"clear":false}}}"#,
        r#"{"dotenv":[],"variables":{"A":{"reference":""}}}"#,
        r#"{"dotenv":[],"variables":{"A":{}}}"#,
        r#"{"dotenv":[{"path":"relative","required":true}],"variables":{}}"#,
    ];
    for document in bad {
        assert!(
            serde_json::from_str::<HookEnvironment>(document).is_err(),
            "{document}"
        );
    }
    for name in ["", "1A", "a-b", "ä", "X\0"] {
        assert!(EnvName::new(name).is_err());
    }
    assert!(HookEnvironment::new().set("A", "x\0y").is_err());
    assert!(HookEnvironment::new().reference("A", "x\0y").is_err());
    assert!(
        HookEnvironment::new()
            .set("Name", "x")
            .expect("set")
            .clear("NAME")
            .is_err()
    );
    assert!(
        HookEnvironmentHost::new()
            .allow_process(["Name", "NAME"])
            .is_err()
    );
}

#[test]
fn stored_limits_are_enforced_at_and_above_every_bound() {
    assert!(EnvName::new("N".repeat(MAX_ENV_NAME_BYTES)).is_ok());
    assert!(EnvName::new("N".repeat(MAX_ENV_NAME_BYTES + 1)).is_err());
    assert!(
        HookEnvironment::new()
            .set("A", "x".repeat(MAX_ENV_VALUE_BYTES))
            .is_ok()
    );
    assert!(
        HookEnvironment::new()
            .set("A", "x".repeat(MAX_ENV_VALUE_BYTES + 1))
            .is_err()
    );
    assert!(
        HookEnvironment::new()
            .reference("A", "x".repeat(MAX_ENV_REFERENCE_BYTES))
            .is_ok()
    );
    assert!(
        HookEnvironment::new()
            .reference("A", "x".repeat(MAX_ENV_REFERENCE_BYTES + 1))
            .is_err()
    );
    let mut policy = HookEnvironment::new();
    for n in 0..MAX_ENV_ENTRIES {
        policy = policy.clear(format!("N{n}")).expect("entry bound");
    }
    assert!(policy.clone().clear("EXTRA").is_err());
    assert!(policy.clear("N0").is_ok());
    let root = tempfile::tempdir().expect("root");
    let mut policy = HookEnvironment::new();
    for n in 0..MAX_ENV_DOTENV_SOURCES {
        policy = policy
            .dotenv(root.path().join(format!("{n}.env")), false)
            .expect("source bound");
    }
    assert!(policy.dotenv(root.path().join("extra"), false).is_err());
    let source = DotenvSource::new(root.path().join("file.env"), true).expect("source");
    assert!(source.required());
    assert_eq!(source.path(), root.path().join("file.env"));
    assert!(
        HookEnvironment::new()
            .dotenv(source.path(), true)
            .expect("source")
            .dotenv(root.path().join("./file.env"), false)
            .is_err()
    );
    assert!(DotenvSource::new(root.path().join("../escape"), true).is_err());
    let prefix = if cfg!(windows) { "C:\\" } else { "/" };
    let exact = format!("{prefix}{}", "x".repeat(MAX_ENV_PATH_BYTES - prefix.len()));
    assert!(DotenvSource::new(&exact, false).is_ok());
    assert!(DotenvSource::new(format!("{exact}x"), false).is_err());
    // JSON escaping is counted in canonical document bytes, independently of
    // the per-value UTF-8 ceiling.
    let mut map = serde_json::Map::new();
    for n in 0..16 {
        map.insert(format!("N{n}"), json!({"set":"x".repeat(16_000)}));
    }
    let document = json!({"dotenv":[],"variables":map});
    let mut policy: HookEnvironment =
        serde_json::from_value(document).expect("under document limit");
    let before = serde_json::to_string(&policy).expect("encode").len();
    let remaining = MAX_ENV_DOCUMENT_BYTES - before - ",\"Z\":{\"set\":\"\"}".len();
    policy = policy
        .set("Z", "x".repeat(remaining))
        .expect("exact document bound");
    assert_eq!(
        serde_json::to_string(&policy).expect("encode").len(),
        MAX_ENV_DOCUMENT_BYTES
    );
    assert!(policy.set("Z", "x".repeat(remaining + 1)).is_err());
}

#[test]
fn rust_pairing_is_checked_after_public_field_mutation_and_at_store_write() {
    let policy = HookEnvironment::new().clear("A").expect("clear");
    let mut spec = ScheduleSpec::new("rust", "* * * * *", BUDGET, json!({})).expect("spec");
    assert!(spec.clone().with_environment(policy.clone()).is_err());
    spec.environment = policy;
    let fixture = common::Fixture::new();
    assert!(fixture.scheduler.start(spec.clone()).is_err());
    let store = SqliteScheduleStore::open_in_memory(ScheduleLimits::default()).expect("store");
    let now: Timestamp = "2026-10-01T00:00:00Z".parse().expect("time");
    assert!(
        store
            .create(
                &NewSchedule {
                    spec,
                    jitter: Duration::ZERO,
                    next_run_at: now
                },
                now
            )
            .is_err()
    );
}

#[test]
fn first_release_schema_roundtrip_empty_default_corruption_and_non_destructive_rejection() {
    let root = tempfile::tempdir().expect("root");
    let path = root.path().join("cron.db");
    let store = SqliteScheduleStore::open(&path, ScheduleLimits::default()).expect("store");
    let now: Timestamp = "2026-10-01T00:00:00Z".parse().expect("time");
    let environment = HookEnvironment::new()
        .set("PERSISTED", "deliberate-plaintext")
        .expect("set");
    let row = store
        .create(
            &NewSchedule {
                spec: spec(
                    "stored",
                    HookTarget::python("module.py").expect("hook"),
                    environment.clone(),
                ),
                jitter: Duration::ZERO,
                next_run_at: now,
            },
            now,
        )
        .expect("create");
    assert_eq!(
        store
            .get(row.id)
            .expect("get")
            .expect("row")
            .spec
            .environment,
        environment
    );
    let connection = rusqlite::Connection::open(&path).expect("connection");
    assert_eq!(SCHEMA_VERSION, 1);
    let encoded: String = connection
        .query_row("SELECT hook_environment FROM sc_cron_schedule", [], |r| {
            r.get(0)
        })
        .expect("document");
    assert!(encoded.contains("deliberate-plaintext"));
    let default:String=connection.query_row("SELECT dflt_value FROM pragma_table_info('sc_cron_schedule') WHERE name='hook_environment'",[],|r|r.get(0)).expect("default");
    assert_eq!(default, "'{\"dotenv\":[],\"variables\":{}}'");
    connection.execute("UPDATE sc_cron_schedule SET hook_environment = '{\"dotenv\":[],\"variables\":{\"A\":{\"clear\":false}}}'",[]).expect("corrupt");
    assert!(store.list_active().expect("bounded skip").is_empty());
    drop(store);
    drop(connection);
    assert!(SqliteScheduleStore::open(&path, ScheduleLimits::default()).is_ok());
    let incompatible = root.path().join("incompatible.db");
    let connection = rusqlite::Connection::open(&incompatible).expect("connection");
    connection.execute_batch("CREATE TABLE sc_cron_schedule(id INTEGER PRIMARY KEY, value TEXT); INSERT INTO sc_cron_schedule VALUES(1,'preserve'); PRAGMA user_version=1;").expect("incompatible");
    let error =
        SqliteScheduleStore::open(&incompatible, ScheduleLimits::default()).expect_err("reject");
    assert!(error.recovery().contains("recreate"));
    let kept: String = connection
        .query_row("SELECT value FROM sc_cron_schedule", [], |r| r.get(0))
        .expect("untouched");
    assert_eq!(kept, "preserve");
}
