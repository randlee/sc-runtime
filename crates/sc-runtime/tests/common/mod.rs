use sc_runtime::DaemonConfig;
use sc_transport::{AppName, Endpoint};

#[allow(dead_code)]
pub fn app() -> AppName {
    AppName::new("sc-runtime-test").expect("app name")
}

#[allow(dead_code)]
pub fn config() -> DaemonConfig {
    #[cfg(windows)]
    {
        let mut config = DaemonConfig::default();
        config.endpoint_flag = Some(format!("tcp:{}", ephemeral_tcp()));
        return config;
    }
    #[cfg(not(windows))]
    {
        DaemonConfig::default()
    }
}

#[cfg(windows)]
fn ephemeral_tcp() -> std::net::SocketAddr {
    std::net::SocketAddr::from((std::net::Ipv4Addr::LOCALHOST, 0))
}

#[allow(dead_code)]
pub fn http(endpoint: &Endpoint) -> reqwest::Client {
    let mut builder = reqwest::Client::builder();
    #[cfg(unix)]
    if let Endpoint::Uds(path) = endpoint {
        builder = builder.unix_socket(path.clone());
    }
    #[cfg(not(unix))]
    let _ = endpoint;
    builder.build().expect("client")
}

#[allow(dead_code)]
pub fn url(endpoint: &Endpoint, path: &str) -> String {
    match endpoint {
        Endpoint::Tcp(addr) => format!("http://{addr}{path}"),
        Endpoint::Uds(_) => format!("http://localhost{path}"),
        _ => format!("http://localhost{path}"),
    }
}
