//! Widget documents shared by the service and its adapters.

use std::fmt;

use serde::{Deserialize, Serialize};
#[cfg(feature = "schema")]
use utoipa::ToSchema;

/// Maximum accepted widget-name length, in bytes.
pub const MAX_WIDGET_NAME_LEN: usize = 256;

/// Maximum accepted widget-id length, in bytes.
pub const MAX_WIDGET_ID_LEN: usize = 64;

/// REST path for `widget.create`.
pub const WIDGET_CREATE_PATH: &str = "/ops/widget.create";

/// REST path for `widget.get`.
pub const WIDGET_GET_PATH: &str = "/ops/widget.get";

/// REST path for `widget.delete`.
pub const WIDGET_DELETE_PATH: &str = "/ops/widget.delete";

/// REST path for `delete row`.
pub const DELETE_ROW_PATH: &str = "/ops/delete.row";

/// Body of `widget.create`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema, ToSchema))]
pub struct CreateWidget {
    /// Caller-supplied widget name. Blank names are rejected by the service.
    ///
    /// Names longer than [`MAX_WIDGET_NAME_LEN`] bytes are rejected at this
    /// type before the service runs.
    pub name: WidgetName,
}

/// Body of `widget.get`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema, ToSchema))]
pub struct GetWidget {
    /// Identifier returned by `widget.create`.
    pub id: WidgetId,
}

/// Body of `widget.delete`. It calls the same delete as [`DeleteRow`] on `widgets`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema, ToSchema))]
pub struct DeleteWidget {
    /// Identifier returned by `widget.create`.
    pub id: WidgetId,
}

/// Body of `delete row <table> <id>`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema, ToSchema))]
pub struct DeleteRow {
    /// Table name. The example store accepts `widgets`.
    pub table: String,
    /// Row id.
    pub id: String,
}

/// Row a delete command removed, or named when the store cannot remove it yet.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema, ToSchema))]
pub struct DeletedRow {
    /// Table the command addressed.
    pub table: String,
    /// Id the command addressed.
    pub id: String,
}

/// One stored widget.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(schemars::JsonSchema, ToSchema))]
pub struct Widget {
    /// Stable identifier assigned by the store.
    pub id: WidgetId,
    /// Name stored for this widget.
    pub name: WidgetName,
}

/// Validated widget name. At most [`MAX_WIDGET_NAME_LEN`] bytes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(into = "String")]
#[cfg_attr(
    feature = "schema",
    derive(schemars::JsonSchema, ToSchema),
    schemars(extend("maxLength" = 256)),
    schema(value_type = String, description = "At most 256 bytes")
)]
pub struct WidgetName(String);

impl WidgetName {
    /// Parses a widget name.
    ///
    /// # Errors
    ///
    /// Returns a static message when `raw` is longer than [`MAX_WIDGET_NAME_LEN`]
    /// bytes. Empty and whitespace-only names parse; the service rejects those.
    pub fn parse(raw: &str) -> Result<Self, &'static str> {
        if raw.len() > MAX_WIDGET_NAME_LEN {
            return Err("widget name is longer than 256 bytes");
        }
        Ok(Self(raw.to_owned()))
    }

    /// Returns the name text.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for WidgetName {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl TryFrom<String> for WidgetName {
    type Error = &'static str;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::parse(&value)
    }
}

impl From<WidgetName> for String {
    fn from(value: WidgetName) -> Self {
        value.0
    }
}

impl<'de> Deserialize<'de> for WidgetName {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let raw = String::deserialize(deserializer)?;
        Self::parse(&raw).map_err(serde::de::Error::custom)
    }
}

/// Validated widget identifier.
///
/// Ids are 1 to [`MAX_WIDGET_ID_LEN`] ASCII letters, digits, hyphens, or
/// underscores. The store assigns 32 hex characters.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(into = "String")]
#[cfg_attr(
    feature = "schema",
    derive(schemars::JsonSchema, ToSchema),
    schemars(extend("maxLength" = 64, "minLength" = 1)),
    schema(value_type = String, pattern = "^[A-Za-z0-9_-]{1,64}$")
)]
pub struct WidgetId(String);

impl WidgetId {
    /// Parses a widget id.
    ///
    /// # Errors
    ///
    /// Returns a static message when `raw` is empty, longer than
    /// [`MAX_WIDGET_ID_LEN`] bytes, or outside the id alphabet.
    pub fn parse(raw: &str) -> Result<Self, &'static str> {
        if raw.is_empty() || raw.len() > MAX_WIDGET_ID_LEN || !raw.bytes().all(is_id_byte) {
            return Err("widget id must be 1-64 ascii letters, digits, hyphens, or underscores");
        }
        Ok(Self(raw.to_owned()))
    }

    /// Returns the id text.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for WidgetId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl TryFrom<String> for WidgetId {
    type Error = &'static str;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::parse(&value)
    }
}

impl From<WidgetId> for String {
    fn from(value: WidgetId) -> Self {
        value.0
    }
}

impl<'de> Deserialize<'de> for WidgetId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let raw = String::deserialize(deserializer)?;
        Self::parse(&raw).map_err(serde::de::Error::custom)
    }
}

fn is_id_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || byte == b'-' || byte == b'_'
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn widget_id_rejects_oversize_blank_and_odd_bytes() {
        assert!(WidgetId::parse("").is_err());
        assert!(WidgetId::parse(&"a".repeat(MAX_WIDGET_ID_LEN + 1)).is_err());
        assert!(WidgetId::parse("has space").is_err());
        assert!(WidgetId::parse("missing-widget-id").is_ok());
        let parsed = WidgetId::parse("abc").expect("id");
        let encoded = serde_json::to_string(&parsed).expect("json");
        assert_eq!(encoded, "\"abc\"");
        assert!(serde_json::from_str::<WidgetId>(&format!("\"{}\"", "b".repeat(65))).is_err());
    }

    #[test]
    fn widget_name_rejects_oversize_and_keeps_blank() {
        assert!(WidgetName::parse(" ").is_ok());
        assert!(WidgetName::parse(&"n".repeat(MAX_WIDGET_NAME_LEN + 1)).is_err());
        assert!(serde_json::from_str::<WidgetName>(&format!("\"{}\"", "n".repeat(257))).is_err());
    }
}
