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
    fn actor_did(&self) -> &str {
        "kamn:did:agent:test"
    }

    fn register(&self) -> Result<String, AgentLibError> {
        Ok(format!(
            r#"{{"did":"kamn:did:agent:test","profile_commitment":"{DIGEST}"}}"#
        ))
    }

    fn accept_task(&self, task_id: &str, _payload: &str) -> Result<String, AgentLibError> {
        let authority = match task_id {
            "wrong-resource" => task_authority("other-task", "accepted"),
            "wrong-actor" => task_authority(task_id, "accepted")
                .replace("kamn:did:agent:test", "kamn:did:agent:other"),
            "wrong-action" => {
                task_authority(task_id, "accepted").replace("task:accept", "task:complete")
            }
            "wrong-state" => task_authority(task_id, "accepted")
                .replace(r#""state":"accepted""#, r#""state":"submitted""#),
            "bad-digest" => task_authority(task_id, "accepted").replace(DIGEST, "sha256:BAD"),
            _ => task_authority(task_id, "accepted"),
        };
        Ok(authority)
    }

    fn complete_task(&self, task_id: &str, _payload: &str) -> Result<String, AgentLibError> {
        Ok(format!(r#"{{"task_id":"{task_id}","state":"completed"}}"#))
    }

    fn release_escrow(&self, escrow_id: &str, _: &str) -> Result<String, AgentLibError> {
        Ok(format!(
            r#"{{"actor_did":"kamn:did:agent:test","escrow_id":"{escrow_id}","state":"release-authorized","receipt_id":"escrow-transition-receipt-1","receipt_digest":"{DIGEST}","action":"escrow:release-authorize","settlement_receipt_id":"settlement-intent-1","settlement_receipt_digest":"{DIGEST}","settlement_receipt_action":"settlement:confirmed","settlement_receipt_resource_id":"{escrow_id}","settlement_receipt_state":"confirmed"}}"#
        ))
    }

    unsupported_backend_methods!();
}

fn unsupported() -> Result<String, AgentLibError> {
    Err(AgentLibError::UnsupportedOperation(
        "unused test backend method",
    ))
}

fn task_authority(task_id: &str, state: &str) -> String {
    let action = if state == "accepted" {
        "task:accept"
    } else {
        "task:complete"
    };
    format!(
        r#"{{"actor_did":"kamn:did:agent:test","task_id":"{task_id}","state":"{state}","receipt_id":"task-transition-receipt-1","receipt_digest":"{DIGEST}","action":"{action}"}}"#
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
    let digest_field = format!(r#""service_receipt_digest":"{DIGEST}""#);
    assert!(response.contains(digest_field.as_str()));
}

#[test]
fn finalized_release_wraps_distinct_settlement_receipt_authority() {
    let response = dispatch_tool_request_json(
        &AuthorityBackend,
        r#"{"id":"settled","tool":"release_escrow","escrow_id":"escrow-1","payload":"{}"}"#,
    )
    .expect("dispatch should return an envelope");
    assert!(response.contains(r#""action":"settlement:confirmed""#));
    assert!(response.contains(r#""service_receipt_id":"settlement-intent-1""#));
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
    let commitment_field = format!(r#""profile_commitment":"{DIGEST}""#);
    assert!(response.contains(commitment_field.as_str()));
}

#[test]
fn mismatched_or_malformed_mutation_authority_fails_closed() {
    for task_id in [
        "wrong-resource",
        "wrong-actor",
        "wrong-action",
        "wrong-state",
        "bad-digest",
    ] {
        let request = format!(
            r#"{{"id":"invalid","tool":"accept_task","task_id":"{task_id}","payload":"{{}}"}}"#
        );
        let response = dispatch_tool_request_json(&AuthorityBackend, request.as_str())
            .expect("dispatch should return an authority error");
        assert!(response.contains(r#""ok":false"#), "{response}");
        assert!(
            response.contains("MCP_AUTHORITY_RECEIPT_INVALID"),
            "{response}"
        );
    }
}
