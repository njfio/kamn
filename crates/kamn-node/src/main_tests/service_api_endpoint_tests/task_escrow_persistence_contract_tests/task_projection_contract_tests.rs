use super::super::*;
use serde_json::Value;

#[path = "task_projection_contract_tests/support.rs"]
mod projection_support;
use projection_support::ProjectionCase;

pub(super) const CREATOR: &str = "kamn:did:agent:projection-creator";
pub(super) const PROVIDER: &str = "kamn:did:agent:projection-provider";
pub(super) const VERIFIER: &str = "kamn:did:agent:projection-verifier";
pub(super) const OUTSIDER: &str = "kamn:did:agent:projection-outsider";
const UNREGISTERED: &str = "kamn:did:agent:projection-unregistered";

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

#[test]
fn integration_unregistered_agent_cannot_retrieve_verifier_projection() {
    let case = ProjectionCase::new("unregistered-verifier");
    let task_id = case.seed_transaction();

    let response = case.query(UNREGISTERED, 1, &task_id, "verifier-view");

    assert!(response.contains("HTTP/1.1 403 Forbidden"), "{response}");
    assert!(response.contains("AGENT_NOT_REGISTERED"), "{response}");
    case.cleanup();
}

#[test]
fn integration_projection_requires_bound_escrow() {
    let case = ProjectionCase::new("missing-escrow");
    let task_id = case.seed_task();

    let response = case.query(VERIFIER, 2, &task_id, "verifier-view");

    assert!(response.contains("HTTP/1.1 409 Conflict"), "{response}");
    assert!(
        response.contains("TASK_ESCROW_BINDING_MISSING"),
        "{response}"
    );
    case.cleanup();
}

#[test]
fn integration_projection_fails_closed_for_inconsistent_durable_state() {
    let case = ProjectionCase::new("inconsistent-state");
    let task_id = case.seed_transaction();
    case.replace_escrow_transaction_id("transaction-tampered");

    let response = case.query(VERIFIER, 2, &task_id, "verifier-view");

    assert!(
        response.contains("HTTP/1.1 500 Internal Server Error"),
        "{response}"
    );
    assert!(
        response.contains("TRANSACTION_PROJECTION_INCONSISTENT"),
        "{response}"
    );
    case.cleanup();
}

fn payload(response: &str) -> Value {
    assert_ok(response);
    parse_service_api_payload(extract_http_response_body(response)).expect("projection payload")
}

fn assert_ok(response: &str) {
    assert!(response.contains("HTTP/1.1 200 OK"), "{response}");
}
