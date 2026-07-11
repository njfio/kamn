use super::super::*;
use super::support::*;

const CREATOR: &str = "kamn:did:agent:escrow-denial-creator";
const OTHER: &str = "kamn:did:agent:escrow-denial-other";

#[test]
fn integration_task_bound_escrow_rejects_non_creator_funder() {
    let case = DenialCase::new("wrong-funder");
    let task = case.accepted_task();
    let body = case.funding_body(task.task_id.as_str(), OTHER, "transaction-2", valid_terms());

    let response = case.request(OTHER, 4, body.as_str());

    assert!(response.contains("HTTP/1.1 403 Forbidden"), "{response}");
    assert!(response.contains("ESCROW_FUNDER_MISMATCH"), "{response}");
    case.cleanup();
}

#[test]
fn integration_task_bound_escrow_rejects_transaction_and_terms_mismatch() {
    let case = DenialCase::new("agreement-mismatch");
    let task = case.accepted_task();
    let transaction = case.funding_body(task.task_id.as_str(), CREATOR, "wrong", valid_terms());
    let terms = case.funding_body(
        task.task_id.as_str(),
        CREATOR,
        "transaction-2",
        wrong_terms(),
    );

    let transaction_response = case.request(CREATOR, 4, transaction.as_str());
    let terms_response = case.request(CREATOR, 5, terms.as_str());

    assert!(transaction_response.contains("ESCROW_TRANSACTION_MISMATCH"));
    assert!(terms_response.contains("ESCROW_TERMS_MISMATCH"));
    case.cleanup();
}

#[test]
fn integration_task_bound_escrow_release_enforces_authority_and_replays_receipt() {
    let case = DenialCase::new("release-authority-retry");
    let task = case.accepted_task();
    let body = case.funding_body(
        task.task_id.as_str(),
        CREATOR,
        "transaction-2",
        valid_terms(),
    );
    let funded = response_payload(&case.request(CREATOR, 4, body.as_str()));
    case.complete(task.task_id.as_str(), 5);
    let escrow_id = funded["escrow_id"].as_str().expect("escrow id");

    let denied = case.release(OTHER, 6, escrow_id, "release-retry");
    let released = response_payload(&case.release(CREATOR, 7, escrow_id, "release-retry"));
    let replayed = response_payload(&case.release(CREATOR, 8, escrow_id, "release-retry"));

    assert!(denied.contains("ESCROW_RELEASE_AUTHORITY_MISMATCH"));
    assert_eq!(replayed["receipt_id"], released["receipt_id"]);
    let state = read_state_json(case.state_file.as_path());
    assert_eq!(
        state["escrow_transition_receipts"]
            .as_array()
            .unwrap()
            .len(),
        2
    );
    case.cleanup();
}

struct DenialCase {
    _env: ServiceApiTestEnvGuards,
    _state_guard: EnvVarGuard,
    snapshot: crate::service_api_endpoint::ServiceApiSnapshot,
    state_file: std::path::PathBuf,
}

impl DenialCase {
    fn new(label: &str) -> Self {
        let env = acquire_service_api_test_env();
        let state_file = unique_named_state_file(format!("kamn-escrow-denial-{label}").as_str());
        let (_, state_guard) = set_state_file_env(state_file.as_path());
        Self {
            _env: env,
            _state_guard: state_guard,
            snapshot: build_task_escrow_snapshot("127.0.0.1:34231"),
            state_file,
        }
    }

    fn accepted_task(&self) -> crate::service_api_endpoint::ServiceApiTaskCreateBody {
        let task = create_task(
            &self.snapshot,
            reserve_loopback_addr().as_str(),
            CREATOR,
            2,
            r#"{"description":"escrow denial"}"#,
        );
        accept_task(
            &self.snapshot,
            reserve_loopback_addr().as_str(),
            CREATOR,
            3,
            task.task_id.as_str(),
        );
        task
    }

    fn funding_body(
        &self,
        task_id: &str,
        authority: &str,
        transaction: &str,
        terms: &str,
    ) -> String {
        serde_json::json!({
            "task_id": task_id,
            "transaction_id": transaction,
            "beneficiary_did": test_service_api_sender_did(CREATOR),
            "amount_lamports": 10_000,
            "network": "solana-devnet",
            "terms_digest": terms,
            "release_authority_did": test_service_api_sender_did(authority),
            "release_policy": "task-completed",
            "idempotency_key": format!("denial-{transaction}-{authority}"),
        })
        .to_string()
    }

    fn request(&self, actor: &str, nonce: u64, body: &str) -> String {
        authorized_signed_request(
            &self.snapshot,
            reserve_loopback_addr().as_str(),
            SignedRequest {
                max_requests: 1,
                method: "POST",
                path: "/v1/escrow/fund",
                caller_did: actor,
                nonce,
                body,
                extra_headers: &[],
            },
        )
    }

    fn complete(&self, task_id: &str, nonce: u64) {
        complete_task(
            &self.snapshot,
            reserve_loopback_addr().as_str(),
            CREATOR,
            nonce,
            task_id,
        );
    }

    fn release(&self, actor: &str, nonce: u64, escrow_id: &str, key: &str) -> String {
        let path = format!("/v1/escrow/{escrow_id}/release");
        let body = serde_json::json!({"idempotency_key": key}).to_string();
        authorized_signed_request(
            &self.snapshot,
            reserve_loopback_addr().as_str(),
            SignedRequest {
                max_requests: 1,
                method: "POST",
                path: path.as_str(),
                caller_did: actor,
                nonce,
                body: body.as_str(),
                extra_headers: &[],
            },
        )
    }

    fn cleanup(self) {
        let _ = fs::remove_file(self.state_file);
    }
}

fn valid_terms() -> &'static str {
    "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
}

fn wrong_terms() -> &'static str {
    "cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc"
}

fn response_payload(response: &str) -> Value {
    parse_service_api_payload(extract_http_response_body(response))
        .expect("response payload should deserialize")
}
