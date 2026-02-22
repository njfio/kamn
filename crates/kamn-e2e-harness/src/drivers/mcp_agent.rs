use crate::drivers::{DriverExecutionResult, HarnessDriver};
use crate::ExecutionMode;

/// MCP-agent driver scaffold for both Tau and generic MCP runtimes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct McpAgentDriver {
    mode: ExecutionMode,
}

impl McpAgentDriver {
    /// Creates an MCP driver for a specific MCP execution mode.
    pub fn new(mode: ExecutionMode) -> Result<Self, String> {
        if matches!(mode, ExecutionMode::McpTau | ExecutionMode::McpAny) {
            return Ok(Self { mode });
        }
        Err("McpAgentDriver requires mcp-tau or mcp-any mode".to_owned())
    }
}

impl HarnessDriver for McpAgentDriver {
    fn mode(&self) -> ExecutionMode {
        self.mode
    }

    fn execute(&self, scenario_id: &'static str) -> DriverExecutionResult {
        DriverExecutionResult {
            scenario_id,
            status: "pass",
        }
    }
}
