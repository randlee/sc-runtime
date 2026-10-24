# sc-runtime

`sc-runtime` 0.1.0 is published on crates.io under the MIT license. The repository is <https://github.com/randlee/sc-runtime>. It is published after `sc-config`, `sc-transport`, and `sc-command`.

`sc-runtime` assembles one daemon: the instance lock, the project's stores,
one router, and one listener. The application loads configuration before
`Daemon::builder`. This crate does not read a config file and does not depend
on `sc-config`.

`run` is available only after `stores` and `routes`. `mcp` is optional.
Inside `run` the order is lock, stores, routes and optional MCP, then bind.
SIGINT and SIGTERM shut the listener down and return `Ok(())`.

```rust,no_run
use sc_runtime::{Daemon, DaemonConfig};
use sc_transport::AppName;
use utoipa_axum::router::OpenApiRouter;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let app = AppName::new("my-app")?;
    let config = DaemonConfig::default();
    Daemon::builder(&app, &config)
        .stores(|| async { Ok(()) })
        .routes(|()| OpenApiRouter::new())
        .run()
        .await?;
    Ok(())
}
```

`GET /health` returns status 200 and an `sc_command::Envelope` success body.
`GET /openapi.json` returns the OpenAPI document from the project's router.
An rmcp service passed to `mcp` is mounted at `/mcp`.

The in-process fixture `sc_runtime::testing::DaemonFixture` is behind the
non-default feature `test-util`. A production build leaves that feature off.
