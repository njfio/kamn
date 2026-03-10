use super::super::shared_support::{assert_deploy_contains_all};

const DEPLOY_COMPAT_CONTAINS_SERVICE_API_AXUM_PROTOCOL_TAXONOMY_RUNBOOK_PARITY_MARKERS_DEPLOY_MARKERS: &[&str] = &[
    "## Service API Axum Protocol Taxonomy and Runbook Marker Parity Contracts (Issue #4267)",
    "protocol_taxonomy_mapping_status=verified",
    "runbook_marker_parity_status=verified",
    "protocol_taxonomy_runbook_reason_taxonomy_version=kamn.runtime.service-api-axum-protocol-taxonomy-runbook-reason-taxonomy.v1",
    "protocol_taxonomy_runbook_reason_codes_csv=protocol_taxonomy_mapping_drift_detected,runbook_marker_parity_mismatch",
    "protocol_compliance_reason_taxonomy_version=kamn.runtime.service-api-protocol-compliance-reason-taxonomy.v1",
    "request_validation_reason_taxonomy_version=kamn.runtime.service-api-request-validation-reason-taxonomy.v1",
    "error_envelope_reason_taxonomy_version=kamn.runtime.service-api-error-envelope-reason-taxonomy.v1",
    "service_api_axum_protocol_mismatch_reason_taxonomy_version=kamn.runtime.service-api-axum-protocol-mismatch-reason-taxonomy.v1",
    "protocol_taxonomy_mapping_drift_detected",
    "runbook_marker_parity_mismatch",
    "validate_service_api_axum_ingress_live_contract_lane.sh",
    "Regression: #4272",
    "Regression: #4273",
];

#[test]
fn deploy_compat_contains_service_api_axum_protocol_taxonomy_runbook_parity_markers() {
    assert_deploy_contains_all(DEPLOY_COMPAT_CONTAINS_SERVICE_API_AXUM_PROTOCOL_TAXONOMY_RUNBOOK_PARITY_MARKERS_DEPLOY_MARKERS, "deploy_compat_contains_service_api_axum_protocol_taxonomy_runbook_parity_markers");
}

const DEPLOY_COMPAT_CONTAINS_SERVICE_API_AXUM_ADMISSION_DECISION_TAXONOMY_RUNBOOK_PARITY_MARKERS_DEPLOY_MARKERS: &[&str] = &[
    "## Service API Axum Admission Decision Taxonomy and Runbook Marker Parity Contracts (Issue #4222)",
    "admission_decision_taxonomy_mapping_status=verified",
    "admission_decision_runbook_marker_parity_status=verified",
    "admission_decision_taxonomy_runbook_reason_taxonomy_version=kamn.runtime.service-api-axum-admission-decision-runbook-reason-taxonomy.v1",
    "admission_decision_taxonomy_runbook_reason_codes_csv=admission_decision_taxonomy_mapping_drift_detected,admission_runbook_marker_parity_mismatch",
    "admission_decision_reason_taxonomy_version=kamn.runtime.service-api-admission-decision-reason-taxonomy.v1",
    "admission_decision_reason_codes_csv=admission_decision_accept,admission_decision_defer,admission_decision_reject",
    "admission_decision_taxonomy_status=verified",
    "admission_decision_accept_status=verified",
    "admission_decision_defer_status=verified",
    "admission_decision_reject_status=verified",
    "protocol_taxonomy_mapping_drift_detected",
    "runbook_marker_parity_mismatch",
    "validate_service_api_axum_ingress_live_contract_lane.sh",
    "Regression: #4227",
    "Regression: #4228",
];

#[test]
fn deploy_compat_contains_service_api_axum_admission_decision_taxonomy_runbook_parity_markers() {
    assert_deploy_contains_all(DEPLOY_COMPAT_CONTAINS_SERVICE_API_AXUM_ADMISSION_DECISION_TAXONOMY_RUNBOOK_PARITY_MARKERS_DEPLOY_MARKERS, "deploy_compat_contains_service_api_axum_admission_decision_taxonomy_runbook_parity_markers");
}

const DEPLOY_COMPAT_CONTAINS_SERVICE_API_AXUM_ADMISSION_BACKPRESSURE_EVIDENCE_MARKERS_DEPLOY_MARKERS: &[&str] = &[
    "## Service API Axum Admission/Backpressure Evidence Convergence Contracts (Issue #4223)",
    "service_api_axum_evidence_convergence_status=verified",
    "promotion_decision_reason_mapping_status=verified",
    "service_api_axum_evidence_reason_taxonomy_version=kamn.runtime.service-api-axum-evidence-convergence-reason-taxonomy.v1",
    "service_api_axum_evidence_reason_codes_csv=service_api_axum_evidence_link_missing,service_api_axum_evidence_payload_tamper_detected,service_api_axum_promotion_decision_reason_mapping_mismatch",
    "promotion_decision_reason_taxonomy_version=kamn.runtime.service-api-axum-protocol-mismatch-reason-taxonomy.v1",
    "promotion_decision_reason_codes_csv=service_api_axum_policy_required_field_missing,service_api_axum_policy_marker_missing,service_api_axum_policy_protocol_taxonomy_mismatch,service_api_axum_policy_limit_contract_mismatch,ci_fast_gate_failed,service_api_axum_policy_expected_decision_mismatch,service_api_axum_policy_violation",
    "service_api_axum_evidence_link_missing:source_report_file",
    "service_api_axum_promotion_decision_reason_mapping_mismatch",
    "check_service_api_axum_ingress_live_evidence_convergence.sh --report-file /tmp/service-api-axum-ingress-contract-lane-report.json --policy-file /tmp/service-api-axum-ingress-policy-report.json --output-json /tmp/service-api-axum-ingress-convergence-report.json",
    "Regression: #4229",
    "Regression: #4230",
];

