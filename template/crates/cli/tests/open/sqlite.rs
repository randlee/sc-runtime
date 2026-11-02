async fn open_temp() -> Result<service::Stores, sc_runtime::RuntimeError> {
    let dir = tempfile::tempdir().expect("db dir");
    let stores = service::Stores::open(dir.path().join("widgets.db"))
        .await
        .map_err(daemon::map_store_open)?;
    std::mem::forget(dir);
    Ok(stores)
}
