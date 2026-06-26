use super::super::support::*;
use kamn_core::{DidLifecycleMutationAction, DidLifecycleMutationRequest, DidRegistryError};

#[test]
fn functional_lifecycle_chain_submission_through_kolme_adapter_returns_typed_outcome() {
    let mut registry = registry();
    let did = parse_did("kamn:did:agent:lifecycle-chain-functional");
    let mut document = document_for(&did, "claude-4");
    set_operator(&mut document, "kamn:did:human:ops-chain-functional");
    registry
        .register(did.clone(), document)
        .expect("register should succeed");

    let mut adapter = lifecycle_adapter();
    let result = registry
        .submit_lifecycle_mutation_via_chain_adapter(
            &mut adapter,
            DidLifecycleMutationRequest {
                did: did.clone(),
                actor_did: "kamn:did:human:ops-chain-functional".to_owned(),
                nonce: 1,
                action: DidLifecycleMutationAction::Revoke,
            },
        )
        .expect("lifecycle chain submission should succeed");

    assert_eq!(result.retry_class, DidSubmissionRetryClass::NewSubmission);
    assert_eq!(
        result.evidence.reason_code,
        "did_lifecycle_mutation_allowed"
    );
    assert!(matches!(
        result.outcome,
        DidChainSubmissionOutcome::Submitted(_)
    ));
}

#[test]
fn integration_lifecycle_chain_submission_allows_retry_without_reapplying_mutation() {
    let mut registry = registry();
    let did = parse_did("kamn:did:agent:lifecycle-chain-integration");
    let mut document = document_for(&did, "claude-4");
    set_operator(&mut document, "kamn:did:human:ops-chain-integration");
    registry
        .register(did.clone(), document)
        .expect("register should succeed");

    let rotated_document = {
        let mut value = document_for(&did, "gpt-5");
        set_operator(&mut value, "kamn:did:human:ops-chain-integration");
        value
    };
    let request = DidLifecycleMutationRequest {
        did: did.clone(),
        actor_did: "kamn:did:human:ops-chain-integration".to_owned(),
        nonce: 8,
        action: DidLifecycleMutationAction::Rotate {
            document: rotated_document,
        },
    };

    let mut adapter = lifecycle_adapter();
    let first = registry
        .submit_lifecycle_mutation_via_chain_adapter(&mut adapter, request.clone())
        .expect("first lifecycle submission should succeed");
    let second = registry
        .submit_lifecycle_mutation_via_chain_adapter(&mut adapter, request)
        .expect("retry lifecycle submission should succeed");

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
    assert_eq!(
        registry
            .resolve(&did)
            .expect("rotate should remain applied")
            .metadata
            .model_family,
        "gpt-5"
    );
}

#[test]
fn regression_lifecycle_chain_submission_rejects_conflicting_same_nonce_payload() {
    let mut registry = registry();
    let did = parse_did("kamn:did:agent:lifecycle-chain-regression");
    let mut document = document_for(&did, "claude-4");
    set_operator(&mut document, "kamn:did:human:ops-chain-regression");
    registry
        .register(did.clone(), document)
        .expect("register should succeed");

    let mut adapter = lifecycle_adapter();
    registry
        .submit_lifecycle_mutation_via_chain_adapter(
            &mut adapter,
            DidLifecycleMutationRequest {
                did: did.clone(),
                actor_did: "kamn:did:human:ops-chain-regression".to_owned(),
                nonce: 4,
                action: DidLifecycleMutationAction::Revoke,
            },
        )
        .expect("first lifecycle submission should succeed");

    let conflicting = registry.submit_lifecycle_mutation_via_chain_adapter(
        &mut adapter,
        DidLifecycleMutationRequest {
            did: did.clone(),
            actor_did: "kamn:did:human:ops-chain-regression".to_owned(),
            nonce: 4,
            action: DidLifecycleMutationAction::Recover {
                document: {
                    let mut recovered = document_for(&did, "gpt-5");
                    set_operator(&mut recovered, "kamn:did:human:ops-chain-regression");
                    recovered
                },
            },
        },
    );

    assert!(matches!(
        conflicting,
        Err(DidRegistryError::ConflictingSubmissionIdempotencyKey { .. })
    ));
}
