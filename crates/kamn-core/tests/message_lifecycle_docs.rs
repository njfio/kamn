const DOC: &str = include_str!("../../../docs/foundation/message-lifecycle.md");

#[test]
fn doc_contains_processor_proof_gated_validation_rules() {
    assert!(DOC.contains("## Processor Proof-Gated Validation"));
    assert!(DOC.contains("validate_with_processor_proof"));
    assert!(DOC.contains("Delivered -> Validated"));
}

#[test]
fn regression_requires_tampered_proof_transition_block_rule() {
    // Regression: #510
    assert!(DOC.contains("tampered proof artifacts must not advance message state"));
}
