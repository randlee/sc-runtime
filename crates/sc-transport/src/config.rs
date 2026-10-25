//! Endpoint and instance-root configuration shared by a CLI and a daemon.

use std::path::PathBuf;

use serde::Deserialize;

/// Endpoint and instance-root inputs the resolver reads from caller configuration.
///
/// v0.1 binds exactly one listener. This type is [`non_exhaustive`] so a later
/// dual-bind can add a field. It has no `additional_endpoints` field, and v0.1
/// does not reserve a value that callers are expected to ignore.
///
/// Construct it with [`Default`], then set the fields you need. Other crates
/// cannot use a struct literal.
///
/// # Examples
///
/// ```
/// use sc_transport::TransportConfig;
///
/// fn assert_de<T: serde::de::DeserializeOwned>() {}
///
/// assert_de::<TransportConfig>();
/// let mut config = TransportConfig::default();
/// config.tcp_port = Some(9);
/// # let _ = config;
/// ```
#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize)]
#[non_exhaustive]
pub struct TransportConfig {
    /// Endpoint text from configuration, in the same form as `--endpoint`.
    ///
    /// `None` means this source is absent. A present value that does not parse
    /// is an error and does not fall through to the platform default.
    #[serde(default)]
    pub endpoint: Option<String>,
    /// Instance root used unchanged in place of the per-user default.
    #[serde(default)]
    pub instance_root: Option<PathBuf>,
    /// TCP port for the Windows platform default (`127.0.0.1`).
    ///
    /// This crate has no hard-coded port. On Windows, an absent port with no
    /// endpoint override is [`crate::TransportError::MissingTcpPort`].
    #[serde(default)]
    pub tcp_port: Option<u16>,
}
