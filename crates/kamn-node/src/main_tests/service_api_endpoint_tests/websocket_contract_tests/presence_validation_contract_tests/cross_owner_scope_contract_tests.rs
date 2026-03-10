use super::super::support::*;

#[test]
fn regression_service_api_endpoint_websocket_presence_mode_rejects_cross_owner_scope() {
    let harness = build_websocket_harness("127.0.0.1:34061", 1);
    let response = cross_owner_scope_response(&harness);
    assert_websocket_forbidden(response, "m9_realtime_owner_scope_denied");
    assert_server_ok(
        harness.server,
        "service api endpoint should stop cleanly after cross-owner websocket rejection",
    );
}

fn cross_owner_scope_response(harness: &WebsocketHarness) -> Vec<u8> {
    send_signed_websocket_request(
        &harness.snapshot,
        harness.bind_addr.as_str(),
        "kamn:did:agent:ws-presence-client-scope",
        41,
        &[
            ("X-KAMN-Events-Mode", "presence"),
            ("X-KAMN-Presence-Owner-DID", "kamn:did:owner:alpha"),
            ("X-KAMN-Presence-Target-Owner-DID", "kamn:did:owner:beta"),
            (
                "X-KAMN-Presence-Target-Agent-DID",
                "kamn:did:agent:beta-target",
            ),
            ("X-KAMN-Presence-Gateway-Node", "gateway-beta"),
            ("X-KAMN-Presence-Connected-Since", "1709000100"),
            ("X-KAMN-Presence-Last-Heartbeat", "1709000105"),
        ],
    )
}
