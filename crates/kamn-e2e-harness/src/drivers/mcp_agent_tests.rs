use super::driver_core::live_execution_enabled_from_env;
use super::probe_protocol_support::validate_probe_health_response;
use super::tool_call_support::run_live_s15_mcp_tool_call;
use super::{
    build_framed_jsonrpc_request, escape_json_scalar, json_optional_string_field,
    json_optional_u64_field, parse_framed_jsonrpc_payloads, parse_s15_budget_env_u128,
    run_live_s01_mcp_probe, run_live_s02_mcp_direct_message_probe,
    run_live_s03_mcp_group_channel_probe, run_live_s04_mcp_task_lifecycle_probe,
    run_live_s04_mcp_tool_call, run_live_s05_mcp_escrow_settlement_probe,
    run_live_s06_mcp_proof_verification_probe, run_live_s07_mcp_replay_protection_probe,
    run_live_s08_mcp_crash_recovery_probe, run_live_s09_mcp_transport_failover_probe,
    run_live_s10_mcp_topology_coherence_probe, run_live_s11_mcp_signer_rotation_probe,
    run_live_s12_mcp_retention_deletion_probe, run_live_s13_mcp_bridge_forwarding_probe,
    run_live_s13_mcp_tool_call, run_live_s14_mcp_batch_merkle_probe, run_live_s14_mcp_tool_call,
    run_live_s15_mcp_performance_smoke_probe, validate_live_s05_release_escrow_response,
    validate_probe_initialize_response, validate_s07_replay_reason_marker,
    validate_s08_mcp_message_receipt_fields, validate_s08_mcp_query_message_response,
    validate_s12_content_field_coherence, validate_s12_content_id_match,
    validate_s13_bridge_field_coherence, validate_s13_bridge_id_match,
    validate_s14_mcp_verify_proof_response, validate_s15_latency_budget_samples, McpAgentDriver,
    MCP_AGENT_LIVE_ENV,
};
use crate::ExecutionMode;

#[path = "mcp_agent_tests/base_contract_tests.rs"]
mod base_contract_tests;
#[path = "mcp_agent_tests/driver_path_contract_tests.rs"]
mod driver_path_contract_tests;
#[path = "mcp_agent_tests/message_and_transport_contract_tests.rs"]
mod message_and_transport_contract_tests;
#[path = "mcp_agent_tests/missing_binary_probe_contract_tests.rs"]
mod missing_binary_probe_contract_tests;
#[path = "mcp_agent_tests/missing_binary_probe_extended_contract_tests.rs"]
mod missing_binary_probe_extended_contract_tests;
#[path = "mcp_agent_tests/payload_and_budget_contract_tests.rs"]
mod payload_and_budget_contract_tests;
#[path = "mcp_agent_tests/probe_protocol_contract_tests.rs"]
mod probe_protocol_contract_tests;
#[path = "mcp_agent_tests/rotation_batch_performance_contract_tests.rs"]
mod rotation_batch_performance_contract_tests;
#[path = "mcp_agent_tests/support.rs"]
mod support;
#[path = "mcp_agent_tests/tool_call_contract_tests.rs"]
mod tool_call_contract_tests;
#[path = "mcp_agent_tests/validator_contract_tests.rs"]
mod validator_contract_tests;

pub(crate) use support::*;
