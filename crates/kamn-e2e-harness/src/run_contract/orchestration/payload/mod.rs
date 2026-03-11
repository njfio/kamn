mod phase_sections;
mod runtime_sections;
mod scenario_sections;

use crate::{
    ExecutionMode, LifecycleStatusTotals, OrchestrationPhaseResult, PhaseResultStatus,
    RunCommandConfig,
};

use super::super::{escape_json, ExternalRuntimeProbeSummary, ScenarioExecutionResult};
use super::phase_model::compute_lifecycle_summary;

pub(super) fn render_run_output(
    config: &RunCommandConfig,
    mode: ExecutionMode,
    selected: &[crate::scenarios::ScenarioDefinition],
    scenario_results: &[ScenarioExecutionResult],
    scenario_totals: LifecycleStatusTotals,
    phase_results: &[OrchestrationPhaseResult],
    evidence_status: PhaseResultStatus,
    external_runtime_probe: Option<ExternalRuntimeProbeSummary>,
) -> String {
    let lifecycle_summary = compute_lifecycle_summary(phase_results);
    let runtime = runtime_payload(
        config,
        mode,
        selected,
        scenario_results,
        scenario_totals.clone(),
        phase_results,
        evidence_status,
        external_runtime_probe,
    );
    let scenario = scenario_payload(selected, scenario_results);
    let phase = phase_payload(phase_results, &lifecycle_summary);
    render_run_output_json(config, mode, runtime, scenario, phase)
}

struct RuntimePayload {
    integration_config: String,
    runtime_external_execution: String,
    runtime_orchestration: String,
    runtime_lifecycle_execution: String,
    runtime_validation_execution: String,
    runtime_readiness: String,
    process_runtime: String,
    process_lifecycle: &'static str,
    spawn_timeline: &'static str,
    spawn_plan: String,
    spawn_execution: &'static str,
    live_process_execution: &'static str,
    mode_execution_contract: String,
    evidence_contract: String,
    live_execution: String,
    live_validation: String,
}

struct ScenarioPayload {
    count: usize,
    ids: String,
    results: String,
    contracts: String,
}

struct PhasePayload {
    count: usize,
    labels: String,
    results: String,
    phase_totals: String,
    step_totals: String,
}

fn runtime_payload(
    config: &RunCommandConfig,
    mode: ExecutionMode,
    selected: &[crate::scenarios::ScenarioDefinition],
    scenario_results: &[ScenarioExecutionResult],
    scenario_totals: LifecycleStatusTotals,
    phase_results: &[OrchestrationPhaseResult],
    evidence_status: PhaseResultStatus,
    external_runtime_probe: Option<ExternalRuntimeProbeSummary>,
) -> RuntimePayload {
    RuntimePayload {
        integration_config: runtime_sections::integration_config_json(config, mode),
        runtime_external_execution: runtime_sections::runtime_external_execution_json(
            config.external_execution,
            external_runtime_probe.as_ref(),
        ),
        runtime_orchestration: runtime_sections::runtime_orchestration_json(
            config.external_execution,
            external_runtime_probe.as_ref(),
        ),
        runtime_lifecycle_execution: runtime_sections::runtime_lifecycle_execution_json(
            config.external_execution,
            external_runtime_probe.as_ref(),
        ),
        runtime_validation_execution: runtime_sections::runtime_validation_execution_json(
            config.external_execution,
            external_runtime_probe.as_ref(),
            scenario_totals.clone(),
            evidence_status,
        ),
        runtime_readiness: runtime_sections::runtime_readiness_json(
            config,
            mode,
            selected.len() as u64,
        ),
        process_runtime: runtime_sections::process_runtime_json(mode),
        process_lifecycle: runtime_sections::process_lifecycle_json(),
        spawn_timeline: runtime_sections::spawn_timeline_json(),
        spawn_plan: runtime_sections::spawn_plan_json(mode),
        spawn_execution: runtime_sections::spawn_execution_json(),
        live_process_execution: runtime_sections::live_process_execution_json(),
        mode_execution_contract: runtime_sections::mode_execution_contract_json(
            mode,
            selected.len() as u64,
            scenario_results.len() as u64,
        ),
        evidence_contract: runtime_sections::evidence_contract_json(evidence_status),
        live_execution: runtime_sections::live_execution_json(
            phase_results,
            scenario_totals.clone(),
            evidence_status,
        ),
        live_validation: runtime_sections::live_validation_json(scenario_totals),
    }
}

fn scenario_payload(
    selected: &[crate::scenarios::ScenarioDefinition],
    scenario_results: &[ScenarioExecutionResult],
) -> ScenarioPayload {
    ScenarioPayload {
        count: selected.len(),
        ids: scenario_sections::scenario_ids_json(selected),
        results: scenario_sections::scenario_results_json(scenario_results),
        contracts: scenario_sections::scenario_contracts_json(selected, scenario_results),
    }
}

fn phase_payload(
    phase_results: &[OrchestrationPhaseResult],
    lifecycle_summary: &crate::LifecycleSummary,
) -> PhasePayload {
    PhasePayload {
        count: phase_results.len(),
        labels: phase_sections::phase_labels_json(),
        results: phase_sections::phase_results_json(phase_results),
        phase_totals: phase_sections::totals_json(&lifecycle_summary.phase_totals),
        step_totals: phase_sections::totals_json(&lifecycle_summary.step_totals),
    }
}

fn render_run_output_json(
    config: &RunCommandConfig,
    mode: ExecutionMode,
    runtime: RuntimePayload,
    scenario: ScenarioPayload,
    phase: PhasePayload,
) -> String {
    format!(
        "{{\"command\":\"run\",\"mode\":\"{}\",\"evidence_dir\":\"{}\",\"integration_config\":{},\"runtime_external_execution\":{},\"runtime_orchestration\":{},\"runtime_lifecycle_execution\":{},\"runtime_validation_execution\":{},\"runtime_readiness\":{},\"process_runtime\":{},\"process_lifecycle\":{},\"spawn_timeline\":{},\"spawn_plan\":{},\"spawn_execution\":{},\"live_process_execution\":{},\"mode_execution_contract\":{},\"evidence_contract\":{},\"live_execution\":{},\"live_validation\":{},\"scenario_count\":{},\"scenario_ids\":[{}],\"scenario_results\":[{}],\"scenario_contracts\":[{}],\"phase_count\":{},\"phases\":[{}],\"phase_results\":[{}],\"lifecycle_summary\":{{\"phase_totals\":{},\"step_totals\":{}}}}}",
        mode.as_str(),
        escape_json(config.evidence_dir.as_str()),
        runtime.integration_config,
        runtime.runtime_external_execution,
        runtime.runtime_orchestration,
        runtime.runtime_lifecycle_execution,
        runtime.runtime_validation_execution,
        runtime.runtime_readiness,
        runtime.process_runtime,
        runtime.process_lifecycle,
        runtime.spawn_timeline,
        runtime.spawn_plan,
        runtime.spawn_execution,
        runtime.live_process_execution,
        runtime.mode_execution_contract,
        runtime.evidence_contract,
        runtime.live_execution,
        runtime.live_validation,
        scenario.count,
        scenario.ids,
        scenario.results,
        scenario.contracts,
        phase.count,
        phase.labels,
        phase.results,
        phase.phase_totals,
        phase.step_totals,
    )
}
