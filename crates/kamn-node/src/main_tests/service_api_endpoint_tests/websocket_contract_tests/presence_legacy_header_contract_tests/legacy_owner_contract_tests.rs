use super::super::support::*;

#[test]
fn regression_service_api_endpoint_websocket_presence_mode_rejects_legacy_owner_did_header() {
    let harness = build_websocket_harness("127.0.0.1:34062", 1);
    let response = legacy_owner_response(&harness);
    assert_websocket_bad_request(
        response,
        WS_PRESENCE_OWNER_DID_INVALID_REASON_CODE,
        Some("invalid presence owner did header"),
    );
    assert_server_ok(
        harness.server,
        "service api endpoint should stop cleanly after legacy owner websocket rejection",
    );
}

fn legacy_owner_response(harness: &WebsocketHarness) -> Vec<u8> {
    send_signed_websocket_request(
        &harness.snapshot,
        harness.bind_addr.as_str(),
        "kamn:did:agent:ws-presence-client-legacy-owner",
        44,
        &[
            ("X-KAMN-Events-Mode", "presence"),
            ("X-KAMN-Presence-Owner-DID", "did:kamn:owner:legacy-alpha"),
            (
                "X-KAMN-Presence-Target-Agent-DID",
                "kamn:did:agent:ws-presence-client-legacy-owner",
            ),
        ],
    )
}
