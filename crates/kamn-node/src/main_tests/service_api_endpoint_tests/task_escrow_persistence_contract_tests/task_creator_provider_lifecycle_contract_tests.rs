use super::super::*;
use crate::service_api_endpoint::ServiceApiTaskTransitionBody;

#[path = "task_creator_provider_lifecycle_contract_tests/support.rs"]
mod lifecycle_support;
#[path = "task_creator_provider_lifecycle_contract_tests/payloads.rs"]
mod payloads;
use lifecycle_support::LifecycleCase;
use payloads::{assert_response, completion_body, retry_body, valid_create_body};

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

    let early = case.raw_request(
        PROVIDER,
        2,
        "POST",
        complete_path.as_str(),
        retry_body("complete-early"),
    );
    assert_response(&early, "409 Conflict", "TASK_STATE_CONFLICT");

    let accept_path = format!("/v1/tasks/{}/accept", task.task_id);
    let accepted = case.raw_request(
        PROVIDER,
        3,
        "POST",
        accept_path.as_str(),
        retry_body("accept-valid"),
    );
    assert!(accepted.contains("HTTP/1.1 200 OK"), "{accepted}");
    let missing_evidence = case.raw_request(
        PROVIDER,
        4,
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

#[test]
fn integration_task_create_retry_is_idempotent_and_conflict_safe() {
    let case = LifecycleCase::new("create-idempotency");
    case.register(PROVIDER, 1);
    let first = case.create_valid_task(1);
    let provider_did = test_service_api_sender_did(PROVIDER);
    let body = valid_create_body(provider_did.as_str());

    let retried = case.raw_request(CREATOR, 2, "POST", "/v1/tasks/create", body.as_str());
    let retry_payload: ServiceApiTaskCreateBody =
        parse_service_api_payload(extract_http_response_body(&retried)).expect("retry payload");
    assert_eq!(retry_payload.task_id, first.task_id);

    let conflict_body = body.replace("transaction-lifecycle-001", "transaction-conflict");
    let conflict = case.raw_request(
        CREATOR,
        3,
        "POST",
        "/v1/tasks/create",
        conflict_body.as_str(),
    );
    assert_response(&conflict, "409 Conflict", "TASK_IDEMPOTENCY_CONFLICT");
    assert_eq!(
        case.state()["tasks"]
            .as_object()
            .expect("tasks should be an object")
            .len(),
        1
    );
    case.cleanup();
}

#[test]
fn integration_task_transition_retry_returns_one_durable_receipt() {
    let case = LifecycleCase::new("transition-idempotency");
    case.register(PROVIDER, 1);
    let task = case.create_valid_task(1);
    let path = format!("/v1/tasks/{}/accept", task.task_id);

    let first = case.raw_request(PROVIDER, 2, "POST", path.as_str(), retry_body("accept-1"));
    let retried = case.raw_request(PROVIDER, 3, "POST", path.as_str(), retry_body("accept-1"));
    assert!(first.contains("HTTP/1.1 200 OK"), "{first}");
    assert!(retried.contains("HTTP/1.1 200 OK"), "{retried}");
    let first_payload: ServiceApiTaskTransitionBody =
        parse_service_api_payload(extract_http_response_body(&first)).expect("first payload");
    let retry_payload: ServiceApiTaskTransitionBody =
        parse_service_api_payload(extract_http_response_body(&retried)).expect("retry payload");
    assert_eq!(retry_payload, first_payload);
    let state = case.state();
    let receipts = state["task_transition_receipts"]
        .as_array()
        .expect("transition receipts should be an array");
    assert_eq!(receipts.len(), 1);
    assert_eq!(receipts[0]["action"], "task:accept");
    assert_eq!(receipts[0]["transaction_id"], "transaction-lifecycle-001");
    case.cleanup();
}

#[test]
fn integration_task_completion_evidence_and_receipts_survive_restart() {
    let case = LifecycleCase::new("restart-proof");
    case.register(PROVIDER, 1);
    let task = case.create_valid_task(1);
    let accept_path = format!("/v1/tasks/{}/accept", task.task_id);
    let complete_path = format!("/v1/tasks/{}/complete", task.task_id);
    let accepted = case.raw_request(
        PROVIDER,
        2,
        "POST",
        accept_path.as_str(),
        retry_body("accept-valid"),
    );
    assert!(accepted.contains("HTTP/1.1 200 OK"), "{accepted}");

    let completion = completion_body("complete-valid");
    let completed = case.raw_request(
        PROVIDER,
        3,
        "POST",
        complete_path.as_str(),
        completion.as_str(),
    );
    assert!(completed.contains("HTTP/1.1 200 OK"), "{completed}");
    let state = case.state();
    assert_eq!(
        state["tasks"][&task.task_id]["completion_evidence_digest"],
        "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"
    );
    assert_eq!(
        state["task_transition_receipts"]
            .as_array()
            .expect("transition receipts should be an array")
            .len(),
        2
    );
    case.cleanup();
}
