use super::super::fairness_deletion_support::assert_contains_all;
use super::super::DOC;
use super::support::assert_runtime_lane_contract_markers;

#[test]
fn doc_contains_runtime_service_api_axum_ingress_contract_lane_ci_mode_markers() {
    assert_runtime_lane_contract_markers(
        "## Runtime Service API Axum Ingress Contract Lane",
        &[
            "validate_service_api_axum_ingress_live.sh --output-json /tmp/service-api-axum-ingress-live-summary.json",
            "check_service_api_axum_ingress_live_policy.sh --report-file /tmp/service-api-axum-ingress-live-summary.json --expected-final-decision GO --ci-fast-gate PASS --output-json /tmp/service-api-axum-ingress-policy.json",
            "validate_service_api_axum_ingress_live_contract_lane.sh --output-json /tmp/service-api-axum-ingress-contract-lane-report.json --policy-output-json /tmp/service-api-axum-ingress-policy.json",
            "check_service_api_axum_ingress_live_evidence_convergence.sh --report-file /tmp/service-api-axum-ingress-contract-lane-report.json --policy-file /tmp/service-api-axum-ingress-policy.json --output-json /tmp/service-api-axum-ingress-convergence-report.json",
            "test_validate_service_api_axum_ingress_live_contract_lane.sh",
            "test_check_service_api_axum_ingress_live_policy.sh",
            "test_check_service_api_axum_ingress_live_evidence_convergence.sh",
        ],
        "service api axum ingress run-mode commands remain excluded from ci-fast-gate and ci-tools fast mode.",
        &[
            "service_api_axum_policy_marker_missing:concurrency_status",
            "service_api_axum_evidence_link_missing:source_report_file",
            "service_api_axum_promotion_decision_reason_mapping_mismatch",
        ],
        "service api axum ingress",
    );
    assert_axum_admission_markers();
    assert_axum_evidence_markers();
}

fn assert_axum_admission_markers() {
    assert_contains_all(
        DOC,
        &[
            "admission backpressure evidence convergence governance remains deterministic via:",
            "admission saturation, in-flight, and queue-budget governance remains deterministic via:",
            "admission decision taxonomy (accept/defer/reject) and runbook marker parity remains deterministic via:",
            "admission_inflight_budget_status=verified",
            "admission_queue_budget_status=verified",
            "admission_inflight_budget_limit=32",
            "admission_queue_budget_limit=1",
            "admission_budget_reason_taxonomy_version=kamn.runtime.service-api-admission-budget-reason-taxonomy.v1",
            "admission_budget_reason_codes_csv=admission_inflight_budget_mismatch,admission_queue_budget_mismatch",
            "admission_decision_taxonomy_status=verified",
            "admission_decision_accept_status=verified",
            "admission_decision_defer_status=verified",
            "admission_decision_reject_status=verified",
        ],
        "service api axum admission",
    );
}

fn assert_axum_evidence_markers() {
    assert_contains_all(
        DOC,
        &[
            "admission_decision_reason_taxonomy_version=kamn.runtime.service-api-admission-decision-reason-taxonomy.v1",
            "admission_decision_reason_codes_csv=admission_decision_accept,admission_decision_defer,admission_decision_reject",
            "admission_decision_taxonomy_mapping_status=verified",
            "admission_decision_runbook_marker_parity_status=verified",
            "admission_decision_taxonomy_runbook_reason_taxonomy_version=kamn.runtime.service-api-axum-admission-decision-runbook-reason-taxonomy.v1",
            "admission_decision_taxonomy_runbook_reason_codes_csv=admission_decision_taxonomy_mapping_drift_detected,admission_runbook_marker_parity_mismatch",
            "service_api_axum_evidence_convergence_status=verified",
            "promotion_decision_reason_mapping_status=verified",
            "service_api_axum_evidence_reason_taxonomy_version=kamn.runtime.service-api-axum-evidence-convergence-reason-taxonomy.v1",
            "service_api_axum_evidence_reason_codes_csv=service_api_axum_evidence_link_missing,service_api_axum_evidence_payload_tamper_detected,service_api_axum_promotion_decision_reason_mapping_mismatch",
            "service_api_axum_policy_admission_budget_reason_taxonomy_version_mismatch",
            "service_api_axum_policy_admission_decision_reason_taxonomy_version_mismatch",
            "service_api_axum_policy_admission_decision_reason_codes_csv_mismatch",
            "service_api_axum_policy_marker_missing:admission_decision_defer_status",
            "service_api_axum_policy_admission_inflight_budget_limit_mismatch",
            "service_api_axum_policy_admission_queue_budget_limit_mismatch",
        ],
        "service api axum evidence",
    );
}
