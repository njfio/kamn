use super::{
    is_live_bound_scenario_id, run_live_s01_mcp_probe, run_live_s02_mcp_direct_message_probe,
    run_live_s03_mcp_group_channel_probe, run_live_s04_mcp_task_lifecycle_probe,
    run_live_s05_mcp_escrow_settlement_probe, run_live_s06_mcp_proof_verification_probe,
    run_live_s07_mcp_replay_protection_probe, run_live_s08_mcp_crash_recovery_probe,
    run_live_s09_mcp_transport_failover_probe, run_live_s10_mcp_topology_coherence_probe,
    run_live_s11_mcp_signer_rotation_probe, run_live_s12_mcp_retention_deletion_probe,
    run_live_s13_mcp_bridge_forwarding_probe, run_live_s14_mcp_batch_merkle_probe,
    run_live_s15_mcp_performance_smoke_probe,
    runner_registry::{explicit_runner_map, shared_runner_map, LiveRunnerMap},
};
use crate::drivers::{DriverExecutionResult, HarnessDriver};
use crate::ExecutionMode;
use std::sync::Arc;

pub trait McpRunner: Fn() -> Result<(), String> + Send + Sync + 'static {}
impl<T> McpRunner for T where T: Fn() -> Result<(), String> + Send + Sync + 'static {}

#[derive(Clone)]
/// MCP-agent driver with optional live execution for S-01 through S-15.
pub struct McpAgentDriver {
    mode: ExecutionMode,
    live_execution_enabled: bool,
    live_runners: LiveRunnerMap,
}

impl std::fmt::Debug for McpAgentDriver {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("McpAgentDriver")
            .field("mode", &self.mode)
            .field("live_execution_enabled", &self.live_execution_enabled)
            .finish()
    }
}

impl McpAgentDriver {
    /// Creates deterministic MCP driver instance with live mode disabled.
    pub fn new(mode: ExecutionMode) -> Result<Self, String> {
        Self::with_probe(mode, false, || Ok(()))
    }

    /// Creates MCP driver with environment-driven live toggle.
    pub fn from_env(mode: ExecutionMode) -> Result<Self, String> {
        Self::with_runners(
            mode,
            live_execution_enabled_from_env(),
            run_live_s01_mcp_probe,
            run_live_s02_mcp_direct_message_probe,
            run_live_s03_mcp_group_channel_probe,
            run_live_s04_mcp_task_lifecycle_probe,
            run_live_s05_mcp_escrow_settlement_probe,
            (
                run_live_s06_mcp_proof_verification_probe,
                run_live_s07_mcp_replay_protection_probe,
                run_live_s08_mcp_crash_recovery_probe,
                run_live_s09_mcp_transport_failover_probe,
                run_live_s10_mcp_topology_coherence_probe,
                run_live_s11_mcp_signer_rotation_probe,
                run_live_s12_mcp_retention_deletion_probe,
                run_live_s13_mcp_bridge_forwarding_probe,
                run_live_s14_mcp_batch_merkle_probe,
                run_live_s15_mcp_performance_smoke_probe,
            ),
        )
    }

    /// Creates MCP driver with one probe reused for all live-bound scenarios.
    pub fn with_probe<F: McpRunner>(
        mode: ExecutionMode,
        live_execution_enabled: bool,
        live_probe: F,
    ) -> Result<Self, String> {
        validate_mode(mode)?;
        Ok(Self::from_runner_map(
            mode,
            live_execution_enabled,
            shared_runner_map(Arc::new(live_probe)),
        ))
    }

    #[rustfmt::skip]
    /// Creates MCP driver with explicit per-scenario probe implementations.
    pub fn with_runners<F: McpRunner, G: McpRunner, H: McpRunner, I: McpRunner, J: McpRunner, K: McpRunner, L: McpRunner, M: McpRunner, N: McpRunner, O: McpRunner, P: McpRunner, Q: McpRunner, R: McpRunner, S: McpRunner, T: McpRunner>(
        mode: ExecutionMode, live_execution_enabled: bool, discovery_probe: F, direct_message_probe: G, group_channel_probe: H, task_lifecycle_probe: I, escrow_probe: J, trailing_probes: (K, L, M, N, O, P, Q, R, S, T),
    ) -> Result<Self, String> {
        validate_mode(mode)?;
        let (proof, replay, crash, transport, topology, signer, retention, bridge, merkle, performance) = trailing_probes;
        Ok(Self::from_runner_map(mode, live_execution_enabled, explicit_runner_map([
            Arc::new(discovery_probe), Arc::new(direct_message_probe), Arc::new(group_channel_probe), Arc::new(task_lifecycle_probe), Arc::new(escrow_probe),
            Arc::new(proof), Arc::new(replay), Arc::new(crash), Arc::new(transport), Arc::new(topology),
            Arc::new(signer), Arc::new(retention), Arc::new(bridge), Arc::new(merkle), Arc::new(performance),
        ])))
    }

    fn from_runner_map(
        mode: ExecutionMode,
        live_execution_enabled: bool,
        live_runners: LiveRunnerMap,
    ) -> Self {
        Self {
            mode,
            live_execution_enabled,
            live_runners,
        }
    }

    fn live_probe_for_scenario(&self, scenario_id: &'static str) -> Option<Result<(), String>> {
        self.live_runners
            .get(scenario_id)
            .map(|runner| runner.as_ref()())
    }
}

impl HarnessDriver for McpAgentDriver {
    fn mode(&self) -> ExecutionMode {
        self.mode
    }

    fn execute(&self, scenario_id: &'static str) -> DriverExecutionResult {
        DriverExecutionResult {
            scenario_id,
            status: status_for_scenario(self, scenario_id),
        }
    }
}

pub(crate) fn live_execution_enabled_from_env() -> bool {
    super::shared_live_execution_enabled_from_env(super::MCP_AGENT_LIVE_ENV)
}

fn validate_mode(mode: ExecutionMode) -> Result<(), String> {
    if matches!(mode, ExecutionMode::McpTau | ExecutionMode::McpAny) {
        return Ok(());
    }
    Err("McpAgentDriver requires mcp-tau or mcp-any mode".to_owned())
}

fn status_for_scenario(driver: &McpAgentDriver, scenario_id: &'static str) -> &'static str {
    if !is_live_bound_scenario_id(scenario_id) {
        return "pass";
    }
    if !driver.live_execution_enabled {
        return "fail";
    }
    match driver.live_probe_for_scenario(scenario_id) {
        Some(result) if result.is_ok() => "pass",
        Some(_) | None => "fail",
    }
}
