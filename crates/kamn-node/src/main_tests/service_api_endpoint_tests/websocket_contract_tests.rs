use super::*;

fn send_websocket_upgrade_request(addr: &str, path: &str, headers: &[(&str, &str)]) -> Vec<u8> {
    send_websocket_upgrade_request_with_version(addr, path, "13", headers)
}

fn send_websocket_upgrade_request_with_version(
    addr: &str,
    path: &str,
    websocket_version: &str,
    headers: &[(&str, &str)],
) -> Vec<u8> {
    send_websocket_upgrade_request_with_version_close_observation(
        addr,
        path,
        websocket_version,
        headers,
    )
    .0
}

fn send_websocket_upgrade_request_with_version_close_observation(
    addr: &str,
    path: &str,
    websocket_version: &str,
    headers: &[(&str, &str)],
) -> (Vec<u8>, bool) {
    let mut stream = TcpStream::connect(addr).expect("endpoint should accept websocket connection");
    stream
        .set_read_timeout(Some(Duration::from_secs(2)))
        .expect("websocket read timeout should be configurable");
    let enriched_headers = enrich_signed_headers_with_scope("GET", path, headers);
    let mut header_lines = String::new();
    for (name, value) in &enriched_headers {
        header_lines.push_str(name);
        header_lines.push_str(": ");
        header_lines.push_str(value);
        header_lines.push_str("\r\n");
    }
    let request = format!(
        "GET {path} HTTP/1.1\r\nHost: {addr}\r\nConnection: Upgrade\r\nUpgrade: websocket\r\nSec-WebSocket-Key: test-kamn-key\r\nSec-WebSocket-Version: {websocket_version}\r\n{}Content-Length: 0\r\n\r\n",
        header_lines
    );
    stream
        .write_all(request.as_bytes())
        .expect("websocket upgrade request should write");
    let mut response = Vec::new();
    let mut peer_closed = false;
    let mut chunk = [0_u8; 1024];
    loop {
        match stream.read(&mut chunk) {
            Ok(0) => {
                peer_closed = true;
                break;
            }
            Ok(read_count) => response.extend_from_slice(&chunk[..read_count]),
            Err(error) if matches!(error.kind(), ErrorKind::WouldBlock | ErrorKind::TimedOut) => {
                break;
            }
            Err(error) => panic!("websocket response should be readable: {error}"),
        }
    }
    (response, peer_closed)
}

fn parse_websocket_response_frames(response: &[u8]) -> (String, Vec<String>) {
    let header_end = response
        .windows(4)
        .position(|window| window == b"\r\n\r\n")
        .map(|index| index + 4)
        .expect("websocket response should include header terminator");
    let header = std::str::from_utf8(&response[..header_end])
        .expect("websocket header should be utf-8")
        .to_owned();
    let mut frames = Vec::new();
    let frame_bytes = &response[header_end..];
    let mut frame_index = 0_usize;

    while frame_index + 2 <= frame_bytes.len() {
        let first = frame_bytes[frame_index];
        let second = frame_bytes[frame_index + 1];
        let opcode = first & 0x0f;
        assert_eq!(
            first & 0x80,
            0x80,
            "fragmented websocket frames are not supported by test parser"
        );
        assert_eq!(second & 0x80, 0, "server websocket frame must be unmasked");

        let mut payload_index = frame_index + 2;
        let payload_len = match second & 0x7f {
            value @ 0..=125 => value as usize,
            126 => {
                assert!(
                    frame_bytes.len() >= frame_index + 4,
                    "websocket frame extended payload length must be available"
                );
                payload_index = frame_index + 4;
                u16::from_be_bytes([frame_bytes[frame_index + 2], frame_bytes[frame_index + 3]])
                    as usize
            }
            127 => {
                assert!(
                    frame_bytes.len() >= frame_index + 10,
                    "websocket frame 64-bit payload length must be available"
                );
                payload_index = frame_index + 10;
                u64::from_be_bytes([
                    frame_bytes[frame_index + 2],
                    frame_bytes[frame_index + 3],
                    frame_bytes[frame_index + 4],
                    frame_bytes[frame_index + 5],
                    frame_bytes[frame_index + 6],
                    frame_bytes[frame_index + 7],
                    frame_bytes[frame_index + 8],
                    frame_bytes[frame_index + 9],
                ]) as usize
            }
            _ => unreachable!("websocket payload marker is constrained to 7 bits"),
        };

        assert!(
            frame_bytes.len() >= payload_index + payload_len,
            "websocket frame payload length must be available"
        );
        let payload_slice = &frame_bytes[payload_index..payload_index + payload_len];
        frame_index = payload_index + payload_len;

        if opcode == 0x8 {
            break;
        }
        if opcode == 0x1 {
            frames.push(
                std::str::from_utf8(payload_slice)
                    .expect("websocket payload should be utf-8")
                    .to_owned(),
            );
        }
    }

    (header, frames)
}

