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
use std::env;
use std::process::{Command, Stdio};
use std::sync::Arc;

const CLI_SCRIPTED_LIVE_ENV: &str = "KAMN_E2E_CLI_SCRIPTED_LIVE";
const CLI_BINARY_ENV: &str = "KAMN_E2E_CLI_BINARY";
const DEFAULT_CLI_BINARY: &str = "kamn-cli";
const AGENT_LIB_DETERMINISTIC_IDENTITY_OPT_IN_ENV: &str =
    "KAMN_AGENT_LIB_ALLOW_DETERMINISTIC_IDENTITY";
const AGENT_LIB_DETERMINISTIC_IDENTITY_OPT_IN_VALUE: &str = "1";
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
const DEFAULT_S06_MESSAGE_ID: &str = "s06-live-proof";
const DEFAULT_S06_TX_HASH: &str = "sha256:s06-live-proof";
const DEFAULT_S06_BLOCK_HEIGHT: u64 = 1;
const DEFAULT_S06_FINALITY: &str = "final";

type LiveCliRunner = dyn Fn() -> Result<(), String> + Send + Sync + 'static;

#[path = "cli_scripted/live_probe_tranche_one.rs"]
mod live_probe_tranche_one;
#[path = "cli_scripted/live_probe_tranche_two.rs"]
mod live_probe_tranche_two;

use live_probe_tranche_one::{
    run_live_s01_cli_health_probe, run_live_s02_cli_direct_message_probe,
    run_live_s03_cli_group_channel_probe, run_live_s04_cli_task_lifecycle_probe,
    run_live_s05_cli_escrow_settlement_probe,
};
use live_probe_tranche_two::{
    run_live_s06_cli_proof_verification_probe, run_live_s07_cli_replay_protection_probe,
    run_live_s08_cli_crash_recovery_probe, run_live_s09_cli_transport_failover_probe,
    run_live_s10_cli_topology_coherence_probe, validate_s08_message_receipt_fields,
    validate_s08_query_message_response,
};

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

fn live_execution_enabled_from_env() -> bool {
    shared_live_execution_enabled_from_env(CLI_SCRIPTED_LIVE_ENV)
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
    let base_agent_name = env_var_or_default("KAMN_E2E_S12_AGENT_NAME", DEFAULT_S12_AGENT_NAME);
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
    let base_agent_name = env_var_or_default("KAMN_E2E_S13_AGENT_NAME", DEFAULT_S13_AGENT_NAME);
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
                .map_err(|_| format!("cli live s14 invalid block height env value: {raw}"))
        })
        .transpose()?
        .unwrap_or(DEFAULT_S14_BLOCK_HEIGHT);
    let finality = env_var_or_default("KAMN_E2E_S14_FINALITY", DEFAULT_S14_FINALITY);
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
    let base_agent_name = env_var_or_default("KAMN_E2E_S15_AGENT_NAME", DEFAULT_S15_AGENT_NAME);
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
        .env(
            AGENT_LIB_DETERMINISTIC_IDENTITY_OPT_IN_ENV,
            AGENT_LIB_DETERMINISTIC_IDENTITY_OPT_IN_VALUE,
        )
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
        .stderr(Stdio::null())
        .env(
            AGENT_LIB_DETERMINISTIC_IDENTITY_OPT_IN_ENV,
            AGENT_LIB_DETERMINISTIC_IDENTITY_OPT_IN_VALUE,
        );
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

#[cfg(test)]
#[path = "cli_scripted_tests.rs"]
mod cli_scripted_tests;
