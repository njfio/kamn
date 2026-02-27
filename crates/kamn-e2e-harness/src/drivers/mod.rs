use crate::ExecutionMode;

/// CLI-scripted mode driver.
pub mod cli_scripted;
/// MCP-agent mode driver.
pub mod mcp_agent;
/// SDK-direct mode driver.
pub mod sdk_direct;
/// Shared internal helpers used by multiple E2E drivers.
pub(crate) mod shared_helpers;

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

/// Process-wide lock used to serialize environment-variable access.
///
/// Driver unit tests mutate `KAMN_E2E_*` vars and production/runtime contract code
/// reads those vars. Guarding both sides with one lock prevents cross-test
/// contamination when integration tests execute in parallel.
pub(crate) fn test_env_lock() -> &'static std::sync::Mutex<()> {
    use std::sync::{Mutex, OnceLock};

    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
}