fn parse_websocket_response(response: &[u8]) -> (String, String) {
    let (header, frames) = parse_websocket_response_frames(response);
    let payload = frames
        .into_iter()
        .next()
        .expect("websocket response should include at least one text frame");
    (header, payload)
}

#[test]
fn integration_service_api_endpoint_websocket_upgrade_streams_state_transition_event() {
    let _env = acquire_service_api_test_env();
    let parsed = parse_args(vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
        "--runtime-mode".to_owned(),
        "api".to_owned(),
        "--api-bind".to_owned(),
        "127.0.0.1:34055".to_owned(),
    ])
    .expect("api args should parse");
    let report = execute(parsed).expect("api execution should succeed");
    let snapshot = build_service_api_snapshot(&report);

    let bind_addr = reserve_loopback_addr();
    let endpoint_config = ServiceApiEndpointConfig {
        bind_addr: bind_addr.clone(),
        max_requests: 1,
        idle_timeout_ms: 2_000,
        body_limit_bytes: DEFAULT_SERVICE_API_BODY_LIMIT_BYTES,
        concurrency_limit: DEFAULT_SERVICE_API_CONCURRENCY_LIMIT,
        rate_limit_per_second: DEFAULT_SERVICE_API_RATE_LIMIT_PER_SECOND,
    };
    let server_snapshot = snapshot.clone();
    let server =
        thread::spawn(move || serve_service_api_endpoint(&endpoint_config, &server_snapshot));
    wait_for_endpoint_ready(bind_addr.as_str());

    let sender_did = "kamn:did:agent:ws-client-1";
    let nonce = 19_u64;
    let state_hash = format!(
        "service-api:{}:{}",
        snapshot.chain_id.as_str(),
        snapshot.chain_version.as_str()
    );
    let signature =
        service_api_request_signature_for_fields(sender_did, nonce, state_hash.as_str(), "");
    let response = send_websocket_upgrade_request(
        bind_addr.as_str(),
        "/v1/events/ws",
        &[
            ("X-KAMN-Sender-DID", sender_did),
            ("X-KAMN-Request-Nonce", "19"),
            ("X-KAMN-Request-Signature", signature.as_str()),
        ],
    );
    let (header, payload) = parse_websocket_response(response.as_slice());
    assert!(header.contains("HTTP/1.1 101 Switching Protocols"));
    let normalized_header = header.to_ascii_lowercase();
    assert!(normalized_header.contains("upgrade: websocket"));
    assert!(normalized_header.contains("x-kamn-websocket-contract: v1"));
    assert!(payload.contains("\"event\":\"state-transition\""));
    assert!(payload.contains("\"runtime_mode\":\"api\""));
    assert!(payload.contains("\"role\":\"processor\""));

    let server_result = server.join().expect("endpoint thread should complete");
    assert!(
        server_result.is_ok(),
        "service api endpoint should stop cleanly after websocket request budget"
    );
}
#[test]
fn integration_service_api_endpoint_websocket_upgrade_keeps_connection_open_after_initial_event() {
    let _env = acquire_service_api_test_env();
    let parsed = parse_args(vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
        "--runtime-mode".to_owned(),
        "api".to_owned(),
        "--api-bind".to_owned(),
        "127.0.0.1:34065".to_owned(),
    ])
    .expect("api args should parse");
    let report = execute(parsed).expect("api execution should succeed");
    let snapshot = build_service_api_snapshot(&report);

    let bind_addr = reserve_loopback_addr();
    let endpoint_config = ServiceApiEndpointConfig {
        bind_addr: bind_addr.clone(),
        max_requests: 2,
        idle_timeout_ms: 2_000,
        body_limit_bytes: DEFAULT_SERVICE_API_BODY_LIMIT_BYTES,
        concurrency_limit: DEFAULT_SERVICE_API_CONCURRENCY_LIMIT,
        rate_limit_per_second: DEFAULT_SERVICE_API_RATE_LIMIT_PER_SECOND,
    };
    let server_snapshot = snapshot.clone();
    let server =
        thread::spawn(move || serve_service_api_endpoint(&endpoint_config, &server_snapshot));
    wait_for_endpoint_ready(bind_addr.as_str());

    let sender_did = "kamn:did:agent:ws-client-multi";
    let nonce = 57_u64;
    let state_hash = format!(
        "service-api:{}:{}",
        snapshot.chain_id.as_str(),
        snapshot.chain_version.as_str()
    );
    let signature =
        service_api_request_signature_for_fields(sender_did, nonce, state_hash.as_str(), "");
    let read_start = Instant::now();
    let (response, peer_closed) = send_websocket_upgrade_request_with_version_close_observation(
        bind_addr.as_str(),
        "/v1/events/ws",
        "13",
        &[
            ("X-KAMN-Sender-DID", sender_did),
            ("X-KAMN-Request-Nonce", "57"),
            ("X-KAMN-Request-Signature", signature.as_str()),
        ],
    );
    let (_header, frames) = parse_websocket_response_frames(response.as_slice());
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
    let read_elapsed = read_start.elapsed();
    let remained_open_or_timed_out = !peer_closed || read_elapsed >= Duration::from_millis(1_500);
    assert!(
        remained_open_or_timed_out,
        "websocket stream should not close immediately after initial frame; peer_closed={peer_closed} elapsed={read_elapsed:?}"
    );
    let server_result = server.join().expect("endpoint thread should complete");
    let ended_cleanly_or_timeout = match &server_result {
        Ok(()) => true,
        Err(error) => error.contains("service api timed out after"),
    };
    assert!(
        ended_cleanly_or_timeout,
        "websocket keep-open test should end via request budget completion or idle-timeout fail-close: {server_result:?}"
    );
}

