//! Validated application name for endpoint and instance-root resolution.

use std::fmt;

use crate::error::TransportError;

/// Client wire identity for [`crate::resolve_endpoint`] and [`crate::instance_root`].
///
/// Construction is the only validation site. The accepted characters are ASCII
/// `A-Z`, `a-z`, `0-9`, `-`, and `_`, with a length of 1 through 64 bytes.
/// Rejection uses the stable code `APP.INVALID_NAME`.
///
/// # Examples
///
/// ```
/// use sc_transport::AppName;
///
/// let name = AppName::new("my-app")?;
/// assert_eq!(name.as_str(), "my-app");
/// assert!(AppName::new("my app").is_err());
/// # Ok::<(), sc_transport::TransportError>(())
/// ```
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AppName(String);

impl AppName {
    /// Validates `name` and returns the newtype.
    ///
    /// # Errors
    ///
    /// Returns [`TransportError::InvalidName`] when `name` is empty, longer
    /// than 64 bytes, or contains a character outside the allowed set.
    pub fn new(name: impl AsRef<str>) -> Result<Self, TransportError> {
        let name = name.as_ref();
        if !is_valid(name) {
            return Err(TransportError::invalid_name(name));
        }
        Ok(Self(name.to_owned()))
    }

    /// Returns the validated name.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl AsRef<str> for AppName {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl fmt::Display for AppName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl TryFrom<&str> for AppName {
    type Error = TransportError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl TryFrom<String> for AppName {
    type Error = TransportError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

fn is_valid(name: &str) -> bool {
    let bytes = name.as_bytes();
    (1..=64).contains(&bytes.len())
        && bytes
            .iter()
            .all(|byte| byte.is_ascii_alphanumeric() || *byte == b'-' || *byte == b'_')
}
