use crate::drivers::{DriverExecutionResult, HarnessDriver};
use crate::ExecutionMode;
use std::env;
use std::process::{Command, Stdio};
use std::sync::Arc;

const CLI_SCRIPTED_LIVE_ENV: &str = "KAMN_E2E_CLI_SCRIPTED_LIVE";
const CLI_BINARY_ENV: &str = "KAMN_E2E_CLI_BINARY";
const DEFAULT_CLI_BINARY: &str = "kamn-cli";
const DEFAULT_S04_CREATE_TASK_PAYLOAD: &str =
    r#"{"title":"cli-scripted-live-s04","description":"live task lifecycle probe"}"#;
const DEFAULT_S04_ESCROW_AMOUNT: u64 = 1;

type LiveCliRunner = dyn Fn() -> Result<(), String> + Send + Sync + 'static;

/// CLI-scripted driver with optional live execution for S-01 and S-04.
#[derive(Clone)]
pub struct CliScriptedDriver {
    live_execution_enabled: bool,
    discovery_runner: Arc<LiveCliRunner>,
    task_lifecycle_runner: Arc<LiveCliRunner>,
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
            run_live_s04_cli_task_lifecycle_probe,
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
            task_lifecycle_runner: live_runner,
        }
    }

    /// Creates CLI-scripted driver with explicit per-scenario live runners.
    pub fn with_runners<F, G>(
        live_execution_enabled: bool,
        discovery_runner: F,
        task_lifecycle_runner: G,
    ) -> Self
    where
        F: Fn() -> Result<(), String> + Send + Sync + 'static,
        G: Fn() -> Result<(), String> + Send + Sync + 'static,
    {
        Self {
            live_execution_enabled,
            discovery_runner: Arc::new(discovery_runner),
            task_lifecycle_runner: Arc::new(task_lifecycle_runner),
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
            "S-04" => Some((self.task_lifecycle_runner)()),
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

fn run_live_s04_cli_task_lifecycle_probe() -> Result<(), String> {
    let cli_binary = env::var(CLI_BINARY_ENV).unwrap_or_else(|_| DEFAULT_CLI_BINARY.to_owned());
    let endpoint = env::var("KAMN_ENDPOINT").unwrap_or_else(|_| "http://localhost:8080".to_owned());
    let create_task_payload = env::var("KAMN_E2E_S04_CREATE_TASK_PAYLOAD")
        .unwrap_or_else(|_| DEFAULT_S04_CREATE_TASK_PAYLOAD.to_owned());

    let create_output = run_cli_command_capture_stdout(
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
    let fund_output = run_cli_command_capture_stdout(
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
    )?;
    let escrow_id = parse_text_output_field(fund_output.as_str(), "escrow_id")
        .ok_or_else(|| {
            format!("cli live s04 fund-escrow response missing escrow_id field: {fund_output}")
        })?
        .to_owned();
    if escrow_id.trim().is_empty() {
        return Err("cli live s04 fund-escrow returned empty escrow_id".to_owned());
    }

    let accept_output = run_cli_command_capture_stdout(
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
    )?;
    let accept_state =
        parse_text_output_field(accept_output.as_str(), "state").ok_or_else(|| {
            format!("cli live s04 accept-task response missing state field: {accept_output}")
        })?;
    if accept_state.trim().is_empty() {
        return Err("cli live s04 accept-task returned empty state".to_owned());
    }

    let complete_output = run_cli_command_capture_stdout(
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
    )?;
    let complete_state =
        parse_text_output_field(complete_output.as_str(), "state").ok_or_else(|| {
            format!("cli live s04 complete-task response missing state field: {complete_output}")
        })?;
    if complete_state.trim().is_empty() {
        return Err("cli live s04 complete-task returned empty state".to_owned());
    }

    let release_output = run_cli_command_capture_stdout(
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

fn run_cli_command_capture_stdout(
    cli_binary: &str,
    args: &[&str],
    step: &str,
) -> Result<String, String> {
    let output = Command::new(cli_binary)
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
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
        run_live_s04_cli_task_lifecycle_probe, CliScriptedDriver, CLI_BINARY_ENV,
        CLI_SCRIPTED_LIVE_ENV,
    };
    use std::ffi::OsString;
    use std::sync::{Mutex, OnceLock};

    fn env_lock() -> &'static Mutex<()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(()))
    }

    fn with_env_vars<F>(updates: &[(&str, Option<&str>)], test: F)
    where
        F: FnOnce(),
    {
        let _guard = env_lock().lock().expect("env lock");
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
}
