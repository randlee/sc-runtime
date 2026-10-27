//! One membership decoder. The two-store adaptor loads this module's file.

mod membership_toml;

pub(crate) use membership_toml::membership_from_toml;
