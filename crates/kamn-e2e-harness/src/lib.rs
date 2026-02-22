#![warn(missing_docs)]
//! E2E harness scaffold crate.

use std::collections::HashSet;
use std::path::Path;

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
            ExecutionMode::parse(mode.as_str())?;
            let evidence_dir =
                evidence_dir.ok_or_else(|| "missing required flag --evidence-dir".to_owned())?;
            let scenarios_csv =
                scenarios_csv.ok_or_else(|| "missing required flag --scenarios".to_owned())?;
            let scenario_ids = parse_scenario_csv(scenarios_csv.as_str())?;
            Ok(HarnessCommand::Run(RunCommandConfig {
                mode,
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
    let selected = select_scenarios(config.scenario_ids.as_slice())?;
    let scenario_ids = selected
        .iter()
        .map(|item| format!("\"{}\"", item.id))
        .collect::<Vec<_>>()
        .join(",");
    Ok(format!(
        "{{\"command\":\"run\",\"mode\":\"{}\",\"evidence_dir\":\"{}\",\"scenario_count\":{},\"scenario_ids\":[{}]}}",
        mode.as_str(),
        escape_json(config.evidence_dir.as_str()),
        selected.len(),
        scenario_ids
    ))
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
    if !Path::new(config.kolme_chain_dump.as_str()).is_file() {
        return Err(format!(
            "kolme chain dump file not found: {}",
            config.kolme_chain_dump
        ));
    }
    let manifest_path = Path::new(config.evidence_dir.as_str()).join("manifest.json");
    let manifest_json = std::fs::read_to_string(&manifest_path).map_err(|error| {
        format!(
            "failed to read evidence manifest {}: {error}",
            manifest_path.display()
        )
    })?;
    let report_json = verify::generate_verification_report_json(manifest_json.as_str())?;
    if let Some(parent) = Path::new(config.output.as_str()).parent() {
        std::fs::create_dir_all(parent).map_err(|error| {
            format!(
                "failed to create verify output parent {}: {error}",
                parent.display()
            )
        })?;
    }
    std::fs::write(config.output.as_str(), report_json.as_bytes()).map_err(|error| {
        format!(
            "failed to write verify output {}: {error}",
            Path::new(config.output.as_str()).display()
        )
    })?;
    Ok(report_json)
}

#[cfg(test)]
mod tests {
    use super::{all_execution_modes, ExecutionMode};

    #[test]
    fn unit_execution_mode_parse_roundtrip() {
        for mode in all_execution_modes() {
            let parsed = ExecutionMode::parse(mode.as_str()).expect("mode should parse");
            assert_eq!(parsed, mode);
        }
    }
}
