const DOC: &str = include_str!("../../../docs/foundation/token-config.md");

#[test]
fn doc_contains_token_launch_handoff_evidence_contract() {
    assert!(DOC.contains("## Token Launch Handoff Evidence Contract"));
    assert!(DOC.contains("generate_token_launch_handoff_evidence_bundle.sh"));
    assert!(DOC.contains("check_token_launch_handoff_policy.sh"));
    assert!(DOC.contains("run_token_launch_handoff_contract_lane.sh"));
    assert!(DOC.contains("run_token_launch_handoff_deep_lane.sh"));
    assert!(DOC.contains("fixtures/token_launch/handoff_invariant_cases.json"));
}

#[test]
fn regression_requires_token_launch_handoff_guard_marker() {
    // Regression: #714
    assert!(DOC.contains(
        "supply/allocation invariant drift and insufficient approvals force `NO-GO` (`Regression: #714`)."
    ));
}
