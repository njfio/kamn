const DOC: &str = include_str!("../../../docs/observability/schema.md");

#[test]
fn observability_schema_contains_runtime_endpoint_checker_taxonomy_markers() {
    assert!(DOC.contains("## Runtime Observability Endpoint Emission Payload Contract"));
    assert!(DOC.contains(
        "reason_taxonomy_version=kamn.runtime.observability-endpoint-reason-taxonomy.v1"
    ));
    assert!(DOC.contains(
        "reason_codes_csv=runtime_observability_endpoint_readiness_progress_stalled,runtime_observability_stream_parity_bypass_detected,ci_local_observability_endpoint_budget_boundary_exceeded,runtime_observability_policy_required_field_missing,runtime_observability_policy_schema_drift"
    ));
    assert!(
        DOC.contains("runtime_observability_policy_required_field_missing:<surface>.<field-name>")
    );
    assert!(DOC.contains("runtime_observability_policy_schema_drift:<surface>.schema_version"));
}

#[test]
fn observability_schema_contains_runtime_endpoint_fail_closed_envelope_markers() {
    assert!(DOC.contains("schema_version=kamn.runtime.observability.endpoint-fail-closed.v1"));
    assert!(DOC.contains("status=fail_closed"));
    assert!(DOC.contains("final_decision=NO-GO"));
    assert!(DOC.contains("surface in {metrics,health,readiness,stream}"));
}

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
