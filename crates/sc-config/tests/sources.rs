//! Built-in sources: REQ-CFG-0006.

#[path = "support/mod.rs"]
mod support;

use sc_config::{ConfigError, EnvSource, FileSource, Layers, Source, ValueSource};
use serde_json::{Value, json};
use support::{TempDir, environment, write_file};

fn load(layers: Layers) -> Result<Value, ConfigError> {
    layers.load()
}

#[test]
fn a_required_file_that_is_absent_is_missing_default() {
    let dir = TempDir::new();
    let path = dir.path().join("a.json");
    match load(Layers::new().push(FileSource::new(&path))).unwrap_err() {
        ConfigError::MissingDefault { path: reported } => assert_eq!(reported, path),
        other => panic!("unexpected {other}"),
    }
}

#[test]
fn an_optional_file_that_is_absent_contributes_nothing() {
    let dir = TempDir::new();
    let layers = Layers::new().push(FileSource::new(dir.path().join("a.json")).optional());
    assert_eq!(load(layers).unwrap(), json!({}));
}

#[test]
fn a_directory_at_the_path_is_a_read_error() {
    let dir = TempDir::new();
    let error = load(Layers::new().push(FileSource::new(dir.path()).optional())).unwrap_err();
    assert_eq!(error.code(), "CFG.READ");
}

#[test]
fn accepts_rejects_keys_outside_the_accepted_paths() {
    let dir = TempDir::new();
    write_file(dir.path(), "a.json", r#"{"a":1,"b":2}"#);
    let path = dir.path().join("a.json");
    let error = load(Layers::new().push(FileSource::new(&path).accepts([["a"]]))).unwrap_err();
    match &error {
        ConfigError::UnknownKey {
            path: reported,
            key,
        } => {
            assert_eq!(reported, &path);
            assert_eq!(key, "b");
        }
        other => panic!("unexpected {other}"),
    }
    assert_eq!(error.code(), "CFG.UNKNOWN_KEY");
    let text = error.to_string();
    assert!(text.contains("b") && text.contains("a.json"), "{text}");
}

#[test]
fn accepts_allows_a_path_and_everything_under_it() {
    let dir = TempDir::new();
    write_file(dir.path(), "a.json", r#"{"laya":{"port":1}}"#);
    let source = FileSource::new(dir.path().join("a.json")).accepts([["laya"]]);
    assert_eq!(
        load(Layers::new().push(source)).unwrap(),
        json!({"laya":{"port":1}})
    );
    let narrower = FileSource::new(dir.path().join("a.json")).accepts([["laya", "host"]]);
    let error = load(Layers::new().push(narrower)).unwrap_err();
    assert_eq!(error.code(), "CFG.UNKNOWN_KEY");
}

#[test]
fn env_source_uses_its_literal_prefix() {
    let env = environment(&[("DECIDR_LAYA__PORT", "1"), ("MY_APP__X", "2")]);
    let layers = Layers::new().push(EnvSource::new("DECIDR_", env));
    assert_eq!(load(layers).unwrap(), json!({"laya": {"port": 1}}));
}

#[test]
fn an_empty_prefix_matches_every_variable() {
    let layers = Layers::new().push(EnvSource::new("", environment(&[("A__B", "1")])));
    assert_eq!(load(layers).unwrap(), json!({"a": {"b": 1}}));
}

#[test]
fn an_alias_maps_one_exact_variable_and_applies_after_prefixed_ones() {
    let env = environment(&[
        ("SC_PROJECT", "x"),
        ("MY_APP__PROJECT", "y"),
        ("SC_PROJECT_2", "z"),
    ]);
    let source = EnvSource::new("MY_APP__", env).alias("SC_PROJECT", ["project"]);
    assert_eq!(
        load(Layers::new().push(source)).unwrap(),
        json!({"project": "x"})
    );
}

#[test]
fn an_override_through_a_non_object_below_is_an_override_error() {
    let below = ValueSource::new(Source::default("d"), json!({"a": 1}));
    let env = EnvSource::new("MY_APP__", environment(&[("MY_APP__A__B", "2")]));
    let error = load(Layers::new().push(below).push(env)).unwrap_err();
    match error {
        ConfigError::Override { variable } => assert_eq!(variable, "MY_APP__A__B"),
        other => panic!("unexpected {other}"),
    }
}

#[test]
fn an_env_object_replaces_what_is_below_it() {
    let below = ValueSource::new(
        Source::default("d"),
        json!({"daemon": {"host": "h", "port": 1}}),
    );
    let env = EnvSource::new(
        "MY_APP__",
        environment(&[("MY_APP__DAEMON", r#"{"port":2}"#)]),
    );
    let value = load(Layers::new().push(below).push(env)).unwrap();
    assert_eq!(value, json!({"daemon": {"port": 2}}));
}

#[test]
fn a_longer_variable_name_wins_over_the_name_it_extends() {
    let env = environment(&[("MY_APP__A__B", "2"), ("MY_APP__A", r#"{"b":1,"c":1}"#)]);
    let value = load(Layers::new().push(EnvSource::new("MY_APP__", env))).unwrap();
    assert_eq!(value, json!({"a": {"b": 2, "c": 1}}));
}

#[test]
fn value_source_contributes_its_value() {
    let flags = ValueSource::new(Source::flag("--port"), json!({"port": 9000}));
    assert_eq!(
        load(Layers::new().push(flags)).unwrap(),
        json!({"port": 9000})
    );
}

#[test]
fn new_error_variants_have_codes_and_name_what_they_carry() {
    let failed = ConfigError::SourceFailed {
        origin: Source::custom("keychain", "login"),
        message: "locked".to_owned(),
    };
    assert_eq!(failed.code(), "CFG.SOURCE");
    assert!(failed.to_string().contains("keychain login"));
}
