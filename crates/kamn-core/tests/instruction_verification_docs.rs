const DOC: &str = include_str!("../../../docs/foundation/instruction-verification.md");

#[test]
fn doc_contains_instruction_verification_scope_and_checks() {
    assert!(DOC.contains("# Instruction Verification Pipeline"));
    assert!(DOC.contains("## Verification Checks"));
    assert!(DOC.contains("InstructionVerifier::verify(...)"));
}

#[test]
fn regression_requires_overlong_claim_window_rejection_rule() {
    // Regression: #409
    assert!(DOC.contains("bounded claim validity window"));
    assert!(DOC.contains("OverlongValidityWindow"));
    assert!(DOC.contains("overlong validity window is rejected (`Regression: #409`)"));
}

#[test]
fn regression_requires_replay_claim_rejection_rule() {
    // Regression: #414
    assert!(DOC.contains("one-time claim consumption"));
    assert!(DOC.contains("ReplayClaim"));
    assert!(DOC.contains("replayed claim is rejected (`Regression: #414`)"));
}

#[test]
fn regression_requires_inclusion_proof_binding_rules() {
    // Regression: #448
    assert!(DOC.contains("inclusion proof reference"));
    assert!(DOC.contains("MissingInclusionProofReference"));
    assert!(DOC.contains("InclusionProofMismatch"));
    assert!(DOC.contains(
        "mismatched or missing inclusion proof reference is rejected (`Regression: #448`)"
    ));
}

#[test]
fn regression_requires_sender_did_validation_rules() {
    // Regression: #453
    assert!(DOC.contains("sender DID format validation"));
    assert!(DOC.contains("InvalidClaimSenderDid"));
    assert!(DOC.contains("InvalidRecordSenderDid"));
    assert!(DOC.contains("malformed claim or record sender DID is rejected (`Regression: #453`)"));
}

#[test]
fn regression_requires_non_empty_signature_rules() {
    // Regression: #553
    assert!(DOC.contains("Claim and on-chain signatures must be non-empty."));
    assert!(DOC.contains("MissingClaimSignature"));
    assert!(DOC.contains("MissingRecordSignature"));
    assert!(DOC.contains("empty claim or record signatures are rejected (`Regression: #553`)."));
}