#[test]
fn regression_service_api_endpoint_websocket_stream_delivers_live_message_event_after_upgrade() {
    // Regression: #5905
    let _env = acquire_service_api_test_env();
    let parsed = parse_args(vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
        "--runtime-mode".to_owned(),
        "api".to_owned(),
        "--api-bind".to_owned(),
        "127.0.0.1:34071".to_owned(),
    ])
    .expect("api args should parse");
    let report = execute(parsed).expect("api execution should succeed");
    let snapshot = build_service_api_snapshot(&report);

    let bind_addr = reserve_loopback_addr();
    let endpoint_config = ServiceApiEndpointConfig {
        bind_addr: bind_addr.clone(),
        max_requests: 6,
        idle_timeout_ms: 2_000,
        body_limit_bytes: DEFAULT_SERVICE_API_BODY_LIMIT_BYTES,
        concurrency_limit: DEFAULT_SERVICE_API_CONCURRENCY_LIMIT,
        rate_limit_per_second: DEFAULT_SERVICE_API_RATE_LIMIT_PER_SECOND,
    };
    let server_snapshot = snapshot.clone();
    let server =
        thread::spawn(move || serve_service_api_endpoint(&endpoint_config, &server_snapshot));
    wait_for_endpoint_ready(bind_addr.as_str());

    let state_hash = format!(
        "service-api:{}:{}",
        snapshot.chain_id.as_str(),
        snapshot.chain_version.as_str()
    );
    let websocket_sender_did = "kamn:did:agent:ws-live-stream-client";
    let websocket_signature = service_api_request_signature_for_fields(
        websocket_sender_did,
        601,
        state_hash.as_str(),
        "",
    );

    let post_bind_addr = bind_addr.clone();
    let post_state_hash = state_hash.clone();
    let post_thread = thread::spawn(move || {
        thread::sleep(Duration::from_millis(75));
        let sender_did = "kamn:did:agent:ws-live-stream-publisher";
        let first_body = "{\"message\":\"websocket-live-event-1\"}";
        let first_signature =
            service_api_request_signature_for_fields(sender_did, 602, &post_state_hash, first_body);
        let first_response = send_http_request_with_headers(
            post_bind_addr.as_str(),
            "POST",
            "/v1/messages/send",
            first_body,
            &[
                ("X-KAMN-Sender-DID", sender_did),
                ("X-KAMN-Request-Nonce", "602"),
                ("X-KAMN-Request-Signature", first_signature.as_str()),
                ("X-KAMN-Authz-Scope", "messages:write"),
            ],
        );
        thread::sleep(Duration::from_millis(25));
        let second_body = "{\"message\":\"websocket-live-event-2\"}";
        let second_signature = service_api_request_signature_for_fields(
            sender_did,
            603,
            &post_state_hash,
            second_body,
        );
        let second_response = send_http_request_with_headers(
            post_bind_addr.as_str(),
            "POST",
            "/v1/messages/send",
            second_body,
            &[
                ("X-KAMN-Sender-DID", sender_did),
                ("X-KAMN-Request-Nonce", "603"),
                ("X-KAMN-Request-Signature", second_signature.as_str()),
                ("X-KAMN-Authz-Scope", "messages:write"),
            ],
        );
        (first_response, second_response)
    });

    let websocket_response = send_websocket_upgrade_request(
        bind_addr.as_str(),
        "/v1/events/ws",
        &[
            ("X-KAMN-Sender-DID", websocket_sender_did),
            ("X-KAMN-Request-Nonce", "601"),
            ("X-KAMN-Request-Signature", websocket_signature.as_str()),
        ],
    );
    let (first_post_response, second_post_response) = post_thread
        .join()
        .expect("post request thread should complete");
    assert!(
        first_post_response.contains("HTTP/1.1 202 Accepted"),
        "first publisher request should be accepted: {first_post_response}"
    );
    assert!(
        second_post_response.contains("HTTP/1.1 202 Accepted"),
        "second publisher request should be accepted: {second_post_response}"
    );

    let (_header, frames) = parse_websocket_response_frames(websocket_response.as_slice());
    let created_sequences = frames
        .iter()
        .filter_map(|frame| {
            let payload: Value = serde_json::from_str(frame).ok()?;
            if payload.get("event").and_then(Value::as_str) != Some("service-api.message.created") {
                return None;
            }
            payload.get("sequence").and_then(Value::as_u64)
        })
        .collect::<Vec<u64>>();
    assert!(
        created_sequences.len() >= 2,
        "websocket stream should include multiple live message-created event frames after upgrade: {frames:?}"
    );
    let mut unique_sequences = created_sequences;
    unique_sequences.sort_unstable();
    unique_sequences.dedup();
    assert!(
        unique_sequences.len() >= 2
            && unique_sequences[1] > unique_sequences[0],
        "message-created websocket event sequence should advance across events: {unique_sequences:?}"
    );

    let server_result = server.join().expect("endpoint thread should complete");
    let ended_cleanly_or_timeout = match &server_result {
        Ok(()) => true,
        Err(error) => error.contains("service api timed out after"),
    };
    assert!(
        ended_cleanly_or_timeout,
        "service api endpoint should end via request budget completion or idle-timeout fail-close after websocket live stream regression test: {server_result:?}"
    );
}

