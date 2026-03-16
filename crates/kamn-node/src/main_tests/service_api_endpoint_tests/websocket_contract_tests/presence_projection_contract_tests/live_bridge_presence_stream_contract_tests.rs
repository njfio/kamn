use super::super::super::*;
use super::super::support::*;
use crate::service_api_endpoint::ServiceApiSnapshot;
use serde_json::Value;
use std::thread;
use std::time::Duration;

const LIVE_SOLANA_DEVNET_RPC_URL: &str = "https://api.devnet.solana.com";

#[test]
fn integration_service_api_endpoint_websocket_presence_mode_streams_live_bridge_projection_event() {
    let (_env, _live_rpc_guard, harness) = build_live_bridge_presence_harness();
    let publisher = spawn_live_bridge_publish_thread(harness.bind_addr.clone(), harness.snapshot.clone());
    let sender_did = test_service_api_sender_did("kamn:did:agent:ws-live-bridge-presence-client");
    let websocket_response = send_live_presence_request(
        &harness.snapshot,
        harness.bind_addr.as_str(),
        sender_did.as_str(),
        711,
    );
    let (submit_response, forward_response) = publisher.join().expect("bridge publish thread should complete");
    assert_live_bridge_requests_accepted(submit_response.as_str(), forward_response.as_str());
    super::assert_presence_projection_response(websocket_response.as_slice(), sender_did.as_str());
    assert_live_bridge_forwarded_frame(websocket_response.as_slice());
    assert_server_ok_or_timeout(harness.server, "service api endpoint should preserve live bridge presence frames");
}

fn build_live_bridge_presence_harness() -> (ServiceApiTestEnvGuards, EnvVarGuard, WebsocketHarness) {
    let env = acquire_service_api_test_env();
    let live_rpc_guard =
        EnvVarGuard::set("KAMN_SERVICE_API_LIVE_SOLANA_BRIDGE_RPC_URL", Some(LIVE_SOLANA_DEVNET_RPC_URL));
    let harness = build_websocket_harness("127.0.0.1:34074", 3);
    (env, live_rpc_guard, harness)
}

fn send_live_presence_request(
    snapshot: &ServiceApiSnapshot,
    bind_addr: &str,
    sender_did: &str,
    nonce: u64,
) -> Vec<u8> {
    let signature = websocket_signature(snapshot, sender_did, nonce);
    let nonce_text = nonce.to_string();
    let mut headers = vec![
        ("X-KAMN-Sender-DID", sender_did),
        ("X-KAMN-Request-Nonce", nonce_text.as_str()),
        ("X-KAMN-Request-Signature", signature.as_str()),
    ];
    headers.extend_from_slice(presence_headers(sender_did).as_slice());
    send_websocket_upgrade_request_with_timeout(
        bind_addr,
        WEBSOCKET_EVENTS_PATH,
        "13",
        Duration::from_secs(4),
        headers.as_slice(),
    )
}

fn presence_headers(sender_did: &str) -> [(&str, &str); 8] {
    [
        ("X-KAMN-Events-Mode", "presence"),
        ("X-KAMN-Presence-Owner-DID", "kamn:did:owner:alpha"),
        ("X-KAMN-Presence-Target-Owner-DID", "kamn:did:owner:alpha"),
        ("X-KAMN-Presence-Target-Agent-DID", sender_did),
        ("X-KAMN-Presence-Gateway-Node", "gateway-alpha"),
        ("X-KAMN-Presence-Connected-Since", "1709000000"),
        ("X-KAMN-Presence-Last-Heartbeat", "1709000005"),
        ("X-KAMN-Presence-Capabilities", "ws,notify"),
    ]
}

fn spawn_live_bridge_publish_thread(
    bind_addr: String,
    snapshot: ServiceApiSnapshot,
) -> thread::JoinHandle<(String, String)> {
    thread::spawn(move || {
        thread::sleep(Duration::from_millis(75));
        let state_hash = state_hash(&snapshot);
        let submit_response = submit_live_bridge(bind_addr.as_str(), state_hash.as_str(), 712);
        let bridge_id = submitted_bridge_id(submit_response.as_str());
        let forward_response =
            forward_live_bridge(bind_addr.as_str(), state_hash.as_str(), 713, bridge_id.as_str());
        (submit_response, forward_response)
    })
}

fn submit_live_bridge(bind_addr: &str, state_hash: &str, nonce: u64) -> String {
    let sender_did = "kamn:did:agent:ws-live-bridge-presence-publisher";
    let body = r#"{"source_message_id":"msg-ws-live-bridge-presence-source"}"#;
    let signature = service_api_request_signature_for_fields(sender_did, nonce, state_hash, body);
    let nonce_text = nonce.to_string();
    send_http_request_with_headers(
        bind_addr,
        "POST",
        "/v1/bridge/submit",
        body,
        &[
            ("X-KAMN-Sender-DID", sender_did),
            ("X-KAMN-Request-Nonce", nonce_text.as_str()),
            ("X-KAMN-Request-Signature", signature.as_str()),
            ("X-KAMN-Authz-Scope", "bridge:write"),
        ],
    )
}

fn forward_live_bridge(bind_addr: &str, state_hash: &str, nonce: u64, bridge_id: &str) -> String {
    let sender_did = "kamn:did:agent:ws-live-bridge-presence-publisher";
    let signature = service_api_request_signature_for_fields(sender_did, nonce, state_hash, "");
    let nonce_text = nonce.to_string();
    send_http_request_with_headers(
        bind_addr,
        "POST",
        format!("/v1/bridge/{bridge_id}/forward").as_str(),
        "",
        &[
            ("X-KAMN-Sender-DID", sender_did),
            ("X-KAMN-Request-Nonce", nonce_text.as_str()),
            ("X-KAMN-Request-Signature", signature.as_str()),
            ("X-KAMN-Authz-Scope", "bridge:write"),
        ],
    )
}

fn submitted_bridge_id(submit_response: &str) -> String {
    let payload: Value = parse_service_api_payload(extract_http_response_body(submit_response))
        .expect("bridge submit payload should deserialize");
    payload["bridge_id"]
        .as_str()
        .expect("bridge submit payload should include bridge id")
        .to_owned()
}

fn assert_live_bridge_requests_accepted(submit_response: &str, forward_response: &str) {
    assert!(
        submit_response.contains("HTTP/1.1 202 Accepted"),
        "live bridge submit should be accepted: {submit_response}",
    );
    assert!(
        forward_response.contains("HTTP/1.1 200 OK"),
        "live bridge forward should be accepted: {forward_response}",
    );
}

fn assert_live_bridge_forwarded_frame(response: &[u8]) {
    let (_header, frames) = parse_websocket_response_frames(response);
    let forwarded = bridge_forwarded_frames(frames.as_slice());
    assert!(
        !forwarded.is_empty(),
        "presence-mode websocket stream should include a bridge forwarded event: {frames:?}",
    );
    let payload = &forwarded[0];
    assert_eq!(
        payload["event"].as_str(),
        Some("service-api.bridge.forwarded")
    );
    assert!(payload["target_message_id"].as_str().is_some());
    assert!(payload["forward_tx_hash"].as_str().is_some());
}

fn bridge_forwarded_frames(frames: &[String]) -> Vec<Value> {
    frames
        .iter()
        .filter_map(|frame| serde_json::from_str::<Value>(frame).ok())
        .filter(|payload| payload.get("event").and_then(Value::as_str) == Some("service-api.bridge.forwarded"))
        .collect()
}
