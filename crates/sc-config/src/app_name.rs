use crate::error::ConfigError;

/// Validated application name used as the configuration identity.
///
/// Construction is the only validation site. The allowed characters are ASCII
/// `A-Z`, `a-z`, `0-9`, `-`, and `_`, and the length is 1 through 64 bytes.
/// [`env_prefix`](Self::env_prefix) upper-cases ASCII letters and maps both
/// `-` and `_` to `_`, so `my-app` and `my_app` share the prefix `MY_APP__`.
/// Display the name with [`as_str`](Self::as_str) or [`AsRef<str>`].
/// This type does not implement [`Deref`](std::ops::Deref).
///
/// # Examples
///
/// ```
/// use sc_config::AppName;
///
/// let app = AppName::new("my-app")?;
/// let _name = app.as_str();
/// # Ok::<(), sc_config::ConfigError>(())
/// ```
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct AppName(String);

impl AppName {
    /// Validates `name` and returns an application name.
    ///
    /// # Errors
    ///
    /// Returns [`ConfigError::InvalidName`] with code `APP.INVALID_NAME` when
    /// `name` is empty, longer than 64 bytes, or contains a character outside
    /// ASCII letters, digits, `-`, and `_`.
    pub fn new(name: &str) -> Result<Self, ConfigError> {
        if is_valid(name) {
            Ok(Self(name.to_owned()))
        } else {
            Err(ConfigError::InvalidName {
                name: name.to_owned(),
            })
        }
    }

    /// Returns the validated name.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Returns the environment-variable prefix for this application name.
    ///
    /// ASCII letters are converted to upper case. `-` and `_` both become
    /// `_`. The result ends with `__`. `my-app` and `my_app` both return
    /// `MY_APP__`.
    ///
    /// # Examples
    ///
    /// ```
    /// use sc_config::AppName;
    ///
    /// let dashed = AppName::new("my-app")?;
    /// let underscored = AppName::new("my_app")?;
    /// assert_eq!(dashed.env_prefix(), "MY_APP__");
    /// assert_eq!(underscored.env_prefix(), dashed.env_prefix());
    /// # Ok::<(), sc_config::ConfigError>(())
    /// ```
    pub fn env_prefix(&self) -> String {
        let mut prefix = String::with_capacity(self.as_str().len().saturating_add(2));
        for byte in self.as_str().bytes() {
            let mapped = if byte == b'-' {
                b'_'
            } else {
                byte.to_ascii_uppercase()
            };
            prefix.push(char::from(mapped));
        }
        prefix.push_str("__");
        prefix
    }
}

impl AsRef<str> for AppName {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl std::fmt::Display for AppName {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl TryFrom<&str> for AppName {
    type Error = ConfigError;

    fn try_from(name: &str) -> Result<Self, Self::Error> {
        Self::new(name)
    }
}

fn is_valid(name: &str) -> bool {
    let bytes = name.as_bytes();
    if bytes.is_empty() || bytes.len() > 64 {
        return false;
    }
    bytes
        .iter()
        .all(|byte| byte.is_ascii_alphanumeric() || *byte == b'-' || *byte == b'_')
}
