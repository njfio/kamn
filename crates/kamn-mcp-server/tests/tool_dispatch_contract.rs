use kamn_agent_lib::AgentLibError;
use kamn_mcp_server::{dispatch_tool_request_json, McpToolBackend};

#[derive(Debug, Default)]
struct TestBackend;

impl McpToolBackend for TestBackend {
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
        Ok(format!(r#"{{"task_id":"task-{}"}}"#, payload.len()))
    }

    fn health(&self) -> Result<String, AgentLibError> {
        Ok(r#"{"status":"ok","runtime_mode":"api"}"#.to_owned())
    }
}

#[test]
fn spec_c03_mcp_dispatch_returns_structured_success_for_supported_tool() {
    let backend = TestBackend;
    let response = dispatch_tool_request_json(&backend, r#"{"id":"req-1","tool":"health"}"#)
        .expect("supported tool dispatch should succeed");

    assert!(
        response.contains(r#""ok":true"#),
        "response should mark success: {response}"
    );
    assert!(
        response.contains(r#""tool":"health""#),
        "response should preserve tool name: {response}"
    );
    assert!(
        response.contains(r#""status":"ok""#),
        "response should include backend result payload: {response}"
    );
}

#[test]
fn spec_c04_mcp_dispatch_returns_structured_error_for_unsupported_tool() {
    let backend = TestBackend;
    let response = dispatch_tool_request_json(&backend, r#"{"id":"req-2","tool":"accept_task"}"#)
        .expect("unsupported operation should still return a structured envelope");

    assert!(
        response.contains(r#""ok":false"#),
        "response should mark failure: {response}"
    );
    assert!(
        response.contains(r#""kind":"unsupported_operation""#),
        "response should encode unsupported kind: {response}"
    );
    assert!(
        response.contains(r#""tool":"accept_task""#),
        "response should preserve requested tool: {response}"
    );
}
