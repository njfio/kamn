use crate::drivers::{DriverExecutionResult, HarnessDriver};
use crate::ExecutionMode;
use std::env;
use std::process::{Command, Stdio};
use std::sync::Arc;

const CLI_SCRIPTED_LIVE_ENV: &str = "KAMN_E2E_CLI_SCRIPTED_LIVE";
const CLI_BINARY_ENV: &str = "KAMN_E2E_CLI_BINARY";
const DEFAULT_CLI_BINARY: &str = "kamn-cli";
const DEFAULT_S02_AGENT_NAME: &str = "kamn-e2e-cli-s02";
const DEFAULT_S02_MESSAGE_PAYLOAD: &str = r#"{"message":"cli-scripted-live-s02"}"#;
const DEFAULT_S02_REPLY_PAYLOAD: &str = r#"{"message":"cli-scripted-live-s02-reply"}"#;
const DEFAULT_S03_AGENT_NAME: &str = "kamn-e2e-cli-s03";
const DEFAULT_S03_CHANNEL_PAYLOAD: &str =
    r#"{"name":"cli-scripted-live-s03","members":["alice","bob","carol"]}"#;
const DEFAULT_S03_MESSAGE_PAYLOAD: &str = r#"{"message":"cli-scripted-live-s03-channel-message"}"#;
const DEFAULT_S04_AGENT_NAME: &str = "kamn-e2e-cli-s04";
const DEFAULT_S04_CREATE_TASK_PAYLOAD: &str =
    r#"{"title":"cli-scripted-live-s04","description":"live task lifecycle probe"}"#;
const DEFAULT_S04_ESCROW_AMOUNT: u64 = 1;
const DEFAULT_S05_AGENT_NAME: &str = "kamn-e2e-cli-s05";
const DEFAULT_S05_FUND_ESCROW_PAYLOAD: &str = r#"{"task_id":"cli-scripted-live-s05","amount":1}"#;
const DEFAULT_S07_AGENT_NAME: &str = "kamn-e2e-cli-s07";
const DEFAULT_S07_MESSAGE_PAYLOAD: &str = r#"{"message":"cli-scripted-live-s07-replay"}"#;
const DEFAULT_S08_AGENT_NAME: &str = "kamn-e2e-cli-s08";
const DEFAULT_S08_PRE_MESSAGE_PAYLOAD: &str = r#"{"message":"cli-scripted-live-s08-pre"}"#;
const DEFAULT_S08_POST_MESSAGE_PAYLOAD: &str = r#"{"message":"cli-scripted-live-s08-post"}"#;
const DEFAULT_S09_AGENT_NAME: &str = "kamn-e2e-cli-s09";
const DEFAULT_S09_PRE_MESSAGE_PAYLOAD: &str = r#"{"message":"cli-scripted-live-s09-pre"}"#;
const DEFAULT_S09_POST_MESSAGE_PAYLOAD: &str = r#"{"message":"cli-scripted-live-s09-post"}"#;
const DEFAULT_S10_AGENT_NAME: &str = "kamn-e2e-cli-s10";
const DEFAULT_S10_MESSAGE_PAYLOAD: &str = r#"{"message":"cli-scripted-live-s10-topology"}"#;
const DEFAULT_S11_PRIMARY_AGENT_NAME: &str = "kamn-e2e-cli-s11-primary";
const DEFAULT_S11_MESSAGE_PAYLOAD: &str = r#"{"message":"cli-scripted-live-s11-primary"}"#;
const DEFAULT_S11_ROTATED_MESSAGE_PAYLOAD: &str = r#"{"message":"cli-scripted-live-s11-rotated"}"#;
const DEFAULT_S11_STALE_MESSAGE_PAYLOAD: &str = r#"{"message":"cli-scripted-live-s11-stale"}"#;
const DEFAULT_S12_AGENT_NAME: &str = "kamn-e2e-cli-s12";
const DEFAULT_S12_REGISTER_CONTENT_PAYLOAD: &str =
    r#"{"content":"cli-scripted-live-s12","retention_class":"standard"}"#;
const DEFAULT_S13_AGENT_NAME: &str = "kamn-e2e-cli-s13";
const DEFAULT_S13_SUBMIT_BRIDGE_PAYLOAD: &str =
    r#"{"source_message_id":"cli-scripted-live-s13","target_network":"testnet"}"#;
const DEFAULT_S14_AGENT_NAME: &str = "kamn-e2e-cli-s14";
const DEFAULT_S14_BATCH_MESSAGE_PAYLOAD_A: &str = r#"{"message":"cli-scripted-live-s14-batch-a"}"#;
const DEFAULT_S14_BATCH_MESSAGE_PAYLOAD_B: &str = r#"{"message":"cli-scripted-live-s14-batch-b"}"#;
const DEFAULT_S14_BLOCK_HEIGHT: u64 = 1;
const DEFAULT_S14_FINALITY: &str = "final";
const DEFAULT_S15_AGENT_NAME: &str = "kamn-e2e-cli-s15";
const DEFAULT_S15_MESSAGE_PAYLOAD: &str = r#"{"message":"cli-scripted-live-s15-performance"}"#;
const DEFAULT_S15_ITERATIONS: u64 = 3;
const DEFAULT_S15_MAX_TOTAL_MILLIS: u128 = 5_000;
const DEFAULT_S15_MAX_P50_MILLIS: u128 = 2_500;
const DEFAULT_S15_MAX_P99_MILLIS: u128 = 5_000;
const S07_REPLAY_REASON_MARKER: &str = "service_api_auth_replay_nonce_detected";
const DEFAULT_S06_MESSAGE_ID: &str = "s06-live-proof";
const DEFAULT_S06_TX_HASH: &str = "sha256:s06-live-proof";
const DEFAULT_S06_BLOCK_HEIGHT: u64 = 1;
const DEFAULT_S06_FINALITY: &str = "final";

type LiveCliRunner = dyn Fn() -> Result<(), String> + Send + Sync + 'static;

fn env_var_or_default(key: &str, default: &str) -> String {
    match env::var(key) {
        Ok(value) => value,
        Err(_) => default.to_owned(),
    }
}

fn env_var_or_else<F>(key: &str, fallback: F) -> String
where
    F: FnOnce() -> String,
{
    match env::var(key) {
        Ok(value) => value,
        Err(_) => fallback(),
    }
}


/// CLI-scripted driver with optional live execution for S-01 through S-15.
#[derive(Clone)]
pub struct CliScriptedDriver {
    live_execution_enabled: bool,
    discovery_runner: Arc<LiveCliRunner>,
    direct_message_runner: Arc<LiveCliRunner>,
    group_channel_runner: Arc<LiveCliRunner>,
    task_lifecycle_runner: Arc<LiveCliRunner>,
    escrow_settlement_runner: Arc<LiveCliRunner>,
    proof_verification_runner: Arc<LiveCliRunner>,
    replay_protection_runner: Arc<LiveCliRunner>,
    crash_recovery_runner: Arc<LiveCliRunner>,
    transport_failover_runner: Arc<LiveCliRunner>,
    topology_coherence_runner: Arc<LiveCliRunner>,
    signer_rotation_runner: Arc<LiveCliRunner>,
    retention_deletion_runner: Arc<LiveCliRunner>,
    bridge_forwarding_runner: Arc<LiveCliRunner>,
    batch_merkle_runner: Arc<LiveCliRunner>,
    performance_smoke_runner: Arc<LiveCliRunner>,
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
        let live_runner: Arc<LiveCliRunner> = Arc::new(live_runner);
        Self {
            live_execution_enabled,
            discovery_runner: live_runner.clone(),
            direct_message_runner: live_runner.clone(),
            task_lifecycle_runner: live_runner.clone(),
            group_channel_runner: live_runner.clone(),
            escrow_settlement_runner: live_runner.clone(),
            proof_verification_runner: live_runner.clone(),
            replay_protection_runner: live_runner.clone(),
            crash_recovery_runner: live_runner.clone(),
            transport_failover_runner: live_runner.clone(),
            topology_coherence_runner: live_runner.clone(),
            signer_rotation_runner: live_runner.clone(),
            retention_deletion_runner: live_runner.clone(),
            bridge_forwarding_runner: live_runner.clone(),
            batch_merkle_runner: live_runner.clone(),
            performance_smoke_runner: live_runner,
        }
    }

    /// Creates CLI-scripted driver with explicit per-scenario live runners.
    pub fn with_runners<F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T>(
        live_execution_enabled: bool,
        discovery_runner: F,
        direct_message_runner: G,
        group_channel_runner: H,
        task_lifecycle_runner: I,
        escrow_settlement_runner: J,
        proof_replay_crash_failover_topology_signer_retention_bridge_merkle_and_performance_runners: (
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
            proof_verification_runner,
            replay_protection_runner,
            crash_recovery_runner,
            transport_failover_runner,
            topology_coherence_runner,
            signer_rotation_runner,
            retention_deletion_runner,
            bridge_forwarding_runner,
            batch_merkle_runner,
            performance_smoke_runner,
        ) = proof_replay_crash_failover_topology_signer_retention_bridge_merkle_and_performance_runners;
        Self {
            live_execution_enabled,
            discovery_runner: Arc::new(discovery_runner),
            direct_message_runner: Arc::new(direct_message_runner),
            group_channel_runner: Arc::new(group_channel_runner),
            task_lifecycle_runner: Arc::new(task_lifecycle_runner),
            escrow_settlement_runner: Arc::new(escrow_settlement_runner),
            proof_verification_runner: Arc::new(proof_verification_runner),
            replay_protection_runner: Arc::new(replay_protection_runner),
            crash_recovery_runner: Arc::new(crash_recovery_runner),
            transport_failover_runner: Arc::new(transport_failover_runner),
            topology_coherence_runner: Arc::new(topology_coherence_runner),
            signer_rotation_runner: Arc::new(signer_rotation_runner),
            retention_deletion_runner: Arc::new(retention_deletion_runner),
            bridge_forwarding_runner: Arc::new(bridge_forwarding_runner),
            batch_merkle_runner: Arc::new(batch_merkle_runner),
            performance_smoke_runner: Arc::new(performance_smoke_runner),
        }
    }
}

impl HarnessDriver for CliScriptedDriver {
    fn mode(&self) -> ExecutionMode {
        ExecutionMode::CliScripted
    }

