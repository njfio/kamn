use kamn_sdk::{
    AgentDid, AgentMetadata, AgentQuery, KamnAgent, KamnTransport, LiveTransportConfig,
    LiveTransportKamnClient, Message, SdkError, TransportMode, service_signature_for_fields,
};
use std::collections::{BTreeMap, BTreeSet};
use std::io::{ErrorKind, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Mutex, OnceLock};
use std::thread;
use std::time::{Duration, Instant};

const CHAIN_ID: &str = "kamn-sdk-live";
const CHAIN_VERSION: &str = "1";
const LIVE_CHAIN_ID_ENV: &str = "KAMN_SDK_LIVE_CHAIN_ID";
const LIVE_CHAIN_VERSION_ENV: &str = "KAMN_SDK_LIVE_CHAIN_VERSION";
const LIVE_REQUESTER_DID_ENV: &str = "KAMN_SDK_LIVE_REQUESTER_DID";
const DEFAULT_LIVE_REQUESTER_DID: &str = "kamn:did:agent:live-sdk";
const SERVICE_AUTH_PRIVATE_KEY_ENV: &str = "KAMN_SERVICE_API_AUTH_PRIVATE_KEY_HEX";
const REQUEST_AUTH_SCOPE_HEADER: &str = "x-kamn-authz-scope";
const TEST_SERVICE_AUTH_PRIVATE_KEY_HEX: &str =
    "658c3528422eb527b4c108b8f6d1e5f629543c304ea49cf608c67794424291c4";

fn did(identifier: &str) -> AgentDid {
    AgentDid::parse(format!("kamn:did:agent:{identifier}")).expect("did should parse")
}

fn metadata(agent_type: &str, model: &str, capabilities: &[&str]) -> AgentMetadata {
    AgentMetadata {
        agent_type: agent_type.to_owned(),
        model_family: model.to_owned(),
        capabilities: capabilities
            .iter()
            .map(|value| (*value).to_owned())
            .collect(),
    }
}

fn ensure_live_test_env() {
    std::env::set_var(
        SERVICE_AUTH_PRIVATE_KEY_ENV,
        TEST_SERVICE_AUTH_PRIVATE_KEY_HEX,
    );
    std::env::set_var(LIVE_CHAIN_ID_ENV, CHAIN_ID);
    std::env::set_var(LIVE_CHAIN_VERSION_ENV, CHAIN_VERSION);
    std::env::set_var(LIVE_REQUESTER_DID_ENV, "kamn:did:agent:live-tester");
}

fn env_lock() -> &'static Mutex<()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
}

fn with_env_lock<T>(callback: impl FnOnce() -> T) -> T {
    let lock = env_lock();
    let guard = lock.lock().expect("env lock should not be poisoned");
    let output = callback();
    drop(guard);
    output
}

fn reserve_loopback_addr() -> String {
    let listener = TcpListener::bind("127.0.0.1:0").expect("listener should bind");
    let addr = listener.local_addr().expect("local addr should resolve");
    drop(listener);
    addr.to_string()
}

fn wait_for_server_ready(bind_addr: &str) {
    assert!(
        !bind_addr.trim().is_empty(),
        "server address must not be empty"
    );
    thread::sleep(Duration::from_millis(40));
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

fn write_http_response(stream: &mut TcpStream, status: u16, body: &str) -> Result<(), String> {
    let status_text = match status {
        200 => "200 OK",
        201 => "201 Created",
        202 => "202 Accepted",
        401 => "401 Unauthorized",
        _ => "500 Internal Server Error",
    };
    let payload = format!(
        "HTTP/1.1 {status_text}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
        body.len(),
        body
    );
    stream
        .write_all(payload.as_bytes())
        .map_err(|error| format!("response write failed: {error}"))
}

fn required_scope_for_route(method: &str, path: &str) -> Option<&'static str> {
    Some(match (method, path) {
        ("POST", "/v1/agents/register") => "agents:write",
        ("POST", "/v1/agents/search") => "agents:read",
        ("POST", "/v1/messages/send") => "messages:write",
        ("GET", _) if path.starts_with("/v1/agents/") => "agents:read",
        _ => return None,
    })
}

