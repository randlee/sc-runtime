# sc-transport

`sc-transport` 0.1.0 is published on crates.io under the MIT license. The repository is <https://github.com/randlee/sc-runtime>.

`sc-transport` resolves local daemon endpoints, sends JSON `GET` and `POST`
requests with reqwest, and optionally binds UDS/TCP listeners when built with
the `server` feature. Failures are `TransportError` values; the crate does not
start daemons or decode application envelopes.

```rust,no_run
use sc_transport::{resolve_endpoint, AppName, Endpoint, TransportConfig};

fn main() -> Result<(), sc_transport::TransportError> {
    let app = AppName::new("my-app")?;
    let mut config = TransportConfig::default();
    config.instance_root = Some(std::env::temp_dir());
    let endpoint = resolve_endpoint(
        &app,
        Some(std::ffi::OsStr::new("tcp:127.0.0.1:9")),
        None,
        &config,
    )?;
    let _endpoint: Endpoint = endpoint;
    Ok(())
}
```

Use [`Client`](src/client.rs) for async calls after you have an [`Endpoint`].
Enable `server` for [`bind`](src/server.rs) when wiring `axum::serve`.
