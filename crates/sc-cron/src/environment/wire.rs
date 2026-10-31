//! Strict canonical wire form. Duplicate fields and variable names are errors.

use serde::de::{MapAccess, Visitor};
use serde::ser::SerializeMap;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use super::*;

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Source {
    path: String,
    required: bool,
}
#[derive(Serialize, Default)]
struct WireOperation {
    #[serde(skip_serializing_if = "Option::is_none")]
    set: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    reference: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    clear: Option<bool>,
}
impl<'de> Deserialize<'de> for WireOperation {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct OneOperation;
        impl<'de> Visitor<'de> for OneOperation {
            type Value = WireOperation;
            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str("one environment operation")
            }
            fn visit_map<M: MapAccess<'de>>(self, mut map: M) -> Result<Self::Value, M::Error> {
                let (key, value) = map
                    .next_entry::<String, serde_json::Value>()?
                    .ok_or_else(|| serde::de::Error::custom("missing environment operation"))?;
                if map.next_entry::<String, serde::de::IgnoredAny>()?.is_some() {
                    return Err(serde::de::Error::custom("multiple environment operations"));
                }
                let mut op = WireOperation::default();
                match (key.as_str(), value) {
                    ("set", serde_json::Value::String(value)) => op.set = Some(value),
                    ("reference", serde_json::Value::String(value)) => op.reference = Some(value),
                    ("clear", serde_json::Value::Bool(true)) => op.clear = Some(true),
                    _ => return Err(serde::de::Error::custom("invalid environment operation")),
                }
                Ok(op)
            }
        }
        deserializer.deserialize_map(OneOperation)
    }
}
struct Variables(BTreeMap<String, WireOperation>);
impl Serialize for Variables {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.0.serialize(serializer)
    }
}
impl<'de> Deserialize<'de> for Variables {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct Unique;
        impl<'de> Visitor<'de> for Unique {
            type Value = Variables;
            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str("unique environment names")
            }
            fn visit_map<M: MapAccess<'de>>(self, mut map: M) -> Result<Self::Value, M::Error> {
                let mut entries = BTreeMap::new();
                while let Some((key, value)) = map.next_entry::<String, WireOperation>()? {
                    if entries.len() >= MAX_ENV_ENTRIES || entries.insert(key, value).is_some() {
                        return Err(serde::de::Error::custom("invalid environment variables"));
                    }
                }
                Ok(Variables(entries))
            }
        }
        deserializer.deserialize_map(Unique)
    }
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Wire {
    dotenv: Vec<Source>,
    variables: Variables,
}

impl Serialize for HookEnvironment {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let sources: Vec<_> = self
            .sources
            .iter()
            .map(|s| Source {
                path: s.path.to_string_lossy().into_owned(),
                required: s.required,
            })
            .collect();
        let variables = Variables(
            self.variables
                .iter()
                .map(|(name, operation)| {
                    let wire = match operation {
                        Operation::Set(value) => WireOperation {
                            set: Some(value.clone()),
                            ..WireOperation::default()
                        },
                        Operation::Reference(reference) => WireOperation {
                            reference: Some(reference.clone()),
                            ..WireOperation::default()
                        },
                        Operation::Clear => WireOperation {
                            clear: Some(true),
                            ..WireOperation::default()
                        },
                    };
                    (name.0.clone(), wire)
                })
                .collect(),
        );
        let mut map = serializer.serialize_map(Some(2))?;
        map.serialize_entry("dotenv", &sources)?;
        map.serialize_entry("variables", &variables)?;
        map.end()
    }
}
impl<'de> Deserialize<'de> for HookEnvironment {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let wire = Wire::deserialize(deserializer)
            .map_err(|_| serde::de::Error::custom("invalid hook environment"))?;
        let build = || -> Result<Self, CronError> {
            let mut policy = Self::new();
            for source in wire.dotenv {
                policy = policy.dotenv(source.path, source.required)?;
            }
            for (name, operation) in wire.variables.0 {
                policy = match (operation.set, operation.reference, operation.clear) {
                    (Some(value), None, None) => policy.set(name, value)?,
                    (None, Some(reference), None) => policy.reference(name, reference)?,
                    (None, None, Some(true)) => policy.clear(name)?,
                    _ => return Err(invalid()),
                };
            }
            Ok(policy)
        };
        build().map_err(|_| serde::de::Error::custom("invalid hook environment"))
    }
}
impl HookEnvironment {
    pub(crate) fn encode(&self) -> Result<String, CronError> {
        serde_json::to_string(self).map_err(|_| invalid())
    }
    pub(crate) fn decode(document: &str) -> Result<Self, CronError> {
        if document.len() > MAX_ENV_DOCUMENT_BYTES {
            return Err(invalid());
        }
        serde_json::from_str(document).map_err(|_| invalid())
    }
}
