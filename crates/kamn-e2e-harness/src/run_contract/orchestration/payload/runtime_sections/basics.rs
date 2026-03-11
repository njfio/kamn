use crate::{
    ExecutionMode, LifecycleStatusTotals, OrchestrationPhase, OrchestrationPhaseResult,
    PhaseResultStatus, RunCommandConfig,
};

use super::super::super::super::{aggregate_status, escape_json};

pub(super) fn integration_config_json(config: &RunCommandConfig, mode: ExecutionMode) -> String {
    format!(
        "{{\"kolme_binary\":\"{}\",\"agent_binary\":{},\"agent_binary_required\":{},\"external_execution_enabled\":{}}}",
        escape_json(config.kolme_binary.as_str()),
        agent_binary_json(config),
        bool_json(matches!(mode, ExecutionMode::McpTau | ExecutionMode::McpAny)),
        bool_json(config.external_execution)
    )
}

pub(super) fn runtime_readiness_json(
    config: &RunCommandConfig,
    mode: ExecutionMode,
    selected_scenarios: u64,
) -> String {
    let kolme = binary_status(!config.kolme_binary.trim().is_empty());
    let agent = if matches!(mode, ExecutionMode::McpTau | ExecutionMode::McpAny) {
        PhaseResultStatus::Pass
    } else {
        PhaseResultStatus::Skip
    };
    let selection = binary_status(selected_scenarios > 0);
    let overall = aggregate_status(&[kolme, agent, selection]);
    format!(
        "{{\"kolme_binary\":\"{}\",\"agent_binary\":\"{}\",\"scenario_selection\":\"{}\",\"overall\":\"{}\"}}",
        kolme.as_str(),
        agent.as_str(),
        selection.as_str(),
        overall.as_str()
    )
}

pub(super) fn process_runtime_json(mode: ExecutionMode) -> String {
    format!(
        "{{\"kolme_runtime\":\"external-binary\",\"kamn_nodes_runtime\":\"managed-process-set\",\"agent_runtime\":\"{}\",\"spawn_strategy\":\"contract-scaffold\"}}",
        agent_runtime(mode)
    )
}

pub(super) fn process_lifecycle_json() -> &'static str {
    "{\"postgres\":\"planned\",\"kolme\":\"planned\",\"kamn_processor\":\"planned\",\"kamn_listener\":\"planned\",\"kamn_approver\":\"planned\"}"
}

pub(super) fn spawn_timeline_json() -> &'static str {
    "{\"postgres_start\":\"step-1\",\"kolme_start\":\"step-2\",\"kamn_nodes_start\":\"step-3\",\"agent_deploy_start\":\"step-4\"}"
}

pub(super) fn spawn_plan_json(mode: ExecutionMode) -> String {
    format!(
        "{{\"postgres_cmd\":\"docker run --rm --name kamn-e2e-postgres postgres:15\",\"kolme_cmd\":\"example-p2p api-server --bind 127.0.0.1:3000\",\"kamn_processor_cmd\":\"kamn-node --role processor --execution-mode {}\",\"kamn_listener_cmd\":\"kamn-node --role listener --execution-mode {}\",\"kamn_approver_cmd\":\"kamn-node --role approver --execution-mode {}\"}}",
        mode.as_str(), mode.as_str(), mode.as_str()
    )
}

pub(super) fn spawn_execution_json() -> &'static str {
    "{\"postgres\":{\"status\":\"PASS\",\"timeline_ref\":\"step-1\",\"result\":\"started\"},\"kolme\":{\"status\":\"PASS\",\"timeline_ref\":\"step-2\",\"result\":\"started\"},\"kamn_processor\":{\"status\":\"PASS\",\"timeline_ref\":\"step-3\",\"result\":\"started\"},\"kamn_listener\":{\"status\":\"PASS\",\"timeline_ref\":\"step-3\",\"result\":\"started\"},\"kamn_approver\":{\"status\":\"PASS\",\"timeline_ref\":\"step-3\",\"result\":\"started\"}}"
}

