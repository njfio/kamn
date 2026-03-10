pub(super) use crate::drivers::shared_helpers::{
    env_var_or_default, env_var_or_else, is_live_bound_scenario_id,
    live_execution_enabled_from_env as shared_live_execution_enabled_from_env,
    live_s07_probe_agent_suffix, parse_s15_budget_env_u128,
    validate_live_s05_release_escrow_response, validate_s07_replay_reason_marker,
    validate_s12_content_field_coherence, validate_s12_content_id_match,
    validate_s13_bridge_field_coherence, validate_s13_bridge_id_match,
    validate_s15_latency_budget_samples,
};
use kamn_mcp_server::json_optional_bool_field;

const MCP_AGENT_LIVE_ENV: &str = "KAMN_E2E_MCP_AGENT_LIVE";
const MCP_AGENT_BINARY_ENV: &str = "KAMN_E2E_MCP_AGENT_BINARY";
const DEFAULT_MCP_AGENT_BINARY: &str = "kamn-mcp-server";
const DEFAULT_MCP_AGENT_NAME: &str = "kamn-e2e-mcp-probe";
const DEFAULT_MCP_AGENT_KEY_FILE: &str = "/tmp/kamn-e2e-mcp.key";
const DEFAULT_KAMN_ENDPOINT: &str = "http://localhost:8080";
const DEFAULT_S02_MESSAGE_PAYLOAD: &str = r#"{"message":"mcp-agent-live-s02"}"#;
const DEFAULT_S02_REPLY_PAYLOAD: &str = r#"{"message":"mcp-agent-live-s02-reply"}"#;
const DEFAULT_S03_CHANNEL_PAYLOAD: &str =
    r#"{"name":"mcp-agent-live-s03","members":["alice","bob","carol"]}"#;
const DEFAULT_S03_MESSAGE_PAYLOAD: &str = r#"{"message":"mcp-agent-live-s03-channel-message"}"#;
const DEFAULT_S04_CREATE_TASK_PAYLOAD: &str =
    r#"{"title":"mcp-agent-live-s04","description":"live task lifecycle probe"}"#;
const DEFAULT_S04_ESCROW_AMOUNT: u64 = 1;
const DEFAULT_S05_FUND_ESCROW_PAYLOAD: &str = r#"{"task_id":"mcp-agent-live-s05","amount":1}"#;
const DEFAULT_S07_AGENT_NAME: &str = "kamn-e2e-mcp-s07";
const DEFAULT_S07_MESSAGE_PAYLOAD: &str = r#"{"message":"mcp-agent-live-s07-replay"}"#;
const DEFAULT_S08_AGENT_NAME: &str = "kamn-e2e-mcp-s08";
const DEFAULT_S08_PRE_MESSAGE_PAYLOAD: &str = r#"{"message":"mcp-agent-live-s08-pre"}"#;
const DEFAULT_S08_POST_MESSAGE_PAYLOAD: &str = r#"{"message":"mcp-agent-live-s08-post"}"#;
const DEFAULT_S09_AGENT_NAME: &str = "kamn-e2e-mcp-s09";
const DEFAULT_S09_PRE_MESSAGE_PAYLOAD: &str = r#"{"message":"mcp-agent-live-s09-pre"}"#;
const DEFAULT_S09_POST_MESSAGE_PAYLOAD: &str = r#"{"message":"mcp-agent-live-s09-post"}"#;
const DEFAULT_S10_AGENT_NAME: &str = "kamn-e2e-mcp-s10";
const DEFAULT_S10_MESSAGE_PAYLOAD: &str = r#"{"message":"mcp-agent-live-s10-topology"}"#;
const DEFAULT_S11_PRIMARY_AGENT_NAME: &str = "kamn-e2e-mcp-s11-primary";
const DEFAULT_S11_MESSAGE_PAYLOAD: &str = r#"{"message":"mcp-agent-live-s11-primary"}"#;
const DEFAULT_S11_ROTATED_MESSAGE_PAYLOAD: &str = r#"{"message":"mcp-agent-live-s11-rotated"}"#;
const DEFAULT_S11_STALE_MESSAGE_PAYLOAD: &str = r#"{"message":"mcp-agent-live-s11-stale"}"#;
const DEFAULT_S12_AGENT_NAME: &str = "kamn-e2e-mcp-s12";
const DEFAULT_S12_REGISTER_CONTENT_PAYLOAD: &str =
    r#"{"content":"mcp-agent-live-s12","retention_class":"standard"}"#;
