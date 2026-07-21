use kamn_agent_lib::AgentLibError;
use kamn_mcp_server::{dispatch_tool_request_json, McpToolBackend};

const DIGEST: &str = "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";

macro_rules! unsupported_backend_methods {
    () => {
        fn send_message(&self, _: &str) -> Result<String, AgentLibError> {
            unsupported()
        }
        fn create_channel(&self, _: &str) -> Result<String, AgentLibError> {
            unsupported()
        }
        fn list_messages(&self, _: &str) -> Result<String, AgentLibError> {
            unsupported()
        }
        fn query_message(&self, _: &str) -> Result<String, AgentLibError> {
            unsupported()
        }
        fn query_task(&self, _: &str) -> Result<String, AgentLibError> {
            unsupported()
        }
        fn query_participant_task_projection(&self, _: &str) -> Result<String, AgentLibError> {
            unsupported()
        }
        fn query_verifier_task_projection(&self, _: &str) -> Result<String, AgentLibError> {
            unsupported()
        }
        fn query_agent_profile(&self, _: &str) -> Result<String, AgentLibError> {
            unsupported()
        }
        fn register_content(&self, _: &str) -> Result<String, AgentLibError> {
            unsupported()
        }
        fn expire_content(&self, _: &str) -> Result<String, AgentLibError> {
            unsupported()
        }
        fn tombstone_content(&self, _: &str) -> Result<String, AgentLibError> {
            unsupported()
        }
        fn query_content(&self, _: &str) -> Result<String, AgentLibError> {
            unsupported()
        }
        fn submit_bridge_message(&self, _: &str) -> Result<String, AgentLibError> {
            unsupported()
        }
        fn forward_bridge_message(&self, _: &str) -> Result<String, AgentLibError> {
            unsupported()
        }
        fn query_bridge_message(&self, _: &str) -> Result<String, AgentLibError> {
            unsupported()
        }
        fn create_task(&self, _: &str) -> Result<String, AgentLibError> {
            unsupported()
        }
        fn fund_escrow(&self, _: &str) -> Result<String, AgentLibError> {
            unsupported()
        }
        fn release_escrow(&self, _: &str, _: &str) -> Result<String, AgentLibError> {
            unsupported()
        }
        fn health(&self) -> Result<String, AgentLibError> {
            unsupported()
        }
        fn verify_proof(&self, _: &str, _: &str, _: u64, _: &str) -> Result<String, AgentLibError> {
            unsupported()
        }
    };
}

struct AuthorityBackend;

impl McpToolBackend for AuthorityBackend {
    fn register(&self) -> Result<String, AgentLibError> {
        Ok(format!(
            r#"{{"did":"kamn:did:agent:test","profile_commitment":"{DIGEST}"}}"#
        ))
    }

    fn accept_task(&self, task_id: &str, _payload: &str) -> Result<String, AgentLibError> {
        Ok(task_authority(task_id, "accepted"))
    }

    fn complete_task(&self, task_id: &str, _payload: &str) -> Result<String, AgentLibError> {
        Ok(format!(r#"{{"task_id":"{task_id}","state":"completed"}}"#))
    }

    unsupported_backend_methods!();
}

fn unsupported() -> Result<String, AgentLibError> {
    Err(AgentLibError::UnsupportedOperation(
        "unused test backend method",
    ))
}

fn task_authority(task_id: &str, state: &str) -> String {
    format!(
        r#"{{"actor_did":"kamn:did:agent:test","task_id":"{task_id}","state":"{state}","receipt_id":"task-transition-receipt-1","receipt_digest":"{DIGEST}"}}"#
    )
}

#[test]
fn mutation_result_is_wrapped_as_service_authority_v1() {
    let response = dispatch_tool_request_json(
        &AuthorityBackend,
        r#"{"id":"req-1","tool":"accept_task","task_id":"task-1","payload":"{}"}"#,
    )
    .expect("dispatch should return an envelope");

    assert!(response.contains(r#""schema_version":"kamn.mcp.authority-receipt.v1""#));
    assert!(response.contains(r#""authority_kind":"service-receipt""#));
    assert!(response.contains(r#""source":"kamn-service""#));
    assert!(response.contains(r#""service_receipt_id":"task-transition-receipt-1""#));
    assert!(response.contains(r#""service_receipt_digest":"sha256:""#));
}

#[test]
fn legacy_mutation_result_fails_closed() {
    let response = dispatch_tool_request_json(
        &AuthorityBackend,
        r#"{"id":"req-2","tool":"complete_task","task_id":"task-1","payload":"{}"}"#,
    )
    .expect("dispatch should return an error envelope");

    assert!(response.contains(r#""ok":false"#));
    assert!(response.contains("MCP_AUTHORITY_RECEIPT_MISSING"));
}

#[test]
fn registration_uses_service_profile_commitment_authority() {
    let response =
        dispatch_tool_request_json(&AuthorityBackend, r#"{"id":"req-3","tool":"register"}"#)
            .expect("dispatch should return an envelope");

    assert!(response.contains(r#""schema_version":"kamn.mcp.authority-receipt.v1""#));
    assert!(response.contains(r#""authority_kind":"service-profile-commitment""#));
    assert!(response.contains(r#""profile_commitment":"sha256:""#));
}
