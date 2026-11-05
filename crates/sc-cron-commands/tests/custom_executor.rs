//! The default-feature replacement seam shares the canonical wire response.
use sc_command::Envelope;
use sc_cron_commands::{CronExecutor, CronRequest, CronResponse, EndResult, ExecutorFuture};
use std::sync::Arc;
use std::time::Instant;

struct ApplicationExecutor;
impl CronExecutor for ApplicationExecutor {
    fn execute(&self, _request: CronRequest, _deadline: Instant) -> ExecutorFuture<'_> {
        Box::pin(async {
            Envelope::success(CronResponse::Ended(EndResult {
                id: 7,
                generation: 2,
            }))
        })
    }
}

#[tokio::test(flavor = "current_thread")]
async fn default_only_application_executor_uses_the_same_json_protocol() {
    let request = serde_json::from_str::<CronRequest>(r#"{"version":"1","command":"cron.end","request":{"name":"nightly","generation":2,"reason":"operator"}}"#).unwrap();
    let executor: Arc<dyn CronExecutor> = Arc::new(ApplicationExecutor);
    let response = executor.execute(request, Instant::now()).await;
    assert_eq!(
        serde_json::to_value(response).unwrap(),
        serde_json::json!({
            "version":"1","ok":true,"data":{"result":"ended","data":{"id":7,"generation":2}},"error":null
        })
    );
}
