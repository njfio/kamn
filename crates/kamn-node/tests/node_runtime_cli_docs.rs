const DOC: &str = include_str!("../../../docs/foundation/node-runtime-cli.md");

#[test]
fn doc_contains_output_mode_scope_and_rules() {
    assert!(DOC.contains("## Scope Delivered"));
    assert!(DOC.contains("--output text"));
    assert!(DOC.contains("--output json"));
    assert!(DOC.contains("ConfigError::InvalidOutputMode"));
    assert!(DOC.contains("--profile local-listener"));
    assert!(DOC.contains("ConfigError::InvalidNodeProfile"));
    assert!(DOC.contains("--diagnostics snapshot"));
    assert!(DOC.contains("ConfigError::InvalidDiagnosticsMode"));
}

#[test]
fn doc_contains_deterministic_json_fields() {
    assert!(DOC.contains("JSON output is deterministic and includes:"));
    assert!(DOC.contains("diagnostics_mode"));
    assert!(DOC.contains("profile"));
    assert!(DOC.contains("component_count"));
    assert!(DOC.contains("sync_mode"));
    assert!(DOC.contains("components"));
}

#[test]
fn doc_contains_local_profile_rules() {
    assert!(DOC.contains("## Local Profile Rules"));
    assert!(DOC.contains("chain_id`: `kamn-localnet`"));
    assert!(DOC.contains("storage_dir`: role-scoped"));
    assert!(DOC.contains("Explicit CLI flags override profile defaults"));
}

#[test]
fn doc_contains_diagnostics_snapshot_rules() {
    assert!(DOC.contains("## Diagnostics Snapshot Rules"));
    assert!(DOC.contains("`basic` (default)"));
    assert!(DOC.contains("`snapshot`"));
    assert!(DOC.contains("component_count"));
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

#[test]
fn regression_requires_invalid_profile_rule() {
    // Regression: #310
    assert!(DOC.contains("Invalid profiles are rejected with explicit typed error."));
}

#[test]
fn regression_requires_invalid_diagnostics_mode_rule() {
    // Regression: #313
    assert!(DOC.contains("Invalid diagnostics modes are rejected with explicit typed error."));
}
