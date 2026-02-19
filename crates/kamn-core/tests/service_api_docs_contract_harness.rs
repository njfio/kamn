const DOC: &str = include_str!("../../../docs/service/api-contract.md");

struct DocContractCase {
    case_id: &'static str,
    document_label: &'static str,
    document: &'static str,
    required_markers: &'static [&'static str],
}

const DOC_CONTRACT_CASES: &[DocContractCase] = &[
    DocContractCase {
        case_id: "service_api_invalid_frame_handling_matrix_markers",
        document_label: "docs/service/api-contract.md",
        document: DOC,
        required_markers: &[
            "## Invalid-Frame Handling Matrix",
            "X-KAMN-WebSocket-Contract != v1",
            "service_api_websocket_session_reason_taxonomy_version=kamn.runtime.service-api.websocket-session-reason-taxonomy.v1",
            "service_api_ws_protocol_contract_drift_detected",
            "service_api_ws_session_frame_too_short",
            "service_api_ws_session_frame_opcode_invalid",
            "service_api_ws_session_frame_mask_invalid",
            "service_api_ws_session_frame_length_mismatch",
            "service_api_ws_session_frame_payload_utf8_invalid",
            "Regression: #4317",
        ],
    },
    DocContractCase {
        case_id: "service_api_async_lifecycle_rejection_taxonomy_markers",
        document_label: "docs/service/api-contract.md",
        document: DOC,
        required_markers: &[
            "## Async Lifecycle Rejection Taxonomy (Issue #4316)",
            "service_api_lifecycle_rejection_reason_taxonomy_version=kamn.runtime.service-api.lifecycle-rejection-reason-taxonomy.v1",
            "service_api_ingress_concurrency_limit_exceeded",
            "service_api_ingress_rate_limit_exceeded",
            "service_api_ingress_sender_rate_limit_exceeded",
            "service_api_ingress_sender_suspended",
            "service_api_ingress_sender_duplicate_message_id",
            "service_api_ingress_sender_insufficient_deposit",
            "service_api_ingress_anti_spam_engine_invalid",
            "## Async Lifecycle Rejection Projection Matrix",
            "async-lifecycle-limiter",
            "sender-admission-limiter",
            "async-lifecycle-engine",
            "Regression: #4316",
        ],
    },
];

#[test]
fn functional_service_api_doc_contract_cases_require_markers() {
    for case in DOC_CONTRACT_CASES {
        for marker in case.required_markers {
            assert!(
                case.document.contains(marker),
                "missing marker in {} for case {}: {}",
                case.document_label,
                case.case_id,
                marker
            );
        }
    }
}

#[test]
fn regression_service_api_doc_contract_case_inventory_remains_stable() {
    // Regression: #5193
    assert_eq!(DOC_CONTRACT_CASES.len(), 2);
    assert!(DOC_CONTRACT_CASES
        .iter()
        .all(|case| !case.required_markers.is_empty()));
    let total_marker_count: usize = DOC_CONTRACT_CASES
        .iter()
        .map(|case| case.required_markers.len())
        .sum();
    assert_eq!(total_marker_count, 24);
}
