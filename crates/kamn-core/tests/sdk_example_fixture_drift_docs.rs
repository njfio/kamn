const DOC: &str = include_str!("../../../docs/planning/sdk-parity-wave.md");

#[test]
fn doc_contains_fixture_drift_checker_contract_commands() {
    assert!(DOC.contains("## SDK Example Fixture Drift Checker Contract (Issue #940)"));
    assert!(DOC.contains("check_example_fixture_drift.py"));
    assert!(DOC.contains("run_example_fixture_drift_contract_lane.sh"));
    assert!(DOC.contains("check_example_fixture_drift_policy.sh"));
    assert!(DOC.contains("fixtures/sdk_parity/register_validation_snapshot.json"));
}

#[test]
fn regression_requires_fixture_drift_fail_closed_marker() {
    // Regression: #940
    assert!(DOC.contains("Regression: #940"));
}
