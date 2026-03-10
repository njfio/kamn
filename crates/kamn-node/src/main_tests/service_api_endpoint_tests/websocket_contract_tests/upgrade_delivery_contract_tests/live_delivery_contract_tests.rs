use super::super::super::*;
use super::super::support::*;
use crate::service_api_endpoint::ServiceApiSnapshot;

#[test]
fn regression_service_api_endpoint_websocket_stream_delivers_live_message_event_after_upgrade() {
    let harness = build_websocket_harness("127.0.0.1:34071", 6);
    let post_thread =
        spawn_live_message_publish_thread(harness.bind_addr.clone(), harness.snapshot.clone());
    let websocket_response = send_signed_websocket_request(
        &harness.snapshot,
        harness.bind_addr.as_str(),
        "kamn:did:agent:ws-live-stream-client",
        601,
        &[],
    );
    let (first_post_response, second_post_response) = post_thread
        .join()
        .expect("post request thread should complete");
    assert_publisher_requests_accepted(first_post_response.as_str(), second_post_response.as_str());
    assert_live_delivery_frames(websocket_response.as_slice());
    assert_server_ok_or_timeout(
        harness.server,
        "service api endpoint should end via request budget completion or idle-timeout fail-close after websocket live stream regression test",
    );
}

fn spawn_live_message_publish_thread(
    bind_addr: String,
    snapshot: ServiceApiSnapshot,
) -> thread::JoinHandle<(String, String)> {
    thread::spawn(move || {
        thread::sleep(Duration::from_millis(75));
        let hash = state_hash(&snapshot);
        let first = send_live_message(
            bind_addr.as_str(),
            hash.as_str(),
            602,
            "{\"message\":\"websocket-live-event-1\"}",
        );
        thread::sleep(Duration::from_millis(25));
        let second = send_live_message(
            bind_addr.as_str(),
            hash.as_str(),
            603,
            "{\"message\":\"websocket-live-event-2\"}",
        );
        (first, second)
    })
}

fn send_live_message(bind_addr: &str, state_hash: &str, nonce: u64, body: &str) -> String {
    let sender_did = "kamn:did:agent:ws-live-stream-publisher";
    let signature = service_api_request_signature_for_fields(sender_did, nonce, state_hash, body);
    let nonce_text = nonce.to_string();
    send_http_request_with_headers(
        bind_addr,
        "POST",
        "/v1/messages/send",
        body,
        &[
            ("X-KAMN-Sender-DID", sender_did),
            ("X-KAMN-Request-Nonce", nonce_text.as_str()),
            ("X-KAMN-Request-Signature", signature.as_str()),
            ("X-KAMN-Authz-Scope", "messages:write"),
        ],
    )
}

fn assert_publisher_requests_accepted(first_post_response: &str, second_post_response: &str) {
    assert!(
        first_post_response.contains("HTTP/1.1 202 Accepted"),
        "first publisher request should be accepted: {first_post_response}",
    );
    assert!(
        second_post_response.contains("HTTP/1.1 202 Accepted"),
        "second publisher request should be accepted: {second_post_response}",
    );
}

fn assert_live_delivery_frames(response: &[u8]) {
    let (_header, frames) = parse_websocket_response_frames(response);
    let mut unique_sequences = message_created_sequences(frames.as_slice());
    assert!(
        unique_sequences.len() >= 2,
        "websocket stream should include multiple live message-created event frames after upgrade: {frames:?}",
    );
    unique_sequences.sort_unstable();
    unique_sequences.dedup();
    assert!(
        unique_sequences.len() >= 2 && unique_sequences[1] > unique_sequences[0],
        "message-created websocket event sequence should advance across events: {unique_sequences:?}",
    );
}

fn message_created_sequences(frames: &[String]) -> Vec<u64> {
    frames
        .iter()
        .filter_map(|frame| {
            let payload: Value = serde_json::from_str(frame).ok()?;
            if payload.get("event").and_then(Value::as_str) != Some("service-api.message.created") {
                return None;
            }
            payload.get("sequence").and_then(Value::as_u64)
        })
        .collect()
}
