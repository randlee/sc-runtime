//! Intentionally fails: runtime0.2 accepts the Utoipa6 router identity.
pub fn former_router() {
    let app = sc_transport::AppName::new("former-router").expect("valid application name");
    let _ = sc_runtime::Daemon::builder(&app, &sc_runtime::DaemonConfig::default())
        .stores(|| async { Ok(()) })
        .routes(|()| utoipa_axum::router::OpenApiRouter::new());
}
