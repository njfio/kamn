const DOC: &str = include_str!("../../../docs/ci/strategy.md");

#[test]
fn doc_contains_make_and_demo_scope_contract_rules() {
    assert!(DOC.contains("make check"));
    assert!(DOC.contains("make test"));
    assert!(DOC.contains("make demo"));
    assert!(DOC.contains("run_localhost_signed_integration_contract_lane_tests"));
    assert!(DOC.contains("sdk-live-localhost-integration"));
    assert!(DOC.contains("run_localhost_signed_integration_contract_lane.sh"));
    assert!(DOC.contains("scripts/ci/select_targets.sh"));
    assert!(DOC.contains("run_kolme_version_compatibility_contract_tests=true"));
    assert!(DOC.contains("run_local_bootstrap_health_checks.sh"));
    assert!(DOC.contains("run_local_e2e_integration_lane.sh"));
    assert!(DOC.contains("KAMN_KOLME_LOCAL_HEAVY=1"));
    assert!(
        DOC.contains("local-only heavy Kolme run-mode commands remain excluded from ci-fast-gate.")
    );
}

#[test]
fn regression_requires_make_and_selector_demo_contract_marker() {
    // Regression: #900
    assert!(DOC.contains("Regression: #900"));
    assert!(DOC.contains("make-target and selector workflow drift"));
    assert!(DOC.contains("Regression: #1419"));
}
