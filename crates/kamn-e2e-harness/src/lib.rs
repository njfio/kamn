#![warn(missing_docs)]
//! E2E harness scaffold crate.

use std::collections::HashSet;
use std::path::Path;

pub use agent_transaction_demo::{
    execute_agent_transaction_demo_contract, parse_agent_transaction_demo_config,
    AgentTransactionDemoConfig,
};
pub use agent_transaction_pi_command::{build_pi_actor_command, AgentTransactionRole};
pub use mvp_demo::{
    build_runtime_receipt_chain_from_actor_paths, execute_mvp_demo_contract,
    execute_verify_mvp_demo_contract, verify_pi_transaction_actor_paths, LiveTaskEvidencePaths,
    MvpDemoCommandConfig, VerifyMvpDemoCommandConfig,
};

mod agent_transaction_demo;
mod agent_transaction_pi_command;

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
/// MVP evaluator demo report contracts.
pub mod mvp_demo;
mod run_contract;
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
    /// Generate the MVP evaluator demo proof report.
    DemoMvp(Box<MvpDemoCommandConfig>),
    /// Verify an MVP evaluator demo proof report.
    VerifyMvpDemo(VerifyMvpDemoCommandConfig),
    /// Run the canonical three-agent devnet transaction demo.
    DemoAgentTransaction,
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
        "demo-mvp" => parse_demo_mvp_command(args.as_slice()),
        "demo-agent-transaction" => {
            if args.len() != 1 {
                return Err("demo-agent-transaction does not accept flags".to_owned());
            }
            Ok(HarnessCommand::DemoAgentTransaction)
        }
        "verify-mvp-demo" => parse_verify_mvp_demo_command(args.as_slice()),
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

fn parse_demo_mvp_command(args: &[String]) -> Result<HarnessCommand, String> {
    let mut output_root = None;
    let mut agent_harness_evidence_path = None;
    let mut index = 1;
    while index < args.len() {
        let (parsed_output_root, advanced) = parse_flag_value(args, index, "--output-root")?;
        if let Some(value) = parsed_output_root {
            output_root = Some(value);
            index = advanced + 1;
            continue;
        }
        let (parsed_agent_harness, advanced) =
            parse_flag_value(args, index, "--agent-harness-evidence")?;
        if let Some(value) = parsed_agent_harness {
            agent_harness_evidence_path = Some(value);
            index = advanced + 1;
            continue;
        }
        return Err(format!("unknown demo-mvp flag: {}", args[index]));
    }
    Ok(HarnessCommand::DemoMvp(Box::new(MvpDemoCommandConfig {
        output_root: output_root
            .unwrap_or_else(|| mvp_demo::DEFAULT_MVP_DEMO_OUTPUT_ROOT.to_owned()),
        devnet_mode: mvp_devnet_mode_from_env(),
        solana_rpc_url: mvp_solana_rpc_url_from_env(),
        devnet_settlement_command: None,
        localhost_signed_demo_command: None,
        service_api_vertical_slice_command: None,
        service_api_websocket_command: None,
        agent_harness_evidence_path: agent_harness_evidence_path
            .or_else(mvp_agent_harness_evidence_path_from_env),
        live_task_evidence: mvp_live_task_evidence_from_env()?,
        pi_transaction_actor_paths: mvp_pi_transaction_actor_paths_from_env()?,
    })))
}

fn mvp_pi_transaction_actor_paths_from_env() -> Result<Option<[String; 3]>, String> {
    complete_pi_actor_paths([
        std::env::var("KAMN_MVP_PI_TRANSACTION_AGENT_A_FILE").ok(),
        std::env::var("KAMN_MVP_PI_TRANSACTION_AGENT_B_FILE").ok(),
        std::env::var("KAMN_MVP_PI_TRANSACTION_AGENT_C_FILE").ok(),
    ])
}

fn parse_verify_mvp_demo_command(args: &[String]) -> Result<HarnessCommand, String> {
    let parsed = parse_verify_mvp_demo_args(args)?;
    let report = parsed
        .report
        .ok_or_else(|| "missing required flag --report".to_owned())?;
    Ok(HarnessCommand::VerifyMvpDemo(VerifyMvpDemoCommandConfig {
        report,
        agent_harness_evidence_path: parsed.agent_harness,
        pi_transaction_actor_paths: complete_pi_actor_paths(parsed.pi_actors)?,
    }))
}

struct VerifyMvpDemoArgs {
    report: Option<String>,
    agent_harness: Option<String>,
    pi_actors: [Option<String>; 3],
}

