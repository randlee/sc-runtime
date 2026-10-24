#[path = "support/mod.rs"]
mod support;

use serde_json::json;
use support::{load_value, snapshot, write_file, TempDir};

const DEFAULT_WORKED: &str = r#"{
  "daemon": { "host": "127.0.0.1", "port": 7400 },
  "logging": { "level": "info", "targets": ["stdout", "file"] },
  "stores": { "sqlite": { "path": "data/app.db" } }
}"#;

const LOCAL_WORKED: &str = r#"{
  "daemon": { "port": 9000 },
  "logging": { "level": null, "targets": ["stdout"] },
  "stores": "disabled"
}"#;

#[test]
fn merges_worked_example() {
    let dir = TempDir::new();
    write_file(dir.path(), "default.json", DEFAULT_WORKED);
    write_file(dir.path(), "local.json", LOCAL_WORKED);
    let value = load_value(dir.path(), "my-app", &[]).unwrap();
    assert_eq!(
        value,
        json!({
            "daemon": { "host": "127.0.0.1", "port": 9000 },
            "logging": { "level": null, "targets": ["stdout"] },
            "stores": "disabled"
        })
    );
}

#[test]
fn missing_local_keeps_default() {
    let dir = TempDir::new();
    write_file(dir.path(), "default.json", r#"{"a":1,"b":[2]}"#);
    let value = load_value(dir.path(), "my-app", &[]).unwrap();
    assert_eq!(value, json!({"a": 1, "b": [2]}));
}

#[test]
fn array_overlay_replaces_the_array() {
    let dir = TempDir::new();
    write_file(dir.path(), "default.json", r#"{"a":[1,2,3]}"#);
    write_file(dir.path(), "local.json", r#"{"a":[9]}"#);
    let value = load_value(dir.path(), "my-app", &[]).unwrap();
    assert_eq!(value, json!({"a": [9]}));
}

#[test]
fn null_overlay_keeps_sibling_keys() {
    let dir = TempDir::new();
    write_file(dir.path(), "default.json", r#"{"a":{"b":1,"c":2}}"#);
    write_file(dir.path(), "local.json", r#"{"a":{"b":null}}"#);
    let value = load_value(dir.path(), "my-app", &[]).unwrap();
    assert_eq!(value, json!({"a": {"b": null, "c": 2}}));
}

#[test]
fn object_overlay_replaces_a_scalar() {
    let dir = TempDir::new();
    write_file(dir.path(), "default.json", r#"{"a":1}"#);
    write_file(dir.path(), "local.json", r#"{"a":{"b":2}}"#);
    let value = load_value(dir.path(), "my-app", &[]).unwrap();
    assert_eq!(value, json!({"a": {"b": 2}}));
}

#[test]
fn environment_outranks_both_files() {
    let dir = TempDir::new();
    write_file(dir.path(), "default.json", r#"{"a":1}"#);
    write_file(dir.path(), "local.json", r#"{"a":2}"#);
    let value = load_value(dir.path(), "my-app", &[("MY_APP__A", "3")]).unwrap();
    assert_eq!(value, json!({"a": 3}));
}

#[test]
fn missing_default_is_an_error() {
    let dir = TempDir::new();
    let error = load_value(dir.path(), "my-app", &[]).unwrap_err();
    assert!(matches!(
        error,
        sc_config::ConfigError::MissingDefault { .. }
    ));
}

#[test]
fn load_does_not_change_the_directory() {
    let dir = TempDir::new();
    write_file(dir.path(), "default.json", r#"{"a":1}"#);
    write_file(dir.path(), "local.json", r#"{"b":2}"#);
    let before = snapshot(dir.path());
    load_value(dir.path(), "my-app", &[]).unwrap();
    assert_eq!(snapshot(dir.path()), before);
}

#[test]
fn app_name_does_not_select_files() {
    let dir = TempDir::new();
    write_file(dir.path(), "default.json", DEFAULT_WORKED);
    write_file(dir.path(), "local.json", LOCAL_WORKED);
    let mine = load_value(dir.path(), "my-app", &[]).unwrap();
    let other = load_value(dir.path(), "other-app", &[]).unwrap();
    assert_eq!(mine, other);
}

#[test]
fn two_directories_stay_independent() {
    let first = TempDir::new();
    let second = TempDir::new();
    write_file(first.path(), "default.json", r#"{"a":1}"#);
    write_file(second.path(), "default.json", r#"{"a":2}"#);
    assert_eq!(
        load_value(first.path(), "my-app", &[]).unwrap(),
        json!({"a": 1})
    );
    assert_eq!(
        load_value(second.path(), "my-app", &[]).unwrap(),
        json!({"a": 2})
    );
}

#[test]
fn empty_environment_keeps_the_file_value() {
    let dir = TempDir::new();
    write_file(dir.path(), "default.json", r#"{"a":0}"#);
    assert_eq!(
        load_value(dir.path(), "my-app", &[]).unwrap(),
        json!({"a": 0})
    );
}

#[test]
fn in_memory_environment_overrides_without_process_state() {
    let dir = TempDir::new();
    write_file(dir.path(), "default.json", r#"{"a":0}"#);
    assert_eq!(
        load_value(dir.path(), "my-app", &[("MY_APP__A", "1")]).unwrap(),
        json!({"a": 1})
    );
}

#[test]
fn second_load_sees_a_rewritten_file() {
    let dir = TempDir::new();
    write_file(dir.path(), "default.json", r#"{"a":0}"#);
    let loader = sc_config::Loader::new(
        sc_config::AppName::new("my-app").unwrap(),
        dir.path().to_path_buf(),
        Vec::new(),
    );
    let first: serde_json::Value = loader.load().unwrap();
    write_file(dir.path(), "default.json", r#"{"a":1}"#);
    let second: serde_json::Value = loader.load().unwrap();
    assert_eq!(first, json!({"a": 0}));
    assert_eq!(second, json!({"a": 1}));
}
