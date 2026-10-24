//! Endpoint and instance-root resolution.

use std::ffi::OsStr;
use std::net::{Ipv4Addr, SocketAddr};
use std::path::{Path, PathBuf};

use crate::app_name::AppName;
use crate::config::TransportConfig;
use crate::endpoint::Endpoint;
use crate::error::TransportError;

#[cfg(unix)]
const DAEMON_SOCKET_FILE: &str = "daemon.sock";

/// Returns the default instance root for `app` and the current user.
///
/// On Unix the base is `XDG_RUNTIME_DIR` when that variable is non-empty,
/// otherwise `HOME` joined with `.local/state`. On Windows the base is
/// `LOCALAPPDATA`. The returned path is `<base>/<app>` and is not created.
///
/// # Errors
///
/// Returns [`TransportError::InstanceRootUnavailable`] when the base location
/// is missing.
///
/// # Examples
///
/// ```no_run
/// use sc_transport::{instance_root, AppName};
///
/// let app = AppName::new("my-app")?;
/// let root = instance_root(&app)?;
/// # let _ = root;
/// # Ok::<(), sc_transport::TransportError>(())
/// ```
pub fn instance_root(app: &AppName) -> Result<PathBuf, TransportError> {
    let base = user_base()?;
    instance_root_from_base(app, Some(&base))
}

/// Builds `<base>/<app>`, or fails when `base` is absent.
///
/// `instance_root` obtains `base` from the operating system and calls this
/// function. The directory is not created. The function stays crate-private
/// so tests can supply a base without publishing a second resolver.
///
/// # Errors
///
/// Returns [`TransportError::InstanceRootUnavailable`] when `base` is `None`.
pub(crate) fn instance_root_from_base(
    app: &AppName,
    base: Option<&Path>,
) -> Result<PathBuf, TransportError> {
    let Some(base) = base else {
        return Err(TransportError::instance_root(
            "per-user base directory is absent",
        ));
    };
    if base.as_os_str().is_empty() {
        return Err(TransportError::instance_root(
            "per-user base directory is empty",
        ));
    }
    Ok(base.join(app.as_str()))
}

/// Resolves one endpoint from the first source that supplies a value.
///
/// Precedence is the `--endpoint` flag (`flag`), then `SC_ENDPOINT`
/// (`environment`), then [`TransportConfig::endpoint`], then the platform
/// default. `None` means that source is absent. A present value that does not
/// parse is an error and does not fall through.
///
/// The Unix default is a UDS path `<instance-root>/daemon.sock`. An explicit
/// [`TransportConfig::instance_root`] is used unchanged. The Windows default
/// is TCP on `127.0.0.1` with [`TransportConfig::tcp_port`]. This function
/// does not create directories and does not bind.
///
/// # Errors
///
/// Returns [`TransportError`] when a supplied value does not parse, when the
/// instance root cannot be determined, or when the Windows default has no port.
///
/// # Examples
///
/// ```
/// use sc_transport::{resolve_endpoint, AppName, Endpoint, TransportConfig};
///
/// let app = AppName::new("my-app")?;
/// let endpoint = resolve_endpoint(
///     &app,
///     Some(std::ffi::OsStr::new("tcp:127.0.0.1:9")),
///     None,
///     &TransportConfig::default(),
/// )?;
/// assert_eq!(endpoint, "tcp:127.0.0.1:9".parse::<Endpoint>()?);
/// # Ok::<(), sc_transport::TransportError>(())
/// ```
pub fn resolve_endpoint(
    app: &AppName,
    flag: Option<&OsStr>,
    environment: Option<&OsStr>,
    config: &TransportConfig,
) -> Result<Endpoint, TransportError> {
    if let Some(flag) = flag {
        return Endpoint::parse_os(flag);
    }
    if let Some(environment) = environment {
        return Endpoint::parse_os(environment);
    }
    if let Some(configured) = &config.endpoint {
        return configured.parse();
    }
    platform_default(app, config)
}