fn validate_auth(
    method: &str,
    path: &str,
    body: &str,
    headers: &BTreeMap<String, String>,
    replay_guard: &mut BTreeSet<(String, u64)>,
    expected_agent_sender_did: &str,
) -> Result<(), (u16, &'static str, &'static str, &'static str)> {
    ensure_live_test_env();
    let did_value = headers
        .get("x-kamn-sender-did")
        .ok_or((
            401,
            "unauthorized",
            "service_api_auth_sender_did_header_missing",
            "missing required header: x-kamn-sender-did",
        ))?
        .to_owned();
    let did = AgentDid::parse(did_value.clone()).map_err(|_| {
        (
            401,
            "unauthorized",
            "service_api_auth_sender_did_invalid",
            "invalid sender did",
        )
    })?;
    let nonce = headers
        .get("x-kamn-request-nonce")
        .ok_or((
            401,
            "unauthorized",
            "service_api_auth_nonce_header_missing",
            "missing required header: x-kamn-request-nonce",
        ))?
        .parse::<u64>()
        .map_err(|_| {
            (
                401,
                "unauthorized",
                "service_api_auth_nonce_invalid",
                "invalid request nonce header: x-kamn-request-nonce",
            )
        })?;
    let signature = headers.get("x-kamn-request-signature").ok_or((
        401,
        "unauthorized",
        "service_api_auth_signature_header_missing",
        "missing required header: x-kamn-request-signature",
    ))?;
    let expected = service_signature_for_fields(&did, nonce, CHAIN_ID, CHAIN_VERSION, body)
        .map_err(|_| {
            (
                401,
                "unauthorized",
                "service_api_auth_signature_verification_failed",
                "signature verification failed for request envelope",
            )
        })?;
    if expected != *signature {
        return Err((
            401,
            "unauthorized",
            "service_api_auth_signature_verification_failed",
            "signature verification failed for request envelope",
        ));
    }
    if !replay_guard.insert((did_value, nonce)) {
        return Err((
            401,
            "unauthorized",
            "service_api_auth_replay_nonce_detected",
            "request nonce replay detected for sender",
        ));
    }

    if method == "GET"
        && path.starts_with("/v1/agents/")
        && did.as_str() != expected_agent_sender_did
    {
        return Err((
            401,
            "unauthorized",
            "service_api_auth_sender_did_invalid",
            "agent route sender did mismatch",
        ));
    }

    if let Some(expected_scope) = required_scope_for_route(method, path) {
        let scope = headers.get(REQUEST_AUTH_SCOPE_HEADER).ok_or((
            401,
            "unauthorized",
            "service_api_auth_scope_header_missing",
            "missing required header: x-kamn-authz-scope",
        ))?;
        if scope != expected_scope {
            return Err((
                401,
                "unauthorized",
                "service_api_auth_scope_route_mismatch",
                "scope route mismatch",
            ));
        }
    }
    Ok(())
}

