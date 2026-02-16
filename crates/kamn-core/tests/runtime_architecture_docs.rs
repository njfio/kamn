const DOC: &str = include_str!("../../../docs/architecture/runtime.md");

#[test]
fn doc_contains_runtime_extraction_fallback_taxonomy_contract_markers() {
    assert!(DOC.contains("## Runtime Extraction Fallback Taxonomy"));
    assert!(DOC.contains("Task: `#4537`"));
    assert!(DOC.contains("Subtasks: `#4542`, `#4543`"));
    assert!(DOC.contains(
        "runtime_error_reason_taxonomy_version=kamn.runtime.local-full-runtime-error-reason-taxonomy.v1"
    ));
    assert!(DOC.contains(
        "runtime_error_reason_codes_csv=runtime_full_shutdown_gate_drift_detected,runtime_fallback_classification_unstable,ci_local_runtime_extraction_budget_boundary_exceeded"
    ));
    assert!(DOC.contains("runtime_shutdown_gate_status=verified"));
    assert!(DOC.contains("runtime_fallback_classification_status=verified"));
    assert!(DOC.contains("ci_local_runtime_extraction_budget_boundary_status=verified"));
    assert!(DOC.contains("runtime_full_shutdown_gate_drift_detected"));
    assert!(DOC.contains("runtime_fallback_classification_unstable"));
    assert!(DOC.contains("ci_local_runtime_extraction_budget_boundary_exceeded"));
}

#[test]
fn doc_contains_runtime_extraction_fallback_evidence_entrypoints() {
    assert!(DOC.contains("## Evidence and Policy Entrypoints"));
    assert!(DOC.contains(
        "validate_local_full_runtime_live.sh --mode dry-run --output-json /tmp/local-full-runtime-live-summary.json"
    ));
    assert!(DOC.contains(
        "check_local_full_runtime_live_policy.sh --report-file /tmp/local-full-runtime-live-summary.json --expected-final-decision GO --ci-fast-gate PASS --output-json /tmp/local-full-runtime-live-policy.json"
    ));
    assert!(DOC.contains(
        "validate_local_full_runtime_live_contract_lane.sh --output-json /tmp/local-full-runtime-live-contract-lane-report.json --policy-output-json /tmp/local-full-runtime-live-policy.json"
    ));
    assert!(DOC.contains("240"));
}
