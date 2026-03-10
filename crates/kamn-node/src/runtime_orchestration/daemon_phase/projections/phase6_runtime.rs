mod builders;
mod fixtures;

use super::super::super::*;
use builders::{build_phase6_request, build_phase6_runtime};
use fixtures::build_phase6_registries;

pub(crate) const fn phase6_runtime_reason_taxonomy_version() -> &'static str {
    "kamn.runtime.daemon.phase6.reason-taxonomy.v1"
}

pub(crate) const fn phase6_runtime_reason_codes_csv() -> &'static str {
    "m10_phase6_scheduler_cycle_applied,m10_phase6_scheduler_cycle_deferred,m10_phase6_scheduler_signal_invalid,m10_phase6_execution_budget_due_candidates_exceeded"
}

pub(crate) struct DaemonPhase6RuntimeProjection {
    pub(crate) reason_code: &'static str,
    pub(crate) total_cycles: u64,
    pub(crate) executed_cycles: u64,
    pub(crate) deferred_cycles: u64,
    pub(crate) fail_closed_cycles: u64,
}

pub(crate) fn execute_daemon_phase6_runtime_projection(
    _max_ticks: u64,
    tick_interval_ms: u64,
    has_shutdown_signal: bool,
    regressed_now_epoch_seconds: Option<u64>,
) -> Result<DaemonPhase6RuntimeProjection, ConfigError> {
    let mut context = build_phase6_runtime_context(
        "kamn:did:owner:daemon-phase6",
        tick_interval_ms,
        has_shutdown_signal,
    )?;
    let reason_code = project_phase6_reason_code(&mut context, regressed_now_epoch_seconds)?;
    Ok(build_phase6_runtime_projection(
        reason_code,
        context.runtime.state(),
    ))
}

struct Phase6RuntimeContext {
    m8_registry: kamn_core::DataLayerM8ComplianceRegistry,
    m10_registry: kamn_core::DataLayerM10PartitionLifecycleRegistry,
    runtime: kamn_core::DataLayerM10Phase6SchedulerRuntime,
    base_request: kamn_core::DataLayerM10Phase6ExecutionTickRequest,
}

fn build_phase6_runtime_context(
    owner_did: &str,
    tick_interval_ms: u64,
    has_shutdown_signal: bool,
) -> Result<Phase6RuntimeContext, ConfigError> {
    let (m8_registry, m10_registry, partition_message_ids_by_month) =
        build_phase6_registries(owner_did, has_shutdown_signal)?;
    Ok(Phase6RuntimeContext {
        m8_registry,
        m10_registry,
        runtime: build_phase6_runtime(has_shutdown_signal)?,
        base_request: build_phase6_request(
            owner_did,
            tick_interval_ms,
            has_shutdown_signal,
            partition_message_ids_by_month,
        ),
    })
}

fn project_phase6_reason_code(
    context: &mut Phase6RuntimeContext,
    regressed_now_epoch_seconds: Option<u64>,
) -> Result<&'static str, ConfigError> {
    let mut reason_code = run_base_phase6_cycle(context)?;
    if let Some(regressed_now_epoch_seconds) = regressed_now_epoch_seconds {
        reason_code = run_regressed_phase6_cycle(context, regressed_now_epoch_seconds);
    }
    Ok(reason_code)
}

fn run_base_phase6_cycle(context: &mut Phase6RuntimeContext) -> Result<&'static str, ConfigError> {
    context
        .runtime
        .run_cycle(
            &mut context.m8_registry,
            &mut context.m10_registry,
            context.base_request.clone(),
        )
        .map(|report| report.reason_code)
        .map_err(|error| ConfigError::RuntimeDaemonLifecycle(error.to_string()))
}

fn run_regressed_phase6_cycle(
    context: &mut Phase6RuntimeContext,
    regressed_now_epoch_seconds: u64,
) -> &'static str {
    let mut regressed_request = context.base_request.clone();
    regressed_request.now_epoch_seconds = regressed_now_epoch_seconds;
    let _ = context.runtime.run_cycle(
        &mut context.m8_registry,
        &mut context.m10_registry,
        regressed_request,
    );
    context.runtime.state().last_reason_code
}

fn build_phase6_runtime_projection(
    reason_code: &'static str,
    runtime_state: &kamn_core::DataLayerM10Phase6SchedulerRuntimeState,
) -> DaemonPhase6RuntimeProjection {
    DaemonPhase6RuntimeProjection {
        reason_code,
        total_cycles: runtime_state.total_cycles,
        executed_cycles: runtime_state.executed_cycles,
        deferred_cycles: runtime_state.deferred_cycles,
        fail_closed_cycles: runtime_state.fail_closed_cycles,
    }
}
