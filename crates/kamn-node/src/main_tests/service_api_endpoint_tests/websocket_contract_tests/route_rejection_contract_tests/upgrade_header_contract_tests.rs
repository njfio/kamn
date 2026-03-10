use super::super::super::*;
use super::super::support::*;

#[test]
fn regression_service_api_endpoint_websocket_route_rejects_missing_upgrade_headers() {
    let harness = build_websocket_harness("127.0.0.1:34056", 1);
    let signature = websocket_signature(&harness.snapshot, "kamn:did:agent:ws-client-2", 23);
    let response = send_http_request_with_headers(
        harness.bind_addr.as_str(),
        "GET",
        WEBSOCKET_EVENTS_PATH,
        "",
        &[
            ("X-KAMN-Sender-DID", "kamn:did:agent:ws-client-2"),
            ("X-KAMN-Request-Nonce", "23"),
            ("X-KAMN-Request-Signature", signature.as_str()),
        ],
    );
    assert_upgrade_header_missing_response(response.as_str());
    assert_server_ok(
        harness.server,
        "service api endpoint should stop cleanly after websocket rejection budget",
    );
}

fn assert_upgrade_header_missing_response(response: &str) {
    assert!(response.contains("HTTP/1.1 400 Bad Request"));
    let payload = parse_error_envelope_from_http_response(response);
    assert_eq!(payload.error, "bad-request");
    assert_eq!(payload.reason_code, "service_api_ws_upgrade_header_missing");
    assert!(payload
        .message
        .contains("missing required websocket upgrade header"));
}
