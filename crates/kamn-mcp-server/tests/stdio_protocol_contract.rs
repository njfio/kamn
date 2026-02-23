use kamn_agent_lib::AgentLibError;
use kamn_mcp_server::{process_stdio_input, McpToolBackend};

#[derive(Debug, Default)]
struct ProtocolBackend;

impl McpToolBackend for ProtocolBackend {
    fn register(&self) -> Result<String, AgentLibError> {
        Ok(r#"{"did":"kamn:did:agent:test"}"#.to_owned())
    }

    fn send_message(&self, payload: &str) -> Result<String, AgentLibError> {
        Ok(format!(r#"{{"message_id":"msg-{}"}}"#, payload.len()))
    }

    fn create_channel(&self, payload: &str) -> Result<String, AgentLibError> {
        Ok(format!(r#"{{"channel_id":"channel-{}"}}"#, payload.len()))
    }

    fn list_messages(&self, channel_id: &str) -> Result<String, AgentLibError> {
        Ok(format!(
            r#"{{"channel_id":"{}","messages":["msg-a","msg-b"]}}"#,
            channel_id
        ))
    }

    fn query_message(&self, message_id: &str) -> Result<String, AgentLibError> {
        Ok(format!(
            r#"{{"message_id":"{}","status":"created"}}"#,
            message_id
        ))
    }

    fn query_task(&self, task_id: &str) -> Result<String, AgentLibError> {
        Ok(format!(
            r#"{{"task_id":"{}","state":"submitted"}}"#,
            task_id
        ))
    }

    fn query_agent_profile(&self, did: &str) -> Result<String, AgentLibError> {
        Ok(format!(r#"{{"did":"{}","reputation_score":777}}"#, did))
    }

    fn register_content(&self, payload: &str) -> Result<String, AgentLibError> {
        Ok(format!(
            r#"{{"content_id":"content-{}","retention_class":"standard","lifecycle_state":"retained","redaction_status":"none"}}"#,
            payload.len()
        ))
    }

    fn expire_content(&self, content_id: &str) -> Result<String, AgentLibError> {
        Ok(format!(
            r#"{{"content_id":"{}","lifecycle_state":"expired","redaction_status":"none"}}"#,
            content_id
        ))
    }

    fn tombstone_content(&self, content_id: &str) -> Result<String, AgentLibError> {
        Ok(format!(
            r#"{{"content_id":"{}","lifecycle_state":"tombstoned","redaction_status":"redacted"}}"#,
            content_id
        ))
    }

    fn query_content(&self, content_id: &str) -> Result<String, AgentLibError> {
        Ok(format!(
            r#"{{"content_id":"{}","lifecycle_state":"tombstoned","redaction_status":"redacted"}}"#,
            content_id
        ))
    }

    fn submit_bridge_message(&self, payload: &str) -> Result<String, AgentLibError> {
        Ok(format!(
            r#"{{"bridge_id":"bridge-{}","source_message_id":"source-{}","bridge_status":"submitted"}}"#,
            payload.len(),
            payload.len()
        ))
    }

    fn forward_bridge_message(&self, bridge_id: &str) -> Result<String, AgentLibError> {
        Ok(format!(
            r#"{{"bridge_id":"{}","bridge_status":"forwarded","target_message_id":"target-{}","forward_tx_hash":"sha256:bridge-{}"}}"#,
            bridge_id, bridge_id, bridge_id
        ))
    }

    fn query_bridge_message(&self, bridge_id: &str) -> Result<String, AgentLibError> {
        Ok(format!(
            r#"{{"bridge_id":"{}","bridge_status":"forwarded","target_message_id":"target-{}","forward_tx_hash":"sha256:bridge-{}"}}"#,
            bridge_id, bridge_id, bridge_id
        ))
    }

    fn create_task(&self, payload: &str) -> Result<String, AgentLibError> {
        Ok(format!(
            r#"{{"task_id":"task-{}","state":"created"}}"#,
            payload.len()
        ))
    }

    fn accept_task(&self, task_id: &str) -> Result<String, AgentLibError> {
        Ok(format!(r#"{{"task_id":"{}","state":"accepted"}}"#, task_id))
    }

    fn complete_task(&self, task_id: &str) -> Result<String, AgentLibError> {
        Ok(format!(
            r#"{{"task_id":"{}","state":"completed"}}"#,
            task_id
        ))
    }

    fn fund_escrow(&self, payload: &str) -> Result<String, AgentLibError> {
        Ok(format!(
            r#"{{"escrow_id":"escrow-{}","state":"funded"}}"#,
            payload.len()
        ))
    }

    fn release_escrow(&self, escrow_id: &str) -> Result<String, AgentLibError> {
        Ok(format!(
            r#"{{"escrow_id":"{}","state":"released"}}"#,
            escrow_id
        ))
    }

    fn health(&self) -> Result<String, AgentLibError> {
        Ok(r#"{"status":"ok","runtime_mode":"api"}"#.to_owned())
    }

    fn verify_proof(
        &self,
        message_id: &str,
        tx_hash: &str,
        block_height: u64,
        finality: &str,
    ) -> Result<String, AgentLibError> {
        Ok(format!(
            r#"{{"message_id":"{}","tx_hash":"{}","block_height":{},"finality":"{}","verified":true}}"#,
            message_id, tx_hash, block_height, finality
        ))
    }
}

fn frame_request(body: &str) -> String {
    format!("Content-Length: {}\r\n\r\n{}", body.len(), body)
}

fn parse_framed_json(response: &str) -> String {
    let marker = "\r\n\r\n";
    let split = response
        .find(marker)
        .expect("framed response should include header/body split");
    let header = &response[..split];
    let body = &response[split + marker.len()..];
    let declared_length = header
        .strip_prefix("Content-Length: ")
        .expect("header should start with content length")
        .trim()
        .parse::<usize>()
        .expect("content length should be numeric");
    assert_eq!(
        declared_length,
        body.len(),
        "declared content length should match JSON body bytes",
    );
    body.to_owned()
}

#[test]
fn spec_c01_mcp_initialize_framed_jsonrpc_response_contract() {
    let backend = ProtocolBackend;
    let request =
        frame_request(r#"{"jsonrpc":"2.0","id":"req-1","method":"initialize","params":{}}"#);

    let responses = process_stdio_input(&backend, request.as_str()).expect("input should parse");
    assert_eq!(responses.len(), 1, "initialize should return one response");
    assert!(
        responses[0].starts_with("Content-Length: "),
        "response should be framed",
    );

    let body = parse_framed_json(responses[0].as_str());
    assert!(
        body.contains(r#""jsonrpc":"2.0""#),
        "initialize response should preserve jsonrpc marker: {body}",
    );
    assert!(
        body.contains(r#""id":"req-1""#),
        "initialize response should preserve request id: {body}",
    );
    assert!(
        body.contains(r#""serverInfo""#),
        "initialize response should include server info: {body}",
    );
}

#[test]
fn spec_c02_mcp_tools_list_framed_tool_inventory_contract() {
    let backend = ProtocolBackend;
    let request = frame_request(r#"{"jsonrpc":"2.0","id":"req-2","method":"tools/list"}"#);

    let responses = process_stdio_input(&backend, request.as_str()).expect("input should parse");
    assert_eq!(responses.len(), 1, "tools/list should return one response");

    let body = parse_framed_json(responses[0].as_str());
    assert!(
        body.contains(r#""tools""#),
        "tools/list response should include tools field: {body}",
    );
    assert!(
        body.contains(r#""name":"register""#),
        "tools/list should include register tool: {body}",
    );
    assert!(
        body.contains(r#""name":"verify_proof""#),
        "tools/list should include verify_proof tool: {body}",
    );
    assert!(
        body.contains(r#""name":"query_task""#),
        "tools/list should include query_task tool: {body}",
    );
    assert!(
        body.contains(r#""name":"query_agent_profile""#),
        "tools/list should include query_agent_profile tool: {body}",
    );
    assert!(
        body.contains(r#""name":"register_content""#),
        "tools/list should include register_content tool: {body}",
    );
    assert!(
        body.contains(r#""name":"expire_content""#),
        "tools/list should include expire_content tool: {body}",
    );
    assert!(
        body.contains(r#""name":"tombstone_content""#),
        "tools/list should include tombstone_content tool: {body}",
    );
    assert!(
        body.contains(r#""name":"query_content""#),
        "tools/list should include query_content tool: {body}",
    );
    assert!(
        body.contains(r#""name":"submit_bridge_message""#),
        "tools/list should include submit_bridge_message tool: {body}",
    );
    assert!(
        body.contains(r#""name":"forward_bridge_message""#),
        "tools/list should include forward_bridge_message tool: {body}",
    );
    assert!(
        body.contains(r#""name":"query_bridge_message""#),
        "tools/list should include query_bridge_message tool: {body}",
    );
}

#[test]
fn spec_c03_mcp_tools_call_health_dispatch_contract() {
    let backend = ProtocolBackend;
    let request = frame_request(
        r#"{"jsonrpc":"2.0","id":"req-3","method":"tools/call","params":{"name":"health","arguments":{}}}"#,
    );

    let responses = process_stdio_input(&backend, request.as_str()).expect("input should parse");
    let body = parse_framed_json(responses[0].as_str());
    assert!(
        body.contains(r#""jsonrpc":"2.0""#),
        "tools/call should return JSON-RPC response: {body}",
    );
    assert!(
        body.contains(r#""status":"ok""#),
        "health payload should be present in result: {body}",
    );
}

#[test]
fn spec_c04_mcp_method_not_found_error_contract() {
    let backend = ProtocolBackend;
    let request = frame_request(r#"{"jsonrpc":"2.0","id":"req-4","method":"unknown/method"}"#);

    let responses = process_stdio_input(&backend, request.as_str()).expect("input should parse");
    let body = parse_framed_json(responses[0].as_str());
    assert!(
        body.contains(r#""error""#),
        "unsupported method should return error envelope: {body}",
    );
    assert!(
        body.contains(r#""code":-32601"#),
        "unsupported method should map to method not found code: {body}",
    );
}

#[test]
fn spec_c05_mcp_invalid_params_error_contract() {
    let backend = ProtocolBackend;
    let request =
        frame_request(r#"{"jsonrpc":"2.0","id":"req-5","method":"tools/call","params":{}}"#);

    let responses = process_stdio_input(&backend, request.as_str()).expect("input should parse");
    let body = parse_framed_json(responses[0].as_str());
    assert!(
        body.contains(r#""error""#),
        "malformed tools/call should return error envelope: {body}",
    );
    assert!(
        body.contains(r#""code":-32602"#),
        "malformed tools/call should map to invalid params code: {body}",
    );
}

#[test]
fn spec_c07_mcp_tools_call_query_task_and_profile_dispatch_contract() {
    let backend = ProtocolBackend;

    let query_task_request = frame_request(
        r#"{"jsonrpc":"2.0","id":"req-7","method":"tools/call","params":{"name":"query_task","arguments":{"task_id":"task-1"}}}"#,
    );
    let query_task_responses =
        process_stdio_input(&backend, query_task_request.as_str()).expect("input should parse");
    let query_task_body = parse_framed_json(query_task_responses[0].as_str());
    assert!(
        query_task_body.contains(r#""jsonrpc":"2.0""#),
        "query_task should return JSON-RPC response: {query_task_body}",
    );
    assert!(
        query_task_body.contains(r#""tool":"query_task""#),
        "query_task payload should preserve tool marker: {query_task_body}",
    );
    assert!(
        query_task_body.contains(r#""state":"submitted""#),
        "query_task payload should include state projection: {query_task_body}",
    );

    let query_profile_request = frame_request(
        r#"{"jsonrpc":"2.0","id":"req-8","method":"tools/call","params":{"name":"query_agent_profile","arguments":{"did":"kamn:did:agent:alice"}}}"#,
    );
    let query_profile_responses =
        process_stdio_input(&backend, query_profile_request.as_str()).expect("input should parse");
    let query_profile_body = parse_framed_json(query_profile_responses[0].as_str());
    assert!(
        query_profile_body.contains(r#""tool":"query_agent_profile""#),
        "query_agent_profile payload should preserve tool marker: {query_profile_body}",
    );
    assert!(
        query_profile_body.contains(r#""reputation_score":777"#),
        "query_agent_profile payload should include reputation score: {query_profile_body}",
    );
}

#[test]
fn spec_c08_mcp_tools_call_content_lifecycle_dispatch_contract() {
    let backend = ProtocolBackend;

    let register_request = frame_request(
        r#"{"jsonrpc":"2.0","id":"req-9","method":"tools/call","params":{"name":"register_content","arguments":{"payload":"{\"content\":\"abc\"}"}}}"#,
    );
    let register_responses =
        process_stdio_input(&backend, register_request.as_str()).expect("input should parse");
    let register_body = parse_framed_json(register_responses[0].as_str());
    assert!(
        register_body.contains(r#""tool":"register_content""#),
        "register_content payload should preserve tool marker: {register_body}",
    );
    assert!(
        register_body.contains(r#""retention_class":"standard""#),
        "register_content payload should include retention projection: {register_body}",
    );

    let expire_request = frame_request(
        r#"{"jsonrpc":"2.0","id":"req-10","method":"tools/call","params":{"name":"expire_content","arguments":{"content_id":"content-1"}}}"#,
    );
    let expire_responses =
        process_stdio_input(&backend, expire_request.as_str()).expect("input should parse");
    let expire_body = parse_framed_json(expire_responses[0].as_str());
    assert!(
        expire_body.contains(r#""lifecycle_state":"expired""#),
        "expire_content payload should include lifecycle projection: {expire_body}",
    );

    let tombstone_request = frame_request(
        r#"{"jsonrpc":"2.0","id":"req-11","method":"tools/call","params":{"name":"tombstone_content","arguments":{"content_id":"content-1"}}}"#,
    );
    let tombstone_responses =
        process_stdio_input(&backend, tombstone_request.as_str()).expect("input should parse");
    let tombstone_body = parse_framed_json(tombstone_responses[0].as_str());
    assert!(
        tombstone_body.contains(r#""redaction_status":"redacted""#),
        "tombstone_content payload should include redaction projection: {tombstone_body}",
    );

    let query_request = frame_request(
        r#"{"jsonrpc":"2.0","id":"req-12","method":"tools/call","params":{"name":"query_content","arguments":{"content_id":"content-1"}}}"#,
    );
    let query_responses =
        process_stdio_input(&backend, query_request.as_str()).expect("input should parse");
    let query_body = parse_framed_json(query_responses[0].as_str());
    assert!(
        query_body.contains(r#""tool":"query_content""#),
        "query_content payload should preserve tool marker: {query_body}",
    );
    assert!(
        query_body.contains(r#""lifecycle_state":"tombstoned""#),
        "query_content payload should include lifecycle projection: {query_body}",
    );

    let submit_bridge_request = frame_request(
        r#"{"jsonrpc":"2.0","id":"req-13","method":"tools/call","params":{"name":"submit_bridge_message","arguments":{"payload":"{\"source_message_id\":\"msg-1\"}"}}}"#,
    );
    let submit_bridge_responses =
        process_stdio_input(&backend, submit_bridge_request.as_str()).expect("input should parse");
    let submit_bridge_body = parse_framed_json(submit_bridge_responses[0].as_str());
    assert!(
        submit_bridge_body.contains(r#""tool":"submit_bridge_message""#),
        "submit_bridge_message payload should preserve tool marker: {submit_bridge_body}",
    );
    assert!(
        submit_bridge_body.contains(r#""bridge_status":"submitted""#),
        "submit_bridge_message payload should include submitted status: {submit_bridge_body}",
    );

    let forward_bridge_request = frame_request(
        r#"{"jsonrpc":"2.0","id":"req-14","method":"tools/call","params":{"name":"forward_bridge_message","arguments":{"bridge_id":"bridge-1"}}}"#,
    );
    let forward_bridge_responses =
        process_stdio_input(&backend, forward_bridge_request.as_str()).expect("input should parse");
    let forward_bridge_body = parse_framed_json(forward_bridge_responses[0].as_str());
    assert!(
        forward_bridge_body.contains(r#""bridge_status":"forwarded""#),
        "forward_bridge_message payload should include forwarded status: {forward_bridge_body}",
    );

    let query_bridge_request = frame_request(
        r#"{"jsonrpc":"2.0","id":"req-15","method":"tools/call","params":{"name":"query_bridge_message","arguments":{"bridge_id":"bridge-1"}}}"#,
    );
    let query_bridge_responses =
        process_stdio_input(&backend, query_bridge_request.as_str()).expect("input should parse");
    let query_bridge_body = parse_framed_json(query_bridge_responses[0].as_str());
    assert!(
        query_bridge_body.contains(r#""tool":"query_bridge_message""#),
        "query_bridge_message payload should preserve tool marker: {query_bridge_body}",
    );
    assert!(
        query_bridge_body.contains(r#""forward_tx_hash":"sha256:bridge-bridge-1""#),
        "query_bridge_message payload should include forward hash projection: {query_bridge_body}",
    );
}

#[test]
fn spec_c06_line_mode_dispatch_remains_supported_contract() {
    let backend = ProtocolBackend;
    let responses =
        process_stdio_input(&backend, r#"{"id":"req-6","tool":"health"}"#).expect("line mode");
    assert_eq!(responses.len(), 1, "line mode should return one response");
    assert!(
        responses[0].contains(r#""ok":true"#),
        "line mode should preserve existing dispatch shape: {}",
        responses[0]
    );
}
