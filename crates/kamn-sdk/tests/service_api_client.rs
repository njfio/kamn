use kamn_sdk::{
    service_signature_for_fields, AgentDid, SdkError, ServiceApiClient, ServiceRequestAuth,
};
use std::collections::{BTreeMap, BTreeSet};
use std::io::{ErrorKind, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::thread;
use std::time::{Duration, Instant};

const CHAIN_ID: &str = "kolme-localnet";
const CHAIN_VERSION: &str = "v0";

fn reserve_loopback_addr() -> String {
    let listener = TcpListener::bind("127.0.0.1:0").expect("listener should bind");
    let addr = listener.local_addr().expect("local addr should resolve");
    drop(listener);
    addr.to_string()
}

fn parse_http_request(
    stream: &mut TcpStream,
) -> Result<(String, String, String, BTreeMap<String, String>), String> {
    let mut request = Vec::new();
    let mut chunk = [0_u8; 1024];
    stream
        .set_read_timeout(Some(Duration::from_secs(2)))
        .map_err(|error| format!("request read-timeout failed: {error}"))?;

    let mut expected_total_bytes: Option<usize> = None;
    let mut header_end: Option<usize> = None;
    loop {
        match stream.read(&mut chunk) {
            Ok(0) => break,
            Ok(read_count) => {
                request.extend_from_slice(&chunk[..read_count]);
                if header_end.is_none() {
                    header_end = request
                        .windows(4)
                        .position(|window| window == b"\r\n\r\n")
                        .map(|index| index + 4);
                    if let Some(header_end_index) = header_end {
                        let header = String::from_utf8(request[..header_end_index].to_vec())
                            .map_err(|_| "request header was not valid utf-8".to_owned())?;
                        let content_length = parse_content_length(header.as_str())?;
                        expected_total_bytes = Some(header_end_index + content_length);
                    }
                }
                if let Some(total) = expected_total_bytes {
                    if request.len() >= total {
                        break;
                    }
                }
            }
            Err(error) if matches!(error.kind(), ErrorKind::WouldBlock | ErrorKind::TimedOut) => {
                break;
            }
            Err(error) => return Err(format!("request read failed: {error}")),
        }
    }

    let request_text =
        String::from_utf8(request).map_err(|_| "request was not valid utf-8".to_owned())?;
    let Some((request_head, request_body)) = request_text.split_once("\r\n\r\n") else {
        return Err("request header terminator missing".to_owned());
    };
    let request_line = request_head
        .lines()
        .next()
        .ok_or_else(|| "request line missing".to_owned())?;
    let mut headers = BTreeMap::new();
    for line in request_head.lines().skip(1) {
        if line.trim().is_empty() {
            continue;
        }
        let (name, value) = line
            .split_once(':')
            .ok_or_else(|| "request header line missing ':' separator".to_owned())?;
        headers.insert(name.trim().to_ascii_lowercase(), value.trim().to_owned());
    }
    let mut parts = request_line.split_whitespace();
    let method = parts
        .next()
        .ok_or_else(|| "request method missing".to_owned())?
        .to_owned();
    let path = parts
        .next()
        .ok_or_else(|| "request path missing".to_owned())?
        .to_owned();
    Ok((method, path, request_body.to_owned(), headers))
}

fn parse_content_length(header: &str) -> Result<usize, String> {
    let value = header
        .lines()
        .find_map(|line| {
            let (name, raw_value) = line.split_once(':')?;
            if name.eq_ignore_ascii_case("Content-Length") {
                return Some(raw_value.trim());
            }
            None
        })
        .unwrap_or("0");
    value
        .parse::<usize>()
        .map_err(|_| "invalid content-length header".to_owned())
}

fn write_http_response(stream: &mut TcpStream, status: u16, body: &str) -> Result<(), String> {
    let status_text = match status {
        200 => "200 OK",
        201 => "201 Created",
        202 => "202 Accepted",
        400 => "400 Bad Request",
        401 => "401 Unauthorized",
        404 => "404 Not Found",
        409 => "409 Conflict",
        _ => "500 Internal Server Error",
    };
    let payload = format!(
        "HTTP/1.1 {status_text}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        body.len(),
        body
    );
    stream
        .write_all(payload.as_bytes())
        .map_err(|error| format!("service api write failed: {error}"))
}

fn deterministic_tag(payload: &[u8]) -> u64 {
    let mut acc: u64 = 0xcbf29ce484222325;
    for byte in payload {
        acc = acc.wrapping_mul(0x00000100000001B3);
        acc ^= u64::from(*byte);
    }
    acc
}

fn write_websocket_upgrade_response(stream: &mut TcpStream) -> Result<(), String> {
    let handshake = "HTTP/1.1 101 Switching Protocols\r\nUpgrade: websocket\r\nConnection: Upgrade\r\nSec-WebSocket-Accept: kamn-test-accept\r\nX-KAMN-WebSocket-Contract: v1\r\n\r\n";
    stream
        .write_all(handshake.as_bytes())
        .map_err(|error| format!("websocket handshake write failed: {error}"))?;
    let payload =
        r#"{"event":"state-transition","runtime_mode":"api","role":"processor","sequence":1}"#;
    let mut frame = Vec::with_capacity(2 + payload.len());
    frame.push(0x81);
    frame.push(payload.len() as u8);
    frame.extend_from_slice(payload.as_bytes());
    stream
        .write_all(frame.as_slice())
        .map_err(|error| format!("websocket frame write failed: {error}"))
}

fn validate_auth(
    body: &str,
    headers: &BTreeMap<String, String>,
    replay_guard: &mut BTreeSet<(String, u64)>,
) -> Result<(), (u16, &'static str)> {
    let did = headers
        .get("x-kamn-sender-did")
        .ok_or((401, "missing required header: x-kamn-sender-did"))?
        .to_owned();
    AgentDid::parse(did.clone()).map_err(|_| (401, "invalid sender did"))?;
    let nonce = headers
        .get("x-kamn-request-nonce")
        .ok_or((401, "missing required header: x-kamn-request-nonce"))?
        .parse::<u64>()
        .map_err(|_| (401, "invalid request nonce header: x-kamn-request-nonce"))?;
    if nonce == 0 {
        return Err((401, "request nonce must be positive: x-kamn-request-nonce"));
    }
    let signature = headers
        .get("x-kamn-request-signature")
        .ok_or((401, "missing required header: x-kamn-request-signature"))?;
    let expected = service_signature_for_fields(
        &AgentDid::parse(did.clone()).map_err(|_| (401, "invalid sender did"))?,
        nonce,
        CHAIN_ID,
        CHAIN_VERSION,
        body,
    );
    if &expected != signature {
        return Err((401, "signature verification failed for request envelope"));
    }
    if !replay_guard.insert((did, nonce)) {
        return Err((409, "request nonce replay detected for sender"));
    }
    Ok(())
}

fn run_service_contract_server(bind_addr: String, max_requests: u64) -> Result<(), String> {
    let listener = TcpListener::bind(bind_addr.as_str())
        .map_err(|error| format!("server bind failed: {error}"))?;
    listener
        .set_nonblocking(true)
        .map_err(|error| format!("server nonblocking mode failed: {error}"))?;

    let deadline = Instant::now() + Duration::from_secs(2);
    let mut served = 0_u64;
    let mut replay_guard: BTreeSet<(String, u64)> = BTreeSet::new();
    while served < max_requests {
        if Instant::now() > deadline {
            return Err("server timed out before serving request budget".to_owned());
        }
        match listener.accept() {
            Ok((mut stream, _)) => {
                let (method, path, body, headers) = parse_http_request(&mut stream)?;
                if method == "GET" && path == "/healthz" {
                    write_http_response(
                        &mut stream,
                        200,
                        r#"{"status":"ok","runtime_mode":"api","role":"processor","observability_source":"unknown","observability_health":"unknown"}"#,
                    )?;
                    served = served.saturating_add(1);
                    continue;
                }
                if method == "GET" && path == "/metrics" {
                    let body = "kamn_service_api_health{runtime_mode=\"api\"} 1\n";
                    let response = format!(
                        "HTTP/1.1 200 OK\r\nContent-Type: text/plain; version=0.0.4\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                        body.len(),
                        body
                    );
                    stream
                        .write_all(response.as_bytes())
                        .map_err(|error| format!("metrics write failed: {error}"))?;
                    served = served.saturating_add(1);
                    continue;
                }

                if let Err((status, reason)) = validate_auth(&body, &headers, &mut replay_guard) {
                    let payload = format!(
                        "{{\"error\":\"{}\",\"reason\":\"{}\"}}",
                        if status == 409 {
                            "replay"
                        } else {
                            "unauthorized"
                        },
                        reason
                    );
                    write_http_response(&mut stream, status, payload.as_str())?;
                    served = served.saturating_add(1);
                    continue;
                }

                if method == "GET" && path == "/v1/events/ws" {
                    let upgrade = headers.get("upgrade").cloned().unwrap_or_default();
                    let connection = headers.get("connection").cloned().unwrap_or_default();
                    let websocket_key = headers
                        .get("sec-websocket-key")
                        .cloned()
                        .unwrap_or_default();
                    let version = headers
                        .get("sec-websocket-version")
                        .cloned()
                        .unwrap_or_default();
                    if !upgrade.eq_ignore_ascii_case("websocket")
                        || !connection.to_ascii_lowercase().contains("upgrade")
                        || websocket_key.trim().is_empty()
                        || version.trim() != "13"
                    {
                        write_http_response(
                            &mut stream,
                            400,
                            r#"{"error":"bad-request","reason":"websocket upgrade required"}"#,
                        )?;
                    } else {
                        write_websocket_upgrade_response(&mut stream)?;
                    }
                    served = served.saturating_add(1);
                    continue;
                }

                if method == "POST" && path == "/v1/messages/send" {
                    let message_id =
                        format!("msg-local-{:016x}", deterministic_tag(body.as_bytes()));
                    let payload = format!(
                        "{{\"message_id\":\"{}\",\"status\":\"created\",\"runtime_mode\":\"api\"}}",
                        message_id
                    );
                    write_http_response(&mut stream, 202, payload.as_str())?;
                } else if method == "GET" && path.starts_with("/v1/messages/") {
                    let message_id = path.trim_start_matches("/v1/messages/");
                    let payload = format!(
                        "{{\"message_id\":\"{}\",\"status\":\"created\"}}",
                        message_id
                    );
                    write_http_response(&mut stream, 200, payload.as_str())?;
                } else if method == "POST" && path == "/v1/channels/create" {
                    let channel_id =
                        format!("channel-local-{:016x}", deterministic_tag(body.as_bytes()));
                    let payload = format!(
                        "{{\"channel_id\":\"{}\",\"status\":\"created\"}}",
                        channel_id
                    );
                    write_http_response(&mut stream, 201, payload.as_str())?;
                } else if method == "POST" && path == "/v1/tasks/create" {
                    let task_id = format!("task-local-{:016x}", deterministic_tag(body.as_bytes()));
                    let payload =
                        format!("{{\"task_id\":\"{}\",\"state\":\"submitted\"}}", task_id);
                    write_http_response(&mut stream, 201, payload.as_str())?;
                } else if method == "GET" && path.starts_with("/v1/tasks/") {
                    let task_id = path.trim_start_matches("/v1/tasks/");
                    let payload =
                        format!("{{\"task_id\":\"{}\",\"state\":\"submitted\"}}", task_id);
                    write_http_response(&mut stream, 200, payload.as_str())?;
                } else if method == "GET" && path.starts_with("/v1/agents/") {
                    let did = path.trim_start_matches("/v1/agents/");
                    let payload = format!("{{\"did\":\"{}\",\"reputation_score\":500}}", did);
                    write_http_response(&mut stream, 200, payload.as_str())?;
                } else {
                    write_http_response(&mut stream, 404, "not found")?;
                }

                served = served.saturating_add(1);
            }
            Err(error) if error.kind() == ErrorKind::WouldBlock => {
                thread::sleep(Duration::from_millis(5));
            }
            Err(error) => return Err(format!("server accept failed: {error}")),
        }
    }
    Ok(())
}

fn wait_for_server_ready(addr: &str) {
    assert!(!addr.trim().is_empty(), "server address must not be empty");
    thread::sleep(Duration::from_millis(40));
}

fn auth(sender: &AgentDid, nonce: u64, body: &str) -> ServiceRequestAuth {
    ServiceRequestAuth::new(
        sender.clone(),
        nonce,
        service_signature_for_fields(sender, nonce, CHAIN_ID, CHAIN_VERSION, body),
    )
    .expect("request auth should build")
}

#[test]
fn unit_service_api_client_rejects_invalid_endpoint_scheme() {
    assert_eq!(
        ServiceApiClient::connect("tcp://127.0.0.1:35001"),
        Err(SdkError::InvalidInput {
            field: "service.endpoint",
            reason: "must start with http:// or https://",
        })
    );
}

#[test]
fn functional_service_api_client_executes_signed_http_route_contracts() {
    let bind_addr = reserve_loopback_addr();
    let server_addr = bind_addr.clone();
    let server = thread::spawn(move || run_service_contract_server(server_addr, 8));
    wait_for_server_ready(bind_addr.as_str());

    let client = ServiceApiClient::connect(format!("http://{bind_addr}").as_str())
        .expect("client should connect");
    let sender = AgentDid::parse("kamn:did:agent:sdk-client").expect("sender did should parse");

    let send_payload = r#"{"message":"hello from sdk"}"#;
    let send_response = client
        .send_message(send_payload, &auth(&sender, 1, send_payload))
        .expect("send message should succeed");
    assert!(send_response.message_id.starts_with("msg-local-"));
    assert_eq!(send_response.status, "created");

    let message_status = client
        .get_message(send_response.message_id.as_str(), &auth(&sender, 2, ""))
        .expect("get message should succeed");
    assert_eq!(message_status.message_id, send_response.message_id);
    assert_eq!(message_status.status, "created");

    let channel_payload = r#"{"name":"ops"}"#;
    let channel_response = client
        .create_channel(channel_payload, &auth(&sender, 3, channel_payload))
        .expect("create channel should succeed");
    assert!(channel_response.channel_id.starts_with("channel-local-"));

    let task_payload = r#"{"task":"triage"}"#;
    let task_response = client
        .create_task(task_payload, &auth(&sender, 4, task_payload))
        .expect("create task should succeed");
    assert!(task_response.task_id.starts_with("task-local-"));

    let task_status = client
        .get_task(task_response.task_id.as_str(), &auth(&sender, 5, ""))
        .expect("get task should succeed");
    assert_eq!(task_status.state, "submitted");

    let profile = client
        .get_agent_profile(sender.as_str(), &auth(&sender, 6, ""))
        .expect("agent profile should resolve");
    assert_eq!(profile.did, sender.as_str());
    assert_eq!(profile.reputation_score, 500);

    let health = client.health().expect("health route should succeed");
    assert_eq!(health.status, "ok");
    assert_eq!(health.runtime_mode, "api");

    let metrics = client.metrics().expect("metrics route should succeed");
    assert!(
        metrics.contains("kamn_service_api_health{runtime_mode=\"api\"} 1"),
        "metrics contract should include service health gauge"
    );

    let server_result = server.join().expect("server thread should join");
    assert!(
        server_result.is_ok(),
        "test service contract server should satisfy request budget"
    );
}

#[test]
fn integration_service_api_client_reads_websocket_event_frame() {
    let bind_addr = reserve_loopback_addr();
    let server_addr = bind_addr.clone();
    let server = thread::spawn(move || run_service_contract_server(server_addr, 1));
    wait_for_server_ready(bind_addr.as_str());

    let client = ServiceApiClient::connect(format!("http://{bind_addr}").as_str())
        .expect("client should connect");
    let sender = AgentDid::parse("kamn:did:agent:sdk-events").expect("sender did should parse");

    let event = client
        .read_event_once(&auth(&sender, 9, ""))
        .expect("event read should succeed");
    assert_eq!(event.event, "state-transition");
    assert_eq!(event.runtime_mode, "api");
    assert_eq!(event.role, "processor");
    assert_eq!(event.sequence, 1);

    let server_result = server.join().expect("server thread should join");
    assert!(
        server_result.is_ok(),
        "test service contract server should satisfy websocket request budget"
    );
}

#[test]
fn regression_service_api_client_rejects_replayed_nonce() {
    // Regression: #2946
    let bind_addr = reserve_loopback_addr();
    let server_addr = bind_addr.clone();
    let server = thread::spawn(move || run_service_contract_server(server_addr, 2));
    wait_for_server_ready(bind_addr.as_str());

    let client = ServiceApiClient::connect(format!("http://{bind_addr}").as_str())
        .expect("client should connect");
    let sender = AgentDid::parse("kamn:did:agent:sdk-replay").expect("sender did should parse");
    let payload = r#"{"message":"nonce replay"}"#;
    let replay_auth = auth(&sender, 11, payload);

    client
        .send_message(payload, &replay_auth)
        .expect("first send should pass");
    assert_eq!(
        client
            .send_message(payload, &replay_auth)
            .expect_err("replayed nonce should fail closed"),
        SdkError::Conflict("request rejected by service api")
    );

    let server_result = server.join().expect("server thread should join");
    assert!(
        server_result.is_ok(),
        "test service contract server should satisfy replay request budget"
    );
}
