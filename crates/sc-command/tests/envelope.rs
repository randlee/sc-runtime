//! Default-feature envelope, error, and conversion tests.

use sc_command::{DISCRIMINANT_CODE, ENVELOPE_VERSION, Envelope, ErrorKind, OpError};
use sc_observability_types::{ErrorCode, Remediation};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct Sample {
    name: String,
}

fn sample() -> Sample {
    Sample {
        name: "sample".to_owned(),
    }
}

fn full_error() -> OpError {
    OpError::new(
        ErrorKind::Unavailable,
        ErrorCode::new_static("DAEMON.NOT_RUNNING"),
        "the daemon is not running",
    )
    .with_details(json!({"endpoint": "uds:app.sock"}))
    .with_suggestion("run my-app daemon start")
}

fn fixture(name: &str) -> Value {
    let text = match name {
        "success.json" => include_str!("fixtures/success.json"),
        "failure.json" => include_str!("fixtures/failure.json"),
        _ => panic!("unknown fixture {name}"),
    };
    serde_json::from_str(text).expect("fixture json")
}

fn assert_traits<T: std::fmt::Debug + Clone + PartialEq>() {}

fn round_trip_value<T>(value: &T) -> T
where
    T: Serialize + DeserializeOwned,
{
    let text = serde_json::to_string(value).expect("serialize");
    serde_json::from_str(&text).expect("deserialize")
}

#[test]
fn success_envelope_has_four_keys() {
    let envelope = Envelope::success(sample());
    let value = serde_json::to_value(&envelope).expect("serialize");
    let object = value.as_object().expect("object");
    let mut keys = object.keys().cloned().collect::<Vec<_>>();
    keys.sort();
    assert_eq!(keys, ["data", "error", "ok", "version"]);
    assert_eq!(value["ok"], true);
    assert_eq!(
        value["data"],
        serde_json::to_value(sample()).expect("sample")
    );
    assert!(value["error"].is_null());
}

#[test]
fn failure_envelope_has_four_keys() {
    let error = full_error();
    let envelope = Envelope::<Sample>::failure(error.clone());
    let value = serde_json::to_value(&envelope).expect("serialize");
    let object = value.as_object().expect("object");
    assert_eq!(object.len(), 4);
    assert_eq!(value["ok"], false);
    assert!(value["data"].is_null());
    assert_eq!(
        value["error"],
        serde_json::to_value(&error).expect("error json")
    );
}

#[test]
fn success_and_failure_share_version() {
    let success = serde_json::to_value(Envelope::success(sample())).expect("success");
    let failure = serde_json::to_value(Envelope::<Sample>::failure(full_error())).expect("failure");
    assert_eq!(success["version"], failure["version"]);
    assert_eq!(success["version"], ENVELOPE_VERSION);
}

#[test]
fn envelope_round_trips_success_and_failure() {
    let success = Envelope::success(sample());
    let failure = Envelope::<Sample>::failure(full_error());
    assert_eq!(round_trip_value(&success), success);
    assert_eq!(round_trip_value(&failure), failure);
}

#[test]
fn unit_success_keeps_null_data() {
    let envelope = Envelope::success(());
    let value = serde_json::to_value(&envelope).expect("serialize");
    assert_eq!(value["ok"], true);
    assert!(value["data"].is_null());
    assert_eq!(round_trip_value(&envelope), envelope);
    assert!(envelope.is_ok());
}

#[test]
fn option_payload_round_trips_some_none_and_failure() {
    let some = Envelope::success(Some(sample()));
    let none = Envelope::success(None::<Sample>);
    let failure = Envelope::<Option<Sample>>::failure(full_error());
    let none_json = serde_json::to_value(&none).expect("none");
    assert!(none_json["data"].is_null());
    assert_eq!(none_json["ok"], true);
    assert_eq!(round_trip_value(&some), some);
    assert_eq!(round_trip_value(&none), none);
    assert!(round_trip_value(&none).is_ok());
    assert_eq!(round_trip_value(&failure), failure);
}

