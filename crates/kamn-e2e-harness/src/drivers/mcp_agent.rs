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
const DEFAULT_S04_CREATE_TASK_PAYLOAD: &str =
    r#"{"title":"mcp-agent-live-s04","description":"live task lifecycle probe"}"#;
const DEFAULT_S04_ESCROW_AMOUNT: u64 = 1;

type LiveMcpProbe = dyn Fn() -> Result<(), String> + Send + Sync + 'static;

/// MCP-agent driver for Tau and generic MCP runtimes.
#[derive(Clone)]
pub struct McpAgentDriver {
    mode: ExecutionMode,
    live_execution_enabled: bool,
    discovery_probe: Arc<LiveMcpProbe>,
    task_lifecycle_probe: Arc<LiveMcpProbe>,
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
            run_live_s04_mcp_task_lifecycle_probe,
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
            task_lifecycle_probe: live_probe,
        })
    }

    /// Creates MCP driver with explicit per-scenario probe implementations.
    pub fn with_probes<F, G>(
        mode: ExecutionMode,
        live_execution_enabled: bool,
        discovery_probe: F,
        task_lifecycle_probe: G,
    ) -> Result<Self, String>
    where
        F: Fn() -> Result<(), String> + Send + Sync + 'static,
        G: Fn() -> Result<(), String> + Send + Sync + 'static,
    {
        if !matches!(mode, ExecutionMode::McpTau | ExecutionMode::McpAny) {
            return Err("McpAgentDriver requires mcp-tau or mcp-any mode".to_owned());
        }
        Ok(Self {
            mode,
            live_execution_enabled,
            discovery_probe: Arc::new(discovery_probe),
            task_lifecycle_probe: Arc::new(task_lifecycle_probe),
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
            "S-04" => Some((self.task_lifecycle_probe)()),
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

    let create_arguments = format!(
        "{{\"payload\":\"{}\"}}",
        escape_json_scalar(create_task_payload.as_str())
    );
    let create_response = run_live_s04_mcp_tool_call(
        binary.as_str(),
        endpoint.as_str(),
        agent_name.as_str(),
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
        agent_name.as_str(),
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
        agent_name.as_str(),
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
        agent_name.as_str(),
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
        agent_name.as_str(),
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
        live_execution_enabled_from_env, parse_bool_flag, parse_framed_jsonrpc_payloads,
        run_live_s01_mcp_probe, run_live_s04_mcp_task_lifecycle_probe, run_live_s04_mcp_tool_call,
        validate_probe_health_response, validate_probe_initialize_response, McpAgentDriver,
        MCP_AGENT_BINARY_ENV, MCP_AGENT_LIVE_ENV,
    };
    use super::{env, ExecutionMode};
    use std::ffi::OsString;
    use std::fs;
    use std::os::unix::fs::PermissionsExt;
    use std::sync::{Mutex, OnceLock};
    use std::time::{SystemTime, UNIX_EPOCH};

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

    fn unique_temp_script_path(stem: &str) -> std::path::PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time should be after unix epoch")
            .as_nanos();
        std::env::temp_dir().join(format!("{stem}-{}-{nonce}.py", std::process::id()))
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
        let script_source = r#"#!/usr/bin/env python3
import sys
init_payload = '{"jsonrpc":"2.0","id":"probe-init","result":{"serverInfo":{"name":"kamn"}}}'
tool_payload = '{"jsonrpc":"2.0","id":"probe-request","result":{"ok":true}}'
sys.stdout.write(
    f"Content-Length: {len(init_payload)}\r\n\r\n{init_payload}"
    f"Content-Length: {len(tool_payload)}\r\n\r\n{tool_payload}"
)
"#;
        fs::write(&script_path, script_source).expect("script fixture should be written");
        let mut permissions = fs::metadata(&script_path)
            .expect("script metadata should be readable")
            .permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(&script_path, permissions)
            .expect("script fixture should be executable");

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
