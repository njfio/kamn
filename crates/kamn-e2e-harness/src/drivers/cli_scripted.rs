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
const DEFAULT_S04_AGENT_NAME: &str = "kamn-e2e-cli-s04";
const DEFAULT_S04_CREATE_TASK_PAYLOAD: &str =
    r#"{"title":"cli-scripted-live-s04","description":"live task lifecycle probe"}"#;
const DEFAULT_S04_ESCROW_AMOUNT: u64 = 1;
const DEFAULT_S06_MESSAGE_ID: &str = "s06-live-proof";
const DEFAULT_S06_TX_HASH: &str = "sha256:s06-live-proof";
const DEFAULT_S06_BLOCK_HEIGHT: u64 = 1;
const DEFAULT_S06_FINALITY: &str = "final";

type LiveCliRunner = dyn Fn() -> Result<(), String> + Send + Sync + 'static;

/// CLI-scripted driver with optional live execution for S-01, S-02, S-04, and S-06.
#[derive(Clone)]
pub struct CliScriptedDriver {
    live_execution_enabled: bool,
    discovery_runner: Arc<LiveCliRunner>,
    direct_message_runner: Arc<LiveCliRunner>,
    task_lifecycle_runner: Arc<LiveCliRunner>,
    proof_verification_runner: Arc<LiveCliRunner>,
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
            run_live_s04_cli_task_lifecycle_probe,
            run_live_s06_cli_proof_verification_probe,
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
            proof_verification_runner: live_runner,
        }
    }

    /// Creates CLI-scripted driver with explicit per-scenario live runners.
    pub fn with_runners<F, G, H, I>(
        live_execution_enabled: bool,
        discovery_runner: F,
        direct_message_runner: G,
        task_lifecycle_runner: H,
        proof_verification_runner: I,
    ) -> Self
    where
        F: Fn() -> Result<(), String> + Send + Sync + 'static,
        G: Fn() -> Result<(), String> + Send + Sync + 'static,
        H: Fn() -> Result<(), String> + Send + Sync + 'static,
        I: Fn() -> Result<(), String> + Send + Sync + 'static,
    {
        Self {
            live_execution_enabled,
            discovery_runner: Arc::new(discovery_runner),
            direct_message_runner: Arc::new(direct_message_runner),
            task_lifecycle_runner: Arc::new(task_lifecycle_runner),
            proof_verification_runner: Arc::new(proof_verification_runner),
        }
    }
}

impl HarnessDriver for CliScriptedDriver {
    fn mode(&self) -> ExecutionMode {
        ExecutionMode::CliScripted
    }

    fn execute(&self, scenario_id: &'static str) -> DriverExecutionResult {
        let status = match self.live_runner_for_scenario(scenario_id) {
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

impl CliScriptedDriver {
    fn live_runner_for_scenario(&self, scenario_id: &'static str) -> Option<Result<(), String>> {
        if !self.live_execution_enabled {
            return None;
        }
        match scenario_id {
            "S-01" => Some((self.discovery_runner)()),
            "S-02" => Some((self.direct_message_runner)()),
            "S-04" => Some((self.task_lifecycle_runner)()),
            "S-06" => Some((self.proof_verification_runner)()),
            _ => None,
        }
    }
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
    let cli_binary = env::var(CLI_BINARY_ENV).unwrap_or_else(|_| DEFAULT_CLI_BINARY.to_owned());
    let endpoint = env::var("KAMN_ENDPOINT").unwrap_or_else(|_| "http://localhost:8080".to_owned());

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
    let cli_binary = env::var(CLI_BINARY_ENV).unwrap_or_else(|_| DEFAULT_CLI_BINARY.to_owned());
    let endpoint = env::var("KAMN_ENDPOINT").unwrap_or_else(|_| "http://localhost:8080".to_owned());
    let base_agent_name =
        env::var("KAMN_AGENT_NAME").unwrap_or_else(|_| DEFAULT_S02_AGENT_NAME.to_owned());
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

fn run_live_s04_cli_task_lifecycle_probe() -> Result<(), String> {
    let cli_binary = env::var(CLI_BINARY_ENV).unwrap_or_else(|_| DEFAULT_CLI_BINARY.to_owned());
    let endpoint = env::var("KAMN_ENDPOINT").unwrap_or_else(|_| "http://localhost:8080".to_owned());
    let base_agent_name =
        env::var("KAMN_AGENT_NAME").unwrap_or_else(|_| DEFAULT_S04_AGENT_NAME.to_owned());
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

fn run_live_s06_cli_proof_verification_probe() -> Result<(), String> {
    let cli_binary = env::var(CLI_BINARY_ENV).unwrap_or_else(|_| DEFAULT_CLI_BINARY.to_owned());
    let endpoint = env::var("KAMN_ENDPOINT").unwrap_or_else(|_| "http://localhost:8080".to_owned());
    let message_id = env::var("KAMN_E2E_S06_PROOF_MESSAGE_ID")
        .unwrap_or_else(|_| DEFAULT_S06_MESSAGE_ID.to_owned());
    let tx_hash =
        env::var("KAMN_E2E_S06_PROOF_TX_HASH").unwrap_or_else(|_| DEFAULT_S06_TX_HASH.to_owned());
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
        env::var("KAMN_E2E_S06_PROOF_FINALITY").unwrap_or_else(|_| DEFAULT_S06_FINALITY.to_owned());
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

fn run_cli_command_capture_stdout(
    cli_binary: &str,
    args: &[&str],
    step: &str,
) -> Result<String, String> {
    run_cli_command_capture_stdout_with_optional_agent_name(cli_binary, args, step, None)
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

#[cfg(test)]
mod tests {
    use super::env;
    use super::{
        live_execution_enabled_from_env, parse_bool_flag, parse_text_output_field,
        run_cli_command_capture_stdout, run_live_s01_cli_health_probe,
        run_live_s02_cli_direct_message_probe, run_live_s04_cli_task_lifecycle_probe,
        run_live_s06_cli_proof_verification_probe, CliScriptedDriver, CLI_BINARY_ENV,
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
    fn unit_parse_text_output_field_extracts_known_keys_and_missing_is_none() {
        let output = "task_id=task-1 state=created";
        assert_eq!(parse_text_output_field(output, "task_id"), Some("task-1"));
        assert_eq!(parse_text_output_field(output, "state"), Some("created"));
        assert_eq!(parse_text_output_field(output, "missing"), None);
    }

    #[test]
    fn unit_cli_scripted_driver_debug_includes_live_toggle_field() {
        let driver = CliScriptedDriver::with_runner(false, || Ok(()));
        let debug = format!("{driver:?}");
        assert!(debug.contains("CliScriptedDriver"));
        assert!(debug.contains("live_execution_enabled"));
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
}
