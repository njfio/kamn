const DOC: &str = include_str!("../../../docs/foundation/node-runtime-cli.md");

#[test]
fn doc_contains_output_mode_scope_and_rules() {
    assert!(DOC.contains("## Scope Delivered"));
    assert!(DOC.contains("--output text"));
    assert!(DOC.contains("--output json"));
    assert!(DOC.contains("ConfigError::InvalidOutputMode"));
}

#[test]
fn doc_contains_deterministic_json_fields() {
    assert!(DOC.contains("JSON output is deterministic and includes:"));
    assert!(DOC.contains("sync_mode"));
    assert!(DOC.contains("components"));
}

#[test]
fn doc_contains_fast_and_cost_effective_validation_lane() {
    assert!(DOC.contains("## Fast and Cost-Effective Validation"));
    assert!(DOC.contains("cargo test -p kamn-node"));
    assert!(DOC.contains("cargo clippy -p kamn-node -- -D warnings"));
}

#[test]
fn regression_requires_invalid_output_mode_rule() {
    // Regression: #307
    assert!(DOC.contains("Invalid modes are rejected with explicit typed error."));
}
