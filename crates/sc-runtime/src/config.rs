//! Daemon settings the application loads before [`crate::Daemon::builder`].

use std::ffi::OsString;

use sc_transport::TransportConfig;
use serde::Deserialize;

/// Settings `run` reads. The application loads this value. This crate does not.
///
/// [`transport`](Self::transport) is the `sc-transport` configuration, embedded
/// unchanged. A project's CLI depends on `sc-transport` and uses that type
/// directly. [`endpoint_flag`](Self::endpoint_flag) is the `--endpoint` value
/// the application's `main` already parsed. Absent means the flag was not
/// passed.
///
/// The endpoint environment variable is not a field. `run` passes this value to
/// [`sc_transport::resolve_endpoint_from_env`], which reads that variable.
///
/// Construct it with [`Default`] or deserialize it. The type is
/// non-exhaustive. v0.1 binds the one endpoint inside `transport`.
///
/// # Examples
///
/// ```
/// use sc_runtime::DaemonConfig;
///
/// fn assert_de<T: serde::de::DeserializeOwned>() {}
/// assert_de::<DaemonConfig>();
/// let mut config = DaemonConfig::default();
/// config.transport.tcp_port = Some(9);
/// # let _ = config;
/// ```
#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize)]
#[non_exhaustive]
pub struct DaemonConfig {
    /// Endpoint, instance root, and Windows TCP port from `sc-transport`.
    #[serde(flatten)]
    pub transport: TransportConfig,
    /// `--endpoint` text. `None` means the flag was absent.
    ///
    /// A present value that does not parse fails endpoint resolution. It does
    /// not fall through to the environment or the platform default.
    #[serde(default)]
    pub endpoint_flag: Option<String>,
}

impl DaemonConfig {
    pub(crate) fn flag_os(&self) -> Option<OsString> {
        self.endpoint_flag.as_ref().map(OsString::from)
    }
}
