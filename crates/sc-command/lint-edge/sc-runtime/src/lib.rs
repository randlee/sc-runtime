//! Workspace member that depends on `sc-command`.
//!
//! sc-lint 0.8.0 accepts `allowed_dependents = ["sc-runtime"]` only when that
//! package exists and depends on `sc-command`. Sprint a-5 replaces this
//! package with `crates/sc-runtime` and deletes this directory.

/// Returns the envelope contract version from `sc-command`.
#[must_use]
pub fn envelope_version() -> &'static str {
    sc_command::ENVELOPE_VERSION
}
