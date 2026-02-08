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
