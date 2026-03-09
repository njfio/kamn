use crate::drivers::shared_helpers::{
    env_var_or_default, env_var_or_else, is_live_bound_scenario_id,
    live_execution_enabled_from_env as shared_live_execution_enabled_from_env,
    live_s07_probe_agent_suffix, parse_s15_budget_env_u128,
    validate_live_s05_release_escrow_response, validate_s07_replay_reason_marker,
    validate_s12_content_field_coherence, validate_s12_content_id_match,
    validate_s13_bridge_field_coherence, validate_s13_bridge_id_match,
    validate_s15_latency_budget_samples,
};
use crate::drivers::{DriverExecutionResult, HarnessDriver};
use crate::ExecutionMode;
use kamn_mcp_server::json_optional_bool_field;
use std::env;
use std::io::Write;
use std::process::{Command, Stdio};
use std::sync::Arc;

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

/// MCP-agent driver for Tau and generic MCP runtimes.
#[derive(Clone)]
pub struct McpAgentDriver {
    mode: ExecutionMode,
    live_execution_enabled: bool,
    discovery_probe: Arc<LiveMcpProbe>,
    direct_message_probe: Arc<LiveMcpProbe>,
    group_channel_probe: Arc<LiveMcpProbe>,
    task_lifecycle_probe: Arc<LiveMcpProbe>,
    escrow_settlement_probe: Arc<LiveMcpProbe>,
    proof_verification_probe: Arc<LiveMcpProbe>,
    replay_protection_probe: Arc<LiveMcpProbe>,
    crash_recovery_probe: Arc<LiveMcpProbe>,
    transport_failover_probe: Arc<LiveMcpProbe>,
    topology_coherence_probe: Arc<LiveMcpProbe>,
    signer_rotation_probe: Arc<LiveMcpProbe>,
    retention_deletion_probe: Arc<LiveMcpProbe>,
    bridge_forwarding_probe: Arc<LiveMcpProbe>,
    batch_merkle_probe: Arc<LiveMcpProbe>,
    performance_smoke_probe: Arc<LiveMcpProbe>,
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
        Self::with_probes(
            mode,
            live_execution_enabled_from_env(),
            run_live_s01_mcp_probe,
            run_live_s02_mcp_direct_message_probe,
            run_live_s03_mcp_group_channel_probe,
            run_live_s04_mcp_task_lifecycle_probe,
            (
                run_live_s05_mcp_escrow_settlement_probe,
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
    pub fn with_probe<F>(
        mode: ExecutionMode,
        live_execution_enabled: bool,
        live_probe: F,
    ) -> Result<Self, String>
    where
        F: Fn() -> Result<(), String> + Send + Sync + 'static,
    {
        if !matches!(mode, ExecutionMode::McpTau | ExecutionMode::McpAny) {
            return Err("McpAgentDriver requires mcp-tau or mcp-any mode".to_owned());
        }
        let live_probe: Arc<LiveMcpProbe> = Arc::new(live_probe);
        Ok(Self {
            mode,
            live_execution_enabled,
            discovery_probe: live_probe.clone(),
            direct_message_probe: live_probe.clone(),
            task_lifecycle_probe: live_probe.clone(),
            group_channel_probe: live_probe.clone(),
            escrow_settlement_probe: live_probe.clone(),
            proof_verification_probe: live_probe.clone(),
            replay_protection_probe: live_probe.clone(),
            crash_recovery_probe: live_probe.clone(),
            transport_failover_probe: live_probe.clone(),
            topology_coherence_probe: live_probe.clone(),
            signer_rotation_probe: live_probe.clone(),
            retention_deletion_probe: live_probe.clone(),
            bridge_forwarding_probe: live_probe.clone(),
            batch_merkle_probe: live_probe.clone(),
            performance_smoke_probe: live_probe,
        })
    }

    /// Creates MCP driver with explicit per-scenario probe implementations.
    pub fn with_probes<F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T>(
        mode: ExecutionMode,
        live_execution_enabled: bool,
        discovery_probe: F,
        direct_message_probe: G,
        group_channel_probe: H,
        task_lifecycle_probe: I,
        escrow_proof_replay_crash_failover_topology_signer_retention_bridge_merkle_and_performance_probes: (
            J,
            K,
            L,
            M,
            N,
            O,
            P,
            Q,
            R,
            S,
            T,
        ),
    ) -> Result<Self, String>
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
        if !matches!(mode, ExecutionMode::McpTau | ExecutionMode::McpAny) {
            return Err("McpAgentDriver requires mcp-tau or mcp-any mode".to_owned());
        }
        let (
            escrow_settlement_probe,
            proof_verification_probe,
            replay_protection_probe,
            crash_recovery_probe,
            transport_failover_probe,
            topology_coherence_probe,
            signer_rotation_probe,
            retention_deletion_probe,
            bridge_forwarding_probe,
            batch_merkle_probe,
            performance_smoke_probe,
        ) = escrow_proof_replay_crash_failover_topology_signer_retention_bridge_merkle_and_performance_probes;
        Ok(Self {
            mode,
            live_execution_enabled,
            discovery_probe: Arc::new(discovery_probe),
            direct_message_probe: Arc::new(direct_message_probe),
            group_channel_probe: Arc::new(group_channel_probe),
            task_lifecycle_probe: Arc::new(task_lifecycle_probe),
            escrow_settlement_probe: Arc::new(escrow_settlement_probe),
            proof_verification_probe: Arc::new(proof_verification_probe),
            replay_protection_probe: Arc::new(replay_protection_probe),
            crash_recovery_probe: Arc::new(crash_recovery_probe),
            transport_failover_probe: Arc::new(transport_failover_probe),
            topology_coherence_probe: Arc::new(topology_coherence_probe),
            signer_rotation_probe: Arc::new(signer_rotation_probe),
            retention_deletion_probe: Arc::new(retention_deletion_probe),
            bridge_forwarding_probe: Arc::new(bridge_forwarding_probe),
            batch_merkle_probe: Arc::new(batch_merkle_probe),
            performance_smoke_probe: Arc::new(performance_smoke_probe),
        })
    }
}

impl HarnessDriver for McpAgentDriver {
    fn mode(&self) -> ExecutionMode {
        self.mode
    }

    fn execute(&self, scenario_id: &'static str) -> DriverExecutionResult {
        let status = if !is_live_bound_scenario_id(scenario_id) {
            "pass"
        } else if !self.live_execution_enabled {
            "fail"
        } else {
            match self.live_probe_for_scenario(scenario_id) {
                Some(result) if result.is_ok() => "pass",
                Some(_) => "fail",
                None => "fail",
            }
        };
        DriverExecutionResult {
            scenario_id,
            status,
        }
    }
}

impl McpAgentDriver {
    fn live_probe_for_scenario(&self, scenario_id: &'static str) -> Option<Result<(), String>> {
        match scenario_id {
            "S-01" => Some((self.discovery_probe)()),
            "S-02" => Some((self.direct_message_probe)()),
            "S-03" => Some((self.group_channel_probe)()),
            "S-04" => Some((self.task_lifecycle_probe)()),
            "S-05" => Some((self.escrow_settlement_probe)()),
            "S-06" => Some((self.proof_verification_probe)()),
            "S-07" => Some((self.replay_protection_probe)()),
            "S-08" => Some((self.crash_recovery_probe)()),
            "S-09" => Some((self.transport_failover_probe)()),
            "S-10" => Some((self.topology_coherence_probe)()),
            "S-11" => Some((self.signer_rotation_probe)()),
            "S-12" => Some((self.retention_deletion_probe)()),
            "S-13" => Some((self.bridge_forwarding_probe)()),
            "S-14" => Some((self.batch_merkle_probe)()),
            "S-15" => Some((self.performance_smoke_probe)()),
            _ => None,
        }
    }
}

fn live_execution_enabled_from_env() -> bool {
    shared_live_execution_enabled_from_env(MCP_AGENT_LIVE_ENV)
}

fn run_live_s01_mcp_probe() -> Result<(), String> {
    let binary = env_var_or_default(MCP_AGENT_BINARY_ENV, DEFAULT_MCP_AGENT_BINARY);
    let endpoint = env_var_or_default("KAMN_ENDPOINT", DEFAULT_KAMN_ENDPOINT);
    let agent_name = env_var_or_default("KAMN_AGENT_NAME", DEFAULT_MCP_AGENT_NAME);
    let key_file = env_var_or_default("KAMN_AGENT_KEY_FILE", DEFAULT_MCP_AGENT_KEY_FILE);

    let mut child = Command::new(binary.as_str())
        .arg("--endpoint")
        .arg(endpoint.as_str())
        .arg("--agent-name")
        .arg(agent_name.as_str())
        .arg("--key-file")
        .arg(key_file.as_str())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|error| format!("mcp live probe failed to spawn: {error}"))?;

    if let Some(mut stdin) = child.stdin.take() {
        let initialize_request = build_framed_jsonrpc_request(
            r#"{"jsonrpc":"2.0","id":"probe-init","method":"initialize","params":{}}"#,
        );
        let health_request = build_framed_jsonrpc_request(
            r#"{"jsonrpc":"2.0","id":"probe-health","method":"tools/call","params":{"name":"health","arguments":{}}}"#,
        );
        let framed_requests = format!("{initialize_request}{health_request}");
        stdin
            .write_all(framed_requests.as_bytes())
            .map_err(|error| {
                format!("mcp live probe failed to write framed request stream: {error}")
            })?;
    }

    let output = child
        .wait_with_output()
        .map_err(|error| format!("mcp live probe failed to read response: {error}"))?;
    if !output.status.success() {
        let exit_status = output
            .status
            .code()
            .map(|value| value.to_string())
            .unwrap_or_else(|| "signal".to_owned());
        return Err(format!("mcp live probe failed (exit_status={exit_status})"));
    }

    let stdout = String::from_utf8_lossy(output.stdout.as_slice());
    let payloads = parse_framed_jsonrpc_payloads(stdout.as_ref())
        .map_err(|error| format!("mcp live probe invalid framed output: {error}"))?;

