use super::super::*;

#[test]
fn functional_approver_quorum_authorizes_outbound_with_threshold_attestations() {
    let evaluator =
        ApproverQuorumEvaluator::new(2).expect("approver quorum evaluator should build");
    let input = ApproverQuorumInput::new(
        "outbound-action-1",
        "payload-hash-1",
        vec![
            ApproverAttestation::new("kamn:did:agent:approver-a", "payload-hash-1", "att-1")
                .expect("valid attestation"),
            ApproverAttestation::new("kamn:did:agent:approver-b", "payload-hash-1", "att-2")
                .expect("valid attestation"),
        ],
    )
    .expect("valid outbound authorization input");
    let decision = evaluator
        .authorize(input)
        .expect("outbound action should be authorized");
    assert!(decision.authorized);
    assert_eq!(decision.required_approvals, 2);
    assert_eq!(
        decision.approved_by,
        vec![
            "kamn:did:agent:approver-a".to_owned(),
            "kamn:did:agent:approver-b".to_owned()
        ]
    );
}

#[test]
fn unit_approver_quorum_rejects_zero_required_approvals() {
    let error =
        ApproverQuorumEvaluator::new(0).expect_err("zero required approvals must be rejected");
    assert_eq!(
        error,
        ApproverQuorumError::InvalidRequiredApprovals { required: 0 }
    );
}

#[test]
fn integration_daemon_outbound_approver_quorum_rejects_under_threshold() {
    let evaluator =
        ApproverQuorumEvaluator::new(2).expect("approver quorum evaluator should build");
    let input = ApproverQuorumInput::new(
        "outbound-action-under-threshold",
        "payload-hash-2",
        vec![
            ApproverAttestation::new("kamn:did:agent:approver-a", "payload-hash-2", "att-1")
                .expect("valid attestation"),
        ],
    )
    .expect("valid outbound authorization input");
    let error = authorize_daemon_outbound_action(&evaluator, input)
        .expect_err("under-threshold approvals must be rejected");
    assert_eq!(
        error,
        ApproverQuorumError::InsufficientApprovals {
            required: 2,
            received: 1
        }
    );
}

#[test]
fn regression_malformed_approver_payload_is_rejected() {
    let evaluator =
        ApproverQuorumEvaluator::new(1).expect("approver quorum evaluator should build");
    let input = ApproverQuorumInput::new(
        "outbound-action-malformed",
        "payload-hash-expected",
        vec![ApproverAttestation::new(
            "kamn:did:agent:approver-a",
            "payload-hash-tampered",
            "att-1",
        )
        .expect("valid attestation")],
    )
    .expect("valid outbound authorization input");
    let error = evaluator
        .authorize(input)
        .expect_err("payload mismatch must be rejected");
    assert_eq!(
        error,
        ApproverQuorumError::PayloadDigestMismatch {
            expected: "payload-hash-expected".to_owned(),
            found: "payload-hash-tampered".to_owned()
        }
    );
}

#[test]
fn regression_outbound_under_quorum_is_rejected() {
    let evaluator =
        ApproverQuorumEvaluator::new(3).expect("approver quorum evaluator should build");
    let input = ApproverQuorumInput::new(
        "outbound-action-regression",
        "payload-hash-regression",
        vec![
            ApproverAttestation::new(
                "kamn:did:agent:approver-a",
                "payload-hash-regression",
                "att-1",
            )
            .expect("valid attestation"),
            ApproverAttestation::new(
                "kamn:did:agent:approver-b",
                "payload-hash-regression",
                "att-2",
            )
            .expect("valid attestation"),
        ],
    )
    .expect("valid outbound authorization input");
    let error = evaluator
        .authorize(input)
        .expect_err("under-threshold approvals must be rejected");
    assert_eq!(
        error,
        ApproverQuorumError::InsufficientApprovals {
            required: 3,
            received: 2
        }
    );
}
