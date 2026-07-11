use super::super::*;
use super::support::{
    build_task_escrow_snapshot, raw_signed_request, read_state_json, register_agent_profile,
    set_state_file_env, unique_named_state_file, SignedRequest,
};
use crate::service_api_endpoint::ServiceApiSnapshot;
use std::path::{Path, PathBuf};

const TASK_CREATE_PATH: &str = "/v1/tasks/create";

#[test]
fn integration_service_api_rejects_signed_unregistered_task_creator() {
    let case = AuthorizationCase::new("unregistered");
    let response = case.create_task("kamn:did:agent:unregistered", 1);

    assert_forbidden(&response, "AGENT_NOT_REGISTERED");
    assert!(read_state_json(&case.state_file)["tasks"]
        .as_object()
        .is_some_and(|v| v.is_empty()));
    case.cleanup();
}

#[test]
fn integration_service_api_rejects_self_asserted_scope_without_grant() {
    let case = AuthorizationCase::new("scope-without-grant");
    let actor = "kamn:did:agent:scope-without-grant";
    let registered_did = case.register(actor, 1);

    let response = case.create_task(actor, 2);

    assert_forbidden(&response, "ACTION_NOT_GRANTED");
    assert_secret_free_denial_receipt(&case.state_file, registered_did.as_str());
    case.cleanup();
}

#[test]
fn integration_service_api_persists_active_grant_and_allow_receipt_across_restart() {
    let case = AuthorizationCase::new("active-grant-restart");
    let actor = "kamn:did:agent:active-grant";
    let registered_did = case.register(actor, 1);
    provision_grant(&case.state_file, registered_did.as_str(), "active");

    let response = case.create_task(actor, 2);

    assert!(response.contains("HTTP/1.1 201 Created"), "{response}");
    assert_decision_receipt(
        &case.state_file,
        registered_did.as_str(),
        "allow",
        "AUTHORIZED",
    );
    case.cleanup();
}

#[test]
fn integration_service_api_persists_revoked_grant_denial_across_restart() {
    let case = AuthorizationCase::new("revoked-grant-restart");
    let actor = "kamn:did:agent:revoked-grant";
    let registered_did = case.register(actor, 1);
    provision_grant(&case.state_file, registered_did.as_str(), "revoked");

    let response = case.create_task(actor, 2);

    assert_forbidden(&response, "ACTION_NOT_GRANTED");
    assert_decision_receipt(
        &case.state_file,
        registered_did.as_str(),
        "deny",
        "ACTION_NOT_GRANTED",
    );
    case.cleanup();
}

#[test]
fn integration_service_api_denial_does_not_consume_request_nonce() {
    let case = AuthorizationCase::new("deny-nonce-retry");
    let actor = "kamn:did:agent:deny-nonce-retry";
    let registered_did = case.register(actor, 1);

    let denied = case.create_task(actor, 2);
    assert_forbidden(&denied, "ACTION_NOT_GRANTED");
    provision_grant(&case.state_file, registered_did.as_str(), "active");

    let retried = case.create_task(actor, 2);
    assert!(retried.contains("HTTP/1.1 201 Created"), "{retried}");
    case.cleanup();
}

struct AuthorizationCase {
    _env: ServiceApiTestEnvGuards,
    _state_file_guard: EnvVarGuard,
    snapshot: ServiceApiSnapshot,
    state_file: PathBuf,
}

impl AuthorizationCase {
    fn new(label: &str) -> Self {
        let env = acquire_service_api_test_env();
        let state_file =
            unique_named_state_file(format!("kamn-node-service-api-authz-{label}").as_str());
        let (_, state_file_guard) = set_state_file_env(state_file.as_path());
        Self {
            _env: env,
            _state_file_guard: state_file_guard,
            snapshot: build_task_escrow_snapshot("127.0.0.1:34210"),
            state_file,
        }
    }

    fn register(&self, actor: &str, nonce: u64) -> String {
        register_agent_profile(
            &self.snapshot,
            reserve_loopback_addr().as_str(),
            actor,
            nonce,
            r#"{"agent_type":"assistant","model_family":"pi","capabilities":["tasks:write"]}"#,
        )
        .did
    }

    fn create_task(&self, actor: &str, nonce: u64) -> String {
        let provider_did = test_service_api_sender_did(actor);
        let body = serde_json::json!({
            "provider_did": provider_did,
            "transaction_id": "transaction-authorization-contract",
            "terms_digest": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            "idempotency_key": "authorization-contract-create",
            "description": "grant contract",
        })
        .to_string();
        raw_signed_request(
            &self.snapshot,
            reserve_loopback_addr().as_str(),
            SignedRequest {
                max_requests: 1,
                method: "POST",
                path: TASK_CREATE_PATH,
                caller_did: actor,
                nonce,
                body: body.as_str(),
                extra_headers: &[("X-KAMN-Authz-Scope", "tasks:write")],
            },
        )
    }

    fn cleanup(self) {
        let _ = fs::remove_file(self.state_file);
    }
}

fn provision_grant(path: &Path, actor: &str, status: &str) {
    let mut state = read_state_json(path);
    state["agent_grants"] = serde_json::json!({
        "grant-task-create": {
            "did": actor,
            "resource": "transaction:new",
            "role": "initiator",
            "action": "task:create",
            "status": status,
            "idempotency_key": "grant-task-create"
        }
    });
    fs::write(
        path,
        serde_json::to_vec_pretty(&state).expect("state should serialize"),
    )
    .expect("grant fixture should persist");
}

fn assert_forbidden(response: &str, reason_code: &str) {
    assert!(response.contains("HTTP/1.1 403 Forbidden"), "{response}");
    assert!(response.contains(reason_code), "{response}");
}

fn assert_secret_free_denial_receipt(path: &Path, actor: &str) {
    assert_decision_receipt(path, actor, "deny", "ACTION_NOT_GRANTED");
    let receipt = read_state_json(path)["authorization_receipts"][0].clone();
    let rendered = receipt.to_string().to_ascii_lowercase();
    for forbidden in [
        "signature",
        "public_key",
        "nonce",
        "x-kamn-authz-scope",
        "description",
    ] {
        assert!(
            !rendered.contains(forbidden),
            "receipt leaked {forbidden}: {receipt}"
        );
    }
}

fn assert_decision_receipt(path: &Path, actor: &str, decision: &str, reason: &str) {
    let state = read_state_json(path);
    let receipt = &state["authorization_receipts"][0];
    assert_eq!(receipt["actor_did"], actor);
    assert_eq!(receipt["resource"], "transaction:new");
    assert_eq!(receipt["action"], "task:create");
    assert_eq!(receipt["role"], "initiator");
    assert_eq!(receipt["decision"], decision);
    assert_eq!(receipt["reason_code"], reason);
    assert!(receipt["correlation_id"]
        .as_str()
        .is_some_and(|v| !v.is_empty()));
}
