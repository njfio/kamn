use super::super::super::*;
use super::super::support::*;
use crate::service_api_endpoint::ServiceApiSnapshot;

const LIVE_SOLANA_DEVNET_RPC_URL: &str = "https://api.devnet.solana.com";

#[test]
fn integration_service_api_endpoint_websocket_streams_live_bridge_forwarded_event_after_upgrade() {
    let _env = acquire_service_api_test_env();
    let _live_rpc_guard = EnvVarGuard::set(
        "KAMN_SERVICE_API_LIVE_SOLANA_BRIDGE_RPC_URL",
        Some(LIVE_SOLANA_DEVNET_RPC_URL),
    );
    let harness = build_websocket_harness("127.0.0.1:34072", 3);
    let publisher = spawn_live_bridge_publish_thread(harness.bind_addr.clone(), harness.snapshot.clone());
    let websocket_response = send_signed_websocket_request(
        &harness.snapshot,
        harness.bind_addr.as_str(),
        "kamn:did:agent:ws-live-bridge-client",
        701,
        &[],
    );
    let (submit_response, forward_response) = publisher
        .join()
        .expect("bridge publish thread should complete");
    assert_live_bridge_requests_accepted(submit_response.as_str(), forward_response.as_str());
    assert_live_bridge_forwarded_frame(websocket_response.as_slice());
    assert_server_ok_or_timeout(
        harness.server,
        "service api endpoint should end via request budget completion or idle-timeout fail-close after websocket live bridge test",
    );
}

fn spawn_live_bridge_publish_thread(
    bind_addr: String,
    snapshot: ServiceApiSnapshot,
) -> thread::JoinHandle<(String, String)> {
    thread::spawn(move || {
        thread::sleep(Duration::from_millis(75));
        let state_hash = state_hash(&snapshot);
        let submit_response = submit_live_bridge(bind_addr.as_str(), state_hash.as_str(), 702);
        let bridge_id = submitted_bridge_id(submit_response.as_str());
        let forward_response =
            forward_live_bridge(bind_addr.as_str(), state_hash.as_str(), 703, bridge_id.as_str());
        (submit_response, forward_response)
    })
}

fn submit_live_bridge(bind_addr: &str, state_hash: &str, nonce: u64) -> String {
    let sender_did = "kamn:did:agent:ws-live-bridge-publisher";
    let body = r#"{"source_message_id":"msg-ws-live-bridge-source"}"#;
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
    let sender_did = "kamn:did:agent:ws-live-bridge-publisher";
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

fn submitted_bridge_id(submit_response: &str) -> String {
    let body = extract_http_response_body(submit_response);
    let payload: Value =
        parse_service_api_payload(body).expect("bridge submit payload should deserialize");
    payload["bridge_id"]
        .as_str()
        .expect("bridge submit payload should include bridge id")
        .to_owned()
}

fn assert_live_bridge_forwarded_frame(response: &[u8]) {
    let (_header, frames) = parse_websocket_response_frames(response);
    let forwarded = bridge_forwarded_frames(frames.as_slice());
    assert!(
        !forwarded.is_empty(),
        "websocket stream should include a bridge forwarded event after live bridge forward: {frames:?}",
    );
    let payload = &forwarded[0];
    let bridge_id = payload["bridge_id"]
        .as_str()
        .expect("bridge forwarded frame must include bridge id");
    assert_eq!(
        payload["event"].as_str(),
        Some("service-api.bridge.forwarded")
    );
    assert_ne!(
        payload["target_message_id"],
        Value::String(format!("msg-bridge-target-{bridge_id}")),
        "live websocket bridge target id must not be placeholder"
    );
    assert_ne!(
        payload["forward_tx_hash"],
        Value::String(format!("sha256:bridge-forwarded-{bridge_id}")),
        "live websocket bridge forward hash must not be placeholder"
    );
}

fn bridge_forwarded_frames(frames: &[String]) -> Vec<Value> {
    frames
        .iter()
        .filter_map(|frame| {
            let payload: Value = serde_json::from_str(frame).ok()?;
            if payload.get("event").and_then(Value::as_str) != Some("service-api.bridge.forwarded") {
                return None;
            }
            Some(payload)
        })
        .collect()
}
