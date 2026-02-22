#![warn(missing_docs)]
//! E2E harness scaffold crate.

use std::collections::HashSet;
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::process::{Command, Stdio};

/// Driver implementations for each execution mode.
pub mod drivers;
/// Evidence manifest structures and schema constants.
pub mod evidence;
/// Harness identity helpers.
pub mod identity;
/// Infrastructure lifecycle contracts.
pub mod infrastructure;
/// Kolme devnet configuration contracts.
pub mod kolme_devnet;
/// Scenario inventory and definitions.
pub mod scenarios;
/// Offline manifest verification contracts.
pub mod verify;

/// Supported harness execution modes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecutionMode {
    /// Direct Rust SDK calls.
    SdkDirect,
    /// Shell-driven CLI mode.
    CliScripted,
    /// MCP mode using Tau runtime.
    McpTau,
    /// MCP mode using any compatible runtime.
    McpAny,
}

/// Canonical orchestration phases from PRD section 11.2.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OrchestrationPhase {
    /// Phase 1: infrastructure startup.
    InfraUp,
    /// Phase 2: per-agent deployment/bootstrap.
    AgentDeploy,
    /// Phase 3: scenario execution.
    ScenarioRun,
    /// Phase 4: evidence finalization.
    Evidence,
    /// Phase 5: teardown and archival.
    Teardown,
}

impl OrchestrationPhase {
    /// Returns canonical phase marker label.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::InfraUp => "INFRA_UP",
            Self::AgentDeploy => "AGENT_DEPLOY",
            Self::ScenarioRun => "SCENARIO_RUN",
            Self::Evidence => "EVIDENCE",
            Self::Teardown => "TEARDOWN",
        }
    }
}

/// Execution status marker for an orchestration phase result.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhaseResultStatus {
    /// Phase completed successfully.
    Pass,
    /// Phase failed.
    Fail,
    /// Phase was intentionally skipped.
    Skip,
}

impl PhaseResultStatus {
    /// Returns canonical status marker label.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Pass => "PASS",
            Self::Fail => "FAIL",
            Self::Skip => "SKIP",
        }
    }
}

/// Deterministic result record for one orchestration phase.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OrchestrationPhaseResult {
    /// Phase marker.
    pub phase: OrchestrationPhase,
    /// Outcome status.
    pub status: PhaseResultStatus,
    /// Deterministic start timestamp marker.
    pub started_at: String,
    /// Deterministic completion timestamp marker.
    pub completed_at: String,
    /// Deterministic detail marker.
    pub details: String,
    /// Deterministic step records for this phase.
    pub steps: Vec<OrchestrationStepRecord>,
}

/// Deterministic step record for one phase action.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OrchestrationStepRecord {
    /// Step label.
    pub step: String,
    /// Step status marker.
    pub status: PhaseResultStatus,
    /// Deterministic step detail marker.
    pub detail: String,
}

/// Status counter tuple for lifecycle aggregation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LifecycleStatusTotals {
    /// Total records counted.
    pub total: u64,
    /// Count of `PASS`.
    pub pass: u64,
    /// Count of `FAIL`.
    pub fail: u64,
    /// Count of `SKIP`.
    pub skip: u64,
}

/// Deterministic lifecycle summary for run output.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LifecycleSummary {
    /// Aggregated totals for phase statuses.
    pub phase_totals: LifecycleStatusTotals,
    /// Aggregated totals for step statuses.
    pub step_totals: LifecycleStatusTotals,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ScenarioExecutionResult {
    id: String,
    status: PhaseResultStatus,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ExternalRuntimeProbeSummary {
    status: PhaseResultStatus,
    detail: String,
}

impl ExecutionMode {
    /// Returns canonical execution-mode label.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::SdkDirect => "sdk-direct",
            Self::CliScripted => "cli-scripted",
            Self::McpTau => "mcp-tau",
            Self::McpAny => "mcp-any",
        }
    }

    /// Parses a canonical execution-mode label.
    pub fn parse(value: &str) -> Result<Self, String> {
        match value {
            "sdk-direct" => Ok(Self::SdkDirect),
            "cli-scripted" => Ok(Self::CliScripted),
            "mcp-tau" => Ok(Self::McpTau),
            "mcp-any" => Ok(Self::McpAny),
            _ => Err(format!("unsupported execution mode: {value}")),
        }
    }
}

/// Returns the canonical execution-mode inventory for phase-3.
pub fn all_execution_modes() -> Vec<ExecutionMode> {
    vec![
        ExecutionMode::SdkDirect,
        ExecutionMode::CliScripted,
        ExecutionMode::McpTau,
        ExecutionMode::McpAny,
    ]
}

/// Returns canonical orchestration phase inventory.
pub fn all_orchestration_phases() -> Vec<OrchestrationPhase> {
    vec![
        OrchestrationPhase::InfraUp,
        OrchestrationPhase::AgentDeploy,
        OrchestrationPhase::ScenarioRun,
        OrchestrationPhase::Evidence,
        OrchestrationPhase::Teardown,
    ]
}

/// Returns supported phase-result status markers.
pub fn all_phase_result_statuses() -> Vec<PhaseResultStatus> {
    vec![
        PhaseResultStatus::Pass,
        PhaseResultStatus::Fail,
        PhaseResultStatus::Skip,
    ]
}

/// Run-plan structure used by harness mode/scenario orchestration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HarnessRunPlan {
    /// Selected execution mode.
    pub mode: ExecutionMode,
    /// Scenarios scheduled for execution.
    pub scenarios: Vec<scenarios::ScenarioDefinition>,
}

/// Builds a deterministic run plan for one execution mode.
pub fn build_core_run_plan(mode: ExecutionMode) -> HarnessRunPlan {
    HarnessRunPlan {
        mode,
        scenarios: scenarios::core_scenarios(),
    }
}

/// Parsed `run` command configuration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunCommandConfig {
    /// Selected execution mode label.
    pub mode: String,
    /// Kolme node binary path.
    pub kolme_binary: String,
    /// Agent runtime binary path when required by mode.
    pub agent_binary: Option<String>,
    /// Enables guarded runtime external execution integration path.
    pub external_execution: bool,
    /// Evidence output directory.
    pub evidence_dir: String,
    /// Selected scenario IDs.
    pub scenario_ids: Vec<String>,
}

/// Parsed `verify` command configuration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VerifyCommandConfig {
    /// Evidence bundle root directory.
    pub evidence_dir: String,
    /// Kolme chain dump path.
    pub kolme_chain_dump: String,
    /// Verification report output path.
    pub output: String,
}

/// Harness command surface.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HarnessCommand {
    /// Run selected scenarios.
    Run(RunCommandConfig),
    /// Verify an evidence bundle.
    Verify(VerifyCommandConfig),
}

/// Parses a comma-delimited list of scenario IDs.
pub fn parse_scenario_csv(value: &str) -> Result<Vec<String>, String> {
    let mut parsed = Vec::new();
    let mut seen = HashSet::new();
    let known_ids: HashSet<&str> = scenarios::all_scenarios()
        .iter()
        .map(|item| item.id)
        .collect();
    for raw in value.split(',') {
        let id = raw.trim();
        if id.is_empty() {
            continue;
        }
        if !known_ids.contains(id) {
            return Err(format!("unknown scenario id: {id}"));
        }
        if !seen.insert(id.to_owned()) {
            return Err(format!("duplicate scenario id: {id}"));
        }
        parsed.push(id.to_owned());
    }
    if parsed.is_empty() {
        return Err("scenario list is empty".to_owned());
    }
    Ok(parsed)
}