fn run_live_transport_contract_server(
    bind_addr: String,
    max_requests: u64,
    expected_agent_sender_did: &str,
    expected_message_body: Option<String>,
) -> Result<(), String> {
    let listener =
        TcpListener::bind(bind_addr.as_str()).map_err(|error| format!("bind failed: {error}"))?;
    listener
        .set_nonblocking(true)
        .map_err(|error| format!("nonblocking setup failed: {error}"))?;

    let deadline = Instant::now() + Duration::from_secs(2);
    let mut served = 0_u64;
    let mut replay_guard = BTreeSet::new();
    let mut registered_metadata: Option<(String, String, Vec<String>)> = None;
    while served < max_requests {
        if Instant::now() > deadline {
            return Err("server timed out before serving request budget".to_owned());
        }
        match listener.accept() {
            Ok((mut stream, _)) => {
                let (method, path, body, headers) = parse_http_request(&mut stream)?;
                if let Err((status, error, reason_code, message)) = validate_auth(
                    method.as_str(),
                    path.as_str(),
                    body.as_str(),
                    &headers,
                    &mut replay_guard,
                    expected_agent_sender_did,
                ) {
                    let payload = format!(
                        "{{\"error\":\"{error}\",\"reason_code\":\"{reason_code}\",\"message\":\"{message}\"}}"
                    );
                    write_http_response(&mut stream, status, payload.as_str())?;
                    served = served.saturating_add(1);
                    continue;
                }

                if method == "POST" && path == "/v1/messages/send" {
                    if let Some(expected_body) = expected_message_body.as_ref() {
                        if body != *expected_body {
                            return Err(format!(
                                "message payload mismatch, expected `{expected_body}` got `{body}`"
                            ));
                        }
                    }
                    let payload = r#"{"message_id":"msg-live-contract-001","status":"created","runtime_mode":"api"}"#;
                    write_http_response(&mut stream, 202, payload)?;
                } else if method == "POST" && path == "/v1/agents/register" {
                    let parsed: serde_json::Value =
                        serde_json::from_str(body.as_str()).map_err(|error| {
                            format!("registration payload should be valid json: {error}")
                        })?;
                    let agent_type = parsed
                        .get("agent_type")
                        .and_then(serde_json::Value::as_str)
                        .ok_or_else(|| "registration payload missing agent_type".to_owned())?
                        .to_owned();
                    let model_family = parsed
                        .get("model_family")
                        .and_then(serde_json::Value::as_str)
                        .ok_or_else(|| "registration payload missing model_family".to_owned())?
                        .to_owned();
                    let capabilities = parsed
                        .get("capabilities")
                        .and_then(serde_json::Value::as_array)
                        .ok_or_else(|| "registration payload missing capabilities".to_owned())?
                        .iter()
                        .map(|value| {
                            value
                                .as_str()
                                .map(str::to_owned)
                                .ok_or_else(|| "registration capability must be string".to_owned())
                        })
                        .collect::<Result<Vec<_>, _>>()?;
                    registered_metadata = Some((
                        agent_type.clone(),
                        model_family.clone(),
                        capabilities.clone(),
                    ));
                    let payload = format!(
                        "{{\"did\":\"{}\",\"reputation_score\":777,\"agent_type\":\"{}\",\"model_family\":\"{}\",\"capabilities\":{}}}",
                        expected_agent_sender_did,
                        agent_type,
                        model_family,
                        serde_json::to_string(&capabilities)
                            .map_err(|error| format!("capability serialization failed: {error}"))?
                    );
                    write_http_response(&mut stream, 201, payload.as_str())?;
                } else if method == "POST" && path == "/v1/agents/search" {
                    let parsed: serde_json::Value =
                        serde_json::from_str(body.as_str()).map_err(|error| {
                            format!("search payload should be valid json: {error}")
                        })?;
                    let capability = parsed
                        .get("capability")
                        .and_then(serde_json::Value::as_str)
                        .map(str::to_owned);
                    let model_family = parsed
                        .get("model_family")
                        .and_then(serde_json::Value::as_str)
                        .map(str::to_owned);
                    let candidate_rows = vec![
                        serde_json::json!({
                            "did": "kamn:did:agent:alpha",
                            "reputation_score": 777,
                            "agent_type": "assistant",
                            "model_family": "gpt-5",
                            "capabilities": ["text", "code"],
                        }),
                        serde_json::json!({
                            "did": "kamn:did:agent:beta",
                            "reputation_score": 650,
                            "agent_type": "assistant",
                            "model_family": "gpt-4.1",
                            "capabilities": ["text"],
                        }),
                    ];
                    let filtered: Vec<serde_json::Value> = candidate_rows
                        .into_iter()
                        .filter(|row| match model_family.as_deref() {
                            Some(expected) => row
                                .get("model_family")
                                .and_then(serde_json::Value::as_str)
                                == Some(expected),
                            None => true,
                        })
                        .filter(|row| match capability.as_deref() {
                            Some(expected) => row
                                .get("capabilities")
                                .and_then(serde_json::Value::as_array)
                                .map(|values| {
                                    values.iter().any(|value| {
                                        value.as_str().map(str::trim) == Some(expected)
                                    })
                                })
                                .unwrap_or(false),
                            None => true,
                        })
                        .collect();
                    let payload = serde_json::to_string(&filtered)
                        .map_err(|error| format!("search result serialization failed: {error}"))?;
                    write_http_response(&mut stream, 200, payload.as_str())?;
                } else if method == "GET" && path.starts_with("/v1/agents/") {
                    let did = path.trim_start_matches("/v1/agents/");
                    let (agent_type, model_family, capabilities) =
                        registered_metadata.clone().unwrap_or_else(|| {
                            (
                                "service-agent".to_owned(),
                                "service-api".to_owned(),
                                vec!["profile:read".to_owned()],
                            )
                        });
                    let payload = format!(
                        "{{\"did\":\"{}\",\"reputation_score\":777,\"agent_type\":\"{}\",\"model_family\":\"{}\",\"capabilities\":{}}}",
                        did,
                        agent_type,
                        model_family,
                        serde_json::to_string(&capabilities)
                            .map_err(|error| format!("capability serialization failed: {error}"))?
                    );
                    write_http_response(&mut stream, 200, payload.as_str())?;
                } else {
                    let payload = r#"{"error":"not-found","reason_code":"service_api_route_not_found","message":"route not found"}"#;
                    write_http_response(&mut stream, 401, payload)?;
                }

                served = served.saturating_add(1);
            }
            Err(error) if matches!(error.kind(), ErrorKind::WouldBlock) => {
                thread::sleep(Duration::from_millis(5));
            }
            Err(error) => return Err(format!("accept failed: {error}")),
        }
    }

    Ok(())
}

