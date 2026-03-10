use super::super::super::*;
use super::super::support::*;

#[test]
fn integration_service_api_endpoint_websocket_upgrade_streams_state_transition_event() {
    let harness = build_websocket_harness("127.0.0.1:34055", 1);
    let response = send_signed_websocket_request(
        &harness.snapshot,
        harness.bind_addr.as_str(),
        "kamn:did:agent:ws-client-1",
        19,
        &[],
    );
    assert_upgrade_state_transition_response(response.as_slice());
    assert_server_ok(
        harness.server,
        "service api endpoint should stop cleanly after websocket request budget",
    );
}

#[test]
fn integration_service_api_endpoint_websocket_upgrade_keeps_connection_open_after_initial_event() {
    let harness = build_websocket_harness("127.0.0.1:34065", 2);
    let read_start = Instant::now();
    let (response, peer_closed) = send_signed_websocket_request_with_close_observation(
        &harness.snapshot,
        harness.bind_addr.as_str(),
        "kamn:did:agent:ws-client-multi",
        57,
        &[],
    );
    assert_initial_state_transition_frame(response.as_slice());
    assert_connection_remains_open(peer_closed, read_start.elapsed());
    assert_server_ok_or_timeout(
        harness.server,
        "websocket keep-open test should end via request budget completion or idle-timeout fail-close",
    );
}

fn assert_upgrade_state_transition_response(response: &[u8]) {
    let (header, payload) = parse_websocket_response(response);
    assert!(header.contains("HTTP/1.1 101 Switching Protocols"));
    let normalized_header = header.to_ascii_lowercase();
    assert!(normalized_header.contains("upgrade: websocket"));
    assert!(normalized_header.contains("x-kamn-websocket-contract: v1"));
    assert!(payload.contains("\"event\":\"state-transition\""));
    assert!(payload.contains("\"runtime_mode\":\"api\""));
    assert!(payload.contains("\"role\":\"processor\""));
}

fn assert_initial_state_transition_frame(response: &[u8]) {
    let (_header, frames) = parse_websocket_response_frames(response);
    assert!(
        !frames.is_empty(),
        "websocket stream should emit an initial state-transition event frame"
    );
    let first: Value = serde_json::from_str(frames[0].as_str())
        .expect("initial websocket state-transition frame should be json");
    assert_eq!(
        first.get("event").and_then(Value::as_str),
        Some("state-transition")
    );
    assert_eq!(first.get("sequence").and_then(Value::as_u64), Some(1));
}

fn assert_connection_remains_open(peer_closed: bool, read_elapsed: Duration) {
    let remained_open_or_timed_out = !peer_closed || read_elapsed >= Duration::from_millis(1_500);
    assert!(
        remained_open_or_timed_out,
        "websocket stream should not close immediately after initial frame; peer_closed={peer_closed} elapsed={read_elapsed:?}",
    );
}
