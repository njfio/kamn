const DOC: &str = include_str!("../../../docs/foundation/anti-spam-controls.md");

#[test]
fn doc_contains_enforcement_rules_and_telemetry() {
    assert!(DOC.contains("## Enforcement Rules"));
    assert!(DOC.contains("Deposit gate"));
    assert!(DOC.contains("Per-agent rate limit"));
    assert!(DOC.contains("Suspension policy"));
    assert!(DOC.contains("## Telemetry Surface"));
    assert!(DOC.contains("rejected due to duplicate message ID"));
}

#[test]
fn doc_contains_fast_validation_commands() {
    assert!(DOC.contains("## Fast and Cost-Effective Validation"));
    assert!(DOC.contains("cargo test -p kamn-core --test anti_spam_controls"));
    assert!(DOC.contains("cargo clippy -- -D warnings"));
}

#[test]
fn regression_requires_deposit_threshold_boundary_rule() {
    // Regression: #186
    assert!(DOC.contains("sender deposit must be at least `minimum_sybil_deposit`."));
}
