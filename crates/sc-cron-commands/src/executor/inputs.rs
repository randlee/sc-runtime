//! Conversion through the core's canonical validators before any native call.
use std::io::{self, Write};
use std::time::Duration;

use sc_command::OpError;
use sc_cron::{
    EndSchedule, Generation, HookEnvironment, HookTarget, ScheduleId, ScheduleName, ScheduleQuery,
    ScheduleSelection, ScheduleSpec,
};

use super::errors;
use crate::{
    ApplicationTargetBindings, CronRequest, EndRequest, EnvironmentOperation, EnvironmentPolicy,
    ExecutionTarget, ListRequest, ScheduleState, StartRequest,
};

struct Limit(usize);
impl Write for Limit {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        self.0 = self
            .0
            .checked_sub(bytes.len())
            .ok_or_else(|| io::Error::other("request size limit"))?;
        Ok(bytes.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

pub(super) fn request(request: &CronRequest) -> Result<(), OpError> {
    if request.version != "1" {
        return Err(errors::invalid("unsupported_version", "set version to 1"));
    }
    serde_json::to_writer(Limit(crate::contract::MAX_REQUEST_BYTES), request).map_err(|_| {
        errors::invalid(
            "request_size",
            "reduce the serialized request to at most 2 MiB",
        )
    })
}

pub(super) fn start(
    request: StartRequest,
    bindings: &(impl ApplicationTargetBindings + ?Sized),
) -> Result<ScheduleSpec, OpError> {
    validate_target(&request.target)?;
    let environment = environment(request.environment)?;
    // Validate the original opaque payload and all scalars before application dispatch.
    ScheduleSpec::new(
        &request.name,
        &request.cron,
        Duration::from_millis(request.timeout_ms),
        request.payload.clone(),
    )
    .map_err(errors::native)?;
    if matches!(request.target, ExecutionTarget::Rust) && !environment.is_empty() {
        return Err(errors::invalid(
            "rust_environment",
            "remove environment entries for a Rust target",
        ));
    }
    let resolved = bindings.resolve(&request.target, request.payload)?;
    validate_native_target(&resolved.hook)?;
    ScheduleSpec::with_hook(
        &request.name,
        &request.cron,
        Duration::from_millis(request.timeout_ms),
        resolved.payload,
        resolved.hook,
    )
    .and_then(|spec| spec.with_environment(environment))
    .map_err(errors::native)
}

fn validate_target(target: &ExecutionTarget) -> Result<(), OpError> {
    match target {
        ExecutionTarget::Rust => Ok(()),
        ExecutionTarget::Python { module } => HookTarget::python(module)
            .map(|_| ())
            .map_err(errors::native),
        ExecutionTarget::Monty { source } => {
            if !cfg!(feature = "monty") {
                return Err(errors::unsupported());
            }
            HookTarget::monty(source.clone())
                .map(|_| ())
                .map_err(errors::native)
        }
    }
}

fn validate_native_target(target: &HookTarget) -> Result<(), OpError> {
    match target {
        HookTarget::Rust => Ok(()),
        HookTarget::Python { module } => HookTarget::python(module)
            .map(|_| ())
            .map_err(errors::native),
        HookTarget::Monty { source } => {
            if !cfg!(feature = "monty") {
                return Err(errors::unsupported());
            }
            HookTarget::monty(source.clone())
                .map(|_| ())
                .map_err(errors::native)
        }
        _ => Err(errors::unsupported()),
    }
}

fn environment(policy: EnvironmentPolicy) -> Result<HookEnvironment, OpError> {
    let mut environment = HookEnvironment::new();
    for source in policy.dotenv {
        environment = environment
            .dotenv(source.path, source.required)
            .map_err(errors::native)?;
    }
    for (name, operation) in policy.variables {
        environment = match operation {
            EnvironmentOperation::Set(value) => environment.set(name, value),
            EnvironmentOperation::Reference(reference) => environment.reference(name, reference),
            EnvironmentOperation::Clear(true) => environment.clear(name),
            EnvironmentOperation::Clear(false) => {
                return Err(errors::invalid(
                    "environment_clear",
                    "set clear to true or omit the environment operation",
                ));
            }
        }
        .map_err(errors::native)?;
    }
    Ok(environment)
}

pub(super) fn end(request: EndRequest) -> Result<EndSchedule, OpError> {
    let generation = Generation::new(request.generation).ok_or_else(|| {
        errors::invalid(
            "generation",
            "supply the positive current schedule generation from list",
        )
    })?;
    if request.reason.is_empty()
        || request.reason.len() > sc_cron::MAX_PAYLOAD_BYTES
        || request.reason.contains('\0')
    {
        return Err(errors::invalid(
            "end_reason",
            "supply a nonempty NUL-free reason within the documented payload size limit",
        ));
    }
    Ok(EndSchedule {
        name: ScheduleName::new(request.name).map_err(errors::native)?,
        generation,
        reason: request.reason,
    })
}

pub(super) fn query(request: ListRequest) -> Result<ScheduleQuery, OpError> {
    if !(1..=1000).contains(&request.limit) {
        return Err(errors::invalid(
            "limit",
            "set limit to an integer from 1 through 1000",
        ));
    }
    if request.cursor == Some(0) {
        return Err(errors::invalid(
            "cursor",
            "omit cursor or use a positive next_cursor returned by list",
        ));
    }
    let selection = match request.state {
        ScheduleState::Active => ScheduleSelection::Active,
        ScheduleState::Ended => ScheduleSelection::Ended,
        ScheduleState::All => ScheduleSelection::All,
    };
    Ok(ScheduleQuery {
        selection,
        name: None,
        after_id: request.cursor.map(ScheduleId::new),
        limit: request.limit,
    })
}
