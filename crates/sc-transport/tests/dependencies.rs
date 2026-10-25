//! Executable guard for the default and server dependency graphs.

use std::collections::BTreeSet;
use std::process::Command;

const FORBIDDEN: &[&str] = &[
    "sc-observability",
    "sc-observability-types",
    "sc-observability-otlp",
    "sc-command",
    "sc-config",
    "sc-runtime",
    "hyperlocal",
    "rustls",
    "rustls-webpki",
    "hyper-rustls",
    "aws-lc-rs",
    "aws-lc-sys",
];

#[test]
fn cargo_tree_matches_the_allowed_graph() {
    let default_tree = cargo_tree(&[]);
    assert_absent(&default_tree, FORBIDDEN);
    assert_absent(&default_tree, &["axum"]);
    assert_eq!(
        direct_names(&cargo_tree(&["--depth", "1"])),
        BTreeSet::from([
            "reqwest".to_owned(),
            "serde".to_owned(),
            "serde_json".to_owned(),
            "tokio".to_owned(),
        ])
    );

    let server_tree = cargo_tree(&["--features", "server"]);
    assert_absent(&server_tree, FORBIDDEN);
    assert!(
        server_tree.lines().any(|line| line.starts_with("axum ")),
        "{server_tree}"
    );
    assert_eq!(
        direct_names(&cargo_tree(&["--depth", "1", "--features", "server"])),
        BTreeSet::from([
            "axum".to_owned(),
            "reqwest".to_owned(),
            "serde".to_owned(),
            "serde_json".to_owned(),
            "tokio".to_owned(),
        ])
    );
}

fn cargo_tree(extra: &[&str]) -> String {
    let target = std::env::temp_dir().join(format!(
        "sc-transport-tree-{}-{}",
        std::process::id(),
        extra.join("_").replace('-', "_")
    ));
    let mut command = Command::new(env!("CARGO"));
    command
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .env("CARGO_TARGET_DIR", &target)
        .args([
            "tree",
            "-p",
            "sc-transport",
            "-e",
            "normal",
            "--prefix",
            "none",
            "--locked",
            "--offline",
        ])
        .args(extra);
    let output = command
        .output()
        .unwrap_or_else(|err| panic!("spawn cargo tree: {err}"));
    assert!(
        output.status.success(),
        "cargo tree {extra:?} failed\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).unwrap_or_else(|err| panic!("cargo tree utf-8: {err}"))
}

fn direct_names(tree: &str) -> BTreeSet<String> {
    tree.lines()
        .filter_map(|line| line.split_whitespace().next())
        .filter(|name| *name != "sc-transport")
        .map(str::to_owned)
        .collect()
}

fn assert_absent(tree: &str, names: &[&str]) {
    for line in tree.lines() {
        for name in names {
            let prefix = format!("{name} ");
            assert!(
                !line.starts_with(&prefix),
                "forbidden crate {name} in `{line}`"
            );
        }
    }
}