#[test]
fn envelope_and_error_implement_debug_clone_partial_eq() {
    assert_traits::<Envelope<Sample>>();
    assert_traits::<OpError>();
    assert_traits::<ErrorKind>();
    fn assert_error<E: std::error::Error>() {}
    fn assert_display<T: std::fmt::Display>() {}
    assert_error::<OpError>();
    assert_display::<OpError>();
    assert_display::<ErrorKind>();
    assert_eq!(
        full_error().to_string(),
        "DAEMON.NOT_RUNNING: the daemon is not running"
    );
}

#[test]
fn op_error_serializes_five_fields_and_string_suggestion() {
    let error = full_error();
    let value = serde_json::to_value(&error).expect("serialize");
    let object = value.as_object().expect("object");
    let mut keys = object.keys().cloned().collect::<Vec<_>>();
    keys.sort();
    assert_eq!(
        keys,
        ["code", "details", "kind", "message", "suggested_action"]
    );
    assert_eq!(value["suggested_action"], "run my-app daemon start");
    assert!(value["suggested_action"].is_string());
    assert_eq!(value["code"], "DAEMON.NOT_RUNNING");
    assert!(value["message"].is_string());
    let code: ErrorCode = error.code.clone();
    assert_eq!(code.as_str(), "DAEMON.NOT_RUNNING");
}

#[test]
fn op_error_round_trips() {
    let error = full_error();
    assert_eq!(round_trip_value(&error), error);
}

#[test]
fn empty_details_and_suggestion_are_null() {
    let error = OpError::new(
        ErrorKind::Internal,
        ErrorCode::new_static("SAMPLE.FAILED"),
        "failed",
    );
    let value = serde_json::to_value(&error).expect("serialize");
    for key in value.as_object().expect("object").keys() {
        assert!(matches!(
            key.as_str(),
            "kind" | "code" | "message" | "details" | "suggested_action"
        ));
    }
    assert!(value["details"].is_null());
    assert!(value["suggested_action"].is_null());
    assert!(value.get("kind").is_some());
    assert!(value.get("code").is_some());
    assert!(value.get("message").is_some());
}

#[test]
fn tagged_remediation_object_is_not_the_wire_form() {
    let error = OpError::new(
        ErrorKind::Internal,
        ErrorCode::new_static("SAMPLE.FAILED"),
        "failed",
    )
    .with_details(json!({"endpoint": "uds:app.sock"}))
    .with_remediation(Remediation::not_recoverable("manual repair"));
    let value = serde_json::to_value(&error).expect("serialize");
    assert!(value["suggested_action"].is_null());
    assert_eq!(value["details"]["endpoint"], "uds:app.sock");
    assert_eq!(value["details"]["justification"], "manual repair");
    assert!(error.suggested_action().is_none());
    let decoded: OpError = serde_json::from_value(value.clone()).expect("decode");
    assert!(decoded.remediation.is_none());
    assert_eq!(
        decoded.details.as_ref().expect("details")["justification"],
        "manual repair"
    );
    assert_eq!(serde_json::to_value(&decoded).expect("again"), value);

    let bare = OpError::new(
        ErrorKind::Internal,
        ErrorCode::new_static("SAMPLE.FAILED"),
        "failed",
    )
    .with_remediation(Remediation::not_recoverable("manual repair"));
    let bare_value = serde_json::to_value(&bare).expect("serialize");
    assert!(bare_value["suggested_action"].is_null());
    assert_eq!(bare_value["details"]["justification"], "manual repair");

    let multi = OpError::new(
        ErrorKind::Conflict,
        ErrorCode::new_static("SAMPLE.CONFLICT"),
        "conflict",
    )
    .with_remediation(Remediation::recoverable(
        "retry the write",
        ["reload the record"],
    ));
    let multi_value = serde_json::to_value(&multi).expect("serialize");
    assert_eq!(multi_value["suggested_action"], "retry the write");
    assert_eq!(multi.suggested_action(), Some("retry the write"));

    let empty = OpError::new(
        ErrorKind::Internal,
        ErrorCode::new_static("SAMPLE.FAILED"),
        "failed",
    )
    .with_remediation(Remediation::recoverable("", std::iter::empty::<String>()));
    let empty_value = serde_json::to_value(&empty).expect("serialize");
    assert!(empty_value["suggested_action"].is_null());
    assert!(empty.suggested_action().is_none());
}

