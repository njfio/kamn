use crate::drivers::{DriverExecutionResult, HarnessDriver};
use crate::ExecutionMode;
use kamn_agent_lib::KamnAgentHandle;
use std::sync::Arc;

const SDK_DIRECT_LIVE_ENV: &str = "KAMN_E2E_SDK_DIRECT_LIVE";
const DEFAULT_KOLME_ENDPOINT: &str = "http://localhost:3000";
const DEFAULT_AGENT_NAME: &str = "kamn-e2e-sdk-direct";

type DiscoveryProbe = dyn Fn() -> Result<(), String> + Send + Sync + 'static;

/// SDK-direct driver with optional live execution for S-01.
#[derive(Clone)]
pub struct SdkDirectDriver {
    live_execution_enabled: bool,
    discovery_probe: Arc<DiscoveryProbe>,
}

impl std::fmt::Debug for SdkDirectDriver {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("SdkDirectDriver")
            .field("live_execution_enabled", &self.live_execution_enabled)
            .finish()
    }
}

impl Default for SdkDirectDriver {
    fn default() -> Self {
        Self::from_env()
    }
}

impl SdkDirectDriver {
    /// Builds SDK-direct driver from environment configuration.
    pub fn from_env() -> Self {
        Self::with_probe(
            live_execution_enabled_from_env(),
            run_live_s01_discovery_probe,
        )
    }

    /// Creates SDK-direct driver with explicit toggle and probe implementation.
    pub fn with_probe<F>(live_execution_enabled: bool, discovery_probe: F) -> Self
    where
        F: Fn() -> Result<(), String> + Send + Sync + 'static,
    {
        Self {
            live_execution_enabled,
            discovery_probe: Arc::new(discovery_probe),
        }
    }
}

impl HarnessDriver for SdkDirectDriver {
    fn mode(&self) -> ExecutionMode {
        ExecutionMode::SdkDirect
    }

    fn execute(&self, scenario_id: &'static str) -> DriverExecutionResult {
        let status = if self.live_execution_enabled && scenario_id == "S-01" {
            if (self.discovery_probe)().is_ok() {
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
    std::env::var(SDK_DIRECT_LIVE_ENV)
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

fn run_live_s01_discovery_probe() -> Result<(), String> {
    let endpoint =
        std::env::var("KAMN_ENDPOINT").unwrap_or_else(|_| "http://localhost:8080".to_owned());
    let kolme_endpoint =
        std::env::var("KAMN_KOLME_ENDPOINT").unwrap_or_else(|_| DEFAULT_KOLME_ENDPOINT.to_owned());
    let agent_name =
        std::env::var("KAMN_AGENT_NAME").unwrap_or_else(|_| DEFAULT_AGENT_NAME.to_owned());

    let handle = KamnAgentHandle::connect(
        endpoint.as_str(),
        kolme_endpoint.as_str(),
        agent_name.as_str(),
    )
    .map_err(|error| format!("sdk-direct live discovery connect failed: {error}"))?;

    let did = handle.identity().did().as_str();
    if did.trim().is_empty() {
        return Err("sdk-direct live discovery failed: empty DID".to_owned());
    }

    let health = handle
        .health()
        .map_err(|error| format!("sdk-direct live discovery health check failed: {error}"))?;
    if health.status.trim().is_empty() {
        return Err("sdk-direct live discovery failed: empty health status".to_owned());
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{
        live_execution_enabled_from_env, parse_bool_flag, run_live_s01_discovery_probe,
        SdkDirectDriver, SDK_DIRECT_LIVE_ENV,
    };
    use std::env;
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
        with_env_vars(
            &[
                (SDK_DIRECT_LIVE_ENV, Some("1")),
                ("KAMN_ENDPOINT", Some("http://localhost:8080")),
            ],
            || {
                assert!(
                    live_execution_enabled_from_env(),
                    "truthy env value should enable live SDK-direct mode",
                );
            },
        );

        with_env_vars(&[(SDK_DIRECT_LIVE_ENV, Some("0"))], || {
            assert!(
                !live_execution_enabled_from_env(),
                "falsey env value should disable live SDK-direct mode",
            );
        });
    }

    #[test]
    fn unit_run_live_s01_discovery_probe_rejects_invalid_endpoint() {
        with_env_vars(
            &[
                ("KAMN_ENDPOINT", Some("not-a-valid-endpoint")),
                ("KAMN_KOLME_ENDPOINT", Some("http://localhost:3000")),
                ("KAMN_AGENT_NAME", Some("sdk-driver-test")),
            ],
            || {
                let error =
                    run_live_s01_discovery_probe().expect_err("invalid endpoint should fail");
                assert!(
                    error.contains("connect failed"),
                    "probe error should reflect connection failure: {error}",
                );
            },
        );
    }

    #[test]
    fn unit_sdk_direct_driver_debug_includes_live_toggle_field() {
        let driver = SdkDirectDriver::with_probe(false, || Ok(()));
        let debug = format!("{driver:?}");
        assert!(
            debug.contains("SdkDirectDriver"),
            "debug output should include struct name: {debug}",
        );
        assert!(
            debug.contains("live_execution_enabled"),
            "debug output should include live toggle field: {debug}",
        );
    }
}