fn deterministic_message_id(service_message_id: &str) -> u64 {
    let mut acc: u64 = 0xcbf29ce484222325;
    for byte in service_message_id.as_bytes() {
        acc ^= u64::from(*byte);
        acc = acc.wrapping_mul(0x00000100000001B3);
    }
    acc
}

#[test]
fn unit_live_transport_config_rejects_non_http_endpoint() {
    with_env_lock(|| {
        assert_eq!(
            LiveTransportConfig::new("wss://live.kamn.testnet"),
            Err(SdkError::InvalidInput {
                field: "transport.endpoint",
                reason: "must start with http:// or https://",
            })
        );
    });
}

#[test]
fn spec_c01_live_transport_source_has_no_global_in_memory_registry() {
    with_env_lock(|| {
        let source = include_str!("../src/live.rs");
        assert!(
            !source.contains("InMemoryKamnClient"),
            "live transport must not proxy through in-memory client simulation"
        );
        assert!(
            !source.contains("OnceLock"),
            "live transport must not use process-global endpoint registry"
        );
    });
}

#[test]
fn spec_c02_live_transport_send_executes_network_contract() {
    with_env_lock(|| {
        ensure_live_test_env();
        let bind_addr = reserve_loopback_addr();
        let server_addr = bind_addr.clone();
        let server = thread::spawn(move || {
            run_live_transport_contract_server(server_addr, 1, "kamn:did:agent:live-tester", None)
        });
        wait_for_server_ready(bind_addr.as_str());

        let mut client = LiveTransportKamnClient::connect(format!("http://{bind_addr}").as_str())
            .expect("live client should connect");
        let message_id = client
            .send(Message {
                from: did("sender-live-contract"),
                to: did("recipient-live-contract"),
                body: "live contract payload".to_owned(),
                channel: None,
            })
            .expect("live send should succeed");
        assert_eq!(
            message_id.0,
            deterministic_message_id("msg-live-contract-001")
        );

        let server_result = server.join().expect("server thread should join");
        assert!(
            server_result.is_ok(),
            "test service contract server should satisfy request budget"
        );
    });
}

#[test]
fn spec_c03_live_transport_resolve_and_reputation_use_network_contract() {
    with_env_lock(|| {
        ensure_live_test_env();
        let bind_addr = reserve_loopback_addr();
        let server_addr = bind_addr.clone();
        let server = thread::spawn(move || {
            run_live_transport_contract_server(server_addr, 2, "kamn:did:agent:live-tester", None)
        });
        wait_for_server_ready(bind_addr.as_str());

        let client = LiveTransportKamnClient::connect(format!("http://{bind_addr}").as_str())
            .expect("live client should connect");
        let target = did("agent-profile-target");

        let document = client.resolve(&target).expect("resolve should succeed");
        assert_eq!(document.id, target);
        assert_eq!(document.metadata.agent_type, "service-agent");
        assert_eq!(document.metadata.model_family, "service-api");
        assert_eq!(document.service_endpoint, format!("http://{bind_addr}"));

        let reputation = client
            .get_reputation(&did("agent-profile-target"))
            .expect("reputation query should succeed");
        assert_eq!(reputation.did, did("agent-profile-target"));
        assert_eq!(reputation.score, 777);

        let server_result = server.join().expect("server thread should join");
        assert!(
            server_result.is_ok(),
            "test service contract server should satisfy request budget"
        );
    });
}

