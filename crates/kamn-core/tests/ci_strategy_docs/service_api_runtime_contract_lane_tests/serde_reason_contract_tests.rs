use super::support::assert_runtime_lane_markers;

#[test]
fn doc_contains_runtime_service_api_serde_payload_parity_contract_lane_ci_mode_markers() {
    assert_runtime_lane_markers(
        "## Runtime Service API Serde Payload Parity Contract Lane",
        &[
            "validate_service_api_serde_payload_parity_live.sh --output-json /tmp/service-api-serde-payload-parity-live-summary.json",
            "check_service_api_serde_payload_parity_live_policy.sh --report-file /tmp/service-api-serde-payload-parity-live-summary.json --expected-final-decision GO --ci-fast-gate PASS --output-json /tmp/service-api-serde-payload-parity-policy.json",
            "validate_service_api_serde_payload_parity_live_contract_lane.sh --output-json /tmp/service-api-serde-payload-parity-contract-lane-report.json --policy-output-json /tmp/service-api-serde-payload-parity-policy.json",
            "test_validate_service_api_serde_payload_parity_live_contract_lane.sh",
            "test_check_service_api_serde_payload_parity_live_policy.sh",
        ],
        "service api serde payload parity contract-lane commands remain excluded from ci-fast-gate and ci-tools fast mode.",
        &["service_api_serde_payload_policy_marker_missing:route_payload_parity_status"],
        "service api serde payload parity",
    );
}

#[test]
fn doc_contains_runtime_service_api_reason_code_compatibility_contract_lane_ci_mode_markers() {
    assert_runtime_lane_markers(
        "## Runtime Service API Reason-Code Compatibility Contract Lane",
        &[
            "validate_service_api_reason_code_compatibility_live.sh --output-json /tmp/service-api-reason-code-compatibility-live-summary.json",
            "check_service_api_reason_code_compatibility_live_policy.sh --report-file /tmp/service-api-reason-code-compatibility-live-summary.json --expected-final-decision GO --ci-fast-gate PASS --output-json /tmp/service-api-reason-code-compatibility-policy.json",
            "validate_service_api_reason_code_compatibility_live_contract_lane.sh --output-json /tmp/service-api-reason-code-compatibility-contract-lane-report.json --policy-output-json /tmp/service-api-reason-code-compatibility-policy.json",
            "test_validate_service_api_reason_code_compatibility_live_contract_lane.sh",
            "test_check_service_api_reason_code_compatibility_live_policy.sh",
        ],
        "service api reason-code compatibility contract-lane commands remain excluded from ci-fast-gate and ci-tools fast mode.",
        &["service_api_reason_code_policy_marker_missing:route_error_mapping_status"],
        "service api reason-code compatibility",
    );
}
