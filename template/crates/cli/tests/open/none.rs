async fn open_temp() -> Result<service::Stores, sc_runtime::RuntimeError> {
    Ok(service::Stores::open().await)
}
