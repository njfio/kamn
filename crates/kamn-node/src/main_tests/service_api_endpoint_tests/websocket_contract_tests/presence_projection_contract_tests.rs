use super::support::*;
use super::*;

#[test]
fn integration_service_api_endpoint_websocket_presence_mode_streams_bridge_projection_event() {
    let harness = build_websocket_harness("127.0.0.1:34058", 1);
    let sender_did = test_service_api_sender_did("kamn:did:agent:ws-presence-client-1");
    let response = send_signed_websocket_request(
        &harness.snapshot,
        harness.bind_addr.as_str(),
        sender_did.as_str(),
        31,
        &[
            ("X-KAMN-Events-Mode", "presence"),
            ("X-KAMN-Presence-Owner-DID", "kamn:did:owner:alpha"),
            ("X-KAMN-Presence-Target-Owner-DID", "kamn:did:owner:alpha"),
            ("X-KAMN-Presence-Target-Agent-DID", sender_did.as_str()),
            ("X-KAMN-Presence-Gateway-Node", "gateway-alpha"),
            ("X-KAMN-Presence-Connected-Since", "1709000000"),
            ("X-KAMN-Presence-Last-Heartbeat", "1709000005"),
            ("X-KAMN-Presence-Capabilities", "ws,notify"),
        ],
    );
    assert_presence_projection_response(response.as_slice(), sender_did.as_str());
    assert_server_ok(
        harness.server,
        "service api endpoint should stop cleanly after websocket presence request budget",
    );
}

fn assert_presence_projection_response(response: &[u8], sender_did: &str) {
    let (header, payload) = parse_websocket_response(response);
    assert!(header.contains("HTTP/1.1 101 Switching Protocols"));
    let payload_json: Value =
        serde_json::from_str(payload.as_str()).expect("presence websocket payload should be json");
    assert_presence_identity_fields(&payload_json, sender_did);
    assert_presence_visibility_fields(&payload_json);
}

fn assert_presence_identity_fields(payload_json: &Value, sender_did: &str) {
    assert_eq!(payload_json.get("event").and_then(Value::as_str), Some("m9.presence.snapshot"));
    assert_eq!(payload_json.get("transport_profile").and_then(Value::as_str), Some("websocket"));
    assert_eq!(payload_json.get("requester_owner_did").and_then(Value::as_str), Some("kamn:did:owner:alpha"));
    assert_eq!(payload_json.get("requester_agent_did").and_then(Value::as_str), Some(sender_did));
    assert_eq!(payload_json.get("target_owner_did").and_then(Value::as_str), Some("kamn:did:owner:alpha"));
    assert_eq!(payload_json.get("target_agent_did").and_then(Value::as_str), Some(sender_did));
}

fn assert_presence_visibility_fields(payload_json: &Value) {
    assert_eq!(payload_json.get("visible").and_then(Value::as_bool), Some(true));
    assert_eq!(payload_json.get("target_gateway_node").and_then(Value::as_str), Some("gateway-alpha"));
    assert_eq!(
        payload_json
            .get("target_last_heartbeat_epoch_seconds")
            .and_then(Value::as_u64),
        Some(1_709_000_005),
    );
    assert_eq!(payload_json.get("reason_code").and_then(Value::as_str), Some("m9_gateway_presence_visible"));
}
