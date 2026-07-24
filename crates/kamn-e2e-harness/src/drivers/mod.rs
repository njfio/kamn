use crate::ExecutionMode;

mod authoritative_settlement_observation;
/// CLI-scripted mode driver.
pub mod cli_scripted;
/// MCP-agent mode driver.
pub mod mcp_agent;
/// SDK-direct mode driver.
pub mod sdk_direct;
pub use authoritative_settlement_observation::{
    normalize_authoritative_settlement, AuthoritativeSettlementObservation,
    AuthoritativeSettlementReplayGuard,
};
/// Shared helper surface for duplicated driver internals.
pub(crate) mod shared_helpers;

/// Driver result emitted for one scenario execution.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DriverExecutionResult {
    /// Scenario identifier.
    pub scenario_id: &'static str,
    /// Deterministic status marker.
    pub status: &'static str,
    /// Optional failure detail preserved from the underlying probe.
    pub detail: Option<String>,
}

pub(crate) fn passing_driver_result(scenario_id: &'static str) -> DriverExecutionResult {
    DriverExecutionResult {
        scenario_id,
        status: "pass",
        detail: None,
    }
}

pub(crate) fn failing_driver_result(
    scenario_id: &'static str,
    detail: Option<String>,
) -> DriverExecutionResult {
    DriverExecutionResult {
        scenario_id,
        status: "fail",
        detail,
    }
}

pub(crate) fn live_probe_driver_result<F>(
    scenario_id: &'static str,
    live_bound: bool,
    live_execution_enabled: bool,
    probe_result: F,
) -> DriverExecutionResult
where
    F: FnOnce() -> Option<Result<(), String>>,
{
    if !live_bound {
        return passing_driver_result(scenario_id);
    }
    if !live_execution_enabled {
        return failing_driver_result(scenario_id, None);
    }
    match probe_result() {
        Some(Ok(())) => passing_driver_result(scenario_id),
        Some(Err(error)) => failing_driver_result(scenario_id, Some(error)),
        None => failing_driver_result(scenario_id, None),
    }
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