#[test]
fn regression_live_transport_unreachable_endpoint_fails_closed() {
    with_env_lock(|| {
        ensure_live_test_env();
        let mut client = LiveTransportKamnClient::connect("http://127.0.0.1:1")
            .expect("endpoint format should be accepted");

        let error = client
            .send(Message {
                from: did("unreachable-sender"),
                to: did("unreachable-recipient"),
                body: "payload".to_owned(),
                channel: None,
            })
            .expect_err("send should fail when endpoint is unavailable");
        assert_eq!(
            error,
            SdkError::TransportFailure("failed to connect to service endpoint")
        );
    });
}

#[test]
fn regression_live_transport_duplicate_service_message_id_reuses_alias() {
    with_env_lock(|| {
        ensure_live_test_env();
        let bind_addr = reserve_loopback_addr();
        let server_addr = bind_addr.clone();
        let server = thread::spawn(move || {
            run_live_transport_contract_server(server_addr, 2, "kamn:did:agent:live-tester", None)
        });
        wait_for_server_ready(bind_addr.as_str());

        let mut client = LiveTransportKamnClient::connect(format!("http://{bind_addr}").as_str())
            .expect("live client should connect");
        let first = client
            .send(Message {
                from: did("sender-live-contract"),
                to: did("recipient-live-contract"),
                body: "live contract payload one".to_owned(),
                channel: None,
            })
            .expect("first send should succeed");
        let second = client
            .send(Message {
                from: did("sender-live-contract"),
                to: did("recipient-live-contract"),
                body: "live contract payload two".to_owned(),
                channel: None,
            })
            .expect("second send should succeed");
        assert_eq!(first, second, "same service id must map to same sdk alias");

        let server_result = server.join().expect("server thread should join");
        assert!(
            server_result.is_ok(),
            "test service contract server should satisfy request budget"
        );
    });
}

#[test]
fn spec_c04_live_transport_send_escapes_json_payload_contract() {
    with_env_lock(|| {
        ensure_live_test_env();
        let bind_addr = reserve_loopback_addr();
        let server_addr = bind_addr.clone();
        let expected_payload = "{\"from\":\"kamn:did:agent:sender-escape\",\"to\":\"kamn:did:agent:recipient-escape\",\"body\":\"line\\n\\t\\\"slash\\\\bell\\u0007\",\"channel_id\":\"ops\\\"lane\"}".to_owned();
        let expected_for_server = expected_payload.clone();
        let server = thread::spawn(move || {
            run_live_transport_contract_server(
                server_addr,
                1,
                "kamn:did:agent:live-tester",
                Some(expected_for_server),
            )
        });
        wait_for_server_ready(bind_addr.as_str());

        let mut client = LiveTransportKamnClient::connect(format!("http://{bind_addr}").as_str())
            .expect("live client should connect");
        let message_id = client
            .send(Message {
                from: did("sender-escape"),
                to: did("recipient-escape"),
                body: "line\n\t\"slash\\bell\u{0007}".to_owned(),
                channel: Some(kamn_sdk::ChannelId("ops\"lane".to_owned())),
            })
            .expect("send should succeed");
        assert_eq!(
            message_id.0,
            deterministic_message_id("msg-live-contract-001")
        );

        let server_result = server.join().expect("server thread should join");
        assert!(
            server_result.is_ok(),
            "message payload must match json-escaped contract"
        );
        assert!(
            expected_payload.contains("\\u0007"),
            "expected payload fixture should include control-char escape marker"
        );
    });
}

