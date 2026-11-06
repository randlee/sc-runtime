#[path = "../../../../../crates/sc-cron/tests/hook_environment/owned_child.rs"]
mod owned_child;
mod common {
    pub(super) const WAIT: std::time::Duration = std::time::Duration::from_secs(5);
}
fn python_program() -> &'static str {
    if cfg!(windows) { "python" } else { "python3" }
}
pub(super) fn supervised(name: &str) -> bool {
    if std::env::var("SC_C5_CONSUMER_OWNED_TEST").as_deref() == Ok(name) {
        return false;
    }
    let mut command = std::process::Command::new(std::env::current_exe().unwrap());
    command
        .args(["--exact", name, "--nocapture"])
        .env("SC_C5_CONSUMER_OWNED_TEST", name);
    let output = owned_child::output(&mut command, std::time::Duration::from_secs(30));
    assert!(
        output.status.success(),
        "owned consumer failed: {}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    true
}
