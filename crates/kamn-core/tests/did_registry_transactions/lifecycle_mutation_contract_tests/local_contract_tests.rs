use super::super::support::*;
use kamn_core::{DidLifecycleMutationAction, DidLifecycleMutationRequest, DidRegistryError};

#[test]
fn unit_lifecycle_mutation_nonce_guards_emit_deterministic_reason_codes() {
    let mut registry = registry();
    let did = parse_did("kamn:did:agent:lifecycle-unit-1");
    let mut initial_document = document_for(&did, "claude-4");
    set_operator(&mut initial_document, "kamn:did:human:ops-1");
    registry
        .register(did.clone(), initial_document.clone())
        .expect("register should succeed");

    let zero_nonce_error = registry
        .apply_lifecycle_mutation(DidLifecycleMutationRequest {
            did: did.clone(),
            actor_did: "kamn:did:human:ops-1".to_owned(),
            nonce: 0,
            action: DidLifecycleMutationAction::Revoke,
        })
        .expect_err("zero nonce mutation should fail");
    assert_eq!(
        zero_nonce_error.reason_code(),
        "did_lifecycle_mutation_nonce_invalid"
    );

    apply_mutation(
        &mut registry,
        &did,
        "kamn:did:human:ops-1",
        1,
        DidLifecycleMutationAction::Rotate {
            document: initial_document,
        },
    );

    let replay_error = registry
        .apply_lifecycle_mutation(DidLifecycleMutationRequest {
            did: did.clone(),
            actor_did: "kamn:did:human:ops-1".to_owned(),
            nonce: 1,
            action: DidLifecycleMutationAction::Revoke,
        })
        .expect_err("replayed nonce should fail");
    assert_replay_conflict(replay_error);
}

#[test]
fn functional_lifecycle_rotate_mutation_updates_document_and_emits_allowed_reason_code() {
    let mut registry = registry();
    let did = parse_did("kamn:did:agent:lifecycle-func-1");
    let mut initial_document = document_for(&did, "claude-4");
    set_operator(&mut initial_document, "kamn:did:human:ops-2");
    registry
        .register(did.clone(), initial_document)
        .expect("register should succeed");

    let mut rotated_document = document_for(&did, "gpt-5");
    set_operator(&mut rotated_document, "kamn:did:human:ops-2");
    let evidence = apply_mutation(
        &mut registry,
        &did,
        "kamn:did:human:ops-2",
        7,
        DidLifecycleMutationAction::Rotate {
            document: rotated_document,
        },
    );

    assert_eq!(evidence.reason_code, "did_lifecycle_mutation_allowed");
    assert_eq!(
        registry
            .resolve(&did)
            .expect("resolve should succeed")
            .metadata
            .model_family,
        "gpt-5"
    );
}

#[test]
fn integration_lifecycle_revoke_then_recover_restores_active_resolution() {
    let mut registry = registry();
    let did = parse_did("kamn:did:agent:lifecycle-int-1");
    let mut initial_document = document_for(&did, "claude-4");
    set_operator(&mut initial_document, "kamn:did:human:ops-3");
    registry
        .register(did.clone(), initial_document)
        .expect("register should succeed");

    apply_mutation(
        &mut registry,
        &did,
        "kamn:did:human:ops-3",
        1,
        DidLifecycleMutationAction::Revoke,
    );
    assert_eq!(
        registry.resolve(&did),
        Err(DidRegistryError::Revoked(did.as_str().to_owned()))
    );

    let mut recovered_document = document_for(&did, "claude-4.1");
    set_operator(&mut recovered_document, "kamn:did:human:ops-3");
    let recovery_evidence = apply_mutation(
        &mut registry,
        &did,
        "kamn:did:human:ops-3",
        2,
        DidLifecycleMutationAction::Recover {
            document: recovered_document,
        },
    );

    assert_eq!(
        recovery_evidence.reason_code,
        "did_lifecycle_mutation_allowed"
    );
    assert_eq!(
        registry
            .resolve(&did)
            .expect("resolve should recover")
            .metadata
            .model_family,
        "claude-4.1"
    );
}

#[test]
fn regression_lifecycle_replayed_or_unauthorized_mutation_fails_closed() {
    let mut registry = registry();
    let did = parse_did("kamn:did:agent:lifecycle-reg-1");
    let mut document = document_for(&did, "claude-4");
    set_operator(&mut document, "kamn:did:human:ops-4");
    registry
        .register(did.clone(), document)
        .expect("register should succeed");

    let unauthorized_error = registry
        .apply_lifecycle_mutation(DidLifecycleMutationRequest {
            did: did.clone(),
            actor_did: "kamn:did:human:intruder-4".to_owned(),
            nonce: 1,
            action: DidLifecycleMutationAction::Revoke,
        })
        .expect_err("unauthorized mutation must fail");
    assert_eq!(
        unauthorized_error.reason_code(),
        "did_lifecycle_mutation_unauthorized_actor"
    );

    apply_mutation(
        &mut registry,
        &did,
        "kamn:did:human:ops-4",
        1,
        DidLifecycleMutationAction::Revoke,
    );

    let replay_error = registry
        .apply_lifecycle_mutation(DidLifecycleMutationRequest {
            did: did.clone(),
            actor_did: "kamn:did:human:ops-4".to_owned(),
            nonce: 1,
            action: DidLifecycleMutationAction::Recover {
                document: {
                    let mut recovered = document_for(&did, "claude-4.2");
                    set_operator(&mut recovered, "kamn:did:human:ops-4");
                    recovered
                },
            },
        })
        .expect_err("replayed mutation nonce must fail");
    assert_replay_conflict(replay_error);
}