#[test]
fn integration_service_api_endpoint_websocket_presence_mode_streams_bridge_projection_event() {
    let _env = acquire_service_api_test_env();
    let parsed = parse_args(vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
        "--runtime-mode".to_owned(),
        "api".to_owned(),
        "--api-bind".to_owned(),
        "127.0.0.1:34058".to_owned(),
    ])
    .expect("api args should parse");
    let report = execute(parsed).expect("api execution should succeed");
    let snapshot = build_service_api_snapshot(&report);

    let bind_addr = reserve_loopback_addr();
    let endpoint_config = ServiceApiEndpointConfig {
        bind_addr: bind_addr.clone(),
        max_requests: 1,
        idle_timeout_ms: 2_000,
        body_limit_bytes: DEFAULT_SERVICE_API_BODY_LIMIT_BYTES,
        concurrency_limit: DEFAULT_SERVICE_API_CONCURRENCY_LIMIT,
        rate_limit_per_second: DEFAULT_SERVICE_API_RATE_LIMIT_PER_SECOND,
    };
    let server_snapshot = snapshot.clone();
    let server =
        thread::spawn(move || serve_service_api_endpoint(&endpoint_config, &server_snapshot));
    wait_for_endpoint_ready(bind_addr.as_str());

    let sender_did = "kamn:did:agent:ws-presence-client-1";
    let nonce = 31_u64;
    let state_hash = format!(
        "service-api:{}:{}",
        snapshot.chain_id.as_str(),
        snapshot.chain_version.as_str()
    );
    let signature =
        service_api_request_signature_for_fields(sender_did, nonce, state_hash.as_str(), "");
    let response = send_websocket_upgrade_request(
        bind_addr.as_str(),
        "/v1/events/ws",
        &[
            ("X-KAMN-Sender-DID", sender_did),
            ("X-KAMN-Request-Nonce", "31"),
            ("X-KAMN-Request-Signature", signature.as_str()),
            ("X-KAMN-Events-Mode", "presence"),
            ("X-KAMN-Presence-Owner-DID", "kamn:did:owner:alpha"),
            ("X-KAMN-Presence-Target-Owner-DID", "kamn:did:owner:alpha"),
            ("X-KAMN-Presence-Target-Agent-DID", sender_did),
            ("X-KAMN-Presence-Gateway-Node", "gateway-alpha"),
            ("X-KAMN-Presence-Connected-Since", "1709000000"),
            ("X-KAMN-Presence-Last-Heartbeat", "1709000005"),
            ("X-KAMN-Presence-Capabilities", "ws,notify"),
        ],
    );
    let (header, payload) = parse_websocket_response(response.as_slice());
    assert!(header.contains("HTTP/1.1 101 Switching Protocols"));
    let payload_json: Value =
        serde_json::from_str(payload.as_str()).expect("presence websocket payload should be json");
    assert_eq!(
        payload_json.get("event").and_then(Value::as_str),
        Some("m9.presence.snapshot")
    );
    assert_eq!(
        payload_json
            .get("transport_profile")
            .and_then(Value::as_str),
        Some("websocket")
    );
    assert_eq!(
        payload_json
            .get("requester_owner_did")
            .and_then(Value::as_str),
        Some("kamn:did:owner:alpha")
    );
    assert_eq!(
        payload_json
            .get("requester_agent_did")
            .and_then(Value::as_str),
        Some(sender_did)
    );
    assert_eq!(
        payload_json.get("target_owner_did").and_then(Value::as_str),
        Some("kamn:did:owner:alpha")
    );
    assert_eq!(
        payload_json.get("target_agent_did").and_then(Value::as_str),
        Some(sender_did)
    );
    assert_eq!(
        payload_json.get("visible").and_then(Value::as_bool),
        Some(true)
    );
    assert_eq!(
        payload_json
            .get("target_gateway_node")
            .and_then(Value::as_str),
        Some("gateway-alpha")
    );
    assert_eq!(
        payload_json
            .get("target_last_heartbeat_epoch_seconds")
            .and_then(Value::as_u64),
        Some(1_709_000_005)
    );
    assert_eq!(
        payload_json.get("reason_code").and_then(Value::as_str),
        Some("m9_gateway_presence_visible")
    );

    let server_result = server.join().expect("endpoint thread should complete");
    assert!(
        server_result.is_ok(),
        "service api endpoint should stop cleanly after websocket presence request budget"
    );
}

