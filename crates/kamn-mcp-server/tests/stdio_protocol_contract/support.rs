use kamn_agent_lib::AgentLibError;
use kamn_mcp_server::McpToolBackend;

#[derive(Debug, Default)]
pub(crate) struct ProtocolBackend;

impl McpToolBackend for ProtocolBackend {
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
        Ok(format!(
            r#"{{"task_id":"task-{payload_len}","state":"created"}}"#
        ))
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
        Ok(format!(
            r#"{{"message_id":"{message_id}","tx_hash":"{tx_hash}","block_height":{block_height},"finality":"{finality}","verified":true}}"#
        ))
    }
}

pub(crate) fn frame_request(body: &str) -> String {
    let body_len = body.len();
    format!("Content-Length: {body_len}\r\n\r\n{body}")
}

pub(crate) fn parse_framed_json(response: &str) -> String {
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