#[test]
fn suggested_action_object_is_rejected() {
    let mut value = serde_json::to_value(full_error()).expect("serialize");
    value["suggested_action"] = json!({"kind": "recoverable", "steps": ["nope"]});
    let error = serde_json::from_value::<OpError>(value).expect_err("object rejected");
    assert!(error.to_string().contains(DISCRIMINANT_CODE));
}

#[test]
fn result_conversion_preserves_payload_and_version() {
    let ok_envelope: Envelope<Sample> = Ok(sample()).into();
    let ok_json = serde_json::to_value(&ok_envelope).expect("ok");
    assert_eq!(ok_json["ok"], true);
    assert_eq!(ok_json["data"]["name"], "sample");
    assert!(ok_json["error"].is_null());

    let err_envelope: Envelope<Sample> = Err(full_error()).into();
    let err_json = serde_json::to_value(&err_envelope).expect("err");
    assert_eq!(err_json["ok"], false);
    assert!(err_json["data"].is_null());
    assert_eq!(
        err_json["error"],
        serde_json::to_value(full_error()).expect("error")
    );
    assert_eq!(ok_json["version"], err_json["version"]);
}

#[test]
fn into_result_returns_payload_or_error() {
    let success = Envelope::success(sample());
    assert_eq!(success.into_result(), Ok(sample()));
    let failure = Envelope::<Sample>::failure(full_error());
    assert_eq!(failure.into_result(), Err(full_error()));
}

#[test]
fn version_does_not_admit_unknown_keys() {
    let mut extra = fixture("success.json");
    extra["version"] = json!("2");
    extra["retry_after"] = json!(1);
    let error = serde_json::from_value::<Envelope<Value>>(extra).expect_err("unknown key");
    assert!(error.to_string().contains(DISCRIMINANT_CODE));
}

#[test]
fn result_conversion_has_no_serialize_bound() {
    struct NotSerialize;
    fn assert_from() {
        let _: Envelope<NotSerialize> = Ok(NotSerialize).into();
    }
    assert_from();
}

#[test]
fn success_fixture_round_trips_unknown_data_fields() {
    let parsed = fixture("success.json");
    let envelope: Envelope<Value> = serde_json::from_value(parsed.clone()).expect("decode");
    assert_eq!(serde_json::to_value(&envelope).expect("encode"), parsed);
    assert!(parsed["error"].is_null());
    assert!(parsed["data"]["meta"].is_object());
    assert!(parsed["data"]["tags"].is_array());
}

#[test]
fn failure_fixture_preserves_details_for_value_and_typed_payload() {
    let parsed = fixture("failure.json");
    let as_value: Envelope<Value> = serde_json::from_value(parsed.clone()).expect("value");
    assert_eq!(serde_json::to_value(&as_value).expect("encode"), parsed);
    assert!(parsed["data"].is_null());

    let mut typed_input = parsed.clone();
    typed_input["data"] = Value::Null;
    let as_sample: Envelope<Sample> = serde_json::from_value(typed_input.clone()).expect("sample");
    let encoded = serde_json::to_value(&as_sample).expect("encode");
    assert_eq!(encoded["error"]["details"], parsed["error"]["details"]);
    assert!(encoded["data"].is_null());
}

#[test]
fn received_version_is_preserved() {
    let mut parsed = fixture("success.json");
    parsed["version"] = json!("9");
    let envelope: Envelope<Value> = serde_json::from_value(parsed.clone()).expect("decode");
    assert_eq!(envelope.version(), "9");
    assert_ne!(envelope.version(), ENVELOPE_VERSION);
    assert_eq!(serde_json::to_value(&envelope).expect("encode"), parsed);
}