#[test]
fn regression_service_api_endpoint_websocket_presence_mode_rejects_unsupported_mode() {
    let _env = acquire_service_api_test_env();
    let parsed = parse_args(vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
        "--runtime-mode".to_owned(),
        "api".to_owned(),
        "--api-bind".to_owned(),
        "127.0.0.1:34059".to_owned(),
    ])
    .expect("api args should parse");
    let report = execute(parsed).expect("api execution should succeed");
    let snapshot = build_service_api_snapshot(&report);

    let bind_addr = reserve_loopback_addr();
    let endpoint_config = ServiceApiEndpointConfig {
        bind_addr: bind_addr.clone(),
        max_requests: 1,
        idle_timeout_ms: 2_000,
        body_limit_bytes: DEFAULT_SERVICE_API_BODY_LIMIT_BYTES,
        concurrency_limit: DEFAULT_SERVICE_API_CONCURRENCY_LIMIT,
        rate_limit_per_second: DEFAULT_SERVICE_API_RATE_LIMIT_PER_SECOND,
    };
    let server_snapshot = snapshot.clone();
    let server =
        thread::spawn(move || serve_service_api_endpoint(&endpoint_config, &server_snapshot));
    wait_for_endpoint_ready(bind_addr.as_str());

    let sender_did = "kamn:did:agent:ws-presence-client-unsupported";
    let nonce = 37_u64;
    let state_hash = format!(
        "service-api:{}:{}",
        snapshot.chain_id.as_str(),
        snapshot.chain_version.as_str()
    );
    let signature =
        service_api_request_signature_for_fields(sender_did, nonce, state_hash.as_str(), "");
    let response = send_websocket_upgrade_request(
        bind_addr.as_str(),
        "/v1/events/ws",
        &[
            ("X-KAMN-Sender-DID", sender_did),
            ("X-KAMN-Request-Nonce", "37"),
            ("X-KAMN-Request-Signature", signature.as_str()),
            ("X-KAMN-Events-Mode", "presence-v2"),
        ],
    );
    let response_text =
        String::from_utf8(response).expect("unsupported mode response should be utf-8");
    assert!(response_text.contains("HTTP/1.1 400 Bad Request"));
    let payload = parse_error_envelope_from_http_response(response_text.as_str());
    assert_eq!(payload.error, "bad-request");
    assert_eq!(payload.reason_code, "service_api_ws_events_mode_invalid");

    let server_result = server.join().expect("endpoint thread should complete");
    assert!(
        server_result.is_ok(),
        "service api endpoint should stop cleanly after unsupported websocket mode rejection"
    );
}

