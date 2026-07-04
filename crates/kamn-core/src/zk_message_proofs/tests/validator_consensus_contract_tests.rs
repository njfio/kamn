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
    let input = duplicate_validator_input().expect("input should parse");
    assert_eq!(
        evaluator.evaluate(input),
        Err(ValidatorProofConsensusError::DuplicateValidator(
            "kamn:did:agent:validator-a".to_owned()
        ))
    );
}

fn duplicate_validator_input() -> Result<ValidatorProofConsensusInput, ValidatorProofConsensusError>
{
    ValidatorProofConsensusInput::new(
        "urn:uuid:message-1",
        "artifact-1",
        vec![
            valid_attestation("attestation-1"),
            valid_attestation("attestation-2"),
        ],
    )
}

fn valid_attestation(attestation_id: &str) -> ValidatorProofAttestation {
    ValidatorProofAttestation::new(
        attestation_id,
        "kamn:did:agent:validator-a",
        "urn:uuid:message-1",
        "artifact-1",
        ValidatorProofVerdict::Valid,
    )
    .expect("valid attestation")
}
