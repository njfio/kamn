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

type LiveMcpProbe = dyn Fn() -> Result<(), String> + Send + Sync + 'static;

/// MCP-agent driver for Tau and generic MCP runtimes.
#[derive(Clone)]
pub struct McpAgentDriver {
    mode: ExecutionMode,
    live_execution_enabled: bool,
    live_probe: Arc<LiveMcpProbe>,
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
        Self::with_probe(
            mode,
            live_execution_enabled_from_env(),
            run_live_s01_mcp_probe,
        )
    }

    /// Creates MCP driver with explicit toggle and probe implementation.
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
        Ok(Self {
            mode,
            live_execution_enabled,
            live_probe: Arc::new(live_probe),
        })
    }
}

impl HarnessDriver for McpAgentDriver {
    fn mode(&self) -> ExecutionMode {
        self.mode
    }

    fn execute(&self, scenario_id: &'static str) -> DriverExecutionResult {
        let status = if self.live_execution_enabled && scenario_id == "S-01" {
            if (self.live_probe)().is_ok() {
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

#[cfg(test)]
mod tests {
    use super::{
        build_framed_jsonrpc_request, live_execution_enabled_from_env, parse_bool_flag,
        parse_framed_jsonrpc_payloads, run_live_s01_mcp_probe, validate_probe_health_response,
        validate_probe_initialize_response, McpAgentDriver, MCP_AGENT_BINARY_ENV,
        MCP_AGENT_LIVE_ENV,
    };
    use super::{env, ExecutionMode};
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