#[test]
fn regression_service_api_endpoint_websocket_presence_mode_rejects_missing_owner_header() {
    let _env = acquire_service_api_test_env();
    let parsed = parse_args(vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
        "--runtime-mode".to_owned(),
        "api".to_owned(),
        "--api-bind".to_owned(),
        "127.0.0.1:34060".to_owned(),
    ])
    .expect("api args should parse");
    let report = execute(parsed).expect("api execution should succeed");
    let snapshot = build_service_api_snapshot(&report);

    let bind_addr = reserve_loopback_addr();
    let endpoint_config = ServiceApiEndpointConfig {
        bind_addr: bind_addr.clone(),
        max_requests: 1,
        idle_timeout_ms: 2_000,
        body_limit_bytes: DEFAULT_SERVICE_API_BODY_LIMIT_BYTES,
        concurrency_limit: DEFAULT_SERVICE_API_CONCURRENCY_LIMIT,
        rate_limit_per_second: DEFAULT_SERVICE_API_RATE_LIMIT_PER_SECOND,
    };
    let server_snapshot = snapshot.clone();
    let server =
        thread::spawn(move || serve_service_api_endpoint(&endpoint_config, &server_snapshot));
    wait_for_endpoint_ready(bind_addr.as_str());

    let sender_did = "kamn:did:agent:ws-presence-client-missing-owner";
    let nonce = 43_u64;
    let state_hash = format!(
        "service-api:{}:{}",
        snapshot.chain_id.as_str(),
        snapshot.chain_version.as_str()
    );
    let signature =
        service_api_request_signature_for_fields(sender_did, nonce, state_hash.as_str(), "");
    let response = send_websocket_upgrade_request(
        bind_addr.as_str(),
        "/v1/events/ws",
        &[
            ("X-KAMN-Sender-DID", sender_did),
            ("X-KAMN-Request-Nonce", "43"),
            ("X-KAMN-Request-Signature", signature.as_str()),
            ("X-KAMN-Events-Mode", "presence"),
            ("X-KAMN-Presence-Target-Agent-DID", sender_did),
        ],
    );
    let response_text =
        String::from_utf8(response).expect("missing-owner response should be utf-8");
    assert!(response_text.contains("HTTP/1.1 400 Bad Request"));
    let payload = parse_error_envelope_from_http_response(response_text.as_str());
    assert_eq!(payload.error, "bad-request");
    assert_eq!(
        payload.reason_code,
        "service_api_ws_presence_owner_did_header_missing"
    );

    let server_result = server.join().expect("endpoint thread should complete");
    assert!(
        server_result.is_ok(),
        "service api endpoint should stop cleanly after missing-owner websocket rejection"
    );
}

