use super::super::{
    ProofWatchdogProjectionKind, ProofWatchdogProjector, ProofWatchdogSeverity,
    ValidatorProofAttestation, ValidatorProofConsensusEvaluator, ValidatorProofConsensusInput,
    ValidatorProofConsensusStatus, ValidatorProofVerdict,
};

#[test]
fn watchdog_projection_is_nominal_for_aligned_valid_consensus() {
    let mut evaluator =
        ValidatorProofConsensusEvaluator::new(2).expect("valid quorum should build");
    let input = aligned_valid_input().expect("input should parse");
    let decision = evaluator
        .evaluate(input)
        .expect("aligned valid consensus should succeed");
    let projection = ProofWatchdogProjector::new().project(&decision);
    assert_eq!(
        decision.status,
        ValidatorProofConsensusStatus::ConsensusValid
    );
    assert_eq!(
        decision.validator_dids,
        vec![
            "kamn:did:agent:validator-a".to_owned(),
            "kamn:did:agent:validator-z".to_owned()
        ]
    );
    assert_eq!(
        projection.kind,
        ProofWatchdogProjectionKind::ConsensusAligned
    );
    assert_eq!(projection.severity, ProofWatchdogSeverity::Info);
}

fn aligned_valid_input() -> Result<ValidatorProofConsensusInput, super::super::ValidatorProofConsensusError> {
    ValidatorProofConsensusInput::new(
        "urn:uuid:message-1",
        "artifact-1",
        vec![
            valid_attestation("attestation-1", "kamn:did:agent:validator-z"),
            valid_attestation("attestation-2", "kamn:did:agent:validator-a"),
        ],
    )
}

fn valid_attestation(attestation_id: &str, validator_did: &str) -> ValidatorProofAttestation {
    ValidatorProofAttestation::new(
        attestation_id,
        validator_did,
        "urn:uuid:message-1",
        "artifact-1",
        ValidatorProofVerdict::Valid,
    )
    .expect("valid attestation")
}
