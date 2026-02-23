use crate::drivers::{DriverExecutionResult, HarnessDriver};
use crate::ExecutionMode;
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
const S07_REPLAY_REASON_MARKER: &str = "service_api_auth_replay_nonce_detected";
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
            replay_protection_probe: live_probe,
        })
    }

    /// Creates MCP driver with explicit per-scenario probe implementations.
    pub fn with_probes<F, G, H, I, J, K, L>(
        mode: ExecutionMode,
        live_execution_enabled: bool,
        discovery_probe: F,
        direct_message_probe: G,
        group_channel_probe: H,
        task_lifecycle_probe: I,
        escrow_proof_and_replay_probes: (J, K, L),
    ) -> Result<Self, String>
    where
        F: Fn() -> Result<(), String> + Send + Sync + 'static,
        G: Fn() -> Result<(), String> + Send + Sync + 'static,
        H: Fn() -> Result<(), String> + Send + Sync + 'static,
        I: Fn() -> Result<(), String> + Send + Sync + 'static,
        J: Fn() -> Result<(), String> + Send + Sync + 'static,
        K: Fn() -> Result<(), String> + Send + Sync + 'static,
        L: Fn() -> Result<(), String> + Send + Sync + 'static,
    {
        if !matches!(mode, ExecutionMode::McpTau | ExecutionMode::McpAny) {
            return Err("McpAgentDriver requires mcp-tau or mcp-any mode".to_owned());
        }
        let (escrow_settlement_probe, proof_verification_probe, replay_protection_probe) =
            escrow_proof_and_replay_probes;
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
        })
    }
}

impl HarnessDriver for McpAgentDriver {
    fn mode(&self) -> ExecutionMode {
        self.mode
    }

    fn execute(&self, scenario_id: &'static str) -> DriverExecutionResult {
        let status = match self.live_probe_for_scenario(scenario_id) {
            Some(result) if result.is_ok() => "pass",
            Some(_) => "fail",
            None => "pass",
        };
        DriverExecutionResult {
            scenario_id,
            status,
        }
    }
}

impl McpAgentDriver {
    fn live_probe_for_scenario(&self, scenario_id: &'static str) -> Option<Result<(), String>> {
        if !self.live_execution_enabled {
            return None;
        }
        match scenario_id {
            "S-01" => Some((self.discovery_probe)()),
            "S-02" => Some((self.direct_message_probe)()),
            "S-03" => Some((self.group_channel_probe)()),
            "S-04" => Some((self.task_lifecycle_probe)()),
            "S-05" => Some((self.escrow_settlement_probe)()),
            "S-06" => Some((self.proof_verification_probe)()),
            "S-07" => Some((self.replay_protection_probe)()),
            _ => None,
        }
    }
}

fn live_execution_enabled_from_env() -> bool {
    env::var(MCP_AGENT_LIVE_ENV)
        .ok()
        .map(|value| parse_bool_flag(value.as_str()))
        .unwrap_or(false)
}

fn parse_bool_flag(value: &str) -> bool {
    matches!(
        value.trim().to_ascii_lowercase().as_str(),
        "1" | "true" | "yes" | "on"
    )
}

