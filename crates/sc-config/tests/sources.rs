//! Built-in sources: REQ-CFG-0006.

#[path = "support/mod.rs"]
mod support;

use sc_config::{ConfigError, EnvSource, FileSource, LayerSource, Layers, Source, ValueSource};
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
    let source = FileSource::new(dir.path().join("a.json")).optional();
    assert!(source.load(&json!({})).unwrap().is_none());
    assert_eq!(load(Layers::new().push(source)).unwrap(), json!({}));
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
    let expected = format!("key b in {} is not accepted", path.display());
    assert!(text.contains(&expected), "{text}");
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

#[test]
fn an_alias_replaces_whole_and_is_attributed_to_its_variable() {
    let below = ValueSource::new(Source::default("d"), json!({"project": {"a": 1, "b": 2}}));
    let env = EnvSource::new("MY_APP__", environment(&[("SC_PROJECT", r#"{"c":3}"#)]))
        .alias("SC_PROJECT", ["project"]);
    let resolved = Layers::new().push(below).push(env).resolve().unwrap();
    assert_eq!(resolved.value, json!({"project": {"c": 3}}));
    let rows = resolved.rows();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].path, ["project", "c"]);
    assert_eq!(rows[0].source, Source::env("SC_PROJECT"));
}

#[test]
fn deserialize_lists_the_real_file_paths_and_only_files() {
    #[derive(Debug, serde::Deserialize)]
    struct Strict {
        _n: u8,
    }
    let dir = TempDir::new();
    write_file(dir.path(), "a.json", r#"{"_n":"x"}"#);
    let path = dir.path().join("a.json");
    let labelled = ValueSource::new(Source::file("not-a-file.json"), json!({}));
    let layers = Layers::new().push(FileSource::new(&path)).push(labelled);
    match layers.load::<Strict>().unwrap_err() {
        ConfigError::Deserialize { files, .. } => assert_eq!(files, vec![path]),
        other => panic!("unexpected {other}"),
    }
}

// macOS file systems reject non-UTF-8 names, so only Linux can create this file.
#[cfg(target_os = "linux")]
#[test]
fn a_non_utf8_path_is_reported_unchanged() {
    use std::ffi::OsString;
    use std::os::unix::ffi::OsStringExt;

    #[derive(Debug, serde::Deserialize)]
    struct Strict {
        _n: u8,
    }
    let dir = TempDir::new();
    let mut raw = dir.path().as_os_str().to_owned().into_vec();
    raw.extend_from_slice(b"/\xff.json");
    let path = std::path::PathBuf::from(OsString::from_vec(raw));
    std::fs::write(&path, r#"{"_n":"x"}"#).unwrap();
    match Layers::new()
        .push(FileSource::new(&path))
        .load::<Strict>()
        .unwrap_err()
    {
        ConfigError::Deserialize { files, .. } => assert_eq!(files, vec![path]),
        other => panic!("unexpected {other}"),
    }
}
