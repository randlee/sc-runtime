//! A downstream application uses only sc-cron paths for logging contracts.
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use sc_cron::observability::types::{ErrorContext, Remediation, error_codes};
use sc_cron::observability::{CanonicalLog, EventError, LogEvent, NoopCanonicalLog, ServiceName, SharedCanonicalLog};
use sc_cron::{
    Job, ManualClock, RunOutcome, ScheduleLimits, ScheduleSpec, ScheduleStore, Scheduler,
    SeqJitter, SqliteScheduleStore, StopDecision,
};

struct Receiver {
    service: ServiceName,
    events: Mutex<Vec<LogEvent>>,
    reject: bool,
}
impl CanonicalLog for Receiver {
    fn service_name(&self) -> &ServiceName {
        &self.service
    }

    fn emit(&self, event: LogEvent) -> Result<(), EventError> {
        self.events.lock().unwrap().push(event);
        if self.reject {
            return Err(EventError::Routing {
                context: Box::new(ErrorContext::new(
                    error_codes::DIAGNOSTIC_INVALID,
                    "consumer admission rejected",
                    Remediation::recoverable("Retry admission.", [] as [&str; 0]),
                )),
            });
        }
        Ok(())
    }
}

struct FailingJob;
#[async_trait::async_trait]
impl Job for FailingJob {
    async fn run(&self, _: &serde_json::Value, _: Instant) -> RunOutcome {
        RunOutcome::Failed("consumer failure detail".into())
    }
    async fn should_stop(&self, _: &serde_json::Value) -> StopDecision {
        StopDecision::Continue
    }
}

#[tokio::test]
async fn downstream_receiver_observes_real_events_and_rejection_does_not_change_runs() {
    for reject in [false, true] {
        let receiver = Arc::new(Receiver {
            service: ServiceName::new("consumer-owned").unwrap(),
            events: Mutex::new(Vec::new()),
            reject,
        });
        let logger: SharedCanonicalLog = receiver.clone();
        let store =
            Arc::new(SqliteScheduleStore::open_in_memory(ScheduleLimits::default()).unwrap());
        let clock = Arc::new(ManualClock::new("2026-10-19T00:00:00Z".parse().unwrap()));
        let scheduler = Scheduler::new_with_logger(
            store.clone(),
            clock.clone(),
            Arc::new(SeqJitter::default()),
            logger,
        );
        let row = scheduler
            .start(
                ScheduleSpec::new(
                    "consumer-job",
                    "0 * * * * *",
                    Duration::from_secs(20),
                    serde_json::json!({"secret": "not-in-event"}),
                )
                .unwrap(),
            )
            .unwrap();
        clock.advance(chrono::Duration::seconds(60));
        let runs = scheduler.run_due(Arc::new(FailingJob)).await.unwrap();
        assert_eq!(runs.len(), 1);
        let outcome = RunOutcome::Failed("consumer failure detail".into());
        assert_eq!(runs[0].outcome, outcome);
        assert_eq!(
            store.get(row.id).unwrap().unwrap().last_outcome,
            Some(outcome)
        );
        assert!(
            scheduler
                .run_due(Arc::new(FailingJob))
                .await
                .unwrap()
                .is_empty()
        );

        let events = receiver.events.lock().unwrap();
        assert_eq!(
            events.iter().map(|e| e.action.as_str()).collect::<Vec<_>>(),
            ["logger.attached", "due.launched", "due.finished"]
        );
        assert!(events.iter().all(|e| e.service == receiver.service));
        let launched = &events[1];
        let finished = &events[2];
        assert_eq!(finished.target.as_str(), "cron.scheduler");
        assert_eq!(finished.fields["schedule"], row.id.to_string());
        assert_eq!(finished.fields["error"], "consumer failure detail");
        assert_eq!(finished.outcome.as_ref().unwrap().as_str(), "failed");
        assert_eq!(
            finished.diagnostic.as_ref().unwrap().code.as_str(),
            "JOB_FAILED"
        );
        assert_eq!(
            finished.diagnostic.as_ref().unwrap().timestamp,
            finished.timestamp
        );
        assert!(finished.correlation_id.is_some());
        assert_eq!(finished.correlation_id, launched.correlation_id);
        assert!(
            !serde_json::to_string(&*events)
                .unwrap()
                .contains("not-in-event")
        );
    }
}

#[test]
fn downstream_default_receiver_is_available_through_the_facade() {
    let receiver: SharedCanonicalLog = Arc::new(NoopCanonicalLog::new(
        ServiceName::new("downstream-default").unwrap(),
    ));
    assert_eq!(receiver.service_name().as_str(), "downstream-default");
}