/// Resolves an endpoint, reading `SC_ENDPOINT` from the process environment.
///
/// The variable is read with `std::env::var_os` and passed to
/// [`resolve_endpoint`]. Prefer [`resolve_endpoint`] when tests run in parallel.
///
/// # Errors
///
/// Returns the same errors as [`resolve_endpoint`].
pub fn resolve_endpoint_from_env(
    app: &AppName,
    flag: Option<&OsStr>,
    config: &TransportConfig,
) -> Result<Endpoint, TransportError> {
    let environment = std::env::var_os("SC_ENDPOINT");
    resolve_endpoint(app, flag, environment.as_deref(), config)
}

fn platform_default(app: &AppName, config: &TransportConfig) -> Result<Endpoint, TransportError> {
    #[cfg(unix)]
    {
        let root = match &config.instance_root {
            Some(root) => root.clone(),
            None => instance_root(app)?,
        };
        Ok(Endpoint::Uds(root.join(DAEMON_SOCKET_FILE)))
    }
    #[cfg(not(unix))]
    {
        let _ = (app, &config.instance_root);
        windows_default_endpoint(config.tcp_port)
    }
}

#[cfg_attr(all(unix, not(test)), allow(dead_code))]
pub(crate) fn windows_default_endpoint(port: Option<u16>) -> Result<Endpoint, TransportError> {
    let Some(port) = port else {
        return Err(TransportError::missing_tcp_port());
    };
    Ok(Endpoint::Tcp(SocketAddr::from((Ipv4Addr::LOCALHOST, port))))
}

fn user_base() -> Result<PathBuf, TransportError> {
    #[cfg(unix)]
    {
        if let Some(runtime) = std::env::var_os("XDG_RUNTIME_DIR")
            && !runtime.is_empty()
        {
            return Ok(PathBuf::from(runtime));
        }
        if let Some(home) = std::env::var_os("HOME")
            && !home.is_empty()
        {
            return Ok(PathBuf::from(home).join(".local").join("state"));
        }
        Err(TransportError::instance_root(
            "set XDG_RUNTIME_DIR or HOME, or pass TransportConfig::instance_root",
        ))
    }
    #[cfg(not(unix))]
    {
        if let Some(local) = std::env::var_os("LOCALAPPDATA")
            && !local.is_empty()
        {
            return Ok(PathBuf::from(local));
        }
        Err(TransportError::instance_root(
            "set LOCALAPPDATA, or pass TransportConfig::instance_root",
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_windows_port_is_an_error() {
        let err = windows_default_endpoint(None).unwrap_err();
        assert!(matches!(
            &err,
            TransportError::MissingTcpPort { code, detail }
                if *code == "TRANSPORT.MISSING_TCP_PORT" && detail.contains("tcp_port")
        ));
        assert!(err.to_string().contains("tcp_port"), "{err}");
    }

    #[test]
    fn path_builder_uses_the_supplied_base() {
        let one = tempfile::tempdir().unwrap();
        let two = tempfile::tempdir().unwrap();
        let name = AppName::new("app-a").unwrap();
        let first = instance_root_from_base(&name, Some(one.path())).unwrap();
        let second = instance_root_from_base(&name, Some(two.path())).unwrap();
        assert_ne!(first, second);
        assert!(first.starts_with(one.path()));
        assert!(second.starts_with(two.path()));
    }

    #[test]
    fn absent_base_is_an_error() {
        let err = instance_root_from_base(&AppName::new("app-a").unwrap(), None).unwrap_err();
        assert!(matches!(
            err,
            TransportError::InstanceRootUnavailable { .. }
        ));
        assert_eq!(err.code(), "TRANSPORT.INSTANCE_ROOT_UNAVAILABLE");
    }

    #[test]
    fn configured_windows_port_is_loopback() {
        let endpoint = windows_default_endpoint(Some(9)).unwrap();
        let Endpoint::Tcp(addr) = endpoint else {
            panic!("expected tcp");
        };
        assert!(addr.ip().is_loopback());
        assert_eq!(addr.port(), 9);
    }
}