#[test]
fn regression_service_api_endpoint_websocket_presence_mode_rejects_cross_owner_scope() {
    let _env = acquire_service_api_test_env();
    let parsed = parse_args(vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
        "--runtime-mode".to_owned(),
        "api".to_owned(),
        "--api-bind".to_owned(),
        "127.0.0.1:34061".to_owned(),
    ])
    .expect("api args should parse");
    let report = execute(parsed).expect("api execution should succeed");
    let snapshot = build_service_api_snapshot(&report);

    let bind_addr = reserve_loopback_addr();
    let endpoint_config = ServiceApiEndpointConfig {
        bind_addr: bind_addr.clone(),
        max_requests: 1,
        idle_timeout_ms: 2_000,
        body_limit_bytes: DEFAULT_SERVICE_API_BODY_LIMIT_BYTES,
        concurrency_limit: DEFAULT_SERVICE_API_CONCURRENCY_LIMIT,
        rate_limit_per_second: DEFAULT_SERVICE_API_RATE_LIMIT_PER_SECOND,
    };
    let server_snapshot = snapshot.clone();
    let server =
        thread::spawn(move || serve_service_api_endpoint(&endpoint_config, &server_snapshot));
    wait_for_endpoint_ready(bind_addr.as_str());

    let sender_did = "kamn:did:agent:ws-presence-client-scope";
    let nonce = 41_u64;
    let state_hash = format!(
        "service-api:{}:{}",
        snapshot.chain_id.as_str(),
        snapshot.chain_version.as_str()
    );
    let signature =
        service_api_request_signature_for_fields(sender_did, nonce, state_hash.as_str(), "");
    let response = send_websocket_upgrade_request(
        bind_addr.as_str(),
        "/v1/events/ws",
        &[
            ("X-KAMN-Sender-DID", sender_did),
            ("X-KAMN-Request-Nonce", "41"),
            ("X-KAMN-Request-Signature", signature.as_str()),
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
    );
    let response_text =
        String::from_utf8(response).expect("cross-owner scope denial response should be utf-8");
    assert!(response_text.contains("HTTP/1.1 403 Forbidden"));
    let payload = parse_error_envelope_from_http_response(response_text.as_str());
    assert_eq!(payload.error, "forbidden");
    assert_eq!(payload.reason_code, "m9_realtime_owner_scope_denied");

    let server_result = server.join().expect("endpoint thread should complete");
    assert!(
        server_result.is_ok(),
        "service api endpoint should stop cleanly after cross-owner websocket rejection"
    );
}

