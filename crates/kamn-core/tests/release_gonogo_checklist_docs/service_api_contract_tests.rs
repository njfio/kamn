use super::support::assert_checklist_contains_all;

const CHECKLIST_CONTAINS_GO_NO_GO_EVIDENCE_TEMPLATE_MARKERS: &[&str] = &[
    "## Go/No-Go Evidence Template",
    "Release candidate:",
    "Schema target version:",
    "Rollback trigger status:",
    "Final decision: GO | NO-GO",
];

#[test]
fn checklist_contains_go_no_go_evidence_template() {
    assert_checklist_contains_all(CHECKLIST_CONTAINS_GO_NO_GO_EVIDENCE_TEMPLATE_MARKERS, "checklist_contains_go_no_go_evidence_template");
}

const CHECKLIST_CONTAINS_MESSAGE_ANCHORING_MISMATCH_TAMPER_GATE_MARKERS: &[&str] = &[
    "## Message Anchoring Mismatch/Tamper Gate (Issue #4419)",
    "run_message_proof_anchoring_contract_lane.sh",
    "validate_message_proof_anchoring_live.sh",
    "anchoring_gate_reason_taxonomy_version=kamn.kolme.message-proof-anchoring-gate-reason-taxonomy.v1",
    "anchoring_gate_reason_codes_csv=message_anchor_evidence_mismatch,message_anchor_evidence_tamper_detected,message_proof_anchor_conflicting_key,message_proof_anchor_invalid_state,ci_fast_gate_failed,local_heavy_opt_in_required",
    "ci_smoke_local_heavy_boundary_status=verified",
    "local_heavy_lane_execution_mode=opt_in",
    "Regression: #4419",
];

#[test]
fn checklist_contains_message_anchoring_mismatch_tamper_gate() {
    assert_checklist_contains_all(CHECKLIST_CONTAINS_MESSAGE_ANCHORING_MISMATCH_TAMPER_GATE_MARKERS, "checklist_contains_message_anchoring_mismatch_tamper_gate");
}

const CHECKLIST_CONTAINS_SERVICE_API_PROTOCOL_SESSION_REASON_MAPPING_GATE_MARKERS: &[&str] = &[
    "## Service API Protocol/Session Reason Mapping Gate (Issue #4318)",
    "service_api_protocol_session_reason_taxonomy_version=kamn.runtime.service-api.protocol-session-reason-taxonomy.v1",
    "service_api_protocol_session_reason_codes_csv=service_api_ws_upgrade_header_missing,service_api_ws_connection_header_missing,service_api_ws_key_header_missing,service_api_ws_version_header_missing,service_api_ws_upgrade_header_invalid,service_api_ws_connection_header_invalid,service_api_ws_key_header_empty,service_api_ws_version_header_invalid,service_api_payload_json_syntax_invalid,service_api_payload_structure_invalid,service_api_payload_io_error,service_api_auth_replay_nonce_detected,service_api_websocket_upgrade_required,service_api_protocol_session_docs_marker_missing",
    "service_api_ws_upgrade_header_missing",
    "service_api_ws_version_header_invalid",
    "service_api_payload_json_syntax_invalid",
    "service_api_auth_replay_nonce_detected",
    "service_api_protocol_session_docs_marker_missing",
    "Regression: #4318",
];

#[test]
fn checklist_contains_service_api_protocol_session_reason_mapping_gate() {
    assert_checklist_contains_all(CHECKLIST_CONTAINS_SERVICE_API_PROTOCOL_SESSION_REASON_MAPPING_GATE_MARKERS, "checklist_contains_service_api_protocol_session_reason_mapping_gate");
}

