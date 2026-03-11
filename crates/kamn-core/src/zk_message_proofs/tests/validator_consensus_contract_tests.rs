use super::super::{
    ValidatorProofAttestation, ValidatorProofConsensusError, ValidatorProofConsensusEvaluator,
    ValidatorProofConsensusInput, ValidatorProofVerdict,
};

#[test]
fn validator_attestation_rejects_invalid_did() {
    let error = ValidatorProofAttestation::new(
        "attestation-1",
        "validator-a",
        "urn:uuid:message-1",
        "artifact-1",
        ValidatorProofVerdict::Valid,
    )
    .expect_err("invalid validator did should be rejected");
    assert!(matches!(
        error,
        ValidatorProofConsensusError::InvalidValidatorDid(_)
    ));
}

#[test]
fn validator_consensus_rejects_duplicate_validator_attestations() {
    let mut evaluator =
        ValidatorProofConsensusEvaluator::new(2).expect("valid quorum should build");
    let input = ValidatorProofConsensusInput::new(
        "urn:uuid:message-1",
        "artifact-1",
        vec![
            ValidatorProofAttestation::new(
                "attestation-1",
                "kamn:did:agent:validator-a",
                "urn:uuid:message-1",
                "artifact-1",
                ValidatorProofVerdict::Valid,
            )
            .expect("valid attestation"),
            ValidatorProofAttestation::new(
                "attestation-2",
                "kamn:did:agent:validator-a",
                "urn:uuid:message-1",
                "artifact-1",
                ValidatorProofVerdict::Valid,
            )
            .expect("valid attestation"),
        ],
    )
    .expect("input should parse");
    assert_eq!(
        evaluator.evaluate(input),
        Err(ValidatorProofConsensusError::DuplicateValidator(
            "kamn:did:agent:validator-a".to_owned()
        ))
    );
}
