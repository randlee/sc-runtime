//! Typed operation error carried in an [`Envelope`](crate::Envelope).

use sc_observability_types::{ErrorCode, RecoverableSteps, Remediation};
use serde::de::{self, DeserializeOwned};
use serde::ser::SerializeMap;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::{Map, Value};

use crate::DISCRIMINANT_CODE;

/// Coarse class of an [`OpError`]. The HTTP status is a pure function of this
/// class.
///
/// | Variant | Wire string | HTTP status |
/// |---|---|---|
/// | [`InvalidInput`](Self::InvalidInput) | `invalid_input` | 400 |
/// | [`NotFound`](Self::NotFound) | `not_found` | 404 |
/// | [`Conflict`](Self::Conflict) | `conflict` | 409 |
/// | [`Unavailable`](Self::Unavailable) | `unavailable` | 503 |
/// | [`Internal`](Self::Internal) | `internal` | 500 |
///
/// `invalid_input`, `not_found`, and `conflict` are caller errors (4xx).
/// `unavailable` and `internal` are internal errors (5xx).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ErrorKind {
    /// The caller sent a value the operation cannot accept.
    InvalidInput,
    /// The named resource does not exist.
    NotFound,
    /// The operation conflicts with the current state.
    Conflict,
    /// A dependency the operation needs is not available.
    Unavailable,
    /// The operation failed for a reason the caller cannot fix.
    Internal,
}

impl ErrorKind {
    /// Every variant, in declaration order.
    ///
    /// Adding a variant without extending this array fails to compile.
    #[must_use]
    pub const fn all() -> [Self; 5] {
        [
            Self::InvalidInput,
            Self::NotFound,
            Self::Conflict,
            Self::Unavailable,
            Self::Internal,
        ]
    }

    /// HTTP status that belongs to this kind.
    #[must_use]
    pub const fn http_status_code(self) -> u16 {
        match self {
            Self::InvalidInput => 400,
            Self::NotFound => 404,
            Self::Conflict => 409,
            Self::Unavailable => 503,
            Self::Internal => 500,
        }
    }

    /// Returns whether this kind maps to a 4xx status.
    #[must_use]
    pub const fn is_caller_error(self) -> bool {
        matches!(self, Self::InvalidInput | Self::NotFound | Self::Conflict)
    }

    /// Stable inventory code for this kind's HTTP mapping.
    ///
    /// This is the library's name for the mapping. It is not the
    /// [`OpError::code`] a service chooses for a particular failure.
    #[must_use]
    pub const fn mapping_code(self) -> &'static str {
        match self {
            Self::InvalidInput => "ENVELOPE.KIND.INVALID_INPUT",
            Self::NotFound => "ENVELOPE.KIND.NOT_FOUND",
            Self::Conflict => "ENVELOPE.KIND.CONFLICT",
            Self::Unavailable => "ENVELOPE.KIND.UNAVAILABLE",
            Self::Internal => "ENVELOPE.KIND.INTERNAL",
        }
    }
}

/// Operation error inside a response envelope.
///
/// `code` is [`ErrorCode`]. `remediation`, when set, is [`Remediation`].
/// The wire object always has the keys `kind`, `code`, `message`, `details`,
/// and `suggested_action`. `details` and `suggested_action` are JSON `null`
/// when absent. `suggested_action` is a JSON string when remediation is
/// present, never the tagged `Remediation` object.
///
/// The string form keeps the first recovery step, or the not-recoverable
/// justification. A multi-step [`Remediation`] or a [`Remediation::NotRecoverable`]
/// value therefore does not round-trip as the same Rust variant. A single-step
/// recoverable remediation does.
#[derive(Debug, Clone, PartialEq)]
pub struct OpError {
    /// Coarse class. Selects the HTTP status.
    pub kind: ErrorKind,
    /// Stable machine-readable code.
    pub code: ErrorCode,
    /// Human-readable summary.
    pub message: String,
    /// Structured context. `None` is JSON `null` on the wire.
    pub details: Option<Value>,
    /// Structured remediation. `None` is JSON `null` under `suggested_action`.
    pub remediation: Option<Remediation>,
}

impl OpError {
    /// Builds an error with no details and no remediation.
    #[must_use]
    pub fn new(kind: ErrorKind, code: ErrorCode, message: impl Into<String>) -> Self {
        Self {
            kind,
            code,
            message: message.into(),
            details: None,
            remediation: None,
        }
    }

    /// Attaches structured details.
    #[must_use]
    pub fn with_details(mut self, details: Value) -> Self {
        self.details = Some(details);
        self
    }

    /// Attaches one recoverable suggestion.
    ///
    /// The text is stored as [`Remediation::recoverable`] and written on the
    /// wire as a JSON string.
    #[must_use]
    pub fn with_suggestion(mut self, suggestion: impl Into<String>) -> Self {
        self.remediation = Some(Remediation::recoverable(
            suggestion,
            std::iter::empty::<String>(),
        ));
        self
    }

