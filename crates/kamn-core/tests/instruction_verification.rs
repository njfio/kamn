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
    }
}

fn sample_claim() -> InstructionClaim {
    InstructionClaim {
        instruction_id: "ins_001".to_owned(),
        from_did: "kamn:did:agent:alpha".to_owned(),
        payload_hash: "payload_hash_abc".to_owned(),
        signature: "sig_123".to_owned(),
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
