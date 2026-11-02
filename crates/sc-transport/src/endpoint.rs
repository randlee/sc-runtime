//! Resolved UDS and TCP addresses, and the text form flags and config share.

use std::ffi::OsStr;
use std::fmt;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::str::FromStr;

use crate::error::TransportError;

/// A resolved daemon address.
///
/// The text form used by `--endpoint`, `SC_ENDPOINT`, and
/// [`crate::TransportConfig::endpoint`] is `uds:<path>` or `tcp:<socket-addr>`.
/// `tcp:` requires a literal [`SocketAddr`] (`127.0.0.1:9`, `[::1]:9`), not a
/// DNS name. An empty string is invalid and does not mean "absent".
///
/// # Examples
///
/// ```
/// use sc_transport::Endpoint;
///
/// let endpoint: Endpoint = "tcp:127.0.0.1:9".parse()?;
/// assert_eq!(endpoint.to_string(), "tcp:127.0.0.1:9");
/// # Ok::<(), sc_transport::TransportError>(())
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum Endpoint {
    /// Unix domain socket at `path`.
    Uds(PathBuf),
    /// TCP socket address.
    Tcp(SocketAddr),
}

impl Endpoint {
    /// Parses endpoint text from an `OsStr` flag or environment value.
    ///
    /// # Errors
    ///
    /// Returns [`TransportError::InvalidEndpoint`] when `text` is empty, is not
    /// valid Unicode on Windows, or is not `uds:<path>` or `tcp:<socket-addr>`.
    pub fn parse_os(text: &OsStr) -> Result<Self, TransportError> {
        parse_bytes(&os_bytes(text)?, text)
    }
}

impl FromStr for Endpoint {
    type Err = TransportError;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        Self::parse_os(OsStr::new(text))
    }
}

impl fmt::Display for Endpoint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Uds(path) => write!(f, "uds:{}", path.display()),
            Self::Tcp(addr) => write!(f, "tcp:{addr}"),
        }
    }
}

fn parse_bytes(bytes: &[u8], original: &OsStr) -> Result<Endpoint, TransportError> {
    if bytes.is_empty() {
        return Err(TransportError::invalid_endpoint(
            display_os(original),
            "endpoint text is empty",
        ));
    }
    if let Some(rest) = bytes.strip_prefix(b"uds:") {
        if rest.is_empty() {
            return Err(TransportError::invalid_endpoint(
                display_os(original),
                "uds path is empty",
            ));
        }
        return Ok(Endpoint::Uds(path_from_bytes(rest, original)?));
    }
    if let Some(rest) = bytes.strip_prefix(b"tcp:") {
        let text = std::str::from_utf8(rest).map_err(|_| {
            TransportError::invalid_endpoint(display_os(original), "tcp address is not utf-8")
        })?;
        let addr = text.parse::<SocketAddr>().map_err(|err| {
            TransportError::invalid_endpoint(display_os(original), err.to_string())
        })?;
        return Ok(Endpoint::Tcp(addr));
    }
    Err(TransportError::invalid_endpoint(
        display_os(original),
        "endpoint must start with uds: or tcp:",
    ))
}

fn display_os(text: &OsStr) -> String {
    text.to_string_lossy().into_owned()
}

#[cfg(unix)]
fn os_bytes(text: &OsStr) -> Result<std::borrow::Cow<'_, [u8]>, TransportError> {
    use std::os::unix::ffi::OsStrExt;
    Ok(std::borrow::Cow::Borrowed(text.as_bytes()))
}

#[cfg(windows)]
fn os_bytes(text: &OsStr) -> Result<std::borrow::Cow<'_, [u8]>, TransportError> {
    match text.to_str() {
        Some(text) => Ok(std::borrow::Cow::Owned(text.as_bytes().to_vec())),
        None => Err(TransportError::invalid_endpoint(
            display_os(text),
            "endpoint text is not valid Unicode",
        )),
    }
}

#[cfg(unix)]
fn path_from_bytes(bytes: &[u8], _original: &OsStr) -> Result<PathBuf, TransportError> {
    use std::os::unix::ffi::OsStrExt;
    Ok(PathBuf::from(OsStr::from_bytes(bytes)))
}

#[cfg(windows)]
fn path_from_bytes(bytes: &[u8], original: &OsStr) -> Result<PathBuf, TransportError> {
    match std::str::from_utf8(bytes) {
        Ok(text) => Ok(PathBuf::from(text)),
        Err(_) => Err(TransportError::invalid_endpoint(
            display_os(original),
            "endpoint text is not valid Unicode",
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::Endpoint;

    #[cfg(unix)]
    #[test]
    fn non_unicode_uds_path_is_preserved() {
        use std::ffi::OsStr;
        use std::os::unix::ffi::OsStrExt;

        let mut bytes = b"uds:/nonexistent/".to_vec();
        bytes.push(0xff);
        let text = OsStr::from_bytes(&bytes);
        let endpoint = Endpoint::parse_os(text).expect("unix path bytes are an endpoint");
        let Endpoint::Uds(path) = endpoint else {
            panic!("expected a uds endpoint");
        };
        assert!(path.as_os_str().as_bytes().ends_with(&[0xff]));
    }

    #[cfg(windows)]
    #[test]
    fn non_unicode_endpoint_is_invalid() {
        use std::ffi::OsString;
        use std::os::windows::ffi::OsStringExt;

        use crate::TransportError;

        let mut units: Vec<u16> = "tcp:".encode_utf16().collect();
        units.push(0xD800);
        let text = OsString::from_wide(&units);
        let err = Endpoint::parse_os(&text).expect_err("non-unicode");
        match err {
            TransportError::InvalidEndpoint { ref detail, .. } => {
                assert!(detail.contains("Unicode"), "{detail}");
            }
            other => panic!("expected InvalidEndpoint, got {other:?}"),
        }
    }
}
