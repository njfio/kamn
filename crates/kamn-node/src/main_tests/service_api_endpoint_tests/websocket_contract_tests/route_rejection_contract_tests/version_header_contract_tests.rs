use super::super::support::*;

#[test]
fn regression_service_api_endpoint_websocket_rejects_invalid_version_header() {
    let harness = build_websocket_harness("127.0.0.1:34057", 1);
    let response = send_signed_websocket_request_with_version(
        &harness.snapshot,
        harness.bind_addr.as_str(),
        "kamn:did:agent:ws-client-3",
        29,
        "12",
        &[],
    );
    assert_websocket_bad_request(
        response,
        "service_api_ws_version_header_invalid",
        Some("invalid websocket version header"),
    );
    assert_server_ok(
        harness.server,
        "service api endpoint should stop cleanly after websocket version rejection",
    );
}
