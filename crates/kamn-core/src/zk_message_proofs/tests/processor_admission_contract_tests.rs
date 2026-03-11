use super::super::{
    ProcessorProofAdmissionEvaluator, ProcessorProofAdmissionInput, ProcessorProofArtifact,
    ZkDesignError,
};

#[test]
fn processor_proof_artifact_rejects_empty_artifact_id() {
    assert_eq!(
        ProcessorProofArtifact::new(
            "",
            "urn:uuid:message-1",
            "fnv1a64:abc",
            "proof:ok:artifact-1"
        ),
        Err(ZkDesignError::InvalidProofArtifact(
            "artifact_id must not be empty".to_owned()
        ))
    );
}

#[test]
fn processor_admission_rejects_invalid_proof_value() {
    let artifact = ProcessorProofArtifact::new(
        "artifact-1",
        "urn:uuid:message-1",
        "fnv1a64:abc",
        "proof:tampered:artifact-1",
    )
    .expect("artifact should parse");
    let input = ProcessorProofAdmissionInput::new("urn:uuid:message-1", "fnv1a64:abc", artifact)
        .expect("input should parse");
    let mut evaluator = ProcessorProofAdmissionEvaluator::new();
    assert_eq!(
        evaluator.evaluate(input),
        Err(ZkDesignError::ProofVerificationFailed {
            artifact_id: "artifact-1".to_owned(),
            reason: "proof value failed deterministic verification".to_owned()
        })
    );
}
