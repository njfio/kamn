use super::super::*;
use super::support::*;
use serde_json::Value;

const CREATOR: &str = "kamn:did:agent:projection-creator";
const PROVIDER: &str = "kamn:did:agent:projection-provider";
const VERIFIER: &str = "kamn:did:agent:projection-verifier";
const OUTSIDER: &str = "kamn:did:agent:projection-outsider";

#[test]
fn integration_participants_receive_private_runtime_projection() {
    let case = ProjectionCase::new("participant-private");
    let task_id = case.seed_transaction();

    let creator = case.query(CREATOR, 4, &task_id, "participant-view");
    let provider = case.query(PROVIDER, 3, &task_id, "participant-view");

    assert_ok(&creator);
    assert_ok(&provider);
    let creator = payload(&creator);
    let provider = payload(&provider);
    assert_eq!(creator["view_scope"], "participant-private");
    assert_eq!(creator["participant_role"], "creator");
    assert_eq!(provider["participant_role"], "provider");
    assert!(creator["task_receipt_ids"].is_array());
    assert_eq!(creator["public_commitment"], provider["public_commitment"]);
    case.cleanup();
}

#[test]
fn integration_verifier_projection_is_allowlisted_and_matches_public_commitment() {
    let case = ProjectionCase::new("verifier-allowlist");
    let task_id = case.seed_transaction();

    let participant = payload(&case.query(CREATOR, 4, &task_id, "participant-view"));
    let verifier = payload(&case.query(VERIFIER, 2, &task_id, "verifier-view"));

    assert_eq!(verifier["view_scope"], "restricted-public");
    assert_eq!(
        verifier["public_commitment"],
        participant["public_commitment"]
    );
    assert_eq!(verifier["task_id"], participant["task_id"]);
    assert_eq!(verifier["escrow_id"], participant["escrow_id"]);
    assert!(verifier.get("task_receipt_ids").is_none());
    assert!(verifier.get("completion_evidence_digest").is_none());
    case.cleanup();
}

#[test]
fn integration_unrelated_agent_cannot_retrieve_participant_projection() {
    let case = ProjectionCase::new("participant-denial");
    let task_id = case.seed_transaction();

    let response = case.query(OUTSIDER, 2, &task_id, "participant-view");

    assert!(response.contains("HTTP/1.1 403 Forbidden"), "{response}");
    assert!(
        response.contains("TASK_PARTICIPANT_VIEW_FORBIDDEN"),
        "{response}"
    );
    case.cleanup();
}

struct ProjectionCase {
    _env: ServiceApiTestEnvGuards,
    _state_guard: EnvVarGuard,
    snapshot: crate::service_api_endpoint::ServiceApiSnapshot,
    state_file: std::path::PathBuf,
}

impl ProjectionCase {
    fn new(label: &str) -> Self {
        let env = acquire_service_api_test_env();
        let state_file = unique_named_state_file(format!("kamn-projection-{label}").as_str());
        let (_, state_guard) = set_state_file_env(state_file.as_path());
        Self {
            _env: env,
            _state_guard: state_guard,
            snapshot: build_task_escrow_snapshot("127.0.0.1:34250"),
            state_file,
        }
    }

    fn seed_transaction(&self) -> String {
        self.register(CREATOR, 1);
        self.register(PROVIDER, 1);
        self.register(VERIFIER, 1);
        self.register(OUTSIDER, 1);
        let provider = test_service_api_sender_did(PROVIDER);
        let body = serde_json::json!({
            "provider_did": provider,
            "transaction_id": "transaction-projection-001",
            "terms_digest": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            "idempotency_key": "create-projection-001",
            "description": "runtime disclosure projection",
        });
        let task = create_task(
            &self.snapshot,
            reserve_loopback_addr().as_str(),
            CREATOR,
            2,
            body.to_string().as_str(),
        );
        accept_task(
            &self.snapshot,
            reserve_loopback_addr().as_str(),
            PROVIDER,
            2,
            task.task_id.as_str(),
        );
        let escrow = serde_json::json!({"task_id": task.task_id, "amount_lamports": 10_000});
        fund_escrow(
            &self.snapshot,
            reserve_loopback_addr().as_str(),
            CREATOR,
            3,
            escrow.to_string().as_str(),
        );
        task.task_id
    }

    fn register(&self, actor: &str, nonce: u64) {
        register_agent_profile(
            &self.snapshot,
            reserve_loopback_addr().as_str(),
            actor,
            nonce,
            r#"{"agent_type":"agent","model_family":"pi","capabilities":["tasks"]}"#,
        );
    }

    fn query(&self, actor: &str, nonce: u64, task_id: &str, view: &str) -> String {
        let path = format!("/v1/tasks/{task_id}/{view}");
        authorized_signed_request(
            &self.snapshot,
            reserve_loopback_addr().as_str(),
            SignedRequest {
                max_requests: 1,
                method: "GET",
                path: path.as_str(),
                caller_did: actor,
                nonce,
                body: "",
                extra_headers: &[("X-KAMN-Authz-Scope", "tasks:read")],
            },
        )
    }

    fn cleanup(self) {
        let _ = fs::remove_file(self.state_file);
    }
}

fn payload(response: &str) -> Value {
    assert_ok(response);
    parse_service_api_payload(extract_http_response_body(response)).expect("projection payload")
}

fn assert_ok(response: &str) {
    assert!(response.contains("HTTP/1.1 200 OK"), "{response}");
}
