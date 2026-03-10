use super::support::*;
use kamn_core::{DidRegistryError, DidSubmissionFinalityStatus};

#[test]
fn retry_classification_is_deterministic_for_duplicate_submission() {
    let mut registry = registry();
    let did = parse_did("kamn:did:agent:agent-7");
    let document = document_for(&did, "claude-4");

    let first = registry.register_with_retry_guard(did.clone(), document.clone()).expect("initial submission should succeed");
    assert_eq!(first, DidSubmissionRetryClass::NewSubmission);

    let duplicate_before_finality = registry.register_with_retry_guard(did.clone(), document.clone()).expect("duplicate submission should classify retry state");
    assert_eq!(duplicate_before_finality, DidSubmissionRetryClass::RetryableInFlight);

    confirmed_register_finality(&mut registry, &did, &document, 7, "receipt-7");

    let duplicate_after_finality = registry.register_with_retry_guard(did.clone(), document).expect("duplicate submission should no-op once finalized");
    assert_eq!(duplicate_after_finality, DidSubmissionRetryClass::FinalizedNoRetry);
}

#[test]
fn integration_register_retry_and_finality_boundary_is_idempotent() {
    let mut registry = registry();
    let did = parse_did("kamn:did:agent:agent-8");
    let document = document_for(&did, "gpt-5");

    registry.register_with_retry_guard(did.clone(), document.clone()).expect("first submit should succeed");
    confirmed_register_finality(&mut registry, &did, &document, 9, "receipt-9");
    confirmed_register_finality(&mut registry, &did, &document, 9, "receipt-9");

    assert_eq!(registry.register_with_retry_guard(did.clone(), document), Ok(DidSubmissionRetryClass::FinalizedNoRetry));
    assert_eq!(registry.resolve(&did).expect("did should stay resolvable").id, did.as_str().to_owned());
}

#[test]
fn regression_register_finality_rejects_stale_or_conflicting_updates() {
    let mut registry = registry();
    let did = parse_did("kamn:did:agent:agent-9");
    let document = document_for(&did, "gpt-5");
    let idempotency_key = registry.idempotency_key_for_register(&did, &document).expect("idempotency key should derive");

    registry.register_with_retry_guard(did.clone(), document).expect("submit should succeed");
    registry.record_register_finality(&did, &idempotency_key, 11, DidSubmissionFinalityStatus::Confirmed, "receipt-11").expect("initial finality should succeed");

    assert_eq!(registry.record_register_finality(&did, &idempotency_key, 10, DidSubmissionFinalityStatus::Confirmed, "receipt-10"), Err(DidRegistryError::StaleFinalityUpdate { did: did.as_str().to_owned(), current_sequence: 11, attempted_sequence: 10 }));
    assert_eq!(registry.record_register_finality(&did, &idempotency_key, 11, DidSubmissionFinalityStatus::Rejected, "receipt-11-conflict"), Err(DidRegistryError::ConflictingFinalityUpdate { did: did.as_str().to_owned(), sequence: 11 }));
}
