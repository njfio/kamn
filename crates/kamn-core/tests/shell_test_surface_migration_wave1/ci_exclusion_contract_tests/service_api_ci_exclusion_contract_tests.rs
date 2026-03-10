use crate::ci_exclusion_contract_tests::support::{
    assert_ci_tools_surface_and_doc, assert_fast_gate_exclusion, load_ci_exclusion_context,
};

#[test]
fn spec_c11_service_api_reason_code_compatibility_ci_exclusion_policy_markers() {
    assert_service_api_ci_exclusion(
        "bash scripts/runtime/validate_service_api_reason_code_compatibility_live.sh",
        "bash \"$ROOT_DIR/scripts/runtime/test_validate_service_api_reason_code_compatibility_live_contract_lane.sh\"",
        &[
            "bash \"$ROOT_DIR/scripts/runtime/test_check_service_api_reason_code_compatibility_live_policy.sh\"",
            "bash \"$ROOT_DIR/scripts/runtime/test_validate_service_api_reason_code_compatibility_live_contract_lane.sh\"",
        ],
        "ci-tools service API reason-code compatibility command surface",
        "service api reason-code compatibility contract-lane commands remain excluded from ci-fast-gate and ci-tools fast mode.",
        "service API reason-code compatibility lane",
    );
}

#[test]
fn spec_c12_service_api_serde_payload_parity_ci_exclusion_policy_markers() {
    assert_service_api_ci_exclusion(
        "bash scripts/runtime/validate_service_api_serde_payload_parity_live.sh",
        "bash \"$ROOT_DIR/scripts/runtime/test_validate_service_api_serde_payload_parity_live_contract_lane.sh\"",
        &[
            "bash \"$ROOT_DIR/scripts/runtime/test_check_service_api_serde_payload_parity_live_policy.sh\"",
            "bash \"$ROOT_DIR/scripts/runtime/test_validate_service_api_serde_payload_parity_live_contract_lane.sh\"",
        ],
        "ci-tools service API serde payload parity command surface",
        "service api serde payload parity contract-lane commands remain excluded from ci-fast-gate and ci-tools fast mode.",
        "service API serde payload parity lane",
    );
}

#[test]
fn spec_c13_service_api_validation_negative_matrix_ci_exclusion_policy_markers() {
    assert_service_api_ci_exclusion(
        "bash scripts/runtime/validate_service_api_validation_negative_matrix_live.sh --mode run",
        "bash \"$ROOT_DIR/scripts/runtime/test_validate_service_api_validation_negative_matrix_live_contract_lane.sh\"",
        &[
            "bash \"$ROOT_DIR/scripts/runtime/test_check_service_api_validation_negative_matrix_live_policy.sh\"",
            "bash \"$ROOT_DIR/scripts/runtime/test_validate_service_api_validation_negative_matrix_live_contract_lane.sh\"",
        ],
        "ci-tools service API validation negative-matrix command surface",
        "service api validation negative-matrix contract-lane commands remain excluded from ci-fast-gate and ci-tools fast mode.",
        "service API validation negative-matrix lane",
    );
}

fn assert_service_api_ci_exclusion(
    workflow_marker: &str,
    fast_mode_marker: &str,
    ci_tools_markers: &[&str],
    ci_tools_label: &str,
    doc_marker: &str,
    lane_label: &str,
) {
    let context = load_ci_exclusion_context();
    assert_fast_gate_exclusion(&context, workflow_marker, fast_mode_marker, lane_label);
    assert_ci_tools_surface_and_doc(
        &context,
        ci_tools_markers,
        ci_tools_label,
        doc_marker,
        lane_label,
    );
}
