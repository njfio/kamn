use super::{
    is_live_bound_scenario_id, run_live_s01_discovery_probe, run_live_s02_direct_message_probe,
    run_live_s03_group_channel_probe, run_live_s04_task_lifecycle_probe,
    run_live_s05_escrow_settlement_probe, run_live_s06_proof_verification_probe,
    run_live_s07_replay_protection_probe, run_live_s08_crash_recovery_probe,
    run_live_s09_transport_failover_probe, run_live_s10_topology_coherence_probe,
    run_live_s11_signer_rotation_probe, run_live_s12_retention_deletion_probe,
    run_live_s13_bridge_forwarding_probe, run_live_s14_batch_merkle_probe,
    run_live_s15_performance_smoke_probe, shared_live_execution_enabled_from_env, LiveProbe,
    SDK_DIRECT_LIVE_ENV,
};
use crate::drivers::{live_probe_driver_result, DriverExecutionResult, HarnessDriver};
use crate::ExecutionMode;
use std::collections::BTreeMap;
use std::sync::Arc;

type ProbeFn = fn() -> Result<(), String>;
type LiveProbeMap = BTreeMap<&'static str, Arc<LiveProbe>>;
type TrancheTwoAndThreeProbeFns = [ProbeFn; 10];

const LIVE_SCENARIO_IDS: [&str; 15] = [
    "S-01", "S-02", "S-03", "S-04", "S-05", "S-06", "S-07", "S-08", "S-09", "S-10", "S-11", "S-12",
    "S-13", "S-14", "S-15",
];
const EARLY_SCENARIO_IDS: [&str; 5] = ["S-01", "S-02", "S-03", "S-04", "S-05"];
const LATE_SCENARIO_IDS: [&str; 10] = [
    "S-06", "S-07", "S-08", "S-09", "S-10", "S-11", "S-12", "S-13", "S-14", "S-15",
];

/// SDK-direct driver with optional live execution for S-01 through S-15.
#[derive(Clone)]
pub struct SdkDirectDriver {
    live_execution_enabled: bool,
    live_probes: LiveProbeMap,
}

impl std::fmt::Debug for SdkDirectDriver {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("SdkDirectDriver")
            .field("live_execution_enabled", &self.live_execution_enabled)
            .finish()
    }
}

impl Default for SdkDirectDriver {
    fn default() -> Self {
        Self::from_env()
    }
}

impl SdkDirectDriver {
    /// Builds SDK-direct driver from environment configuration.
    pub fn from_env() -> Self {
        Self::with_probes(
            shared_live_execution_enabled_from_env(SDK_DIRECT_LIVE_ENV),
            run_live_s01_discovery_probe,
            run_live_s02_direct_message_probe,
            run_live_s03_group_channel_probe,
            run_live_s04_task_lifecycle_probe,
            run_live_s05_escrow_settlement_probe,
            tranche_two_and_three_probe_fns(),
        )
    }

    /// Creates SDK-direct driver with one probe reused for all live-bound scenarios.
    pub fn with_probe<F>(live_execution_enabled: bool, live_probe: F) -> Self
    where
        F: Fn() -> Result<(), String> + Send + Sync + 'static,
    {
        Self::from_probe_map(
            live_execution_enabled,
            shared_probe_map(Arc::new(live_probe)),
        )
    }

    fn with_probes(
        live_execution_enabled: bool,
        discovery_probe: ProbeFn,
        direct_message_probe: ProbeFn,
        group_channel_probe: ProbeFn,
        task_lifecycle_probe: ProbeFn,
        escrow_settlement_probe: ProbeFn,
        tranche_two_and_three_probes: TrancheTwoAndThreeProbeFns,
    ) -> Self {
        Self::from_probe_map(
            live_execution_enabled,
            explicit_probe_map(
                discovery_probe,
                direct_message_probe,
                group_channel_probe,
                task_lifecycle_probe,
                escrow_settlement_probe,
                tranche_two_and_three_probes,
            ),
        )
    }

    fn from_probe_map(live_execution_enabled: bool, live_probes: LiveProbeMap) -> Self {
        Self {
            live_execution_enabled,
            live_probes,
        }
    }

    fn live_probe_for_scenario(&self, scenario_id: &'static str) -> Option<Result<(), String>> {
        self.live_probes
            .get(scenario_id)
            .map(|probe| probe.as_ref()())
    }

    fn execution_result_for_scenario(&self, scenario_id: &'static str) -> DriverExecutionResult {
        live_probe_driver_result(
            scenario_id,
            is_live_bound_scenario_id(scenario_id),
            self.live_execution_enabled,
            || self.live_probe_for_scenario(scenario_id),
        )
    }
}

impl HarnessDriver for SdkDirectDriver {
    fn mode(&self) -> ExecutionMode {
        ExecutionMode::SdkDirect
    }

    fn execute(&self, scenario_id: &'static str) -> DriverExecutionResult {
        self.execution_result_for_scenario(scenario_id)
    }
}

fn shared_probe_map(live_probe: Arc<LiveProbe>) -> LiveProbeMap {
    LIVE_SCENARIO_IDS
        .into_iter()
        .map(|scenario_id| (scenario_id, Arc::clone(&live_probe)))
        .collect()
}

fn explicit_probe_map(
    discovery_probe: ProbeFn,
    direct_message_probe: ProbeFn,
    group_channel_probe: ProbeFn,
    task_lifecycle_probe: ProbeFn,
    escrow_settlement_probe: ProbeFn,
    tranche_two_and_three_probes: TrancheTwoAndThreeProbeFns,
) -> LiveProbeMap {
    let mut live_probes = scenario_map(
        EARLY_SCENARIO_IDS,
        [
            discovery_probe,
            direct_message_probe,
            group_channel_probe,
            task_lifecycle_probe,
            escrow_settlement_probe,
        ],
    );
    live_probes.extend(scenario_map(
        LATE_SCENARIO_IDS,
        tranche_two_and_three_probes,
    ));
    live_probes
}

fn tranche_two_and_three_probe_fns() -> TrancheTwoAndThreeProbeFns {
    [
        run_live_s06_proof_verification_probe,
        run_live_s07_replay_protection_probe,
        run_live_s08_crash_recovery_probe,
        run_live_s09_transport_failover_probe,
        run_live_s10_topology_coherence_probe,
        run_live_s11_signer_rotation_probe,
        run_live_s12_retention_deletion_probe,
        run_live_s13_bridge_forwarding_probe,
        run_live_s14_batch_merkle_probe,
        run_live_s15_performance_smoke_probe,
    ]
}

fn scenario_map<const N: usize>(
    scenario_ids: [&'static str; N],
    probes: [ProbeFn; N],
) -> LiveProbeMap {
    scenario_ids
        .into_iter()
        .zip(probes)
        .map(|(scenario_id, probe)| scenario_probe(scenario_id, probe))
        .collect()
}

fn scenario_probe(scenario_id: &'static str, probe: ProbeFn) -> (&'static str, Arc<LiveProbe>) {
    (scenario_id, Arc::new(probe))
}
