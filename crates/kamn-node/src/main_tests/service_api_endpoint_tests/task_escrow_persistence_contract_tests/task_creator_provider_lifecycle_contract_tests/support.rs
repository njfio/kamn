use super::super::super::*;
use super::super::support::{
    authorized_signed_request, build_task_escrow_snapshot, create_task, raw_signed_request,
    read_state_json, register_agent_profile, set_state_file_env, unique_named_state_file,
    SignedRequest,
};
use super::payloads::valid_create_body;
use super::{CREATOR, PROVIDER};
use crate::service_api_endpoint::{ServiceApiSnapshot, ServiceApiTaskCreateBody};
use std::path::PathBuf;

pub(super) struct LifecycleCase {
    _env: ServiceApiTestEnvGuards,
    _state_guard: EnvVarGuard,
    snapshot: ServiceApiSnapshot,
    state_file: PathBuf,
}

impl LifecycleCase {
    pub(super) fn new(label: &str) -> Self {
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

    pub(super) fn register(&self, actor: &str, nonce: u64) {
        register_agent_profile(
            &self.snapshot,
            reserve_loopback_addr().as_str(),
            actor,
            nonce,
            r#"{"agent_type":"provider","model_family":"pi","capabilities":["tasks"]}"#,
        );
    }

    pub(super) fn create_valid_task(&self, nonce: u64) -> ServiceApiTaskCreateBody {
        let provider_did = test_service_api_sender_did(PROVIDER);
        let body = valid_create_body(provider_did.as_str());
        create_task(
            &self.snapshot,
            reserve_loopback_addr().as_str(),
            CREATOR,
            nonce,
            body.as_str(),
        )
    }

    pub(super) fn authorized_request(
        &self,
        actor: &str,
        nonce: u64,
        method: &str,
        path: &str,
        body: &str,
    ) -> String {
        signed_request(&self.snapshot, actor, nonce, method, path, body, true)
    }

    pub(super) fn raw_request(
        &self,
        actor: &str,
        nonce: u64,
        method: &str,
        path: &str,
        body: &str,
    ) -> String {
        signed_request(&self.snapshot, actor, nonce, method, path, body, false)
    }

    pub(super) fn state(&self) -> serde_json::Value {
        read_state_json(self.state_file.as_path())
    }

    pub(super) fn cleanup(self) {
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

#[test]
fn integration_task_create_retry_preserves_service_receipt_authority() {
    let case = LifecycleCase::new("create-receipt-authority");
    case.register(PROVIDER, 1);
    let first = case.create_valid_task(1);
    let retried = case.create_valid_task(2);

    assert_eq!(retried.receipt_id, first.receipt_id);
    assert_eq!(retried.receipt_digest, first.receipt_digest);
    assert!(first.receipt_id.starts_with("task-transition-receipt-"));
    assert!(first.receipt_digest.starts_with("sha256:"));
    assert_eq!(
        case.state()["task_transition_receipts"]
            .as_array()
            .unwrap()
            .len(),
        1
    );
    case.cleanup();
}
