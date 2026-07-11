use super::super::*;
use super::support::*;

const ACTOR: &str = "kamn:did:agent:task-bound-escrow-creator";

#[test]
fn integration_task_bound_escrow_funding_persists_canonical_agreement() {
    let case = EscrowCase::new("canonical-funding");
    let task = case.accepted_task();
    let body = case.funding_body(task.task_id.as_str(), "escrow-fund-1");

    let response = case.request(4, "POST", "/v1/escrow/fund", body.as_str());
    assert!(response.contains("HTTP/1.1 200 OK"), "{response}");
    let payload: Value = parse_service_api_payload(extract_http_response_body(&response))
        .expect("escrow payload should deserialize");
    assert_eq!(payload["transaction_id"], "transaction-2");
    assert_eq!(payload["network"], "solana-devnet");
    assert_eq!(payload["amount_lamports"], 10_000);
    assert_eq!(payload["claim_scope"], "local-only");
    case.cleanup();
}

#[test]
fn integration_task_bound_escrow_rejects_release_before_task_completion() {
    let case = EscrowCase::new("early-release");
    let task = case.accepted_task();
    let body = case.funding_body(task.task_id.as_str(), "escrow-fund-early");
    let funded = case.request(4, "POST", "/v1/escrow/fund", body.as_str());
    let payload: Value = parse_service_api_payload(extract_http_response_body(&funded))
        .expect("escrow payload should deserialize");
    let escrow_id = payload["escrow_id"]
        .as_str()
        .expect("escrow id should be a string");
    let path = format!("/v1/escrow/{escrow_id}/release");

    let released = case.request(
        5,
        "POST",
        path.as_str(),
        r#"{"idempotency_key":"escrow-release-early"}"#,
    );
    assert!(released.contains("HTTP/1.1 409 Conflict"), "{released}");
    assert!(
        released.contains("ESCROW_RELEASE_NOT_ELIGIBLE"),
        "{released}"
    );
    case.cleanup();
}

struct EscrowCase {
    _env: ServiceApiTestEnvGuards,
    _state_guard: EnvVarGuard,
    snapshot: crate::service_api_endpoint::ServiceApiSnapshot,
    state_file: std::path::PathBuf,
}

impl EscrowCase {
    fn new(label: &str) -> Self {
        let env = acquire_service_api_test_env();
        let state_file =
            unique_named_state_file(format!("kamn-task-bound-escrow-{label}").as_str());
        let (_, state_guard) = set_state_file_env(state_file.as_path());
        Self {
            _env: env,
            _state_guard: state_guard,
            snapshot: build_task_escrow_snapshot("127.0.0.1:34230"),
            state_file,
        }
    }

    fn accepted_task(&self) -> crate::service_api_endpoint::ServiceApiTaskCreateBody {
        let task = create_task(
            &self.snapshot,
            reserve_loopback_addr().as_str(),
            ACTOR,
            2,
            r#"{"description":"task-bound escrow"}"#,
        );
        accept_task(
            &self.snapshot,
            reserve_loopback_addr().as_str(),
            ACTOR,
            3,
            task.task_id.as_str(),
        );
        task
    }

    fn funding_body(&self, task_id: &str, key: &str) -> String {
        serde_json::json!({
            "task_id": task_id,
            "transaction_id": "transaction-2",
            "beneficiary_did": test_service_api_sender_did(ACTOR),
            "amount_lamports": 10_000,
            "network": "solana-devnet",
            "terms_digest": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            "release_authority_did": test_service_api_sender_did(ACTOR),
            "release_policy": "task-completed",
            "idempotency_key": key,
        })
        .to_string()
    }

    fn request(&self, nonce: u64, method: &str, path: &str, body: &str) -> String {
        authorized_signed_request(
            &self.snapshot,
            reserve_loopback_addr().as_str(),
            SignedRequest {
                max_requests: 1,
                method,
                path,
                caller_did: ACTOR,
                nonce,
                body,
                extra_headers: &[],
            },
        )
    }

    fn cleanup(self) {
        let _ = fs::remove_file(self.state_file);
    }
}
