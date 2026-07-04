use super::super::super::*;

pub(crate) const WEBSOCKET_EVENTS_PATH: &str = "/v1/events/ws";
pub(crate) const WS_PRESENCE_OWNER_DID_INVALID_REASON_CODE: &str =
    "service_api_ws_presence_owner_did_header_invalid";
pub(crate) const WS_PRESENCE_TARGET_OWNER_DID_INVALID_REASON_CODE: &str =
    "service_api_ws_presence_target_owner_did_header_invalid";
pub(crate) const WS_PRESENCE_TARGET_AGENT_DID_INVALID_REASON_CODE: &str =
    "service_api_ws_presence_target_agent_did_header_invalid";

pub(crate) fn send_websocket_upgrade_request(
    addr: &str,
    path: &str,
    headers: &[(&str, &str)],
) -> Vec<u8> {
    send_websocket_upgrade_request_with_timeout(addr, path, "13", Duration::from_secs(2), headers)
}

pub(crate) fn send_websocket_upgrade_request_with_version(
    addr: &str,
    path: &str,
    websocket_version: &str,
    headers: &[(&str, &str)],
) -> Vec<u8> {
    send_websocket_upgrade_request_with_timeout(
        addr,
        path,
        websocket_version,
        Duration::from_secs(2),
        headers,
    )
}

pub(crate) fn send_websocket_upgrade_request_with_timeout(
    addr: &str,
    path: &str,
    websocket_version: &str,
    read_timeout: Duration,
    headers: &[(&str, &str)],
) -> Vec<u8> {
    send_websocket_upgrade_request_with_version_close_observation_and_timeout(
        addr,
        path,
        websocket_version,
        read_timeout,
        headers,
    )
    .0
}

pub(crate) fn send_websocket_upgrade_request_with_version_close_observation(
    addr: &str,
    path: &str,
    websocket_version: &str,
    headers: &[(&str, &str)],
) -> (Vec<u8>, bool) {
    send_websocket_upgrade_request_with_version_close_observation_and_timeout(
        addr,
        path,
        websocket_version,
        Duration::from_secs(2),
        headers,
    )
}

pub(crate) fn send_websocket_upgrade_request_with_version_close_observation_and_timeout(
    addr: &str,
    path: &str,
    websocket_version: &str,
    read_timeout: Duration,
    headers: &[(&str, &str)],
) -> (Vec<u8>, bool) {
    let mut stream = websocket_stream(addr, read_timeout);
    let enriched_headers = enrich_signed_headers_with_scope("GET", path, headers);
    let header_lines = websocket_header_lines(&enriched_headers);
    let request = websocket_upgrade_request(addr, path, websocket_version, header_lines.as_str());
    stream
        .write_all(request.as_bytes())
        .expect("websocket upgrade request should write");
    read_websocket_response(&mut stream)
}

fn websocket_stream(addr: &str, read_timeout: Duration) -> TcpStream {
    let stream = TcpStream::connect(addr).expect("endpoint should accept websocket connection");
    stream
        .set_read_timeout(Some(read_timeout))
        .expect("websocket read timeout should be configurable");
    stream
}

fn websocket_header_lines(headers: &[(String, String)]) -> String {
    let mut header_lines = String::new();
    for (name, value) in headers {
        header_lines.push_str(name.as_str());
        header_lines.push_str(": ");
        header_lines.push_str(value.as_str());
        header_lines.push_str("\r\n");
    }
    header_lines
}

fn websocket_upgrade_request(
    addr: &str,
    path: &str,
    websocket_version: &str,
    header_lines: &str,
) -> String {
    format!(
        "GET {path} HTTP/1.1\r\nHost: {addr}\r\nConnection: Upgrade\r\nUpgrade: websocket\r\nSec-WebSocket-Key: test-kamn-key\r\nSec-WebSocket-Version: {websocket_version}\r\n{header_lines}Content-Length: 0\r\n\r\n",
    )
}

fn read_websocket_response(stream: &mut TcpStream) -> (Vec<u8>, bool) {
    let mut response = Vec::new();
    let mut chunk = [0_u8; 1024];
    loop {
        match stream.read(&mut chunk) {
            Ok(0) => return (response, true),
            Ok(read_count) => response.extend_from_slice(&chunk[..read_count]),
            Err(error) if matches!(error.kind(), ErrorKind::WouldBlock | ErrorKind::TimedOut) => {
                return (response, false);
            }
            Err(error) => panic!("websocket response should be readable: {error}"),
        }
    }
}

pub(crate) fn assert_websocket_bad_request(
    response: Vec<u8>,
    reason_code: &str,
    message_fragment: Option<&str>,
) {
    let response_text = String::from_utf8(response).expect("websocket rejection should be utf-8");
    assert!(response_text.contains("HTTP/1.1 400 Bad Request"));
    let payload = parse_error_envelope_from_http_response(response_text.as_str());
    assert_eq!(payload.error, "bad-request");
    assert_eq!(payload.reason_code, reason_code);
    if let Some(fragment) = message_fragment {
        assert!(payload.message.contains(fragment));
    }
}

pub(crate) fn assert_websocket_forbidden(response: Vec<u8>, reason_code: &str) {
    let response_text = String::from_utf8(response).expect("websocket rejection should be utf-8");
    assert!(response_text.contains("HTTP/1.1 403 Forbidden"));
    let payload = parse_error_envelope_from_http_response(response_text.as_str());
    assert_eq!(payload.error, "forbidden");
    assert_eq!(payload.reason_code, reason_code);
}
