const DOC: &str = include_str!("../../../docs/service/api-contract.md");

#[test]
fn service_api_contract_contains_websocket_invalid_frame_handling_matrix() {
    assert!(DOC.contains("## Invalid-Frame Handling Matrix"));
    assert!(DOC.contains("X-KAMN-WebSocket-Contract != v1"));
    assert!(DOC.contains(
        "service_api_websocket_session_reason_taxonomy_version=kamn.runtime.service-api.websocket-session-reason-taxonomy.v1"
    ));
    assert!(DOC.contains("service_api_ws_protocol_contract_drift_detected"));
    assert!(DOC.contains("service_api_ws_session_frame_too_short"));
    assert!(DOC.contains("service_api_ws_session_frame_opcode_invalid"));
    assert!(DOC.contains("service_api_ws_session_frame_mask_invalid"));
    assert!(DOC.contains("service_api_ws_session_frame_length_mismatch"));
    assert!(DOC.contains("service_api_ws_session_frame_payload_utf8_invalid"));
    assert!(DOC.contains("Regression: #4317"));
}
