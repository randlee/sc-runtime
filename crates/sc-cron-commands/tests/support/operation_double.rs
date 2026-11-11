//! Canonical dev-only native operation double, packaged with adapter tests.
use sc_cron::{
    CronError, CronOperationFuture, CronOperations, EndSchedule, EndedSchedule, Schedule,
    SchedulePage, ScheduleQuery, ScheduleSpec, StoreOperationContext,
};
use std::collections::VecDeque;
use std::sync::Mutex;

/// Captured native call and its real child scope.
#[derive(Debug)]
pub enum OperationCall {
    /// Start invocation.
    Start(StoreOperationContext, ScheduleSpec),
    /// End invocation.
    End(StoreOperationContext, EndSchedule),
    /// Query invocation.
    Query(StoreOperationContext, ScheduleQuery),
}

/// Queued typed native replies and exact call arguments.
#[derive(Default)]
pub struct RecordingOperations {
    /// Calls in invocation order.
    pub calls: Mutex<Vec<OperationCall>>,
    /// Queued start replies.
    pub starts: Mutex<VecDeque<Result<Schedule, CronError>>>,
    /// Queued end replies.
    pub ends: Mutex<VecDeque<Result<EndedSchedule, CronError>>>,
    /// Queued query replies.
    pub queries: Mutex<VecDeque<Result<SchedulePage, CronError>>>,
}
impl CronOperations for RecordingOperations {
    fn start(
        &self,
        context: StoreOperationContext,
        spec: ScheduleSpec,
    ) -> CronOperationFuture<'_, Schedule> {
        self.calls
            .lock()
            .expect("calls")
            .push(OperationCall::Start(context, spec));
        let reply = self
            .starts
            .lock()
            .expect("starts")
            .pop_front()
            .expect("queued start reply");
        Box::pin(async move { reply })
    }
    fn end(
        &self,
        context: StoreOperationContext,
        request: EndSchedule,
    ) -> CronOperationFuture<'_, EndedSchedule> {
        self.calls
            .lock()
            .expect("calls")
            .push(OperationCall::End(context, request));
        let reply = self
            .ends
            .lock()
            .expect("ends")
            .pop_front()
            .expect("queued end reply");
        Box::pin(async move { reply })
    }
    fn query(
        &self,
        context: StoreOperationContext,
        query: ScheduleQuery,
    ) -> CronOperationFuture<'_, SchedulePage> {
        self.calls
            .lock()
            .expect("calls")
            .push(OperationCall::Query(context, query));
        let reply = self
            .queries
            .lock()
            .expect("queries")
            .pop_front()
            .expect("queued query reply");
        Box::pin(async move { reply })
    }
}