const DEFAULT_S13_AGENT_NAME: &str = "kamn-e2e-mcp-s13";
const DEFAULT_S13_SUBMIT_BRIDGE_PAYLOAD: &str =
    r#"{"source_message_id":"mcp-agent-live-s13","target_network":"testnet"}"#;
const DEFAULT_S14_AGENT_NAME: &str = "kamn-e2e-mcp-s14";
const DEFAULT_S14_BATCH_MESSAGE_PAYLOAD_A: &str = r#"{"message":"mcp-agent-live-s14-batch-a"}"#;
const DEFAULT_S14_BATCH_MESSAGE_PAYLOAD_B: &str = r#"{"message":"mcp-agent-live-s14-batch-b"}"#;
const DEFAULT_S14_BLOCK_HEIGHT: u64 = 1;
const DEFAULT_S14_FINALITY: &str = "final";
const DEFAULT_S15_AGENT_NAME: &str = "kamn-e2e-mcp-s15";
const DEFAULT_S15_MESSAGE_PAYLOAD: &str = r#"{"message":"mcp-agent-live-s15-performance"}"#;
const DEFAULT_S15_ITERATIONS: u64 = 3;
const DEFAULT_S15_MAX_TOTAL_MILLIS: u128 = 5_000;
const DEFAULT_S15_MAX_P50_MILLIS: u128 = 2_500;
const DEFAULT_S15_MAX_P99_MILLIS: u128 = 5_000;
const DEFAULT_S06_MESSAGE_ID: &str = "s06-live-proof";
const DEFAULT_S06_TX_HASH: &str = "sha256:s06-live-proof";
const DEFAULT_S06_BLOCK_HEIGHT: u64 = 1;
const DEFAULT_S06_FINALITY: &str = "final";

type LiveMcpProbe = dyn Fn() -> Result<(), String> + Send + Sync + 'static;

#[path = "mcp_agent/driver_core.rs"]
mod driver_core;
#[path = "mcp_agent/live_probe_tranche_one.rs"]
mod live_probe_tranche_one;
#[path = "mcp_agent/live_probe_tranche_three.rs"]
mod live_probe_tranche_three;
#[path = "mcp_agent/live_probe_tranche_two.rs"]
mod live_probe_tranche_two;
#[path = "mcp_agent/probe_protocol_support.rs"]
mod probe_protocol_support;
#[path = "mcp_agent/runner_registry.rs"]
mod runner_registry;
#[path = "mcp_agent/tool_call_support.rs"]
mod tool_call_support;

pub use driver_core::McpAgentDriver;
use live_probe_tranche_one::{
    run_live_s01_mcp_probe, run_live_s02_mcp_direct_message_probe,
    run_live_s03_mcp_group_channel_probe, run_live_s04_mcp_task_lifecycle_probe,
    run_live_s05_mcp_escrow_settlement_probe,
};
use live_probe_tranche_three::{
    run_live_s11_mcp_signer_rotation_probe, run_live_s12_mcp_retention_deletion_probe,
    run_live_s13_mcp_bridge_forwarding_probe, run_live_s14_mcp_batch_merkle_probe,
    run_live_s15_mcp_performance_smoke_probe, validate_s14_mcp_verify_proof_response,
};
use live_probe_tranche_two::{
    run_live_s06_mcp_proof_verification_probe, run_live_s07_mcp_replay_protection_probe,
    run_live_s08_mcp_crash_recovery_probe, run_live_s09_mcp_transport_failover_probe,
    run_live_s10_mcp_topology_coherence_probe,
};
use probe_protocol_support::{
    build_framed_jsonrpc_request, escape_json_scalar, json_optional_string_field,
    json_optional_u64_field, parse_framed_jsonrpc_payloads, validate_probe_initialize_response,
};
pub(crate) use tool_call_support::{
    run_live_s03_mcp_tool_call, run_live_s04_mcp_tool_call, run_live_s05_mcp_tool_call,
    run_live_s06_mcp_tool_call, run_live_s07_mcp_tool_call, run_live_s08_mcp_tool_call,
    run_live_s11_mcp_tool_call, run_live_s12_mcp_tool_call, run_live_s13_mcp_tool_call,
    run_live_s14_mcp_tool_call, run_named_mcp_tool_call,
};

#[cfg(test)]
#[path = "mcp_agent_tests.rs"]
mod mcp_agent_tests;
