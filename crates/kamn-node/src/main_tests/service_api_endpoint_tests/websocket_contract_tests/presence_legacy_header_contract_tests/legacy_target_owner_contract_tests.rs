use super::super::support::*;

#[test]
fn regression_service_api_endpoint_websocket_presence_mode_rejects_legacy_target_owner_did_header() {
    let harness = build_websocket_harness("127.0.0.1:34063", 1);
    let response = legacy_target_owner_response(&harness);
    assert_websocket_bad_request(
        response,
        WS_PRESENCE_TARGET_OWNER_DID_INVALID_REASON_CODE,
        Some("invalid presence target owner did header"),
    );
    assert_server_ok(
        harness.server,
        "service api endpoint should stop cleanly after legacy target owner websocket rejection",
    );
}

fn legacy_target_owner_response(harness: &WebsocketHarness) -> Vec<u8> {
    send_signed_websocket_request(
        &harness.snapshot,
        harness.bind_addr.as_str(),
        "kamn:did:agent:ws-presence-client-legacy-target-owner",
        45,
        &[
            ("X-KAMN-Events-Mode", "presence"),
            ("X-KAMN-Presence-Owner-DID", "kamn:did:owner:alpha"),
            (
                "X-KAMN-Presence-Target-Owner-DID",
                "did:kamn:owner:legacy-beta",
            ),
            (
                "X-KAMN-Presence-Target-Agent-DID",
                "kamn:did:agent:ws-presence-client-legacy-target-owner",
            ),
        ],
    )
}
