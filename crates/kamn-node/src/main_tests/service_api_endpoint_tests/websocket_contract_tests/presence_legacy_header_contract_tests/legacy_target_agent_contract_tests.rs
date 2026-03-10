use super::super::support::*;

#[test]
fn regression_service_api_endpoint_websocket_presence_mode_rejects_legacy_target_agent_did_header() {
    let harness = build_websocket_harness("127.0.0.1:34064", 1);
    let response = legacy_target_agent_response(&harness);
    assert_websocket_bad_request(
        response,
        WS_PRESENCE_TARGET_AGENT_DID_INVALID_REASON_CODE,
        Some("invalid presence target agent did header"),
    );
    assert_server_ok(
        harness.server,
        "service api endpoint should stop cleanly after legacy target agent websocket rejection",
    );
}

fn legacy_target_agent_response(harness: &WebsocketHarness) -> Vec<u8> {
    send_signed_websocket_request(
        &harness.snapshot,
        harness.bind_addr.as_str(),
        "kamn:did:agent:ws-presence-client-legacy-target-agent",
        46,
        &[
            ("X-KAMN-Events-Mode", "presence"),
            ("X-KAMN-Presence-Owner-DID", "kamn:did:owner:alpha"),
            (
                "X-KAMN-Presence-Target-Agent-DID",
                "did:kamn:agent:legacy-gamma",
            ),
        ],
    )
}
