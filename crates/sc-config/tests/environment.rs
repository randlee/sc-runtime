#[path = "support/mod.rs"]
mod support;

use serde::Deserialize;
use serde_json::json;
use support::{load_value, write_file, TempDir};

const MERGED: &str = r#"{
  "daemon": { "host": "127.0.0.1", "port": 9000 },
  "logging": { "level": "info", "targets": ["stdout", "file"] }
}"#;

#[test]
fn applies_the_worked_example_environment() {
    let dir = TempDir::new();
    write_file(dir.path(), "default.json", MERGED);
    let value = load_value(
        dir.path(),
        "my-app",
        &[
            ("MY_APP__DAEMON__PORT", "8080"),
            ("MY_APP__DAEMON__HOST", "0.0.0.0"),
            ("MY_APP__DAEMON__TLS", "true"),
            ("MY_APP__DAEMON__MAX_CONNECTIONS", "5"),
            ("MY_APP__LOGGING__LEVEL", "debug"),
            ("MY_APP__LOGGING__TARGETS", r#"["stderr"]"#),
            ("MY_APP__STORES__SQLITE__PATH", "data/other.db"),
            ("MY_APP__LABEL", r#""8080""#),
            ("OTHER_APP__DAEMON__PORT", "1"),
            ("MY_APP_DAEMON_PORT", "2"),
            ("PATH", "/usr/bin"),
        ],
    )
    .unwrap();
    assert_eq!(
        value,
        json!({
            "daemon": {
                "host": "0.0.0.0",
                "port": 8080,
                "tls": true,
                "max_connections": 5
            },
            "logging": { "level": "debug", "targets": ["stderr"] },
            "stores": { "sqlite": { "path": "data/other.db" } },
            "label": "8080"
        })
    );
}

#[test]
fn json_values_keep_their_types() {
    let cases = [
        ("8080", json!(8080)),
        ("true", json!(true)),
        ("null", json!(null)),
        (r#"["a"]"#, json!(["a"])),
        (r#"{"x":1}"#, json!({"x": 1})),
    ];
    for (raw, expected) in cases {
        let dir = TempDir::new();
        write_file(dir.path(), "default.json", "{}");
        let value = load_value(dir.path(), "my-app", &[("MY_APP__A", raw)]).unwrap();
        assert_eq!(value, json!({"a": expected}));
    }
}

#[test]
fn plain_values_fall_back_to_strings() {
    let cases = [
        ("debug", json!("debug")),
        ("0.0.0.0", json!("0.0.0.0")),
        ("", json!("")),
        (r#""8080""#, json!("8080")),
    ];
    for (raw, expected) in cases {
        let dir = TempDir::new();
        write_file(dir.path(), "default.json", "{}");
        let value = load_value(dir.path(), "my-app", &[("MY_APP__A", raw)]).unwrap();
        assert_eq!(value, json!({"a": expected}));
    }
}

#[test]
fn non_prefix_names_are_ignored() {
    let dir = TempDir::new();
    write_file(dir.path(), "default.json", r#"{"daemon":{"port":1}}"#);
    let value = load_value(
        dir.path(),
        "my-app",
        &[
            ("OTHER_APP__DAEMON__PORT", "2"),
            ("MY_APP_DAEMON_PORT", "3"),
            ("my_app__daemon__port", "4"),
        ],
    )
    .unwrap();
    assert_eq!(value, json!({"daemon": {"port": 1}}));
}

#[test]
fn missing_objects_are_created() {
    let dir = TempDir::new();
    write_file(dir.path(), "default.json", "{}");
    let value = load_value(dir.path(), "my-app", &[("MY_APP__A__B__C", "1")]).unwrap();
    assert_eq!(value, json!({"a": {"b": {"c": 1}}}));
}

#[test]
fn descent_through_a_number_names_the_variable() {
    let dir = TempDir::new();
    write_file(dir.path(), "default.json", r#"{"daemon":{"port":7400}}"#);
    let error = load_value(dir.path(), "my-app", &[("MY_APP__DAEMON__PORT__X", "1")]).unwrap_err();
    match error {
        sc_config::ConfigError::Override { variable } => {
            assert_eq!(variable, "MY_APP__DAEMON__PORT__X");
        }
        other => panic!("unexpected {other}"),
    }
}

#[test]
fn object_override_replaces_instead_of_merging() {
    let dir = TempDir::new();
    write_file(
        dir.path(),
        "default.json",
        r#"{"daemon":{"host":"h","port":1}}"#,
    );
    let value = load_value(dir.path(), "my-app", &[("MY_APP__DAEMON", r#"{"port":2}"#)]).unwrap();
    assert_eq!(value, json!({"daemon": {"port": 2}}));
}

#[test]
fn numeric_override_deserializes_into_u16() {
    #[derive(Debug, Deserialize, PartialEq)]
    struct Root {
        daemon: Daemon,
    }
    #[derive(Debug, Deserialize, PartialEq)]
    struct Daemon {
        port: u16,
    }

    let dir = TempDir::new();
    write_file(dir.path(), "default.json", r#"{"daemon":{"port":1}}"#);
    let loaded: Root = sc_config::Loader::new(
        sc_config::AppName::new("my-app").unwrap(),
        dir.path().to_path_buf(),
        support::environment(&[("MY_APP__DAEMON__PORT", "8080")]),
    )
    .load()
    .unwrap();
    assert_eq!(loaded.daemon.port, 8080);
}

#[test]
fn longer_path_wins_regardless_of_input_order() {
    let orders = [
        [
            ("MY_APP__DAEMON", r#"{"port":1,"host":"h"}"#),
            ("MY_APP__DAEMON__PORT", "2"),
        ],
        [
            ("MY_APP__DAEMON__PORT", "2"),
            ("MY_APP__DAEMON", r#"{"port":1,"host":"h"}"#),
        ],
    ];
    for order in orders {
        let dir = TempDir::new();
        write_file(dir.path(), "default.json", "{}");
        let value = load_value(dir.path(), "my-app", &order).unwrap();
        assert_eq!(value, json!({"daemon": {"port": 2, "host": "h"}}));
    }
}

#[test]
fn empty_segments_cannot_be_applied() {
    for name in ["MY_APP__", "MY_APP__A__", "MY_APP__A____B"] {
        let dir = TempDir::new();
        write_file(dir.path(), "default.json", "{}");
        let error = load_value(dir.path(), "my-app", &[(name, "1")]).unwrap_err();
        match error {
            sc_config::ConfigError::Override { variable } => assert_eq!(variable, name),
            other => panic!("unexpected {other}"),
        }
    }
}

#[test]
fn odd_override_text_does_not_panic() {
    let huge = "[".repeat(1_000_000);
    let awkward = ["{", "]", huge.as_str(), ""];
    for raw in awkward {
        let dir = TempDir::new();
        write_file(dir.path(), "default.json", "{}");
        load_value(dir.path(), "my-app", &[("MY_APP__A", raw)]).unwrap();
    }
}

#[cfg(unix)]
#[test]
fn non_unicode_prefix_match_is_an_error_and_other_names_are_ignored() {
    use std::ffi::OsString;
    use std::os::unix::ffi::OsStringExt;

    let dir = TempDir::new();
    write_file(dir.path(), "default.json", r#"{"a":0}"#);
    let ignored = OsString::from_vec(vec![0xff, 0xfe]);
    let loader = sc_config::Loader::new(
        sc_config::AppName::new("my-app").unwrap(),
        dir.path().to_path_buf(),
        vec![(ignored, OsString::from("1"))],
    );
    let value: serde_json::Value = loader.load().unwrap();
    assert_eq!(value, json!({"a": 0}));

    let mut prefixed = b"MY_APP__".to_vec();
    prefixed.push(0xff);
    let error = sc_config::Loader::new(
        sc_config::AppName::new("my-app").unwrap(),
        dir.path().to_path_buf(),
        vec![(OsString::from_vec(prefixed), OsString::from("1"))],
    )
    .load::<serde_json::Value>()
    .unwrap_err();
    assert_eq!(error.code(), "CFG.NON_UNICODE");

    let value_error = sc_config::Loader::new(
        sc_config::AppName::new("my-app").unwrap(),
        dir.path().to_path_buf(),
        vec![(
            OsString::from("MY_APP__A"),
            OsString::from_vec(vec![0xff, 0xfe]),
        )],
    )
    .load::<serde_json::Value>()
    .unwrap_err();
    assert_eq!(value_error.code(), "CFG.NON_UNICODE");
    assert!(value_error.to_string().contains("MY_APP__A"));
}
