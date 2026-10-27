//! Proves rusqlite 0.39 and template sqlx 0.9 sqlite resolve one libsqlite3-sys.

use std::path::PathBuf;
use std::process::Command;

#[test]
fn rusqlite_0_39_resolves_one_libsqlite3_sys_with_sqlx_0_9_sqlite() {
    let manifest =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/sqlx-resolve/Cargo.toml");
    let output = Command::new(env!("CARGO"))
        .args([
            "metadata",
            "--format-version",
            "1",
            "--locked",
            "--manifest-path",
        ])
        .arg(&manifest)
        .output()
        .expect("spawn cargo metadata");
    assert!(
        output.status.success(),
        "cargo metadata failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let metadata = String::from_utf8(output.stdout).expect("metadata utf-8");
    let versions = package_versions(&metadata, "libsqlite3-sys");
    assert_eq!(
        versions.len(),
        1,
        "expected one libsqlite3-sys, found {versions:?}"
    );
    let version = versions[0].as_str();
    assert!(
        version.starts_with("0.37."),
        "sqlx 0.9 sqlite accepts libsqlite3-sys >=0.30.1, <0.38.0, found {version}"
    );
    let rusqlite = package_versions(&metadata, "rusqlite");
    assert!(
        rusqlite.iter().any(|version| version.starts_with("0.39.")),
        "fixture did not resolve rusqlite 0.39, found {rusqlite:?}"
    );
}

/// Package objects in `cargo metadata` are `"name":"<name>","version":"<ver>"`.
/// Resolve nodes repeat the name without that version field.
fn package_versions(metadata: &str, name: &str) -> Vec<String> {
    let needle = format!("\"name\":\"{name}\",\"version\":\"");
    let mut versions = Vec::new();
    let mut rest = metadata;
    while let Some(index) = rest.find(&needle) {
        let tail = &rest[index + needle.len()..];
        let Some(end) = tail.find('"') else {
            break;
        };
        versions.push(tail[..end].to_owned());
        rest = &tail[end..];
    }
    versions.sort();
    versions.dedup();
    versions
}
