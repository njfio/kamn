pub(super) use crate::drivers::shared_helpers::{
    env_var_or_default, env_var_or_else, is_live_bound_scenario_id,
    live_execution_enabled_from_env as shared_live_execution_enabled_from_env,
    live_s07_probe_agent_suffix, parse_s15_budget_env_u128,
    validate_live_s05_release_escrow_response, validate_s15_latency_budget_samples,
};
use crate::drivers::{DriverExecutionResult, HarnessDriver};
use crate::ExecutionMode;
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
const DEFAULT_S06_MESSAGE_ID: &str = "s06-live-proof";
const DEFAULT_S06_TX_HASH: &str = "sha256:s06-live-proof";
const DEFAULT_S06_BLOCK_HEIGHT: u64 = 1;
const DEFAULT_S06_FINALITY: &str = "final";

type LiveCliRunner = dyn Fn() -> Result<(), String> + Send + Sync + 'static;

#[path = "cli_scripted/live_probe_tranche_one.rs"]
mod live_probe_tranche_one;
#[path = "cli_scripted/live_probe_tranche_three.rs"]
mod live_probe_tranche_three;
#[path = "cli_scripted/live_probe_tranche_two.rs"]
mod live_probe_tranche_two;

use live_probe_tranche_one::{
    run_live_s01_cli_health_probe, run_live_s02_cli_direct_message_probe,
    run_live_s03_cli_group_channel_probe, run_live_s04_cli_task_lifecycle_probe,
    run_live_s05_cli_escrow_settlement_probe,
};
use live_probe_tranche_three::{
    run_live_s11_cli_signer_rotation_probe, run_live_s12_cli_retention_deletion_probe,
    run_live_s13_cli_bridge_forwarding_probe, run_live_s14_cli_batch_merkle_probe,
    run_live_s15_cli_performance_smoke_probe,
};
use live_probe_tranche_two::{
    run_live_s06_cli_proof_verification_probe, run_live_s07_cli_replay_protection_probe,
    run_live_s08_cli_crash_recovery_probe, run_live_s09_cli_transport_failover_probe,
    run_live_s10_cli_topology_coherence_probe, validate_s08_distinct_message_ids,
    validate_s08_message_receipt_fields, validate_s08_query_message_response,
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
