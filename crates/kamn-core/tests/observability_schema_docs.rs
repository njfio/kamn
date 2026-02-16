const DOC: &str = include_str!("../../../docs/observability/schema.md");

#[test]
fn observability_schema_contains_slo_threshold_and_gate_taxonomy_matrix() {
    assert!(DOC.contains("## SLO Threshold and Gate Reason Taxonomy Matrix (Issue #4462)"));
    assert!(DOC.contains(
        "reason_taxonomy_version=kamn.release.gonogo-slo-threshold-convergence-reason-taxonomy.v1"
    ));
    assert!(DOC.contains(
        "reason_codes_csv=gonogo_slo_policy_file_missing,gonogo_slo_policy_invalid_json,gonogo_slo_policy_schema_mismatch,gonogo_slo_policy_status_not_pass,gonogo_slo_policy_final_decision_not_go,gonogo_slo_policy_reason_key_mismatch,gonogo_slo_policy_reason_codes_not_empty,gonogo_slo_policy_freshness_window_exceeded"
    ));
    assert!(DOC.contains("reason_codes_value=none|<csv>"));
}

#[test]
fn observability_schema_contains_slo_threshold_drift_failure_cases() {
    assert!(DOC.contains("Threshold drift failure cases"));
    assert!(DOC.contains("gonogo_slo_policy_reason_key_mismatch"));
    assert!(DOC.contains("gonogo_slo_policy_reason_codes_not_empty"));
    assert!(DOC.contains("Regression: #4467"));
}
