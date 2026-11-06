fn main() {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../../crates/sc-cron/tests/hook_environment");
    println!(
        "cargo:rustc-env=SC_CRON_TEST_UNIX_BRIDGE={}",
        root.join("owned_child_unix.py").display()
    );
    println!(
        "cargo:rustc-env=SC_CRON_TEST_WINDOWS_BRIDGE={}",
        root.join("owned_child.py").display()
    );
}