fn run_live_s01_mcp_probe() -> Result<(), String> {
    let binary =
        env::var(MCP_AGENT_BINARY_ENV).unwrap_or_else(|_| DEFAULT_MCP_AGENT_BINARY.to_owned());
    let endpoint = env::var("KAMN_ENDPOINT").unwrap_or_else(|_| DEFAULT_KAMN_ENDPOINT.to_owned());
    let agent_name =
        env::var("KAMN_AGENT_NAME").unwrap_or_else(|_| DEFAULT_MCP_AGENT_NAME.to_owned());
    let key_file =
        env::var("KAMN_AGENT_KEY_FILE").unwrap_or_else(|_| DEFAULT_MCP_AGENT_KEY_FILE.to_owned());

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
    let binary =
        env::var(MCP_AGENT_BINARY_ENV).unwrap_or_else(|_| DEFAULT_MCP_AGENT_BINARY.to_owned());
    let endpoint = env::var("KAMN_ENDPOINT").unwrap_or_else(|_| DEFAULT_KAMN_ENDPOINT.to_owned());
    let agent_name =
        env::var("KAMN_AGENT_NAME").unwrap_or_else(|_| DEFAULT_MCP_AGENT_NAME.to_owned());
    let key_file =
        env::var("KAMN_AGENT_KEY_FILE").unwrap_or_else(|_| DEFAULT_MCP_AGENT_KEY_FILE.to_owned());
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
    let binary =
        env::var(MCP_AGENT_BINARY_ENV).unwrap_or_else(|_| DEFAULT_MCP_AGENT_BINARY.to_owned());
    let endpoint = env::var("KAMN_ENDPOINT").unwrap_or_else(|_| DEFAULT_KAMN_ENDPOINT.to_owned());
    let agent_name =
        env::var("KAMN_AGENT_NAME").unwrap_or_else(|_| DEFAULT_MCP_AGENT_NAME.to_owned());
    let key_file =
        env::var("KAMN_AGENT_KEY_FILE").unwrap_or_else(|_| DEFAULT_MCP_AGENT_KEY_FILE.to_owned());
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
    let binary =
        env::var(MCP_AGENT_BINARY_ENV).unwrap_or_else(|_| DEFAULT_MCP_AGENT_BINARY.to_owned());
    let endpoint = env::var("KAMN_ENDPOINT").unwrap_or_else(|_| DEFAULT_KAMN_ENDPOINT.to_owned());
    let agent_name =
        env::var("KAMN_AGENT_NAME").unwrap_or_else(|_| DEFAULT_MCP_AGENT_NAME.to_owned());
    let key_file =
        env::var("KAMN_AGENT_KEY_FILE").unwrap_or_else(|_| DEFAULT_MCP_AGENT_KEY_FILE.to_owned());
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
    let binary =
        env::var(MCP_AGENT_BINARY_ENV).unwrap_or_else(|_| DEFAULT_MCP_AGENT_BINARY.to_owned());
    let endpoint = env::var("KAMN_ENDPOINT").unwrap_or_else(|_| DEFAULT_KAMN_ENDPOINT.to_owned());
    let agent_name =
        env::var("KAMN_AGENT_NAME").unwrap_or_else(|_| DEFAULT_MCP_AGENT_NAME.to_owned());
    let key_file =
        env::var("KAMN_AGENT_KEY_FILE").unwrap_or_else(|_| DEFAULT_MCP_AGENT_KEY_FILE.to_owned());
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
    )?;

    Ok(())
}

fn validate_live_s05_release_escrow_response(
    expected_escrow_id: &str,
    released_escrow_id: &str,
    release_state: &str,
) -> Result<(), String> {
    if released_escrow_id != expected_escrow_id {
        return Err(format!(
            "mcp live s05 release_escrow returned mismatched escrow_id: expected={expected_escrow_id}, got={released_escrow_id}"
        ));
    }
    if release_state.trim().is_empty() {
        return Err("mcp live s05 release_escrow returned empty state".to_owned());
    }
    Ok(())
}