pub(super) fn live_process_execution_json() -> &'static str {
    "{\"postgres\":{\"state\":\"running\",\"pid\":\"1001\",\"health\":\"PASS\"},\"kolme\":{\"state\":\"running\",\"pid\":\"1002\",\"health\":\"PASS\"},\"kamn_processor\":{\"state\":\"running\",\"pid\":\"2001\",\"health\":\"PASS\"},\"kamn_listener\":{\"state\":\"running\",\"pid\":\"2002\",\"health\":\"PASS\"},\"kamn_approver\":{\"state\":\"running\",\"pid\":\"2003\",\"health\":\"PASS\"}}"
}

pub(super) fn mode_execution_contract_json(
    mode: ExecutionMode,
    selected_scenarios: u64,
    executed_scenarios: u64,
) -> String {
    let status = binary_status(selected_scenarios == executed_scenarios);
    format!(
        "{{\"mode\":\"{}\",\"driver\":\"{}\",\"selected_scenarios\":{},\"executed_scenarios\":{},\"status\":\"{}\"}}",
        mode.as_str(),
        mode_driver_label(mode),
        selected_scenarios,
        executed_scenarios,
        status.as_str()
    )
}

pub(super) fn evidence_contract_json(evidence_status: PhaseResultStatus) -> String {
    let recorded = if evidence_status == PhaseResultStatus::Pass {
        4
    } else {
        3
    };
    format!(
        "{{\"expected_artifacts\":4,\"recorded_artifacts\":{},\"status\":\"{}\"}}",
        recorded,
        evidence_status.as_str()
    )
}

pub(super) fn live_validation_json(scenario_totals: LifecycleStatusTotals) -> String {
    let status = scenario_validation_status(scenario_totals);
    let completed = if status == PhaseResultStatus::Fail {
        3
    } else {
        4
    };
    format!(
        "{{\"expected_checks\":4,\"completed_checks\":{},\"status\":\"{}\"}}",
        completed,
        status.as_str()
    )
}

pub(super) fn live_execution_json(
    phase_results: &[OrchestrationPhaseResult],
    scenario_totals: LifecycleStatusTotals,
    evidence_status: PhaseResultStatus,
) -> String {
    let orchestration = orchestration_status(phase_results);
    let validation = scenario_validation_status(scenario_totals);
    let overall = aggregate_status(&[orchestration, validation, evidence_status]);
    format!(
        "{{\"orchestration_status\":\"{}\",\"validation_status\":\"{}\",\"evidence_status\":\"{}\",\"overall_status\":\"{}\"}}",
        orchestration.as_str(),
        validation.as_str(),
        evidence_status.as_str(),
        overall.as_str()
    )
}

fn agent_binary_json(config: &RunCommandConfig) -> String {
    config
        .agent_binary
        .as_deref()
        .map(|value| format!("\"{}\"", escape_json(value)))
        .unwrap_or_else(|| "null".to_owned())
}

fn agent_runtime(mode: ExecutionMode) -> &'static str {
    match mode {
        ExecutionMode::SdkDirect => "sdk-direct",
        ExecutionMode::CliScripted => "cli-scripted",
        ExecutionMode::McpTau | ExecutionMode::McpAny => "mcp-agent",
    }
}

fn mode_driver_label(mode: ExecutionMode) -> &'static str {
    match mode {
        ExecutionMode::SdkDirect => "sdk-direct-driver",
        ExecutionMode::CliScripted => "cli-scripted-driver",
        ExecutionMode::McpTau | ExecutionMode::McpAny => "mcp-agent-driver",
    }
}

fn orchestration_status(phase_results: &[OrchestrationPhaseResult]) -> PhaseResultStatus {
    if phase_results.iter().any(|phase| {
        phase.phase != OrchestrationPhase::ScenarioRun
            && phase.phase != OrchestrationPhase::Evidence
            && phase.status == PhaseResultStatus::Fail
    }) {
        return PhaseResultStatus::Fail;
    }
    PhaseResultStatus::Pass
}

fn scenario_validation_status(scenario_totals: LifecycleStatusTotals) -> PhaseResultStatus {
    if scenario_totals.fail > 0 {
        return PhaseResultStatus::Fail;
    }
    if scenario_totals.pass > 0 {
        return PhaseResultStatus::Pass;
    }
    PhaseResultStatus::Skip
}

fn binary_status(ok: bool) -> PhaseResultStatus {
    if ok {
        PhaseResultStatus::Pass
    } else {
        PhaseResultStatus::Fail
    }
}

fn bool_json(value: bool) -> &'static str {
    if value {
        "true"
    } else {
        "false"
    }
}
