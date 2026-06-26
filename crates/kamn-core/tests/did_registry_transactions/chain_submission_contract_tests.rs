use super::support::*;
use kamn_core::DidRegistryError;

#[test]
fn functional_chain_submission_adapter_returns_typed_submitted_outcome() {
    let mut registry = registry();
    let mut adapter = registration_adapter();
    let did = parse_did("kamn:did:agent:agent-10");
    let document = document_for(&did, "claude-4");

    let result = registry
        .submit_registration_via_chain_adapter(&mut adapter, did.clone(), document)
        .expect("submit should succeed");
    assert_eq!(result.retry_class, DidSubmissionRetryClass::NewSubmission);
    assert!(matches!(
        result.outcome,
        DidChainSubmissionOutcome::Submitted(_)
    ));
}

#[test]
fn integration_chain_submission_adapter_deduplicates_retry_outcomes() {
    let mut registry = registry();
    let mut adapter = registration_adapter();
    let did = parse_did("kamn:did:agent:agent-11");
    let document = document_for(&did, "gpt-5");

    let first = registry
        .submit_registration_via_chain_adapter(&mut adapter, did.clone(), document.clone())
        .expect("first submit should succeed");
    let second = registry
        .submit_registration_via_chain_adapter(&mut adapter, did.clone(), document)
        .expect("retry submit should succeed");

    assert_eq!(first.retry_class, DidSubmissionRetryClass::NewSubmission);
    assert_eq!(
        second.retry_class,
        DidSubmissionRetryClass::RetryableInFlight
    );
    assert!(matches!(
        first.outcome,
        DidChainSubmissionOutcome::Submitted(_)
    ));
    assert!(matches!(
        second.outcome,
        DidChainSubmissionOutcome::Duplicate(_)
    ));
}

#[test]
fn regression_registration_chain_submission_rejects_malformed_document_payload() {
    let mut registry = registry();
    let mut adapter = registration_adapter();
    let did = parse_did("kamn:did:agent:agent-13");
    let mut malformed_document = document_for(&did, "gpt-5");
    malformed_document.id = "kamn:did:agent:agent-13-malformed".to_owned();

    let error = registry
        .submit_registration_via_chain_adapter(&mut adapter, did, malformed_document)
        .expect_err("malformed registration payload must fail closed");
    assert_eq!(error.reason_code(), "did_registry_document_did_mismatch");
    assert!(matches!(
        error,
        DidRegistryError::DocumentDidMismatch { .. }
    ));
}

#[test]
fn regression_registration_chain_submission_rejects_duplicate_registration_payload_drift() {
    let mut registry = registry();
    let mut adapter = registration_adapter();
    let did = parse_did("kamn:did:agent:agent-14");
    let initial_document = document_for(&did, "claude-4");

    let first = registry
        .submit_registration_via_chain_adapter(&mut adapter, did.clone(), initial_document)
        .expect("first registration submission should succeed");
    assert_eq!(first.retry_class, DidSubmissionRetryClass::NewSubmission);
    assert!(matches!(
        first.outcome,
        DidChainSubmissionOutcome::Submitted(_)
    ));

    let drifted_document = document_for(&did, "gpt-5");
    let error = registry
        .submit_registration_via_chain_adapter(&mut adapter, did, drifted_document)
        .expect_err("drifted duplicate registration payload must fail closed");
    assert_eq!(error.reason_code(), "did_registry_submission_key_conflict");
    assert!(matches!(
        error,
        DidRegistryError::ConflictingSubmissionIdempotencyKey { .. }
    ));
}

#[test]
fn regression_chain_submission_adapter_exposes_rejected_outcome_without_panicking() {
    let mut registry = registry();
    let mut adapter = registration_adapter();
    let did = parse_did("kamn:did:agent:agent-12");
    let document = document_for(&did, "gpt-5");
    let idempotency_key = registry
        .idempotency_key_for_register(&did, &document)
        .expect("idempotency key should derive");

    adapter.reject_idempotency_key(&idempotency_key, "simulated-ledger-reject");
    let rejected = registry
        .submit_registration_via_chain_adapter(&mut adapter, did.clone(), document)
        .expect("submission result should remain typed");

    assert!(matches!(
        rejected.outcome,
        DidChainSubmissionOutcome::Rejected { .. }
    ));
}
