use crate::drivers::shared_helpers::{
    env_var_or_default, env_var_or_else, is_live_bound_scenario_id,
    live_execution_enabled_from_env as shared_live_execution_enabled_from_env,
    live_s07_probe_agent_suffix, parse_s15_budget_env_u128, validate_s07_replay_reason_marker,
    validate_s12_content_field_coherence, validate_s12_content_id_match,
    validate_s13_bridge_field_coherence, validate_s13_bridge_id_match,
    validate_s15_latency_budget_samples,
};
use kamn_agent_lib::{KamnAgentHandle, KolmeProofReceipt};

#[path = "sdk_direct/driver_core.rs"]
mod driver_core;
#[path = "sdk_direct/live_probe_tranche_one.rs"]
mod live_probe_tranche_one;
#[path = "sdk_direct/live_probe_tranche_three.rs"]
mod live_probe_tranche_three;
#[path = "sdk_direct/live_probe_tranche_two.rs"]
mod live_probe_tranche_two;

use live_probe_tranche_one::{
    run_live_s01_discovery_probe, run_live_s02_direct_message_probe,
    run_live_s03_group_channel_probe, run_live_s04_task_lifecycle_probe,
    run_live_s05_escrow_settlement_probe,
};
use live_probe_tranche_three::{
    run_live_s11_signer_rotation_probe, run_live_s12_retention_deletion_probe,
    run_live_s13_bridge_forwarding_probe, run_live_s14_batch_merkle_probe,
    run_live_s15_performance_smoke_probe,
};
use live_probe_tranche_two::{
    run_live_s06_proof_verification_probe, run_live_s07_replay_protection_probe,
    run_live_s08_crash_recovery_probe, run_live_s09_transport_failover_probe,
    run_live_s10_topology_coherence_probe, validate_s08_distinct_message_ids,
    validate_s08_message_receipt_fields, validate_s08_query_message_response,
};

const SDK_DIRECT_LIVE_ENV: &str = "KAMN_E2E_SDK_DIRECT_LIVE";
const DEFAULT_KOLME_ENDPOINT: &str = "http://localhost:3000";
pub(crate) const DEFAULT_AGENT_NAME: &str = "kamn-e2e-sdk-direct";

type LiveProbe = dyn Fn() -> Result<(), String> + Send + Sync + 'static;

pub use driver_core::SdkDirectDriver;

#[cfg(test)]
#[path = "sdk_direct_tests.rs"]
mod sdk_direct_tests;
