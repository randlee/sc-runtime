#[path = "support/mod.rs"]
mod support;

use std::path::Path;

use sc_config::ConfigError;
use serde::Deserialize;
use support::{load_value, write_file, TempDir};

fn assert_error<E: std::error::Error>() {}

#[test]
fn config_error_implements_error() {
    assert_error::<ConfigError>();
}

#[test]
fn missing_default_carries_the_path() {
    let dir = TempDir::new();
    let error = load_value(dir.path(), "my-app", &[]).unwrap_err();
    match error {
        ConfigError::MissingDefault { path } => {
            assert!(path.ends_with("default.json"));
            assert!(error_text(&ConfigError::MissingDefault { path }).contains("default.json"));
        }
        other => panic!("unexpected {other}"),
    }
}

fn error_text(error: &ConfigError) -> String {
    error.to_string()
}

#[test]
fn unreadable_default_and_local_are_read_errors() {
    let dir = TempDir::new();
    std::fs::create_dir(dir.path().join("default.json")).unwrap();
    let error = load_value(dir.path(), "my-app", &[]).unwrap_err();
    match &error {
        ConfigError::Read { path, .. } => {
            assert!(path.ends_with("default.json"));
            assert!(error.to_string().contains(&path.display().to_string()));
        }
        other => panic!("unexpected {other}"),
    }

    let dir = TempDir::new();
    write_file(dir.path(), "default.json", "{}");
    std::fs::create_dir(dir.path().join("local.json")).unwrap();
    let error = load_value(dir.path(), "my-app", &[]).unwrap_err();
    match error {
        ConfigError::Read { path, .. } => assert!(path.ends_with("local.json")),
        other => panic!("unexpected {other}"),
    }
}

#[test]
fn invalid_json_reports_line_and_path() {
    let dir = TempDir::new();
    write_file(dir.path(), "default.json", "{\"a\": 1,\n  \"b\": }");
    let error = load_value(dir.path(), "my-app", &[]).unwrap_err();
    match &error {
        ConfigError::Parse { path, line, .. } => {
            assert!(path.ends_with("default.json"));
            assert_eq!(*line, 2);
            let text = error.to_string();
            assert!(text.contains(&path.display().to_string()));
            assert!(text.contains("line 2"));
        }
        other => panic!("unexpected {other}"),
    }
}

#[test]
fn invalid_local_json_and_non_utf8_are_parse_errors() {
    let dir = TempDir::new();
    write_file(dir.path(), "default.json", "{}");
    write_file(dir.path(), "local.json", "{");
    let error = load_value(dir.path(), "my-app", &[]).unwrap_err();
    match error {
        ConfigError::Parse { path, .. } => assert!(path.ends_with("local.json")),
        other => panic!("unexpected {other}"),
    }

    let dir = TempDir::new();
    write_file(dir.path(), "default.json", [0xff, 0xfe, 0x00]);
    let error = load_value(dir.path(), "my-app", &[]).unwrap_err();
    match error {
        ConfigError::Parse { path, .. } => assert!(path.ends_with("default.json")),
        other => panic!("unexpected {other}"),
    }
}

#[test]
fn failed_override_carries_the_variable_name() {
    let dir = TempDir::new();
    write_file(dir.path(), "default.json", r#"{"a":1}"#);
    let error = load_value(dir.path(), "my-app", &[("MY_APP__A__B", "2")]).unwrap_err();
    match &error {
        ConfigError::Override { variable } => {
            assert_eq!(variable, "MY_APP__A__B");
            assert!(error.to_string().contains("MY_APP__A__B"));
        }
        other => panic!("unexpected {other}"),
    }
}

#[test]
fn type_mismatch_lists_files_and_variables() {
    #[derive(Debug, Deserialize)]
    struct Target {
        #[allow(dead_code)]
        port: u16,
        #[allow(dead_code)]
        name: String,
    }

    let dir = TempDir::new();
    write_file(dir.path(), "default.json", r#"{"port":"not-a-number"}"#);
    write_file(dir.path(), "local.json", "{}");
    let error = sc_config::Loader::new(
        sc_config::AppName::new("my-app").unwrap(),
        dir.path().to_path_buf(),
        support::environment(&[("MY_APP__NAME", "x")]),
    )
    .load::<Target>()
    .unwrap_err();
    match &error {
        ConfigError::Deserialize {
            files, variables, ..
        } => {
            assert!(files.iter().any(|path| path.ends_with("default.json")));
            assert!(files.iter().any(|path| path.ends_with("local.json")));
            assert!(variables.iter().any(|name| name == "MY_APP__NAME"));
            let text = error.to_string();
            assert!(text.contains("MY_APP__NAME"));
            assert!(files
                .iter()
                .all(|path| text.contains(&path.display().to_string())));
        }
        other => panic!("unexpected {other}"),
    }
}

#[test]
fn stable_codes_match_the_inventory() {
    let missing = ConfigError::MissingDefault {
        path: Path::new("default.json").to_path_buf(),
    };
    assert_eq!(missing.code(), "CFG.MISSING_DEFAULT");
    let read = ConfigError::Read {
        path: Path::new("local.json").to_path_buf(),
        source: std::io::Error::other("denied"),
    };
    assert_eq!(read.code(), "CFG.READ");
    let parse = ConfigError::Parse {
        path: Path::new("default.json").to_path_buf(),
        line: 2,
        column: 4,
        message: "eof".to_owned(),
    };
    assert_eq!(parse.code(), "CFG.PARSE");
    assert!(parse.to_string().contains("line 2"));
    let over = ConfigError::Override {
        variable: "MY_APP__A".to_owned(),
    };
    assert_eq!(over.code(), "CFG.OVERRIDE");
    let typed = ConfigError::Deserialize {
        message: "type".to_owned(),
        files: vec![Path::new("default.json").to_path_buf()],
        variables: vec!["MY_APP__A".to_owned()],
    };
    assert_eq!(typed.code(), "CFG.DESERIALIZE");
    assert!(typed.to_string().contains("MY_APP__A"));
    let name = ConfigError::InvalidName {
        name: "my app".to_owned(),
    };
    assert_eq!(name.code(), "APP.INVALID_NAME");
    assert!(!name.recovery().is_empty());
}

#[test]
fn malformed_input_returns_errors() {
    let empty = TempDir::new();
    write_file(empty.path(), "default.json", "");
    assert!(load_value(empty.path(), "my-app", &[]).is_err());

    let bytes = TempDir::new();
    write_file(bytes.path(), "default.json", [0xff, 0xfe, 0x00]);
    assert!(load_value(bytes.path(), "my-app", &[]).is_err());

    let truncated = TempDir::new();
    write_file(truncated.path(), "default.json", r#"{"a":"#);
    assert!(load_value(truncated.path(), "my-app", &[]).is_err());

    let missing = TempDir::new();
    let absent = missing.path().join("does-not-exist");
    let error = load_value(&absent, "my-app", &[]).unwrap_err();
    assert!(matches!(error, ConfigError::MissingDefault { .. }));

    let array = TempDir::new();
    write_file(array.path(), "default.json", "[1,2]");
    let error = load_value(array.path(), "my-app", &[("MY_APP__A", "1")]).unwrap_err();
    assert!(matches!(error, ConfigError::Override { .. }));
}