    fn execute(&self, scenario_id: &'static str) -> DriverExecutionResult {
        let status = if !is_live_bound_scenario_id(scenario_id) {
            "pass"
        } else if !self.live_execution_enabled {
            "fail"
        } else {
            match self.live_runner_for_scenario(scenario_id) {
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

impl CliScriptedDriver {
    fn live_runner_for_scenario(&self, scenario_id: &'static str) -> Option<Result<(), String>> {
        match scenario_id {
            "S-01" => Some((self.discovery_runner)()),
            "S-02" => Some((self.direct_message_runner)()),
            "S-03" => Some((self.group_channel_runner)()),
            "S-04" => Some((self.task_lifecycle_runner)()),
            "S-05" => Some((self.escrow_settlement_runner)()),
            "S-06" => Some((self.proof_verification_runner)()),
            "S-07" => Some((self.replay_protection_runner)()),
            "S-08" => Some((self.crash_recovery_runner)()),
            "S-09" => Some((self.transport_failover_runner)()),
            "S-10" => Some((self.topology_coherence_runner)()),
            "S-11" => Some((self.signer_rotation_runner)()),
            "S-12" => Some((self.retention_deletion_runner)()),
            "S-13" => Some((self.bridge_forwarding_runner)()),
            "S-14" => Some((self.batch_merkle_runner)()),
            "S-15" => Some((self.performance_smoke_runner)()),
            _ => None,
        }
    }
}

fn is_live_bound_scenario_id(scenario_id: &str) -> bool {
    matches!(
        scenario_id,
        "S-01"
            | "S-02"
            | "S-03"
            | "S-04"
            | "S-05"
            | "S-06"
            | "S-07"
            | "S-08"
            | "S-09"
            | "S-10"
            | "S-11"
            | "S-12"
            | "S-13"
            | "S-14"
            | "S-15"
    )
}

fn live_execution_enabled_from_env() -> bool {
    env::var(CLI_SCRIPTED_LIVE_ENV)
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

fn run_live_s01_cli_health_probe() -> Result<(), String> {
    let cli_binary = env_var_or_default(CLI_BINARY_ENV, DEFAULT_CLI_BINARY);
    let endpoint = env_var_or_default("KAMN_ENDPOINT", "http://localhost:8080");

    let status = Command::new(cli_binary.as_str())
        .arg("health")
        .arg("--endpoint")
        .arg(endpoint.as_str())
        .arg("--format")
        .arg("text")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map_err(|error| format!("cli live health probe failed to spawn: {error}"))?;

    if status.success() {
        return Ok(());
    }

    let exit_status = status
        .code()
        .map(|value| value.to_string())
        .unwrap_or_else(|| "signal".to_owned());
    Err(format!(
        "cli live health probe failed (exit_status={exit_status})"
    ))
}

fn run_live_s02_cli_direct_message_probe() -> Result<(), String> {
    let cli_binary = env_var_or_default(CLI_BINARY_ENV, DEFAULT_CLI_BINARY);
    let endpoint = env_var_or_default("KAMN_ENDPOINT", "http://localhost:8080");
    let base_agent_name =
        env_var_or_default("KAMN_AGENT_NAME", DEFAULT_S02_AGENT_NAME);
    let message_payload = env::var("KAMN_E2E_S02_MESSAGE_PAYLOAD")
        .unwrap_or_else(|_| DEFAULT_S02_MESSAGE_PAYLOAD.to_owned());
    let reply_payload = env::var("KAMN_E2E_S02_REPLY_PAYLOAD")
        .unwrap_or_else(|_| DEFAULT_S02_REPLY_PAYLOAD.to_owned());
    let send_agent_name = format!("{base_agent_name}-send");
    let query_agent_name = format!("{base_agent_name}-query");
    let reply_agent_name = format!("{base_agent_name}-reply");
    let reply_query_agent_name = format!("{base_agent_name}-query-reply");

    let send_output = run_cli_command_capture_stdout_with_agent_name(
        cli_binary.as_str(),
        &[
            "send-message",
            "--endpoint",
            endpoint.as_str(),
            "--format",
            "text",
            message_payload.as_str(),
        ],
        "cli live s02 send-message",
        send_agent_name.as_str(),
    )?;
    let message_id = parse_text_output_field(send_output.as_str(), "message_id")
        .ok_or_else(|| {
            format!("cli live s02 send-message response missing message_id field: {send_output}")
        })?
        .to_owned();
    if message_id.trim().is_empty() {
        return Err("cli live s02 send-message returned empty message_id".to_owned());
    }
    let send_status = parse_text_output_field(send_output.as_str(), "status").ok_or_else(|| {
        format!("cli live s02 send-message response missing status field: {send_output}")
    })?;
    if send_status.trim().is_empty() {
        return Err("cli live s02 send-message returned empty status".to_owned());
    }

    let query_output = run_cli_command_capture_stdout_with_agent_name(
        cli_binary.as_str(),
        &[
            "query-message",
            "--endpoint",
            endpoint.as_str(),
            "--format",
            "text",
            message_id.as_str(),
        ],
        "cli live s02 query-message",
        query_agent_name.as_str(),
    )?;
    let queried_message_id = parse_text_output_field(query_output.as_str(), "message_id")
        .ok_or_else(|| {
            format!("cli live s02 query-message response missing message_id field: {query_output}")
        })?;
    if queried_message_id != message_id {
        return Err(format!(
            "cli live s02 query-message returned mismatched message_id: expected={message_id}, got={queried_message_id}"
        ));
    }
    let queried_status =
        parse_text_output_field(query_output.as_str(), "status").ok_or_else(|| {
            format!("cli live s02 query-message response missing status field: {query_output}")
        })?;
    if queried_status.trim().is_empty() {
        return Err("cli live s02 query-message returned empty status".to_owned());
    }

    let reply_output = run_cli_command_capture_stdout_with_agent_name(
        cli_binary.as_str(),
        &[
            "send-message",
            "--endpoint",
            endpoint.as_str(),
            "--format",
            "text",
            reply_payload.as_str(),
        ],
        "cli live s02 reply send-message",
        reply_agent_name.as_str(),
    )?;
    let reply_message_id = parse_text_output_field(reply_output.as_str(), "message_id")
        .ok_or_else(|| {
            format!(
                "cli live s02 reply send-message response missing message_id field: {reply_output}"
            )
        })?
        .to_owned();
    if reply_message_id.trim().is_empty() {
        return Err("cli live s02 reply send-message returned empty message_id".to_owned());
    }
    let reply_send_status =
        parse_text_output_field(reply_output.as_str(), "status").ok_or_else(|| {
            format!("cli live s02 reply send-message response missing status field: {reply_output}")
        })?;
    if reply_send_status.trim().is_empty() {
        return Err("cli live s02 reply send-message returned empty status".to_owned());
    }

    let reply_query_output = run_cli_command_capture_stdout_with_agent_name(
        cli_binary.as_str(),
        &[
            "query-message",
            "--endpoint",
            endpoint.as_str(),
            "--format",
            "text",
            reply_message_id.as_str(),
        ],
        "cli live s02 reply query-message",
        reply_query_agent_name.as_str(),
    )?;
    let reply_queried_message_id = parse_text_output_field(reply_query_output.as_str(), "message_id")
        .ok_or_else(|| {
            format!(
                "cli live s02 reply query-message response missing message_id field: {reply_query_output}"
            )
        })?;
    if reply_queried_message_id != reply_message_id {
        return Err(format!(
            "cli live s02 reply query-message returned mismatched message_id: expected={reply_message_id}, got={reply_queried_message_id}"
        ));
    }
    let reply_queried_status =
        parse_text_output_field(reply_query_output.as_str(), "status").ok_or_else(|| {
            format!(
                "cli live s02 reply query-message response missing status field: {reply_query_output}"
            )
        })?;
    if reply_queried_status.trim().is_empty() {
        return Err("cli live s02 reply query-message returned empty status".to_owned());
    }

    Ok(())
}

fn run_live_s03_cli_group_channel_probe() -> Result<(), String> {
    let cli_binary = env_var_or_default(CLI_BINARY_ENV, DEFAULT_CLI_BINARY);
    let endpoint = env_var_or_default("KAMN_ENDPOINT", "http://localhost:8080");
    let base_agent_name =
        env_var_or_default("KAMN_AGENT_NAME", DEFAULT_S03_AGENT_NAME);
    let channel_payload = env::var("KAMN_E2E_S03_CHANNEL_PAYLOAD")
        .unwrap_or_else(|_| DEFAULT_S03_CHANNEL_PAYLOAD.to_owned());
    let message_payload = env::var("KAMN_E2E_S03_MESSAGE_PAYLOAD")
        .unwrap_or_else(|_| DEFAULT_S03_MESSAGE_PAYLOAD.to_owned());
    let create_agent_name = format!("{base_agent_name}-create-channel");
    let send_agent_name = format!("{base_agent_name}-send-message");
    let query_agent_name = format!("{base_agent_name}-query-message");
    let list_agent_name = format!("{base_agent_name}-list-messages");

    let create_output = run_cli_command_capture_stdout_with_agent_name(
        cli_binary.as_str(),
        &[
            "create-channel",
            "--endpoint",
            endpoint.as_str(),
            "--format",
            "text",
            channel_payload.as_str(),
        ],
        "cli live s03 create-channel",
        create_agent_name.as_str(),
    )?;
    let channel_id = parse_text_output_field(create_output.as_str(), "channel_id")
        .ok_or_else(|| {
            format!(
                "cli live s03 create-channel response missing channel_id field: {create_output}"
            )
        })?
        .to_owned();
    if channel_id.trim().is_empty() {
        return Err("cli live s03 create-channel returned empty channel_id".to_owned());
    }
    let create_status =
        parse_text_output_field(create_output.as_str(), "status").ok_or_else(|| {
            format!("cli live s03 create-channel response missing status field: {create_output}")
        })?;
    if create_status.trim().is_empty() {
        return Err("cli live s03 create-channel returned empty status".to_owned());
    }

    let send_output = run_cli_command_capture_stdout_with_agent_name(
        cli_binary.as_str(),
        &[
            "send-message",
            "--endpoint",
            endpoint.as_str(),
            "--format",
            "text",
            message_payload.as_str(),
        ],
        "cli live s03 send-message",
        send_agent_name.as_str(),
    )?;
    let message_id = parse_text_output_field(send_output.as_str(), "message_id")
        .ok_or_else(|| {
            format!("cli live s03 send-message response missing message_id field: {send_output}")
        })?
        .to_owned();
    if message_id.trim().is_empty() {
        return Err("cli live s03 send-message returned empty message_id".to_owned());
    }
    let send_status = parse_text_output_field(send_output.as_str(), "status").ok_or_else(|| {
        format!("cli live s03 send-message response missing status field: {send_output}")
    })?;
    if send_status.trim().is_empty() {
        return Err("cli live s03 send-message returned empty status".to_owned());
    }

    let query_output = run_cli_command_capture_stdout_with_agent_name(
        cli_binary.as_str(),
        &[
            "query-message",
            "--endpoint",
            endpoint.as_str(),
            "--format",
            "text",
            message_id.as_str(),
        ],
        "cli live s03 query-message",
        query_agent_name.as_str(),
    )?;
    let queried_message_id = parse_text_output_field(query_output.as_str(), "message_id")
        .ok_or_else(|| {
            format!("cli live s03 query-message response missing message_id field: {query_output}")
        })?;
    if queried_message_id != message_id {
        return Err(format!(
            "cli live s03 query-message returned mismatched message_id: expected={message_id}, got={queried_message_id}"
        ));
    }
    let queried_status =
        parse_text_output_field(query_output.as_str(), "status").ok_or_else(|| {
            format!("cli live s03 query-message response missing status field: {query_output}")
        })?;
    if queried_status.trim().is_empty() {
        return Err("cli live s03 query-message returned empty status".to_owned());
    }

    let list_output = run_cli_command_capture_stdout_with_agent_name(
        cli_binary.as_str(),
        &[
            "list-messages",
            "--endpoint",
            endpoint.as_str(),
            "--format",
            "text",
            channel_id.as_str(),
        ],
        "cli live s03 list-messages",
        list_agent_name.as_str(),
    )?;
    let listed_channel_id = parse_text_output_field(list_output.as_str(), "channel_id")
        .ok_or_else(|| {
            format!("cli live s03 list-messages response missing channel_id field: {list_output}")
        })?;
    if listed_channel_id != channel_id {
        return Err(format!(
            "cli live s03 list-messages returned mismatched channel_id: expected={channel_id}, got={listed_channel_id}"
        ));
    }
    let _listed_messages =
        parse_text_output_field(list_output.as_str(), "messages").ok_or_else(|| {
            format!("cli live s03 list-messages response missing messages field: {list_output}")
        })?;

    Ok(())
}

fn run_live_s04_cli_task_lifecycle_probe() -> Result<(), String> {
    let cli_binary = env_var_or_default(CLI_BINARY_ENV, DEFAULT_CLI_BINARY);
    let endpoint = env_var_or_default("KAMN_ENDPOINT", "http://localhost:8080");
    let base_agent_name =
        env_var_or_default("KAMN_AGENT_NAME", DEFAULT_S04_AGENT_NAME);
    let create_task_payload = env::var("KAMN_E2E_S04_CREATE_TASK_PAYLOAD")
        .unwrap_or_else(|_| DEFAULT_S04_CREATE_TASK_PAYLOAD.to_owned());
    let create_agent_name = format!("{base_agent_name}-create");
    let fund_agent_name = format!("{base_agent_name}-fund");
    let accept_agent_name = format!("{base_agent_name}-accept");
    let complete_agent_name = format!("{base_agent_name}-complete");
    let release_agent_name = format!("{base_agent_name}-release");

    let create_output = run_cli_command_capture_stdout_with_agent_name(
        cli_binary.as_str(),
        &[
            "create-task",
            "--endpoint",
            endpoint.as_str(),
            "--format",
            "text",
            create_task_payload.as_str(),
        ],
        "cli live s04 create-task",
        create_agent_name.as_str(),
    )?;
    let task_id = parse_text_output_field(create_output.as_str(), "task_id")
        .ok_or_else(|| {
            format!("cli live s04 create-task response missing task_id field: {create_output}")
        })?
        .to_owned();
    if task_id.trim().is_empty() {
        return Err("cli live s04 create-task returned empty task_id".to_owned());
    }

    let fund_payload = format!(
        "{{\"task_id\":\"{}\",\"amount\":{}}}",
        task_id, DEFAULT_S04_ESCROW_AMOUNT
    );
    let fund_output = run_cli_command_capture_stdout_with_agent_name(
        cli_binary.as_str(),
        &[
            "fund-escrow",
            "--endpoint",
            endpoint.as_str(),
            "--format",
            "text",
            fund_payload.as_str(),
        ],
        "cli live s04 fund-escrow",
        fund_agent_name.as_str(),
    )?;
    let escrow_id = parse_text_output_field(fund_output.as_str(), "escrow_id")
        .ok_or_else(|| {
            format!("cli live s04 fund-escrow response missing escrow_id field: {fund_output}")
        })?
        .to_owned();
    if escrow_id.trim().is_empty() {
        return Err("cli live s04 fund-escrow returned empty escrow_id".to_owned());
    }

    let accept_output = run_cli_command_capture_stdout_with_agent_name(
        cli_binary.as_str(),
        &[
            "accept-task",
            "--endpoint",
            endpoint.as_str(),
            "--format",
            "text",
            task_id.as_str(),
        ],
        "cli live s04 accept-task",
        accept_agent_name.as_str(),
    )?;
    let accept_state =
        parse_text_output_field(accept_output.as_str(), "state").ok_or_else(|| {
            format!("cli live s04 accept-task response missing state field: {accept_output}")
        })?;
    if accept_state.trim().is_empty() {
        return Err("cli live s04 accept-task returned empty state".to_owned());
    }

    let complete_output = run_cli_command_capture_stdout_with_agent_name(
        cli_binary.as_str(),
        &[
            "complete-task",
            "--endpoint",
            endpoint.as_str(),
            "--format",
            "text",
            task_id.as_str(),
        ],
        "cli live s04 complete-task",
        complete_agent_name.as_str(),
    )?;
    let complete_state =
        parse_text_output_field(complete_output.as_str(), "state").ok_or_else(|| {
            format!("cli live s04 complete-task response missing state field: {complete_output}")
        })?;
    if complete_state.trim().is_empty() {
        return Err("cli live s04 complete-task returned empty state".to_owned());
    }

    let release_output = run_cli_command_capture_stdout_with_agent_name(
        cli_binary.as_str(),
        &[
            "release-escrow",
            "--endpoint",
            endpoint.as_str(),
            "--format",
            "text",
            escrow_id.as_str(),
        ],
        "cli live s04 release-escrow",
        release_agent_name.as_str(),
    )?;
    let release_state =
        parse_text_output_field(release_output.as_str(), "state").ok_or_else(|| {
            format!("cli live s04 release-escrow response missing state field: {release_output}")
        })?;
    if release_state.trim().is_empty() {
        return Err("cli live s04 release-escrow returned empty state".to_owned());
    }

    Ok(())
}

fn run_live_s05_cli_escrow_settlement_probe() -> Result<(), String> {
    let cli_binary = env_var_or_default(CLI_BINARY_ENV, DEFAULT_CLI_BINARY);
    let endpoint = env_var_or_default("KAMN_ENDPOINT", "http://localhost:8080");
    let base_agent_name =
        env_var_or_default("KAMN_AGENT_NAME", DEFAULT_S05_AGENT_NAME);
    let fund_payload = env::var("KAMN_E2E_S05_FUND_ESCROW_PAYLOAD")
        .unwrap_or_else(|_| DEFAULT_S05_FUND_ESCROW_PAYLOAD.to_owned());
    let fund_agent_name = format!("{base_agent_name}-fund");
    let release_agent_name = format!("{base_agent_name}-release");

    let fund_output = run_cli_command_capture_stdout_with_agent_name(
        cli_binary.as_str(),
        &[
            "fund-escrow",
            "--endpoint",
            endpoint.as_str(),
            "--format",
            "text",
            fund_payload.as_str(),
        ],
        "cli live s05 fund-escrow",
        fund_agent_name.as_str(),
    )?;
    let escrow_id = parse_text_output_field(fund_output.as_str(), "escrow_id")
        .ok_or_else(|| {
            format!("cli live s05 fund-escrow response missing escrow_id field: {fund_output}")
        })?
        .to_owned();
    if escrow_id.trim().is_empty() {
        return Err("cli live s05 fund-escrow returned empty escrow_id".to_owned());
    }
    let fund_state = parse_text_output_field(fund_output.as_str(), "state").ok_or_else(|| {
        format!("cli live s05 fund-escrow response missing state field: {fund_output}")
    })?;
    if fund_state.trim().is_empty() {
        return Err("cli live s05 fund-escrow returned empty state".to_owned());
    }

    let release_output = run_cli_command_capture_stdout_with_agent_name(
        cli_binary.as_str(),
        &[
            "release-escrow",
            "--endpoint",
            endpoint.as_str(),
            "--format",
            "text",
            escrow_id.as_str(),
        ],
        "cli live s05 release-escrow",
        release_agent_name.as_str(),
    )?;
    let released_escrow_id = parse_text_output_field(release_output.as_str(), "escrow_id")
        .ok_or_else(|| {
            format!(
                "cli live s05 release-escrow response missing escrow_id field: {release_output}"
            )
        })?;
    let release_state =
        parse_text_output_field(release_output.as_str(), "state").ok_or_else(|| {
            format!("cli live s05 release-escrow response missing state field: {release_output}")
        })?;
    validate_live_s05_release_escrow_response(
        escrow_id.as_str(),
        released_escrow_id,
        release_state,
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
            "cli live s05 release-escrow returned mismatched escrow_id: expected={expected_escrow_id}, got={released_escrow_id}"
        ));
    }
    if release_state.trim().is_empty() {
        return Err("cli live s05 release-escrow returned empty state".to_owned());
    }
    Ok(())
}

fn run_live_s06_cli_proof_verification_probe() -> Result<(), String> {
    let cli_binary = env_var_or_default(CLI_BINARY_ENV, DEFAULT_CLI_BINARY);
    let endpoint = env_var_or_default("KAMN_ENDPOINT", "http://localhost:8080");
    let message_id = env::var("KAMN_E2E_S06_PROOF_MESSAGE_ID")
        .unwrap_or_else(|_| DEFAULT_S06_MESSAGE_ID.to_owned());
    let tx_hash =
        env_var_or_default("KAMN_E2E_S06_PROOF_TX_HASH", DEFAULT_S06_TX_HASH);
    let block_height = env::var("KAMN_E2E_S06_PROOF_BLOCK_HEIGHT")
        .ok()
        .map(|raw| {
            raw.trim()
                .parse::<u64>()
                .map_err(|_| format!("cli live s06 invalid block height env value: {raw}"))
        })
        .transpose()?
        .unwrap_or(DEFAULT_S06_BLOCK_HEIGHT);
    let finality =
        env_var_or_default("KAMN_E2E_S06_PROOF_FINALITY", DEFAULT_S06_FINALITY);
    let block_height_value = block_height.to_string();

    let output = run_cli_command_capture_stdout(
        cli_binary.as_str(),
        &[
            "verify-proof",
            "--endpoint",
            endpoint.as_str(),
            "--format",
            "text",
            message_id.as_str(),
            tx_hash.as_str(),
            block_height_value.as_str(),
            finality.as_str(),
        ],
        "cli live s06 verify-proof",
    )?;

    let verified = parse_text_output_field(output.as_str(), "verified").ok_or_else(|| {
        format!("cli live s06 verify-proof response missing verified field: {output}")
    })?;
    if verified != "true" {
        return Err(format!(
            "cli live s06 verify-proof returned verified={verified}"
        ));
    }

    let reported_finality =
        parse_text_output_field(output.as_str(), "finality").ok_or_else(|| {
            format!("cli live s06 verify-proof response missing finality field: {output}")
        })?;
    if reported_finality != "FINAL" {
        return Err(format!(
            "cli live s06 verify-proof returned non-final finality: {reported_finality}"
        ));
    }

    let reported_height =
        parse_text_output_field(output.as_str(), "block_height").ok_or_else(|| {
            format!("cli live s06 verify-proof response missing block_height field: {output}")
        })?;
    let parsed_height = reported_height.parse::<u64>().map_err(|_| {
        format!("cli live s06 verify-proof returned invalid block_height: {output}")
    })?;
    if parsed_height == 0 {
        return Err("cli live s06 verify-proof returned block_height=0".to_owned());
    }

    Ok(())
}

fn run_live_s07_cli_replay_protection_probe() -> Result<(), String> {
    let cli_binary = env_var_or_default(CLI_BINARY_ENV, DEFAULT_CLI_BINARY);
    let endpoint = env_var_or_default("KAMN_ENDPOINT", "http://localhost:8080");
    let base_agent_name =
        env_var_or_default("KAMN_E2E_S07_AGENT_NAME", DEFAULT_S07_AGENT_NAME);
    let message_payload = env::var("KAMN_E2E_S07_REPLAY_PAYLOAD")
        .unwrap_or_else(|_| DEFAULT_S07_MESSAGE_PAYLOAD.to_owned());
    let replay_agent_name = format!(
        "{base_agent_name}-{}",
        live_s07_probe_agent_suffix().as_str()
    );

    let initial_output = run_cli_command_capture_stdout_with_agent_name(
        cli_binary.as_str(),
        &[
            "send-message",
            "--endpoint",
            endpoint.as_str(),
            "--format",
            "text",
            message_payload.as_str(),
        ],
        "cli live s07 initial send-message",
        replay_agent_name.as_str(),
    )?;
    let initial_message_id = parse_text_output_field(initial_output.as_str(), "message_id")
        .ok_or_else(|| {
            format!(
                "cli live s07 initial send-message response missing message_id field: {initial_output}"
            )
        })?;
    if initial_message_id.trim().is_empty() {
        return Err("cli live s07 initial send-message returned empty message_id".to_owned());
    }
    let initial_status =
        parse_text_output_field(initial_output.as_str(), "status").ok_or_else(|| {
            format!(
                "cli live s07 initial send-message response missing status field: {initial_output}"
            )
        })?;
    if initial_status.trim().is_empty() {
        return Err("cli live s07 initial send-message returned empty status".to_owned());
    }

    let replay_error = run_cli_command_expect_failure_with_agent_name(
        cli_binary.as_str(),
        &[
            "send-message",
            "--endpoint",
            endpoint.as_str(),
            "--format",
            "text",
            message_payload.as_str(),
        ],
        "cli live s07 replay send-message",
        replay_agent_name.as_str(),
    )?;
    validate_s07_replay_reason_marker(replay_error.as_str(), "cli live s07 replay send-message")?;

    Ok(())
}

fn run_live_s08_cli_crash_recovery_probe() -> Result<(), String> {
    let cli_binary = env_var_or_default(CLI_BINARY_ENV, DEFAULT_CLI_BINARY);
    let endpoint = env_var_or_default("KAMN_ENDPOINT", "http://localhost:8080");
    let base_agent_name =
        env_var_or_default("KAMN_E2E_S08_AGENT_NAME", DEFAULT_S08_AGENT_NAME);
    let pre_message_payload = env::var("KAMN_E2E_S08_PRE_MESSAGE_PAYLOAD")
        .unwrap_or_else(|_| DEFAULT_S08_PRE_MESSAGE_PAYLOAD.to_owned());
    let post_message_payload = env::var("KAMN_E2E_S08_POST_MESSAGE_PAYLOAD")
        .unwrap_or_else(|_| DEFAULT_S08_POST_MESSAGE_PAYLOAD.to_owned());

    let pre_send_output = run_cli_command_capture_stdout_with_agent_name(
        cli_binary.as_str(),
        &[
            "send-message",
            "--endpoint",
            endpoint.as_str(),
            "--format",
            "text",
            pre_message_payload.as_str(),
        ],
        "cli live s08 pre-boundary send-message",
        format!("{base_agent_name}-pre-send").as_str(),
    )?;
    let pre_message_id = validate_s08_message_receipt_fields(
        pre_send_output.as_str(),
        "cli live s08 pre-boundary send-message",
    )?;

    let pre_query_output = run_cli_command_capture_stdout_with_agent_name(
        cli_binary.as_str(),
        &[
            "query-message",
            "--endpoint",
            endpoint.as_str(),
            "--format",
            "text",
            pre_message_id.as_str(),
        ],
        "cli live s08 pre-boundary query-message",
        format!("{base_agent_name}-pre-query").as_str(),
    )?;
    validate_s08_query_message_response(
        pre_query_output.as_str(),
        pre_message_id.as_str(),
        "cli live s08 pre-boundary query-message",
    )?;

    run_cli_command_capture_stdout_with_agent_name(
        cli_binary.as_str(),
        &[
            "health",
            "--endpoint",
            endpoint.as_str(),
            "--format",
            "text",
        ],
        "cli live s08 boundary health check",
        format!("{base_agent_name}-boundary").as_str(),
    )?;

    let post_send_output = run_cli_command_capture_stdout_with_agent_name(
        cli_binary.as_str(),
        &[
            "send-message",
            "--endpoint",
            endpoint.as_str(),
            "--format",
            "text",
            post_message_payload.as_str(),
        ],
        "cli live s08 post-boundary send-message",
        format!("{base_agent_name}-post-send").as_str(),
    )?;
    let post_message_id = validate_s08_message_receipt_fields(
        post_send_output.as_str(),
        "cli live s08 post-boundary send-message",
    )?;
    if post_message_id == pre_message_id {
        return Err(
            "cli live s08 post-boundary send-message returned duplicate message_id".to_owned(),
        );
    }

    let post_query_output = run_cli_command_capture_stdout_with_agent_name(
        cli_binary.as_str(),
        &[
            "query-message",
            "--endpoint",
            endpoint.as_str(),
            "--format",
            "text",
            post_message_id.as_str(),
        ],
        "cli live s08 post-boundary query-message",
        format!("{base_agent_name}-post-query").as_str(),
    )?;
    validate_s08_query_message_response(
        post_query_output.as_str(),
        post_message_id.as_str(),
        "cli live s08 post-boundary query-message",
    )?;

    Ok(())
}

fn run_live_s09_cli_transport_failover_probe() -> Result<(), String> {
    let cli_binary = env_var_or_default(CLI_BINARY_ENV, DEFAULT_CLI_BINARY);
    let primary_endpoint =
        env_var_or_default("KAMN_ENDPOINT", "http://localhost:8080");
    let failover_endpoint =
        env_var_or_else("KAMN_E2E_S09_FAILOVER_ENDPOINT", || primary_endpoint.clone());
    let base_agent_name =
        env_var_or_default("KAMN_E2E_S09_AGENT_NAME", DEFAULT_S09_AGENT_NAME);
    let pre_message_payload = env::var("KAMN_E2E_S09_PRE_MESSAGE_PAYLOAD")
        .unwrap_or_else(|_| DEFAULT_S09_PRE_MESSAGE_PAYLOAD.to_owned());
    let post_message_payload = env::var("KAMN_E2E_S09_POST_MESSAGE_PAYLOAD")
        .unwrap_or_else(|_| DEFAULT_S09_POST_MESSAGE_PAYLOAD.to_owned());

    let pre_send_output = run_cli_command_capture_stdout_with_agent_name(
        cli_binary.as_str(),
        &[
            "send-message",
            "--endpoint",
            primary_endpoint.as_str(),
            "--format",
            "text",
            pre_message_payload.as_str(),
        ],
        "cli live s09 pre-failover send-message",
        format!("{base_agent_name}-pre-send").as_str(),
    )?;
    let pre_message_id = validate_s08_message_receipt_fields(
        pre_send_output.as_str(),
        "cli live s09 pre-failover send-message",
    )?;

    let pre_query_output = run_cli_command_capture_stdout_with_agent_name(
        cli_binary.as_str(),
        &[
            "query-message",
            "--endpoint",
            primary_endpoint.as_str(),
            "--format",
            "text",
            pre_message_id.as_str(),
        ],
        "cli live s09 pre-failover query-message",
        format!("{base_agent_name}-pre-query").as_str(),
    )?;
    validate_s08_query_message_response(
        pre_query_output.as_str(),
        pre_message_id.as_str(),
        "cli live s09 pre-failover query-message",
    )?;

    let boundary_output = run_cli_command_capture_stdout_with_agent_name(
        cli_binary.as_str(),
        &[
            "health",
            "--endpoint",
            failover_endpoint.as_str(),
            "--format",
            "text",
        ],
        "cli live s09 failover boundary health check",
        format!("{base_agent_name}-boundary").as_str(),
    )?;
    let boundary_status =
        parse_text_output_field(boundary_output.as_str(), "status").ok_or_else(|| {
            format!(
                "cli live s09 failover boundary health response missing status field: {boundary_output}"
            )
        })?;
    if boundary_status.trim().is_empty() {
        return Err("cli live s09 failover boundary health returned empty status".to_owned());
    }

    let post_send_output = run_cli_command_capture_stdout_with_agent_name(
        cli_binary.as_str(),
        &[
            "send-message",
            "--endpoint",
            failover_endpoint.as_str(),
            "--format",
            "text",
            post_message_payload.as_str(),
        ],
        "cli live s09 post-failover send-message",
        format!("{base_agent_name}-post-send").as_str(),
    )?;
    let post_message_id = validate_s08_message_receipt_fields(
        post_send_output.as_str(),
        "cli live s09 post-failover send-message",
    )?;
    if post_message_id == pre_message_id {
        return Err(
            "cli live s09 post-failover send-message returned duplicate message_id".to_owned(),
        );
    }

    let post_query_output = run_cli_command_capture_stdout_with_agent_name(
        cli_binary.as_str(),
        &[
            "query-message",
            "--endpoint",
            failover_endpoint.as_str(),
            "--format",
            "text",
            post_message_id.as_str(),
        ],
        "cli live s09 post-failover query-message",
        format!("{base_agent_name}-post-query").as_str(),
    )?;
    validate_s08_query_message_response(
        post_query_output.as_str(),
        post_message_id.as_str(),
        "cli live s09 post-failover query-message",
    )?;

    Ok(())
}

fn run_live_s10_cli_topology_coherence_probe() -> Result<(), String> {
    let cli_binary = env_var_or_default(CLI_BINARY_ENV, DEFAULT_CLI_BINARY);
    let primary_endpoint = env::var("KAMN_E2E_S10_PRIMARY_ENDPOINT")
        .or_else(|_| env::var("KAMN_ENDPOINT"))
        .unwrap_or_else(|_| "http://localhost:8080".to_owned());
    let secondary_endpoint =
        env_var_or_else("KAMN_E2E_S10_SECONDARY_ENDPOINT", || primary_endpoint.clone());
    let tertiary_endpoint =
        env_var_or_else("KAMN_E2E_S10_TERTIARY_ENDPOINT", || secondary_endpoint.clone());
    let base_agent_name =
        env_var_or_default("KAMN_E2E_S10_AGENT_NAME", DEFAULT_S10_AGENT_NAME);
    let message_payload = env::var("KAMN_E2E_S10_MESSAGE_PAYLOAD")
        .unwrap_or_else(|_| DEFAULT_S10_MESSAGE_PAYLOAD.to_owned());

    let primary_send_output = run_cli_command_capture_stdout_with_agent_name(
        cli_binary.as_str(),
        &[
            "send-message",
            "--endpoint",
            primary_endpoint.as_str(),
            "--format",
            "text",
            message_payload.as_str(),
        ],
        "cli live s10 primary send-message",
        format!("{base_agent_name}-primary-send").as_str(),
    )?;
    let message_id = validate_s08_message_receipt_fields(
        primary_send_output.as_str(),
        "cli live s10 primary send-message",
    )?;

    let secondary_query_output = run_cli_command_capture_stdout_with_agent_name(
        cli_binary.as_str(),
        &[
            "query-message",
            "--endpoint",
            secondary_endpoint.as_str(),
            "--format",
            "text",
            message_id.as_str(),
        ],
        "cli live s10 secondary query-message",
        format!("{base_agent_name}-secondary-query").as_str(),
    )?;
    validate_s08_query_message_response(
        secondary_query_output.as_str(),
        message_id.as_str(),
        "cli live s10 secondary query-message",
    )?;

    let tertiary_query_output = run_cli_command_capture_stdout_with_agent_name(
        cli_binary.as_str(),
        &[
            "query-message",
            "--endpoint",
            tertiary_endpoint.as_str(),
            "--format",
            "text",
            message_id.as_str(),
        ],
        "cli live s10 tertiary query-message",
        format!("{base_agent_name}-tertiary-query").as_str(),
    )?;
    validate_s08_query_message_response(
        tertiary_query_output.as_str(),
        message_id.as_str(),
        "cli live s10 tertiary query-message",
    )?;

    let secondary_health_output = run_cli_command_capture_stdout_with_agent_name(
        cli_binary.as_str(),
        &[
            "health",
            "--endpoint",
            secondary_endpoint.as_str(),
            "--format",
            "text",
        ],
        "cli live s10 secondary health check",
        format!("{base_agent_name}-secondary-boundary").as_str(),
    )?;
    let secondary_health_status =
        parse_text_output_field(secondary_health_output.as_str(), "status").ok_or_else(|| {
            format!(
                "cli live s10 secondary health response missing status field: {secondary_health_output}"
            )
        })?;
    if secondary_health_status.trim().is_empty() {
        return Err("cli live s10 secondary health check returned empty status".to_owned());
    }

    let tertiary_health_output = run_cli_command_capture_stdout_with_agent_name(
        cli_binary.as_str(),
        &[
            "health",
            "--endpoint",
            tertiary_endpoint.as_str(),
            "--format",
            "text",
        ],
        "cli live s10 tertiary health check",
        format!("{base_agent_name}-tertiary-boundary").as_str(),
    )?;
    let tertiary_health_status =
        parse_text_output_field(tertiary_health_output.as_str(), "status").ok_or_else(|| {
            format!(
                "cli live s10 tertiary health response missing status field: {tertiary_health_output}"
            )
        })?;
    if tertiary_health_status.trim().is_empty() {
        return Err("cli live s10 tertiary health check returned empty status".to_owned());
    }

    Ok(())
}

fn run_live_s11_cli_signer_rotation_probe() -> Result<(), String> {
    let cli_binary = env_var_or_default(CLI_BINARY_ENV, DEFAULT_CLI_BINARY);
    let endpoint = env_var_or_default("KAMN_ENDPOINT", "http://localhost:8080");
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

    let primary_send_output = run_cli_command_capture_stdout_with_agent_name(
        cli_binary.as_str(),
        &[
            "send-message",
            "--endpoint",
            endpoint.as_str(),
            "--format",
            "text",
            message_payload.as_str(),
        ],
        "cli live s11 primary send-message",
        primary_agent_name.as_str(),
    )?;
    let primary_message_id = validate_s08_message_receipt_fields(
        primary_send_output.as_str(),
        "cli live s11 primary send-message",
    )?;

    let primary_query_output = run_cli_command_capture_stdout_with_agent_name(
        cli_binary.as_str(),
        &[
            "query-message",
            "--endpoint",
            endpoint.as_str(),
            "--format",
            "text",
            primary_message_id.as_str(),
        ],
        "cli live s11 primary query-message",
        format!("{primary_agent_name}-query").as_str(),
    )?;
    validate_s08_query_message_response(
        primary_query_output.as_str(),
        primary_message_id.as_str(),
        "cli live s11 primary query-message",
    )?;

    let rotated_send_output = run_cli_command_capture_stdout_with_agent_name(
        cli_binary.as_str(),
        &[
            "send-message",
            "--endpoint",
            endpoint.as_str(),
            "--format",
            "text",
            rotated_message_payload.as_str(),
        ],
        "cli live s11 rotated send-message",
        rotated_agent_name.as_str(),
    )?;
    let rotated_message_id = validate_s08_message_receipt_fields(
        rotated_send_output.as_str(),
        "cli live s11 rotated send-message",
    )?;
    if rotated_message_id == primary_message_id {
        return Err("cli live s11 rotated send-message returned duplicate message_id".to_owned());
    }

    let rotated_query_output = run_cli_command_capture_stdout_with_agent_name(
        cli_binary.as_str(),
        &[
            "query-message",
            "--endpoint",
            endpoint.as_str(),
            "--format",
            "text",
            rotated_message_id.as_str(),
        ],
        "cli live s11 rotated query-message",
        format!("{rotated_agent_name}-query").as_str(),
    )?;
    validate_s08_query_message_response(
        rotated_query_output.as_str(),
        rotated_message_id.as_str(),
        "cli live s11 rotated query-message",
    )?;

    let stale_primary_error = run_cli_command_expect_failure_with_agent_name(
        cli_binary.as_str(),
        &[
            "send-message",
            "--endpoint",
            endpoint.as_str(),
            "--format",
            "text",
            stale_message_payload.as_str(),
        ],
        "cli live s11 stale-primary send-message",
        primary_agent_name.as_str(),
    )?;
    validate_s07_replay_reason_marker(
        stale_primary_error.as_str(),
        "cli live s11 stale-primary send-message",
    )?;

    Ok(())
}

fn run_live_s12_cli_retention_deletion_probe() -> Result<(), String> {
    let cli_binary = env_var_or_default(CLI_BINARY_ENV, DEFAULT_CLI_BINARY);
    let endpoint = env_var_or_default("KAMN_ENDPOINT", "http://localhost:8080");
    let base_agent_name =
        env_var_or_default("KAMN_E2E_S12_AGENT_NAME", DEFAULT_S12_AGENT_NAME);
    let register_payload = env::var("KAMN_E2E_S12_REGISTER_CONTENT_PAYLOAD")
        .unwrap_or_else(|_| DEFAULT_S12_REGISTER_CONTENT_PAYLOAD.to_owned());

    let register_output = run_cli_command_capture_stdout_with_agent_name(
        cli_binary.as_str(),
        &[
            "register-content",
            "--endpoint",
            endpoint.as_str(),
            "--format",
            "text",
            register_payload.as_str(),
        ],
        "cli live s12 register-content",
        format!("{base_agent_name}-register").as_str(),
    )?;
    let content_id = parse_text_output_field(register_output.as_str(), "content_id")
        .ok_or_else(|| {
            format!(
                "cli live s12 register-content response missing content_id field: {register_output}"
            )
        })?
        .to_owned();
    if content_id.trim().is_empty() {
        return Err("cli live s12 register-content returned empty content_id".to_owned());
    }
    let retention_class =
        parse_text_output_field(register_output.as_str(), "retention_class").ok_or_else(|| {
            format!(
                "cli live s12 register-content response missing retention_class field: {register_output}"
            )
        })?;
    if retention_class.trim().is_empty() {
        return Err("cli live s12 register-content returned empty retention_class".to_owned());
    }

    let expire_output = run_cli_command_capture_stdout_with_agent_name(
        cli_binary.as_str(),
        &[
            "expire-content",
            "--endpoint",
            endpoint.as_str(),
            "--format",
            "text",
            content_id.as_str(),
        ],
        "cli live s12 expire-content",
        format!("{base_agent_name}-expire").as_str(),
    )?;
    let expired_content_id = parse_text_output_field(expire_output.as_str(), "content_id")
        .ok_or_else(|| {
            format!(
                "cli live s12 expire-content response missing content_id field: {expire_output}"
            )
        })?;
    validate_s12_content_id_match(
        content_id.as_str(),
        expired_content_id,
        "cli live s12 expire-content",
    )?;
    let expired_lifecycle_state =
        parse_text_output_field(expire_output.as_str(), "lifecycle_state").ok_or_else(|| {
            format!(
            "cli live s12 expire-content response missing lifecycle_state field: {expire_output}"
        )
        })?;
    if expired_lifecycle_state.trim().is_empty() {
        return Err("cli live s12 expire-content returned empty lifecycle_state".to_owned());
    }

    let tombstone_output = run_cli_command_capture_stdout_with_agent_name(
        cli_binary.as_str(),
        &[
            "tombstone-content",
            "--endpoint",
            endpoint.as_str(),
            "--format",
            "text",
            content_id.as_str(),
        ],
        "cli live s12 tombstone-content",
        format!("{base_agent_name}-tombstone").as_str(),
    )?;
    let tombstoned_content_id = parse_text_output_field(tombstone_output.as_str(), "content_id")
        .ok_or_else(|| {
            format!(
                "cli live s12 tombstone-content response missing content_id field: {tombstone_output}"
            )
        })?;
    validate_s12_content_id_match(
        content_id.as_str(),
        tombstoned_content_id,
        "cli live s12 tombstone-content",
    )?;
    let tombstoned_lifecycle_state =
        parse_text_output_field(tombstone_output.as_str(), "lifecycle_state").ok_or_else(|| {
            format!(
                "cli live s12 tombstone-content response missing lifecycle_state field: {tombstone_output}"
            )
        })?;
    if tombstoned_lifecycle_state.trim().is_empty() {
        return Err("cli live s12 tombstone-content returned empty lifecycle_state".to_owned());
    }
    let tombstoned_redaction_status =
        parse_text_output_field(tombstone_output.as_str(), "redaction_status").ok_or_else(|| {
            format!(
                "cli live s12 tombstone-content response missing redaction_status field: {tombstone_output}"
            )
        })?;
    if tombstoned_redaction_status.trim().is_empty() {
        return Err("cli live s12 tombstone-content returned empty redaction_status".to_owned());
    }

    let query_output = run_cli_command_capture_stdout_with_agent_name(
        cli_binary.as_str(),
        &[
            "query-content",
            "--endpoint",
            endpoint.as_str(),
            "--format",
            "text",
            content_id.as_str(),
        ],
        "cli live s12 query-content",
        format!("{base_agent_name}-query").as_str(),
    )?;
    let queried_content_id = parse_text_output_field(query_output.as_str(), "content_id")
        .ok_or_else(|| {
            format!("cli live s12 query-content response missing content_id field: {query_output}")
        })?;
    validate_s12_content_id_match(
        content_id.as_str(),
        queried_content_id,
        "cli live s12 query-content",
    )?;
    let queried_lifecycle_state = parse_text_output_field(query_output.as_str(), "lifecycle_state")
        .ok_or_else(|| {
            format!(
                "cli live s12 query-content response missing lifecycle_state field: {query_output}"
            )
        })?;
    validate_s12_content_field_coherence(
        tombstoned_lifecycle_state,
        queried_lifecycle_state,
        "lifecycle_state",
        "cli live s12 query-content",
    )?;
    let queried_redaction_status =
        parse_text_output_field(query_output.as_str(), "redaction_status").ok_or_else(|| {
            format!(
                "cli live s12 query-content response missing redaction_status field: {query_output}"
            )
        })?;
    validate_s12_content_field_coherence(
        tombstoned_redaction_status,
        queried_redaction_status,
        "redaction_status",
        "cli live s12 query-content",
    )?;

    Ok(())
}

fn run_live_s13_cli_bridge_forwarding_probe() -> Result<(), String> {
    let cli_binary = env_var_or_default(CLI_BINARY_ENV, DEFAULT_CLI_BINARY);
    let endpoint = env_var_or_default("KAMN_ENDPOINT", "http://localhost:8080");
    let base_agent_name =
        env_var_or_default("KAMN_E2E_S13_AGENT_NAME", DEFAULT_S13_AGENT_NAME);
    let submit_payload = env::var("KAMN_E2E_S13_SUBMIT_BRIDGE_PAYLOAD")
        .unwrap_or_else(|_| DEFAULT_S13_SUBMIT_BRIDGE_PAYLOAD.to_owned());

    let submit_output = run_cli_command_capture_stdout_with_agent_name(
        cli_binary.as_str(),
        &[
            "submit-bridge-message",
            "--endpoint",
            endpoint.as_str(),
            "--format",
            "text",
            submit_payload.as_str(),
        ],
        "cli live s13 submit-bridge-message",
        format!("{base_agent_name}-submit").as_str(),
    )?;
    let bridge_id = parse_text_output_field(submit_output.as_str(), "bridge_id")
        .ok_or_else(|| {
            format!(
                "cli live s13 submit-bridge-message response missing bridge_id field: {submit_output}"
            )
        })?
        .to_owned();
    if bridge_id.trim().is_empty() {
        return Err("cli live s13 submit-bridge-message returned empty bridge_id".to_owned());
    }
    let source_message_id =
        parse_text_output_field(submit_output.as_str(), "source_message_id").ok_or_else(|| {
            format!(
                "cli live s13 submit-bridge-message response missing source_message_id field: {submit_output}"
            )
        })?;
    if source_message_id.trim().is_empty() {
        return Err(
            "cli live s13 submit-bridge-message returned empty source_message_id".to_owned(),
        );
    }
    let submit_bridge_status =
        parse_text_output_field(submit_output.as_str(), "bridge_status").ok_or_else(|| {
            format!(
                "cli live s13 submit-bridge-message response missing bridge_status field: {submit_output}"
            )
        })?;
    if submit_bridge_status.trim().is_empty() {
        return Err("cli live s13 submit-bridge-message returned empty bridge_status".to_owned());
    }

    let forward_output = run_cli_command_capture_stdout_with_agent_name(
        cli_binary.as_str(),
        &[
            "forward-bridge-message",
            "--endpoint",
            endpoint.as_str(),
            "--format",
            "text",
            bridge_id.as_str(),
        ],
        "cli live s13 forward-bridge-message",
        format!("{base_agent_name}-forward").as_str(),
    )?;
    let forwarded_bridge_id = parse_text_output_field(forward_output.as_str(), "bridge_id")
        .ok_or_else(|| {
            format!(
                "cli live s13 forward-bridge-message response missing bridge_id field: {forward_output}"
            )
        })?;
    validate_s13_bridge_id_match(
        bridge_id.as_str(),
        forwarded_bridge_id,
        "cli live s13 forward-bridge-message",
    )?;
    let forwarded_bridge_status =
        parse_text_output_field(forward_output.as_str(), "bridge_status").ok_or_else(|| {
            format!(
                "cli live s13 forward-bridge-message response missing bridge_status field: {forward_output}"
            )
        })?;
    if forwarded_bridge_status.trim().is_empty() {
        return Err("cli live s13 forward-bridge-message returned empty bridge_status".to_owned());
    }
    let forwarded_target_message_id =
        parse_text_output_field(forward_output.as_str(), "target_message_id").ok_or_else(|| {
            format!(
                "cli live s13 forward-bridge-message response missing target_message_id field: {forward_output}"
            )
        })?;
    if forwarded_target_message_id.trim().is_empty() {
        return Err(
            "cli live s13 forward-bridge-message returned empty target_message_id".to_owned(),
        );
    }
    let forwarded_tx_hash =
        parse_text_output_field(forward_output.as_str(), "forward_tx_hash").ok_or_else(|| {
            format!(
                "cli live s13 forward-bridge-message response missing forward_tx_hash field: {forward_output}"
            )
        })?;
    if forwarded_tx_hash.trim().is_empty() {
        return Err(
            "cli live s13 forward-bridge-message returned empty forward_tx_hash".to_owned(),
        );
    }

    let query_output = run_cli_command_capture_stdout_with_agent_name(
        cli_binary.as_str(),
        &[
            "query-bridge-message",
            "--endpoint",
            endpoint.as_str(),
            "--format",
            "text",
            bridge_id.as_str(),
        ],
        "cli live s13 query-bridge-message",
        format!("{base_agent_name}-query").as_str(),
    )?;
    let queried_bridge_id = parse_text_output_field(query_output.as_str(), "bridge_id")
        .ok_or_else(|| {
            format!(
                "cli live s13 query-bridge-message response missing bridge_id field: {query_output}"
            )
        })?;
    validate_s13_bridge_id_match(
        bridge_id.as_str(),
        queried_bridge_id,
        "cli live s13 query-bridge-message",
    )?;
    let queried_bridge_status =
        parse_text_output_field(query_output.as_str(), "bridge_status").ok_or_else(|| {
            format!(
                "cli live s13 query-bridge-message response missing bridge_status field: {query_output}"
            )
        })?;
    validate_s13_bridge_field_coherence(
        forwarded_bridge_status,
        queried_bridge_status,
        "bridge_status",
        "cli live s13 query-bridge-message",
    )?;
    let queried_target_message_id =
        parse_text_output_field(query_output.as_str(), "target_message_id").ok_or_else(|| {
            format!(
                "cli live s13 query-bridge-message response missing target_message_id field: {query_output}"
            )
        })?;
    validate_s13_bridge_field_coherence(
        forwarded_target_message_id,
        queried_target_message_id,
        "target_message_id",
        "cli live s13 query-bridge-message",
    )?;
    let queried_forward_tx_hash =
        parse_text_output_field(query_output.as_str(), "forward_tx_hash").ok_or_else(|| {
            format!(
                "cli live s13 query-bridge-message response missing forward_tx_hash field: {query_output}"
            )
        })?;
    validate_s13_bridge_field_coherence(
        forwarded_tx_hash,
        queried_forward_tx_hash,
        "forward_tx_hash",
        "cli live s13 query-bridge-message",
    )?;

    Ok(())
}

fn run_live_s14_cli_batch_merkle_probe() -> Result<(), String> {
    let cli_binary = env_var_or_default(CLI_BINARY_ENV, DEFAULT_CLI_BINARY);
    let endpoint = env_var_or_default("KAMN_ENDPOINT", "http://localhost:8080");
    let base_agent_name =
        env_var_or_default("KAMN_E2E_S14_AGENT_NAME", DEFAULT_S14_AGENT_NAME);
    let batch_message_payload_a = env::var("KAMN_E2E_S14_BATCH_MESSAGE_PAYLOAD_A")
        .unwrap_or_else(|_| DEFAULT_S14_BATCH_MESSAGE_PAYLOAD_A.to_owned());
    let batch_message_payload_b = env::var("KAMN_E2E_S14_BATCH_MESSAGE_PAYLOAD_B")
        .unwrap_or_else(|_| DEFAULT_S14_BATCH_MESSAGE_PAYLOAD_B.to_owned());
    let block_height = env::var("KAMN_E2E_S14_BLOCK_HEIGHT")
        .ok()
        .map(|raw| {
            raw.trim()
                .parse::<u64>()
                .map_err(|_| format!("cli live s14 invalid block height env value: {raw}"))
        })
        .transpose()?
        .unwrap_or(DEFAULT_S14_BLOCK_HEIGHT);
    let finality =
        env_var_or_default("KAMN_E2E_S14_FINALITY", DEFAULT_S14_FINALITY);
    let block_height_value = block_height.to_string();

    let batch_a_send_output = run_cli_command_capture_stdout_with_agent_name(
        cli_binary.as_str(),
        &[
            "send-message",
            "--endpoint",
            endpoint.as_str(),
            "--format",
            "text",
            batch_message_payload_a.as_str(),
        ],
        "cli live s14 batch-a send-message",
        format!("{base_agent_name}-batch-a").as_str(),
    )?;
    let batch_a_message_id = validate_s08_message_receipt_fields(
        batch_a_send_output.as_str(),
        "cli live s14 batch-a send-message",
    )?;

    let batch_b_send_output = run_cli_command_capture_stdout_with_agent_name(
        cli_binary.as_str(),
        &[
            "send-message",
            "--endpoint",
            endpoint.as_str(),
            "--format",
            "text",
            batch_message_payload_b.as_str(),
        ],
        "cli live s14 batch-b send-message",
        format!("{base_agent_name}-batch-b").as_str(),
    )?;
    let batch_b_message_id = validate_s08_message_receipt_fields(
        batch_b_send_output.as_str(),
        "cli live s14 batch-b send-message",
    )?;
    if batch_b_message_id == batch_a_message_id {
        return Err("cli live s14 batch-b send-message returned duplicate message_id".to_owned());
    }

    let batch_a_query_output = run_cli_command_capture_stdout_with_agent_name(
        cli_binary.as_str(),
        &[
            "query-message",
            "--endpoint",
            endpoint.as_str(),
            "--format",
            "text",
            batch_a_message_id.as_str(),
        ],
        "cli live s14 batch-a query-message",
        format!("{base_agent_name}-query-a").as_str(),
    )?;
    validate_s08_query_message_response(
        batch_a_query_output.as_str(),
        batch_a_message_id.as_str(),
        "cli live s14 batch-a query-message",
    )?;

    let batch_b_query_output = run_cli_command_capture_stdout_with_agent_name(
        cli_binary.as_str(),
        &[
            "query-message",
            "--endpoint",
            endpoint.as_str(),
            "--format",
            "text",
            batch_b_message_id.as_str(),
        ],
        "cli live s14 batch-b query-message",
        format!("{base_agent_name}-query-b").as_str(),
    )?;
    validate_s08_query_message_response(
        batch_b_query_output.as_str(),
        batch_b_message_id.as_str(),
        "cli live s14 batch-b query-message",
    )?;

    let batch_root = env::var("KAMN_E2E_S14_BATCH_ROOT")
        .unwrap_or_else(|_| format!("sha256:s14:{}:{}", batch_a_message_id, batch_b_message_id));
    if batch_root.trim().is_empty() {
        return Err("cli live s14 batch-root marker must not be empty".to_owned());
    }

    let batch_a_verify_output = run_cli_command_capture_stdout_with_agent_name(
        cli_binary.as_str(),
        &[
            "verify-proof",
            "--endpoint",
            endpoint.as_str(),
            "--format",
            "text",
            batch_a_message_id.as_str(),
            batch_root.as_str(),
            block_height_value.as_str(),
            finality.as_str(),
        ],
        "cli live s14 batch-a verify-proof",
        format!("{base_agent_name}-proof-a").as_str(),
    )?;
    validate_s14_cli_verify_proof_response(
        batch_a_verify_output.as_str(),
        batch_a_message_id.as_str(),
        "cli live s14 batch-a verify-proof",
    )?;

    let batch_b_verify_output = run_cli_command_capture_stdout_with_agent_name(
        cli_binary.as_str(),
        &[
            "verify-proof",
            "--endpoint",
            endpoint.as_str(),
            "--format",
            "text",
            batch_b_message_id.as_str(),
            batch_root.as_str(),
            block_height_value.as_str(),
            finality.as_str(),
        ],
        "cli live s14 batch-b verify-proof",
        format!("{base_agent_name}-proof-b").as_str(),
    )?;
    validate_s14_cli_verify_proof_response(
        batch_b_verify_output.as_str(),
        batch_b_message_id.as_str(),
        "cli live s14 batch-b verify-proof",
    )?;

    Ok(())
}

fn run_live_s15_cli_performance_smoke_probe() -> Result<(), String> {
    let cli_binary = env_var_or_default(CLI_BINARY_ENV, DEFAULT_CLI_BINARY);
    let endpoint = env_var_or_default("KAMN_ENDPOINT", "http://localhost:8080");
    let base_agent_name =
        env_var_or_default("KAMN_E2E_S15_AGENT_NAME", DEFAULT_S15_AGENT_NAME);
    let message_payload = env::var("KAMN_E2E_S15_MESSAGE_PAYLOAD")
        .unwrap_or_else(|_| DEFAULT_S15_MESSAGE_PAYLOAD.to_owned());
    let iterations = env::var("KAMN_E2E_S15_ITERATIONS")
        .ok()
        .map(|raw| {
            raw.trim()
                .parse::<u64>()
                .map_err(|_| format!("cli live s15 invalid iterations env value: {raw}"))
        })
        .transpose()?
        .unwrap_or(DEFAULT_S15_ITERATIONS);
    if iterations == 0 {
        return Err("cli live s15 iterations must be greater than zero".to_owned());
    }

    let max_total_millis = parse_s15_budget_env_u128(
        "KAMN_E2E_S15_MAX_TOTAL_MILLIS",
        DEFAULT_S15_MAX_TOTAL_MILLIS,
        "cli live s15 max-total budget",
    )?;
    let max_p50_millis = parse_s15_budget_env_u128(
        "KAMN_E2E_S15_MAX_P50_MILLIS",
        DEFAULT_S15_MAX_P50_MILLIS,
        "cli live s15 max-p50 budget",
    )?;
    let max_p99_millis = parse_s15_budget_env_u128(
        "KAMN_E2E_S15_MAX_P99_MILLIS",
        DEFAULT_S15_MAX_P99_MILLIS,
        "cli live s15 max-p99 budget",
    )?;

    let total_start = std::time::Instant::now();
    let mut latency_samples = Vec::with_capacity(iterations as usize);
    for iteration in 0..iterations {
        let iteration_start = std::time::Instant::now();
        let send_output = run_cli_command_capture_stdout_with_agent_name(
            cli_binary.as_str(),
            &[
                "send-message",
                "--endpoint",
                endpoint.as_str(),
                "--format",
                "text",
                message_payload.as_str(),
            ],
            "cli live s15 send-message",
            format!("{base_agent_name}-send-{iteration}").as_str(),
        )?;
        let message_id =
            validate_s08_message_receipt_fields(send_output.as_str(), "cli live s15 send-message")?;

        let query_output = run_cli_command_capture_stdout_with_agent_name(
            cli_binary.as_str(),
            &[
                "query-message",
                "--endpoint",
                endpoint.as_str(),
                "--format",
                "text",
                message_id.as_str(),
            ],
            "cli live s15 query-message",
            format!("{base_agent_name}-query-{iteration}").as_str(),
        )?;
        validate_s08_query_message_response(
            query_output.as_str(),
            message_id.as_str(),
            "cli live s15 query-message",
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
        "cli live s15 performance-smoke",
    )
}

fn validate_s08_message_receipt_fields(output: &str, step: &str) -> Result<String, String> {
    let message_id = parse_text_output_field(output, "message_id")
        .ok_or_else(|| format!("{step} response missing message_id field: {output}"))?
        .to_owned();
    if message_id.trim().is_empty() {
        return Err(format!("{step} returned empty message_id"));
    }
    let status = parse_text_output_field(output, "status")
        .ok_or_else(|| format!("{step} response missing status field: {output}"))?;
    if status.trim().is_empty() {
        return Err(format!("{step} returned empty status"));
    }
    Ok(message_id)
}

fn validate_s08_query_message_response(
    output: &str,
    expected_message_id: &str,
    step: &str,
) -> Result<(), String> {
    let queried_message_id = parse_text_output_field(output, "message_id")
        .ok_or_else(|| format!("{step} response missing message_id field: {output}"))?;
    if queried_message_id != expected_message_id {
        return Err(format!(
            "{step} returned mismatched message_id: expected={expected_message_id}, got={queried_message_id}"
        ));
    }
    let queried_status = parse_text_output_field(output, "status")
        .ok_or_else(|| format!("{step} response missing status field: {output}"))?;
    if queried_status.trim().is_empty() {
        return Err(format!("{step} returned empty status"));
    }
    Ok(())
}

fn validate_s12_content_id_match(
    expected_content_id: &str,
    observed_content_id: &str,
    step: &str,
) -> Result<(), String> {
    if observed_content_id != expected_content_id {
        return Err(format!(
            "{step} returned mismatched content_id: expected={expected_content_id}, got={observed_content_id}"
        ));
    }
    Ok(())
}

fn validate_s12_content_field_coherence(
    expected_field_value: &str,
    observed_field_value: &str,
    field_name: &str,
    step: &str,
) -> Result<(), String> {
    if observed_field_value != expected_field_value {
        return Err(format!(
            "{step} {field_name} drift: expected={expected_field_value}, got={observed_field_value}"
        ));
    }
    Ok(())
}

fn validate_s13_bridge_id_match(
    expected_bridge_id: &str,
    observed_bridge_id: &str,
    step: &str,
) -> Result<(), String> {
    if observed_bridge_id != expected_bridge_id {
        return Err(format!(
            "{step} returned mismatched bridge_id: expected={expected_bridge_id}, got={observed_bridge_id}"
        ));
    }
    Ok(())
}

fn validate_s13_bridge_field_coherence(
    expected_field_value: &str,
    observed_field_value: &str,
    field_name: &str,
    step: &str,
) -> Result<(), String> {
    if observed_field_value != expected_field_value {
        return Err(format!(
            "{step} {field_name} drift: expected={expected_field_value}, got={observed_field_value}"
        ));
    }
    Ok(())
}

fn validate_s14_cli_verify_proof_response(
    output: &str,
    expected_message_id: &str,
    step: &str,
) -> Result<(), String> {
    let observed_message_id = parse_text_output_field(output, "message_id")
        .ok_or_else(|| format!("{step} response missing message_id field: {output}"))?;
    if observed_message_id != expected_message_id {
        return Err(format!(
            "{step} returned mismatched message_id: expected={expected_message_id}, got={observed_message_id}"
        ));
    }

    let observed_verified = parse_text_output_field(output, "verified")
        .ok_or_else(|| format!("{step} response missing verified field: {output}"))?;
    if observed_verified != "true" {
        return Err(format!("{step} returned verified={observed_verified}"));
    }

    let observed_finality = parse_text_output_field(output, "finality")
        .ok_or_else(|| format!("{step} response missing finality field: {output}"))?;
    if observed_finality != "FINAL" {
        return Err(format!(
            "{step} returned non-final finality: {observed_finality}"
        ));
    }

    let observed_block_height = parse_text_output_field(output, "block_height")
        .ok_or_else(|| format!("{step} response missing block_height field: {output}"))?;
    let parsed_block_height = observed_block_height
        .parse::<u64>()
        .map_err(|_| format!("{step} returned invalid block_height: {output}"))?;
    if parsed_block_height == 0 {
        return Err(format!("{step} returned block_height=0"));
    }

    Ok(())
}

fn parse_s15_budget_env_u128(
    env_key: &str,
    default_value: u128,
    step: &str,
) -> Result<u128, String> {
    let parsed = env::var(env_key)
        .ok()
        .map(|raw| {
            raw.trim()
                .parse::<u128>()
                .map_err(|_| format!("{step} invalid env value for {env_key}: {raw}"))
        })
        .transpose()?
        .unwrap_or(default_value);
    if parsed == 0 {
        return Err(format!("{step} must be greater than zero for {env_key}"));
    }
    Ok(parsed)
}

fn validate_s15_latency_budget_samples(
    samples_millis: &[u128],
    total_elapsed_millis: u128,
    max_total_millis: u128,
    max_p50_millis: u128,
    max_p99_millis: u128,
    step: &str,
) -> Result<(), String> {
    if samples_millis.is_empty() {
        return Err(format!("{step} produced zero latency samples"));
    }

    let mut sorted = samples_millis.to_vec();
    sorted.sort_unstable();
    let p50_index = percentile_index(sorted.len(), 50);
    let p99_index = percentile_index(sorted.len(), 99);
    let p50 = sorted[p50_index];
    let p99 = sorted[p99_index];

    if total_elapsed_millis > max_total_millis {
        return Err(format!(
            "{step} total elapsed millis exceeded budget: observed={total_elapsed_millis}, max={max_total_millis}"
        ));
    }
    if p50 > max_p50_millis {
        return Err(format!(
            "{step} p50 millis exceeded budget: observed={p50}, max={max_p50_millis}"
        ));
    }
    if p99 > max_p99_millis {
        return Err(format!(
            "{step} p99 millis exceeded budget: observed={p99}, max={max_p99_millis}"
        ));
    }
    Ok(())
}

fn percentile_index(sample_count: usize, percentile: u128) -> usize {
    let numerator = (sample_count as u128)
        .saturating_mul(percentile)
        .saturating_add(100u128.saturating_sub(1));
    let rank = numerator / 100;
    rank.saturating_sub(1).min(sample_count as u128 - 1) as usize
}

fn run_cli_command_capture_stdout(
    cli_binary: &str,
    args: &[&str],
    step: &str,
) -> Result<String, String> {
    run_cli_command_capture_stdout_with_optional_agent_name(cli_binary, args, step, None)
}

fn run_cli_command_expect_failure_with_agent_name(
    cli_binary: &str,
    args: &[&str],
    step: &str,
    agent_name: &str,
) -> Result<String, String> {
    let mut command = Command::new(cli_binary);
    command
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .env("KAMN_AGENT_NAME", agent_name);
    let output = command
        .output()
        .map_err(|error| format!("{step} failed to spawn: {error}"))?;

    if output.status.success() {
        return Err(format!("{step} unexpectedly succeeded"));
    }

    let stderr = String::from_utf8_lossy(output.stderr.as_slice())
        .trim()
        .to_owned();
    if stderr.is_empty() {
        return Err(format!("{step} failed without stderr details"));
    }
    Ok(stderr)
}

fn run_cli_command_capture_stdout_with_agent_name(
    cli_binary: &str,
    args: &[&str],
    step: &str,
    agent_name: &str,
) -> Result<String, String> {
    run_cli_command_capture_stdout_with_optional_agent_name(
        cli_binary,
        args,
        step,
        Some(agent_name),
    )
}

fn run_cli_command_capture_stdout_with_optional_agent_name(
    cli_binary: &str,
    args: &[&str],
    step: &str,
    agent_name: Option<&str>,
) -> Result<String, String> {
    let mut command = Command::new(cli_binary);
    command
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null());
    if let Some(agent_name) = agent_name {
        command.env("KAMN_AGENT_NAME", agent_name);
    }
    let output = command
        .output()
        .map_err(|error| format!("{step} failed to spawn: {error}"))?;

    if !output.status.success() {
        let exit_status = output
            .status
            .code()
            .map(|value| value.to_string())
            .unwrap_or_else(|| "signal".to_owned());
        return Err(format!("{step} failed (exit_status={exit_status})"));
    }

    let stdout = String::from_utf8_lossy(output.stdout.as_slice())
        .trim()
        .to_owned();
    if stdout.is_empty() {
        return Err(format!("{step} returned empty stdout"));
    }
    Ok(stdout)
}

fn parse_text_output_field<'a>(output: &'a str, key: &str) -> Option<&'a str> {
    output.split_whitespace().find_map(|token| {
        let (field, value) = token.split_once('=')?;
        (field == key).then_some(value)
    })
}

fn validate_s07_replay_reason_marker(replay_error: &str, step: &str) -> Result<(), String> {
    if !replay_error.contains(S07_REPLAY_REASON_MARKER) {
        return Err(format!(
            "{step} missing replay reason marker: {replay_error}"
        ));
    }
    Ok(())
}

fn live_s07_probe_agent_suffix() -> String {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_nanos().to_string())
        .unwrap_or_else(|_| "0".to_owned())
}

