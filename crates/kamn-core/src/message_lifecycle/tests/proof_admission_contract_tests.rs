use super::support::{advance_message, build_artifact, register_message};
use super::*;

#[test]
fn validate_with_processor_proof_rejects_non_delivered_state() {
    let mut store = MessageLifecycleStore::new();
    register_message(&mut store, "urn:uuid:msg-3");
    let mut evaluator = ProcessorProofAdmissionEvaluator::new();
    let artifact = build_artifact(
        "artifact-1",
        "urn:uuid:msg-3",
        "fnv1a64:abc",
        "proof:ok:artifact-1",
    );
    assert_eq!(
        store.validate_with_processor_proof(
            "urn:uuid:msg-3",
            "fnv1a64:abc",
            artifact,
            &mut evaluator
        ),
        Err(MessageProofAdmissionError::InvalidValidationState {
            found: MessageStatus::Created
        })
    );
}

#[test]
fn validate_with_processor_proof_maps_proof_errors() {
    let mut store = MessageLifecycleStore::new();
    register_message(&mut store, "urn:uuid:msg-4");
    advance_message(
        &mut store,
        "urn:uuid:msg-4",
        &[
            MessageStatus::Signed,
            MessageStatus::Broadcast,
            MessageStatus::Included,
            MessageStatus::Delivered,
        ],
    );
    let mut evaluator = ProcessorProofAdmissionEvaluator::new();
    let artifact = build_artifact(
        "artifact-2",
        "urn:uuid:msg-4",
        "fnv1a64:abc",
        "proof:tampered:artifact-2",
    );
    assert_eq!(
        store.validate_with_processor_proof(
            "urn:uuid:msg-4",
            "fnv1a64:abc",
            artifact,
            &mut evaluator
        ),
        Err(MessageProofAdmissionError::Proof(
            ZkDesignError::ProofVerificationFailed {
                artifact_id: "artifact-2".to_owned(),
                reason: "proof value failed deterministic verification".to_owned(),
            }
        ))
    );
}
