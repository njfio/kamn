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
        let verified = finality.eq_ignore_ascii_case("final")
            || finality.eq_ignore_ascii_case("finalized")
            || finality.eq_ignore_ascii_case("confirmed");
        Ok(format!(
            r#"{{"message_id":"{}","tx_hash":"{}","block_height":{},"finality":"{}","verified":{}}}"#,
            message_id, tx_hash, block_height, finality, verified
        ))
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
    let response = dispatch_tool_request_json(&backend, r#"{"id":"req-2","tool":"unknown_tool"}"#)
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
        response.contains(r#""tool":"unknown_tool""#),
        "response should preserve requested tool: {response}"
    );
}

#[test]
fn spec_c01_mcp_dispatch_executes_verify_proof_with_structured_success() {
    let backend = TestBackend;
    let response = dispatch_tool_request_json(
        &backend,
        r#"{"id":"req-3","tool":"verify_proof","message_id":"msg-1","tx_hash":"tx-1","block_height":"9","finality":"final"}"#,
    )
    .expect("verify_proof should dispatch successfully");

    assert!(
        response.contains(r#""ok":true"#),
        "response should mark success: {response}"
    );
    assert!(
        response.contains(r#""tool":"verify_proof""#),
        "response should preserve tool name: {response}"
    );
    assert!(
        response.contains(r#""verified":true"#),
        "verify result should include verified projection: {response}"
    );
}

#[test]
fn spec_c02_mcp_dispatch_rejects_malformed_verify_proof_requests() {
    let backend = TestBackend;
    let response = dispatch_tool_request_json(
        &backend,
        r#"{"id":"req-4","tool":"verify_proof","message_id":"msg-1","tx_hash":"tx-1","finality":"final"}"#,
    )
    .expect("dispatcher should return structured invalid-request payload");

    assert!(
        response.contains(r#""ok":false"#),
        "response should mark failure: {response}"
    );
    assert!(
        response.contains(r#""kind":"invalid_request""#),
        "response should encode invalid_request kind: {response}"
    );
}

#[test]
fn spec_c05_mcp_dispatch_executes_task_and_escrow_tools_with_structured_success() {
    let backend = TestBackend;

    let accept = dispatch_tool_request_json(
        &backend,
        r#"{"id":"req-5","tool":"accept_task","task_id":"task-1"}"#,
    )
    .expect("accept_task should dispatch successfully");
    assert!(
        accept.contains(r#""ok":true"#),
        "accept should succeed: {accept}"
    );
    assert!(
        accept.contains(r#""state":"accepted""#),
        "accept payload should include state: {accept}"
    );

    let complete = dispatch_tool_request_json(
        &backend,
        r#"{"id":"req-6","tool":"complete_task","task_id":"task-1"}"#,
    )
    .expect("complete_task should dispatch successfully");
    assert!(
        complete.contains(r#""ok":true"#),
        "complete should succeed: {complete}"
    );
    assert!(
        complete.contains(r#""state":"completed""#),
        "complete payload should include state: {complete}"
    );

    let fund = dispatch_tool_request_json(
        &backend,
        r#"{"id":"req-7","tool":"fund_escrow","payload":"{\"task_id\":\"task-1\",\"amount\":100}"}"#,
    )
    .expect("fund_escrow should dispatch successfully");
    assert!(fund.contains(r#""ok":true"#), "fund should succeed: {fund}");
    assert!(
        fund.contains(r#""state":"funded""#),
        "fund payload should include state: {fund}"
    );

    let release = dispatch_tool_request_json(
        &backend,
        r#"{"id":"req-8","tool":"release_escrow","escrow_id":"escrow-1"}"#,
    )
    .expect("release_escrow should dispatch successfully");
    assert!(
        release.contains(r#""ok":true"#),
        "release should succeed: {release}"
    );
    assert!(
        release.contains(r#""state":"released""#),
        "release payload should include state: {release}"
    );
}

#[test]
fn spec_c06_mcp_dispatch_rejects_malformed_task_and_escrow_requests() {
    let backend = TestBackend;

    for request in [
        r#"{"id":"req-9","tool":"accept_task"}"#,
        r#"{"id":"req-10","tool":"complete_task"}"#,
        r#"{"id":"req-11","tool":"fund_escrow"}"#,
        r#"{"id":"req-12","tool":"release_escrow"}"#,
    ] {
        let response = dispatch_tool_request_json(&backend, request)
            .expect("dispatcher should return structured invalid-request envelope");
        assert!(
            response.contains(r#""ok":false"#),
            "response should mark failure: {response}"
        );
        assert!(
            response.contains(r#""kind":"invalid_request""#),
            "response should encode invalid_request kind: {response}"
        );
    }
}
