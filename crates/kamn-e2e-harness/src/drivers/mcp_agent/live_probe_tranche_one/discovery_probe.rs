use super::super::*;
use std::io::Write;
use std::process::{Command, Stdio};

pub(crate) fn run_live_s01_mcp_probe() -> Result<(), String> {
    let binary = env_var_or_default(MCP_AGENT_BINARY_ENV, DEFAULT_MCP_AGENT_BINARY);
    let endpoint = env_var_or_default("KAMN_ENDPOINT", DEFAULT_KAMN_ENDPOINT);
    let agent_name = env_var_or_default("KAMN_AGENT_NAME", DEFAULT_MCP_AGENT_NAME);
    let key_file = env_var_or_default("KAMN_AGENT_KEY_FILE", DEFAULT_MCP_AGENT_KEY_FILE);

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
