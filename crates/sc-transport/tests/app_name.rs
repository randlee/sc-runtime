use std::path::Path;

use sc_transport::{AppName, TransportError};

#[derive(Debug, serde::Deserialize)]
struct NameVectors {
    code: String,
    accept: Vec<String>,
    reject: Vec<String>,
}

fn app_name_vectors() -> NameVectors {
    let path =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures/app-name-vectors.json");
    let text = std::fs::read_to_string(&path)
        .unwrap_or_else(|err| panic!("read {}: {err}", path.display()));
    serde_json::from_str(&text).unwrap_or_else(|err| panic!("parse {}: {err}", path.display()))
}

#[test]
fn vectors_accept_and_reject_at_construction() {
    let vectors = app_name_vectors();
    assert_eq!(vectors.code, "APP.INVALID_NAME");
    for name in &vectors.accept {
        let parsed = AppName::new(name).unwrap_or_else(|err| panic!("{name}: {err}"));
        assert_eq!(parsed.as_str(), name);
        assert_eq!(AsRef::<str>::as_ref(&parsed), name);
        let via_try = AppName::try_from(name.as_str()).unwrap_or_else(|err| panic!("{err}"));
        assert_eq!(via_try, parsed);
    }
    for name in &vectors.reject {
        let err = match AppName::new(name) {
            Ok(_) => panic!("reject vector must fail"),
            Err(err) => err,
        };
        assert_eq!(err.code(), vectors.code);
        assert_eq!(err.code(), sc_transport::CODE_INVALID_NAME);
        let rendered = err.to_string();
        assert!(
            rendered.contains("empty")
                || rendered.contains("maximum")
                || rendered.contains("contains"),
            "{rendered}"
        );
        match err {
            TransportError::InvalidName {
                code,
                name: rejected,
                ..
            } => {
                assert_eq!(code, "APP.INVALID_NAME");
                assert_eq!(rejected, *name);
            }
            other => panic!("unexpected {other:?}"),
        }
    }
}
