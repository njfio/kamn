#![warn(missing_docs)]
//! E2E harness scaffold crate.

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
