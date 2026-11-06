//! Optional versioned cron DTOs and application-owned execution ports.
//!
//! Defaults contain no scheduler, database, server or async runtime.
#![forbid(unsafe_code)]

pub mod contract;
#[doc(inline)]
pub use contract::{
    CronExecutor, CronOperation, CronRequest, CronResponse, DotenvSource, EndRequest, EndResult,
    EnvironmentOperation, EnvironmentPolicy, ExecutionTarget, ExecutorFuture, ListRequest,
    ListResult, ScheduleState, ScheduleView, StartRequest, TargetKind,
};

#[cfg(feature = "scheduler")]
mod bindings {
    /// Application-authorized native hook and opaque persisted payload.
    pub struct ResolvedTarget {
        /// Existing core hook; no dynamic function registry.
        pub hook: sc_cron::HookTarget,
        /// Application-owned opaque input.
        pub payload: serde_json::Value,
    }

    /// Authorizes execution targets before any schedule operation.
    pub trait ApplicationTargetBindings: Send + Sync {
        /// Resolves an existing target and transforms its persisted payload.
        ///
        /// # Errors
        /// Returns a value-free typed error when authorization or capability fails.
        fn resolve(
            &self,
            target: &crate::ExecutionTarget,
            payload: serde_json::Value,
        ) -> Result<ResolvedTarget, sc_command::OpError>;
    }
}
#[cfg(feature = "scheduler")]
#[doc(inline)]
pub use bindings::{ApplicationTargetBindings, ResolvedTarget};