    /// Attaches remediation from `sc-observability-types`.
    #[must_use]
    pub fn with_remediation(mut self, remediation: Remediation) -> Self {
        self.remediation = Some(remediation);
        self
    }

    /// Wire text of `suggested_action`, when remediation is present.
    #[must_use]
    pub fn suggested_action(&self) -> Option<&str> {
        self.remediation.as_ref().map(suggestion_text)
    }

    /// Maps the plain strings a transport failure carries into this error.
    ///
    /// `code`, `endpoint`, and `cause` are the strings the transport error
    /// exposes. This crate does not depend on the transport crate. The code
    /// `DAEMON.NOT_RUNNING` selects [`ErrorKind::Unavailable`] and the
    /// suggestion `run <app> daemon start`. Any other code selects
    /// [`ErrorKind::Internal`] and no suggestion. `endpoint` and `cause` are
    /// stored in `details`.
    ///
    /// # Examples
    ///
    /// ```
    /// use sc_command::{ErrorKind, OpError};
    ///
    /// let error = OpError::from_transport_strings(
    ///     "DAEMON.NOT_RUNNING",
    ///     "uds:app.sock",
    ///     "connection refused",
    /// );
    /// assert_eq!(error.kind, ErrorKind::Unavailable);
    /// assert_eq!(error.code.as_str(), "DAEMON.NOT_RUNNING");
    /// assert_eq!(error.suggested_action(), Some("run <app> daemon start"));
    /// ```
    #[must_use]
    pub fn from_transport_strings(code: &str, endpoint: &str, cause: &str) -> Self {
        let daemon_down = code == "DAEMON.NOT_RUNNING";
        let kind = if daemon_down {
            ErrorKind::Unavailable
        } else {
            ErrorKind::Internal
        };
        let message = if daemon_down {
            "the daemon is not running".to_owned()
        } else {
            format!("transport failure {code}")
        };
        let remediation = if daemon_down {
            Some(Remediation::recoverable(
                "run <app> daemon start",
                std::iter::empty::<String>(),
            ))
        } else {
            None
        };
        Self {
            kind,
            code: ErrorCode::new_owned(code),
            message,
            details: Some(serde_json::json!({
                "endpoint": endpoint,
                "cause": cause,
            })),
            remediation,
        }
    }
}

impl Serialize for OpError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map = serializer.serialize_map(Some(5))?;
        map.serialize_entry("kind", &self.kind)?;
        map.serialize_entry("code", &self.code)?;
        map.serialize_entry("message", &self.message)?;
        map.serialize_entry("details", &details_json(self.details.as_ref()))?;
        map.serialize_entry(
            "suggested_action",
            &suggestion_json(self.remediation.as_ref()),
        )?;
        map.end()
    }
}

impl<'de> Deserialize<'de> for OpError {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = Value::deserialize(deserializer)?;
        op_error_from_value(value).map_err(de::Error::custom)
    }
}

fn suggestion_text(remediation: &Remediation) -> &str {
    match remediation {
        Remediation::Recoverable { steps } => steps.first_step().unwrap_or(""),
        Remediation::NotRecoverable { justification } => justification.as_str(),
    }
}

fn suggestion_json(remediation: Option<&Remediation>) -> Value {
    match remediation {
        None => Value::Null,
        Some(remediation) => Value::String(suggestion_text(remediation).to_owned()),
    }
}

fn details_json(details: Option<&Value>) -> Value {
    match details {
        None => Value::Null,
        Some(details) => details.clone(),
    }
}

fn op_error_from_value(value: Value) -> Result<OpError, String> {
    let map = match value {
        Value::Object(map) => map,
        _ => {
            return Err(format!("{DISCRIMINANT_CODE}: error must be a JSON object"));
        }
    };
    require_exact_keys(
        &map,
        &["kind", "code", "message", "details", "suggested_action"],
    )?;
    let kind = from_json_value::<ErrorKind>(required(&map, "kind")?)?;
    let code = from_json_value::<ErrorCode>(required(&map, "code")?)?;
    let message = match required(&map, "message")? {
        Value::String(message) => message.clone(),
        _ => {
            return Err(format!(
                "{DISCRIMINANT_CODE}: message must be a JSON string"
            ));
        }
    };
    let details = match required(&map, "details")? {
        Value::Null => None,
        other => Some(other.clone()),
    };
    let remediation = remediation_from_wire(required(&map, "suggested_action")?)?;
    Ok(OpError {
        kind,
        code,
        message,
        details,
        remediation,
    })
}

fn remediation_from_wire(value: &Value) -> Result<Option<Remediation>, String> {
    match value {
        Value::Null => Ok(None),
        Value::String(text) => Ok(Some(Remediation::Recoverable {
            steps: RecoverableSteps::first(text.clone()),
        })),
        _ => Err(format!(
            "{DISCRIMINANT_CODE}: suggested_action must be a JSON string or null"
        )),
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

fn from_json_value<T>(value: &Value) -> Result<T, String>
where
    T: DeserializeOwned,
{
    T::deserialize(value).map_err(|error| format!("{DISCRIMINANT_CODE}: {error}"))
}
