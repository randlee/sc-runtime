//! Response envelope `{version, ok, data, error}`.

use serde::de::{self, DeserializeOwned};
use serde::ser::SerializeMap;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::{Map, Value};

use crate::error::OpError;
use crate::{DISCRIMINANT_CODE, ENVELOPE_VERSION};

/// sc-ai-cli response envelope.
///
/// Success carries `data` and writes `error` as JSON `null`. Failure carries
/// [`OpError`] and writes `data` as JSON `null`. Constructors stamp
/// [`ENVELOPE_VERSION`]. Deserialization keeps the `version` string from the
/// input. An `ok` flag that disagrees with the other field, or an object that
/// does not have exactly the four keys, is rejected with
/// [`DISCRIMINANT_CODE`] in the error.
#[derive(Debug, Clone, PartialEq)]
pub enum Envelope<T> {
    /// `ok` is true and `data` is `T`.
    Success {
        /// Contract version read or written for this value.
        version: String,
        /// Successful payload. JSON `null` is a valid `T` such as `()` or `None`.
        data: T,
    },
    /// `ok` is false and `error` is the operation error.
    Failure {
        /// Contract version read or written for this value.
        version: String,
        /// Operation error.
        error: OpError,
    },
}

impl<T> Envelope<T> {
    /// Builds a success envelope at [`ENVELOPE_VERSION`].
    #[must_use]
    pub fn success(data: T) -> Self {
        Self::Success {
            version: ENVELOPE_VERSION.to_owned(),
            data,
        }
    }

    /// Builds a failure envelope at [`ENVELOPE_VERSION`].
    #[must_use]
    pub fn failure(error: OpError) -> Self {
        Self::Failure {
            version: ENVELOPE_VERSION.to_owned(),
            error,
        }
    }

    /// Contract version stored on this value.
    #[must_use]
    pub fn version(&self) -> &str {
        match self {
            Self::Success { version, .. } | Self::Failure { version, .. } => version,
        }
    }

    /// Reports whether this value is the success case.
    #[must_use]
    pub fn is_ok(&self) -> bool {
        matches!(self, Self::Success { .. })
    }

    /// Returns the payload when this value is the success case.
    #[must_use]
    pub fn data(&self) -> Option<&T> {
        match self {
            Self::Success { data, .. } => Some(data),
            Self::Failure { .. } => None,
        }
    }

    /// Returns the error when this value is the failure case.
    #[must_use]
    pub fn error(&self) -> Option<&OpError> {
        match self {
            Self::Failure { error, .. } => Some(error),
            Self::Success { .. } => None,
        }
    }
}

impl<T> From<Result<T, OpError>> for Envelope<T> {
    fn from(result: Result<T, OpError>) -> Self {
        match result {
            Ok(data) => Self::success(data),
            Err(error) => Self::failure(error),
        }
    }
}

impl<T> Serialize for Envelope<T>
where
    T: Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map = serializer.serialize_map(Some(4))?;
        match self {
            Self::Success { version, data } => {
                map.serialize_entry("version", version)?;
                map.serialize_entry("ok", &true)?;
                map.serialize_entry("data", data)?;
                map.serialize_entry("error", &None::<OpError>)?;
            }
            Self::Failure { version, error } => {
                map.serialize_entry("version", version)?;
                map.serialize_entry("ok", &false)?;
                map.serialize_entry("data", &None::<()>)?;
                map.serialize_entry("error", error)?;
            }
        }
        map.end()
    }
}

impl<'de, T> Deserialize<'de> for Envelope<T>
where
    T: DeserializeOwned,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = Value::deserialize(deserializer)?;
        envelope_from_value(value).map_err(de::Error::custom)
    }
}

fn envelope_from_value<T>(value: Value) -> Result<Envelope<T>, String>
where
    T: DeserializeOwned,
{
    let map = match value {
        Value::Object(map) => map,
        _ => {
            return Err(format!(
                "{DISCRIMINANT_CODE}: envelope must be a JSON object"
            ));
        }
    };
    require_exact_keys(&map, &["version", "ok", "data", "error"])?;
    let version = match required(&map, "version")? {
        Value::String(version) => version.clone(),
        _ => {
            return Err(format!(
                "{DISCRIMINANT_CODE}: version must be a JSON string"
            ));
        }
    };
    let ok = match required(&map, "ok")? {
        Value::Bool(ok) => *ok,
        _ => {
            return Err(format!("{DISCRIMINANT_CODE}: ok must be a JSON boolean"));
        }
    };
    let data = required(&map, "data")?;
    let error = required(&map, "error")?;
    if ok {
        if !error.is_null() {
            return Err(format!(
                "{DISCRIMINANT_CODE}: ok is true and error is not null"
            ));
        }
        let data = T::deserialize(data).map_err(|error| format!("{DISCRIMINANT_CODE}: {error}"))?;
        Ok(Envelope::Success { version, data })
    } else {
        if !data.is_null() {
            return Err(format!(
                "{DISCRIMINANT_CODE}: ok is false and data is not null"
            ));
        }
        if error.is_null() {
            return Err(format!(
                "{DISCRIMINANT_CODE}: ok is false and error is null"
            ));
        }
        let error =
            OpError::deserialize(error).map_err(|error| format!("{DISCRIMINANT_CODE}: {error}"))?;
        Ok(Envelope::Failure { version, error })
    }
}

fn required<'a>(map: &'a Map<String, Value>, key: &str) -> Result<&'a Value, String> {
    map.get(key)
        .ok_or_else(|| format!("{DISCRIMINANT_CODE}: missing {key}"))
}

fn require_exact_keys(map: &Map<String, Value>, keys: &[&str]) -> Result<(), String> {
    for key in map.keys() {
        if !keys.contains(&key.as_str()) {
            return Err(format!("{DISCRIMINANT_CODE}: unknown key {key}"));
        }
    }
    for key in keys {
        if !map.contains_key(*key) {
            return Err(format!("{DISCRIMINANT_CODE}: missing {key}"));
        }
    }
    Ok(())
}
