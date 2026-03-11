use super::super::{
    ProofWatchdogProjectionKind, ProofWatchdogProjector, ProofWatchdogSeverity,
    ValidatorProofAttestation, ValidatorProofConsensusEvaluator, ValidatorProofConsensusInput,
    ValidatorProofConsensusStatus, ValidatorProofVerdict,
};

#[test]
fn watchdog_projection_is_nominal_for_aligned_valid_consensus() {
    let mut evaluator =
        ValidatorProofConsensusEvaluator::new(2).expect("valid quorum should build");
    let input = ValidatorProofConsensusInput::new(
        "urn:uuid:message-1",
        "artifact-1",
        vec![
            ValidatorProofAttestation::new(
                "attestation-1",
                "kamn:did:agent:validator-z",
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