#[test]
fn deploy_compat_contains_service_api_axum_admission_backpressure_evidence_markers() {
    assert_deploy_contains_all(DEPLOY_COMPAT_CONTAINS_SERVICE_API_AXUM_ADMISSION_BACKPRESSURE_EVIDENCE_MARKERS_DEPLOY_MARKERS, "deploy_compat_contains_service_api_axum_admission_backpressure_evidence_markers");
}

const DEPLOY_COMPAT_CONTAINS_FORK_CHOICE_FINALITY_TAXONOMY_RUNBOOK_PARITY_MARKERS_DEPLOY_MARKERS: &[&str] = &[
    "## Fork-Choice Finality Taxonomy and Runbook Marker Parity Contracts (Issue #4252)",
    "finality_taxonomy_mapping_status=verified",
    "runbook_marker_parity_status=verified",
    "convergence_reason_taxonomy_version=kamn.runtime.libp2p-convergence-reason-taxonomy.v1",
    "convergence_reason_codes_csv=fork_choice_stale_block_height",
    "finality_taxonomy_runbook_reason_taxonomy_version=kamn.runtime.libp2p-fork-choice-finality-taxonomy-runbook-reason-taxonomy.v1",
    "finality_taxonomy_runbook_reason_codes_csv=finality_taxonomy_mapping_drift_detected,runbook_marker_parity_mismatch",
    "finality_taxonomy_mapping_drift_detected",
    "runbook_marker_parity_mismatch",
    "check_libp2p_convergence_process_isolated_live_policy.sh --report-file /tmp/libp2p-convergence-process-isolated-live-summary.json --expected-final-decision GO --ci-fast-gate PASS --runbook-file docs/deploy/kolme_devnet_ops.md --output-json /tmp/libp2p-convergence-process-isolated-live-policy.json",
    "check_libp2p_convergence_process_isolated_live_evidence_convergence.sh --report-file /tmp/libp2p-convergence-process-isolated-live-contract-lane-report.json --policy-file /tmp/libp2p-convergence-process-isolated-live-policy.json --output-json /tmp/libp2p-convergence-process-isolated-live-convergence-report.json",
    "promotion_decision_reason_taxonomy_version=kamn.runtime.libp2p-process-isolated-convergence-promotion-decision-reason-taxonomy.v1",
    "promotion_decision_reason_codes_csv=libp2p_process_isolated_convergence_policy_required_field_missing,libp2p_process_isolated_convergence_policy_marker_missing,libp2p_process_isolated_convergence_policy_reason_taxonomy_mismatch,libp2p_process_isolated_convergence_policy_runtime_mode_contract_mismatch,finality_taxonomy_mapping_drift_detected,runbook_marker_parity_mismatch,ci_fast_gate_failed,libp2p_process_isolated_convergence_policy_expected_decision_mismatch,libp2p_process_isolated_convergence_policy_violation",
    "libp2p_finality_evidence_convergence_status=verified",
    "libp2p_finality_evidence_reason_taxonomy_version=kamn.runtime.libp2p-fork-choice-finality-evidence-convergence-reason-taxonomy.v1",
    "libp2p_finality_evidence_reason_codes_csv=libp2p_finality_evidence_link_missing,libp2p_finality_evidence_payload_tamper_detected,libp2p_finality_promotion_decision_reason_mapping_mismatch",
    "libp2p_finality_evidence_link_missing:source_report_file",
    "libp2p_finality_promotion_decision_reason_mapping_mismatch",
    "validate_libp2p_convergence_process_isolated_live_contract_lane.sh",
    "Regression: #4257",
    "Regression: #4258",
    "Regression: #4259",
    "Regression: #4260",
];

#[test]
fn deploy_compat_contains_fork_choice_finality_taxonomy_runbook_parity_markers() {
    assert_deploy_contains_all(DEPLOY_COMPAT_CONTAINS_FORK_CHOICE_FINALITY_TAXONOMY_RUNBOOK_PARITY_MARKERS_DEPLOY_MARKERS, "deploy_compat_contains_fork_choice_finality_taxonomy_runbook_parity_markers");
}