#[cfg(test)]
mod tests {
    use super::env;
    use super::{
        live_execution_enabled_from_env, parse_bool_flag, parse_s15_budget_env_u128,
        parse_text_output_field, percentile_index, run_cli_command_capture_stdout,
        run_live_s01_cli_health_probe, run_live_s02_cli_direct_message_probe,
        run_live_s03_cli_group_channel_probe, run_live_s04_cli_task_lifecycle_probe,
        run_live_s05_cli_escrow_settlement_probe, run_live_s06_cli_proof_verification_probe,
        run_live_s07_cli_replay_protection_probe, run_live_s08_cli_crash_recovery_probe,
        run_live_s09_cli_transport_failover_probe, run_live_s10_cli_topology_coherence_probe,
        run_live_s11_cli_signer_rotation_probe, run_live_s12_cli_retention_deletion_probe,
        run_live_s13_cli_bridge_forwarding_probe, run_live_s14_cli_batch_merkle_probe,
        run_live_s15_cli_performance_smoke_probe, validate_live_s05_release_escrow_response,
        validate_s07_replay_reason_marker, validate_s08_message_receipt_fields,
        validate_s08_query_message_response, validate_s12_content_field_coherence,
        validate_s12_content_id_match, validate_s13_bridge_field_coherence,
        validate_s13_bridge_id_match, validate_s14_cli_verify_proof_response,
        validate_s15_latency_budget_samples, CliScriptedDriver, CLI_BINARY_ENV,
        CLI_SCRIPTED_LIVE_ENV,
    };
    use std::ffi::OsString;
    use std::fs;
    use std::os::unix::fs::PermissionsExt;
    use std::path::PathBuf;
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

