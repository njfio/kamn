use crate::drivers::{DriverExecutionResult, HarnessDriver};
use crate::ExecutionMode;
use std::env;
use std::process::{Command, Stdio};
use std::sync::Arc;

const CLI_SCRIPTED_LIVE_ENV: &str = "KAMN_E2E_CLI_SCRIPTED_LIVE";
const CLI_BINARY_ENV: &str = "KAMN_E2E_CLI_BINARY";
const DEFAULT_CLI_BINARY: &str = "kamn-cli";

type LiveCliRunner = dyn Fn() -> Result<(), String> + Send + Sync + 'static;

/// CLI-scripted driver with optional live execution for S-01.
#[derive(Clone)]
pub struct CliScriptedDriver {
    live_execution_enabled: bool,
    live_runner: Arc<LiveCliRunner>,
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
        Self::with_runner(
            live_execution_enabled_from_env(),
            run_live_s01_cli_health_probe,
        )
    }

    /// Creates CLI-scripted driver with explicit toggle and live runner.
    pub fn with_runner<F>(live_execution_enabled: bool, live_runner: F) -> Self
    where
        F: Fn() -> Result<(), String> + Send + Sync + 'static,
    {
        Self {
            live_execution_enabled,
            live_runner: Arc::new(live_runner),
        }
    }
}

impl HarnessDriver for CliScriptedDriver {
    fn mode(&self) -> ExecutionMode {
        ExecutionMode::CliScripted
    }

    fn execute(&self, scenario_id: &'static str) -> DriverExecutionResult {
        let status = if self.live_execution_enabled && scenario_id == "S-01" {
            if (self.live_runner)().is_ok() {
                "pass"
            } else {
                "fail"
            }
        } else {
            "pass"
        };
        DriverExecutionResult {
            scenario_id,
            status,
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

#[cfg(test)]
mod tests {
    use super::env;
    use super::{
        live_execution_enabled_from_env, parse_bool_flag, run_live_s01_cli_health_probe,
        CliScriptedDriver, CLI_BINARY_ENV, CLI_SCRIPTED_LIVE_ENV,
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
    fn unit_cli_scripted_driver_debug_includes_live_toggle_field() {
        let driver = CliScriptedDriver::with_runner(false, || Ok(()));
        let debug = format!("{driver:?}");
        assert!(debug.contains("CliScriptedDriver"));
        assert!(debug.contains("live_execution_enabled"));
    }
}