#[test]
fn regression_service_api_endpoint_websocket_route_rejects_missing_upgrade_headers() {
    let _env = acquire_service_api_test_env();
    let parsed = parse_args(vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
        "--runtime-mode".to_owned(),
        "api".to_owned(),
        "--api-bind".to_owned(),
        "127.0.0.1:34056".to_owned(),
    ])
    .expect("api args should parse");
    let report = execute(parsed).expect("api execution should succeed");
    let snapshot = build_service_api_snapshot(&report);

    let bind_addr = reserve_loopback_addr();
    let endpoint_config = ServiceApiEndpointConfig {
        bind_addr: bind_addr.clone(),
        max_requests: 1,
        idle_timeout_ms: 2_000,
        body_limit_bytes: DEFAULT_SERVICE_API_BODY_LIMIT_BYTES,
        concurrency_limit: DEFAULT_SERVICE_API_CONCURRENCY_LIMIT,
        rate_limit_per_second: DEFAULT_SERVICE_API_RATE_LIMIT_PER_SECOND,
    };
    let server_snapshot = snapshot.clone();
    let server =
        thread::spawn(move || serve_service_api_endpoint(&endpoint_config, &server_snapshot));
    wait_for_endpoint_ready(bind_addr.as_str());

    let sender_did = "kamn:did:agent:ws-client-2";
    let nonce = 23_u64;
    let state_hash = format!(
        "service-api:{}:{}",
        snapshot.chain_id.as_str(),
        snapshot.chain_version.as_str()
    );
    let signature =
        service_api_request_signature_for_fields(sender_did, nonce, state_hash.as_str(), "");
    let response = send_http_request_with_headers(
        bind_addr.as_str(),
        "GET",
        "/v1/events/ws",
        "",
        &[
            ("X-KAMN-Sender-DID", sender_did),
            ("X-KAMN-Request-Nonce", "23"),
            ("X-KAMN-Request-Signature", signature.as_str()),
        ],
    );
    assert!(response.contains("HTTP/1.1 400 Bad Request"));
    let payload = parse_error_envelope_from_http_response(response.as_str());
    assert_eq!(payload.error, "bad-request");
    assert_eq!(payload.reason_code, "service_api_ws_upgrade_header_missing");
    assert!(payload
        .message
        .contains("missing required websocket upgrade header"));

    let server_result = server.join().expect("endpoint thread should complete");
    assert!(
        server_result.is_ok(),
        "service api endpoint should stop cleanly after websocket rejection budget"
    );
}

#[test]
fn regression_service_api_endpoint_websocket_rejects_invalid_version_header() {
    let _env = acquire_service_api_test_env();
    let parsed = parse_args(vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
        "--runtime-mode".to_owned(),
        "api".to_owned(),
        "--api-bind".to_owned(),
        "127.0.0.1:34057".to_owned(),
    ])
    .expect("api args should parse");
    let report = execute(parsed).expect("api execution should succeed");
    let snapshot = build_service_api_snapshot(&report);

    let bind_addr = reserve_loopback_addr();
    let endpoint_config = ServiceApiEndpointConfig {
        bind_addr: bind_addr.clone(),
        max_requests: 1,
        idle_timeout_ms: 2_000,
        body_limit_bytes: DEFAULT_SERVICE_API_BODY_LIMIT_BYTES,
        concurrency_limit: DEFAULT_SERVICE_API_CONCURRENCY_LIMIT,
        rate_limit_per_second: DEFAULT_SERVICE_API_RATE_LIMIT_PER_SECOND,
    };
    let server_snapshot = snapshot.clone();
    let server =
        thread::spawn(move || serve_service_api_endpoint(&endpoint_config, &server_snapshot));
    wait_for_endpoint_ready(bind_addr.as_str());

    let sender_did = "kamn:did:agent:ws-client-3";
    let nonce = 29_u64;
    let state_hash = format!(
        "service-api:{}:{}",
        snapshot.chain_id.as_str(),
        snapshot.chain_version.as_str()
    );
    let signature =
        service_api_request_signature_for_fields(sender_did, nonce, state_hash.as_str(), "");
    let response = send_websocket_upgrade_request_with_version(
        bind_addr.as_str(),
        "/v1/events/ws",
        "12",
        &[
            ("X-KAMN-Sender-DID", sender_did),
            ("X-KAMN-Request-Nonce", "29"),
            ("X-KAMN-Request-Signature", signature.as_str()),
        ],
    );
    let response_text =
        String::from_utf8(response).expect("invalid websocket version response should be utf-8");
    assert!(response_text.contains("HTTP/1.1 400 Bad Request"));
    let payload = parse_error_envelope_from_http_response(response_text.as_str());
    assert_eq!(payload.error, "bad-request");
    assert_eq!(payload.reason_code, "service_api_ws_version_header_invalid");
    assert!(payload.message.contains("invalid websocket version header"));

    let server_result = server.join().expect("endpoint thread should complete");
    assert!(
        server_result.is_ok(),
        "service api endpoint should stop cleanly after websocket version rejection"
    );
}
