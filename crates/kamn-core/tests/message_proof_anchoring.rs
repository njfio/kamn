use kamn_core::{
    InMemoryKolmeRuntimeCommitClient, InMemoryMessageProofChainAdapter,
    KolmeMessageProofChainAdapter, MessageLifecycleStore, MessageProofAnchorRequest,
    MessageProofAnchorRetryClass, MessageProofAnchorSubmissionOutcome, MessageProofAnchoringError,
    MessageProofAnchoringService, MessageStatus,
};
use std::time::Instant;

fn register_and_advance_broadcast(store: &mut MessageLifecycleStore, message_id: &str) {
    store
        .register(
            message_id,
            "kamn:did:agent:sender-anchor-1",
            vec![
                "kamn:did:agent:recipient-anchor-1".to_owned(),
                "kamn:did:agent:recipient-anchor-2".to_owned(),
            ],
            "2026-02-09T00:10:00.000Z",
            "2026-02-09T00:40:00.000Z",
        )
        .expect("register should succeed");
    store
        .transition(message_id, MessageStatus::Signed)
        .expect("created->signed should succeed");
    store
        .transition(message_id, MessageStatus::Broadcast)
        .expect("signed->broadcast should succeed");
}

#[test]
fn functional_anchor_submission_advances_broadcast_to_included_with_typed_outcome() {
    let mut lifecycle = MessageLifecycleStore::new();
    let message_id = "urn:uuid:msg-anchor-functional";
    register_and_advance_broadcast(&mut lifecycle, message_id);

    let mut anchoring = MessageProofAnchoringService::new();
    let client = InMemoryKolmeRuntimeCommitClient::new("kolme-live").expect("client should build");
    let mut adapter = KolmeMessageProofChainAdapter::new(client, "message-anchor-state")
        .expect("adapter should build");

    let result = anchoring
        .anchor_message_proof_via_chain_adapter(
            &mut lifecycle,
            &mut adapter,
            MessageProofAnchorRequest {
                message_id: message_id.to_owned(),
                actor_did: "kamn:did:agent:sender-anchor-1".to_owned(),
                nonce: 1,
                proof_hash: "fnv1a64:proof-anchor-functional".to_owned(),
            },
        )
        .expect("anchor submission should succeed");

    assert_eq!(
        result.retry_class,
        MessageProofAnchorRetryClass::NewSubmission
    );
    assert!(matches!(
        result.outcome,
        MessageProofAnchorSubmissionOutcome::Submitted(_)
    ));
    assert_eq!(
        lifecycle.status(message_id).expect("status should exist"),
        MessageStatus::Included
    );
}

#[test]
fn integration_anchor_retry_is_duplicate_without_reapplying_state_transition() {
    let mut lifecycle = MessageLifecycleStore::new();
    let message_id = "urn:uuid:msg-anchor-integration";
    register_and_advance_broadcast(&mut lifecycle, message_id);

    let mut anchoring = MessageProofAnchoringService::new();
    let mut adapter = InMemoryMessageProofChainAdapter::new("kolme-local");
    let request = MessageProofAnchorRequest {
        message_id: message_id.to_owned(),
        actor_did: "kamn:did:agent:sender-anchor-1".to_owned(),
        nonce: 7,
        proof_hash: "fnv1a64:proof-anchor-integration".to_owned(),
    };

    let first = anchoring
        .anchor_message_proof_via_chain_adapter(&mut lifecycle, &mut adapter, request.clone())
        .expect("first anchor should succeed");
    let second = anchoring
        .anchor_message_proof_via_chain_adapter(&mut lifecycle, &mut adapter, request)
        .expect("retry anchor should succeed");

    assert_eq!(
        first.retry_class,
        MessageProofAnchorRetryClass::NewSubmission
    );
    assert_eq!(
        second.retry_class,
        MessageProofAnchorRetryClass::RetryableInFlight
    );
    assert!(matches!(
        first.outcome,
        MessageProofAnchorSubmissionOutcome::Submitted(_)
    ));
    assert!(matches!(
        second.outcome,
        MessageProofAnchorSubmissionOutcome::Duplicate(_)
    ));
    assert_eq!(
        lifecycle.status(message_id).expect("status should exist"),
        MessageStatus::Included
    );
}

#[test]
fn regression_anchor_submission_rejects_lifecycle_state_mismatch_before_broadcast() {
    // Regression: #4419
    let mut lifecycle = MessageLifecycleStore::new();
    let message_id = "urn:uuid:msg-anchor-regression-state-mismatch";
    lifecycle
        .register(
            message_id,
            "kamn:did:agent:sender-anchor-1",
            vec!["kamn:did:agent:recipient-anchor-1".to_owned()],
            "2026-02-09T00:10:00.000Z",
            "2026-02-09T00:40:00.000Z",
        )
        .expect("register should succeed");
    lifecycle
        .transition(message_id, MessageStatus::Signed)
        .expect("created->signed should succeed");

    let mut anchoring = MessageProofAnchoringService::new();
    let mut adapter = InMemoryMessageProofChainAdapter::new("kolme-local");
    let error = anchoring
        .anchor_message_proof_via_chain_adapter(
            &mut lifecycle,
            &mut adapter,
            MessageProofAnchorRequest {
                message_id: message_id.to_owned(),
                actor_did: "kamn:did:agent:sender-anchor-1".to_owned(),
                nonce: 1,
                proof_hash: "fnv1a64:proof-anchor-state-mismatch".to_owned(),
            },
        )
        .expect_err("anchor submission must fail closed before broadcast");

    assert_eq!(error.reason_code(), "message_proof_anchor_invalid_state");
    assert!(matches!(
        error,
        MessageProofAnchoringError::InvalidAnchorState { .. }
    ));
}

