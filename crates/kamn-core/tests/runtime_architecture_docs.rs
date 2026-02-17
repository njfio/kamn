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

#[test]
fn doc_contains_runtime_phase_extraction_parity_taxonomy_contract_markers() {
    assert!(DOC.contains("## Runtime Phase Extraction Parity Taxonomy"));
    assert!(DOC.contains("Task: `#4536`"));
    assert!(DOC.contains("Subtasks: `#4540`, `#4541`"));
    assert!(DOC.contains(
        "runtime_phase_parity_reason_taxonomy_version=kamn.runtime.phase-module-extraction-parity-reason-taxonomy.v1"
    ));
    assert!(DOC.contains(
        "runtime_phase_parity_reason_codes_csv=runtime_phase_module_parity_drift_detected,runtime_extraction_evidence_output_unstable,ci_local_runtime_phase_parity_budget_boundary_exceeded"
    ));
    assert!(DOC.contains("runtime_phase_module_parity_status=verified"));
    assert!(DOC.contains("runtime_extraction_evidence_output_status=verified"));
    assert!(DOC.contains("ci_local_runtime_phase_parity_budget_boundary_status=verified"));
    assert!(DOC.contains("runtime_phase_module_parity_drift_detected"));
    assert!(DOC.contains("runtime_extraction_evidence_output_unstable"));
    assert!(DOC.contains("ci_local_runtime_phase_parity_budget_boundary_exceeded"));
}

#[test]
fn doc_contains_runtime_phase_reason_mapper_and_parity_evidence_normalization_markers() {
    assert!(DOC.contains(
        "runtime_phase_parity_reason_codes_value=<normalized runtime extraction reason key>"
    ));
    assert!(DOC.contains(
        "runtime_phase_parity_evidence_outputs_csv=runtime_phase_module_parity_status,runtime_extraction_evidence_output_status,ci_local_runtime_phase_parity_budget_boundary_status"
    ));
    assert!(
        DOC.contains("runtime_phase_parity_reason_mapper=runtime_phase_parity_reason_mapper_v1")
    );
}

#[test]
fn doc_contains_docs_governance_and_rustdoc_navigation_parity_markers() {
    assert!(DOC.contains("## Docs Governance and Rustdoc Navigation Parity"));
    assert!(DOC.contains("Task: `#4524`"));
    assert!(DOC.contains("Subtasks: `#4531`, `#4532`"));
    assert!(DOC.contains(
        "reason_taxonomy_version=kamn.ci.kamn-core-missing-docs-velocity-reason-taxonomy.v1"
    ));
    assert!(DOC.contains(
        "reason_codes_csv=allowlist_fully_graduated,baseline_window_not_elapsed,ci_local_docs_velocity_window_boundary_exceeded,multiple_policy_violations,stagnation_window_exceeded,velocity_target_met,velocity_threshold_config_invalid,velocity_window_under_threshold,window_not_elapsed"
    ));
    assert!(DOC.contains("reason_codes_value=ci_local_docs_velocity_window_boundary_exceeded"));
    assert!(DOC.contains(
        "reason_taxonomy_version=kamn.ci.kamn-core-missing-docs-policy-reason-taxonomy.v1"
    ));
    assert!(DOC.contains("reason_codes_csv=rustdoc_navigation_parity_drift"));
    assert!(DOC.contains("reason_code=rustdoc_navigation_parity_drift"));
    assert!(DOC.contains("test_missing_docs_velocity_guard_contract.sh"));
    assert!(DOC.contains("test_check_kamn_core_missing_docs_policy.sh"));
}

#[test]
fn doc_contains_runtime_module_boundary_parity_drift_markers() {
    assert!(DOC.contains("## Runtime Module Boundary Parity Drift Cases (Issue #4329)"));
    assert!(DOC.contains("Task: `#4329`"));
    assert!(DOC.contains("Subtasks: `#4336`, `#4337`"));
    assert!(DOC.contains(
        "runtime_module_boundary_parity_reason_taxonomy_version=kamn.runtime.module-boundary-parity-reason-taxonomy.v1"
    ));
    assert!(DOC.contains(
        "runtime_module_boundary_parity_reason_codes_csv=runtime_orchestration_dispatch_boundary_drift_detected,runtime_daemon_phase_boundary_drift_detected,runtime_kolme_live_boundary_drift_detected,ci_local_runtime_module_boundary_budget_boundary_exceeded"
    ));
    assert!(DOC.contains(
        "runtime_module_boundary_reason_codes_value=<normalized runtime module-boundary reason key>"
    ));
    assert!(DOC.contains(
        "runtime_module_boundary_evidence_outputs_csv=runtime_module_boundary_parity_status,runtime_module_boundary_evidence_status,ci_local_runtime_module_boundary_budget_boundary_status"
    ));
    assert!(DOC.contains("runtime_module_boundary_parity_status=verified"));
    assert!(DOC.contains("runtime_module_boundary_evidence_status=verified"));
    assert!(DOC.contains("ci_local_runtime_module_boundary_budget_boundary_status=verified"));
    assert!(DOC.contains("ci_local_runtime_module_boundary_budget_boundary_exceeded"));
}

#[test]
fn doc_contains_runtime_module_boundary_parity_guard_commands() {
    assert!(DOC
        .contains("cargo test -p kamn-node --test main_module_extraction_contract -- --nocapture"));
    assert!(DOC.contains("cargo test -p kamn-core --test runtime_architecture_docs -- --nocapture"));
}