    let initialize_response = payloads
        .iter()
        .find(|payload| payload.contains(r#""id":"probe-init""#))
        .ok_or_else(|| "mcp live probe missing initialize response payload".to_owned())?;
    validate_probe_initialize_response(initialize_response)?;

    let health_response = payloads
        .iter()
        .find(|payload| payload.contains(r#""id":"probe-health""#))
        .ok_or_else(|| "mcp live probe missing health response payload".to_owned())?;
    validate_probe_health_response(health_response)?;
    Ok(())
}

fn run_live_s02_mcp_direct_message_probe() -> Result<(), String> {
    let binary = env_var_or_default(MCP_AGENT_BINARY_ENV, DEFAULT_MCP_AGENT_BINARY);
    let endpoint = env_var_or_default("KAMN_ENDPOINT", DEFAULT_KAMN_ENDPOINT);
    let agent_name = env_var_or_default("KAMN_AGENT_NAME", DEFAULT_MCP_AGENT_NAME);
    let key_file = env_var_or_default("KAMN_AGENT_KEY_FILE", DEFAULT_MCP_AGENT_KEY_FILE);
    let message_payload = env::var("KAMN_E2E_S02_MESSAGE_PAYLOAD")
        .unwrap_or_else(|_| DEFAULT_S02_MESSAGE_PAYLOAD.to_owned());
    let reply_payload = env::var("KAMN_E2E_S02_REPLY_PAYLOAD")
        .unwrap_or_else(|_| DEFAULT_S02_REPLY_PAYLOAD.to_owned());
    let send_agent_name = format!("{agent_name}-s02-send");
    let query_agent_name = format!("{agent_name}-s02-query");
    let reply_agent_name = format!("{agent_name}-s02-reply");
    let reply_query_agent_name = format!("{agent_name}-s02-query-reply");

    let send_arguments = format!(
        "{{\"payload\":\"{}\"}}",
        escape_json_scalar(message_payload.as_str())
    );
    let send_response = run_live_s02_mcp_tool_call(
        binary.as_str(),
        endpoint.as_str(),
        send_agent_name.as_str(),
        key_file.as_str(),
        "probe-send-message",
        "send_message",
        send_arguments.as_str(),
    )?;
    let message_id =
        json_optional_string_field(send_response.as_str(), "message_id").ok_or_else(|| {
            format!("mcp live s02 send_message response missing message_id field: {send_response}")
        })?;
    if message_id.trim().is_empty() {
        return Err("mcp live s02 send_message returned empty message_id".to_owned());
    }
    let send_status =
        json_optional_string_field(send_response.as_str(), "status").ok_or_else(|| {
            format!("mcp live s02 send_message response missing status field: {send_response}")
        })?;
    if send_status.trim().is_empty() {
        return Err("mcp live s02 send_message returned empty status".to_owned());
    }

    let query_arguments = format!(
        "{{\"message_id\":\"{}\"}}",
        escape_json_scalar(message_id.as_str())
    );
    let query_response = run_live_s02_mcp_tool_call(
        binary.as_str(),
        endpoint.as_str(),
        query_agent_name.as_str(),
        key_file.as_str(),
        "probe-query-message",
        "query_message",
        query_arguments.as_str(),
    )?;
    let queried_message_id = json_optional_string_field(query_response.as_str(), "message_id")
        .ok_or_else(|| {
            format!(
                "mcp live s02 query_message response missing message_id field: {query_response}"
            )
        })?;
    if queried_message_id != message_id {
        return Err(format!(
            "mcp live s02 query_message returned mismatched message_id: expected={message_id}, got={queried_message_id}"
        ));
    }
    let queried_status =
        json_optional_string_field(query_response.as_str(), "status").ok_or_else(|| {
            format!("mcp live s02 query_message response missing status field: {query_response}")
        })?;
    if queried_status.trim().is_empty() {
        return Err("mcp live s02 query_message returned empty status".to_owned());
    }

    let reply_arguments = format!(
        "{{\"payload\":\"{}\"}}",
        escape_json_scalar(reply_payload.as_str())
    );
    let reply_response = run_live_s02_mcp_tool_call(
        binary.as_str(),
        endpoint.as_str(),
        reply_agent_name.as_str(),
        key_file.as_str(),
        "probe-send-reply",
        "send_message",
        reply_arguments.as_str(),
    )?;
    let reply_message_id =
        json_optional_string_field(reply_response.as_str(), "message_id").ok_or_else(|| {
            format!(
                "mcp live s02 reply send_message response missing message_id field: {reply_response}"
            )
        })?;
    if reply_message_id.trim().is_empty() {
        return Err("mcp live s02 reply send_message returned empty message_id".to_owned());
    }
    let reply_send_status = json_optional_string_field(reply_response.as_str(), "status")
        .ok_or_else(|| {
            format!(
                "mcp live s02 reply send_message response missing status field: {reply_response}"
            )
        })?;
    if reply_send_status.trim().is_empty() {
        return Err("mcp live s02 reply send_message returned empty status".to_owned());
    }

    let reply_query_arguments = format!(
        "{{\"message_id\":\"{}\"}}",
        escape_json_scalar(reply_message_id.as_str())
    );
    let reply_query_response = run_live_s02_mcp_tool_call(
        binary.as_str(),
        endpoint.as_str(),
        reply_query_agent_name.as_str(),
        key_file.as_str(),
        "probe-query-reply-message",
        "query_message",
        reply_query_arguments.as_str(),
    )?;
    let reply_queried_message_id = json_optional_string_field(
        reply_query_response.as_str(),
        "message_id",
    )
    .ok_or_else(|| {
        format!(
            "mcp live s02 reply query_message response missing message_id field: {reply_query_response}"
        )
    })?;
    if reply_queried_message_id != reply_message_id {
        return Err(format!(
            "mcp live s02 reply query_message returned mismatched message_id: expected={reply_message_id}, got={reply_queried_message_id}"
        ));
    }
    let reply_queried_status =
        json_optional_string_field(reply_query_response.as_str(), "status").ok_or_else(|| {
            format!(
                "mcp live s02 reply query_message response missing status field: {reply_query_response}"
            )
        })?;
    if reply_queried_status.trim().is_empty() {
        return Err("mcp live s02 reply query_message returned empty status".to_owned());
    }

    Ok(())
}

fn run_live_s03_mcp_group_channel_probe() -> Result<(), String> {
    let binary = env_var_or_default(MCP_AGENT_BINARY_ENV, DEFAULT_MCP_AGENT_BINARY);
    let endpoint = env_var_or_default("KAMN_ENDPOINT", DEFAULT_KAMN_ENDPOINT);
    let agent_name = env_var_or_default("KAMN_AGENT_NAME", DEFAULT_MCP_AGENT_NAME);
    let key_file = env_var_or_default("KAMN_AGENT_KEY_FILE", DEFAULT_MCP_AGENT_KEY_FILE);
    let channel_payload = env::var("KAMN_E2E_S03_CHANNEL_PAYLOAD")
        .unwrap_or_else(|_| DEFAULT_S03_CHANNEL_PAYLOAD.to_owned());
    let message_payload = env::var("KAMN_E2E_S03_MESSAGE_PAYLOAD")
        .unwrap_or_else(|_| DEFAULT_S03_MESSAGE_PAYLOAD.to_owned());
    let create_agent_name = format!("{agent_name}-s03-create-channel");
    let send_agent_name = format!("{agent_name}-s03-send-message");
    let query_agent_name = format!("{agent_name}-s03-query-message");
    let list_agent_name = format!("{agent_name}-s03-list-messages");

    let create_arguments = format!(
        "{{\"payload\":\"{}\"}}",
        escape_json_scalar(channel_payload.as_str())
    );
    let create_response = run_live_s03_mcp_tool_call(
        binary.as_str(),
        endpoint.as_str(),
        create_agent_name.as_str(),
        key_file.as_str(),
        "probe-create-channel",
        "create_channel",
        create_arguments.as_str(),
    )?;
    let channel_id = json_optional_string_field(create_response.as_str(), "channel_id")
        .ok_or_else(|| {
            format!(
                "mcp live s03 create_channel response missing channel_id field: {create_response}"
            )
        })?;
    if channel_id.trim().is_empty() {
        return Err("mcp live s03 create_channel returned empty channel_id".to_owned());
    }
    let create_status =
        json_optional_string_field(create_response.as_str(), "status").ok_or_else(|| {
            format!("mcp live s03 create_channel response missing status field: {create_response}")
        })?;
    if create_status.trim().is_empty() {
        return Err("mcp live s03 create_channel returned empty status".to_owned());
    }

    let send_arguments = format!(
        "{{\"payload\":\"{}\"}}",
        escape_json_scalar(message_payload.as_str())
    );
    let send_response = run_live_s03_mcp_tool_call(
        binary.as_str(),
        endpoint.as_str(),
        send_agent_name.as_str(),
        key_file.as_str(),
        "probe-send-message",
        "send_message",
        send_arguments.as_str(),
    )?;
    let message_id =
        json_optional_string_field(send_response.as_str(), "message_id").ok_or_else(|| {
            format!("mcp live s03 send_message response missing message_id field: {send_response}")
        })?;
    if message_id.trim().is_empty() {
        return Err("mcp live s03 send_message returned empty message_id".to_owned());
    }
    let send_status =
        json_optional_string_field(send_response.as_str(), "status").ok_or_else(|| {
            format!("mcp live s03 send_message response missing status field: {send_response}")
        })?;
    if send_status.trim().is_empty() {
        return Err("mcp live s03 send_message returned empty status".to_owned());
    }

    let query_arguments = format!(
        "{{\"message_id\":\"{}\"}}",
        escape_json_scalar(message_id.as_str())
    );
    let query_response = run_live_s03_mcp_tool_call(
        binary.as_str(),
        endpoint.as_str(),
        query_agent_name.as_str(),
        key_file.as_str(),
        "probe-query-message",
        "query_message",
        query_arguments.as_str(),
    )?;
    let queried_message_id = json_optional_string_field(query_response.as_str(), "message_id")
        .ok_or_else(|| {
            format!(
                "mcp live s03 query_message response missing message_id field: {query_response}"
            )
        })?;
    if queried_message_id != message_id {
        return Err(format!(
            "mcp live s03 query_message returned mismatched message_id: expected={message_id}, got={queried_message_id}"
        ));
    }
    let queried_status =
        json_optional_string_field(query_response.as_str(), "status").ok_or_else(|| {
            format!("mcp live s03 query_message response missing status field: {query_response}")
        })?;
    if queried_status.trim().is_empty() {
        return Err("mcp live s03 query_message returned empty status".to_owned());
    }

    let list_arguments = format!(
        "{{\"channel_id\":\"{}\"}}",
        escape_json_scalar(channel_id.as_str())
    );
    let list_response = run_live_s03_mcp_tool_call(
        binary.as_str(),
        endpoint.as_str(),
        list_agent_name.as_str(),
        key_file.as_str(),
        "probe-list-messages",
        "list_messages",
        list_arguments.as_str(),
    )?;
    let listed_channel_id = json_optional_string_field(list_response.as_str(), "channel_id")
        .ok_or_else(|| {
            format!("mcp live s03 list_messages response missing channel_id field: {list_response}")
        })?;
    if listed_channel_id != channel_id {
        return Err(format!(
            "mcp live s03 list_messages returned mismatched channel_id: expected={channel_id}, got={listed_channel_id}"
        ));
    }
    if !list_response.contains(r#""messages":["#) {
        return Err(format!(
            "mcp live s03 list_messages response missing messages field: {list_response}"
        ));
    }

    Ok(())
}

fn run_live_s04_mcp_task_lifecycle_probe() -> Result<(), String> {
    let binary = env_var_or_default(MCP_AGENT_BINARY_ENV, DEFAULT_MCP_AGENT_BINARY);
    let endpoint = env_var_or_default("KAMN_ENDPOINT", DEFAULT_KAMN_ENDPOINT);
    let agent_name = env_var_or_default("KAMN_AGENT_NAME", DEFAULT_MCP_AGENT_NAME);
    let key_file = env_var_or_default("KAMN_AGENT_KEY_FILE", DEFAULT_MCP_AGENT_KEY_FILE);
    let create_task_payload = env::var("KAMN_E2E_S04_CREATE_TASK_PAYLOAD")
        .unwrap_or_else(|_| DEFAULT_S04_CREATE_TASK_PAYLOAD.to_owned());
    let create_agent_name = format!("{agent_name}-s04-create");
    let fund_agent_name = format!("{agent_name}-s04-fund");
    let accept_agent_name = format!("{agent_name}-s04-accept");
    let complete_agent_name = format!("{agent_name}-s04-complete");
    let release_agent_name = format!("{agent_name}-s04-release");

    let create_arguments = format!(
        "{{\"payload\":\"{}\"}}",
        escape_json_scalar(create_task_payload.as_str())
    );
    let create_response = run_live_s04_mcp_tool_call(
        binary.as_str(),
        endpoint.as_str(),
        create_agent_name.as_str(),
        key_file.as_str(),
        "probe-create-task",
        "create_task",
        create_arguments.as_str(),
    )?;
    let task_id =
        json_optional_string_field(create_response.as_str(), "task_id").ok_or_else(|| {
            format!("mcp live s04 create_task response missing task_id field: {create_response}")
        })?;
    if task_id.trim().is_empty() {
        return Err("mcp live s04 create_task returned empty task_id".to_owned());
    }

    let fund_payload = format!(
        "{{\"task_id\":\"{}\",\"amount\":{}}}",
        escape_json_scalar(task_id.as_str()),
        DEFAULT_S04_ESCROW_AMOUNT
    );
    let fund_arguments = format!(
        "{{\"payload\":\"{}\"}}",
        escape_json_scalar(fund_payload.as_str())
    );
    let fund_response = run_live_s04_mcp_tool_call(
        binary.as_str(),
        endpoint.as_str(),
        fund_agent_name.as_str(),
        key_file.as_str(),
        "probe-fund-escrow",
        "fund_escrow",
        fund_arguments.as_str(),
    )?;
    let escrow_id =
        json_optional_string_field(fund_response.as_str(), "escrow_id").ok_or_else(|| {
            format!("mcp live s04 fund_escrow response missing escrow_id field: {fund_response}")
        })?;
    if escrow_id.trim().is_empty() {
        return Err("mcp live s04 fund_escrow returned empty escrow_id".to_owned());
    }

    let accept_arguments = format!(
        "{{\"task_id\":\"{}\"}}",
        escape_json_scalar(task_id.as_str())
    );
    let accept_response = run_live_s04_mcp_tool_call(
        binary.as_str(),
        endpoint.as_str(),
        accept_agent_name.as_str(),
        key_file.as_str(),
        "probe-accept-task",
        "accept_task",
        accept_arguments.as_str(),
    )?;
    let accept_state =
        json_optional_string_field(accept_response.as_str(), "state").ok_or_else(|| {
            format!("mcp live s04 accept_task response missing state field: {accept_response}")
        })?;
    if accept_state.trim().is_empty() {
        return Err("mcp live s04 accept_task returned empty state".to_owned());
    }

    let complete_arguments = format!(
        "{{\"task_id\":\"{}\"}}",
        escape_json_scalar(task_id.as_str())
    );
    let complete_response = run_live_s04_mcp_tool_call(
        binary.as_str(),
        endpoint.as_str(),
        complete_agent_name.as_str(),
        key_file.as_str(),
        "probe-complete-task",
        "complete_task",
        complete_arguments.as_str(),
    )?;
    let complete_state = json_optional_string_field(complete_response.as_str(), "state")
        .ok_or_else(|| {
            format!("mcp live s04 complete_task response missing state field: {complete_response}")
        })?;
    if complete_state.trim().is_empty() {
        return Err("mcp live s04 complete_task returned empty state".to_owned());
    }

    let release_arguments = format!(
        "{{\"escrow_id\":\"{}\"}}",
        escape_json_scalar(escrow_id.as_str())
    );
    let release_response = run_live_s04_mcp_tool_call(
        binary.as_str(),
        endpoint.as_str(),
        release_agent_name.as_str(),
        key_file.as_str(),
        "probe-release-escrow",
        "release_escrow",
        release_arguments.as_str(),
    )?;
    let release_state =
        json_optional_string_field(release_response.as_str(), "state").ok_or_else(|| {
            format!("mcp live s04 release_escrow response missing state field: {release_response}")
        })?;
    if release_state.trim().is_empty() {
        return Err("mcp live s04 release_escrow returned empty state".to_owned());
    }

    Ok(())
}

fn run_live_s05_mcp_escrow_settlement_probe() -> Result<(), String> {
    let binary = env_var_or_default(MCP_AGENT_BINARY_ENV, DEFAULT_MCP_AGENT_BINARY);
    let endpoint = env_var_or_default("KAMN_ENDPOINT", DEFAULT_KAMN_ENDPOINT);
    let agent_name = env_var_or_default("KAMN_AGENT_NAME", DEFAULT_MCP_AGENT_NAME);
    let key_file = env_var_or_default("KAMN_AGENT_KEY_FILE", DEFAULT_MCP_AGENT_KEY_FILE);
    let fund_payload = env::var("KAMN_E2E_S05_FUND_ESCROW_PAYLOAD")
        .unwrap_or_else(|_| DEFAULT_S05_FUND_ESCROW_PAYLOAD.to_owned());
    let fund_agent_name = format!("{agent_name}-s05-fund");
    let release_agent_name = format!("{agent_name}-s05-release");

    let fund_arguments = format!(
        "{{\"payload\":\"{}\"}}",
        escape_json_scalar(fund_payload.as_str())
    );
    let fund_response = run_live_s05_mcp_tool_call(
        binary.as_str(),
        endpoint.as_str(),
        fund_agent_name.as_str(),
        key_file.as_str(),
        "probe-fund-escrow",
        "fund_escrow",
        fund_arguments.as_str(),
    )?;
    let escrow_id =
        json_optional_string_field(fund_response.as_str(), "escrow_id").ok_or_else(|| {
            format!("mcp live s05 fund_escrow response missing escrow_id field: {fund_response}")
        })?;
    if escrow_id.trim().is_empty() {
        return Err("mcp live s05 fund_escrow returned empty escrow_id".to_owned());
    }
    let fund_state =
        json_optional_string_field(fund_response.as_str(), "state").ok_or_else(|| {
            format!("mcp live s05 fund_escrow response missing state field: {fund_response}")
        })?;
    if fund_state.trim().is_empty() {
        return Err("mcp live s05 fund_escrow returned empty state".to_owned());
    }

    let release_arguments = format!(
        "{{\"escrow_id\":\"{}\"}}",
        escape_json_scalar(escrow_id.as_str())
    );
    let release_response = run_live_s05_mcp_tool_call(
        binary.as_str(),
        endpoint.as_str(),
        release_agent_name.as_str(),
        key_file.as_str(),
        "probe-release-escrow",
        "release_escrow",
        release_arguments.as_str(),
    )?;
    let released_escrow_id = json_optional_string_field(release_response.as_str(), "escrow_id")
        .ok_or_else(|| {
            format!(
                "mcp live s05 release_escrow response missing escrow_id field: {release_response}"
            )
        })?;
    let release_state =
        json_optional_string_field(release_response.as_str(), "state").ok_or_else(|| {
            format!("mcp live s05 release_escrow response missing state field: {release_response}")
        })?;
    validate_live_s05_release_escrow_response(
        escrow_id.as_str(),
        released_escrow_id.as_str(),
        release_state.as_str(),
        "mcp live s05 release_escrow",
    )?;

    Ok(())
}

fn run_live_s06_mcp_proof_verification_probe() -> Result<(), String> {
    let binary = env_var_or_default(MCP_AGENT_BINARY_ENV, DEFAULT_MCP_AGENT_BINARY);
    let endpoint = env_var_or_default("KAMN_ENDPOINT", DEFAULT_KAMN_ENDPOINT);
    let agent_name = env_var_or_default("KAMN_AGENT_NAME", DEFAULT_MCP_AGENT_NAME);
    let key_file = env_var_or_default("KAMN_AGENT_KEY_FILE", DEFAULT_MCP_AGENT_KEY_FILE);
    let message_id = env::var("KAMN_E2E_S06_PROOF_MESSAGE_ID")
        .unwrap_or_else(|_| DEFAULT_S06_MESSAGE_ID.to_owned());
    let tx_hash = env_var_or_default("KAMN_E2E_S06_PROOF_TX_HASH", DEFAULT_S06_TX_HASH);
    let block_height = env::var("KAMN_E2E_S06_PROOF_BLOCK_HEIGHT")
        .ok()
        .map(|raw| {
            raw.trim()
                .parse::<u64>()
                .map_err(|_| format!("mcp live s06 invalid block height env value: {raw}"))
        })
        .transpose()?
        .unwrap_or(DEFAULT_S06_BLOCK_HEIGHT);
    let finality = env_var_or_default("KAMN_E2E_S06_PROOF_FINALITY", DEFAULT_S06_FINALITY);

    let proof_arguments = format!(
        "{{\"message_id\":\"{}\",\"tx_hash\":\"{}\",\"block_height\":\"{}\",\"finality\":\"{}\"}}",
        escape_json_scalar(message_id.as_str()),
        escape_json_scalar(tx_hash.as_str()),
        block_height,
        escape_json_scalar(finality.as_str()),
    );
    let proof_response = run_live_s06_mcp_tool_call(
        binary.as_str(),
        endpoint.as_str(),
        agent_name.as_str(),
        key_file.as_str(),
        "probe-verify-proof",
        "verify_proof",
        proof_arguments.as_str(),
    )?;

    if !proof_response.contains(r#""verified":true"#) {
        return Err(format!(
            "mcp live s06 verify_proof returned verified=false payload: {proof_response}"
        ));
    }
    let proof_finality = json_optional_string_field(proof_response.as_str(), "finality")
        .ok_or_else(|| {
            format!("mcp live s06 verify_proof response missing finality field: {proof_response}")
        })?;
    if proof_finality.trim() != "FINAL" {
        return Err(format!(
            "mcp live s06 verify_proof returned non-final finality: {proof_finality}"
        ));
    }
    let proof_block_height = json_optional_u64_field(proof_response.as_str(), "block_height")
        .ok_or_else(|| {
            format!(
                "mcp live s06 verify_proof response missing block_height field: {proof_response}"
            )
        })?;
    if proof_block_height == 0 {
        return Err("mcp live s06 verify_proof returned block_height=0".to_owned());
    }

    Ok(())
}

fn run_live_s07_mcp_replay_protection_probe() -> Result<(), String> {
    let binary = env_var_or_default(MCP_AGENT_BINARY_ENV, DEFAULT_MCP_AGENT_BINARY);
    let endpoint = env_var_or_default("KAMN_ENDPOINT", DEFAULT_KAMN_ENDPOINT);
    let key_file = env_var_or_default("KAMN_AGENT_KEY_FILE", DEFAULT_MCP_AGENT_KEY_FILE);
    let base_agent_name = env_var_or_default("KAMN_E2E_S07_AGENT_NAME", DEFAULT_S07_AGENT_NAME);
    let message_payload = env::var("KAMN_E2E_S07_REPLAY_PAYLOAD")
        .unwrap_or_else(|_| DEFAULT_S07_MESSAGE_PAYLOAD.to_owned());
    let replay_agent_name = format!(
        "{base_agent_name}-{}",
        live_s07_probe_agent_suffix().as_str()
    );

    let send_arguments = format!(
        "{{\"payload\":\"{}\"}}",
        escape_json_scalar(message_payload.as_str())
    );
    let first_response = run_live_s07_mcp_tool_call(
        binary.as_str(),
        endpoint.as_str(),
        replay_agent_name.as_str(),
        key_file.as_str(),
        "probe-send-message-initial",
        "send_message",
        send_arguments.as_str(),
    )?;
    let message_id =
        json_optional_string_field(first_response.as_str(), "message_id").ok_or_else(|| {
            format!(
                "mcp live s07 initial send_message response missing message_id field: {first_response}"
            )
        })?;
    if message_id.trim().is_empty() {
        return Err("mcp live s07 initial send_message returned empty message_id".to_owned());
    }
    let send_status =
        json_optional_string_field(first_response.as_str(), "status").ok_or_else(|| {
            format!(
                "mcp live s07 initial send_message response missing status field: {first_response}"
            )
        })?;
    if send_status.trim().is_empty() {
        return Err("mcp live s07 initial send_message returned empty status".to_owned());
    }

    let replay_error = run_live_s07_mcp_tool_call(
        binary.as_str(),
        endpoint.as_str(),
        replay_agent_name.as_str(),
        key_file.as_str(),
        "probe-send-message-replay",
        "send_message",
        send_arguments.as_str(),
    )
    .err()
    .ok_or_else(|| "mcp live s07 replay send_message unexpectedly succeeded".to_owned())?;
    validate_s07_replay_reason_marker(replay_error.as_str(), "mcp live s07 replay send_message")?;

    Ok(())
}

fn run_live_s08_mcp_crash_recovery_probe() -> Result<(), String> {
    let binary = env_var_or_default(MCP_AGENT_BINARY_ENV, DEFAULT_MCP_AGENT_BINARY);
    let endpoint = env_var_or_default("KAMN_ENDPOINT", DEFAULT_KAMN_ENDPOINT);
    let key_file = env_var_or_default("KAMN_AGENT_KEY_FILE", DEFAULT_MCP_AGENT_KEY_FILE);
    let base_agent_name = env_var_or_default("KAMN_E2E_S08_AGENT_NAME", DEFAULT_S08_AGENT_NAME);
    let pre_message_payload = env::var("KAMN_E2E_S08_PRE_MESSAGE_PAYLOAD")
        .unwrap_or_else(|_| DEFAULT_S08_PRE_MESSAGE_PAYLOAD.to_owned());
    let post_message_payload = env::var("KAMN_E2E_S08_POST_MESSAGE_PAYLOAD")
        .unwrap_or_else(|_| DEFAULT_S08_POST_MESSAGE_PAYLOAD.to_owned());

    let pre_send_arguments = format!(
        "{{\"payload\":\"{}\"}}",
        escape_json_scalar(pre_message_payload.as_str())
    );
    let pre_send_response = run_live_s08_mcp_tool_call(
        binary.as_str(),
        endpoint.as_str(),
        format!("{base_agent_name}-pre-send").as_str(),
        key_file.as_str(),
        "probe-send-message-pre",
        "send_message",
        pre_send_arguments.as_str(),
    )?;
    let pre_message_id = validate_s08_mcp_message_receipt_fields(
        pre_send_response.as_str(),
        "mcp live s08 pre-boundary send_message",
    )?;

    let pre_query_arguments = format!(
        "{{\"message_id\":\"{}\"}}",
        escape_json_scalar(pre_message_id.as_str())
    );
    let pre_query_response = run_live_s08_mcp_tool_call(
        binary.as_str(),
        endpoint.as_str(),
        format!("{base_agent_name}-pre-query").as_str(),
        key_file.as_str(),
        "probe-query-message-pre",
        "query_message",
        pre_query_arguments.as_str(),
    )?;
    validate_s08_mcp_query_message_response(
        pre_query_response.as_str(),
        pre_message_id.as_str(),
        "mcp live s08 pre-boundary query_message",
    )?;

    run_live_s08_mcp_tool_call(
        binary.as_str(),
        endpoint.as_str(),
        format!("{base_agent_name}-boundary").as_str(),
        key_file.as_str(),
        "probe-boundary-health",
        "health",
        "{}",
    )?;

    let post_send_arguments = format!(
        "{{\"payload\":\"{}\"}}",
        escape_json_scalar(post_message_payload.as_str())
    );
    let post_send_response = run_live_s08_mcp_tool_call(
        binary.as_str(),
        endpoint.as_str(),
        format!("{base_agent_name}-post-send").as_str(),
        key_file.as_str(),
        "probe-send-message-post",
        "send_message",
        post_send_arguments.as_str(),
    )?;
    let post_message_id = validate_s08_mcp_message_receipt_fields(
        post_send_response.as_str(),
        "mcp live s08 post-boundary send_message",
    )?;
    if post_message_id == pre_message_id {
        return Err(
            "mcp live s08 post-boundary send_message returned duplicate message_id".to_owned(),
        );
    }

    let post_query_arguments = format!(
        "{{\"message_id\":\"{}\"}}",
        escape_json_scalar(post_message_id.as_str())
    );
    let post_query_response = run_live_s08_mcp_tool_call(
        binary.as_str(),
        endpoint.as_str(),
        format!("{base_agent_name}-post-query").as_str(),
        key_file.as_str(),
        "probe-query-message-post",
        "query_message",
        post_query_arguments.as_str(),
    )?;
    validate_s08_mcp_query_message_response(
        post_query_response.as_str(),
        post_message_id.as_str(),
        "mcp live s08 post-boundary query_message",
    )?;

    Ok(())
}

fn run_live_s09_mcp_transport_failover_probe() -> Result<(), String> {
    let binary = env_var_or_default(MCP_AGENT_BINARY_ENV, DEFAULT_MCP_AGENT_BINARY);
    let primary_endpoint = env_var_or_default("KAMN_ENDPOINT", DEFAULT_KAMN_ENDPOINT);
    let failover_endpoint = env_var_or_else("KAMN_E2E_S09_FAILOVER_ENDPOINT", || {
        primary_endpoint.clone()
    });
    let key_file = env_var_or_default("KAMN_AGENT_KEY_FILE", DEFAULT_MCP_AGENT_KEY_FILE);
    let base_agent_name = env_var_or_default("KAMN_E2E_S09_AGENT_NAME", DEFAULT_S09_AGENT_NAME);
    let pre_message_payload = env::var("KAMN_E2E_S09_PRE_MESSAGE_PAYLOAD")
        .unwrap_or_else(|_| DEFAULT_S09_PRE_MESSAGE_PAYLOAD.to_owned());
    let post_message_payload = env::var("KAMN_E2E_S09_POST_MESSAGE_PAYLOAD")
        .unwrap_or_else(|_| DEFAULT_S09_POST_MESSAGE_PAYLOAD.to_owned());

    let pre_send_arguments = format!(
        "{{\"payload\":\"{}\"}}",
        escape_json_scalar(pre_message_payload.as_str())
    );
    let pre_send_response = run_live_s09_mcp_tool_call(
        binary.as_str(),
        primary_endpoint.as_str(),
        format!("{base_agent_name}-pre-send").as_str(),
        key_file.as_str(),
        "probe-send-message-pre",
        "send_message",
        pre_send_arguments.as_str(),
    )?;
    let pre_message_id = validate_s08_mcp_message_receipt_fields(
        pre_send_response.as_str(),
        "mcp live s09 pre-failover send_message",
    )?;

    let pre_query_arguments = format!(
        "{{\"message_id\":\"{}\"}}",
        escape_json_scalar(pre_message_id.as_str())
    );
    let pre_query_response = run_live_s09_mcp_tool_call(
        binary.as_str(),
        primary_endpoint.as_str(),
        format!("{base_agent_name}-pre-query").as_str(),
        key_file.as_str(),
        "probe-query-message-pre",
        "query_message",
        pre_query_arguments.as_str(),
    )?;
    validate_s08_mcp_query_message_response(
        pre_query_response.as_str(),
        pre_message_id.as_str(),
        "mcp live s09 pre-failover query_message",
    )?;

    let boundary_response = run_live_s09_mcp_tool_call(
        binary.as_str(),
        failover_endpoint.as_str(),
        format!("{base_agent_name}-boundary").as_str(),
        key_file.as_str(),
        "probe-boundary-health",
        "health",
        "{}",
    )?;
    let boundary_status =
        json_optional_string_field(boundary_response.as_str(), "status").ok_or_else(|| {
            format!(
                "mcp live s09 failover boundary health response missing status field: {boundary_response}"
            )
        })?;
    if boundary_status.trim().is_empty() {
        return Err("mcp live s09 failover boundary health returned empty status".to_owned());
    }

    let post_send_arguments = format!(
        "{{\"payload\":\"{}\"}}",
        escape_json_scalar(post_message_payload.as_str())
    );
    let post_send_response = run_live_s09_mcp_tool_call(
        binary.as_str(),
        failover_endpoint.as_str(),
        format!("{base_agent_name}-post-send").as_str(),
        key_file.as_str(),
        "probe-send-message-post",
        "send_message",
        post_send_arguments.as_str(),
    )?;
    let post_message_id = validate_s08_mcp_message_receipt_fields(
        post_send_response.as_str(),
        "mcp live s09 post-failover send_message",
    )?;
    if post_message_id == pre_message_id {
        return Err(
            "mcp live s09 post-failover send_message returned duplicate message_id".to_owned(),
        );
    }

    let post_query_arguments = format!(
        "{{\"message_id\":\"{}\"}}",
        escape_json_scalar(post_message_id.as_str())
    );
    let post_query_response = run_live_s09_mcp_tool_call(
        binary.as_str(),
        failover_endpoint.as_str(),
        format!("{base_agent_name}-post-query").as_str(),
        key_file.as_str(),
        "probe-query-message-post",
        "query_message",
        post_query_arguments.as_str(),
    )?;
    validate_s08_mcp_query_message_response(
        post_query_response.as_str(),
        post_message_id.as_str(),
        "mcp live s09 post-failover query_message",
    )?;

    Ok(())
}

fn run_live_s10_mcp_topology_coherence_probe() -> Result<(), String> {
    let binary = env_var_or_default(MCP_AGENT_BINARY_ENV, DEFAULT_MCP_AGENT_BINARY);
    let primary_endpoint = env::var("KAMN_E2E_S10_PRIMARY_ENDPOINT")
        .or_else(|_| env::var("KAMN_ENDPOINT"))
        .unwrap_or_else(|_| DEFAULT_KAMN_ENDPOINT.to_owned());
    let secondary_endpoint = env_var_or_else("KAMN_E2E_S10_SECONDARY_ENDPOINT", || {
        primary_endpoint.clone()
    });
    let tertiary_endpoint = env_var_or_else("KAMN_E2E_S10_TERTIARY_ENDPOINT", || {
        secondary_endpoint.clone()
    });
    let key_file = env_var_or_default("KAMN_AGENT_KEY_FILE", DEFAULT_MCP_AGENT_KEY_FILE);
    let base_agent_name = env_var_or_default("KAMN_E2E_S10_AGENT_NAME", DEFAULT_S10_AGENT_NAME);
    let message_payload = env::var("KAMN_E2E_S10_MESSAGE_PAYLOAD")
        .unwrap_or_else(|_| DEFAULT_S10_MESSAGE_PAYLOAD.to_owned());

    let send_arguments = format!(
        "{{\"payload\":\"{}\"}}",
        escape_json_scalar(message_payload.as_str())
    );
    let primary_send_response = run_live_s10_mcp_tool_call(
        binary.as_str(),
        primary_endpoint.as_str(),
        format!("{base_agent_name}-primary-send").as_str(),
        key_file.as_str(),
        "probe-send-message-primary",
        "send_message",
        send_arguments.as_str(),
    )?;
    let message_id = validate_s08_mcp_message_receipt_fields(
        primary_send_response.as_str(),
        "mcp live s10 primary send_message",
    )?;

    let query_arguments = format!(
        "{{\"message_id\":\"{}\"}}",
        escape_json_scalar(message_id.as_str())
    );
    let secondary_query_response = run_live_s10_mcp_tool_call(
        binary.as_str(),
        secondary_endpoint.as_str(),
        format!("{base_agent_name}-secondary-query").as_str(),
        key_file.as_str(),
        "probe-query-message-secondary",
        "query_message",
        query_arguments.as_str(),
    )?;
    validate_s08_mcp_query_message_response(
        secondary_query_response.as_str(),
        message_id.as_str(),
        "mcp live s10 secondary query_message",
    )?;

    let tertiary_query_response = run_live_s10_mcp_tool_call(
        binary.as_str(),
        tertiary_endpoint.as_str(),
        format!("{base_agent_name}-tertiary-query").as_str(),
        key_file.as_str(),
        "probe-query-message-tertiary",
        "query_message",
        query_arguments.as_str(),
    )?;
    validate_s08_mcp_query_message_response(
        tertiary_query_response.as_str(),
        message_id.as_str(),
        "mcp live s10 tertiary query_message",
    )?;

    let secondary_health_response = run_live_s10_mcp_tool_call(
        binary.as_str(),
        secondary_endpoint.as_str(),
        format!("{base_agent_name}-secondary-boundary").as_str(),
        key_file.as_str(),
        "probe-health-secondary",
        "health",
        "{}",
    )?;
    let secondary_health_status =
        json_optional_string_field(secondary_health_response.as_str(), "status").ok_or_else(
            || {
                format!(
                    "mcp live s10 secondary health response missing status field: {secondary_health_response}"
                )
            },
        )?;
    if secondary_health_status.trim().is_empty() {
        return Err("mcp live s10 secondary health check returned empty status".to_owned());
    }

    let tertiary_health_response = run_live_s10_mcp_tool_call(
        binary.as_str(),
        tertiary_endpoint.as_str(),
        format!("{base_agent_name}-tertiary-boundary").as_str(),
        key_file.as_str(),
        "probe-health-tertiary",
        "health",
        "{}",
    )?;
    let tertiary_health_status =
        json_optional_string_field(tertiary_health_response.as_str(), "status").ok_or_else(
            || {
                format!(
                    "mcp live s10 tertiary health response missing status field: {tertiary_health_response}"
                )
            },
        )?;
    if tertiary_health_status.trim().is_empty() {
        return Err("mcp live s10 tertiary health check returned empty status".to_owned());
    }

    Ok(())
}

fn run_live_s11_mcp_signer_rotation_probe() -> Result<(), String> {
    let binary = env_var_or_default(MCP_AGENT_BINARY_ENV, DEFAULT_MCP_AGENT_BINARY);
    let endpoint = env_var_or_default("KAMN_ENDPOINT", DEFAULT_KAMN_ENDPOINT);
    let key_file = env_var_or_default("KAMN_AGENT_KEY_FILE", DEFAULT_MCP_AGENT_KEY_FILE);
    let primary_agent_name = env::var("KAMN_E2E_S11_PRIMARY_AGENT_NAME")
        .unwrap_or_else(|_| DEFAULT_S11_PRIMARY_AGENT_NAME.to_owned());
    let rotated_agent_name = env::var("KAMN_E2E_S11_ROTATED_AGENT_NAME")
        .unwrap_or_else(|_| format!("{primary_agent_name}-rotated"));
    let message_payload = env::var("KAMN_E2E_S11_MESSAGE_PAYLOAD")
        .unwrap_or_else(|_| DEFAULT_S11_MESSAGE_PAYLOAD.to_owned());
    let rotated_message_payload = env::var("KAMN_E2E_S11_ROTATED_MESSAGE_PAYLOAD")
        .unwrap_or_else(|_| DEFAULT_S11_ROTATED_MESSAGE_PAYLOAD.to_owned());
    let stale_message_payload = env::var("KAMN_E2E_S11_STALE_MESSAGE_PAYLOAD")
        .unwrap_or_else(|_| DEFAULT_S11_STALE_MESSAGE_PAYLOAD.to_owned());

    let primary_send_arguments = format!(
        "{{\"payload\":\"{}\"}}",
        escape_json_scalar(message_payload.as_str())
    );
    let primary_send_response = run_live_s11_mcp_tool_call(
        binary.as_str(),
        endpoint.as_str(),
        primary_agent_name.as_str(),
        key_file.as_str(),
        "probe-send-message-primary",
        "send_message",
        primary_send_arguments.as_str(),
    )?;
    let primary_message_id = validate_s08_mcp_message_receipt_fields(
        primary_send_response.as_str(),
        "mcp live s11 primary send_message",
    )?;

    let primary_query_arguments = format!(
        "{{\"message_id\":\"{}\"}}",
        escape_json_scalar(primary_message_id.as_str())
    );
    let primary_query_response = run_live_s11_mcp_tool_call(
        binary.as_str(),
        endpoint.as_str(),
        format!("{primary_agent_name}-query").as_str(),
        key_file.as_str(),
        "probe-query-message-primary",
        "query_message",
        primary_query_arguments.as_str(),
    )?;
    validate_s08_mcp_query_message_response(
        primary_query_response.as_str(),
        primary_message_id.as_str(),
        "mcp live s11 primary query_message",
    )?;

    let rotated_send_arguments = format!(
        "{{\"payload\":\"{}\"}}",
        escape_json_scalar(rotated_message_payload.as_str())
    );
    let rotated_send_response = run_live_s11_mcp_tool_call(
        binary.as_str(),
        endpoint.as_str(),
        rotated_agent_name.as_str(),
        key_file.as_str(),
        "probe-send-message-rotated",
        "send_message",
        rotated_send_arguments.as_str(),
    )?;
    let rotated_message_id = validate_s08_mcp_message_receipt_fields(
        rotated_send_response.as_str(),
        "mcp live s11 rotated send_message",
    )?;
    if rotated_message_id == primary_message_id {
        return Err("mcp live s11 rotated send_message returned duplicate message_id".to_owned());
    }

    let rotated_query_arguments = format!(
        "{{\"message_id\":\"{}\"}}",
        escape_json_scalar(rotated_message_id.as_str())
    );
    let rotated_query_response = run_live_s11_mcp_tool_call(
        binary.as_str(),
        endpoint.as_str(),
        format!("{rotated_agent_name}-query").as_str(),
        key_file.as_str(),
        "probe-query-message-rotated",
        "query_message",
        rotated_query_arguments.as_str(),
    )?;
    validate_s08_mcp_query_message_response(
        rotated_query_response.as_str(),
        rotated_message_id.as_str(),
        "mcp live s11 rotated query_message",
    )?;

    let stale_send_arguments = format!(
        "{{\"payload\":\"{}\"}}",
        escape_json_scalar(stale_message_payload.as_str())
    );
    let stale_primary_error = run_live_s11_mcp_tool_call(
        binary.as_str(),
        endpoint.as_str(),
        primary_agent_name.as_str(),
        key_file.as_str(),
        "probe-send-message-stale-primary",
        "send_message",
        stale_send_arguments.as_str(),
    )
    .err()
    .ok_or_else(|| "mcp live s11 stale-primary send_message unexpectedly succeeded".to_owned())?;
    validate_s07_replay_reason_marker(
        stale_primary_error.as_str(),
        "mcp live s11 stale-primary send_message",
    )?;

    Ok(())
}

fn run_live_s12_mcp_retention_deletion_probe() -> Result<(), String> {
    let binary = env_var_or_default(MCP_AGENT_BINARY_ENV, DEFAULT_MCP_AGENT_BINARY);
    let endpoint = env_var_or_default("KAMN_ENDPOINT", DEFAULT_KAMN_ENDPOINT);
    let key_file = env_var_or_default("KAMN_AGENT_KEY_FILE", DEFAULT_MCP_AGENT_KEY_FILE);
    let base_agent_name = env_var_or_default("KAMN_E2E_S12_AGENT_NAME", DEFAULT_S12_AGENT_NAME);
    let register_payload = env::var("KAMN_E2E_S12_REGISTER_CONTENT_PAYLOAD")
        .unwrap_or_else(|_| DEFAULT_S12_REGISTER_CONTENT_PAYLOAD.to_owned());

    let register_arguments = format!(
        "{{\"payload\":\"{}\"}}",
        escape_json_scalar(register_payload.as_str())
    );
    let register_response = run_live_s12_mcp_tool_call(
        binary.as_str(),
        endpoint.as_str(),
        format!("{base_agent_name}-register").as_str(),
        key_file.as_str(),
        "probe-register-content",
        "register_content",
        register_arguments.as_str(),
    )?;
    let content_id =
        json_optional_string_field(register_response.as_str(), "content_id").ok_or_else(|| {
            format!(
                "mcp live s12 register_content response missing content_id field: {register_response}"
            )
        })?;
    if content_id.trim().is_empty() {
        return Err("mcp live s12 register_content returned empty content_id".to_owned());
    }
    let retention_class =
        json_optional_string_field(register_response.as_str(), "retention_class").ok_or_else(
            || {
                format!(
                    "mcp live s12 register_content response missing retention_class field: {register_response}"
                )
            },
        )?;
    if retention_class.trim().is_empty() {
        return Err("mcp live s12 register_content returned empty retention_class".to_owned());
    }

    let expire_arguments = format!(
        "{{\"content_id\":\"{}\"}}",
        escape_json_scalar(content_id.as_str())
    );
    let expire_response = run_live_s12_mcp_tool_call(
        binary.as_str(),
        endpoint.as_str(),
        format!("{base_agent_name}-expire").as_str(),
        key_file.as_str(),
        "probe-expire-content",
        "expire_content",
        expire_arguments.as_str(),
    )?;
    let expired_content_id = json_optional_string_field(expire_response.as_str(), "content_id")
        .ok_or_else(|| {
            format!(
                "mcp live s12 expire_content response missing content_id field: {expire_response}"
            )
        })?;
    validate_s12_content_id_match(
        content_id.as_str(),
        expired_content_id.as_str(),
        "mcp live s12 expire_content",
    )?;
    let expired_lifecycle_state =
        json_optional_string_field(expire_response.as_str(), "lifecycle_state").ok_or_else(
            || {
                format!(
            "mcp live s12 expire_content response missing lifecycle_state field: {expire_response}"
        )
            },
        )?;
    if expired_lifecycle_state.trim().is_empty() {
        return Err("mcp live s12 expire_content returned empty lifecycle_state".to_owned());
    }

    let tombstone_arguments = format!(
        "{{\"content_id\":\"{}\"}}",
        escape_json_scalar(content_id.as_str())
    );
    let tombstone_response = run_live_s12_mcp_tool_call(
        binary.as_str(),
        endpoint.as_str(),
        format!("{base_agent_name}-tombstone").as_str(),
        key_file.as_str(),
        "probe-tombstone-content",
        "tombstone_content",
        tombstone_arguments.as_str(),
    )?;
    let tombstoned_content_id =
        json_optional_string_field(tombstone_response.as_str(), "content_id").ok_or_else(|| {
            format!(
                "mcp live s12 tombstone_content response missing content_id field: {tombstone_response}"
            )
        })?;
    validate_s12_content_id_match(
        content_id.as_str(),
        tombstoned_content_id.as_str(),
        "mcp live s12 tombstone_content",
    )?;
    let tombstoned_lifecycle_state =
        json_optional_string_field(tombstone_response.as_str(), "lifecycle_state").ok_or_else(
            || {
                format!(
                    "mcp live s12 tombstone_content response missing lifecycle_state field: {tombstone_response}"
                )
            },
        )?;
    if tombstoned_lifecycle_state.trim().is_empty() {
        return Err("mcp live s12 tombstone_content returned empty lifecycle_state".to_owned());
    }
    let tombstoned_redaction_status =
        json_optional_string_field(tombstone_response.as_str(), "redaction_status").ok_or_else(
            || {
                format!(
                    "mcp live s12 tombstone_content response missing redaction_status field: {tombstone_response}"
                )
            },
        )?;
    if tombstoned_redaction_status.trim().is_empty() {
        return Err("mcp live s12 tombstone_content returned empty redaction_status".to_owned());
    }

    let query_arguments = format!(
        "{{\"content_id\":\"{}\"}}",
        escape_json_scalar(content_id.as_str())
    );
    let query_response = run_live_s12_mcp_tool_call(
        binary.as_str(),
        endpoint.as_str(),
        format!("{base_agent_name}-query").as_str(),
        key_file.as_str(),
        "probe-query-content",
        "query_content",
        query_arguments.as_str(),
    )?;
    let queried_content_id = json_optional_string_field(query_response.as_str(), "content_id")
        .ok_or_else(|| {
            format!(
                "mcp live s12 query_content response missing content_id field: {query_response}"
            )
        })?;
    validate_s12_content_id_match(
        content_id.as_str(),
        queried_content_id.as_str(),
        "mcp live s12 query_content",
    )?;
    let queried_lifecycle_state =
        json_optional_string_field(query_response.as_str(), "lifecycle_state").ok_or_else(
            || {
                format!(
            "mcp live s12 query_content response missing lifecycle_state field: {query_response}"
        )
            },
        )?;
    validate_s12_content_field_coherence(
        tombstoned_lifecycle_state.as_str(),
        queried_lifecycle_state.as_str(),
        "lifecycle_state",
        "mcp live s12 query_content",
    )?;
    let queried_redaction_status =
        json_optional_string_field(query_response.as_str(), "redaction_status").ok_or_else(
            || {
                format!(
            "mcp live s12 query_content response missing redaction_status field: {query_response}"
        )
            },
        )?;
    validate_s12_content_field_coherence(
        tombstoned_redaction_status.as_str(),
        queried_redaction_status.as_str(),
        "redaction_status",
        "mcp live s12 query_content",
    )?;

    Ok(())
}

fn run_live_s13_mcp_bridge_forwarding_probe() -> Result<(), String> {
    let binary = env_var_or_default(MCP_AGENT_BINARY_ENV, DEFAULT_MCP_AGENT_BINARY);
    let endpoint = env_var_or_default("KAMN_ENDPOINT", DEFAULT_KAMN_ENDPOINT);
    let key_file = env_var_or_default("KAMN_AGENT_KEY_FILE", DEFAULT_MCP_AGENT_KEY_FILE);
    let base_agent_name = env_var_or_default("KAMN_E2E_S13_AGENT_NAME", DEFAULT_S13_AGENT_NAME);
    let submit_payload = env::var("KAMN_E2E_S13_SUBMIT_BRIDGE_PAYLOAD")
        .unwrap_or_else(|_| DEFAULT_S13_SUBMIT_BRIDGE_PAYLOAD.to_owned());

    let submit_arguments = format!(
        "{{\"payload\":\"{}\"}}",
        escape_json_scalar(submit_payload.as_str())
    );
    let submit_response = run_live_s13_mcp_tool_call(
        binary.as_str(),
        endpoint.as_str(),
        format!("{base_agent_name}-submit").as_str(),
        key_file.as_str(),
        "probe-submit-bridge-message",
        "submit_bridge_message",
        submit_arguments.as_str(),
    )?;
    let bridge_id = json_optional_string_field(submit_response.as_str(), "bridge_id").ok_or_else(
        || {
            format!(
                "mcp live s13 submit_bridge_message response missing bridge_id field: {submit_response}"
            )
        },
    )?;
    if bridge_id.trim().is_empty() {
        return Err("mcp live s13 submit_bridge_message returned empty bridge_id".to_owned());
    }
    let source_message_id =
        json_optional_string_field(submit_response.as_str(), "source_message_id").ok_or_else(
            || {
                format!(
                    "mcp live s13 submit_bridge_message response missing source_message_id field: {submit_response}"
                )
            },
        )?;
    if source_message_id.trim().is_empty() {
        return Err(
            "mcp live s13 submit_bridge_message returned empty source_message_id".to_owned(),
        );
    }
    let submit_bridge_status =
        json_optional_string_field(submit_response.as_str(), "bridge_status").ok_or_else(|| {
            format!(
                "mcp live s13 submit_bridge_message response missing bridge_status field: {submit_response}"
            )
        })?;
    if submit_bridge_status.trim().is_empty() {
        return Err("mcp live s13 submit_bridge_message returned empty bridge_status".to_owned());
    }

    let forward_arguments = format!(
        "{{\"bridge_id\":\"{}\"}}",
        escape_json_scalar(bridge_id.as_str())
    );
    let forward_response = run_live_s13_mcp_tool_call(
        binary.as_str(),
        endpoint.as_str(),
        format!("{base_agent_name}-forward").as_str(),
        key_file.as_str(),
        "probe-forward-bridge-message",
        "forward_bridge_message",
        forward_arguments.as_str(),
    )?;
    let forwarded_bridge_id = json_optional_string_field(forward_response.as_str(), "bridge_id")
        .ok_or_else(|| {
            format!(
                "mcp live s13 forward_bridge_message response missing bridge_id field: {forward_response}"
            )
        })?;
    validate_s13_bridge_id_match(
        bridge_id.as_str(),
        forwarded_bridge_id.as_str(),
        "mcp live s13 forward_bridge_message",
    )?;
    let forwarded_bridge_status =
        json_optional_string_field(forward_response.as_str(), "bridge_status").ok_or_else(|| {
            format!(
                "mcp live s13 forward_bridge_message response missing bridge_status field: {forward_response}"
            )
        })?;
    if forwarded_bridge_status.trim().is_empty() {
        return Err("mcp live s13 forward_bridge_message returned empty bridge_status".to_owned());
    }
    let forwarded_target_message_id = json_optional_string_field(
        forward_response.as_str(),
        "target_message_id",
    )
    .ok_or_else(|| {
        format!(
            "mcp live s13 forward_bridge_message response missing target_message_id field: {forward_response}"
        )
    })?;
    if forwarded_target_message_id.trim().is_empty() {
        return Err(
            "mcp live s13 forward_bridge_message returned empty target_message_id".to_owned(),
        );
    }
    let forwarded_tx_hash =
        json_optional_string_field(forward_response.as_str(), "forward_tx_hash").ok_or_else(
            || {
                format!(
                    "mcp live s13 forward_bridge_message response missing forward_tx_hash field: {forward_response}"
                )
            },
        )?;
    if forwarded_tx_hash.trim().is_empty() {
        return Err(
            "mcp live s13 forward_bridge_message returned empty forward_tx_hash".to_owned(),
        );
    }

    let query_arguments = format!(
        "{{\"bridge_id\":\"{}\"}}",
        escape_json_scalar(bridge_id.as_str())
    );
    let query_response = run_live_s13_mcp_tool_call(
        binary.as_str(),
        endpoint.as_str(),
        format!("{base_agent_name}-query").as_str(),
        key_file.as_str(),
        "probe-query-bridge-message",
        "query_bridge_message",
        query_arguments.as_str(),
    )?;
    let queried_bridge_id = json_optional_string_field(query_response.as_str(), "bridge_id")
        .ok_or_else(|| {
            format!(
                "mcp live s13 query_bridge_message response missing bridge_id field: {query_response}"
            )
        })?;
    validate_s13_bridge_id_match(
        bridge_id.as_str(),
        queried_bridge_id.as_str(),
        "mcp live s13 query_bridge_message",
    )?;
    let queried_bridge_status =
        json_optional_string_field(query_response.as_str(), "bridge_status").ok_or_else(|| {
            format!(
                "mcp live s13 query_bridge_message response missing bridge_status field: {query_response}"
            )
        })?;
    validate_s13_bridge_field_coherence(
        forwarded_bridge_status.as_str(),
        queried_bridge_status.as_str(),
        "bridge_status",
        "mcp live s13 query_bridge_message",
    )?;
    let queried_target_message_id = json_optional_string_field(
        query_response.as_str(),
        "target_message_id",
    )
    .ok_or_else(|| {
        format!(
            "mcp live s13 query_bridge_message response missing target_message_id field: {query_response}"
        )
    })?;
    validate_s13_bridge_field_coherence(
        forwarded_target_message_id.as_str(),
        queried_target_message_id.as_str(),
        "target_message_id",
        "mcp live s13 query_bridge_message",
    )?;
    let queried_tx_hash =
        json_optional_string_field(query_response.as_str(), "forward_tx_hash").ok_or_else(|| {
            format!(
                "mcp live s13 query_bridge_message response missing forward_tx_hash field: {query_response}"
            )
        })?;
    validate_s13_bridge_field_coherence(
        forwarded_tx_hash.as_str(),
        queried_tx_hash.as_str(),
        "forward_tx_hash",
        "mcp live s13 query_bridge_message",
    )?;

    Ok(())
}

fn run_live_s14_mcp_batch_merkle_probe() -> Result<(), String> {
    let binary = env_var_or_default(MCP_AGENT_BINARY_ENV, DEFAULT_MCP_AGENT_BINARY);
    let endpoint = env_var_or_default("KAMN_ENDPOINT", DEFAULT_KAMN_ENDPOINT);
    let key_file = env_var_or_default("KAMN_AGENT_KEY_FILE", DEFAULT_MCP_AGENT_KEY_FILE);
    let base_agent_name = env_var_or_default("KAMN_E2E_S14_AGENT_NAME", DEFAULT_S14_AGENT_NAME);
    let batch_message_payload_a = env::var("KAMN_E2E_S14_BATCH_MESSAGE_PAYLOAD_A")
        .unwrap_or_else(|_| DEFAULT_S14_BATCH_MESSAGE_PAYLOAD_A.to_owned());
    let batch_message_payload_b = env::var("KAMN_E2E_S14_BATCH_MESSAGE_PAYLOAD_B")
        .unwrap_or_else(|_| DEFAULT_S14_BATCH_MESSAGE_PAYLOAD_B.to_owned());
    let block_height = env::var("KAMN_E2E_S14_BLOCK_HEIGHT")
        .ok()
        .map(|raw| {
            raw.trim()
                .parse::<u64>()
                .map_err(|_| format!("mcp live s14 invalid block height env value: {raw}"))
        })
        .transpose()?
        .unwrap_or(DEFAULT_S14_BLOCK_HEIGHT);
    let finality = env_var_or_default("KAMN_E2E_S14_FINALITY", DEFAULT_S14_FINALITY);

    let batch_a_send_arguments = format!(
        "{{\"payload\":\"{}\"}}",
        escape_json_scalar(batch_message_payload_a.as_str())
    );
    let batch_a_send_response = run_live_s14_mcp_tool_call(
        binary.as_str(),
        endpoint.as_str(),
        format!("{base_agent_name}-batch-a").as_str(),
        key_file.as_str(),
        "probe-send-message-batch-a",
        "send_message",
        batch_a_send_arguments.as_str(),
    )?;
    let batch_a_message_id = validate_s08_mcp_message_receipt_fields(
        batch_a_send_response.as_str(),
        "mcp live s14 batch-a send_message",
    )?;

    let batch_b_send_arguments = format!(
        "{{\"payload\":\"{}\"}}",
        escape_json_scalar(batch_message_payload_b.as_str())
    );
    let batch_b_send_response = run_live_s14_mcp_tool_call(
        binary.as_str(),
        endpoint.as_str(),
        format!("{base_agent_name}-batch-b").as_str(),
        key_file.as_str(),
        "probe-send-message-batch-b",
        "send_message",
        batch_b_send_arguments.as_str(),
    )?;
    let batch_b_message_id = validate_s08_mcp_message_receipt_fields(
        batch_b_send_response.as_str(),
        "mcp live s14 batch-b send_message",
    )?;
    if batch_b_message_id == batch_a_message_id {
        return Err("mcp live s14 batch-b send_message returned duplicate message_id".to_owned());
    }

    let batch_a_query_arguments = format!(
        "{{\"message_id\":\"{}\"}}",
        escape_json_scalar(batch_a_message_id.as_str())
    );
    let batch_a_query_response = run_live_s14_mcp_tool_call(
        binary.as_str(),
        endpoint.as_str(),
        format!("{base_agent_name}-query-a").as_str(),
        key_file.as_str(),
        "probe-query-message-batch-a",
        "query_message",
        batch_a_query_arguments.as_str(),
    )?;
    validate_s08_mcp_query_message_response(
        batch_a_query_response.as_str(),
        batch_a_message_id.as_str(),
        "mcp live s14 batch-a query_message",
    )?;

    let batch_b_query_arguments = format!(
        "{{\"message_id\":\"{}\"}}",
        escape_json_scalar(batch_b_message_id.as_str())
    );
    let batch_b_query_response = run_live_s14_mcp_tool_call(
        binary.as_str(),
        endpoint.as_str(),
        format!("{base_agent_name}-query-b").as_str(),
        key_file.as_str(),
        "probe-query-message-batch-b",
        "query_message",
        batch_b_query_arguments.as_str(),
    )?;
    validate_s08_mcp_query_message_response(
        batch_b_query_response.as_str(),
        batch_b_message_id.as_str(),
        "mcp live s14 batch-b query_message",
    )?;

    let batch_root = env::var("KAMN_E2E_S14_BATCH_ROOT")
        .unwrap_or_else(|_| format!("sha256:s14:{}:{}", batch_a_message_id, batch_b_message_id));
    if batch_root.trim().is_empty() {
        return Err("mcp live s14 batch-root marker must not be empty".to_owned());
    }

    let batch_a_verify_arguments = format!(
        "{{\"message_id\":\"{}\",\"tx_hash\":\"{}\",\"block_height\":\"{}\",\"finality\":\"{}\"}}",
        escape_json_scalar(batch_a_message_id.as_str()),
        escape_json_scalar(batch_root.as_str()),
        block_height,
        escape_json_scalar(finality.as_str()),
    );
    let batch_a_verify_response = run_live_s14_mcp_tool_call(
        binary.as_str(),
        endpoint.as_str(),
        format!("{base_agent_name}-proof-a").as_str(),
        key_file.as_str(),
        "probe-verify-proof-batch-a",
        "verify_proof",
        batch_a_verify_arguments.as_str(),
    )?;
    validate_s14_mcp_verify_proof_response(
        batch_a_verify_response.as_str(),
        batch_a_message_id.as_str(),
        "mcp live s14 batch-a verify_proof",
    )?;

    let batch_b_verify_arguments = format!(
        "{{\"message_id\":\"{}\",\"tx_hash\":\"{}\",\"block_height\":\"{}\",\"finality\":\"{}\"}}",
        escape_json_scalar(batch_b_message_id.as_str()),
        escape_json_scalar(batch_root.as_str()),
        block_height,
        escape_json_scalar(finality.as_str()),
    );
    let batch_b_verify_response = run_live_s14_mcp_tool_call(
        binary.as_str(),
        endpoint.as_str(),
        format!("{base_agent_name}-proof-b").as_str(),
        key_file.as_str(),
        "probe-verify-proof-batch-b",
        "verify_proof",
        batch_b_verify_arguments.as_str(),
    )?;
    validate_s14_mcp_verify_proof_response(
        batch_b_verify_response.as_str(),
        batch_b_message_id.as_str(),
        "mcp live s14 batch-b verify_proof",
    )?;

    Ok(())
}

fn run_live_s15_mcp_performance_smoke_probe() -> Result<(), String> {
    let binary = env_var_or_default(MCP_AGENT_BINARY_ENV, DEFAULT_MCP_AGENT_BINARY);
    let endpoint = env_var_or_default("KAMN_ENDPOINT", DEFAULT_KAMN_ENDPOINT);
    let base_agent_name = env_var_or_default("KAMN_E2E_S15_AGENT_NAME", DEFAULT_S15_AGENT_NAME);
    let key_file = env_var_or_default("KAMN_AGENT_KEY_FILE", DEFAULT_MCP_AGENT_KEY_FILE);
    let message_payload = env::var("KAMN_E2E_S15_MESSAGE_PAYLOAD")
        .unwrap_or_else(|_| DEFAULT_S15_MESSAGE_PAYLOAD.to_owned());
    let iterations = env::var("KAMN_E2E_S15_ITERATIONS")
        .ok()
        .map(|raw| {
            raw.trim()
                .parse::<u64>()
                .map_err(|_| format!("mcp live s15 invalid iterations env value: {raw}"))
        })
        .transpose()?
        .unwrap_or(DEFAULT_S15_ITERATIONS);
    if iterations == 0 {
        return Err("mcp live s15 iterations must be greater than zero".to_owned());
    }

    let max_total_millis = parse_s15_budget_env_u128(
        "KAMN_E2E_S15_MAX_TOTAL_MILLIS",
        DEFAULT_S15_MAX_TOTAL_MILLIS,
        "mcp live s15 max-total budget",
    )?;
    let max_p50_millis = parse_s15_budget_env_u128(
        "KAMN_E2E_S15_MAX_P50_MILLIS",
        DEFAULT_S15_MAX_P50_MILLIS,
        "mcp live s15 max-p50 budget",
    )?;
    let max_p99_millis = parse_s15_budget_env_u128(
        "KAMN_E2E_S15_MAX_P99_MILLIS",
        DEFAULT_S15_MAX_P99_MILLIS,
        "mcp live s15 max-p99 budget",
    )?;

    let send_arguments = format!(
        "{{\"payload\":\"{}\"}}",
        escape_json_scalar(message_payload.as_str())
    );
    let total_start = std::time::Instant::now();
    let mut latency_samples = Vec::with_capacity(iterations as usize);
    for iteration in 0..iterations {
        let iteration_start = std::time::Instant::now();
        let send_response = run_live_s15_mcp_tool_call(
            binary.as_str(),
            endpoint.as_str(),
            format!("{base_agent_name}-send-{iteration}").as_str(),
            key_file.as_str(),
            format!("probe-send-message-s15-{iteration}").as_str(),
            "send_message",
            send_arguments.as_str(),
        )?;
        let message_id = validate_s08_mcp_message_receipt_fields(
            send_response.as_str(),
            "mcp live s15 send_message",
        )?;

        let query_arguments = format!(
            "{{\"message_id\":\"{}\"}}",
            escape_json_scalar(message_id.as_str())
        );
        let query_response = run_live_s15_mcp_tool_call(
            binary.as_str(),
            endpoint.as_str(),
            format!("{base_agent_name}-query-{iteration}").as_str(),
            key_file.as_str(),
            format!("probe-query-message-s15-{iteration}").as_str(),
            "query_message",
            query_arguments.as_str(),
        )?;
        validate_s08_mcp_query_message_response(
            query_response.as_str(),
            message_id.as_str(),
            "mcp live s15 query_message",
        )?;

        latency_samples.push(iteration_start.elapsed().as_millis());
    }
    let total_elapsed_millis = total_start.elapsed().as_millis();

    validate_s15_latency_budget_samples(
        latency_samples.as_slice(),
        total_elapsed_millis,
        max_total_millis,
        max_p50_millis,
        max_p99_millis,
        "mcp live s15 performance-smoke",
    )
}

fn validate_s14_mcp_verify_proof_response(
    response: &str,
    expected_message_id: &str,
    step: &str,
) -> Result<(), String> {
    let observed_message_id = json_optional_string_field(response, "message_id")
        .ok_or_else(|| format!("{step} response missing message_id field: {response}"))?;
    if observed_message_id != expected_message_id {
        return Err(format!(
            "{step} returned mismatched message_id: expected={expected_message_id}, got={observed_message_id}"
        ));
    }
    if !response.contains(r#""verified":true"#) {
        return Err(format!(
            "{step} returned verified=false payload: {response}"
        ));
    }
    let observed_finality = json_optional_string_field(response, "finality")
        .ok_or_else(|| format!("{step} response missing finality field: {response}"))?;
    if observed_finality != "FINAL" {
        return Err(format!(
            "{step} returned non-final finality: {observed_finality}"
        ));
    }
    let observed_block_height = json_optional_u64_field(response, "block_height")
        .ok_or_else(|| format!("{step} response missing block_height field: {response}"))?;
    if observed_block_height == 0 {
        return Err(format!("{step} returned block_height=0"));
    }
    Ok(())
}

fn validate_s08_mcp_message_receipt_fields(response: &str, step: &str) -> Result<String, String> {
    let message_id = json_optional_string_field(response, "message_id")
        .ok_or_else(|| format!("{step} response missing message_id field: {response}"))?;
    if message_id.trim().is_empty() {
        return Err(format!("{step} returned empty message_id"));
    }
    let status = json_optional_string_field(response, "status")
        .ok_or_else(|| format!("{step} response missing status field: {response}"))?;
    if status.trim().is_empty() {
        return Err(format!("{step} returned empty status"));
    }
    Ok(message_id)
}

fn validate_s08_mcp_query_message_response(
    response: &str,
    expected_message_id: &str,
    step: &str,
) -> Result<(), String> {
    let queried_message_id = json_optional_string_field(response, "message_id")
        .ok_or_else(|| format!("{step} response missing message_id field: {response}"))?;
    if queried_message_id != expected_message_id {
        return Err(format!(
            "{step} returned mismatched message_id: expected={expected_message_id}, got={queried_message_id}"
        ));
    }
    let queried_status = json_optional_string_field(response, "status")
        .ok_or_else(|| format!("{step} response missing status field: {response}"))?;
    if queried_status.trim().is_empty() {
        return Err(format!("{step} returned empty status"));
    }
    Ok(())
}

fn run_live_s04_mcp_tool_call(
    binary: &str,
    endpoint: &str,
    agent_name: &str,
    key_file: &str,
    request_id: &str,
    tool_name: &str,
    arguments_json: &str,
) -> Result<String, String> {
    let mut child = Command::new(binary)
        .arg("--endpoint")
        .arg(endpoint)
        .arg("--agent-name")
        .arg(agent_name)
        .arg("--key-file")
        .arg(key_file)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|error| format!("mcp live s04 {tool_name} failed to spawn: {error}"))?;

    if let Some(mut stdin) = child.stdin.take() {
        let initialize_request = build_framed_jsonrpc_request(
            r#"{"jsonrpc":"2.0","id":"probe-init","method":"initialize","params":{}}"#,
        );
        let tool_request_json = format!(
            "{{\"jsonrpc\":\"2.0\",\"id\":\"{request_id}\",\"method\":\"tools/call\",\"params\":{{\"name\":\"{tool_name}\",\"arguments\":{arguments_json}}}}}"
        );
        let tool_request = build_framed_jsonrpc_request(tool_request_json.as_str());
        let framed_requests = format!("{initialize_request}{tool_request}");
        stdin
            .write_all(framed_requests.as_bytes())
            .map_err(|error| {
                format!("mcp live s04 {tool_name} failed to write framed request stream: {error}")
            })?;
    }

    let output = child
        .wait_with_output()
        .map_err(|error| format!("mcp live s04 {tool_name} failed to read response: {error}"))?;
    if !output.status.success() {
        let exit_status = output
            .status
            .code()
            .map(|value| value.to_string())
            .unwrap_or_else(|| "signal".to_owned());
        return Err(format!(
            "mcp live s04 {tool_name} failed (exit_status={exit_status})"
        ));
    }

    let stdout = String::from_utf8_lossy(output.stdout.as_slice());
    let payloads = parse_framed_jsonrpc_payloads(stdout.as_ref())
        .map_err(|error| format!("mcp live s04 {tool_name} invalid framed output: {error}"))?;

    let initialize_response = payloads
        .iter()
        .find(|payload| payload.contains(r#""id":"probe-init""#))
        .ok_or_else(|| format!("mcp live s04 {tool_name} missing initialize response payload"))?;
    validate_probe_initialize_response(initialize_response)?;

    let response_id = format!(r#""id":"{request_id}""#);
    let tool_response = payloads
        .iter()
        .find(|payload| payload.contains(response_id.as_str()))
        .ok_or_else(|| format!("mcp live s04 {tool_name} missing tool response payload"))?;
    if !json_optional_bool_field(tool_response.as_str(), "ok").unwrap_or(false) {
        return Err(format!(
            "mcp live s04 {tool_name} returned non-success payload: {tool_response}"
        ));
    }

    Ok(tool_response.clone())
}

fn run_live_s02_mcp_tool_call(
    binary: &str,
    endpoint: &str,
    agent_name: &str,
    key_file: &str,
    request_id: &str,
    tool_name: &str,
    arguments_json: &str,
) -> Result<String, String> {
    run_live_s04_mcp_tool_call(
        binary,
        endpoint,
        agent_name,
        key_file,
        request_id,
        tool_name,
        arguments_json,
    )
    .map_err(|error| error.replace("mcp live s04", "mcp live s02"))
}

fn run_live_s03_mcp_tool_call(
    binary: &str,
    endpoint: &str,
    agent_name: &str,
    key_file: &str,
    request_id: &str,
    tool_name: &str,
    arguments_json: &str,
) -> Result<String, String> {
    run_live_s04_mcp_tool_call(
        binary,
        endpoint,
        agent_name,
        key_file,
        request_id,
        tool_name,
        arguments_json,
    )
    .map_err(|error| error.replace("mcp live s04", "mcp live s03"))
}

fn run_live_s05_mcp_tool_call(
    binary: &str,
    endpoint: &str,
    agent_name: &str,
    key_file: &str,
    request_id: &str,
    tool_name: &str,
    arguments_json: &str,
) -> Result<String, String> {
    run_live_s04_mcp_tool_call(
        binary,
        endpoint,
        agent_name,
        key_file,
        request_id,
        tool_name,
        arguments_json,
    )
    .map_err(|error| error.replace("mcp live s04", "mcp live s05"))
}

fn run_live_s06_mcp_tool_call(
    binary: &str,
    endpoint: &str,
    agent_name: &str,
    key_file: &str,
    request_id: &str,
    tool_name: &str,
    arguments_json: &str,
) -> Result<String, String> {
    run_live_s04_mcp_tool_call(
        binary,
        endpoint,
        agent_name,
        key_file,
        request_id,
        tool_name,
        arguments_json,
    )
    .map_err(|error| error.replace("mcp live s04", "mcp live s06"))
}

fn run_live_s07_mcp_tool_call(
    binary: &str,
    endpoint: &str,
    agent_name: &str,
    key_file: &str,
    request_id: &str,
    tool_name: &str,
    arguments_json: &str,
) -> Result<String, String> {
    run_live_s04_mcp_tool_call(
        binary,
        endpoint,
        agent_name,
        key_file,
        request_id,
        tool_name,
        arguments_json,
    )
    .map_err(|error| error.replace("mcp live s04", "mcp live s07"))
}

fn run_live_s08_mcp_tool_call(
    binary: &str,
    endpoint: &str,
    agent_name: &str,
    key_file: &str,
    request_id: &str,
    tool_name: &str,
    arguments_json: &str,
) -> Result<String, String> {
    run_live_s04_mcp_tool_call(
        binary,
        endpoint,
        agent_name,
        key_file,
        request_id,
        tool_name,
        arguments_json,
    )
    .map_err(|error| error.replace("mcp live s04", "mcp live s08"))
}

fn run_live_s09_mcp_tool_call(
    binary: &str,
    endpoint: &str,
    agent_name: &str,
    key_file: &str,
    request_id: &str,
    tool_name: &str,
    arguments_json: &str,
) -> Result<String, String> {
    run_live_s04_mcp_tool_call(
        binary,
        endpoint,
        agent_name,
        key_file,
        request_id,
        tool_name,
        arguments_json,
    )
    .map_err(|error| error.replace("mcp live s04", "mcp live s09"))
}

fn run_live_s10_mcp_tool_call(
    binary: &str,
    endpoint: &str,
    agent_name: &str,
    key_file: &str,
    request_id: &str,
    tool_name: &str,
    arguments_json: &str,
) -> Result<String, String> {
    run_live_s04_mcp_tool_call(
        binary,
        endpoint,
        agent_name,
        key_file,
        request_id,
        tool_name,
        arguments_json,
    )
    .map_err(|error| error.replace("mcp live s04", "mcp live s10"))
}

fn run_live_s11_mcp_tool_call(
    binary: &str,
    endpoint: &str,
    agent_name: &str,
    key_file: &str,
    request_id: &str,
    tool_name: &str,
    arguments_json: &str,
) -> Result<String, String> {
    run_live_s04_mcp_tool_call(
        binary,
        endpoint,
        agent_name,
        key_file,
        request_id,
        tool_name,
        arguments_json,
    )
    .map_err(|error| error.replace("mcp live s04", "mcp live s11"))
}

fn run_live_s12_mcp_tool_call(
    binary: &str,
    endpoint: &str,
    agent_name: &str,
    key_file: &str,
    request_id: &str,
    tool_name: &str,
    arguments_json: &str,
) -> Result<String, String> {
    run_live_s04_mcp_tool_call(
        binary,
        endpoint,
        agent_name,
        key_file,
        request_id,
        tool_name,
        arguments_json,
    )
    .map_err(|error| error.replace("mcp live s04", "mcp live s12"))
}

fn run_live_s13_mcp_tool_call(
    binary: &str,
    endpoint: &str,
    agent_name: &str,
    key_file: &str,
    request_id: &str,
    tool_name: &str,
    arguments_json: &str,
) -> Result<String, String> {
    run_live_s04_mcp_tool_call(
        binary,
        endpoint,
        agent_name,
        key_file,
        request_id,
        tool_name,
        arguments_json,
    )
    .map_err(|error| error.replace("mcp live s04", "mcp live s13"))
}

fn run_live_s14_mcp_tool_call(
    binary: &str,
    endpoint: &str,
    agent_name: &str,
    key_file: &str,
    request_id: &str,
    tool_name: &str,
    arguments_json: &str,
) -> Result<String, String> {
    run_live_s04_mcp_tool_call(
        binary,
        endpoint,
        agent_name,
        key_file,
        request_id,
        tool_name,
        arguments_json,
    )
    .map_err(|error| error.replace("mcp live s04", "mcp live s14"))
}

fn run_live_s15_mcp_tool_call(
    binary: &str,
    endpoint: &str,
    agent_name: &str,
    key_file: &str,
    request_id: &str,
    tool_name: &str,
    arguments_json: &str,
) -> Result<String, String> {
    run_live_s04_mcp_tool_call(
        binary,
        endpoint,
        agent_name,
        key_file,
        request_id,
        tool_name,
        arguments_json,
    )
    .map_err(|error| error.replace("mcp live s04", "mcp live s15"))
}

fn build_framed_jsonrpc_request(payload: &str) -> String {
    format!("Content-Length: {}\r\n\r\n{}", payload.len(), payload)
}

fn validate_probe_initialize_response(payload: &str) -> Result<(), String> {
    if !payload.contains(r#""jsonrpc":"2.0""#) {
        return Err(format!(
            "mcp live probe initialize response missing jsonrpc marker: {payload}"
        ));
    }
    if !payload.contains(r#""serverInfo""#) {
        return Err(format!(
            "mcp live probe initialize response missing serverInfo marker: {payload}"
        ));
    }
    Ok(())
}

fn validate_probe_health_response(payload: &str) -> Result<(), String> {
    if !json_optional_bool_field(payload, "ok").unwrap_or(false) {
        return Err(format!(
            "mcp live probe returned non-success health payload: {payload}"
        ));
    }
    Ok(())
}

fn parse_framed_jsonrpc_payloads(stream: &str) -> Result<Vec<String>, String> {
    let mut payloads = Vec::new();
    let mut cursor = 0usize;
    let bytes = stream.as_bytes();

    loop {
        while matches!(bytes.get(cursor), Some(b'\r' | b'\n')) {
            cursor = cursor
                .checked_add(1)
                .ok_or_else(|| "framed stream cursor overflow".to_owned())?;
        }
        if cursor == bytes.len() {
            break;
        }

        let remaining = stream
            .get(cursor..)
            .ok_or_else(|| "framed stream cursor out of bounds".to_owned())?;
        let Some(header_end) = remaining.find("\r\n\r\n") else {
            return Err("missing framed header terminator".to_owned());
        };
        let header = &remaining[..header_end];
        let length_value = header
            .lines()
            .find_map(|line| line.strip_prefix("Content-Length: "))
            .ok_or_else(|| "missing content-length header".to_owned())?;
        let content_length = length_value
            .trim()
            .parse::<usize>()
            .map_err(|_| "content-length must be numeric".to_owned())?;
        let payload_start = cursor
            .checked_add(header_end)
            .and_then(|value| value.checked_add(4))
            .ok_or_else(|| "framed payload start overflow".to_owned())?;
        let payload_end = payload_start
            .checked_add(content_length)
            .ok_or_else(|| "content-length overflows stream cursor".to_owned())?;
        let payload = stream
            .get(payload_start..payload_end)
            .ok_or_else(|| "content-length exceeds available framed payload bytes".to_owned())?;

        payloads.push(payload.to_owned());
        cursor = payload_end;
    }

    if payloads.is_empty() {
        return Err("no framed payloads parsed".to_owned());
    }
    Ok(payloads)
}

fn json_optional_string_field(payload: &str, key: &str) -> Option<String> {
    let marker = format!("\"{key}\":\"");
    let start = payload.find(marker.as_str())? + marker.len();
    let rest = payload.get(start..)?;
    let end = rest.find('"')?;
    Some(rest[..end].to_owned())
}

fn json_optional_u64_field(payload: &str, key: &str) -> Option<u64> {
    let marker = format!("\"{key}\":");
    let start = payload.find(marker.as_str())? + marker.len();
    let rest = payload.get(start..)?.trim_start();
    let digits = rest
        .chars()
        .take_while(|character| character.is_ascii_digit())
        .collect::<String>();
    if digits.is_empty() {
        return None;
    }
    digits.parse::<u64>().ok()
}

fn escape_json_scalar(input: &str) -> String {
    let mut escaped = String::with_capacity(input.len());
    for ch in input.chars() {
        match ch {
            '"' => escaped.push_str("\\\""),
            '\\' => escaped.push_str("\\\\"),
            '\n' => escaped.push_str("\\n"),
            '\r' => escaped.push_str("\\r"),
            '\t' => escaped.push_str("\\t"),
            _ => escaped.push(ch),
        }
    }
    escaped
}

#[cfg(test)]
#[path = "mcp_agent_tests.rs"]
mod mcp_agent_tests;
