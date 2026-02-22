use crate::ExecutionMode;

/// CLI-scripted mode driver.
pub mod cli_scripted;
/// MCP-agent mode driver.
pub mod mcp_agent;
/// SDK-direct mode driver.
pub mod sdk_direct;

/// Driver result emitted for one scenario execution.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DriverExecutionResult {
    /// Scenario identifier.
    pub scenario_id: &'static str,
    /// Deterministic status marker.
    pub status: &'static str,
}

/// Common driver trait.
pub trait HarnessDriver {
    /// Returns the bound execution mode.
    fn mode(&self) -> ExecutionMode;

    /// Executes one scenario in deterministic scaffold mode.
    fn execute(&self, scenario_id: &'static str) -> DriverExecutionResult;
}