#[test]
fn regression_anchor_submission_rejects_tampered_actor_for_same_message_nonce() {
    // Regression: #4419
    let mut lifecycle = MessageLifecycleStore::new();
    let message_id = "urn:uuid:msg-anchor-regression-actor-tamper";
    register_and_advance_broadcast(&mut lifecycle, message_id);

    let mut anchoring = MessageProofAnchoringService::new();
    let mut adapter = InMemoryMessageProofChainAdapter::new("kolme-local");

    anchoring
        .anchor_message_proof_via_chain_adapter(
            &mut lifecycle,
            &mut adapter,
            MessageProofAnchorRequest {
                message_id: message_id.to_owned(),
                actor_did: "kamn:did:agent:sender-anchor-1".to_owned(),
                nonce: 11,
                proof_hash: "fnv1a64:proof-anchor-actor-tamper".to_owned(),
            },
        )
        .expect("first anchor should succeed");

    let tampered_actor = anchoring.anchor_message_proof_via_chain_adapter(
        &mut lifecycle,
        &mut adapter,
        MessageProofAnchorRequest {
            message_id: message_id.to_owned(),
            actor_did: "kamn:did:agent:sender-anchor-2".to_owned(),
            nonce: 11,
            proof_hash: "fnv1a64:proof-anchor-actor-tamper".to_owned(),
        },
    );

    match tampered_actor {
        Err(MessageProofAnchoringError::ConflictingAnchorIdempotencyKey { .. }) => {}
        Err(other) => panic!("unexpected anchoring error: {other:?}"),
        Ok(_) => panic!("tampered actor payload should fail closed"),
    }
}

#[test]
fn regression_anchor_conflicting_payload_for_same_message_rejected_fail_closed() {
    // Regression: #2941
    let mut lifecycle = MessageLifecycleStore::new();
    let message_id = "urn:uuid:msg-anchor-regression-conflict";
    register_and_advance_broadcast(&mut lifecycle, message_id);

    let mut anchoring = MessageProofAnchoringService::new();
    let mut adapter = InMemoryMessageProofChainAdapter::new("kolme-local");

    anchoring
        .anchor_message_proof_via_chain_adapter(
            &mut lifecycle,
            &mut adapter,
            MessageProofAnchorRequest {
                message_id: message_id.to_owned(),
                actor_did: "kamn:did:agent:sender-anchor-1".to_owned(),
                nonce: 9,
                proof_hash: "fnv1a64:proof-anchor-conflict-a".to_owned(),
            },
        )
        .expect("first anchor should succeed");

    let conflicting = anchoring.anchor_message_proof_via_chain_adapter(
        &mut lifecycle,
        &mut adapter,
        MessageProofAnchorRequest {
            message_id: message_id.to_owned(),
            actor_did: "kamn:did:agent:sender-anchor-1".to_owned(),
            nonce: 9,
            proof_hash: "fnv1a64:proof-anchor-conflict-b".to_owned(),
        },
    );

    assert!(matches!(
        conflicting,
        Err(MessageProofAnchoringError::ConflictingAnchorIdempotencyKey { .. })
    ));
}

#[test]
fn regression_anchor_submission_rejects_invalid_actor_did_with_structured_marker() {
    let mut lifecycle = MessageLifecycleStore::new();
    let message_id = "urn:uuid:msg-anchor-invalid-did";
    register_and_advance_broadcast(&mut lifecycle, message_id);

    let mut anchoring = MessageProofAnchoringService::new();
    let mut adapter = InMemoryMessageProofChainAdapter::new("kolme-local");
    let result = anchoring.anchor_message_proof_via_chain_adapter(
        &mut lifecycle,
        &mut adapter,
        MessageProofAnchorRequest {
            message_id: message_id.to_owned(),
            actor_did: "bad-did".to_owned(),
            nonce: 17,
            proof_hash: "fnv1a64:proof-anchor-invalid-did".to_owned(),
        },
    );

    assert_eq!(
        result,
        Err(MessageProofAnchoringError::InvalidActorDid {
            field: "actor_did",
            reason_code: "message_proof_anchor_invalid_actor_did",
            detail: "invalid agent did prefix: bad-did".to_owned(),
        })
    );
}

#[test]
fn performance_anchor_submission_contract_lane_stays_within_budget() {
    let started = Instant::now();

    for round in 0..64 {
        let mut lifecycle = MessageLifecycleStore::new();
        let message_id = format!("urn:uuid:msg-anchor-perf-{round}");
        register_and_advance_broadcast(&mut lifecycle, message_id.as_str());

        let mut anchoring = MessageProofAnchoringService::new();
        let mut adapter = InMemoryMessageProofChainAdapter::new("kolme-local");
        anchoring
            .anchor_message_proof_via_chain_adapter(
                &mut lifecycle,
                &mut adapter,
                MessageProofAnchorRequest {
                    message_id,
                    actor_did: "kamn:did:agent:sender-anchor-1".to_owned(),
                    nonce: 1,
                    proof_hash: "fnv1a64:proof-anchor-perf".to_owned(),
                },
            )
            .expect("anchor should succeed");
    }

    let elapsed_millis = started.elapsed().as_millis();
    assert!(
        elapsed_millis < 900,
        "message proof anchor lane exceeded budget: {elapsed_millis}ms"
    );
}
