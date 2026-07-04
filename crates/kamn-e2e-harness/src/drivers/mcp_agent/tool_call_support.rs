use std::io::Write;
use std::process::{Child, ChildStdin, Command, Stdio};

use super::{
    build_framed_jsonrpc_request, json_optional_bool_field, parse_framed_jsonrpc_payloads,
    validate_probe_initialize_response,
};

macro_rules! scenario_tool_call {
    ($name:ident, $label:literal) => {
        pub(crate) fn $name(
            binary: &str,
            endpoint: &str,
            agent_name: &str,
            key_file: &str,
            request_id: &str,
            tool_name: &str,
            arguments_json: &str,
        ) -> Result<String, String> {
            run_named_mcp_tool_call(
                $label,
                binary,
                endpoint,
                agent_name,
                key_file,
                request_id,
                (tool_name, arguments_json),
            )
        }
    };
}

scenario_tool_call!(run_live_s03_mcp_tool_call, "mcp live s03");
scenario_tool_call!(run_live_s04_mcp_tool_call, "mcp live s04");
scenario_tool_call!(run_live_s05_mcp_tool_call, "mcp live s05");
scenario_tool_call!(run_live_s06_mcp_tool_call, "mcp live s06");
scenario_tool_call!(run_live_s07_mcp_tool_call, "mcp live s07");
scenario_tool_call!(run_live_s08_mcp_tool_call, "mcp live s08");
scenario_tool_call!(run_live_s11_mcp_tool_call, "mcp live s11");
scenario_tool_call!(run_live_s12_mcp_tool_call, "mcp live s12");
scenario_tool_call!(run_live_s13_mcp_tool_call, "mcp live s13");
scenario_tool_call!(run_live_s14_mcp_tool_call, "mcp live s14");
#[cfg(test)]
scenario_tool_call!(run_live_s15_mcp_tool_call, "mcp live s15");

pub(crate) fn run_named_mcp_tool_call(
    step_prefix: &str,
    binary: &str,
    endpoint: &str,
    agent_name: &str,
    key_file: &str,
    request_id: &str,
    tool_call: (&str, &str),
) -> Result<String, String> {
    let (tool_name, arguments_json) = tool_call;
    let stdout = spawn_mcp_tool_call(
        step_prefix,
        binary,
        endpoint,
        agent_name,
        key_file,
        request_id,
        (tool_name, arguments_json),
    )?;
    validate_tool_call_output(step_prefix, request_id, tool_name, stdout.as_str())
}

#[rustfmt::skip]
pub(crate) fn spawn_mcp_tool_call(step_prefix: &str, binary: &str, endpoint: &str, agent_name: &str, key_file: &str, request_id: &str, tool_call: (&str, &str)) -> Result<String, String> {
    let (tool_name, arguments_json) = tool_call;
    let mut child = Command::new(binary)
        .arg("--endpoint").arg(endpoint)
        .arg("--agent-name").arg(agent_name)
        .arg("--key-file").arg(key_file)
        .stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::null())
        .spawn()
        .map_err(|error| format!("{step_prefix} {tool_name} failed to spawn: {error}"))?;
    write_request_stream(step_prefix, request_id, tool_name, arguments_json, child.stdin.take())?;
    wait_for_stdout(step_prefix, tool_name, child)
}

pub(crate) fn write_request_stream(
    step_prefix: &str,
    request_id: &str,
    tool_name: &str,
    arguments_json: &str,
    stdin: Option<ChildStdin>,
) -> Result<(), String> {
    if let Some(mut stdin) = stdin {
        let initialize_request = build_framed_jsonrpc_request(
            r#"{"jsonrpc":"2.0","id":"probe-init","method":"initialize","params":{}}"#,
        );
        let tool_request_json = format!(
            "{{\"jsonrpc\":\"2.0\",\"id\":\"{request_id}\",\"method\":\"tools/call\",\"params\":{{\"name\":\"{tool_name}\",\"arguments\":{arguments_json}}}}}"
        );
        let requests = format!(
            "{initialize_request}{}",
            build_framed_jsonrpc_request(tool_request_json.as_str())
        );
        stdin.write_all(requests.as_bytes()).map_err(|error| {
            format!("{step_prefix} {tool_name} failed to write framed request stream: {error}")
        })?;
    }
    Ok(())
}

pub(crate) fn wait_for_stdout(
    step_prefix: &str,
    tool_name: &str,
    child: Child,
) -> Result<String, String> {
    let output = child
        .wait_with_output()
        .map_err(|error| format!("{step_prefix} {tool_name} failed to read response: {error}"))?;
    if !output.status.success() {
        let exit_status = output
            .status
            .code()
            .map(|value| value.to_string())
            .unwrap_or_else(|| "signal".to_owned());
        return Err(format!(
            "{step_prefix} {tool_name} failed (exit_status={exit_status})"
        ));
    }
    Ok(String::from_utf8_lossy(output.stdout.as_slice()).into_owned())
}

pub(crate) fn validate_tool_call_output(
    step_prefix: &str,
    request_id: &str,
    tool_name: &str,
    stdout: &str,
) -> Result<String, String> {
    let payloads = parse_framed_jsonrpc_payloads(stdout)
        .map_err(|error| format!("{step_prefix} {tool_name} invalid framed output: {error}"))?;
    validate_probe_initialize_response(find_initialize_response(
        step_prefix,
        tool_name,
        &payloads,
    )?)?;
    let tool_response = find_tool_response(step_prefix, request_id, tool_name, &payloads)?;
    if !json_optional_bool_field(tool_response.as_str(), "ok").unwrap_or(false) {
        return Err(format!(
            "{step_prefix} {tool_name} returned non-success payload: {tool_response}"
        ));
    }
    Ok(tool_response.clone())
}

pub(crate) fn find_initialize_response<'a>(
    step_prefix: &str,
    tool_name: &str,
    payloads: &'a [String],
) -> Result<&'a String, String> {
    payloads
        .iter()
        .find(|payload| payload.contains(r#""id":"probe-init""#))
        .ok_or_else(|| format!("{step_prefix} {tool_name} missing initialize response payload"))
}

pub(crate) fn find_tool_response<'a>(
    step_prefix: &str,
    request_id: &str,
    tool_name: &str,
    payloads: &'a [String],
) -> Result<&'a String, String> {
    let response_id = format!(r#""id":"{request_id}""#);
    payloads
        .iter()
        .find(|payload| payload.contains(response_id.as_str()))
        .ok_or_else(|| format!("{step_prefix} {tool_name} missing tool response payload"))
}
