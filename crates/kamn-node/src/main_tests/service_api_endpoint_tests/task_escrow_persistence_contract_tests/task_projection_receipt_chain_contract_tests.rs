use super::super::*;
use super::task_projection_contract_tests::projection_support::ProjectionCase;
use serde_json::Value;

const CREATOR: &str = "kamn:did:agent:projection-creator";
const PROVIDER: &str = "kamn:did:agent:projection-provider";
const VERIFIER: &str = "kamn:did:agent:projection-verifier";
const CHAIN_ERROR: &str = "SERVICE_RECEIPT_CHAIN_INVALID";

#[test]
fn integration_projection_commits_chain_with_actor_scoped_private_receipts() {
    let case = ProjectionCase::new("receipt-chain-privacy");
    let task_id = case.seed_transaction();

    let creator = query_value(&case, CREATOR, 4, &task_id, "participant-view");
    let provider = query_value(&case, PROVIDER, 3, &task_id, "participant-view");
    let verifier = query_value(&case, VERIFIER, 2, &task_id, "verifier-view");

    assert_shared_commitments(&creator, &provider, &verifier);
    assert_actions(&creator, &["task:create", "escrow:fund"]);
    assert_actions(&provider, &["task:accept"]);
    assert_restricted_public(&verifier);
    case.cleanup();
}

#[test]
fn integration_retry_and_restart_preserve_chain_without_duplicate_transition() {
    let case = ProjectionCase::new("receipt-chain-retry");
    let task_id = case.seed_transaction();
    let before = query_value(&case, CREATOR, 4, &task_id, "participant-view");
    let receipt_count = case.state()["task_transition_receipts"]
        .as_array()
        .expect("task receipts")
        .len();

    case.retry_task_create(5);
    let after = query_value(&case, CREATOR, 6, &task_id, "participant-view");

    assert_eq!(
        before["receipt_chain_commitment"],
        after["receipt_chain_commitment"]
    );
    assert_eq!(
        case.state()["task_transition_receipts"]
            .as_array()
            .expect("task receipts")
            .len(),
        receipt_count
    );
    case.cleanup();
}

#[test]
fn integration_projection_rejects_tampered_receipt_order_and_identity() {
    assert_tamper_rejected("receipt-order", |state| {
        state["task_transition_receipts"]
            .as_array_mut()
            .expect("task receipts")
            .reverse();
    });
    assert_tamper_rejected("receipt-duplicate", |state| {
        let receipts = task_receipts(state);
        let duplicate = receipts[0]["receipt_id"].clone();
        receipts[1]["receipt_id"] = duplicate;
    });
}

#[test]
fn integration_projection_rejects_tampered_actor_action_state_and_resource() {
    for (label, field, value) in [
        ("receipt-actor", "actor_did", CREATOR),
        ("receipt-action", "action", "task:complete"),
        ("receipt-state", "resulting_state", "completed"),
        ("receipt-resource", "task_id", "task-cross-resource"),
    ] {
        assert_tamper_rejected(label, |state| {
            task_receipts(state)[1][field] = Value::String(value.to_owned());
        });
    }
}

#[test]
fn integration_projection_rejects_missing_authorization_and_conflicting_key() {
    assert_tamper_rejected("receipt-authorization", |state| {
        state["authorization_receipts"]
            .as_array_mut()
            .expect("authorization receipts")
            .retain(|receipt| receipt["action"] != "task:accept");
    });

    let case = ProjectionCase::new("receipt-conflicting-key");
    let task_id = case.seed_completed_transaction();
    case.mutate_state(|state| {
        let receipts = task_receipts(state);
        let duplicate = receipts[1]["idempotency_key"].clone();
        receipts[2]["idempotency_key"] = duplicate;
    });
    assert_chain_error(&case.query(VERIFIER, 2, &task_id, "verifier-view"));
    case.cleanup();
}

fn assert_tamper_rejected(label: &str, tamper: impl FnOnce(&mut Value)) {
    let case = ProjectionCase::new(label);
    let task_id = case.seed_transaction();
    case.mutate_state(tamper);
    assert_chain_error(&case.query(VERIFIER, 2, &task_id, "verifier-view"));
    case.cleanup();
}

fn task_receipts(state: &mut Value) -> &mut Vec<Value> {
    state["task_transition_receipts"]
        .as_array_mut()
        .expect("task receipts")
}

fn assert_chain_error(response: &str) {
    assert!(response.contains("500 Internal Server Error"), "{response}");
    assert!(response.contains(CHAIN_ERROR), "{response}");
}

fn query_value(case: &ProjectionCase, actor: &str, nonce: u64, task: &str, view: &str) -> Value {
    let response = case.query(actor, nonce, task, view);
    assert!(response.contains("HTTP/1.1 200 OK"), "{response}");
    parse_service_api_payload(extract_http_response_body(&response)).expect("projection payload")
}

fn assert_shared_commitments(creator: &Value, provider: &Value, verifier: &Value) {
    assert_eq!(
        creator["schema_version"],
        "kamn.runtime.task-disclosure-projection.v2"
    );
    assert_eq!(
        creator["receipt_chain_commitment"],
        provider["receipt_chain_commitment"]
    );
    assert_eq!(
        creator["receipt_chain_commitment"],
        verifier["receipt_chain_commitment"]
    );
    assert_eq!(creator["public_commitment"], verifier["public_commitment"]);
    let commitment = creator["receipt_chain_commitment"]
        .as_str()
        .expect("chain commitment");
    assert!(commitment.starts_with("sha256:") && commitment.len() == 71);
}

fn assert_actions(projection: &Value, expected: &[&str]) {
    let receipts = projection["receipt_chain_receipts"]
        .as_array()
        .expect("participant receipt details");
    let actions: Vec<_> = receipts
        .iter()
        .map(|receipt| receipt["action"].as_str().unwrap())
        .collect();
    assert_eq!(actions, expected);
    for receipt in receipts {
        for private in ["actor_did", "correlation_id", "idempotency_key"] {
            assert!(receipt.get(private).is_none(), "{receipt}");
        }
    }
}

fn assert_restricted_public(projection: &Value) {
    for private in [
        "receipt_chain_receipts",
        "task_receipt_ids",
        "completion_evidence_digest",
        "actor_did",
        "correlation_id",
        "idempotency_key",
    ] {
        assert!(projection.get(private).is_none(), "{projection}");
    }
}