const CHECKLIST_CONTAINS_SERVICE_API_AXUM_PROTOCOL_MISMATCH_REASON_MAPPING_GATE_MARKERS: &[&str] = &[
    "## Service API Axum Protocol Mismatch Reason Mapping Gate (Issues #4266, #4270, #4271)",
    "test_check_service_api_axum_ingress_live_policy.sh",
    "test_validate_service_api_axum_ingress_live_contract_lane.sh",
    "service_api_axum_protocol_mismatch_reason_mapping_status=verified",
    "service_api_axum_protocol_mismatch_reason_taxonomy_version=kamn.runtime.service-api-axum-protocol-mismatch-reason-taxonomy.v1",
    "service_api_axum_protocol_mismatch_reason_codes_csv=service_api_axum_policy_required_field_missing,service_api_axum_policy_marker_missing,service_api_axum_policy_protocol_taxonomy_mismatch,service_api_axum_policy_limit_contract_mismatch,ci_fast_gate_failed,service_api_axum_policy_expected_decision_mismatch,service_api_axum_policy_violation",
    "service_api_axum_protocol_mismatch_reason_code=none|<reason>",
    "admission_inflight_budget_status=verified",
    "admission_queue_budget_status=verified",
    "admission_inflight_budget_limit=32",
    "admission_queue_budget_limit=1",
    "admission_budget_reason_taxonomy_version=kamn.runtime.service-api-admission-budget-reason-taxonomy.v1",
    "admission_budget_reason_codes_csv=admission_inflight_budget_mismatch,admission_queue_budget_mismatch",
    "service_api_axum_policy_marker_missing:<field>",
    "service_api_axum_policy_protocol_compliance_reason_taxonomy_version_mismatch",
    "service_api_axum_policy_admission_budget_reason_taxonomy_version_mismatch",
    "service_api_axum_policy_admission_inflight_budget_limit_mismatch",
    "service_api_axum_policy_admission_queue_budget_limit_mismatch",
    "service_api_axum_policy_body_size_limit_mismatch",
    "Regression: #4270",
    "Regression: #4271",
];

#[test]
fn checklist_contains_service_api_axum_protocol_mismatch_reason_mapping_gate() {
    assert_checklist_contains_all(CHECKLIST_CONTAINS_SERVICE_API_AXUM_PROTOCOL_MISMATCH_REASON_MAPPING_GATE_MARKERS, "checklist_contains_service_api_axum_protocol_mismatch_reason_mapping_gate");
}

const CHECKLIST_CONTAINS_SERVICE_API_AXUM_PROTOCOL_TAXONOMY_RUNBOOK_PARITY_GATE_MARKERS: &[&str] = &[
    "## Service API Axum Protocol Taxonomy/Runbook Parity Gate (Issue #4267)",
    "test_validate_service_api_axum_ingress_live_contract_lane.sh",
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
    "Regression: #4272",
    "Regression: #4273",
];

#[test]
fn checklist_contains_service_api_axum_protocol_taxonomy_runbook_parity_gate() {
    assert_checklist_contains_all(CHECKLIST_CONTAINS_SERVICE_API_AXUM_PROTOCOL_TAXONOMY_RUNBOOK_PARITY_GATE_MARKERS, "checklist_contains_service_api_axum_protocol_taxonomy_runbook_parity_gate");
}

const CHECKLIST_CONTAINS_SERVICE_API_AXUM_ADMISSION_DECISION_TAXONOMY_RUNBOOK_PARITY_GATE_MARKERS: &[&str] = &[
    "## Service API Axum Admission Decision Taxonomy/Runbook Parity Gate (Issues #4222, #4227, #4228)",
    "test_validate_service_api_axum_ingress_live_contract_lane.sh",
    "test_check_service_api_axum_ingress_live_policy.sh",
    "admission_decision_taxonomy_status=verified",
    "admission_decision_accept_status=verified",
    "admission_decision_defer_status=verified",
    "admission_decision_reject_status=verified",
    "admission_decision_reason_taxonomy_version=kamn.runtime.service-api-admission-decision-reason-taxonomy.v1",
    "admission_decision_reason_codes_csv=admission_decision_accept,admission_decision_defer,admission_decision_reject",
    "admission_decision_taxonomy_mapping_status=verified",
    "admission_decision_runbook_marker_parity_status=verified",
    "admission_decision_taxonomy_runbook_reason_taxonomy_version=kamn.runtime.service-api-axum-admission-decision-runbook-reason-taxonomy.v1",
    "admission_decision_taxonomy_runbook_reason_codes_csv=admission_decision_taxonomy_mapping_drift_detected,admission_runbook_marker_parity_mismatch",
    "service_api_axum_policy_admission_decision_reason_taxonomy_version_mismatch",
    "service_api_axum_policy_admission_decision_reason_codes_csv_mismatch",
    "service_api_axum_policy_marker_missing:admission_decision_defer_status",
    "protocol_taxonomy_mapping_drift_detected",
    "runbook_marker_parity_mismatch",
    "Regression: #4227",
    "Regression: #4228",
];

#[test]
fn checklist_contains_service_api_axum_admission_decision_taxonomy_runbook_parity_gate() {
    assert_checklist_contains_all(CHECKLIST_CONTAINS_SERVICE_API_AXUM_ADMISSION_DECISION_TAXONOMY_RUNBOOK_PARITY_GATE_MARKERS, "checklist_contains_service_api_axum_admission_decision_taxonomy_runbook_parity_gate");
}
