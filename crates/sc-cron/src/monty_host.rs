//! Host functions a Monty hook may call.
//!
//! Production feeds compile with an empty extension map and then call
//! `cron_stop` / `cron_tick` through `MontyRepl::call_function`. Sprint b-10
//! replaces that call with a progress loop so hook code can call the names in
//! [`planned_allowlist`]. The host will run the Rust `sc-compose` binary for
//! `render_template`. It will not import Python and it will not link
//! `monty-pool`.
//!
//! # b-10 remaining work
//!
//! - Drive tick and stop through Monty's progress API, handling `FunctionCall`.
//! - Deny `OsCall` unless an explicit allowlist says otherwise.
//! - Register `render_template(template, context) -> str` by spawning
//!   `sc-compose render`, and map a failed compose onto `RunOutcome::Failed`.
//! - Count host-call wall time against the feed deadline.
//! - Add an end-to-end test that skips when `sc-compose` is absent and runs on
//!   the Rust 1.96.1 Monty job.

use monty_types::NamedValues;

/// Host function the b-10 bridge will inject. Hook authors call it bare.
pub(crate) const RENDER_TEMPLATE: &str = "render_template";

/// Names b-10 will register. Nothing in this list is registered yet.
pub(crate) fn planned_allowlist() -> &'static [&'static str] {
    &[RENDER_TEMPLATE]
}

/// Extensions passed to `MontyRepl::feed_run` for one compile.
///
/// The map is empty. [`planned_allowlist`] names the functions b-10 will add.
pub(crate) fn extensions() -> NamedValues {
    let _planned = planned_allowlist();
    NamedValues::new()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn host_bridge_registers_nothing_yet() {
        assert_eq!(planned_allowlist(), &["render_template"]);
        let _extensions = extensions();
    }
}
