//! Typed operation error carried in an [`Envelope`](crate::Envelope).

use std::fmt;

use sc_observability_types::{ErrorCode, RecoverableSteps, Remediation};
use serde::de::{self, DeserializeOwned};
use serde::ser::SerializeMap;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::Value;

use crate::DISCRIMINANT_CODE;
use crate::exact::{require_exact_keys, required};

/// Detail key that carries a not-recoverable justification on the wire.
const JUSTIFICATION_KEY: &str = "justification";

const ALL_KINDS: &[ErrorKind] = &[
    ErrorKind::InvalidInput,
    ErrorKind::NotFound,
    ErrorKind::Conflict,
    ErrorKind::Unavailable,
    ErrorKind::Internal,
];

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
///
/// The enum is [`non_exhaustive`](https://doc.rust-lang.org/reference/attributes/type_system.html#the-non_exhaustive-attribute)
/// so a later variant does not break a downstream `match`. The wire spellings
/// stay the five names above. A JSON string outside that set fails
/// deserialization. [`http_status_code`](Self::http_status_code) is the only
/// status table.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[non_exhaustive]
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
    /// Every variant this crate defines, in declaration order.
    #[must_use]
    pub const fn all() -> &'static [Self] {
        ALL_KINDS
    }

    /// HTTP status that belongs to this kind.
    ///
    /// This is the only kind-to-status table. The `server` conversion reads
    /// it through [`StatusCode::from_u16`](https://docs.rs/http/latest/http/status/struct.StatusCode.html#method.from_u16).
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

    const fn wire_name(self) -> &'static str {
        match self {
            Self::InvalidInput => "invalid_input",
            Self::NotFound => "not_found",
            Self::Conflict => "conflict",
            Self::Unavailable => "unavailable",
            Self::Internal => "internal",
        }
    }
}

impl fmt::Display for ErrorKind {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.wire_name())
    }
}

/// Operation error inside a response envelope.
///
/// `code` is [`ErrorCode`]. `remediation`, when set, is [`Remediation`].
/// The wire object always has the keys `kind`, `code`, `message`, `details`,
/// and `suggested_action`. `details` and `suggested_action` are JSON `null`
/// when absent. `suggested_action` is a JSON string when a recoverable
/// remediation has a non-empty first step, never the tagged `Remediation`
/// object.
///
/// A multi-step [`Remediation::Recoverable`] keeps that first step on the
/// wire. An empty first step writes JSON `null`. [`Remediation::NotRecoverable`]
/// writes JSON `null` and copies its justification into `details` under
/// `justification`, so an agent does not execute it as a recovery step.
/// Those two values do not round-trip as the same Rust variant. A single-step
/// recoverable remediation with a non-empty step does.
///
/// [`Display`] prints `{code}: {message}`. The type also implements
/// [`std::error::Error`].
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
    /// wire as a JSON string when it is non-empty.
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

    /// Wire text of `suggested_action` when remediation projects to a
    /// non-empty recovery step.
    #[must_use]
    pub fn suggested_action(&self) -> Option<&str> {
        self.remediation.as_ref().and_then(wire_suggestion)
    }

    /// Maps the plain strings a transport failure carries into this error.
    ///
    /// `code`, `endpoint`, and `cause` are the strings the transport error
    /// exposes. `suggested_action` is the caller's already-rendered recovery
    /// text. This crate does not depend on the transport crate, and it does
    /// not invent an application name or a daemon verb.
    ///
    /// The code `DAEMON.NOT_RUNNING` selects [`ErrorKind::Unavailable`] and
    /// the message `the daemon is not running`. Any other code selects
    /// [`ErrorKind::Internal`] and the message
    /// `transport failure {code} at {endpoint}: {cause}`. `endpoint` and
    /// `cause` are stored in `details` for every code. `None`, or text that
    /// is empty after trimming, leaves remediation unset.
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
    ///     Some("run my-app daemon start"),
    /// );
    /// assert_eq!(error.kind, ErrorKind::Unavailable);
    /// assert_eq!(error.code.as_str(), "DAEMON.NOT_RUNNING");
    /// assert_eq!(error.suggested_action(), Some("run my-app daemon start"));
    /// ```
    #[must_use]
    pub fn from_transport_strings(
        code: &str,
        endpoint: &str,
        cause: &str,
        suggested_action: Option<&str>,
    ) -> Self {
        let daemon_down = code == "DAEMON.NOT_RUNNING";
        let kind = if daemon_down {
            ErrorKind::Unavailable
        } else {
            ErrorKind::Internal
        };
        let message = if daemon_down {
            "the daemon is not running".to_owned()
        } else {
            format!("transport failure {code} at {endpoint}: {cause}")
        };
        Self {
            kind,
            code: ErrorCode::new_owned(code),
            message,
            details: Some(serde_json::json!({
                "endpoint": endpoint,
                "cause": cause,
            })),
            remediation: remediation_from_action(suggested_action),
        }
    }
}

impl fmt::Display for OpError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}: {}", self.code, self.message)
    }
}

impl std::error::Error for OpError {}

impl Serialize for OpError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map = serializer.serialize_map(Some(5))?;
        map.serialize_entry("kind", &self.kind)?;
        map.serialize_entry("code", &self.code)?;
        map.serialize_entry("message", &self.message)?;
        map.serialize_entry(
            "details",
            &details_json(self.details.as_ref(), self.remediation.as_ref()),
        )?;
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

fn remediation_from_action(suggested_action: Option<&str>) -> Option<Remediation> {
    let text = suggested_action?.trim();
    if text.is_empty() {
        None
    } else {
        Some(Remediation::recoverable(text, std::iter::empty::<String>()))
    }
}

fn wire_suggestion(remediation: &Remediation) -> Option<&str> {
    match remediation {
        Remediation::Recoverable { steps } => {
            let step = steps.first_step()?;
            if step.is_empty() { None } else { Some(step) }
        }
        Remediation::NotRecoverable { .. } => None,
    }
}

fn suggestion_json(remediation: Option<&Remediation>) -> Value {
    match remediation.and_then(wire_suggestion) {
        None => Value::Null,
        Some(text) => Value::String(text.to_owned()),
    }
}

fn details_json(details: Option<&Value>, remediation: Option<&Remediation>) -> Value {
    let Some(Remediation::NotRecoverable { justification }) = remediation else {
        return details.cloned().unwrap_or(Value::Null);
    };
    match details {
        None => serde_json::json!({ JUSTIFICATION_KEY: justification }),
        Some(Value::Object(map)) => {
            let mut map = map.clone();
            map.insert(
                JUSTIFICATION_KEY.to_owned(),
                Value::String(justification.clone()),
            );
            Value::Object(map)
        }
        Some(other) => serde_json::json!({
            "value": other,
            "justification": justification,
        }),
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
        Value::String(text) if text.is_empty() => Ok(None),
        Value::String(text) => Ok(Some(Remediation::Recoverable {
            steps: RecoverableSteps::first(text.clone()),
        })),
        _ => Err(format!(
            "{DISCRIMINANT_CODE}: suggested_action must be a JSON string or null"
        )),
    }
}

fn from_json_value<T>(value: &Value) -> Result<T, String>
where
    T: DeserializeOwned,
{
    T::deserialize(value).map_err(|error| format!("{DISCRIMINANT_CODE}: {error}"))
}