fn parse_verify_mvp_demo_args(args: &[String]) -> Result<VerifyMvpDemoArgs, String> {
    let mut parsed = VerifyMvpDemoArgs {
        report: None,
        agent_harness: None,
        pi_actors: [None, None, None],
    };
    let mut index = 1;
    while index < args.len() {
        index = parse_verify_mvp_demo_flag(args, index, &mut parsed)?;
    }
    Ok(parsed)
}

fn parse_verify_mvp_demo_flag(
    args: &[String],
    index: usize,
    parsed: &mut VerifyMvpDemoArgs,
) -> Result<usize, String> {
    for (flag, target) in [
        ("--report", &mut parsed.report),
        ("--agent-harness-evidence", &mut parsed.agent_harness),
    ] {
        let (value, advanced) = parse_flag_value(args, index, flag)?;
        if value.is_some() {
            *target = value;
            return Ok(advanced + 1);
        }
    }
    parse_pi_actor_flag(args, index, &mut parsed.pi_actors)
}

fn parse_pi_actor_flag(
    args: &[String],
    index: usize,
    paths: &mut [Option<String>; 3],
) -> Result<usize, String> {
    for (slot, flag) in [
        "--pi-agent-a-evidence",
        "--pi-agent-b-evidence",
        "--pi-agent-c-evidence",
    ]
    .iter()
    .enumerate()
    {
        let (value, advanced) = parse_flag_value(args, index, flag)?;
        if value.is_some() {
            paths[slot] = value;
            return Ok(advanced + 1);
        }
    }
    Err(format!("unknown verify-mvp-demo flag: {}", args[index]))
}

fn complete_pi_actor_paths(paths: [Option<String>; 3]) -> Result<Option<[String; 3]>, String> {
    if paths.iter().all(Option::is_none) {
        return Ok(None);
    }
    let [a, b, c] = paths;
    match (a, b, c) {
        (Some(a), Some(b), Some(c)) => Ok(Some([a, b, c])),
        _ => {
            Err("verify-mvp-demo requires all three Pi transaction actor evidence paths".to_owned())
        }
    }
}

fn mvp_devnet_mode_from_env() -> String {
    match std::env::var_os("KAMN_MVP_DEVNET_MODE") {
        Some(value) => mvp_devnet_mode_from_os_value(value),
        None => default_mvp_devnet_mode(),
    }
}

fn mvp_devnet_mode_from_os_value(value: std::ffi::OsString) -> String {
    match value.into_string() {
        Ok(text) if !text.trim().is_empty() => text,
        Ok(_) => default_mvp_devnet_mode(),
        Err(_) => "invalid-nonunicode".to_owned(),
    }
}

fn default_mvp_devnet_mode() -> String {
    "optional".to_owned()
}

fn mvp_solana_rpc_url_from_env() -> Option<String> {
    std::env::var("KAMN_MVP_SOLANA_RPC_URL")
        .ok()
        .filter(|value| !value.trim().is_empty())
}

fn mvp_agent_harness_evidence_path_from_env() -> Option<String> {
    std::env::var("KAMN_MVP_AGENT_HARNESS_EVIDENCE")
        .ok()
        .filter(|value| !value.trim().is_empty())
}

fn mvp_live_task_evidence_from_env() -> Result<Option<LiveTaskEvidencePaths>, String> {
    let values = [
        env_value("KAMN_MVP_LIVE_TASK_HANDOFF_FILE"),
        env_value("KAMN_MVP_LIVE_TASK_AGENT_A_RECEIPT_FILE"),
        env_value("KAMN_MVP_LIVE_TASK_AGENT_B_RECEIPT_FILE"),
        env_value("KAMN_MVP_LIVE_TASK_AGENT_C_OBSERVATION_FILE"),
    ];
    if values.iter().all(Option::is_none) {
        return Ok(None);
    }
    if values.iter().any(Option::is_none) {
        return Err(
            "live task evidence configuration must provide all four artifact paths".to_owned(),
        );
    }
    Ok(Some(LiveTaskEvidencePaths {
        handoff: values[0].clone().expect("all values checked"),
        agent_a_receipt: values[1].clone().expect("all values checked"),
        agent_b_receipt: values[2].clone().expect("all values checked"),
        agent_c_observation: values[3].clone().expect("all values checked"),
    }))
}

fn env_value(name: &str) -> Option<String> {
    std::env::var(name)
        .ok()
        .map(|value| value.trim().to_owned())
        .filter(|value| !value.is_empty())
}

pub use run_contract::execute_run_contract;

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
        all_execution_modes, all_orchestration_phases, all_phase_result_statuses,
        run_contract::aggregate_status, ExecutionMode, PhaseResultStatus,
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