#[test]
fn contradictory_and_unknown_envelope_keys_are_rejected() {
    let mut success = fixture("success.json");
    success["error"] = json!({"kind": "internal"});
    let error = serde_json::from_value::<Envelope<Value>>(success).expect_err("contradictory");
    assert!(error.to_string().contains(DISCRIMINANT_CODE));

    let mut failure = fixture("failure.json");
    failure["data"] = json!({"name": "present"});
    assert!(serde_json::from_value::<Envelope<Value>>(failure).is_err());

    let mut missing = fixture("success.json");
    let object = missing.as_object_mut().expect("object");
    object.remove("error");
    assert!(serde_json::from_value::<Envelope<Value>>(missing).is_err());

    let mut extra = fixture("success.json");
    extra["extra"] = json!(1);
    let error = serde_json::from_value::<Envelope<Value>>(extra).expect_err("unknown key");
    assert!(error.to_string().contains(DISCRIMINANT_CODE));
}

#[test]
fn error_kind_table_is_exhaustive() {
    let expected = [
        (ErrorKind::InvalidInput, 400, true, "invalid_input"),
        (ErrorKind::NotFound, 404, true, "not_found"),
        (ErrorKind::Conflict, 409, true, "conflict"),
        (ErrorKind::Unavailable, 503, false, "unavailable"),
        (ErrorKind::Internal, 500, false, "internal"),
    ];
    assert_eq!(ErrorKind::all().len(), expected.len());
    for (kind, status, caller, wire) in expected {
        assert_eq!(kind.http_status_code(), status);
        assert_eq!(kind.is_caller_error(), caller);
        assert_eq!(kind.to_string(), wire);
        assert_eq!(
            serde_json::to_value(kind).expect("kind"),
            Value::String(wire.to_owned())
        );
        let range = if caller { 400..500 } else { 500..600 };
        assert!(range.contains(&status));
    }
}

#[test]
fn transport_strings_use_caller_suggestion_and_omit_placeholder() {
    let error = OpError::from_transport_strings(
        "DAEMON.NOT_RUNNING",
        "uds:app.sock",
        "connection refused",
        Some("run my-app daemon start"),
    );
    assert_eq!(error.kind, ErrorKind::Unavailable);
    assert_eq!(error.code.as_str(), "DAEMON.NOT_RUNNING");
    assert_eq!(error.suggested_action(), Some("run my-app daemon start"));
    let value = serde_json::to_value(&error).expect("serialize");
    assert_eq!(value["suggested_action"], "run my-app daemon start");
    assert_eq!(value["details"]["endpoint"], "uds:app.sock");
    assert_eq!(value["details"]["cause"], "connection refused");
    assert!(
        !serde_json::to_string(&error)
            .expect("text")
            .contains("<app>")
    );

    let unset = OpError::from_transport_strings(
        "DAEMON.NOT_RUNNING",
        "uds:app.sock",
        "connection refused",
        None,
    );
    assert!(unset.suggested_action().is_none());
    assert!(
        !serde_json::to_string(&unset)
            .expect("text")
            .contains("<app>")
    );

    let other = OpError::from_transport_strings(
        "TRANSPORT.TIMEOUT",
        "tcp:daemon",
        "timed out",
        Some("retry the connection"),
    );
    assert_eq!(other.kind, ErrorKind::Internal);
    assert_eq!(other.code.as_str(), "TRANSPORT.TIMEOUT");
    assert_eq!(other.suggested_action(), Some("retry the connection"));
    assert_eq!(
        other.message,
        "transport failure TRANSPORT.TIMEOUT at tcp:daemon: timed out"
    );
    let other_json = serde_json::to_value(&other).expect("serialize");
    assert_eq!(other_json["suggested_action"], "retry the connection");
    assert_eq!(other_json["details"]["endpoint"], "tcp:daemon");
    assert_eq!(other_json["details"]["cause"], "timed out");

    let blank = OpError::from_transport_strings(
        "TRANSPORT.TIMEOUT",
        "tcp:daemon",
        "timed out",
        Some("   "),
    );
    assert!(blank.suggested_action().is_none());
}
