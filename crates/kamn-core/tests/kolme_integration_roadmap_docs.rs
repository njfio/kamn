const ROADMAP: &str = include_str!("../../../docs/planning/kolme-integration-roadmap.md");

#[test]
fn roadmap_contains_version_and_runtime_commit_contract_lane_commands() {
    assert!(ROADMAP.contains("validate_version_compatibility.py"));
    assert!(ROADMAP.contains("run_version_compatibility_contract_lane.sh"));
    assert!(ROADMAP.contains("run_runtime_commit_contract_lane.sh"));
    assert!(ROADMAP.contains("fixtures/kolme_commit/runtime_commit_request_cases.txt"));
}

#[test]
fn regression_guards_include_legacy_and_runtime_commit_markers() {
    assert!(ROADMAP.contains("`Regression: #775`"));
    assert!(ROADMAP.contains("`Regression: #825`"));
}
