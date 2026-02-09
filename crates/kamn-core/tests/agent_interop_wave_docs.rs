const DOC: &str = include_str!("../../../docs/planning/agent-interop-wave.md");

#[test]
fn doc_contains_did_lifecycle_contract_lane_commands() {
    assert!(DOC.contains("## DID Lifecycle Mutation Contract Lane (Issue #889)"));
    assert!(DOC.contains("did_lifecycle_mutation_transactions"));
    assert!(DOC.contains("run_did_registry_contract_lane.sh"));
    assert!(DOC.contains("did_lifecycle_mutation_reason_codes:GO:v1"));
}

#[test]
fn regression_requires_did_lifecycle_drift_fail_closed_marker() {
    // Regression: #889
    assert!(DOC.contains("Regression: #889"));
}

#[test]
fn doc_contains_lifecycle_operator_binding_contract_lane_commands() {
    assert!(DOC.contains("## Lifecycle Operator-Binding Policy Contract Lane (Issue #890)"));
    assert!(DOC.contains("run_lifecycle_operator_binding_contract_lane.sh"));
    assert!(DOC.contains("did_lifecycle_operator_binding_reason_codes:GO:v1"));
}

#[test]
fn regression_requires_lifecycle_operator_binding_drift_fail_closed_marker() {
    // Regression: #890
    assert!(DOC.contains("Regression: #890"));
}
