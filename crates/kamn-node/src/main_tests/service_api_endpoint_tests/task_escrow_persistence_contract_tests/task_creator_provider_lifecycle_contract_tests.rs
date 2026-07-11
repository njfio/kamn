use super::super::*;
use super::support::*;
use crate::service_api_endpoint::ServiceApiSnapshot;
use std::path::PathBuf;

#[path = "task_creator_provider_lifecycle_contract_tests/payloads.rs"]
mod payloads;
use payloads::{assert_response, retry_body, valid_create_body};

const CREATOR: &str = "kamn:did:agent:task-lifecycle-creator";
const PROVIDER: &str = "kamn:did:agent:task-lifecycle-provider";
const OUTSIDER: &str = "kamn:did:agent:task-lifecycle-outsider";

#[test]
fn integration_task_creation_rejects_incomplete_agreement() {
    let case = LifecycleCase::new("incomplete-agreement");
    let response = case.authorized_request(
        CREATOR,
        1,
        "POST",
        "/v1/tasks/create",
        r#"{"description":"missing agreement"}"#,
    );

    assert_response(&response, "400 Bad Request", "TASK_AGREEMENT_INVALID");
    case.cleanup();
}

#[test]
fn integration_task_creation_issues_provider_accept_grant() {
    let case = LifecycleCase::new("provider-grant");
    case.register(PROVIDER, 1);
    let task = case.create_valid_task(1);
    let path = format!("/v1/tasks/{}/accept", task.task_id);

    let response = case.raw_request(PROVIDER, 2, "POST", path.as_str(), retry_body("accept-1"));

    assert!(response.contains("HTTP/1.1 200 OK"), "{response}");
    case.cleanup();
}

#[test]
fn integration_task_accept_rejects_granted_wrong_provider() {
    let case = LifecycleCase::new("wrong-provider");
    case.register(PROVIDER, 1);
    let task = case.create_valid_task(1);
    let path = format!("/v1/tasks/{}/accept", task.task_id);

    let response = case.authorized_request(
        OUTSIDER,
        1,
        "POST",
        path.as_str(),
        retry_body("wrong-provider-accept"),
    );

    assert_response(&response, "403 Forbidden", "TASK_PROVIDER_MISMATCH");
    case.cleanup();
}

#[test]
fn integration_task_complete_rejects_illegal_order_and_missing_evidence() {
    let case = LifecycleCase::new("complete-contract");
    case.register(PROVIDER, 1);
    let task = case.create_valid_task(1);
    let complete_path = format!("/v1/tasks/{}/complete", task.task_id);

    let early = case.authorized_request(
        PROVIDER,
        2,
        "POST",
        complete_path.as_str(),
        retry_body("complete-early"),
    );
    assert_response(&early, "409 Conflict", "TASK_STATE_CONFLICT");

    let accept_path = format!("/v1/tasks/{}/accept", task.task_id);
    let accepted = case.authorized_request(
        PROVIDER,
        2,
        "POST",
        accept_path.as_str(),
        retry_body("accept-valid"),
    );
    assert!(accepted.contains("HTTP/1.1 200 OK"), "{accepted}");
    let missing_evidence = case.authorized_request(
        PROVIDER,
        3,
        "POST",
        complete_path.as_str(),
        retry_body("complete-missing-evidence"),
    );
    assert_response(
        &missing_evidence,
        "400 Bad Request",
        "TASK_COMPLETION_EVIDENCE_INVALID",
    );
    case.cleanup();
}

struct LifecycleCase {
    _env: ServiceApiTestEnvGuards,
    _state_guard: EnvVarGuard,
    snapshot: ServiceApiSnapshot,
    state_file: PathBuf,
}

impl LifecycleCase {
    fn new(label: &str) -> Self {
        let env = acquire_service_api_test_env();
        let state_file = unique_named_state_file(format!("kamn-task-lifecycle-{label}").as_str());
        let (_, state_guard) = set_state_file_env(state_file.as_path());
        Self {
            _env: env,
            _state_guard: state_guard,
            snapshot: build_task_escrow_snapshot("127.0.0.1:34220"),
            state_file,
        }
    }

    fn register(&self, actor: &str, nonce: u64) {
        register_agent_profile(
            &self.snapshot,
            reserve_loopback_addr().as_str(),
            actor,
            nonce,
            r#"{"agent_type":"provider","model_family":"pi","capabilities":["tasks"]}"#,
        );
    }

    fn create_valid_task(&self, nonce: u64) -> ServiceApiTaskCreateBody {
        create_task(
            &self.snapshot,
            reserve_loopback_addr().as_str(),
            CREATOR,
            nonce,
            valid_create_body(),
        )
    }

    fn authorized_request(
        &self,
        actor: &str,
        nonce: u64,
        method: &str,
        path: &str,
        body: &str,
    ) -> String {
        signed_request(&self.snapshot, actor, nonce, method, path, body, true)
    }

    fn raw_request(&self, actor: &str, nonce: u64, method: &str, path: &str, body: &str) -> String {
        signed_request(&self.snapshot, actor, nonce, method, path, body, false)
    }

    fn cleanup(self) {
        let _ = fs::remove_file(self.state_file);
    }
}

fn signed_request(
    snapshot: &ServiceApiSnapshot,
    actor: &str,
    nonce: u64,
    method: &str,
    path: &str,
    body: &str,
    provision: bool,
) -> String {
    let request = SignedRequest {
        max_requests: 1,
        method,
        path,
        caller_did: actor,
        nonce,
        body,
        extra_headers: &[("X-KAMN-Authz-Scope", "tasks:write")],
    };
    let bind_addr = reserve_loopback_addr();
    if provision {
        return authorized_signed_request(snapshot, bind_addr.as_str(), request);
    }
    raw_signed_request(snapshot, bind_addr.as_str(), request)
}
