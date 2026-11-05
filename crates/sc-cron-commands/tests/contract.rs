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
    assert_eq!(request.as_value()["type"], "object");
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

#[test]
fn debug_redacts_nested_request_source_payload_environment_and_references() {
    use sc_cron_commands::*;
    let secret = "CONFIDENTIAL_CANARY";
    let request: CronRequest = serde_json::from_value(serde_json::json!({
        "version":"1","command":"cron.start","request":{
            "name":"safe","cron":"* * * * *","timeout_ms":1000,
            "target":{"kind":"monty","source":secret},"payload":{"secret":secret},
            "environment":{"dotenv":[{"path":secret,"required":true}],
            "variables":{"SET":{"set":secret},"REF":{"reference":secret}}}
        }
    }))
    .unwrap();
    let nested = format!("{request:?}");
    assert!(!nested.contains(secret));
    if let CronOperation::Start(start) = request.operation {
        for debug in [
            format!("{:?}", start.target),
            format!("{:?}", start.environment),
            format!("{:?}", start.environment.dotenv[0]),
            format!("{:?}", start.environment.variables["SET"]),
            format!("{:?}", start.environment.variables["REF"]),
        ] {
            assert!(!debug.contains(secret));
        }
    } else {
        panic!("start request")
    }
}

#[cfg(feature = "schema")]
mod typed_projection {
    #[utoipa::path(get,path="/envelope",responses((status=200,body=sc_command::Envelope<serde_json::Value>)))]
    fn envelope() {}
    #[utoipa::path(get,path="/error",responses((status=400,body=sc_command::OpError)))]
    fn error() {}
    #[utoipa::path(get,path="/kind",responses((status=200,body=sc_command::ErrorKind)))]
    fn kind() {}
    #[utoipa::path(post,path="/cron",request_body=sc_cron_commands::CronRequest,responses((status=200)))]
    fn request() {}
    #[derive(utoipa::OpenApi)]
    #[openapi(paths(envelope, error, kind, request))]
    struct Api;
    #[test]
    fn real_typed_paths_use_canonical_owner_schemas() {
        use utoipa::OpenApi;
        envelope();
        error();
        kind();
        request();
        let document = serde_json::to_value(Api::openapi()).unwrap();
        let schemas = &document["components"]["schemas"];
        let reference=document["paths"]["/envelope"]["get"]["responses"]["200"]["content"]["application/json"]["schema"]["$ref"].as_str().unwrap();
        let name = reference.strip_prefix("#/components/schemas/").unwrap();
        assert!(schemas[name]["oneOf"].is_array());
        assert!(schemas["CronRequest"]["oneOf"].is_array());
        assert_eq!(schemas["OpError"]["additionalProperties"], false);
        assert!(schemas["ErrorKind"]["enum"].is_array());
    }
}
