use kamn_core::{
    InstructionClaim, InstructionRecord, InstructionVerifier, VerificationContext,
    VerificationFailure, VerificationOutcome,
};

fn sample_record() -> InstructionRecord {
    InstructionRecord {
        id: "ins_001".to_owned(),
        from_did: "kamn:did:agent:alpha".to_owned(),
        payload_hash: "payload_hash_abc".to_owned(),
        signature: "sig_123".to_owned(),
        inclusion_proof_ref: "proof:chain:tx-abc".to_owned(),
    }
}

fn sample_claim() -> InstructionClaim {
    InstructionClaim {
        instruction_id: "ins_001".to_owned(),
        from_did: "kamn:did:agent:alpha".to_owned(),
        payload_hash: "payload_hash_abc".to_owned(),
        signature: "sig_123".to_owned(),
        inclusion_proof_ref: "proof:chain:tx-abc".to_owned(),
        expires_at_unix: 200,
    }
}

#[test]
fn verify_returns_valid_for_matching_authorized_nonexpired_claim() {
    let record = sample_record();
    let claim = sample_claim();
    let context = VerificationContext::new(100)
        .with_instruction(record)
        .with_authorized_sender("kamn:did:agent:alpha");

    assert_eq!(
        InstructionVerifier::verify(&claim, &context),
        VerificationOutcome::Valid
    );
}

#[test]
fn verify_rejects_missing_on_chain_instruction() {
    let claim = sample_claim();
    let context = VerificationContext::new(100).with_authorized_sender("kamn:did:agent:alpha");

    assert_eq!(
        InstructionVerifier::verify(&claim, &context),
        VerificationOutcome::Rejected(VerificationFailure::MissingInstruction(
            "ins_001".to_owned()
        ))
    );
}

#[test]
fn verify_rejects_signature_mismatch() {
    let mut record = sample_record();
    record.signature = "sig_other".to_owned();
    let claim = sample_claim();
    let context = VerificationContext::new(100)
        .with_instruction(record)
        .with_authorized_sender("kamn:did:agent:alpha");

    assert_eq!(
        InstructionVerifier::verify(&claim, &context),
        VerificationOutcome::Rejected(VerificationFailure::SignatureMismatch)
    );
}

#[test]
fn verify_rejects_unauthorized_sender() {
    let record = sample_record();
    let claim = sample_claim();
    let context = VerificationContext::new(100).with_instruction(record);

    assert_eq!(
        InstructionVerifier::verify(&claim, &context),
        VerificationOutcome::Rejected(VerificationFailure::UnauthorizedSender(
            "kamn:did:agent:alpha".to_owned()
        ))
    );
}

#[test]
fn verify_rejects_expired_claim() {
    let record = sample_record();
    let mut claim = sample_claim();
    claim.expires_at_unix = 90;
    let context = VerificationContext::new(100)
        .with_instruction(record)
        .with_authorized_sender("kamn:did:agent:alpha");

    assert_eq!(
        InstructionVerifier::verify(&claim, &context),
        VerificationOutcome::Rejected(VerificationFailure::Expired {
            expires_at: 90,
            now: 100,
        })
    );
}

#[test]
fn regression_rejects_overlong_claim_validity_window() {
    // Regression: #409
    let record = sample_record();
    let mut claim = sample_claim();
    claim.expires_at_unix = 100 + (24 * 60 * 60) + 1;
    let context = VerificationContext::new(100)
        .with_instruction(record)
        .with_authorized_sender("kamn:did:agent:alpha");

    assert_eq!(
        InstructionVerifier::verify(&claim, &context),
        VerificationOutcome::Rejected(VerificationFailure::OverlongValidityWindow {
            max_window_secs: 24 * 60 * 60,
            requested_window_secs: (24 * 60 * 60) + 1,
        })
    );
}

#[test]
fn regression_replayed_claim_is_rejected_after_first_use() {
    // Regression: #414
    let record = sample_record();
    let claim = sample_claim();
    let mut context = VerificationContext::new(100)
        .with_instruction(record)
        .with_authorized_sender("kamn:did:agent:alpha");

    assert_eq!(
        InstructionVerifier::verify_and_record(&claim, &mut context),
        VerificationOutcome::Valid
    );
    assert_eq!(
        InstructionVerifier::verify_and_record(&claim, &mut context),
        VerificationOutcome::Rejected(VerificationFailure::ReplayClaim {
            instruction_id: "ins_001".to_owned(),
        })
    );
}

#[test]
fn regression_rejects_missing_inclusion_proof_reference() {
    // Regression: #448
    let record = sample_record();
    let mut claim = sample_claim();
    claim.inclusion_proof_ref.clear();
    let context = VerificationContext::new(100)
        .with_instruction(record)
        .with_authorized_sender("kamn:did:agent:alpha");

    assert_eq!(
        InstructionVerifier::verify(&claim, &context),
        VerificationOutcome::Rejected(VerificationFailure::MissingInclusionProofReference)
    );
}

#[test]
fn regression_rejects_mismatched_inclusion_proof_reference() {
    // Regression: #448
    let record = sample_record();
    let mut claim = sample_claim();
    claim.inclusion_proof_ref = "proof:chain:tx-other".to_owned();
    let context = VerificationContext::new(100)
        .with_instruction(record)
        .with_authorized_sender("kamn:did:agent:alpha");

    assert_eq!(
        InstructionVerifier::verify(&claim, &context),
        VerificationOutcome::Rejected(VerificationFailure::InclusionProofMismatch {
            expected: "proof:chain:tx-abc".to_owned(),
            actual: "proof:chain:tx-other".to_owned(),
        })
    );
}
