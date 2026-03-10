use super::live_probe_tranche_three::validate_s14_cli_verify_proof_response;
use super::{
    live_execution_enabled_from_env, live_s07_probe_agent_suffix, parse_s15_budget_env_u128,
    parse_text_output_field, run_cli_command_capture_stdout,
    run_cli_command_expect_failure_with_agent_name, run_live_s01_cli_health_probe,
    run_live_s02_cli_direct_message_probe, run_live_s03_cli_group_channel_probe,
    run_live_s04_cli_task_lifecycle_probe, run_live_s05_cli_escrow_settlement_probe,
    run_live_s06_cli_proof_verification_probe, run_live_s07_cli_replay_protection_probe,
    run_live_s08_cli_crash_recovery_probe, run_live_s09_cli_transport_failover_probe,
    run_live_s10_cli_topology_coherence_probe, run_live_s11_cli_signer_rotation_probe,
    run_live_s12_cli_retention_deletion_probe, run_live_s13_cli_bridge_forwarding_probe,
    run_live_s14_cli_batch_merkle_probe, run_live_s15_cli_performance_smoke_probe,
    validate_live_s05_release_escrow_response, validate_s08_message_receipt_fields,
    validate_s08_query_message_response, validate_s15_latency_budget_samples, CliScriptedDriver,
    CLI_BINARY_ENV, CLI_SCRIPTED_LIVE_ENV,
};
use crate::drivers::shared_helpers::{
    validate_s07_replay_reason_marker, validate_s12_content_field_coherence,
    validate_s12_content_id_match, validate_s13_bridge_field_coherence,
    validate_s13_bridge_id_match,
};

#[path = "cli_scripted_tests/base_contract_tests.rs"]
mod base_contract_tests;
#[path = "cli_scripted_tests/continuity_probe_contract_tests.rs"]
mod continuity_probe_contract_tests;
#[path = "cli_scripted_tests/driver_path_contract_tests.rs"]
mod driver_path_contract_tests;
#[path = "cli_scripted_tests/live_probe_contract_tests.rs"]
mod live_probe_contract_tests;
#[path = "cli_scripted_tests/missing_binary_probe_contract_tests.rs"]
mod missing_binary_probe_contract_tests;
#[path = "cli_scripted_tests/missing_binary_probe_extended_contract_tests.rs"]
mod missing_binary_probe_extended_contract_tests;
#[path = "cli_scripted_tests/payload_and_budget_contract_tests.rs"]
mod payload_and_budget_contract_tests;
#[path = "cli_scripted_tests/rotation_batch_contract_tests.rs"]
mod rotation_batch_contract_tests;
#[path = "cli_scripted_tests/support.rs"]
mod support;
#[path = "cli_scripted_tests/validator_contract_tests.rs"]
mod validator_contract_tests;

pub(crate) use support::*;