fn run_live_s06_mcp_proof_verification_probe() -> Result<(), String> {
    let binary =
        env::var(MCP_AGENT_BINARY_ENV).unwrap_or_else(|_| DEFAULT_MCP_AGENT_BINARY.to_owned());
    let endpoint = env::var("KAMN_ENDPOINT").unwrap_or_else(|_| DEFAULT_KAMN_ENDPOINT.to_owned());
    let agent_name =
        env::var("KAMN_AGENT_NAME").unwrap_or_else(|_| DEFAULT_MCP_AGENT_NAME.to_owned());
    let key_file =
        env::var("KAMN_AGENT_KEY_FILE").unwrap_or_else(|_| DEFAULT_MCP_AGENT_KEY_FILE.to_owned());
    let message_id = env::var("KAMN_E2E_S06_PROOF_MESSAGE_ID")
        .unwrap_or_else(|_| DEFAULT_S06_MESSAGE_ID.to_owned());
    let tx_hash =
        env::var("KAMN_E2E_S06_PROOF_TX_HASH").unwrap_or_else(|_| DEFAULT_S06_TX_HASH.to_owned());
    let block_height = env::var("KAMN_E2E_S06_PROOF_BLOCK_HEIGHT")
        .ok()
        .map(|raw| {
            raw.trim()
                .parse::<u64>()
                .map_err(|_| format!("mcp live s06 invalid block height env value: {raw}"))
        })
        .transpose()?
        .unwrap_or(DEFAULT_S06_BLOCK_HEIGHT);
    let finality =
        env::var("KAMN_E2E_S06_PROOF_FINALITY").unwrap_or_else(|_| DEFAULT_S06_FINALITY.to_owned());

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
    let binary =
        env::var(MCP_AGENT_BINARY_ENV).unwrap_or_else(|_| DEFAULT_MCP_AGENT_BINARY.to_owned());
    let endpoint = env::var("KAMN_ENDPOINT").unwrap_or_else(|_| DEFAULT_KAMN_ENDPOINT.to_owned());
    let key_file =
        env::var("KAMN_AGENT_KEY_FILE").unwrap_or_else(|_| DEFAULT_MCP_AGENT_KEY_FILE.to_owned());
    let base_agent_name =
        env::var("KAMN_E2E_S07_AGENT_NAME").unwrap_or_else(|_| DEFAULT_S07_AGENT_NAME.to_owned());
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

fn validate_s07_replay_reason_marker(replay_error: &str, step: &str) -> Result<(), String> {
    if !replay_error.contains(S07_REPLAY_REASON_MARKER) {
        return Err(format!(
            "{step} missing replay reason marker: {replay_error}"
        ));
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
    if !tool_response.contains(r#""ok":true"#) {
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

fn live_s07_probe_agent_suffix() -> String {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_nanos().to_string())
        .unwrap_or_else(|_| "0".to_owned())
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
    if !payload.contains(r#""ok":true"#) {
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
mod tests {
    use super::{
        build_framed_jsonrpc_request, escape_json_scalar, json_optional_string_field,
        json_optional_u64_field, live_execution_enabled_from_env, parse_bool_flag,
        parse_framed_jsonrpc_payloads, run_live_s01_mcp_probe,
        run_live_s02_mcp_direct_message_probe, run_live_s03_mcp_group_channel_probe,
        run_live_s04_mcp_task_lifecycle_probe, run_live_s04_mcp_tool_call,
        run_live_s05_mcp_escrow_settlement_probe, run_live_s06_mcp_proof_verification_probe,
        run_live_s07_mcp_replay_protection_probe, validate_live_s05_release_escrow_response,
        validate_probe_health_response, validate_probe_initialize_response,
        validate_s07_replay_reason_marker, McpAgentDriver, MCP_AGENT_BINARY_ENV,
        MCP_AGENT_LIVE_ENV,
    };
    use super::{env, ExecutionMode};
    use std::ffi::OsString;
    use std::fs;
    use std::os::unix::fs::PermissionsExt;
    use std::sync::PoisonError;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn with_env_vars<F>(updates: &[(&str, Option<&str>)], test: F)
    where
        F: FnOnce(),
    {
        let _guard = crate::drivers::test_env_lock()
            .lock()
            .unwrap_or_else(PoisonError::into_inner);
        let previous = updates
            .iter()
            .map(|(key, _)| ((*key).to_owned(), env::var_os(key)))
            .collect::<Vec<(String, Option<OsString>)>>();

        for (key, value) in updates {
            match value {
                Some(value) => {
                    // SAFETY: tests serialize env mutation with a process-wide mutex.
                    unsafe { env::set_var(key, value) }
                }
                None => {
                    // SAFETY: tests serialize env mutation with a process-wide mutex.
                    unsafe { env::remove_var(key) }
                }
            }
        }

        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(test));
        for (key, value) in previous {
            match value {
                Some(value) => {
                    // SAFETY: tests serialize env mutation with a process-wide mutex.
                    unsafe { env::set_var(key, value) }
                }
                None => {
                    // SAFETY: tests serialize env mutation with a process-wide mutex.
                    unsafe { env::remove_var(key) }
                }
            }
        }
        if let Err(payload) = result {
            std::panic::resume_unwind(payload);
        }
    }

    fn unique_temp_script_path(stem: &str) -> std::path::PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time should be after unix epoch")
            .as_nanos();
        std::env::temp_dir().join(format!("{stem}-{}-{nonce}.py", std::process::id()))
    }

    fn write_executable_python_script(script_path: &std::path::Path, source: &str) {
        fs::write(script_path, source).expect("script fixture should be written");
        let mut permissions = fs::metadata(script_path)
            .expect("script metadata should be readable")
            .permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(script_path, permissions).expect("script fixture should be executable");
        // Allow the filesystem/loader state to settle before immediate exec from test probes.
        std::thread::sleep(std::time::Duration::from_millis(5));
    }

    fn write_mcp_tool_response_script(
        script_path: &std::path::Path,
        request_id: &str,
        result_payload: &str,
    ) {
        let init_payload =
            r#"{"jsonrpc":"2.0","id":"probe-init","result":{"serverInfo":{"name":"kamn"}}}"#;
        let tool_payload =
            format!(r#"{{"jsonrpc":"2.0","id":"{request_id}","result":{result_payload}}}"#);
        let script_source = format!(
            r#"#!/usr/bin/env python3
import sys
init_payload = {init_payload:?}
tool_payload = {tool_payload:?}
sys.stdout.write(
    f"Content-Length: {{len(init_payload)}}\r\n\r\n{{init_payload}}"
    f"Content-Length: {{len(tool_payload)}}\r\n\r\n{{tool_payload}}"
)
"#,
        );
        write_executable_python_script(script_path, script_source.as_str());
    }

    fn write_mcp_s03_probe_script(
        script_path: &std::path::Path,
        query_message_id: &str,
        list_channel_id: &str,
        include_messages: bool,
    ) {
        let include_messages_literal = if include_messages { "True" } else { "False" };
        let script_source = format!(
            r#"#!/usr/bin/env python3
import json
import re
import sys

query_message_id = {query_message_id:?}
list_channel_id = {list_channel_id:?}
include_messages = {include_messages_literal}

stream = sys.stdin.read()
request_ids = re.findall(r'"id":"([^"]+)"', stream)
request_id = request_ids[-1] if request_ids else "probe-request"
tool_names = re.findall(r'"name":"([^"]+)"', stream)
tool_name = tool_names[-1] if tool_names else ""

result = {{"ok": True}}
if tool_name == "create_channel":
    result.update({{"channel_id":"channel-1","status":"created"}})
elif tool_name == "send_message":
    result.update({{"message_id":"message-1","status":"sent","channel_id":"channel-1"}})
elif tool_name == "query_message":
    result.update({{"message_id": query_message_id, "status":"sent"}})
elif tool_name == "list_messages":
    result.update({{"channel_id": list_channel_id}})
    if include_messages:
        result.update({{"messages":["message-1"]}})
else:
    result.update({{"error":"unsupported_tool"}})

init_payload = {{"jsonrpc":"2.0","id":"probe-init","result":{{"serverInfo":{{"name":"kamn"}}}}}}
tool_payload = {{"jsonrpc":"2.0","id":request_id,"result":result}}

def frame(payload):
    body = json.dumps(payload, separators=(",", ":"))
    return f"Content-Length: {{len(body)}}\r\n\r\n{{body}}"

sys.stdout.write(frame(init_payload) + frame(tool_payload))
"#,
            query_message_id = query_message_id,
            list_channel_id = list_channel_id,
            include_messages_literal = include_messages_literal,
        );
        write_executable_python_script(script_path, script_source.as_str());
    }

    #[test]
    fn unit_parse_bool_flag_accepts_true_like_values() {
        for value in ["1", "true", "TRUE", "yes", "on"] {
            assert!(parse_bool_flag(value), "expected truthy for {value}");
        }
    }

    #[test]
    fn unit_parse_bool_flag_rejects_false_like_values() {
        for value in ["0", "false", "off", "no", ""] {
            assert!(!parse_bool_flag(value), "expected falsey for {value}");
        }
    }

    #[test]
    fn unit_live_execution_enabled_from_env_honors_true_and_false_markers() {
        with_env_vars(&[(MCP_AGENT_LIVE_ENV, Some("1"))], || {
            assert!(live_execution_enabled_from_env());
        });
        with_env_vars(&[(MCP_AGENT_LIVE_ENV, Some("0"))], || {
            assert!(!live_execution_enabled_from_env());
        });
    }

    #[test]
    fn unit_run_live_s01_mcp_probe_rejects_missing_binary() {
        with_env_vars(
            &[
                (
                    MCP_AGENT_BINARY_ENV,
                    Some("/definitely/missing/kamn-mcp-server"),
                ),
                ("KAMN_ENDPOINT", Some("http://localhost:8080")),
                ("KAMN_AGENT_NAME", Some("probe")),
                ("KAMN_AGENT_KEY_FILE", Some("/tmp/probe.key")),
            ],
            || {
                let error = run_live_s01_mcp_probe().expect_err("missing binary should fail");
                assert!(
                    error.contains("failed to spawn"),
                    "error should reflect spawn failure: {error}",
                );
            },
        );
    }

    #[test]
    fn unit_run_live_s02_mcp_direct_message_probe_rejects_missing_binary() {
        with_env_vars(
            &[
                (
                    MCP_AGENT_BINARY_ENV,
                    Some("/definitely/missing/kamn-mcp-server"),
                ),
                ("KAMN_ENDPOINT", Some("http://localhost:8080")),
                ("KAMN_AGENT_NAME", Some("probe")),
                ("KAMN_AGENT_KEY_FILE", Some("/tmp/probe.key")),
            ],
            || {
                let error = run_live_s02_mcp_direct_message_probe()
                    .expect_err("missing binary should fail");
                assert!(
                    error.contains("failed to spawn"),
                    "error should reflect spawn failure: {error}",
                );
            },
        );
    }

    #[test]
    fn unit_run_live_s03_mcp_group_channel_probe_rejects_missing_binary() {
        with_env_vars(
            &[
                (
                    MCP_AGENT_BINARY_ENV,
                    Some("/definitely/missing/kamn-mcp-server"),
                ),
                ("KAMN_ENDPOINT", Some("http://localhost:8080")),
                ("KAMN_AGENT_NAME", Some("probe")),
                ("KAMN_AGENT_KEY_FILE", Some("/tmp/probe.key")),
            ],
            || {
                let error =
                    run_live_s03_mcp_group_channel_probe().expect_err("missing binary should fail");
                assert!(
                    error.contains("failed to spawn"),
                    "error should reflect spawn failure: {error}",
                );
            },
        );
    }

    #[test]
    fn unit_run_live_s03_mcp_group_channel_probe_rejects_query_message_id_mismatch() {
        let script_path = unique_temp_script_path("kamn-e2e-mcp-s03-query-mismatch");
        write_mcp_s03_probe_script(&script_path, "message-2", "channel-1", true);

        with_env_vars(
            &[
                (
                    MCP_AGENT_BINARY_ENV,
                    Some(
                        script_path
                            .to_str()
                            .expect("script path should be valid utf-8"),
                    ),
                ),
                ("KAMN_ENDPOINT", Some("http://localhost:8080")),
                ("KAMN_AGENT_NAME", Some("probe")),
                ("KAMN_AGENT_KEY_FILE", Some("/tmp/probe.key")),
            ],
            || {
                let error = run_live_s03_mcp_group_channel_probe()
                    .expect_err("mismatched query message_id should fail");
                assert!(
                    error.contains("mismatched message_id"),
                    "error should mention message_id mismatch: {error}",
                );
            },
        );

        fs::remove_file(&script_path).expect("script fixture should be removable");
    }

    #[test]
    fn unit_run_live_s03_mcp_group_channel_probe_rejects_missing_messages_field() {
        let script_path = unique_temp_script_path("kamn-e2e-mcp-s03-missing-messages");
        write_mcp_s03_probe_script(&script_path, "message-1", "channel-1", false);

        with_env_vars(
            &[
                (
                    MCP_AGENT_BINARY_ENV,
                    Some(
                        script_path
                            .to_str()
                            .expect("script path should be valid utf-8"),
                    ),
                ),
                ("KAMN_ENDPOINT", Some("http://localhost:8080")),
                ("KAMN_AGENT_NAME", Some("probe")),
                ("KAMN_AGENT_KEY_FILE", Some("/tmp/probe.key")),
            ],
            || {
                let error = run_live_s03_mcp_group_channel_probe()
                    .expect_err("missing list_messages messages field should fail");
                assert!(
                    error.contains("missing messages field"),
                    "error should mention messages field contract: {error}",
                );
            },
        );

        fs::remove_file(&script_path).expect("script fixture should be removable");
    }

    #[test]
    fn unit_run_live_s04_mcp_task_lifecycle_probe_rejects_missing_binary() {
        with_env_vars(
            &[
                (
                    MCP_AGENT_BINARY_ENV,
                    Some("/definitely/missing/kamn-mcp-server"),
                ),
                ("KAMN_ENDPOINT", Some("http://localhost:8080")),
                ("KAMN_AGENT_NAME", Some("probe")),
                ("KAMN_AGENT_KEY_FILE", Some("/tmp/probe.key")),
            ],
            || {
                let error = run_live_s04_mcp_task_lifecycle_probe()
                    .expect_err("missing binary should fail");
                assert!(
                    error.contains("failed to spawn"),
                    "error should reflect spawn failure: {error}",
                );
            },
        );
    }

    #[test]
    fn unit_run_live_s05_mcp_escrow_settlement_probe_rejects_missing_binary() {
        with_env_vars(
            &[
                (
                    MCP_AGENT_BINARY_ENV,
                    Some("/definitely/missing/kamn-mcp-server"),
                ),
                ("KAMN_ENDPOINT", Some("http://localhost:8080")),
                ("KAMN_AGENT_NAME", Some("probe")),
                ("KAMN_AGENT_KEY_FILE", Some("/tmp/probe.key")),
            ],
            || {
                let error = run_live_s05_mcp_escrow_settlement_probe()
                    .expect_err("missing binary should fail");
                assert!(
                    error.contains("failed to spawn"),
                    "error should reflect spawn failure: {error}",
                );
            },
        );
    }

    #[test]
    fn unit_validate_live_s05_release_escrow_response_rejects_mismatched_escrow_id() {
        let error = validate_live_s05_release_escrow_response("escrow-a", "escrow-b", "released")
            .expect_err("mismatched escrow ids should fail");
        assert!(
            error.contains("mismatched escrow_id"),
            "error should describe escrow-id mismatch: {error}",
        );
    }

    #[test]
    fn unit_run_live_s06_mcp_proof_verification_probe_rejects_missing_binary() {
        with_env_vars(
            &[
                (
                    MCP_AGENT_BINARY_ENV,
                    Some("/definitely/missing/kamn-mcp-server"),
                ),
                ("KAMN_ENDPOINT", Some("http://localhost:8080")),
                ("KAMN_AGENT_NAME", Some("probe")),
                ("KAMN_AGENT_KEY_FILE", Some("/tmp/probe.key")),
            ],
            || {
                let error = run_live_s06_mcp_proof_verification_probe()
                    .expect_err("missing binary should fail");
                assert!(
                    error.contains("failed to spawn"),
                    "error should reflect spawn failure: {error}",
                );
            },
        );
    }

    #[test]
    fn unit_run_live_s07_mcp_replay_protection_probe_rejects_missing_binary() {
        with_env_vars(
            &[
                (
                    MCP_AGENT_BINARY_ENV,
                    Some("/definitely/missing/kamn-mcp-server"),
                ),
                ("KAMN_ENDPOINT", Some("http://localhost:8080")),
                ("KAMN_AGENT_KEY_FILE", Some("/tmp/probe.key")),
            ],
            || {
                let error = run_live_s07_mcp_replay_protection_probe()
                    .expect_err("missing binary should fail");
                assert!(
                    error.contains("failed to spawn"),
                    "error should reflect spawn failure: {error}",
                );
            },
        );
    }

    #[test]
    fn unit_validate_s07_replay_reason_marker_accepts_expected_marker() {
        validate_s07_replay_reason_marker(
            "operation failed: service_api_auth_replay_nonce_detected",
            "test helper",
        )
        .expect("expected marker should be accepted");
    }

    #[test]
    fn unit_validate_s07_replay_reason_marker_rejects_missing_marker() {
        let error = validate_s07_replay_reason_marker("operation failed", "test helper")
            .expect_err("missing marker should fail");
        assert!(
            error.contains("missing replay reason marker"),
            "error should mention replay marker contract: {error}",
        );
    }

    #[test]
    fn unit_live_s07_probe_agent_suffix_is_non_empty_numeric() {
        let suffix = super::live_s07_probe_agent_suffix();
        assert!(!suffix.is_empty(), "suffix should be non-empty");
        assert!(
            suffix.chars().all(|character| character.is_ascii_digit()),
            "suffix should be numeric: {suffix}",
        );
    }

    #[test]
    fn unit_run_live_s06_mcp_proof_verification_probe_accepts_success_payload() {
        let script_path = unique_temp_script_path("kamn-e2e-mcp-s06-success");
        write_mcp_tool_response_script(
            &script_path,
            "probe-verify-proof",
            r#"{"ok":true,"finality":"FINAL","verified":true,"block_height":42}"#,
        );

        with_env_vars(
            &[
                (
                    MCP_AGENT_BINARY_ENV,
                    Some(
                        script_path
                            .to_str()
                            .expect("script path should be valid utf-8"),
                    ),
                ),
                ("KAMN_ENDPOINT", Some("http://localhost:8080")),
                ("KAMN_AGENT_NAME", Some("probe")),
                ("KAMN_AGENT_KEY_FILE", Some("/tmp/probe.key")),
            ],
            || {
                run_live_s06_mcp_proof_verification_probe()
                    .expect("success payload should pass verification probe");
            },
        );

        fs::remove_file(&script_path).expect("script fixture should be removable");
    }

    #[test]
    fn unit_run_live_s04_mcp_tool_call_rejects_missing_binary() {
        let error = run_live_s04_mcp_tool_call(
            "/definitely/missing/kamn-mcp-server",
            "http://localhost:8080",
            "probe",
            "/tmp/probe.key",
            "probe-request",
            "health",
            "{}",
        )
        .expect_err("missing binary should fail");
        assert!(
            error.contains("failed to spawn"),
            "error should reflect spawn failure: {error}",
        );
    }

    #[test]
    fn unit_run_live_s04_mcp_tool_call_rejects_non_success_exit_status() {
        let error = run_live_s04_mcp_tool_call(
            "/bin/false",
            "http://localhost:8080",
            "probe",
            "/tmp/probe.key",
            "probe-request",
            "health",
            "{}",
        )
        .expect_err("non-success status should fail");
        assert!(
            error.contains("exit_status=1"),
            "error should include non-success status marker: {error}",
        );
    }

    #[test]
    fn unit_run_live_s04_mcp_tool_call_accepts_ok_true_payload() {
        let script_path = unique_temp_script_path("kamn-e2e-mcp-tool-call");
        write_mcp_tool_response_script(&script_path, "probe-request", r#"{"ok":true}"#);

        let probe_result = run_live_s04_mcp_tool_call(
            script_path
                .to_str()
                .expect("script path should be valid utf-8"),
            "http://localhost:8080",
            "probe",
            "/tmp/probe.key",
            "probe-request",
            "health",
            "{}",
        );

        fs::remove_file(&script_path).expect("script fixture should be removable");

        let payload = probe_result.expect("ok=true payload should pass");
        assert!(
            payload.contains(r#""ok":true"#),
            "payload should preserve ok=true marker: {payload}",
        );
    }

    #[test]
    fn unit_run_live_s04_mcp_tool_call_success_status_still_requires_framed_payloads() {
        let error = run_live_s04_mcp_tool_call(
            "/bin/true",
            "http://localhost:8080",
            "probe",
            "/tmp/probe.key",
            "probe-request",
            "health",
            "{}",
        )
        .expect_err("success exit without framed output should fail");
        assert!(
            error.contains("invalid framed output"),
            "error should reflect framed payload parse failure: {error}",
        );
    }

    #[test]
    fn unit_json_optional_string_field_extracts_known_value_and_missing_is_none() {
        let payload =
            r#"{"jsonrpc":"2.0","id":"probe","result":{"task_id":"task-1","state":"created"}}"#;
        assert_eq!(
            json_optional_string_field(payload, "task_id"),
            Some("task-1".to_owned())
        );
        assert_eq!(
            json_optional_string_field(payload, "state"),
            Some("created".to_owned())
        );
        assert_eq!(json_optional_string_field(payload, "missing"), None);
    }

    #[test]
    fn unit_json_optional_u64_field_extracts_known_value_and_missing_is_none() {
        let payload = r#"{"jsonrpc":"2.0","id":"probe","result":{"block_height":42,"ok":true}}"#;
        assert_eq!(json_optional_u64_field(payload, "block_height"), Some(42));
        assert_eq!(json_optional_u64_field(payload, "missing"), None);
    }

    #[test]
    fn unit_escape_json_scalar_escapes_quotes_backslashes_and_controls() {
        let escaped = escape_json_scalar("\"\\\n\r\tx");
        assert_eq!(escaped, "\\\"\\\\\\n\\r\\tx");
    }

    #[test]
    fn unit_mcp_agent_driver_debug_includes_mode_and_live_toggle() {
        let driver = McpAgentDriver::with_probe(ExecutionMode::McpTau, false, || Ok(()))
            .expect("driver should build");
        let debug = format!("{driver:?}");
        assert!(debug.contains("McpAgentDriver"));
        assert!(debug.contains("mode"));
        assert!(debug.contains("live_execution_enabled"));
    }

    #[test]
    fn spec_c01_build_framed_jsonrpc_request_includes_content_length_and_body() {
        let request = build_framed_jsonrpc_request(r#"{"jsonrpc":"2.0","id":"req-1"}"#);
        assert!(
            request.starts_with("Content-Length: "),
            "framed request should include content-length prefix: {request}",
        );
        assert!(
            request.contains("\r\n\r\n{\"jsonrpc\":\"2.0\",\"id\":\"req-1\"}"),
            "framed request should include header/body separator + payload: {request}",
        );
    }

    #[test]
    fn spec_c02_parse_framed_jsonrpc_payloads_supports_multiple_frames() {
        let first = build_framed_jsonrpc_request(r#"{"jsonrpc":"2.0","id":"init"}"#);
        let second = build_framed_jsonrpc_request(r#"{"jsonrpc":"2.0","id":"health"}"#);
        let payloads = parse_framed_jsonrpc_payloads(format!("{first}{second}").as_str())
            .expect("framed payloads should parse");
        assert_eq!(
            payloads,
            vec![
                r#"{"jsonrpc":"2.0","id":"init"}"#.to_owned(),
                r#"{"jsonrpc":"2.0","id":"health"}"#.to_owned()
            ]
        );
    }

    #[test]
    fn spec_c03_parse_framed_jsonrpc_payloads_rejects_malformed_stream() {
        let malformed = "Content-Length: 9\r\n\r\n{\"id\":1}";
        let error = parse_framed_jsonrpc_payloads(malformed)
            .expect_err("mismatched content-length should fail");
        assert!(
            error.contains("content-length"),
            "error should mention content-length mismatch: {error}",
        );
    }

    #[test]
    fn spec_c04_parse_framed_jsonrpc_payloads_accepts_leading_newlines() {
        let framed = format!(
            "\n{}",
            build_framed_jsonrpc_request(r#"{"jsonrpc":"2.0","id":"init"}"#)
        );
        let payloads = parse_framed_jsonrpc_payloads(framed.as_str())
            .expect("leading newline should be skipped");
        assert_eq!(
            payloads,
            vec![r#"{"jsonrpc":"2.0","id":"init"}"#.to_owned()]
        );
    }

    #[test]
    fn spec_c05_parse_framed_jsonrpc_payloads_rejects_newline_only_stream() {
        let error =
            parse_framed_jsonrpc_payloads("\n").expect_err("newline-only stream should fail");
        assert!(
            error.contains("no framed payloads parsed"),
            "error should mention missing payloads: {error}",
        );
    }

    #[test]
    fn spec_c06_validate_probe_initialize_response_rejects_missing_jsonrpc_marker() {
        let payload = r#"{"id":"probe-init","result":{"serverInfo":{"name":"kamn"}}}"#;
        let error = validate_probe_initialize_response(payload)
            .expect_err("missing jsonrpc marker should fail");
        assert!(
            error.contains("missing jsonrpc marker"),
            "error should mention jsonrpc marker: {error}",
        );
    }

    #[test]
    fn spec_c07_validate_probe_initialize_response_rejects_missing_server_info_marker() {
        let payload = r#"{"jsonrpc":"2.0","id":"probe-init","result":{}}"#;
        let error = validate_probe_initialize_response(payload)
            .expect_err("missing serverInfo marker should fail");
        assert!(
            error.contains("missing serverInfo marker"),
            "error should mention serverInfo marker: {error}",
        );
    }

    #[test]
    fn spec_c08_validate_probe_health_response_rejects_non_success_payload() {
        let payload = r#"{"jsonrpc":"2.0","id":"probe-health","result":{"ok":false}}"#;
        let error =
            validate_probe_health_response(payload).expect_err("non-success payload should fail");
        assert!(
            error.contains("non-success health payload"),
            "error should mention non-success health payload: {error}",
        );
    }

    #[test]
    fn spec_c09_live_s04_driver_path_fails_closed_when_task_probe_errors() {
        let driver = McpAgentDriver::with_probe(ExecutionMode::McpTau, true, || {
            Err("mcp-agent live s04 task probe failed".to_owned())
        })
        .expect("driver should build");
        let result = crate::drivers::HarnessDriver::execute(&driver, "S-04");
        assert_eq!(
            result.status, "fail",
            "live-enabled S-04 should fail closed on probe error",
        );
    }

    #[test]
    fn spec_c10_live_s06_driver_path_fails_closed_when_proof_probe_errors() {
        let driver = McpAgentDriver::with_probe(ExecutionMode::McpTau, true, || {
            Err("mcp-agent live s06 proof probe failed".to_owned())
        })
        .expect("driver should build");
        let result = crate::drivers::HarnessDriver::execute(&driver, "S-06");
        assert_eq!(
            result.status, "fail",
            "live-enabled S-06 should fail closed on probe error",
        );
    }

    #[test]
    fn spec_c11_live_s02_driver_path_fails_closed_when_message_probe_errors() {
        let driver = McpAgentDriver::with_probe(ExecutionMode::McpTau, true, || {
            Err("mcp-agent live s02 message probe failed".to_owned())
        })
        .expect("driver should build");
        let result = crate::drivers::HarnessDriver::execute(&driver, "S-02");
        assert_eq!(
            result.status, "fail",
            "live-enabled S-02 should fail closed on probe error",
        );
    }

    #[test]
    fn spec_c12_live_s03_driver_path_fails_closed_when_channel_probe_errors() {
        let driver = McpAgentDriver::with_probe(ExecutionMode::McpTau, true, || {
            Err("mcp-agent live s03 channel probe failed".to_owned())
        })
        .expect("driver should build");
        let result = crate::drivers::HarnessDriver::execute(&driver, "S-03");
        assert_eq!(
            result.status, "fail",
            "live-enabled S-03 should fail closed on probe error",
        );
    }

    #[test]
    fn spec_c13_live_s05_driver_path_fails_closed_when_escrow_probe_errors() {
        let driver = McpAgentDriver::with_probe(ExecutionMode::McpTau, true, || {
            Err("mcp-agent live s05 escrow probe failed".to_owned())
        })
        .expect("driver should build");
        let result = crate::drivers::HarnessDriver::execute(&driver, "S-05");
        assert_eq!(
            result.status, "fail",
            "live-enabled S-05 should fail closed on probe error",
        );
    }

    #[test]
    fn spec_c14_live_s07_driver_path_fails_closed_when_replay_probe_errors() {
        let driver = McpAgentDriver::with_probe(ExecutionMode::McpTau, true, || {
            Err("mcp-agent live s07 replay probe failed".to_owned())
        })
        .expect("driver should build");
        let result = crate::drivers::HarnessDriver::execute(&driver, "S-07");
        assert_eq!(
            result.status, "fail",
            "live-enabled S-07 should fail closed on probe error",
        );
    }

    #[test]
    fn spec_c09_validate_probe_initialize_response_accepts_required_markers() {
        let payload = r#"{"jsonrpc":"2.0","id":"probe-init","result":{"serverInfo":{"name":"kamn-mcp-server"}}}"#;
        validate_probe_initialize_response(payload)
            .expect("required initialize markers should pass");
    }

    #[test]
    fn spec_c10_validate_probe_health_response_accepts_success_payload() {
        let payload = r#"{"jsonrpc":"2.0","id":"probe-health","result":{"ok":true}}"#;
        validate_probe_health_response(payload).expect("ok=true health payload should pass");
    }
}
