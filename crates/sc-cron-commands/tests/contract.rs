//! Wire and optional schema declaration checks.
use sc_cron_commands::CronRequest;
#[cfg(feature = "schema")]
use sc_cron_commands::CronResponse;

#[path = "support/operation_double.rs"]
#[allow(dead_code)]
mod operations;

#[test]
fn strict_request_preserves_the_versioned_wire_shape() {
    let source = r#"{"version":"1","command":"cron.end","request":{"name":"nightly","generation":1,"reason":"operator stopped"}}"#;
    let parsed: CronRequest = serde_json::from_str(source).expect("valid request");
    assert_eq!(
        serde_json::to_value(parsed).expect("serialize"),
        serde_json::from_str::<serde_json::Value>(source).expect("json")
    );
    for invalid in [
        source.replace("\"version\":\"1\"", "\"version\":\"2\""),
        source.replace("\"version\":\"1\"", "\"version\":\"1\",\"version\":\"1\""),
        source.replace("\"generation\":1", "\"generation\":1,\"generation\":1"),
        source.replace("\"generation\":1", "\"generation\":4294967296"),
        source.replace("\"name\":", "\"extra\":true,\"name\":"),
        source.replace("\"version\":", "\"extra\":true,\"version\":"),
    ] {
        assert!(serde_json::from_str::<CronRequest>(&invalid).is_err());
    }
}

#[cfg(feature = "schema")]
#[test]
fn owner_schemas_cover_real_request_and_envelope_types() {
    let request = schemars::schema_for!(CronRequest);
    let envelope = schemars::schema_for!(sc_command::Envelope<CronResponse>);
    let error = schemars::schema_for!(sc_command::OpError);
    assert!(request.as_value().to_string().contains("cron.start"));
    for alternative in request.as_value()["oneOf"]
        .as_array()
        .expect("command alternatives")
    {
        assert_eq!(alternative["additionalProperties"], false);
        assert_eq!(
            alternative["properties"]["version"]["enum"],
            serde_json::json!(["1"])
        );
        assert_eq!(
            alternative["required"].as_array().expect("required").len(),
            3
        );
    }
    let alternatives = envelope.as_value()["oneOf"]
        .as_array()
        .expect("wire alternatives");
    assert_eq!(alternatives.len(), 2);
    assert_eq!(
        alternatives[0]["properties"]["ok"]["enum"],
        serde_json::json!([true])
    );
    assert_eq!(alternatives[1]["properties"]["data"]["type"], "null");
    assert_eq!(
        error.as_value()["required"]
            .as_array()
            .expect("fields")
            .len(),
        5
    );
    let schema = <sc_command::Envelope<CronResponse> as utoipa::PartialSchema>::schema();
    assert_eq!(
        serde_json::to_value(schema).expect("OpenAPI schema")["oneOf"]
            .as_array()
            .expect("wire alternatives")
            .len(),
        2
    );
    let _ = <CronRequest as utoipa::PartialSchema>::schema();
}

#[test]
fn environment_wire_matches_core_and_rejects_duplicate_names() {
    let source = r#"{"dotenv":[],"variables":{"TOKEN":{"reference":"secrets/token"},"MODE":{"set":"batch"},"OLD":{"clear":true}}}"#;
    let policy: sc_cron_commands::EnvironmentPolicy =
        serde_json::from_str(source).expect("adapter policy");
    let core: sc_cron::HookEnvironment = serde_json::from_str(source).expect("core policy");
    assert_eq!(
        serde_json::to_value(policy).expect("adapter json"),
        serde_json::to_value(core).expect("core json")
    );
    let duplicate = r#"{"dotenv":[],"variables":{"MODE":{"set":"a"},"MODE":{"set":"b"}}}"#;
    assert!(serde_json::from_str::<sc_cron_commands::EnvironmentPolicy>(duplicate).is_err());
}

#[cfg(feature = "schema")]
#[test]
fn canonical_envelope_schema_accepts_arbitrary_json_payload_types() {
    let schema = <sc_command::Envelope<serde_json::Value> as utoipa::PartialSchema>::schema();
    let json = serde_json::to_value(schema).expect("generic envelope schema");
    assert_eq!(
        json["oneOf"][0]["properties"]["data"],
        serde_json::json!({})
    );
    assert_eq!(json["oneOf"][1]["properties"]["data"]["type"], "null");
}

#[cfg(feature = "schema")]
#[test]
fn canonical_request_schema_has_mcp_object_root_and_typed_openapi_variants() {
    let json = schemars::schema_for!(CronRequest).to_value();
    assert_eq!(json["type"], "object");
    assert_eq!(json["oneOf"].as_array().expect("typed variants").len(), 3);
    let openapi = serde_json::to_value(<CronRequest as utoipa::PartialSchema>::schema())
        .expect("OpenAPI schema");
    assert_eq!(
        openapi["oneOf"].as_array().expect("typed variants").len(),
        3
    );
    for (variant, command) in openapi["oneOf"].as_array().expect("variants").iter().zip([
        "cron.start",
        "cron.end",
        "cron.list",
    ]) {
        assert_eq!(
            variant["properties"]["command"]["enum"],
            serde_json::json!([command])
        );
        assert!(variant["properties"]["request"]["$ref"].is_string());
    }
}
