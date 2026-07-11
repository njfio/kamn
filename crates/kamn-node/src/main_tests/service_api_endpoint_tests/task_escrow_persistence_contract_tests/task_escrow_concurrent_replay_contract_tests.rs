use super::super::*;
use super::support::*;
use crate::service_api_endpoint::ServiceApiSnapshot;
use std::path::Path;

const ACTOR: &str = "kamn:did:agent:concurrent-replay";
const PATH: &str = "/v1/tasks/create";

#[test]
fn integration_service_api_concurrent_same_nonce_records_one_authorization_decision() {
    let _env = acquire_service_api_test_env();
    let state_file = unique_named_state_file("kamn-node-service-api-concurrent-replay");
    let (_, _state_guard) = set_state_file_env(state_file.as_path());
    let snapshot = build_task_escrow_snapshot("127.0.0.1:34211");
    let registered = register_agent_profile(
        &snapshot,
        reserve_loopback_addr().as_str(),
        ACTOR,
        1,
        r#"{"agent_type":"assistant","model_family":"pi","capabilities":["tasks:write"]}"#,
    );
    provision_grant(state_file.as_path(), registered.did.as_str());

    let responses = concurrent_requests(&snapshot, 2);

    assert_response_counts(&responses);
    assert_single_side_effect(state_file.as_path());
    let _ = fs::remove_file(state_file);
}

fn concurrent_requests(snapshot: &ServiceApiSnapshot, nonce: u64) -> Vec<String> {
    let bind_addr = reserve_loopback_addr();
    with_api_server(snapshot, bind_addr.as_str(), 2, |addr| {
        let barrier = std::sync::Arc::new(std::sync::Barrier::new(2));
        thread::scope(|scope| {
            let handles: Vec<_> = (0..2)
                .map(|_| spawn_request(scope, barrier.clone(), snapshot, addr, nonce))
                .collect();
            handles
                .into_iter()
                .map(|handle| handle.join().expect("request thread should complete"))
                .collect()
        })
    })
}

fn spawn_request<'scope>(
    scope: &'scope thread::Scope<'scope, '_>,
    barrier: std::sync::Arc<std::sync::Barrier>,
    snapshot: &'scope ServiceApiSnapshot,
    addr: &'scope str,
    nonce: u64,
) -> thread::ScopedJoinHandle<'scope, String> {
    scope.spawn(move || {
        let body = canonical_body();
        let nonce_text = nonce.to_string();
        let signature = service_api_request_signature_for_fields(
            ACTOR,
            nonce,
            state_hash(snapshot).as_str(),
            body.as_str(),
        );
        barrier.wait();
        send_http_request_with_headers(
            addr,
            "POST",
            PATH,
            body.as_str(),
            &[
                ("X-KAMN-Sender-DID", ACTOR),
                ("X-KAMN-Request-Nonce", nonce_text.as_str()),
                ("X-KAMN-Request-Signature", signature.as_str()),
                ("X-KAMN-Authz-Scope", "tasks:write"),
            ],
        )
    })
}

fn canonical_body() -> String {
    serde_json::json!({
        "provider_did": test_service_api_sender_did(ACTOR),
        "transaction_id": "transaction-concurrent-replay",
        "terms_digest": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        "idempotency_key": "concurrent-replay-create",
        "description": "grant contract",
    })
    .to_string()
}

fn provision_grant(path: &Path, actor: &str) {
    let mut state = read_state_json(path);
    state["agent_grants"] = serde_json::json!({"grant-task-create": {
        "did": actor, "resource": "transaction:new", "role": "initiator",
        "action": "task:create", "status": "active", "idempotency_key": "grant-task-create"
    }});
    fs::write(
        path,
        serde_json::to_vec_pretty(&state).expect("state should serialize"),
    )
    .expect("grant fixture should persist");
}

fn assert_response_counts(responses: &[String]) {
    assert_eq!(
        responses
            .iter()
            .filter(|r| r.contains("201 Created"))
            .count(),
        1
    );
    assert_eq!(
        responses
            .iter()
            .filter(|r| r.contains("409 Conflict"))
            .count(),
        1
    );
}

fn assert_single_side_effect(path: &Path) {
    let state = read_state_json(path);
    let receipts = state["authorization_receipts"]
        .as_array()
        .expect("authorization receipts should be an array");
    let tasks = state["tasks"]
        .as_object()
        .expect("tasks should be an object");
    assert_eq!(receipts.len(), 1);
    assert_eq!(tasks.len(), 1);
}
