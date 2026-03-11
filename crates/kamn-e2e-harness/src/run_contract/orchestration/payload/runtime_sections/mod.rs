mod basics;
mod external;

use crate::{
    ExecutionMode, LifecycleStatusTotals, OrchestrationPhaseResult, PhaseResultStatus,
    RunCommandConfig,
};

use super::super::super::ExternalRuntimeProbeSummary;

pub(super) fn integration_config_json(config: &RunCommandConfig, mode: ExecutionMode) -> String {
    basics::integration_config_json(config, mode)
}

pub(super) fn runtime_readiness_json(
    config: &RunCommandConfig,
    mode: ExecutionMode,
    selected_scenarios: u64,
) -> String {
    basics::runtime_readiness_json(config, mode, selected_scenarios)
}

pub(super) fn process_runtime_json(mode: ExecutionMode) -> String {
    basics::process_runtime_json(mode)
}

pub(super) fn process_lifecycle_json() -> &'static str {
    basics::process_lifecycle_json()
}

pub(super) fn spawn_timeline_json() -> &'static str {
    basics::spawn_timeline_json()
}

pub(super) fn spawn_plan_json(mode: ExecutionMode) -> String {
    basics::spawn_plan_json(mode)
}

pub(super) fn spawn_execution_json() -> &'static str {
    basics::spawn_execution_json()
}

pub(super) fn live_process_execution_json() -> &'static str {
    basics::live_process_execution_json()
}

pub(super) fn mode_execution_contract_json(
    mode: ExecutionMode,
    selected_scenarios: u64,
    executed_scenarios: u64,
) -> String {
    basics::mode_execution_contract_json(mode, selected_scenarios, executed_scenarios)
}

pub(super) fn evidence_contract_json(evidence_status: PhaseResultStatus) -> String {
    basics::evidence_contract_json(evidence_status)
}

pub(super) fn live_validation_json(scenario_totals: LifecycleStatusTotals) -> String {
    basics::live_validation_json(scenario_totals)
}

pub(super) fn live_execution_json(
    phase_results: &[OrchestrationPhaseResult],
    scenario_totals: LifecycleStatusTotals,
    evidence_status: PhaseResultStatus,
) -> String {
    basics::live_execution_json(phase_results, scenario_totals, evidence_status)
}

pub(super) fn runtime_external_execution_json(
    external_execution: bool,
    probe_summary: Option<&ExternalRuntimeProbeSummary>,
) -> String {
    external::runtime_external_execution_json(external_execution, probe_summary)
}

pub(super) fn runtime_orchestration_json(
    external_execution: bool,
    probe_summary: Option<&ExternalRuntimeProbeSummary>,
) -> String {
    external::runtime_orchestration_json(external_execution, probe_summary)
}

pub(super) fn runtime_lifecycle_execution_json(
    external_execution: bool,
    probe_summary: Option<&ExternalRuntimeProbeSummary>,
) -> String {
    external::runtime_lifecycle_execution_json(external_execution, probe_summary)
}

pub(super) fn runtime_validation_execution_json(
    external_execution: bool,
    probe_summary: Option<&ExternalRuntimeProbeSummary>,
    scenario_totals: LifecycleStatusTotals,
    evidence_status: PhaseResultStatus,
) -> String {
    external::runtime_validation_execution_json(
        external_execution,
        probe_summary,
        scenario_totals,
        evidence_status,
    )
}
