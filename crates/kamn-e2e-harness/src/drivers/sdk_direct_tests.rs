use super::{
    live_probe_tranche_one, live_s07_probe_agent_suffix, parse_s15_budget_env_u128,
    run_live_s06_proof_verification_probe, run_live_s07_replay_protection_probe,
    run_live_s08_crash_recovery_probe, run_live_s09_transport_failover_probe,
    run_live_s10_topology_coherence_probe, run_live_s11_signer_rotation_probe,
    run_live_s12_retention_deletion_probe, run_live_s13_bridge_forwarding_probe,
    run_live_s14_batch_merkle_probe, run_live_s15_performance_smoke_probe,
    shared_live_execution_enabled_from_env, validate_s07_replay_reason_marker,
    validate_s08_distinct_message_ids, validate_s08_message_receipt_fields,
    validate_s08_query_message_response, validate_s12_content_field_coherence,
    validate_s12_content_id_match, validate_s13_bridge_field_coherence,
    validate_s13_bridge_id_match, validate_s14_proof_response, validate_s15_latency_budget_samples,
    SdkDirectDriver, SDK_DIRECT_LIVE_ENV,
};

use live_probe_tranche_one::{
    run_live_s01_discovery_probe, run_live_s02_direct_message_probe,
    run_live_s03_group_channel_probe, run_live_s04_task_lifecycle_probe,
    run_live_s05_escrow_settlement_probe, validate_live_s03_list_messages_response,
    validate_live_s03_query_message_response, validate_live_s05_release_escrow_receipt,
};

fn live_execution_enabled_from_env() -> bool {
    shared_live_execution_enabled_from_env(SDK_DIRECT_LIVE_ENV)
}

#[path = "sdk_direct_tests/base_contract_tests.rs"]
mod base_contract_tests;
#[path = "sdk_direct_tests/driver_path_contract_tests.rs"]
mod driver_path_contract_tests;
#[path = "sdk_direct_tests/invalid_endpoint_probe_contract_tests.rs"]
mod invalid_endpoint_probe_contract_tests;
#[path = "sdk_direct_tests/live_probe_contract_tests.rs"]
mod live_probe_contract_tests;
#[path = "sdk_direct_tests/payload_and_budget_contract_tests.rs"]
mod payload_and_budget_contract_tests;
#[path = "sdk_direct_tests/support.rs"]
mod support;
#[path = "sdk_direct_tests/validator_contract_tests.rs"]
mod validator_contract_tests;

pub(crate) use support::*;
