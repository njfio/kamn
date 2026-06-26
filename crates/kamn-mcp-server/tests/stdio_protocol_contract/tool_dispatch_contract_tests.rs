use kamn_mcp_server::process_stdio_input;

use crate::support::{frame_request, parse_framed_json, ProtocolBackend};

#[test]
fn spec_c03_mcp_tools_call_health_dispatch_contract() {
    let backend = ProtocolBackend;
    let request = frame_request(
        r#"{"jsonrpc":"2.0","id":"req-3","method":"tools/call","params":{"name":"health","arguments":{}}}"#,
    );

    let responses = process_stdio_input(&backend, request.as_str()).expect("input should parse");
    let body = parse_framed_json(responses[0].as_str());
    assert!(body.contains(r#""jsonrpc":"2.0""#));
    assert!(body.contains(r#""status":"ok""#));
}

#[test]
fn spec_c07_mcp_tools_call_query_task_and_profile_dispatch_contract() {
    assert_query_task_dispatch();
    assert_query_profile_dispatch();
}

#[test]
fn spec_c08_mcp_tools_call_content_lifecycle_dispatch_contract() {
    assert_register_content_dispatch();
    assert_expire_content_dispatch();
    assert_tombstone_content_dispatch();
    assert_query_content_dispatch();
    assert_submit_bridge_dispatch();
    assert_forward_bridge_dispatch();
    assert_query_bridge_dispatch();
}

fn assert_query_task_dispatch() {
    let body = dispatch_tool_call_body("req-7", "query_task", r#"{"task_id":"task-1"}"#);
    assert!(body.contains(r#""jsonrpc":"2.0""#));
    assert!(body.contains(r#""tool":"query_task""#));
    assert!(body.contains(r#""state":"submitted""#));
}

fn assert_query_profile_dispatch() {
    let body = dispatch_tool_call_body(
        "req-8",
        "query_agent_profile",
        r#"{"did":"kamn:did:agent:alice"}"#,
    );
    assert!(body.contains(r#""tool":"query_agent_profile""#));
    assert!(body.contains(r#""reputation_score":777"#));
}

fn assert_register_content_dispatch() {
    let body = dispatch_tool_call_body(
        "req-9",
        "register_content",
        r#"{"payload":"{\"content\":\"abc\"}"}"#,
    );
    assert!(body.contains(r#""tool":"register_content""#));
    assert!(body.contains(r#""retention_class":"standard""#));
}

fn assert_expire_content_dispatch() {
    let body = dispatch_tool_call_body("req-10", "expire_content", r#"{"content_id":"content-1"}"#);
    assert!(body.contains(r#""lifecycle_state":"expired""#));
}

fn assert_tombstone_content_dispatch() {
    let body = dispatch_tool_call_body(
        "req-11",
        "tombstone_content",
        r#"{"content_id":"content-1"}"#,
    );
    assert!(body.contains(r#""redaction_status":"redacted""#));
}

fn assert_query_content_dispatch() {
    let body = dispatch_tool_call_body("req-12", "query_content", r#"{"content_id":"content-1"}"#);
    assert!(body.contains(r#""tool":"query_content""#));
    assert!(body.contains(r#""lifecycle_state":"tombstoned""#));
}

fn assert_submit_bridge_dispatch() {
    let body = dispatch_tool_call_body(
        "req-13",
        "submit_bridge_message",
        r#"{"payload":"{\"source_message_id\":\"msg-1\"}"}"#,
    );
    assert!(body.contains(r#""tool":"submit_bridge_message""#));
    assert!(body.contains(r#""bridge_status":"submitted""#));
}

fn assert_forward_bridge_dispatch() {
    let body = dispatch_tool_call_body(
        "req-14",
        "forward_bridge_message",
        r#"{"bridge_id":"bridge-1"}"#,
    );
    assert!(body.contains(r#""bridge_status":"forwarded""#));
}

fn assert_query_bridge_dispatch() {
    let body = dispatch_tool_call_body(
        "req-15",
        "query_bridge_message",
        r#"{"bridge_id":"bridge-1"}"#,
    );
    assert!(body.contains(r#""tool":"query_bridge_message""#));
    assert!(body.contains(r#""forward_tx_hash":"sha256:bridge-bridge-1""#));
}

fn dispatch_tool_call_body(request_id: &str, tool_name: &str, arguments: &str) -> String {
    let backend = ProtocolBackend;
    let request = frame_request(&format!(
        r#"{{"jsonrpc":"2.0","id":"{}","method":"tools/call","params":{{"name":"{}","arguments":{}}}}}"#,
        request_id, tool_name, arguments
    ));
    let responses = process_stdio_input(&backend, request.as_str()).expect("input should parse");
    parse_framed_json(responses[0].as_str())
}