    fn unique_temp_script_path(stem: &str) -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time should be after unix epoch")
            .as_nanos();
        std::env::temp_dir().join(format!("{stem}-{}-{nonce}.py", std::process::id()))
    }

    fn write_executable_python_script(script_path: &PathBuf, source: &str) {
        fs::write(script_path, source).expect("script fixture should be written");
        let mut permissions = fs::metadata(script_path)
            .expect("script metadata should be readable")
            .permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(script_path, permissions).expect("script fixture should be executable");
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
        with_env_vars(&[(CLI_SCRIPTED_LIVE_ENV, Some("1"))], || {
            assert!(live_execution_enabled_from_env());
        });
        with_env_vars(&[(CLI_SCRIPTED_LIVE_ENV, Some("0"))], || {
            assert!(!live_execution_enabled_from_env());
        });
    }

    #[test]
    fn unit_run_live_s01_cli_health_probe_rejects_missing_binary() {
        with_env_vars(
            &[
                (CLI_BINARY_ENV, Some("/definitely/missing/kamn-cli")),
                ("KAMN_ENDPOINT", Some("http://localhost:8080")),
            ],
            || {
                let error =
                    run_live_s01_cli_health_probe().expect_err("missing binary should fail");
                assert!(
                    error.contains("failed to spawn"),
                    "error should reflect spawn failure: {error}",
                );
            },
        );
    }

    #[test]
    fn unit_run_live_s02_cli_direct_message_probe_rejects_missing_binary() {
        with_env_vars(
            &[
                (CLI_BINARY_ENV, Some("/definitely/missing/kamn-cli")),
                ("KAMN_ENDPOINT", Some("http://localhost:8080")),
            ],
            || {
                let error = run_live_s02_cli_direct_message_probe()
                    .expect_err("missing binary should fail");
                assert!(
                    error.contains("failed to spawn"),
                    "error should reflect spawn failure: {error}",
                );
            },
        );
    }

    #[test]
    fn unit_run_live_s03_cli_group_channel_probe_rejects_missing_binary() {
        with_env_vars(
            &[
                (CLI_BINARY_ENV, Some("/definitely/missing/kamn-cli")),
                ("KAMN_ENDPOINT", Some("http://localhost:8080")),
            ],
            || {
                let error =
                    run_live_s03_cli_group_channel_probe().expect_err("missing binary should fail");
                assert!(
                    error.contains("failed to spawn"),
                    "error should reflect spawn failure: {error}",
                );
            },
        );
    }

    #[test]
    fn unit_run_live_s03_cli_group_channel_probe_rejects_query_message_id_mismatch() {
        let script_path = unique_temp_script_path("kamn-e2e-cli-s03-query-mismatch");
        let script_source = r#"#!/usr/bin/env python3
import sys

command = sys.argv[1] if len(sys.argv) > 1 else ""
if command == "create-channel":
    sys.stdout.write("channel_id=channel-1 status=created")
elif command == "send-message":
    sys.stdout.write("message_id=message-1 status=sent")
elif command == "query-message":
    sys.stdout.write("message_id=message-2 status=sent")
elif command == "list-messages":
    sys.stdout.write("channel_id=channel-1 messages=[message-1]")
else:
    sys.stderr.write("unsupported command")
    sys.exit(2)
"#;
        write_executable_python_script(&script_path, script_source);

        with_env_vars(
            &[
                (
                    CLI_BINARY_ENV,
                    Some(
                        script_path
                            .to_str()
                            .expect("script path should be valid utf-8"),
                    ),
                ),
                ("KAMN_ENDPOINT", Some("http://localhost:8080")),
            ],
            || {
                let error = run_live_s03_cli_group_channel_probe()
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
    fn unit_run_live_s03_cli_group_channel_probe_rejects_list_channel_id_mismatch() {
        let script_path = unique_temp_script_path("kamn-e2e-cli-s03-list-mismatch");
        let script_source = r#"#!/usr/bin/env python3
import sys

command = sys.argv[1] if len(sys.argv) > 1 else ""
if command == "create-channel":
    sys.stdout.write("channel_id=channel-1 status=created")
elif command == "send-message":
    sys.stdout.write("message_id=message-1 status=sent")
elif command == "query-message":
    sys.stdout.write("message_id=message-1 status=sent")
elif command == "list-messages":
    sys.stdout.write("channel_id=channel-2 messages=[message-1]")
else:
    sys.stderr.write("unsupported command")
    sys.exit(2)
"#;
        write_executable_python_script(&script_path, script_source);

        with_env_vars(
            &[
                (
                    CLI_BINARY_ENV,
                    Some(
                        script_path
                            .to_str()
                            .expect("script path should be valid utf-8"),
                    ),
                ),
                ("KAMN_ENDPOINT", Some("http://localhost:8080")),
            ],
            || {
                let error = run_live_s03_cli_group_channel_probe()
                    .expect_err("mismatched listed channel_id should fail");
                assert!(
                    error.contains("mismatched channel_id"),
                    "error should mention channel_id mismatch: {error}",
                );
            },
        );

        fs::remove_file(&script_path).expect("script fixture should be removable");
    }

    #[test]
    fn unit_run_live_s04_cli_task_lifecycle_probe_rejects_missing_binary() {
        with_env_vars(
            &[
                (CLI_BINARY_ENV, Some("/definitely/missing/kamn-cli")),
                ("KAMN_ENDPOINT", Some("http://localhost:8080")),
            ],
            || {
                let error = run_live_s04_cli_task_lifecycle_probe()
                    .expect_err("missing binary should fail");
                assert!(
                    error.contains("failed to spawn"),
                    "error should reflect spawn failure: {error}",
                );
            },
        );
    }

    #[test]
    fn unit_run_live_s05_cli_escrow_settlement_probe_rejects_missing_binary() {
        with_env_vars(
            &[
                (CLI_BINARY_ENV, Some("/definitely/missing/kamn-cli")),
                ("KAMN_ENDPOINT", Some("http://localhost:8080")),
            ],
            || {
                let error = run_live_s05_cli_escrow_settlement_probe()
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
    fn unit_run_live_s06_cli_proof_verification_probe_rejects_missing_binary() {
        with_env_vars(
            &[
                (CLI_BINARY_ENV, Some("/definitely/missing/kamn-cli")),
                ("KAMN_ENDPOINT", Some("http://localhost:8080")),
            ],
            || {
                let error = run_live_s06_cli_proof_verification_probe()
                    .expect_err("missing binary should fail");
                assert!(
                    error.contains("failed to spawn"),
                    "error should reflect spawn failure: {error}",
                );
            },
        );
    }

    #[test]
    fn unit_run_live_s07_cli_replay_protection_probe_rejects_missing_binary() {
        with_env_vars(
            &[
                (CLI_BINARY_ENV, Some("/definitely/missing/kamn-cli")),
                ("KAMN_ENDPOINT", Some("http://localhost:8080")),
            ],
            || {
                let error = run_live_s07_cli_replay_protection_probe()
                    .expect_err("missing binary should fail");
                assert!(
                    error.contains("failed to spawn"),
                    "error should reflect spawn failure: {error}",
                );
            },
        );
    }

    #[test]
    fn unit_run_live_s08_cli_crash_recovery_probe_rejects_missing_binary() {
        with_env_vars(
            &[
                (CLI_BINARY_ENV, Some("/definitely/missing/kamn-cli")),
                ("KAMN_ENDPOINT", Some("http://localhost:8080")),
            ],
            || {
                let error = run_live_s08_cli_crash_recovery_probe()
                    .expect_err("missing binary should fail");
                assert!(
                    error.contains("failed to spawn"),
                    "error should reflect spawn failure: {error}",
                );
            },
        );
    }

    #[test]
    fn unit_run_live_s08_cli_crash_recovery_probe_accepts_distinct_pre_post_message_ids() {
        let script_path = unique_temp_script_path("kamn-e2e-cli-s08-success");
        let script_source = r#"#!/usr/bin/env python3
import os
import sys

command = sys.argv[1] if len(sys.argv) > 1 else ""
agent_name = os.environ.get("KAMN_AGENT_NAME", "")

if command == "send-message":
    if agent_name.endswith("pre-send"):
        sys.stdout.write("message_id=message-pre status=sent")
    elif agent_name.endswith("post-send"):
        sys.stdout.write("message_id=message-post status=sent")
    else:
        sys.stdout.write("message_id=message-fallback status=sent")
elif command == "query-message":
    message_id = sys.argv[-1] if len(sys.argv) > 0 else "message-fallback"
    sys.stdout.write(f"message_id={message_id} status=sent")
elif command == "health":
    sys.stdout.write("status=ok")
else:
    sys.stderr.write("unsupported command")
    sys.exit(2)
"#;
        write_executable_python_script(&script_path, script_source);

        with_env_vars(
            &[
                (
                    CLI_BINARY_ENV,
                    Some(
                        script_path
                            .to_str()
                            .expect("script path should be valid utf-8"),
                    ),
                ),
                ("KAMN_ENDPOINT", Some("http://localhost:8080")),
            ],
            || {
                run_live_s08_cli_crash_recovery_probe()
                    .expect("distinct pre/post message IDs should pass continuity checks");
            },
        );

        fs::remove_file(&script_path).expect("script fixture should be removable");
    }

    #[test]
    fn unit_run_live_s09_cli_transport_failover_probe_rejects_missing_binary() {
        with_env_vars(
            &[
                (CLI_BINARY_ENV, Some("/definitely/missing/kamn-cli")),
                ("KAMN_ENDPOINT", Some("http://localhost:8080")),
                (
                    "KAMN_E2E_S09_FAILOVER_ENDPOINT",
                    Some("http://localhost:8081"),
                ),
            ],
            || {
                let error = run_live_s09_cli_transport_failover_probe()
                    .expect_err("missing binary should fail");
                assert!(
                    error.contains("failed to spawn"),
                    "error should reflect spawn failure: {error}",
                );
            },
        );
    }

    #[test]
    fn unit_run_live_s09_cli_transport_failover_probe_accepts_distinct_pre_post_message_ids() {
        let script_path = unique_temp_script_path("kamn-e2e-cli-s09-success");
        let script_source = r#"#!/usr/bin/env python3
import os
import sys

command = sys.argv[1] if len(sys.argv) > 1 else ""
agent_name = os.environ.get("KAMN_AGENT_NAME", "")

if command == "send-message":
    if agent_name.endswith("pre-send"):
        sys.stdout.write("message_id=message-pre status=sent")
    elif agent_name.endswith("post-send"):
        sys.stdout.write("message_id=message-post status=sent")
    else:
        sys.stdout.write("message_id=message-fallback status=sent")
elif command == "query-message":
    message_id = sys.argv[-1] if len(sys.argv) > 0 else "message-fallback"
    sys.stdout.write(f"message_id={message_id} status=sent")
elif command == "health":
    sys.stdout.write("status=ok")
else:
    sys.stderr.write("unsupported command")
    sys.exit(2)
"#;
        write_executable_python_script(&script_path, script_source);

        with_env_vars(
            &[
                (
                    CLI_BINARY_ENV,
                    Some(
                        script_path
                            .to_str()
                            .expect("script path should be valid utf-8"),
                    ),
                ),
                ("KAMN_ENDPOINT", Some("http://localhost:8080")),
                (
                    "KAMN_E2E_S09_FAILOVER_ENDPOINT",
                    Some("http://localhost:8081"),
                ),
            ],
            || {
                run_live_s09_cli_transport_failover_probe()
                    .expect("distinct pre/post message IDs should pass continuity checks");
            },
        );

        fs::remove_file(&script_path).expect("script fixture should be removable");
    }

    #[test]
    fn unit_run_live_s10_cli_topology_coherence_probe_rejects_missing_binary() {
        with_env_vars(
            &[
                (CLI_BINARY_ENV, Some("/definitely/missing/kamn-cli")),
                (
                    "KAMN_E2E_S10_PRIMARY_ENDPOINT",
                    Some("http://localhost:8080"),
                ),
                (
                    "KAMN_E2E_S10_SECONDARY_ENDPOINT",
                    Some("http://localhost:8081"),
                ),
                (
                    "KAMN_E2E_S10_TERTIARY_ENDPOINT",
                    Some("http://localhost:8082"),
                ),
            ],
            || {
                let error = run_live_s10_cli_topology_coherence_probe()
                    .expect_err("missing binary should fail");
                assert!(
                    error.contains("failed to spawn"),
                    "error should reflect spawn failure: {error}",
                );
            },
        );
    }

    #[test]
    fn unit_run_live_s10_cli_topology_coherence_probe_accepts_topology_query_continuity() {
        let script_path = unique_temp_script_path("kamn-e2e-cli-s10-success");
        let script_source = r#"#!/usr/bin/env python3
import os
import sys

command = sys.argv[1] if len(sys.argv) > 1 else ""

if command == "send-message":
    sys.stdout.write("message_id=message-primary status=sent")
elif command == "query-message":
    message_id = sys.argv[-1] if len(sys.argv) > 0 else "message-primary"
    sys.stdout.write(f"message_id={message_id} status=sent")
elif command == "health":
    sys.stdout.write("status=ok")
else:
    sys.stderr.write("unsupported command")
    sys.exit(2)
"#;
        write_executable_python_script(&script_path, script_source);

        with_env_vars(
            &[
                (
                    CLI_BINARY_ENV,
                    Some(
                        script_path
                            .to_str()
                            .expect("script path should be valid utf-8"),
                    ),
                ),
                (
                    "KAMN_E2E_S10_PRIMARY_ENDPOINT",
                    Some("http://localhost:8080"),
                ),
                (
                    "KAMN_E2E_S10_SECONDARY_ENDPOINT",
                    Some("http://localhost:8081"),
                ),
                (
                    "KAMN_E2E_S10_TERTIARY_ENDPOINT",
                    Some("http://localhost:8082"),
                ),
            ],
            || {
                run_live_s10_cli_topology_coherence_probe()
                    .expect("topology query continuity should pass");
            },
        );

        fs::remove_file(&script_path).expect("script fixture should be removable");
    }

    #[test]
    fn unit_run_live_s11_cli_signer_rotation_probe_rejects_missing_binary() {
        with_env_vars(
            &[
                (CLI_BINARY_ENV, Some("/definitely/missing/kamn-cli")),
                ("KAMN_ENDPOINT", Some("http://localhost:8080")),
            ],
            || {
                let error = run_live_s11_cli_signer_rotation_probe()
                    .expect_err("missing binary should fail");
                assert!(
                    error.contains("failed to spawn"),
                    "error should reflect spawn failure: {error}",
                );
            },
        );
    }

    #[test]
    fn unit_run_live_s12_cli_retention_deletion_probe_rejects_missing_binary() {
        with_env_vars(
            &[
                (CLI_BINARY_ENV, Some("/definitely/missing/kamn-cli")),
                ("KAMN_ENDPOINT", Some("http://localhost:8080")),
            ],
            || {
                let error = run_live_s12_cli_retention_deletion_probe()
                    .expect_err("missing binary should fail");
                assert!(
                    error.contains("failed to spawn"),
                    "error should reflect spawn failure: {error}",
                );
            },
        );
    }

    #[test]
    fn unit_run_live_s13_cli_bridge_forwarding_probe_rejects_missing_binary() {
        with_env_vars(
            &[
                (CLI_BINARY_ENV, Some("/definitely/missing/kamn-cli")),
                ("KAMN_ENDPOINT", Some("http://localhost:8080")),
            ],
            || {
                let error = run_live_s13_cli_bridge_forwarding_probe()
                    .expect_err("missing binary should fail");
                assert!(
                    error.contains("failed to spawn"),
                    "error should reflect spawn failure: {error}",
                );
            },
        );
    }

    #[test]
    fn unit_run_live_s14_cli_batch_merkle_probe_rejects_missing_binary() {
        with_env_vars(
            &[
                (CLI_BINARY_ENV, Some("/definitely/missing/kamn-cli")),
                ("KAMN_ENDPOINT", Some("http://localhost:8080")),
            ],
            || {
                let error =
                    run_live_s14_cli_batch_merkle_probe().expect_err("missing binary should fail");
                assert!(
                    error.contains("failed to spawn"),
                    "error should reflect spawn failure: {error}",
                );
            },
        );
    }

    #[test]
    fn unit_run_live_s15_cli_performance_smoke_probe_rejects_missing_binary() {
        with_env_vars(
            &[
                (CLI_BINARY_ENV, Some("/definitely/missing/kamn-cli")),
                ("KAMN_ENDPOINT", Some("http://localhost:8080")),
            ],
            || {
                let error = run_live_s15_cli_performance_smoke_probe()
                    .expect_err("missing binary should fail");
                assert!(
                    error.contains("failed to spawn"),
                    "error should reflect spawn failure: {error}",
                );
            },
        );
    }

    #[test]
    fn unit_run_live_s14_cli_batch_merkle_probe_accepts_distinct_batch_ids_and_final_proofs() {
        let script_path = unique_temp_script_path("kamn-e2e-cli-s14-success");
        let script_source = r#"#!/usr/bin/env python3
import os
import sys

command = sys.argv[1] if len(sys.argv) > 1 else ""
agent_name = os.environ.get("KAMN_AGENT_NAME", "")

if command == "send-message":
    if agent_name.endswith("-batch-a"):
        sys.stdout.write("message_id=message-batch-a status=sent")
    elif agent_name.endswith("-batch-b"):
        sys.stdout.write("message_id=message-batch-b status=sent")
    else:
        sys.stdout.write("message_id=message-fallback status=sent")
elif command == "query-message":
    message_id = sys.argv[-1] if len(sys.argv) > 0 else "message-fallback"
    sys.stdout.write(f"message_id={message_id} status=sent")
elif command == "verify-proof":
    message_id = sys.argv[-4] if len(sys.argv) >= 4 else "message-fallback"
    block_height = sys.argv[-2] if len(sys.argv) >= 2 else "1"
    sys.stdout.write(
        f"message_id={message_id} verified=true finality=FINAL block_height={block_height}"
    )
else:
    sys.stderr.write("unsupported command")
    sys.exit(2)
"#;
        write_executable_python_script(&script_path, script_source);

        with_env_vars(
            &[
                (
                    CLI_BINARY_ENV,
                    Some(
                        script_path
                            .to_str()
                            .expect("script path should be valid utf-8"),
                    ),
                ),
                ("KAMN_ENDPOINT", Some("http://localhost:8080")),
                ("KAMN_E2E_S14_AGENT_NAME", Some("kamn-e2e-cli-s14")),
            ],
            || {
                run_live_s14_cli_batch_merkle_probe()
                    .expect("distinct batch IDs with final proofs should pass");
            },
        );

        fs::remove_file(&script_path).expect("script fixture should be removable");
    }

    #[test]
    fn unit_validate_s14_cli_verify_proof_response_accepts_valid_payload() {
        validate_s14_cli_verify_proof_response(
            "message_id=message-1 verified=true finality=FINAL block_height=42",
            "message-1",
            "test helper",
        )
        .expect("valid S-14 proof payload should pass");
    }

    #[test]
    fn unit_validate_s14_cli_verify_proof_response_rejects_mismatched_message_id() {
        let error = validate_s14_cli_verify_proof_response(
            "message_id=message-2 verified=true finality=FINAL block_height=42",
            "message-1",
            "test helper",
        )
        .expect_err("mismatched message_id should fail");
        assert!(
            error.contains("mismatched message_id"),
            "error should mention message_id mismatch: {error}",
        );
    }

    #[test]
    fn unit_validate_s14_cli_verify_proof_response_rejects_unverified_payload() {
        let error = validate_s14_cli_verify_proof_response(
            "message_id=message-1 verified=false finality=FINAL block_height=42",
            "message-1",
            "test helper",
        )
        .expect_err("verified=false should fail");
        assert!(
            error.contains("verified=false"),
            "error should mention verified contract: {error}",
        );
    }

    #[test]
    fn unit_validate_s14_cli_verify_proof_response_rejects_non_final_finality() {
        let error = validate_s14_cli_verify_proof_response(
            "message_id=message-1 verified=true finality=PENDING block_height=42",
            "message-1",
            "test helper",
        )
        .expect_err("non-final finality should fail");
        assert!(
            error.contains("non-final finality"),
            "error should mention finality contract: {error}",
        );
    }

    #[test]
    fn unit_validate_s14_cli_verify_proof_response_rejects_zero_block_height() {
        let error = validate_s14_cli_verify_proof_response(
            "message_id=message-1 verified=true finality=FINAL block_height=0",
            "message-1",
            "test helper",
        )
        .expect_err("block_height=0 should fail");
        assert!(
            error.contains("block_height=0"),
            "error should mention block-height contract: {error}",
        );
    }

    #[test]
    fn unit_validate_s15_latency_budget_samples_accepts_within_budget_samples() {
        validate_s15_latency_budget_samples(&[10, 20, 30], 80, 100, 25, 35, "test helper")
            .expect("within-budget samples should pass");
    }

    #[test]
    fn unit_validate_s15_latency_budget_samples_rejects_total_budget_violation() {
        let error =
            validate_s15_latency_budget_samples(&[10, 20, 30], 120, 100, 25, 35, "test helper")
                .expect_err("total budget violation should fail");
        assert!(
            error.contains("total elapsed millis exceeded budget"),
            "error should mention total budget: {error}",
        );
    }

    #[test]
    fn unit_validate_s15_latency_budget_samples_rejects_p50_budget_violation() {
        let error =
            validate_s15_latency_budget_samples(&[10, 50, 90], 90, 200, 20, 100, "test helper")
                .expect_err("p50 budget violation should fail");
        assert!(
            error.contains("p50 millis exceeded budget"),
            "error should mention p50 budget: {error}",
        );
    }

    #[test]
    fn unit_validate_s15_latency_budget_samples_rejects_p99_budget_violation() {
        let error =
            validate_s15_latency_budget_samples(&[10, 20, 90], 90, 200, 50, 80, "test helper")
                .expect_err("p99 budget violation should fail");
        assert!(
            error.contains("p99 millis exceeded budget"),
            "error should mention p99 budget: {error}",
        );
    }

    #[test]
    fn unit_parse_s15_budget_env_u128_uses_default_when_env_missing() {
        with_env_vars(&[("KAMN_E2E_S15_TOTAL_BUDGET_MS", None)], || {
            let parsed = parse_s15_budget_env_u128(
                "KAMN_E2E_S15_TOTAL_BUDGET_MS",
                91,
                "cli-scripted live s15 test helper",
            )
            .expect("missing env key should use default");
            assert_eq!(parsed, 91);
        });
    }

    #[test]
    fn unit_parse_s15_budget_env_u128_parses_positive_env_value() {
        with_env_vars(&[("KAMN_E2E_S15_TOTAL_BUDGET_MS", Some("143"))], || {
            let parsed = parse_s15_budget_env_u128(
                "KAMN_E2E_S15_TOTAL_BUDGET_MS",
                91,
                "cli-scripted live s15 test helper",
            )
            .expect("valid env key should parse");
            assert_eq!(parsed, 143);
        });
    }

    #[test]
    fn unit_validate_s15_latency_budget_samples_accepts_exact_budget_boundaries() {
        validate_s15_latency_budget_samples(&[10, 20, 30], 60, 60, 20, 30, "test helper")
            .expect("equal total/p50/p99 budget boundaries should pass");
    }

    #[test]
    fn unit_percentile_index_returns_expected_midpoint_index() {
        assert_eq!(
            percentile_index(3, 50),
            1,
            "len=3 and p50 should map to middle sample index",
        );
    }

    #[test]
    fn unit_percentile_index_clamps_percentile_above_hundred_to_last_index() {
        assert_eq!(
            percentile_index(3, 150),
            2,
            "percentiles above 100 should clamp to the last sample index",
        );
    }

    #[test]
    fn unit_validate_s12_content_id_match_rejects_mismatch() {
        let error = validate_s12_content_id_match("content-a", "content-b", "test step")
            .expect_err("mismatched content ids should fail");
        assert!(
            error.contains("mismatched content_id"),
            "error should mention content_id mismatch: {error}",
        );
    }

    #[test]
    fn unit_validate_s12_content_field_coherence_rejects_drift() {
        let error = validate_s12_content_field_coherence(
            "tombstoned",
            "expired",
            "lifecycle_state",
            "test step",
        )
        .expect_err("field drift should fail");
        assert!(
            error.contains("lifecycle_state drift"),
            "error should mention field drift: {error}",
        );
    }

    #[test]
    fn unit_validate_s13_bridge_id_match_rejects_mismatch() {
        let error = validate_s13_bridge_id_match("bridge-a", "bridge-b", "test step")
            .expect_err("mismatched bridge ids should fail");
        assert!(
            error.contains("mismatched bridge_id"),
            "error should mention bridge_id mismatch: {error}",
        );
    }

    #[test]
    fn unit_validate_s13_bridge_field_coherence_rejects_drift() {
        let error =
            validate_s13_bridge_field_coherence("forwarded", "stale", "bridge_status", "test step")
                .expect_err("bridge field drift should fail");
        assert!(
            error.contains("bridge_status drift"),
            "error should mention field drift: {error}",
        );
    }

    #[test]
    fn unit_run_live_s11_cli_signer_rotation_probe_accepts_rotation_continuity() {
        let script_path = unique_temp_script_path("kamn-e2e-cli-s11-success");
        let script_source = r#"#!/usr/bin/env python3
import os
import sys

command = sys.argv[1] if len(sys.argv) > 1 else ""
agent_name = os.environ.get("KAMN_AGENT_NAME", "")
payload = sys.argv[-1] if len(sys.argv) > 0 else ""
primary_agent_name = os.environ.get("KAMN_E2E_S11_PRIMARY_AGENT_NAME", "kamn-e2e-cli-s11-primary")
rotated_agent_name = os.environ.get("KAMN_E2E_S11_ROTATED_AGENT_NAME", f"{primary_agent_name}-rotated")
stale_payload = os.environ.get("KAMN_E2E_S11_STALE_MESSAGE_PAYLOAD", "{\"message\":\"cli-scripted-live-s11-stale\"}")

if command == "send-message":
    if agent_name == primary_agent_name and payload == stale_payload:
        sys.stderr.write("service_api_auth_replay_nonce_detected")
        sys.exit(1)
    if agent_name == primary_agent_name:
        sys.stdout.write("message_id=message-primary status=sent")
    elif agent_name == rotated_agent_name:
        sys.stdout.write("message_id=message-rotated status=sent")
    else:
        sys.stdout.write("message_id=message-fallback status=sent")
elif command == "query-message":
    message_id = sys.argv[-1] if len(sys.argv) > 0 else "message-fallback"
    sys.stdout.write(f"message_id={message_id} status=sent")
else:
    sys.stderr.write("unsupported command")
    sys.exit(2)
"#;
        write_executable_python_script(&script_path, script_source);

        with_env_vars(
            &[
                (
                    CLI_BINARY_ENV,
                    Some(
                        script_path
                            .to_str()
                            .expect("script path should be valid utf-8"),
                    ),
                ),
                ("KAMN_ENDPOINT", Some("http://localhost:8080")),
                (
                    "KAMN_E2E_S11_PRIMARY_AGENT_NAME",
                    Some("kamn-e2e-cli-s11-primary"),
                ),
                (
                    "KAMN_E2E_S11_ROTATED_AGENT_NAME",
                    Some("kamn-e2e-cli-s11-rotated"),
                ),
            ],
            || {
                run_live_s11_cli_signer_rotation_probe()
                    .expect("signer-rotation continuity should pass");
            },
        );

        fs::remove_file(&script_path).expect("script fixture should be removable");
    }

    #[test]
    fn unit_run_live_s06_cli_proof_verification_probe_accepts_success_payload() {
        let script_path = unique_temp_script_path("kamn-e2e-cli-s06-success");
        let script_source = format!(
            r#"#!/usr/bin/env python3
import sys
sys.stdout.write({payload:?})
"#,
            payload = "message_id=s06-live-proof block_height=1 finality=FINAL verified=true"
        );
        write_executable_python_script(&script_path, script_source.as_str());

        with_env_vars(
            &[
                (
                    CLI_BINARY_ENV,
                    Some(
                        script_path
                            .to_str()
                            .expect("script path should be valid utf-8"),
                    ),
                ),
                ("KAMN_ENDPOINT", Some("http://localhost:8080")),
            ],
            || {
                run_live_s06_cli_proof_verification_probe()
                    .expect("success payload should pass verification probe");
            },
        );

        fs::remove_file(&script_path).expect("script fixture should be removable");
    }

    #[test]
    fn unit_run_cli_command_capture_stdout_returns_trimmed_stdout_on_success() {
        let output = run_cli_command_capture_stdout(
            "/bin/sh",
            &["-c", "printf 'task_id=task-1 state=created'"],
            "test helper",
        )
        .expect("successful command should return stdout");
        assert_eq!(output, "task_id=task-1 state=created");
    }

    #[test]
    fn unit_run_cli_command_capture_stdout_rejects_non_success_exit_status() {
        let error = run_cli_command_capture_stdout("/bin/sh", &["-c", "exit 7"], "test helper")
            .expect_err("non-success status should fail");
        assert!(
            error.contains("exit_status=7"),
            "error should include failing exit status: {error}",
        );
    }

    #[test]
    fn unit_run_cli_command_expect_failure_with_agent_name_returns_stderr() {
        let output = super::run_cli_command_expect_failure_with_agent_name(
            "/bin/sh",
            &["-c", "echo replay >&2; exit 2"],
            "test helper",
            "probe",
        )
        .expect("stderr should be captured on expected failure");
        assert_eq!(output, "replay");
    }

    #[test]
    fn unit_run_cli_command_expect_failure_with_agent_name_rejects_success_status() {
        let error = super::run_cli_command_expect_failure_with_agent_name(
            "/bin/sh",
            &["-c", "exit 0"],
            "test helper",
            "probe",
        )
        .expect_err("success status should be rejected");
        assert!(
            error.contains("unexpectedly succeeded"),
            "error should mention unexpected success: {error}",
        );
    }

    #[test]
    fn unit_parse_text_output_field_extracts_known_keys_and_missing_is_none() {
        let output = "task_id=task-1 state=created";
        assert_eq!(parse_text_output_field(output, "task_id"), Some("task-1"));
        assert_eq!(parse_text_output_field(output, "state"), Some("created"));
        assert_eq!(parse_text_output_field(output, "missing"), None);
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
    fn unit_validate_s08_message_receipt_fields_rejects_empty_message_id() {
        let error = validate_s08_message_receipt_fields("message_id= status=sent", "test helper")
            .expect_err("empty message_id should fail");
        assert!(
            error.contains("empty message_id"),
            "error should mention message_id requirement: {error}",
        );
    }

    #[test]
    fn unit_validate_s08_query_message_response_rejects_mismatched_message_id() {
        let error = validate_s08_query_message_response(
            "message_id=message-2 status=sent",
            "message-1",
            "test helper",
        )
        .expect_err("mismatched message_id should fail");
        assert!(
            error.contains("mismatched message_id"),
            "error should mention message_id mismatch: {error}",
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
    fn unit_cli_scripted_driver_debug_includes_live_toggle_field() {
        let driver = CliScriptedDriver::with_runner(false, || Ok(()));
        let debug = format!("{driver:?}");
        assert!(debug.contains("CliScriptedDriver"));
        assert!(debug.contains("live_execution_enabled"));
    }

    #[test]
    fn spec_c00_live_disabled_driver_path_fails_closed_without_runner_invocation() {
        use std::sync::atomic::{AtomicUsize, Ordering};
        use std::sync::Arc;

        let runner_calls = Arc::new(AtomicUsize::new(0));
        let runner_calls_for_closure = Arc::clone(&runner_calls);
        let driver = CliScriptedDriver::with_runner(false, move || {
            runner_calls_for_closure.fetch_add(1, Ordering::SeqCst);
            Ok(())
        });
        let result = crate::drivers::HarnessDriver::execute(&driver, "S-01");
        assert_eq!(
            result.status, "fail",
            "live-disabled S-01 must fail closed instead of reporting pass",
        );
        assert_eq!(
            runner_calls.load(Ordering::SeqCst),
            0,
            "live runner should not execute when toggle is disabled",
        );
    }

    #[test]
    fn spec_c01_live_s04_driver_path_fails_closed_when_task_probe_errors() {
        let driver = CliScriptedDriver::with_runner(true, || {
            Err("cli-scripted live s04 task probe failed".to_owned())
        });
        let result = crate::drivers::HarnessDriver::execute(&driver, "S-04");
        assert_eq!(
            result.status, "fail",
            "live-enabled S-04 should fail closed on probe error",
        );
    }

    #[test]
    fn spec_c02_live_s06_driver_path_fails_closed_when_proof_probe_errors() {
        let driver = CliScriptedDriver::with_runner(true, || {
            Err("cli-scripted live s06 proof probe failed".to_owned())
        });
        let result = crate::drivers::HarnessDriver::execute(&driver, "S-06");
        assert_eq!(
            result.status, "fail",
            "live-enabled S-06 should fail closed on probe error",
        );
    }

    #[test]
    fn spec_c03_live_s02_driver_path_fails_closed_when_message_probe_errors() {
        let driver = CliScriptedDriver::with_runner(true, || {
            Err("cli-scripted live s02 message probe failed".to_owned())
        });
        let result = crate::drivers::HarnessDriver::execute(&driver, "S-02");
        assert_eq!(
            result.status, "fail",
            "live-enabled S-02 should fail closed on probe error",
        );
    }

    #[test]
    fn spec_c04_live_s03_driver_path_fails_closed_when_channel_probe_errors() {
        let driver = CliScriptedDriver::with_runner(true, || {
            Err("cli-scripted live s03 channel probe failed".to_owned())
        });
        let result = crate::drivers::HarnessDriver::execute(&driver, "S-03");
        assert_eq!(
            result.status, "fail",
            "live-enabled S-03 should fail closed on probe error",
        );
    }

    #[test]
    fn spec_c05_live_s05_driver_path_fails_closed_when_escrow_probe_errors() {
        let driver = CliScriptedDriver::with_runner(true, || {
            Err("cli-scripted live s05 escrow probe failed".to_owned())
        });
        let result = crate::drivers::HarnessDriver::execute(&driver, "S-05");
        assert_eq!(
            result.status, "fail",
            "live-enabled S-05 should fail closed on probe error",
        );
    }

    #[test]
    fn spec_c07_live_s07_driver_path_fails_closed_when_replay_probe_errors() {
        let driver = CliScriptedDriver::with_runner(true, || {
            Err("cli-scripted live s07 replay probe failed".to_owned())
        });
        let result = crate::drivers::HarnessDriver::execute(&driver, "S-07");
        assert_eq!(
            result.status, "fail",
            "live-enabled S-07 should fail closed on probe error",
        );
    }

    #[test]
    fn spec_c08_live_s08_driver_path_fails_closed_when_crash_recovery_probe_errors() {
        let driver = CliScriptedDriver::with_runner(true, || {
            Err("cli-scripted live s08 crash-recovery probe failed".to_owned())
        });
        let result = crate::drivers::HarnessDriver::execute(&driver, "S-08");
        assert_eq!(
            result.status, "fail",
            "live-enabled S-08 should fail closed on probe error",
        );
    }

    #[test]
    fn spec_c09_live_s09_driver_path_fails_closed_when_transport_failover_probe_errors() {
        let driver = CliScriptedDriver::with_runner(true, || {
            Err("cli-scripted live s09 transport-failover probe failed".to_owned())
        });
        let result = crate::drivers::HarnessDriver::execute(&driver, "S-09");
        assert_eq!(
            result.status, "fail",
            "live-enabled S-09 should fail closed on probe error",
        );
    }

    #[test]
    fn spec_c10_live_s10_driver_path_fails_closed_when_topology_coherence_probe_errors() {
        let driver = CliScriptedDriver::with_runner(true, || {
            Err("cli-scripted live s10 topology-coherence probe failed".to_owned())
        });
        let result = crate::drivers::HarnessDriver::execute(&driver, "S-10");
        assert_eq!(
            result.status, "fail",
            "live-enabled S-10 should fail closed on probe error",
        );
    }

    #[test]
    fn spec_c11_live_s11_driver_path_fails_closed_when_signer_rotation_probe_errors() {
        let driver = CliScriptedDriver::with_runner(true, || {
            Err("cli-scripted live s11 signer-rotation probe failed".to_owned())
        });
        let result = crate::drivers::HarnessDriver::execute(&driver, "S-11");
        assert_eq!(
            result.status, "fail",
            "live-enabled S-11 should fail closed on probe error",
        );
    }

    #[test]
    fn spec_c12_live_s12_driver_path_fails_closed_when_retention_deletion_probe_errors() {
        let driver = CliScriptedDriver::with_runner(true, || {
            Err("cli-scripted live s12 retention-deletion probe failed".to_owned())
        });
        let result = crate::drivers::HarnessDriver::execute(&driver, "S-12");
        assert_eq!(
            result.status, "fail",
            "live-enabled S-12 should fail closed on probe error",
        );
    }

    #[test]
    fn spec_c13_live_s13_driver_path_fails_closed_when_bridge_forwarding_probe_errors() {
        let driver = CliScriptedDriver::with_runner(true, || {
            Err("cli-scripted live s13 bridge-forwarding probe failed".to_owned())
        });
        let result = crate::drivers::HarnessDriver::execute(&driver, "S-13");
        assert_eq!(
            result.status, "fail",
            "live-enabled S-13 should fail closed on probe error",
        );
    }

    #[test]
    fn spec_c14_live_s14_driver_path_fails_closed_when_batch_merkle_probe_errors() {
        let driver = CliScriptedDriver::with_runner(true, || {
            Err("cli-scripted live s14 batch-merkle probe failed".to_owned())
        });
        let result = crate::drivers::HarnessDriver::execute(&driver, "S-14");
        assert_eq!(
            result.status, "fail",
            "live-enabled S-14 should fail closed on probe error",
        );
    }

    #[test]
    fn spec_c15_live_s15_driver_path_fails_closed_when_performance_smoke_probe_errors() {
        let driver = CliScriptedDriver::with_runner(true, || {
            Err("cli-scripted live s15 performance-smoke probe failed".to_owned())
        });
        let result = crate::drivers::HarnessDriver::execute(&driver, "S-15");
        assert_eq!(
            result.status, "fail",
            "live-enabled S-15 should fail closed on probe error",
        );
    }
}