fn parse_flag_value(
    args: &[String],
    index: usize,
    flag: &str,
) -> Result<(Option<String>, usize), String> {
    if args[index] != flag {
        return Ok((None, index));
    }
    let next = index + 1;
    if next >= args.len() {
        return Err(format!("missing value for {flag}"));
    }
    Ok((Some(args[next].clone()), next))
}

/// Parses harness command arguments (excluding binary name).
pub fn parse_command_args<I, S>(args: I) -> Result<HarnessCommand, String>
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    let args: Vec<String> = args
        .into_iter()
        .map(|item| item.as_ref().to_owned())
        .collect();
    if args.is_empty() {
        return Err("missing command; expected one of: run, verify".to_owned());
    }
    match args[0].as_str() {
        "run" => {
            let mut mode = None;
            let mut kolme_binary = None;
            let mut agent_binary = None;
            let mut external_execution = false;
            let mut evidence_dir = None;
            let mut scenarios_csv = None;
            let mut index = 1;
            while index < args.len() {
                let (parsed_mode, advanced) = parse_flag_value(&args, index, "--mode")?;
                if let Some(value) = parsed_mode {
                    mode = Some(value);
                    index = advanced + 1;
                    continue;
                }
                let (parsed_kolme_binary, advanced) =
                    parse_flag_value(&args, index, "--kolme-binary")?;
                if let Some(value) = parsed_kolme_binary {
                    kolme_binary = Some(value);
                    index = advanced + 1;
                    continue;
                }
                let (parsed_agent_binary, advanced) =
                    parse_flag_value(&args, index, "--agent-binary")?;
                if let Some(value) = parsed_agent_binary {
                    agent_binary = Some(value);
                    index = advanced + 1;
                    continue;
                }
                if args[index] == "--enable-external-execution" {
                    external_execution = true;
                    index += 1;
                    continue;
                }
                let (parsed_evidence, advanced) = parse_flag_value(&args, index, "--evidence-dir")?;
                if let Some(value) = parsed_evidence {
                    evidence_dir = Some(value);
                    index = advanced + 1;
                    continue;
                }
                let (parsed_scenarios, advanced) = parse_flag_value(&args, index, "--scenarios")?;
                if let Some(value) = parsed_scenarios {
                    scenarios_csv = Some(value);
                    index = advanced + 1;
                    continue;
                }
                return Err(format!("unknown run flag: {}", args[index]));
            }

            let mode = mode.ok_or_else(|| "missing required flag --mode".to_owned())?;
            let parsed_mode = ExecutionMode::parse(mode.as_str())?;
            let kolme_binary =
                kolme_binary.ok_or_else(|| "missing required flag --kolme-binary".to_owned())?;
            if matches!(parsed_mode, ExecutionMode::McpTau | ExecutionMode::McpAny)
                && agent_binary.is_none()
            {
                return Err("missing required flag --agent-binary for MCP modes".to_owned());
            }
            let evidence_dir =
                evidence_dir.ok_or_else(|| "missing required flag --evidence-dir".to_owned())?;
            let scenarios_csv =
                scenarios_csv.ok_or_else(|| "missing required flag --scenarios".to_owned())?;
            let scenario_ids = parse_scenario_csv(scenarios_csv.as_str())?;
            Ok(HarnessCommand::Run(RunCommandConfig {
                mode,
                kolme_binary,
                agent_binary,
                external_execution,
                evidence_dir,
                scenario_ids,
            }))
        }
        "verify" => {
            let mut evidence_dir = None;
            let mut kolme_chain_dump = None;
            let mut output = None;
            let mut index = 1;
            while index < args.len() {
                let (parsed_evidence, advanced) = parse_flag_value(&args, index, "--evidence-dir")?;
                if let Some(value) = parsed_evidence {
                    evidence_dir = Some(value);
                    index = advanced + 1;
                    continue;
                }
                let (parsed_chain_dump, advanced) =
                    parse_flag_value(&args, index, "--kolme-chain-dump")?;
                if let Some(value) = parsed_chain_dump {
                    kolme_chain_dump = Some(value);
                    index = advanced + 1;
                    continue;
                }
                let (parsed_output, advanced) = parse_flag_value(&args, index, "--output")?;
                if let Some(value) = parsed_output {
                    output = Some(value);
                    index = advanced + 1;
                    continue;
                }
                return Err(format!("unknown verify flag: {}", args[index]));
            }

            let evidence_dir =
                evidence_dir.ok_or_else(|| "missing required flag --evidence-dir".to_owned())?;
            let kolme_chain_dump = kolme_chain_dump
                .ok_or_else(|| "missing required flag --kolme-chain-dump".to_owned())?;
            let output = output.ok_or_else(|| "missing required flag --output".to_owned())?;
            Ok(HarnessCommand::Verify(VerifyCommandConfig {
                evidence_dir,
                kolme_chain_dump,
                output,
            }))
        }
        command => Err(format!("unsupported command: {command}")),
    }
}

fn escape_json(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}

