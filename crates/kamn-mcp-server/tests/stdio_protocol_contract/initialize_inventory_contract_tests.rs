use kamn_mcp_server::process_stdio_input;
use serde_json::Value;

use crate::support::{frame_request, parse_framed_json, ProtocolBackend};

#[test]
fn spec_c01_mcp_initialize_framed_jsonrpc_response_contract() {
    let backend = ProtocolBackend;
    let request =
        frame_request(r#"{"jsonrpc":"2.0","id":"req-1","method":"initialize","params":{}}"#);

    let responses = process_stdio_input(&backend, request.as_str()).expect("input should parse");
    assert_eq!(responses.len(), 1, "initialize should return one response");
    assert!(
        responses[0].starts_with("Content-Length: "),
        "response should be framed"
    );

    let body = parse_framed_json(responses[0].as_str());
    assert!(body.contains(r#""jsonrpc":"2.0""#));
    assert!(body.contains(r#""id":"req-1""#));
    assert!(body.contains(r#""serverInfo""#));
}

#[test]
fn spec_c02_mcp_tools_list_framed_tool_inventory_contract() {
    let body = tools_list_body();
    assert_tools_inventory(&body);
    let response_json = parse_tools_json(&body);
    let tools = result_tools(&response_json);
    assert_query_task_schema(tools);
    assert_verify_proof_schema(tools);
}

fn tools_list_body() -> String {
    let backend = ProtocolBackend;
    let request = frame_request(r#"{"jsonrpc":"2.0","id":"req-2","method":"tools/list"}"#);
    let responses = process_stdio_input(&backend, request.as_str()).expect("input should parse");
    assert_eq!(responses.len(), 1, "tools/list should return one response");
    parse_framed_json(responses[0].as_str())
}

fn assert_tools_inventory(body: &str) {
    assert!(body.contains(r#""tools""#));
    for name in tool_names() {
        assert!(body.contains(&format!(r#""name":"{}""#, name)));
    }
}

fn tool_names() -> [&'static str; 11] {
    [
        "register",
        "verify_proof",
        "query_task",
        "query_agent_profile",
        "register_content",
        "expire_content",
        "tombstone_content",
        "query_content",
        "submit_bridge_message",
        "forward_bridge_message",
        "query_bridge_message",
    ]
}

fn parse_tools_json(body: &str) -> Value {
    serde_json::from_str(body).expect("tools/list response should parse as JSON")
}

fn result_tools(response_json: &Value) -> &[Value] {
    response_json
        .get("result")
        .and_then(|value| value.get("tools"))
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .expect("tools/list response should expose result.tools array")
}

fn assert_query_task_schema(tools: &[Value]) {
    let query_task = find_tool(tools, "query_task");
    let required = required_schema_fields(query_task, "query_task");
    assert_eq!(required.len(), 1);
    assert_eq!(required.first().and_then(Value::as_str), Some("task_id"));
}

fn assert_verify_proof_schema(tools: &[Value]) {
    let verify_proof = find_tool(tools, "verify_proof");
    let required = required_schema_fields(verify_proof, "verify_proof");
    assert_eq!(required.len(), 4);
    assert!(required
        .iter()
        .any(|field| field.as_str() == Some("block_height")));
}

fn find_tool<'a>(tools: &'a [Value], name: &str) -> &'a Value {
    tools
        .iter()
        .find(|tool| tool.get("name").and_then(Value::as_str) == Some(name))
        .unwrap_or_else(|| panic!("tools/list should expose {name} descriptor"))
}

fn required_schema_fields<'a>(tool: &'a Value, name: &str) -> &'a [Value] {
    tool.get("inputSchema")
        .and_then(|value| value.get("required"))
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .unwrap_or_else(|| panic!("{name} input schema should expose required array"))
}
