use std::path::PathBuf;

use sc_config::AppName;
use serde::Deserialize;

#[derive(Deserialize)]
struct Vectors {
    code: String,
    accept: Vec<String>,
    reject: Vec<String>,
}

#[test]
fn constructor_uses_the_shared_vectors() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/fixtures/app-name-vectors.json");
    let file = std::fs::read(&path).unwrap();
    let vectors: Vectors = serde_json::from_slice(&file).unwrap();

    for name in &vectors.accept {
        let parsed = AppName::new(name).unwrap_or_else(|error| panic!("{name}: {error}"));
        assert_eq!(parsed.as_str(), name);
        let as_ref: &str = parsed.as_ref();
        assert_eq!(as_ref, name);
        let from_try = AppName::try_from(name.as_str()).unwrap();
        assert_eq!(from_try.as_str(), name);
    }

    let dashed = AppName::new("my-app").unwrap();
    let underscored = AppName::new("my_app").unwrap();
    assert_eq!(dashed.env_prefix(), "MY_APP__");
    assert_eq!(underscored.env_prefix(), dashed.env_prefix());
    assert_eq!(AppName::new("My-App").unwrap().env_prefix(), "MY_APP__");

    for name in &vectors.reject {
        let error = AppName::new(name).unwrap_err();
        assert_eq!(error.code(), vectors.code);
        assert_eq!(error.code(), "APP.INVALID_NAME");
        assert!(error.to_string().contains(name));
    }
}