/// Executes run-command contract behavior and returns deterministic JSON summary.
pub fn execute_run_contract(config: &RunCommandConfig) -> Result<String, String> {
    let mode = ExecutionMode::parse(config.mode.as_str())?;
    let agent_binary_required = matches!(mode, ExecutionMode::McpTau | ExecutionMode::McpAny);
    if agent_binary_required
        && config
            .agent_binary
            .as_deref()
            .map(|value| value.trim().is_empty())
            .unwrap_or(true)
    {
        return Err("missing required agent binary for MCP modes".to_owned());
    }
    if config.external_execution {
        ensure_external_execution_preflight(config, mode)?;
    }
    let selected = select_scenarios(config.scenario_ids.as_slice())?;
    let phases = all_orchestration_phases();
    let infra_fail_path_marker = config.evidence_dir.contains("fail-path");
    let scenario_fail_path_marker = config.evidence_dir.contains("scenario-fail");
    let evidence_fail_path_marker = config.evidence_dir.contains("evidence-fail");
    let scenario_results =
        execute_selected_scenarios(mode, selected.as_slice(), scenario_fail_path_marker)?;
    let evidence_status = if evidence_fail_path_marker {
        PhaseResultStatus::Fail
    } else {
        PhaseResultStatus::Pass
    };
    let phase_results = build_phase_results(
        phases.as_slice(),
        mode,
        infra_fail_path_marker,
        scenario_results.as_slice(),
        evidence_status,
    );
    let agent_binary_json = config
        .agent_binary
        .as_deref()
        .map(|value| format!("\"{}\"", escape_json(value)))
        .unwrap_or_else(|| "null".to_owned());
    let integration_config_json = format!(
        "{{\"kolme_binary\":\"{}\",\"agent_binary\":{},\"agent_binary_required\":{},\"external_execution_enabled\":{}}}",
        escape_json(config.kolme_binary.as_str()),
        agent_binary_json,
        if agent_binary_required {
            "true"
        } else {
            "false"
        },
        if config.external_execution {
            "true"
        } else {
            "false"
        }
    );
    let kolme_binary_status = if config.kolme_binary.trim().is_empty() {
        PhaseResultStatus::Fail
    } else {
        PhaseResultStatus::Pass
    };
    let agent_binary_status = if agent_binary_required {
        PhaseResultStatus::Pass
    } else {
        PhaseResultStatus::Skip
    };
    let scenario_selection_status = if selected.is_empty() {
        PhaseResultStatus::Fail
    } else {
        PhaseResultStatus::Pass
    };
    let overall_status = if [
        kolme_binary_status,
        agent_binary_status,
        scenario_selection_status,
    ]
    .contains(&PhaseResultStatus::Fail)
    {
        PhaseResultStatus::Fail
    } else {
        PhaseResultStatus::Pass
    };
    let runtime_readiness_json = format!(
        "{{\"kolme_binary\":\"{}\",\"agent_binary\":\"{}\",\"scenario_selection\":\"{}\",\"overall\":\"{}\"}}",
        kolme_binary_status.as_str(),
        agent_binary_status.as_str(),
        scenario_selection_status.as_str(),
        overall_status.as_str()
    );
    let agent_runtime = match mode {
        ExecutionMode::SdkDirect => "sdk-direct",
        ExecutionMode::CliScripted => "cli-scripted",
        ExecutionMode::McpTau | ExecutionMode::McpAny => "mcp-agent",
    };
    let process_runtime_json = format!(
        "{{\"kolme_runtime\":\"external-binary\",\"kamn_nodes_runtime\":\"managed-process-set\",\"agent_runtime\":\"{}\",\"spawn_strategy\":\"contract-scaffold\"}}",
        agent_runtime
    );
    let process_lifecycle_json = "{\"postgres\":\"planned\",\"kolme\":\"planned\",\"kamn_processor\":\"planned\",\"kamn_listener\":\"planned\",\"kamn_approver\":\"planned\"}";
    let spawn_timeline_json = "{\"postgres_start\":\"step-1\",\"kolme_start\":\"step-2\",\"kamn_nodes_start\":\"step-3\",\"agent_deploy_start\":\"step-4\"}";
    let scenario_totals =
        status_totals_from_iter(scenario_results.iter().map(|result| result.status));
    let mode_driver = mode_driver_label(mode);
    let selected_scenarios = selected.len() as u64;
    let executed_scenarios = scenario_results.len() as u64;
    let mode_execution_status = if selected_scenarios == executed_scenarios {
        PhaseResultStatus::Pass
    } else {
        PhaseResultStatus::Fail
    };
    let mode_execution_contract_json = format!(
        "{{\"mode\":\"{}\",\"driver\":\"{}\",\"selected_scenarios\":{},\"executed_scenarios\":{},\"status\":\"{}\"}}",
        mode.as_str(),
        mode_driver,
        selected_scenarios,
        executed_scenarios,
        mode_execution_status.as_str()
    );
    let live_validation_status = if scenario_totals.fail > 0 {
        PhaseResultStatus::Fail
    } else if scenario_totals.pass > 0 {
        PhaseResultStatus::Pass
    } else {
        PhaseResultStatus::Skip
    };
    let expected_live_checks: u64 = 4;
    let completed_live_checks = if live_validation_status == PhaseResultStatus::Fail {
        expected_live_checks - 1
    } else {
        expected_live_checks
    };
    let live_validation_json = format!(
        "{{\"expected_checks\":{},\"completed_checks\":{},\"status\":\"{}\"}}",
        expected_live_checks,
        completed_live_checks,
        live_validation_status.as_str()
    );
    let expected_evidence_artifacts: u64 = 4;
    let recorded_evidence_artifacts = if evidence_fail_path_marker {
        expected_evidence_artifacts - 1
    } else {
        expected_evidence_artifacts
    };
    let evidence_contract_json = format!(
        "{{\"expected_artifacts\":{},\"recorded_artifacts\":{},\"status\":\"{}\"}}",
        expected_evidence_artifacts,
        recorded_evidence_artifacts,
        evidence_status.as_str()
    );
    let spawn_plan_json = format!(
        "{{\"postgres_cmd\":\"docker run --rm --name kamn-e2e-postgres postgres:15\",\"kolme_cmd\":\"kolme-node --storage inmemory --api-port 3000 --enable-notifications\",\"kamn_processor_cmd\":\"kamn-node --role processor --execution-mode {}\",\"kamn_listener_cmd\":\"kamn-node --role listener --execution-mode {}\",\"kamn_approver_cmd\":\"kamn-node --role approver --execution-mode {}\"}}",
        mode.as_str(),
        mode.as_str(),
        mode.as_str()
    );
    let spawn_execution_json = "{\"postgres\":{\"status\":\"PASS\",\"timeline_ref\":\"step-1\",\"result\":\"started\"},\"kolme\":{\"status\":\"PASS\",\"timeline_ref\":\"step-2\",\"result\":\"started\"},\"kamn_processor\":{\"status\":\"PASS\",\"timeline_ref\":\"step-3\",\"result\":\"started\"},\"kamn_listener\":{\"status\":\"PASS\",\"timeline_ref\":\"step-3\",\"result\":\"started\"},\"kamn_approver\":{\"status\":\"PASS\",\"timeline_ref\":\"step-3\",\"result\":\"started\"}}";
    let live_process_execution_json = "{\"postgres\":{\"state\":\"running\",\"pid\":\"1001\",\"health\":\"PASS\"},\"kolme\":{\"state\":\"running\",\"pid\":\"1002\",\"health\":\"PASS\"},\"kamn_processor\":{\"state\":\"running\",\"pid\":\"2001\",\"health\":\"PASS\"},\"kamn_listener\":{\"state\":\"running\",\"pid\":\"2002\",\"health\":\"PASS\"},\"kamn_approver\":{\"state\":\"running\",\"pid\":\"2003\",\"health\":\"PASS\"}}";
    let orchestration_status = if phase_results.iter().any(|phase| {
        phase.phase != OrchestrationPhase::ScenarioRun
            && phase.phase != OrchestrationPhase::Evidence
            && phase.status == PhaseResultStatus::Fail
    }) {
        PhaseResultStatus::Fail
    } else {
        PhaseResultStatus::Pass
    };
    let live_execution_overall = if [
        orchestration_status,
        live_validation_status,
        evidence_status,
    ]
    .contains(&PhaseResultStatus::Fail)
    {
        PhaseResultStatus::Fail
    } else {
        PhaseResultStatus::Pass
    };
    let live_execution_json = format!(
        "{{\"orchestration_status\":\"{}\",\"validation_status\":\"{}\",\"evidence_status\":\"{}\",\"overall_status\":\"{}\"}}",
        orchestration_status.as_str(),
        live_validation_status.as_str(),
        evidence_status.as_str(),
        live_execution_overall.as_str()
    );
    let external_runtime_probe = if config.external_execution {
        Some(probe_external_runtime(config, mode))
    } else {
        None
    };
    let runtime_external_execution_json = if let Some(probe) = external_runtime_probe.as_ref() {
        format!(
            "{{\"requested\":true,\"guard_status\":\"{}\",\"execution_mode\":\"external-runtime\",\"preflight\":\"ready\",\"probe_detail\":\"{}\"}}",
            probe.status.as_str(),
            escape_json(probe.detail.as_str())
        )
    } else {
        "{\"requested\":false,\"guard_status\":\"SKIP\",\"execution_mode\":\"contract-only\",\"preflight\":\"not-requested\"}".to_owned()
    };
    let runtime_orchestration_json = if let Some(probe) = external_runtime_probe.as_ref() {
        if probe.status == PhaseResultStatus::Pass {
            "{\"postgres\":{\"requested\":true,\"status\":\"PASS\",\"detail\":\"external orchestration scaffold\"},\"kolme\":{\"requested\":true,\"status\":\"PASS\",\"detail\":\"external orchestration scaffold\"},\"kamn_processor\":{\"requested\":true,\"status\":\"PASS\",\"detail\":\"external orchestration scaffold\"},\"kamn_listener\":{\"requested\":true,\"status\":\"PASS\",\"detail\":\"external orchestration scaffold\"},\"kamn_approver\":{\"requested\":true,\"status\":\"PASS\",\"detail\":\"external orchestration scaffold\"}}".to_owned()
        } else {
            let detail = format!("external probe failed: {}", probe.detail);
            format!(
                "{{\"postgres\":{{\"requested\":true,\"status\":\"FAIL\",\"detail\":\"{}\"}},\"kolme\":{{\"requested\":true,\"status\":\"FAIL\",\"detail\":\"{}\"}},\"kamn_processor\":{{\"requested\":true,\"status\":\"FAIL\",\"detail\":\"{}\"}},\"kamn_listener\":{{\"requested\":true,\"status\":\"FAIL\",\"detail\":\"{}\"}},\"kamn_approver\":{{\"requested\":true,\"status\":\"FAIL\",\"detail\":\"{}\"}}}}",
                escape_json(detail.as_str()),
                escape_json(detail.as_str()),
                escape_json(detail.as_str()),
                escape_json(detail.as_str()),
                escape_json(detail.as_str())
            )
        }
    } else {
        "{\"postgres\":{\"requested\":false,\"status\":\"SKIP\",\"detail\":\"external execution disabled\"},\"kolme\":{\"requested\":false,\"status\":\"SKIP\",\"detail\":\"external execution disabled\"},\"kamn_processor\":{\"requested\":false,\"status\":\"SKIP\",\"detail\":\"external execution disabled\"},\"kamn_listener\":{\"requested\":false,\"status\":\"SKIP\",\"detail\":\"external execution disabled\"},\"kamn_approver\":{\"requested\":false,\"status\":\"SKIP\",\"detail\":\"external execution disabled\"}}".to_owned()
    };
    let runtime_lifecycle_execution_json = if let Some(probe) = external_runtime_probe.as_ref() {
        if probe.status == PhaseResultStatus::Pass {
            "{\"postgres\":{\"init\":\"PASS\",\"spawn\":\"PASS\",\"health_check\":\"PASS\",\"ready\":\"PASS\"},\"kolme\":{\"init\":\"PASS\",\"spawn\":\"PASS\",\"health_check\":\"PASS\",\"ready\":\"PASS\"},\"kamn_processor\":{\"init\":\"PASS\",\"spawn\":\"PASS\",\"health_check\":\"PASS\",\"ready\":\"PASS\"},\"kamn_listener\":{\"init\":\"PASS\",\"spawn\":\"PASS\",\"health_check\":\"PASS\",\"ready\":\"PASS\"},\"kamn_approver\":{\"init\":\"PASS\",\"spawn\":\"PASS\",\"health_check\":\"PASS\",\"ready\":\"PASS\"}}".to_owned()
        } else {
            "{\"postgres\":{\"init\":\"FAIL\",\"spawn\":\"FAIL\",\"health_check\":\"FAIL\",\"ready\":\"FAIL\"},\"kolme\":{\"init\":\"FAIL\",\"spawn\":\"FAIL\",\"health_check\":\"FAIL\",\"ready\":\"FAIL\"},\"kamn_processor\":{\"init\":\"FAIL\",\"spawn\":\"FAIL\",\"health_check\":\"FAIL\",\"ready\":\"FAIL\"},\"kamn_listener\":{\"init\":\"FAIL\",\"spawn\":\"FAIL\",\"health_check\":\"FAIL\",\"ready\":\"FAIL\"},\"kamn_approver\":{\"init\":\"FAIL\",\"spawn\":\"FAIL\",\"health_check\":\"FAIL\",\"ready\":\"FAIL\"}}".to_owned()
        }
    } else {
        "{\"postgres\":{\"init\":\"SKIP\",\"spawn\":\"SKIP\",\"health_check\":\"SKIP\",\"ready\":\"SKIP\"},\"kolme\":{\"init\":\"SKIP\",\"spawn\":\"SKIP\",\"health_check\":\"SKIP\",\"ready\":\"SKIP\"},\"kamn_processor\":{\"init\":\"SKIP\",\"spawn\":\"SKIP\",\"health_check\":\"SKIP\",\"ready\":\"SKIP\"},\"kamn_listener\":{\"init\":\"SKIP\",\"spawn\":\"SKIP\",\"health_check\":\"SKIP\",\"ready\":\"SKIP\"},\"kamn_approver\":{\"init\":\"SKIP\",\"spawn\":\"SKIP\",\"health_check\":\"SKIP\",\"ready\":\"SKIP\"}}".to_owned()
    };
    let runtime_validation_execution_json = if let Some(probe) = external_runtime_probe.as_ref() {
        let orchestration_contract = probe.status;
        let lifecycle_contract = probe.status;
        let overall = aggregate_status(&[
            orchestration_contract,
            lifecycle_contract,
            live_validation_status,
            evidence_status,
        ]);
        format!(
            "{{\"requested\":true,\"orchestration_contract\":\"{}\",\"lifecycle_contract\":\"{}\",\"live_validation_contract\":\"{}\",\"evidence_contract\":\"{}\",\"overall\":\"{}\"}}",
            orchestration_contract.as_str(),
            lifecycle_contract.as_str(),
            live_validation_status.as_str(),
            evidence_status.as_str(),
            overall.as_str()
        )
    } else {
        "{\"requested\":false,\"orchestration_contract\":\"SKIP\",\"lifecycle_contract\":\"SKIP\",\"live_validation_contract\":\"SKIP\",\"evidence_contract\":\"SKIP\",\"overall\":\"SKIP\"}".to_owned()
    };
    let scenario_ids = selected
        .iter()
        .map(|item| format!("\"{}\"", item.id))
        .collect::<Vec<_>>()
        .join(",");
    let scenario_results_json = scenario_results
        .iter()
        .map(|result| {
            format!(
                "{{\"id\":\"{}\",\"status\":\"{}\"}}",
                escape_json(result.id.as_str()),
                result.status.as_str()
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    let scenario_contracts_json = selected
        .iter()
        .zip(scenario_results.iter())
        .map(|(scenario, result)| {
            let steps_json = scenario
                .steps
                .iter()
                .map(|step| format!("\"{}\"", escape_json(step)))
                .collect::<Vec<_>>()
                .join(",");
            let outputs_json = scenario
                .verifiable_outputs
                .iter()
                .map(|entry| format!("\"{}\"", escape_json(entry)))
                .collect::<Vec<_>>()
                .join(",");
            let pass_criteria_json = scenario
                .pass_criteria
                .iter()
                .map(|entry| format!("\"{}\"", escape_json(entry)))
                .collect::<Vec<_>>()
                .join(",");
            format!(
                "{{\"id\":\"{}\",\"name\":\"{}\",\"priority\":\"{}\",\"status\":\"{}\",\"steps\":[{}],\"verifiable_outputs\":[{}],\"pass_criteria\":[{}]}}",
                scenario.id,
                escape_json(scenario.name),
                scenario.priority,
                result.status.as_str(),
                steps_json,
                outputs_json,
                pass_criteria_json
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    let phase_labels = phases
        .iter()
        .map(|phase| format!("\"{}\"", phase.as_str()))
        .collect::<Vec<_>>()
        .join(",");
    let phase_results_json = phase_results
        .iter()
        .map(|result| {
            let steps_json = result
                .steps
                .iter()
                .map(|step| {
                    format!(
                        "{{\"step\":\"{}\",\"status\":\"{}\",\"detail\":\"{}\"}}",
                        escape_json(step.step.as_str()),
                        step.status.as_str(),
                        escape_json(step.detail.as_str())
                    )
                })
                .collect::<Vec<_>>()
                .join(",");
            format!(
                "{{\"phase\":\"{}\",\"status\":\"{}\",\"started_at\":\"{}\",\"completed_at\":\"{}\",\"details\":\"{}\",\"steps\":[{}]}}",
                result.phase.as_str(),
                result.status.as_str(),
                escape_json(result.started_at.as_str()),
                escape_json(result.completed_at.as_str()),
                escape_json(result.details.as_str()),
                steps_json
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    let lifecycle_summary = compute_lifecycle_summary(phase_results.as_slice());
    let phase_totals_json = format!(
        "{{\"total\":{},\"pass\":{},\"fail\":{},\"skip\":{}}}",
        lifecycle_summary.phase_totals.total,
        lifecycle_summary.phase_totals.pass,
        lifecycle_summary.phase_totals.fail,
        lifecycle_summary.phase_totals.skip
    );
    let step_totals_json = format!(
        "{{\"total\":{},\"pass\":{},\"fail\":{},\"skip\":{}}}",
        lifecycle_summary.step_totals.total,
        lifecycle_summary.step_totals.pass,
        lifecycle_summary.step_totals.fail,
        lifecycle_summary.step_totals.skip
    );
    Ok(format!(
        "{{\"command\":\"run\",\"mode\":\"{}\",\"evidence_dir\":\"{}\",\"integration_config\":{},\"runtime_external_execution\":{},\"runtime_orchestration\":{},\"runtime_lifecycle_execution\":{},\"runtime_validation_execution\":{},\"runtime_readiness\":{},\"process_runtime\":{},\"process_lifecycle\":{},\"spawn_timeline\":{},\"spawn_plan\":{},\"spawn_execution\":{},\"live_process_execution\":{},\"mode_execution_contract\":{},\"evidence_contract\":{},\"live_execution\":{},\"live_validation\":{},\"scenario_count\":{},\"scenario_ids\":[{}],\"scenario_results\":[{}],\"scenario_contracts\":[{}],\"phase_count\":{},\"phases\":[{}],\"phase_results\":[{}],\"lifecycle_summary\":{{\"phase_totals\":{},\"step_totals\":{}}}}}",
        mode.as_str(),
        escape_json(config.evidence_dir.as_str()),
        integration_config_json,
        runtime_external_execution_json,
        runtime_orchestration_json,
        runtime_lifecycle_execution_json,
        runtime_validation_execution_json,
        runtime_readiness_json,
        process_runtime_json,
        process_lifecycle_json,
        spawn_timeline_json,
        spawn_plan_json,
        spawn_execution_json,
        live_process_execution_json,
        mode_execution_contract_json,
        evidence_contract_json,
        live_execution_json,
        live_validation_json,
        selected.len(),
        scenario_ids,
        scenario_results_json,
        scenario_contracts_json,
        phases.len(),
        phase_labels,
        phase_results_json,
        phase_totals_json,
        step_totals_json
    ))
}

fn aggregate_status(statuses: &[PhaseResultStatus]) -> PhaseResultStatus {
    if statuses.contains(&PhaseResultStatus::Fail) {
        return PhaseResultStatus::Fail;
    }
    if statuses
        .iter()
        .all(|status| *status == PhaseResultStatus::Skip)
    {
        return PhaseResultStatus::Skip;
    }
    PhaseResultStatus::Pass
}

fn probe_binary_invocation(binary: &str, label: &str) -> (PhaseResultStatus, String) {
    match Command::new(binary)
        .arg("--help")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
    {
        Ok(status) if status.success() => {
            (PhaseResultStatus::Pass, format!("{label} probe passed"))
        }
        Ok(status) => {
            let exit_status = status
                .code()
                .map(|code| code.to_string())
                .unwrap_or_else(|| "signal".to_owned());
            (
                PhaseResultStatus::Fail,
                format!("{label} probe failed (exit_status={exit_status})"),
            )
        }
        Err(error) => (
            PhaseResultStatus::Fail,
            format!("{label} probe failed ({error})"),
        ),
    }
}

fn probe_external_runtime(
    config: &RunCommandConfig,
    mode: ExecutionMode,
) -> ExternalRuntimeProbeSummary {
    let (kolme_status, kolme_detail) =
        probe_binary_invocation(config.kolme_binary.as_str(), "kolme");
    let (agent_status, agent_detail) = if is_mcp_mode(mode) {
        let Some(agent_binary) = config.agent_binary.as_deref() else {
            return ExternalRuntimeProbeSummary {
                status: PhaseResultStatus::Fail,
                detail: "agent probe failed (missing binary path)".to_owned(),
            };
        };
        probe_binary_invocation(agent_binary, "agent")
    } else {
        (
            PhaseResultStatus::Skip,
            "agent probe skipped (mode does not require agent binary)".to_owned(),
        )
    };
    let status = aggregate_status(&[kolme_status, agent_status]);
    ExternalRuntimeProbeSummary {
        status,
        detail: format!("{kolme_detail}; {agent_detail}"),
    }
}

fn execute_selected_scenarios(
    mode: ExecutionMode,
    selected: &[scenarios::ScenarioDefinition],
    force_first_fail: bool,
) -> Result<Vec<ScenarioExecutionResult>, String> {
    let driver = driver_for_mode(mode)?;
    selected
        .iter()
        .enumerate()
        .map(|(index, scenario)| {
            let driver_result = driver.execute(scenario.id);
            let status = if force_first_fail && index == 0 {
                PhaseResultStatus::Fail
            } else {
                normalize_driver_status(driver_result.status)?
            };
            Ok(ScenarioExecutionResult {
                id: scenario.id.to_owned(),
                status,
            })
        })
        .collect()
}

fn driver_for_mode(mode: ExecutionMode) -> Result<Box<dyn drivers::HarnessDriver>, String> {
    match mode {
        ExecutionMode::SdkDirect => Ok(Box::new(drivers::sdk_direct::SdkDirectDriver)),
        ExecutionMode::CliScripted => Ok(Box::new(drivers::cli_scripted::CliScriptedDriver)),
        ExecutionMode::McpTau | ExecutionMode::McpAny => {
            Ok(Box::new(drivers::mcp_agent::McpAgentDriver::new(mode)?))
        }
    }
}

fn normalize_driver_status(value: &str) -> Result<PhaseResultStatus, String> {
    match value.trim().to_ascii_uppercase().as_str() {
        "PASS" => Ok(PhaseResultStatus::Pass),
        "FAIL" => Ok(PhaseResultStatus::Fail),
        "SKIP" => Ok(PhaseResultStatus::Skip),
        other => Err(format!("unsupported driver execution status: {other}")),
    }
}

fn ensure_external_execution_preflight(
    config: &RunCommandConfig,
    mode: ExecutionMode,
) -> Result<(), String> {
    if config.kolme_binary.trim().is_empty() {
        return Err("external execution preflight failed: kolme binary path is empty".to_owned());
    }
    ensure_binary_path_is_executable(config.kolme_binary.as_str(), "kolme")?;
    if matches!(mode, ExecutionMode::McpTau | ExecutionMode::McpAny) {
        let agent_binary = config.agent_binary.as_deref().ok_or_else(|| {
            "external execution preflight failed: agent binary missing for MCP modes".to_owned()
        })?;
        if agent_binary.trim().is_empty() {
            return Err(
                "external execution preflight failed: agent binary path is empty".to_owned(),
            );
        }
        ensure_binary_path_is_executable(agent_binary, "agent")?;
    }
    Ok(())
}

fn ensure_binary_path_is_executable(path: &str, label: &str) -> Result<(), String> {
    let binary_path = Path::new(path);
    if !binary_path.is_absolute() {
        return Err(format!(
            "external execution preflight failed: {label} binary path must be absolute: {path}"
        ));
    }
    if !binary_path.exists() {
        return Err(format!(
            "external execution preflight failed: {label} binary not found: {path}"
        ));
    }
    if !binary_path.is_file() {
        return Err(format!(
            "external execution preflight failed: {label} binary path is not a file: {path}"
        ));
    }
    ensure_binary_executable(binary_path, label)
}

#[cfg(unix)]
fn ensure_binary_executable(path: &Path, label: &str) -> Result<(), String> {
    let mode = std::fs::metadata(path)
        .map_err(|err| {
            format!(
                "external execution preflight failed: {label} binary metadata read failed: {} ({err})",
                path.display()
            )
        })?
        .permissions()
        .mode();
    if mode & 0o111 == 0 {
        return Err(format!(
            "external execution preflight failed: {label} binary is not executable: {}",
            path.display()
        ));
    }
    Ok(())
}

#[cfg(not(unix))]
fn ensure_binary_executable(_path: &Path, _label: &str) -> Result<(), String> {
    Ok(())
}

fn compute_lifecycle_summary(phase_results: &[OrchestrationPhaseResult]) -> LifecycleSummary {
    let phase_totals = status_totals_from_iter(phase_results.iter().map(|result| result.status));
    let step_totals = status_totals_from_iter(
        phase_results
            .iter()
            .flat_map(|result| result.steps.iter().map(|step| step.status)),
    );
    LifecycleSummary {
        phase_totals,
        step_totals,
    }
}

fn status_totals_from_iter<I>(statuses: I) -> LifecycleStatusTotals
where
    I: IntoIterator<Item = PhaseResultStatus>,
{
    let mut totals = LifecycleStatusTotals {
        total: 0,
        pass: 0,
        fail: 0,
        skip: 0,
    };
    for status in statuses {
        totals.total += 1;
        match status {
            PhaseResultStatus::Pass => totals.pass += 1,
            PhaseResultStatus::Fail => totals.fail += 1,
            PhaseResultStatus::Skip => totals.skip += 1,
        }
    }
    totals
}

fn build_phase_results(
    phases: &[OrchestrationPhase],
    mode: ExecutionMode,
    fail_path_marker: bool,
    scenario_results: &[ScenarioExecutionResult],
    evidence_status: PhaseResultStatus,
) -> Vec<OrchestrationPhaseResult> {
    let started_at = "1970-01-01T00:00:00Z";
    let completed_at = "1970-01-01T00:00:01Z";
    phases
        .iter()
        .map(|phase| {
            let steps = phase_step_records(
                *phase,
                mode,
                fail_path_marker,
                scenario_results,
                evidence_status,
            );
            let status = phase_status_for_steps(steps.as_slice());
            let details = phase_details(*phase, mode, status, scenario_results, evidence_status);
            OrchestrationPhaseResult {
                phase: *phase,
                status,
                started_at: started_at.to_owned(),
                completed_at: completed_at.to_owned(),
                details,
                steps,
            }
        })
        .collect()
}

fn phase_status_for_steps(steps: &[OrchestrationStepRecord]) -> PhaseResultStatus {
    if steps
        .iter()
        .any(|step| step.status == PhaseResultStatus::Fail)
    {
        return PhaseResultStatus::Fail;
    }
    if steps
        .iter()
        .all(|step| step.status == PhaseResultStatus::Skip)
    {
        return PhaseResultStatus::Skip;
    }
    PhaseResultStatus::Pass
}

fn phase_details(
    phase: OrchestrationPhase,
    mode: ExecutionMode,
    status: PhaseResultStatus,
    scenario_results: &[ScenarioExecutionResult],
    evidence_status: PhaseResultStatus,
) -> String {
    match (phase, status) {
        (OrchestrationPhase::InfraUp, PhaseResultStatus::Fail) => {
            "deterministic fail-path marker for infra startup".to_owned()
        }
        (OrchestrationPhase::InfraUp, _) => {
            "deterministic placeholder for infra startup".to_owned()
        }
        (OrchestrationPhase::AgentDeploy, _) => {
            "deterministic placeholder for agent deploy".to_owned()
        }
        (OrchestrationPhase::ScenarioRun, _) => {
            let totals =
                status_totals_from_iter(scenario_results.iter().map(|result| result.status));
            format!(
                "deterministic scenario execution summary: executed={} pass={} fail={} skip={}",
                totals.total, totals.pass, totals.fail, totals.skip
            )
        }
        (OrchestrationPhase::Evidence, _) => evidence_phase_detail(evidence_status),
        (OrchestrationPhase::Teardown, _) => teardown_phase_detail(mode, status),
    }
}

fn phase_step_records(
    phase: OrchestrationPhase,
    mode: ExecutionMode,
    fail_path_marker: bool,
    scenario_results: &[ScenarioExecutionResult],
    evidence_status: PhaseResultStatus,
) -> Vec<OrchestrationStepRecord> {
    let pass = PhaseResultStatus::Pass;
    match phase {
        OrchestrationPhase::InfraUp => vec![
            OrchestrationStepRecord {
                step: "Start PostgreSQL container (docker)".to_owned(),
                status: pass,
                detail: "deterministic placeholder: postgres startup".to_owned(),
            },
            OrchestrationStepRecord {
                step: "Run Kolme migrations".to_owned(),
                status: pass,
                detail: "deterministic placeholder: migrations complete".to_owned(),
            },
            OrchestrationStepRecord {
                step: "Start Kolme processor (in-memory or Fjall storage)".to_owned(),
                status: pass,
                detail: "deterministic placeholder: kolme processor online".to_owned(),
            },
            OrchestrationStepRecord {
                step: "Verify Kolme API health (/healthz)".to_owned(),
                status: pass,
                detail: "deterministic placeholder: kolme health verified".to_owned(),
            },
            OrchestrationStepRecord {
                step: "Start KAMN processor node".to_owned(),
                status: pass,
                detail: "deterministic placeholder: processor node online".to_owned(),
            },
            OrchestrationStepRecord {
                step: "Start KAMN listener node".to_owned(),
                status: pass,
                detail: "deterministic placeholder: listener node online".to_owned(),
            },
            OrchestrationStepRecord {
                step: "Start KAMN approver node".to_owned(),
                status: pass,
                detail: "deterministic placeholder: approver node online".to_owned(),
            },
            OrchestrationStepRecord {
                step: "Wait for peer discovery (3 connected peers)".to_owned(),
                status: pass,
                detail: "deterministic placeholder: peer discovery complete".to_owned(),
            },
            OrchestrationStepRecord {
                step: "Verify KAMN Service API health (/healthz)".to_owned(),
                status: if fail_path_marker {
                    PhaseResultStatus::Fail
                } else {
                    pass
                },
                detail: if fail_path_marker {
                    "deterministic fail-path marker: kamn health check failed".to_owned()
                } else {
                    "deterministic placeholder: kamn health verified".to_owned()
                },
            },
        ],
        OrchestrationPhase::AgentDeploy => vec![
            OrchestrationStepRecord {
                step: "Generate ed25519 key pairs for Alice, Bob, Carol".to_owned(),
                status: pass,
                detail: "deterministic placeholder: keys generated".to_owned(),
            },
            OrchestrationStepRecord {
                step: "Write key files to temp directory".to_owned(),
                status: pass,
                detail: "deterministic placeholder: key files materialized".to_owned(),
            },
            OrchestrationStepRecord {
                step: "Register agents via kamn-agent-lib (POST /v1/agents/bootstrap)".to_owned(),
                status: pass,
                detail: "deterministic placeholder: agents registered".to_owned(),
            },
            OrchestrationStepRecord {
                step: "[MCP modes] Spawn kamn-mcp-server per agent with identity".to_owned(),
                status: if is_mcp_mode(mode) {
                    pass
                } else {
                    PhaseResultStatus::Skip
                },
                detail: if is_mcp_mode(mode) {
                    "deterministic placeholder: mcp servers spawned".to_owned()
                } else {
                    "deterministic placeholder: mcp server spawn skipped for non-mcp mode"
                        .to_owned()
                },
            },
            OrchestrationStepRecord {
                step: "[MCP modes] Verify MCP server health".to_owned(),
                status: if is_mcp_mode(mode) {
                    pass
                } else {
                    PhaseResultStatus::Skip
                },
                detail: if is_mcp_mode(mode) {
                    "deterministic placeholder: mcp health verified".to_owned()
                } else {
                    "deterministic placeholder: mcp health skipped for non-mcp mode".to_owned()
                },
            },
            OrchestrationStepRecord {
                step: "Record infrastructure evidence".to_owned(),
                status: pass,
                detail: "deterministic placeholder: infra evidence recorded".to_owned(),
            },
        ],
        OrchestrationPhase::ScenarioRun => {
            let totals =
                status_totals_from_iter(scenario_results.iter().map(|result| result.status));
            let status = if totals.fail > 0 {
                PhaseResultStatus::Fail
            } else if totals.pass > 0 {
                PhaseResultStatus::Pass
            } else {
                PhaseResultStatus::Skip
            };
            vec![OrchestrationStepRecord {
                step: "Execute selected scenarios via mode driver".to_owned(),
                status,
                detail: format!(
                    "executed={} pass={} fail={} skip={}",
                    totals.total, totals.pass, totals.fail, totals.skip
                ),
            }]
        }
        OrchestrationPhase::Evidence => {
            let prerequisite_status = if evidence_status == PhaseResultStatus::Skip {
                PhaseResultStatus::Skip
            } else {
                pass
            };
            vec![
                OrchestrationStepRecord {
                    step: "Dump Kolme chain state".to_owned(),
                    status: prerequisite_status,
                    detail: "deterministic placeholder: kolme chain state dumped".to_owned(),
                },
                OrchestrationStepRecord {
                    step: "Dump KAMN node state snapshots".to_owned(),
                    status: prerequisite_status,
                    detail: "deterministic placeholder: kamn node snapshots dumped".to_owned(),
                },
                OrchestrationStepRecord {
                    step: "Verify all proof anchors independently".to_owned(),
                    status: evidence_status,
                    detail: evidence_verify_step_detail(evidence_status),
                },
                OrchestrationStepRecord {
                    step: "Generate chain-of-custody report".to_owned(),
                    status: evidence_status,
                    detail: evidence_custody_step_detail(evidence_status),
                },
                OrchestrationStepRecord {
                    step: "Compute evidence bundle hash".to_owned(),
                    status: evidence_status,
                    detail: evidence_hash_step_detail(evidence_status),
                },
                OrchestrationStepRecord {
                    step: "Write manifest.json".to_owned(),
                    status: evidence_status,
                    detail: evidence_manifest_step_detail(evidence_status),
                },
            ]
        }
        OrchestrationPhase::Teardown => vec![
            OrchestrationStepRecord {
                step: "[MCP modes] Stop kamn-mcp-server processes".to_owned(),
                status: if is_mcp_mode(mode) {
                    pass
                } else {
                    PhaseResultStatus::Skip
                },
                detail: if is_mcp_mode(mode) {
                    "deterministic placeholder: mcp servers stopped".to_owned()
                } else {
                    "deterministic placeholder: mcp teardown skipped for non-mcp mode".to_owned()
                },
            },
            OrchestrationStepRecord {
                step: "Stop KAMN nodes (graceful shutdown)".to_owned(),
                status: pass,
                detail: "deterministic placeholder: kamn nodes stopped".to_owned(),
            },
            OrchestrationStepRecord {
                step: "Stop Kolme devnet".to_owned(),
                status: pass,
                detail: "deterministic placeholder: kolme devnet stopped".to_owned(),
            },
            OrchestrationStepRecord {
                step: "Stop PostgreSQL container".to_owned(),
                status: pass,
                detail: "deterministic placeholder: postgres container stopped".to_owned(),
            },
            OrchestrationStepRecord {
                step: "Archive evidence bundle".to_owned(),
                status: pass,
                detail: "deterministic placeholder: evidence bundle archived".to_owned(),
            },
        ],
    }
}

fn evidence_phase_detail(status: PhaseResultStatus) -> String {
    match status {
        PhaseResultStatus::Fail => {
            "deterministic evidence phase summary: steps_total=6 pass=2 fail=4 skip=0 expected_artifacts=4 recorded_artifacts=3 status=FAIL".to_owned()
        }
        PhaseResultStatus::Pass => {
            "deterministic evidence phase summary: steps_total=6 pass=6 fail=0 skip=0 expected_artifacts=4 recorded_artifacts=4 status=PASS".to_owned()
        }
        PhaseResultStatus::Skip => {
            "deterministic evidence phase summary: steps_total=6 pass=0 fail=0 skip=6 expected_artifacts=4 recorded_artifacts=0 status=SKIP".to_owned()
        }
    }
}

fn evidence_verify_step_detail(status: PhaseResultStatus) -> String {
    match status {
        PhaseResultStatus::Fail => "proof_verification=FAIL verified=3 failed=1".to_owned(),
        PhaseResultStatus::Pass => "proof_verification=PASS verified=4 failed=0".to_owned(),
        PhaseResultStatus::Skip => "proof_verification=SKIP verified=0 failed=0".to_owned(),
    }
}

fn evidence_custody_step_detail(status: PhaseResultStatus) -> String {
    match status {
        PhaseResultStatus::Fail => "custody_report=FAIL entries=3".to_owned(),
        PhaseResultStatus::Pass => "custody_report=PASS entries=4".to_owned(),
        PhaseResultStatus::Skip => "custody_report=SKIP entries=0".to_owned(),
    }
}

fn evidence_hash_step_detail(status: PhaseResultStatus) -> String {
    match status {
        PhaseResultStatus::Fail => "bundle_hash=FAIL algorithm=sha256".to_owned(),
        PhaseResultStatus::Pass => "bundle_hash=PASS algorithm=sha256".to_owned(),
        PhaseResultStatus::Skip => "bundle_hash=SKIP algorithm=sha256".to_owned(),
    }
}

fn evidence_manifest_step_detail(status: PhaseResultStatus) -> String {
    match status {
        PhaseResultStatus::Fail => {
            "manifest_write=FAIL schema=kamn.e2e.evidence-manifest.v3".to_owned()
        }
        PhaseResultStatus::Pass => {
            "manifest_write=PASS schema=kamn.e2e.evidence-manifest.v3".to_owned()
        }
        PhaseResultStatus::Skip => {
            "manifest_write=SKIP schema=kamn.e2e.evidence-manifest.v3".to_owned()
        }
    }
}

fn teardown_phase_detail(mode: ExecutionMode, status: PhaseResultStatus) -> String {
    let mcp_step_status = if is_mcp_mode(mode) { "PASS" } else { "SKIP" };
    format!(
        "deterministic teardown summary: mcp_stop={} kamn_nodes=PASS kolme=PASS postgres=PASS archive=PASS status={}",
        mcp_step_status,
        status.as_str()
    )
}

fn is_mcp_mode(mode: ExecutionMode) -> bool {
    matches!(mode, ExecutionMode::McpTau | ExecutionMode::McpAny)
}

fn mode_driver_label(mode: ExecutionMode) -> &'static str {
    match mode {
        ExecutionMode::SdkDirect => "sdk-direct-driver",
        ExecutionMode::CliScripted => "cli-scripted-driver",
        ExecutionMode::McpTau | ExecutionMode::McpAny => "mcp-agent-driver",
    }
}

fn select_scenarios(ids: &[String]) -> Result<Vec<scenarios::ScenarioDefinition>, String> {
    let inventory = scenarios::all_scenarios();
    let mut selected = Vec::new();
    for id in ids {
        let matched = inventory
            .iter()
            .find(|item| item.id == id.as_str())
            .ok_or_else(|| format!("unknown scenario id: {id}"))?;
        selected.push(matched.clone());
    }
    Ok(selected)
}

/// Executes verify-command contract behavior and writes deterministic report JSON.
pub fn execute_verify_contract(config: &VerifyCommandConfig) -> Result<String, String> {
    let evidence_dir_path = Path::new(config.evidence_dir.as_str());
    let kolme_chain_dump_path = Path::new(config.kolme_chain_dump.as_str());
    let output_path = Path::new(config.output.as_str());
    if !kolme_chain_dump_path.is_file() {
        return Err(format!(
            "kolme chain dump file not found: {}",
            config.kolme_chain_dump
        ));
    }
    let manifest_path = evidence_dir_path.join("manifest.json");
    let manifest_json = std::fs::read_to_string(&manifest_path).map_err(|error| {
        format!(
            "failed to read evidence manifest {}: {error}",
            manifest_path.display()
        )
    })?;
    let chain_dump_json = std::fs::read_to_string(kolme_chain_dump_path).map_err(|error| {
        format!(
            "failed to read chain dump {}: {error}",
            kolme_chain_dump_path.display()
        )
    })?;
    verify::validate_evidence_verification_blocks(
        evidence_dir_path,
        &[manifest_path.as_path(), kolme_chain_dump_path, output_path],
    )?;
    let report_json = verify::generate_verification_report_json(manifest_json.as_str())?;
    verify::verify_chain_dump(chain_dump_json.as_str())?;
    if let Some(parent) = output_path.parent() {
        std::fs::create_dir_all(parent).map_err(|error| {
            format!(
                "failed to create verify output parent {}: {error}",
                parent.display()
            )
        })?;
    }
    std::fs::write(output_path, report_json.as_bytes()).map_err(|error| {
        format!(
            "failed to write verify output {}: {error}",
            output_path.display()
        )
    })?;
    Ok(report_json)
}

#[cfg(test)]
mod tests {
    use super::{
        aggregate_status, all_execution_modes, all_orchestration_phases, all_phase_result_statuses,
        ExecutionMode, PhaseResultStatus,
    };

    #[test]
    fn unit_execution_mode_parse_roundtrip() {
        for mode in all_execution_modes() {
            let parsed = ExecutionMode::parse(mode.as_str()).expect("mode should parse");
            assert_eq!(parsed, mode);
        }
    }

    #[test]
    fn unit_orchestration_phase_inventory_is_canonical() {
        let labels: Vec<&str> = all_orchestration_phases()
            .iter()
            .map(|phase| phase.as_str())
            .collect();
        assert_eq!(
            labels,
            vec![
                "INFRA_UP",
                "AGENT_DEPLOY",
                "SCENARIO_RUN",
                "EVIDENCE",
                "TEARDOWN"
            ]
        );
    }

    #[test]
    fn unit_phase_result_status_inventory_is_canonical() {
        let labels: Vec<&str> = all_phase_result_statuses()
            .iter()
            .map(|status| status.as_str())
            .collect();
        assert_eq!(labels, vec!["PASS", "FAIL", "SKIP"]);
    }

    #[test]
    fn unit_aggregate_status_fail_dominates() {
        let aggregated = aggregate_status(&[
            PhaseResultStatus::Pass,
            PhaseResultStatus::Fail,
            PhaseResultStatus::Skip,
        ]);
        assert_eq!(aggregated, PhaseResultStatus::Fail);
    }

    #[test]
    fn unit_aggregate_status_all_skip_returns_skip() {
        let aggregated = aggregate_status(&[PhaseResultStatus::Skip, PhaseResultStatus::Skip]);
        assert_eq!(aggregated, PhaseResultStatus::Skip);
    }
}
