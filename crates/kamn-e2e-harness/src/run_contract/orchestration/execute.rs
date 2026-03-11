use crate::{all_orchestration_phases, ExecutionMode, PhaseResultStatus, RunCommandConfig};

use super::super::evidence_io::persist_run_evidence_bundle;
use super::super::external_runtime::{ensure_external_execution_preflight, probe_external_runtime};
use super::payload::render_run_output;
use super::phase_model::{build_phase_results, status_totals_from_iter};
use super::scenario_execution::{
    execute_selected_scenarios, execute_selected_scenarios_contract_only, select_scenarios,
};

/// Executes run-command contract behavior and returns deterministic JSON summary.
pub fn execute_run_contract(config: &RunCommandConfig) -> Result<String, String> {
    let mode = validated_mode(config)?;
    run_external_preflight(config, mode)?;
    let selected = select_scenarios(config.scenario_ids.as_slice())?;
    let fail_markers = FailMarkers::from_dir(config.evidence_dir.as_str());
    let scenario_results = scenario_results(config, mode, &selected, fail_markers.scenario)?;
    let evidence_status = phase_status(fail_markers.evidence);
    let summary = execution_summary(mode, fail_markers.infra, &scenario_results, evidence_status);
    persist_run_evidence_bundle(
        config,
        summary.mode,
        selected.as_slice(),
        scenario_results.as_slice(),
        summary.scenario_totals.clone(),
        evidence_status,
    )?;
    Ok(render_contract_output(
        config,
        &selected,
        &scenario_results,
        &summary,
        evidence_status,
    ))
}

#[derive(Clone, Copy)]
struct FailMarkers {
    infra: bool,
    scenario: bool,
    evidence: bool,
}

impl FailMarkers {
    fn from_dir(evidence_dir: &str) -> Self {
        Self {
            infra: evidence_dir.contains("fail-path"),
            scenario: evidence_dir.contains("scenario-fail"),
            evidence: evidence_dir.contains("evidence-fail"),
        }
    }
}

struct RunExecutionSummary {
    mode: ExecutionMode,
    phase_results: Vec<crate::OrchestrationPhaseResult>,
    scenario_totals: crate::LifecycleStatusTotals,
}

fn validated_mode(config: &RunCommandConfig) -> Result<ExecutionMode, String> {
    let mode = ExecutionMode::parse(config.mode.as_str())?;
    if missing_agent_binary(config, mode) {
        return Err("missing required agent binary for MCP modes".to_owned());
    }
    Ok(mode)
}

fn missing_agent_binary(config: &RunCommandConfig, mode: ExecutionMode) -> bool {
    matches!(mode, ExecutionMode::McpTau | ExecutionMode::McpAny)
        && config
            .agent_binary
            .as_deref()
            .map(|value| value.trim().is_empty())
            .unwrap_or(true)
}

fn run_external_preflight(config: &RunCommandConfig, mode: ExecutionMode) -> Result<(), String> {
    if config.external_execution {
        return ensure_external_execution_preflight(config, mode);
    }
    Ok(())
}

fn scenario_results(
    config: &RunCommandConfig,
    mode: ExecutionMode,
    selected: &[crate::scenarios::ScenarioDefinition],
    fail_first: bool,
) -> Result<Vec<super::super::ScenarioExecutionResult>, String> {
    if config.external_execution {
        return execute_selected_scenarios(mode, selected, fail_first);
    }
    Ok(execute_selected_scenarios_contract_only(
        selected, fail_first,
    ))
}

fn execution_summary(
    mode: ExecutionMode,
    infra_fail: bool,
    scenario_results: &[super::super::ScenarioExecutionResult],
    evidence_status: PhaseResultStatus,
) -> RunExecutionSummary {
    let phase_results = build_phase_results(
        all_orchestration_phases().as_slice(),
        mode,
        infra_fail,
        scenario_results,
        evidence_status,
    );
    let scenario_totals =
        status_totals_from_iter(scenario_results.iter().map(|result| result.status));
    RunExecutionSummary {
        mode,
        phase_results,
        scenario_totals,
    }
}

fn render_contract_output(
    config: &RunCommandConfig,
    selected: &[crate::scenarios::ScenarioDefinition],
    scenario_results: &[super::super::ScenarioExecutionResult],
    summary: &RunExecutionSummary,
    evidence_status: PhaseResultStatus,
) -> String {
    render_run_output(
        config,
        summary.mode,
        selected,
        scenario_results,
        summary.scenario_totals.clone(),
        summary.phase_results.as_slice(),
        evidence_status,
        config
            .external_execution
            .then(|| probe_external_runtime(config, summary.mode)),
    )
}

fn phase_status(has_fail_marker: bool) -> PhaseResultStatus {
    if has_fail_marker {
        PhaseResultStatus::Fail
    } else {
        PhaseResultStatus::Pass
    }
}
