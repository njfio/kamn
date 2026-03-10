use super::{
    is_live_bound_scenario_id, run_live_s01_cli_health_probe,
    run_live_s02_cli_direct_message_probe, run_live_s03_cli_group_channel_probe,
    run_live_s04_cli_task_lifecycle_probe, run_live_s05_cli_escrow_settlement_probe,
    run_live_s06_cli_proof_verification_probe, run_live_s07_cli_replay_protection_probe,
    run_live_s08_cli_crash_recovery_probe, run_live_s09_cli_transport_failover_probe,
    run_live_s10_cli_topology_coherence_probe, run_live_s11_cli_signer_rotation_probe,
    run_live_s12_cli_retention_deletion_probe, run_live_s13_cli_bridge_forwarding_probe,
    run_live_s14_cli_batch_merkle_probe, run_live_s15_cli_performance_smoke_probe,
    runner_registry::{explicit_runner_map, shared_runner_map, LiveRunnerMap},
    shared_live_execution_enabled_from_env, CLI_SCRIPTED_LIVE_ENV,
};
use crate::drivers::{DriverExecutionResult, HarnessDriver};
use crate::ExecutionMode;
use std::sync::Arc;

/// CLI-scripted driver with optional live execution for S-01 through S-15.
#[derive(Clone)]
pub struct CliScriptedDriver {
    live_execution_enabled: bool,
    live_runners: LiveRunnerMap,
}

impl std::fmt::Debug for CliScriptedDriver {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("CliScriptedDriver")
            .field("live_execution_enabled", &self.live_execution_enabled)
            .finish()
    }
}

impl Default for CliScriptedDriver {
    fn default() -> Self {
        Self::from_env()
    }
}

impl CliScriptedDriver {
    /// Builds CLI-scripted driver from environment configuration.
    pub fn from_env() -> Self {
        Self::with_runners(
            live_execution_enabled_from_env(),
            run_live_s01_cli_health_probe,
            run_live_s02_cli_direct_message_probe,
            run_live_s03_cli_group_channel_probe,
            run_live_s04_cli_task_lifecycle_probe,
            run_live_s05_cli_escrow_settlement_probe,
            (
                run_live_s06_cli_proof_verification_probe,
                run_live_s07_cli_replay_protection_probe,
                run_live_s08_cli_crash_recovery_probe,
                run_live_s09_cli_transport_failover_probe,
                run_live_s10_cli_topology_coherence_probe,
                run_live_s11_cli_signer_rotation_probe,
                run_live_s12_cli_retention_deletion_probe,
                run_live_s13_cli_bridge_forwarding_probe,
                run_live_s14_cli_batch_merkle_probe,
                run_live_s15_cli_performance_smoke_probe,
            ),
        )
    }

    /// Creates CLI-scripted driver with one runner reused for all live-bound scenarios.
    pub fn with_runner<F>(live_execution_enabled: bool, live_runner: F) -> Self
    where
        F: Fn() -> Result<(), String> + Send + Sync + 'static,
    {
        Self::from_runner_map(
            live_execution_enabled,
            shared_runner_map(Arc::new(live_runner)),
        )
    }

    /// Creates CLI-scripted driver with explicit per-scenario live runners.
    pub fn with_runners<F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T>(
        live_execution_enabled: bool,
        discovery_runner: F,
        direct_message_runner: G,
        group_channel_runner: H,
        task_lifecycle_runner: I,
        escrow_settlement_runner: J,
        late_runners: (K, L, M, N, O, P, Q, R, S, T),
    ) -> Self
    where
        F: Fn() -> Result<(), String> + Send + Sync + 'static,
        G: Fn() -> Result<(), String> + Send + Sync + 'static,
        H: Fn() -> Result<(), String> + Send + Sync + 'static,
        I: Fn() -> Result<(), String> + Send + Sync + 'static,
        J: Fn() -> Result<(), String> + Send + Sync + 'static,
        K: Fn() -> Result<(), String> + Send + Sync + 'static,
        L: Fn() -> Result<(), String> + Send + Sync + 'static,
        M: Fn() -> Result<(), String> + Send + Sync + 'static,
        N: Fn() -> Result<(), String> + Send + Sync + 'static,
        O: Fn() -> Result<(), String> + Send + Sync + 'static,
        P: Fn() -> Result<(), String> + Send + Sync + 'static,
        Q: Fn() -> Result<(), String> + Send + Sync + 'static,
        R: Fn() -> Result<(), String> + Send + Sync + 'static,
        S: Fn() -> Result<(), String> + Send + Sync + 'static,
        T: Fn() -> Result<(), String> + Send + Sync + 'static,
    {
        let (
            proof,
            replay,
            crash,
            transport,
            topology,
            signer,
            retention,
            bridge,
            merkle,
            performance,
        ) = late_runners;
        Self::from_runner_map(
            live_execution_enabled,
            explicit_runner_map(
                [
                    Arc::new(discovery_runner),
                    Arc::new(direct_message_runner),
                    Arc::new(group_channel_runner),
                    Arc::new(task_lifecycle_runner),
                    Arc::new(escrow_settlement_runner),
                ],
                [
                    Arc::new(proof),
                    Arc::new(replay),
                    Arc::new(crash),
                    Arc::new(transport),
                    Arc::new(topology),
                    Arc::new(signer),
                    Arc::new(retention),
                    Arc::new(bridge),
                    Arc::new(merkle),
                    Arc::new(performance),
                ],
            ),
        )
    }

    fn from_runner_map(live_execution_enabled: bool, live_runners: LiveRunnerMap) -> Self {
        Self {
            live_execution_enabled,
            live_runners,
        }
    }

    fn live_runner_for_scenario(&self, scenario_id: &'static str) -> Option<Result<(), String>> {
        self.live_runners
            .get(scenario_id)
            .map(|runner| runner.as_ref()())
    }

    fn status_for_scenario(&self, scenario_id: &'static str) -> &'static str {
        if !is_live_bound_scenario_id(scenario_id) {
            return "pass";
        }
        if !self.live_execution_enabled {
            return "fail";
        }
        match self.live_runner_for_scenario(scenario_id) {
            Some(result) if result.is_ok() => "pass",
            Some(_) | None => "fail",
        }
    }
}

impl HarnessDriver for CliScriptedDriver {
    fn mode(&self) -> ExecutionMode {
        ExecutionMode::CliScripted
    }

    fn execute(&self, scenario_id: &'static str) -> DriverExecutionResult {
        DriverExecutionResult {
            scenario_id,
            status: self.status_for_scenario(scenario_id),
        }
    }
}

pub(crate) fn live_execution_enabled_from_env() -> bool {
    shared_live_execution_enabled_from_env(CLI_SCRIPTED_LIVE_ENV)
}
