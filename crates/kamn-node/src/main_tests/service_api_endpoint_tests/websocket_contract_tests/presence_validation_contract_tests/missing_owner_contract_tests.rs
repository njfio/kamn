use super::super::support::*;

#[test]
fn regression_service_api_endpoint_websocket_presence_mode_rejects_missing_owner_header() {
    let harness = build_websocket_harness("127.0.0.1:34060", 1);
    let response = send_signed_websocket_request(
        &harness.snapshot,
        harness.bind_addr.as_str(),
        "kamn:did:agent:ws-presence-client-missing-owner",
        43,
        &[
            ("X-KAMN-Events-Mode", "presence"),
            (
                "X-KAMN-Presence-Target-Agent-DID",
                "kamn:did:agent:ws-presence-client-missing-owner",
            ),
        ],
    );
    assert_websocket_bad_request(
        response,
        "service_api_ws_presence_owner_did_header_missing",
        None,
    );
    assert_server_ok(
        harness.server,
        "service api endpoint should stop cleanly after missing-owner websocket rejection",
    );
}
