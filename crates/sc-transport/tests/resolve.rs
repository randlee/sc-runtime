use std::ffi::OsStr;

#[cfg(windows)]
use std::net::Ipv4Addr;

use sc_transport::{
    AppName, Endpoint, TransportConfig, TransportError, instance_root, instance_root_from_base,
    resolve_endpoint,
};

fn app(name: &str) -> AppName {
    AppName::new(name).unwrap_or_else(|err| panic!("{err}"))
}

fn config() -> TransportConfig {
    TransportConfig::default()
}

#[test]
fn flag_beats_environment_and_config() {
    let mut cfg = config();
    cfg.endpoint = Some("tcp:127.0.0.1:3".to_owned());
    let endpoint = resolve_endpoint(
        &app("app-a"),
        Some(OsStr::new("tcp:127.0.0.1:1")),
        Some(OsStr::new("tcp:127.0.0.1:2")),
        &cfg,
    )
    .unwrap_or_else(|err| panic!("{err}"));
    assert_eq!(endpoint, "tcp:127.0.0.1:1".parse::<Endpoint>().unwrap());
}

#[test]
fn environment_beats_config() {
    let mut cfg = config();
    cfg.endpoint = Some("tcp:127.0.0.1:3".to_owned());
    let endpoint = resolve_endpoint(
        &app("app-a"),
        None,
        Some(OsStr::new("tcp:127.0.0.1:2")),
        &cfg,
    )
    .unwrap_or_else(|err| panic!("{err}"));
    assert_eq!(endpoint, "tcp:127.0.0.1:2".parse::<Endpoint>().unwrap());
}

#[test]
fn config_is_used_when_it_is_the_only_source() {
    let mut cfg = config();
    cfg.endpoint = Some("uds:configured.sock".to_owned());
    let endpoint =
        resolve_endpoint(&app("app-a"), None, None, &cfg).unwrap_or_else(|err| panic!("{err}"));
    assert_eq!(endpoint, Endpoint::Uds("configured.sock".into()));
}

#[cfg(unix)]
#[test]
fn explicit_instance_root_is_the_unix_default() {
    let dir = tempfile::tempdir().unwrap();
    let mut cfg = config();
    cfg.instance_root = Some(dir.path().to_path_buf());
    let endpoint =
        resolve_endpoint(&app("app-a"), None, None, &cfg).unwrap_or_else(|err| panic!("{err}"));
    assert_eq!(endpoint, Endpoint::Uds(dir.path().join("daemon.sock")));
}

#[cfg(windows)]
#[test]
fn windows_default_uses_configured_port() {
    let mut cfg = config();
    cfg.tcp_port = Some(4242);
    let endpoint =
        resolve_endpoint(&app("app-a"), None, None, &cfg).unwrap_or_else(|err| panic!("{err}"));
    let Endpoint::Tcp(addr) = endpoint else {
        panic!("expected tcp");
    };
    assert_eq!(addr.ip(), Ipv4Addr::LOCALHOST);
    assert_eq!(addr.port(), 4242);
}

#[test]
fn unparseable_flag_does_not_fall_through() {
    let mut cfg = config();
    cfg.endpoint = Some("tcp:127.0.0.1:3".to_owned());
    let err = resolve_endpoint(
        &app("app-a"),
        Some(OsStr::new("not-an-endpoint")),
        Some(OsStr::new("tcp:127.0.0.1:2")),
        &cfg,
    )
    .expect_err("bad flag");
    assert!(matches!(err, TransportError::InvalidEndpoint { .. }));
    assert_eq!(err.code(), "TRANSPORT.INVALID_ENDPOINT");
}

#[test]
fn empty_flag_does_not_fall_through() {
    let err = resolve_endpoint(
        &app("app-a"),
        Some(OsStr::new("")),
        Some(OsStr::new("tcp:127.0.0.1:2")),
        &config(),
    )
    .expect_err("empty flag");
    assert!(matches!(err, TransportError::InvalidEndpoint { .. }));
}

#[test]
fn endpoint_text_round_trips() {
    let dir = tempfile::tempdir().unwrap();
    let uds = Endpoint::Uds(dir.path().join("my sock"));
    let tcp: Endpoint = "tcp:127.0.0.1:9".parse().unwrap();
    let ipv6: Endpoint = "tcp:[::1]:9".parse().unwrap();
    for endpoint in [uds, tcp, ipv6] {
        let text = endpoint.to_string();
        let parsed: Endpoint = text.parse().unwrap();
        assert_eq!(parsed, endpoint);
    }
    assert_eq!(
        Endpoint::Uds(dir.path().join("my sock")).to_string(),
        format!("uds:{}", dir.path().join("my sock").display())
    );
    assert_eq!(
        "tcp:127.0.0.1:9".parse::<Endpoint>().unwrap().to_string(),
        "tcp:127.0.0.1:9"
    );
}

#[test]
fn default_instance_root_is_stable_and_app_specific() {
    let first = instance_root(&app("app-a")).unwrap_or_else(|err| panic!("{err}"));
    let second = instance_root(&app("app-a")).unwrap_or_else(|err| panic!("{err}"));
    let other = instance_root(&app("app-b")).unwrap_or_else(|err| panic!("{err}"));
    assert_eq!(first, second);
    assert_ne!(first, other);
}

#[test]
fn path_builder_uses_the_supplied_base() {
    let one = tempfile::tempdir().unwrap();
    let two = tempfile::tempdir().unwrap();
    let name = app("app-a");
    let first = instance_root_from_base(&name, Some(one.path())).unwrap();
    let second = instance_root_from_base(&name, Some(two.path())).unwrap();
    assert_ne!(first, second);
    assert!(first.starts_with(one.path()));
    assert!(second.starts_with(two.path()));
}

#[test]
fn absent_base_is_an_error() {
    let err = instance_root_from_base(&app("app-a"), None).expect_err("absent base");
    assert!(matches!(
        err,
        TransportError::InstanceRootUnavailable { .. }
    ));
    assert_eq!(err.code(), "TRANSPORT.INSTANCE_ROOT_UNAVAILABLE");
}

#[test]
fn config_type_deserializes() {
    fn assert_de<T: serde::de::DeserializeOwned>() {}
    assert_de::<TransportConfig>();
    let parsed: TransportConfig =
        serde_json::from_str(r#"{"tcp_port":9,"endpoint":"tcp:127.0.0.1:9"}"#).unwrap();
    assert_eq!(parsed.tcp_port, Some(9));
    assert_eq!(parsed.endpoint.as_deref(), Some("tcp:127.0.0.1:9"));
}
