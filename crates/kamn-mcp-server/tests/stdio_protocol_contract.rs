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
