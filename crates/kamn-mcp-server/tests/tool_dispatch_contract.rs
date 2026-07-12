use kamn_agent_lib::AgentLibError;
use kamn_mcp_server::{dispatch_tool_request_json, McpToolBackend};

#[derive(Debug, Default)]
struct TestBackend;

impl McpToolBackend for TestBackend {
    fn register(&self) -> Result<String, AgentLibError> {
        Ok(r#"{"did":"kamn:did:agent:test"}"#.to_owned())
    }

    fn send_message(&self, payload: &str) -> Result<String, AgentLibError> {
        let payload_len = payload.len();
        Ok(format!(r#"{{"message_id":"msg-{payload_len}"}}"#))
    }

    fn create_channel(&self, payload: &str) -> Result<String, AgentLibError> {
        let payload_len = payload.len();
        Ok(format!(r#"{{"channel_id":"channel-{payload_len}"}}"#))
    }

    fn list_messages(&self, channel_id: &str) -> Result<String, AgentLibError> {
        Ok(format!(
            r#"{{"channel_id":"{channel_id}","messages":["msg-a","msg-b"]}}"#
        ))
    }

    fn query_message(&self, message_id: &str) -> Result<String, AgentLibError> {
        Ok(format!(
            r#"{{"message_id":"{message_id}","status":"created"}}"#
        ))
    }

    fn query_task(&self, task_id: &str) -> Result<String, AgentLibError> {
        Ok(format!(r#"{{"task_id":"{task_id}","state":"submitted"}}"#))
    }

    fn query_participant_task_projection(&self, task_id: &str) -> Result<String, AgentLibError> {
        Ok(format!(
            r#"{{"task_id":"{task_id}","view_scope":"participant-private"}}"#
        ))
    }

    fn query_verifier_task_projection(&self, task_id: &str) -> Result<String, AgentLibError> {
        Ok(format!(
            r#"{{"task_id":"{task_id}","view_scope":"restricted-public"}}"#
        ))
    }

    fn query_agent_profile(&self, did: &str) -> Result<String, AgentLibError> {
        Ok(format!(r#"{{"did":"{did}","reputation_score":777}}"#))
    }

    fn register_content(&self, payload: &str) -> Result<String, AgentLibError> {
        let payload_len = payload.len();
        Ok(format!(
            r#"{{"content_id":"content-{payload_len}","retention_class":"standard","lifecycle_state":"retained","redaction_status":"none"}}"#
        ))
    }

    fn expire_content(&self, content_id: &str) -> Result<String, AgentLibError> {
        Ok(format!(
            r#"{{"content_id":"{content_id}","lifecycle_state":"expired","redaction_status":"none"}}"#
        ))
    }

    fn tombstone_content(&self, content_id: &str) -> Result<String, AgentLibError> {
        Ok(format!(
            r#"{{"content_id":"{content_id}","lifecycle_state":"tombstoned","redaction_status":"redacted"}}"#
        ))
    }

    fn query_content(&self, content_id: &str) -> Result<String, AgentLibError> {
        Ok(format!(
            r#"{{"content_id":"{content_id}","lifecycle_state":"tombstoned","redaction_status":"redacted"}}"#
        ))
    }

    fn submit_bridge_message(&self, payload: &str) -> Result<String, AgentLibError> {
        let payload_len = payload.len();
        Ok(format!(
            r#"{{"bridge_id":"bridge-{payload_len}","source_message_id":"source-{payload_len}","bridge_status":"submitted"}}"#
        ))
    }

    fn forward_bridge_message(&self, bridge_id: &str) -> Result<String, AgentLibError> {
        Ok(format!(
            r#"{{"bridge_id":"{bridge_id}","bridge_status":"forwarded","target_message_id":"target-{bridge_id}","forward_tx_hash":"sha256:bridge-{bridge_id}"}}"#
        ))
    }

    fn query_bridge_message(&self, bridge_id: &str) -> Result<String, AgentLibError> {
        Ok(format!(
            r#"{{"bridge_id":"{bridge_id}","bridge_status":"forwarded","target_message_id":"target-{bridge_id}","forward_tx_hash":"sha256:bridge-{bridge_id}"}}"#
        ))
    }

    fn create_task(&self, payload: &str) -> Result<String, AgentLibError> {
        let payload_len = payload.len();
        Ok(format!(r#"{{"task_id":"task-{payload_len}"}}"#))
    }

    fn accept_task(&self, task_id: &str, _payload: &str) -> Result<String, AgentLibError> {
        Ok(format!(r#"{{"task_id":"{task_id}","state":"accepted"}}"#))
    }

    fn complete_task(&self, task_id: &str, _payload: &str) -> Result<String, AgentLibError> {
        Ok(format!(r#"{{"task_id":"{task_id}","state":"completed"}}"#))
    }

    fn fund_escrow(&self, payload: &str) -> Result<String, AgentLibError> {
        let payload_len = payload.len();
        Ok(format!(
            r#"{{"escrow_id":"escrow-{payload_len}","state":"funded"}}"#
        ))
    }

    fn release_escrow(&self, escrow_id: &str, _payload: &str) -> Result<String, AgentLibError> {
        Ok(format!(
            r#"{{"escrow_id":"{escrow_id}","state":"released"}}"#
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
            r#"{{"message_id":"{message_id}","tx_hash":"{tx_hash}","block_height":{block_height},"finality":"{finality}","verified":{verified}}}"#
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
        r#"{"id":"req-5","tool":"accept_task","task_id":"task-1","payload":"{\"idempotency_key\":\"accept-1\"}"}"#,
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
        r#"{"id":"req-6","tool":"complete_task","task_id":"task-1","payload":"{\"idempotency_key\":\"complete-1\"}"}"#,
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
        r#"{"id":"req-8","tool":"release_escrow","escrow_id":"escrow-1","payload":"{\"idempotency_key\":\"release-1\"}"}"#,
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
        r#"{"id":"req-15","tool":"query_task"}"#,
        r#"{"id":"req-16","tool":"query_agent_profile"}"#,
        r#"{"id":"req-17","tool":"register_content"}"#,
        r#"{"id":"req-18","tool":"expire_content"}"#,
        r#"{"id":"req-19","tool":"tombstone_content"}"#,
        r#"{"id":"req-20","tool":"query_content"}"#,
        r#"{"id":"req-21","tool":"submit_bridge_message"}"#,
        r#"{"id":"req-22","tool":"forward_bridge_message"}"#,
        r#"{"id":"req-23","tool":"query_bridge_message"}"#,
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

#[test]
fn spec_c07_mcp_dispatch_executes_query_task_and_query_agent_profile_tools() {
    let backend = TestBackend;

    let task_response = dispatch_tool_request_json(
        &backend,
        r#"{"id":"req-13","tool":"query_task","task_id":"task-1"}"#,
    )
    .expect("query_task should dispatch successfully");
    assert!(
        task_response.contains(r#""ok":true"#),
        "query_task response should mark success: {task_response}"
    );
    assert!(
        task_response.contains(r#""tool":"query_task""#),
        "query_task response should preserve tool marker: {task_response}"
    );
    assert!(
        task_response.contains(r#""state":"submitted""#),
        "query_task response should include state projection: {task_response}"
    );

    let profile_response = dispatch_tool_request_json(
        &backend,
        r#"{"id":"req-14","tool":"query_agent_profile","did":"kamn:did:agent:alice"}"#,
    )
    .expect("query_agent_profile should dispatch successfully");
    assert!(
        profile_response.contains(r#""ok":true"#),
        "query_agent_profile response should mark success: {profile_response}"
    );
    assert!(
        profile_response.contains(r#""tool":"query_agent_profile""#),
        "query_agent_profile response should preserve tool marker: {profile_response}"
    );
    assert!(
        profile_response.contains(r#""reputation_score":777"#),
        "query_agent_profile response should include reputation score: {profile_response}"
    );
}

#[test]
fn spec_c08_mcp_dispatch_executes_content_lifecycle_tools() {
    let backend = TestBackend;

    let register_response = dispatch_tool_request_json(
        &backend,
        r#"{"id":"req-21","tool":"register_content","payload":"{\"content\":\"abc\",\"retention_class\":\"standard\"}"}"#,
    )
    .expect("register_content should dispatch successfully");
    assert!(
        register_response.contains(r#""ok":true"#),
        "register_content response should mark success: {register_response}"
    );
    assert!(
        register_response.contains(r#""tool":"register_content""#),
        "register_content response should preserve tool marker: {register_response}"
    );
    assert!(
        register_response.contains(r#""retention_class":"standard""#),
        "register_content response should include retention class: {register_response}"
    );

    let expire_response = dispatch_tool_request_json(
        &backend,
        r#"{"id":"req-22","tool":"expire_content","content_id":"content-1"}"#,
    )
    .expect("expire_content should dispatch successfully");
    assert!(
        expire_response.contains(r#""ok":true"#),
        "expire_content response should mark success: {expire_response}"
    );
    assert!(
        expire_response.contains(r#""lifecycle_state":"expired""#),
        "expire_content response should include expired lifecycle state: {expire_response}"
    );

    let tombstone_response = dispatch_tool_request_json(
        &backend,
        r#"{"id":"req-23","tool":"tombstone_content","content_id":"content-1"}"#,
    )
    .expect("tombstone_content should dispatch successfully");
    assert!(
        tombstone_response.contains(r#""ok":true"#),
        "tombstone_content response should mark success: {tombstone_response}"
    );
    assert!(
        tombstone_response.contains(r#""redaction_status":"redacted""#),
        "tombstone_content response should include redaction status: {tombstone_response}"
    );

    let query_response = dispatch_tool_request_json(
        &backend,
        r#"{"id":"req-24","tool":"query_content","content_id":"content-1"}"#,
    )
    .expect("query_content should dispatch successfully");
    assert!(
        query_response.contains(r#""ok":true"#),
        "query_content response should mark success: {query_response}"
    );
    assert!(
        query_response.contains(r#""tool":"query_content""#),
        "query_content response should preserve tool marker: {query_response}"
    );
    assert!(
        query_response.contains(r#""lifecycle_state":"tombstoned""#),
        "query_content response should include lifecycle projection: {query_response}"
    );
}

#[test]
fn spec_c09_mcp_dispatch_executes_bridge_lifecycle_tools() {
    let backend = TestBackend;

    let submit_response = dispatch_tool_request_json(
        &backend,
        r#"{"id":"req-25","tool":"submit_bridge_message","payload":"{\"source_message_id\":\"msg-1\"}"}"#,
    )
    .expect("submit_bridge_message should dispatch successfully");
    assert!(
        submit_response.contains(r#""ok":true"#),
        "submit_bridge_message response should mark success: {submit_response}"
    );
    assert!(
        submit_response.contains(r#""tool":"submit_bridge_message""#),
        "submit_bridge_message response should preserve tool marker: {submit_response}"
    );
    assert!(
        submit_response.contains(r#""bridge_status":"submitted""#),
        "submit_bridge_message response should include submitted status: {submit_response}"
    );

    let forward_response = dispatch_tool_request_json(
        &backend,
        r#"{"id":"req-26","tool":"forward_bridge_message","bridge_id":"bridge-1"}"#,
    )
    .expect("forward_bridge_message should dispatch successfully");
    assert!(
        forward_response.contains(r#""ok":true"#),
        "forward_bridge_message response should mark success: {forward_response}"
    );
    assert!(
        forward_response.contains(r#""bridge_status":"forwarded""#),
        "forward_bridge_message response should include forwarded status: {forward_response}"
    );

    let query_response = dispatch_tool_request_json(
        &backend,
        r#"{"id":"req-27","tool":"query_bridge_message","bridge_id":"bridge-1"}"#,
    )
    .expect("query_bridge_message should dispatch successfully");
    assert!(
        query_response.contains(r#""ok":true"#),
        "query_bridge_message response should mark success: {query_response}"
    );
    assert!(
        query_response.contains(r#""tool":"query_bridge_message""#),
        "query_bridge_message response should preserve tool marker: {query_response}"
    );
    assert!(
        query_response.contains(r#""forward_tx_hash":"sha256:bridge-bridge-1""#),
        "query_bridge_message response should include forward hash projection: {query_response}"
    );
}
