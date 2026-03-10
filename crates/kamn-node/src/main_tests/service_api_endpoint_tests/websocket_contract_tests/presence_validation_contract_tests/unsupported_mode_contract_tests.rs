use super::super::support::*;

#[test]
fn regression_service_api_endpoint_websocket_presence_mode_rejects_unsupported_mode() {
    let harness = build_websocket_harness("127.0.0.1:34059", 1);
    let response = send_signed_websocket_request(
        &harness.snapshot,
        harness.bind_addr.as_str(),
        "kamn:did:agent:ws-presence-client-unsupported",
        37,
        &[("X-KAMN-Events-Mode", "presence-v2")],
    );
    assert_websocket_bad_request(response, "service_api_ws_events_mode_invalid", None);
    assert_server_ok(
        harness.server,
        "service api endpoint should stop cleanly after unsupported websocket mode rejection",
    );
}
