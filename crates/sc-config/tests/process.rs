#[path = "support/mod.rs"]
mod support;

use std::process::Command;

use serde_json::json;
use support::{write_file, TempDir};

#[test]
fn load_helper_is_excluded_from_the_published_package() {
    let manifest = include_str!("../Cargo.toml");
    assert!(
        manifest.contains("exclude = [\"src/bin/\"]"),
        "the process helper must stay out of the published package"
    );
    assert!(
        !manifest.contains("[[bin]]"),
        "an explicit bin target is part of the published package"
    );
}

fn helper(current_dir: &std::path::Path) -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_sc-config-load-helper"));
    command.env_clear().current_dir(current_dir);
    if let Some(path) = std::env::var_os("PATH") {
        command.env("PATH", path);
    }
    command
}

#[test]
fn load_reads_the_child_directory_and_environment() {
    let dir = TempDir::new();
    let config = dir.path().join("config");
    std::fs::create_dir(&config).unwrap();
    write_file(&config, "default.json", r#"{"a":0}"#);
    let output = helper(dir.path())
        .env("MY_APP__A", "1")
        .arg("load")
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let value: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(value, json!({"a": 1}));
}

#[test]
fn loader_ignores_the_process_environment() {
    let dir = TempDir::new();
    let config = TempDir::new();
    write_file(config.path(), "default.json", r#"{"a":0}"#);
    let output = helper(dir.path())
        .env("MY_APP__A", "1")
        .arg("loader")
        .arg(config.path())
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let value: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(value, json!({"a": 0}));
}