#[test]
fn regression_live_transport_whitespace_requester_did_falls_back_to_default() {
    with_env_lock(|| {
        ensure_live_test_env();
        std::env::set_var(LIVE_REQUESTER_DID_ENV, "   ");
        let bind_addr = reserve_loopback_addr();
        let server_addr = bind_addr.clone();
        let server = thread::spawn(move || {
            run_live_transport_contract_server(server_addr, 1, DEFAULT_LIVE_REQUESTER_DID, None)
        });
        wait_for_server_ready(bind_addr.as_str());

        let client = LiveTransportKamnClient::connect(format!("http://{bind_addr}").as_str())
            .expect("live client should use default requester did when env is whitespace");
        let _ = client
            .resolve(&did("agent-profile-target"))
            .expect("resolve should succeed with default requester did");

        let server_result = server.join().expect("server thread should join");
        assert!(
            server_result.is_ok(),
            "whitespace requester did env should fallback to default requester did"
        );
    });
}

#[test]
fn spec_c05_live_transport_remaining_unsupported_methods_fail_closed() {
    with_env_lock(|| {
        ensure_live_test_env();
        let client = LiveTransportKamnClient::connect("http://127.0.0.1:65535")
            .expect("endpoint format should be accepted");

        assert_eq!(client.assert_transport_mode(TransportMode::Live), Ok(()));
    });
}

#[test]
fn spec_c06_live_transport_register_and_resolve_use_service_profile_metadata() {
    with_env_lock(|| {
        ensure_live_test_env();
        let bind_addr = reserve_loopback_addr();
        let server_addr = bind_addr.clone();
        let server = thread::spawn(move || {
            run_live_transport_contract_server(server_addr, 2, "kamn:did:agent:live-tester", None)
        });
        wait_for_server_ready(bind_addr.as_str());

        let mut client = LiveTransportKamnClient::connect(format!("http://{bind_addr}").as_str())
            .expect("live client should connect");
        let did = client
            .register(metadata("assistant", "gpt-5", &["text", "code"]))
            .expect("register should succeed over live transport");
        let resolved = client.resolve(&did).expect("resolve should succeed");

        assert_eq!(resolved.id, did);
        assert_eq!(resolved.metadata.agent_type, "assistant");
        assert_eq!(resolved.metadata.model_family, "gpt-5");
        assert_eq!(
            resolved.metadata.capabilities,
            vec!["text".to_owned(), "code".to_owned()]
        );

        let server_result = server.join().expect("server thread should join");
        assert!(
            server_result.is_ok(),
            "live transport register/resolve server should satisfy request budget"
        );
    });
}

#[test]
fn spec_c07_live_transport_search_agents_uses_service_route() {
    with_env_lock(|| {
        ensure_live_test_env();
        let bind_addr = reserve_loopback_addr();
        let server_addr = bind_addr.clone();
        let server = thread::spawn(move || {
            run_live_transport_contract_server(server_addr, 1, DEFAULT_LIVE_REQUESTER_DID, None)
        });
        wait_for_server_ready(bind_addr.as_str());

        let client = LiveTransportKamnClient::connect(format!("http://{bind_addr}").as_str())
            .expect("live client should connect");
        let results = client
            .search_agents(AgentQuery {
                capability: Some("code".to_owned()),
                model_family: Some("gpt-5".to_owned()),
            })
            .expect("live search_agents should succeed");

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].did.as_str(), "kamn:did:agent:alpha");
        assert_eq!(results[0].agent_type, "assistant");
        assert_eq!(results[0].model_family, "gpt-5");
        assert_eq!(
            results[0].capabilities,
            vec!["text".to_owned(), "code".to_owned()]
        );

        let server_result = server.join().expect("server thread should join");
        assert!(
            server_result.is_ok(),
            "live transport search server should satisfy request budget"
        );
    });
}

#[test]
fn regression_transport_mode_mismatch_is_rejected() {
    with_env_lock(|| {
        ensure_live_test_env();
        let live = LiveTransportKamnClient::connect("http://127.0.0.1:65535")
            .expect("connect live should succeed");
        assert_eq!(
            live.assert_transport_mode(TransportMode::InMemory),
            Err(SdkError::TransportModeMismatch {
                expected: "in-memory",
                found: "live",
            })
        );
    });
}
