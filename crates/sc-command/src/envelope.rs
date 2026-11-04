//! Response envelope `{version, ok, data, error}`.

use serde::de::{self, DeserializeOwned};
use serde::ser::SerializeMap;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::Value;

use crate::error::OpError;
use crate::exact::{require_exact_keys, required};
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

    /// Consumes the envelope and returns the success payload or the operation error.
    ///
    /// `version` is not part of the result. A caller that still needs the
    /// contract version reads [`version`](Self::version) before calling this
    /// method.
    ///
    /// # Errors
    ///
    /// Returns `Err` when the envelope is the failure case. The error is the
    /// [`OpError`] the envelope carried.
    ///
    /// # Examples
    ///
    /// ```
    /// use sc_command::Envelope;
    ///
    /// let envelope = Envelope::success("ready");
    /// assert_eq!(envelope.into_result(), Ok("ready"));
    /// ```
    pub fn into_result(self) -> Result<T, OpError> {
        match self {
            Self::Success { data, .. } => Ok(data),
            Self::Failure { error, .. } => Err(error),
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

#[cfg(feature = "schema")]
fn envelope_schema(data: Value, error: Value) -> Value {
    serde_json::json!({
        "oneOf": [
            {"type":"object", "additionalProperties":false,
             "required":["version","ok","data","error"],
             "properties":{"version":{"type":"string"},"ok":{"type":"boolean","enum":[true]},"data":data,"error":{"type":"null"}}},
            {"type":"object", "additionalProperties":false,
             "required":["version","ok","data","error"],
             "properties":{"version":{"type":"string"},"ok":{"type":"boolean","enum":[false]},"data":{"type":"null"},"error":error}}
        ]
    })
}

#[cfg(feature = "schema")]
impl<T: schemars::JsonSchema> schemars::JsonSchema for Envelope<T> {
    fn schema_name() -> std::borrow::Cow<'static, str> {
        format!("Envelope_{}", T::schema_name()).into()
    }

    fn json_schema(generator: &mut schemars::SchemaGenerator) -> schemars::Schema {
        let value = envelope_schema(
            generator.subschema_for::<T>().to_value(),
            generator.subschema_for::<OpError>().to_value(),
        );
        serde_json::from_value(value).expect("canonical Envelope schema is an object")
    }
}

#[cfg(feature = "schema")]
impl<T: utoipa::ToSchema> utoipa::PartialSchema for Envelope<T> {
    fn schema() -> utoipa::openapi::RefOr<utoipa::openapi::schema::Schema> {
        use utoipa::openapi::schema::{AdditionalProperties, ObjectBuilder, OneOfBuilder, Type};
        let object = || {
            ObjectBuilder::new()
                .schema_type(Type::Object)
                .additional_properties(Some(AdditionalProperties::FreeForm(false)))
                .required("version")
                .required("ok")
                .required("data")
                .required("error")
                .property("version", <String as utoipa::PartialSchema>::schema())
        };
        let success = object()
            .property(
                "ok",
                ObjectBuilder::new()
                    .schema_type(Type::Boolean)
                    .enum_values(Some([true])),
            )
            .property("data", T::schema())
            .property("error", ObjectBuilder::new().schema_type(Type::Null));
        let failure = object()
            .property(
                "ok",
                ObjectBuilder::new()
                    .schema_type(Type::Boolean)
                    .enum_values(Some([false])),
            )
            .property("data", ObjectBuilder::new().schema_type(Type::Null))
            .property("error", <OpError as utoipa::PartialSchema>::schema());
        OneOfBuilder::new().item(success).item(failure).into()
    }
}

#[cfg(feature = "schema")]
impl<T: utoipa::ToSchema> utoipa::ToSchema for Envelope<T> {
    fn name() -> std::borrow::Cow<'static, str> {
        format!("Envelope_{}", T::name()).into()
    }

    fn schemas(
        schemas: &mut Vec<(
            String,
            utoipa::openapi::RefOr<utoipa::openapi::schema::Schema>,
        )>,
    ) {
        schemas.push((T::name().into_owned(), T::schema()));
        T::schemas(schemas);
    }
}
