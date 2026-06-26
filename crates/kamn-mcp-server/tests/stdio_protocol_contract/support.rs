use kamn_agent_lib::AgentLibError;
use kamn_mcp_server::McpToolBackend;

#[derive(Debug, Default)]
pub(crate) struct ProtocolBackend;

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

pub(crate) fn frame_request(body: &str) -> String {
    format!("Content-Length: {}\r\n\r\n{}", body.len(), body)
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
