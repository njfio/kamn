use kamn_agent_lib::{AgentIdentity, KamnAgentHandle};
use kamn_mcp_server::{dispatch_tool_request_json, process_stdio_input};
use std::io::{ErrorKind, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::thread;
use std::time::{Duration, Instant};

fn reserve_loopback_addr() -> String {
    let listener = TcpListener::bind("127.0.0.1:0").expect("listener should bind");
    let addr = listener.local_addr().expect("local addr should resolve");
    drop(listener);
    addr.to_string()
}

fn parse_http_request(stream: &mut TcpStream) -> Result<(String, String), String> {
    let mut request = Vec::new();
    let mut chunk = [0_u8; 1024];
    stream
        .set_read_timeout(Some(Duration::from_secs(2)))
        .map_err(|error| format!("request read-timeout failed: {error}"))?;

    loop {
        match stream.read(&mut chunk) {
            Ok(0) => break,
            Ok(read_count) => {
                request.extend_from_slice(&chunk[..read_count]);
                if request.windows(4).any(|window| window == b"\r\n\r\n") {
                    break;
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
    let Some((request_head, _)) = request_text.split_once("\r\n\r\n") else {
        return Err("request header terminator missing".to_owned());
    };
    let request_line = request_head
        .lines()
        .next()
        .ok_or_else(|| "request line missing".to_owned())?;
    let mut parts = request_line.split_whitespace();
    let method = parts
        .next()
        .ok_or_else(|| "request method missing".to_owned())?
        .to_owned();
    let path = parts
        .next()
        .ok_or_else(|| "request path missing".to_owned())?
        .to_owned();
    Ok((method, path))
}

fn write_http_response(stream: &mut TcpStream, status: u16, body: &str) -> Result<(), String> {
    let status_text = match status {
        200 => "200 OK",
        201 => "201 Created",
        202 => "202 Accepted",
        404 => "404 Not Found",
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

fn run_real_backend_service_server(bind_addr: String, max_requests: usize) -> Result<(), String> {
    let listener = TcpListener::bind(bind_addr.as_str())
        .map_err(|error| format!("server bind failed: {error}"))?;
    listener
        .set_nonblocking(true)
        .map_err(|error| format!("server nonblocking mode failed: {error}"))?;

    let deadline = Instant::now() + Duration::from_secs(2);
    let mut served = 0usize;
    while served < max_requests {
        if Instant::now() > deadline {
            return Err("server timed out before serving request budget".to_owned());
        }
        match listener.accept() {
            Ok((mut stream, _)) => {
                let (method, path) = parse_http_request(&mut stream)?;
                if method == "GET" && path == "/healthz" {
                    write_http_response(
                        &mut stream,
                        200,
                        r#"{"status":"ok","runtime_mode":"api","role":"processor","observability_source":"unknown","observability_health":"unknown"}"#,
                    )?;
                } else if method == "GET" && path == "/v1/channels/channel-contract-1/messages" {
                    write_http_response(
                        &mut stream,
                        200,
                        r#"{"channel_id":"channel-contract-1","messages":["msg-a","msg-b"]}"#,
                    )?;
                } else if method == "GET" && path == "/v1/tasks/task-contract-1" {
                    write_http_response(
                        &mut stream,
                        200,
                        r#"{"task_id":"task-contract-1","state":"submitted"}"#,
                    )?;
                } else if method == "GET" && path == "/v1/agents/kamn:did:agent:alice" {
                    write_http_response(
                        &mut stream,
                        200,
                        r#"{"did":"kamn:did:agent:alice","reputation_score":42}"#,
                    )?;
                } else if method == "POST" && path == "/v1/content/register" {
                    write_http_response(
                        &mut stream,
                        201,
                        r#"{"content_id":"content-contract-1","retention_class":"standard","lifecycle_state":"retained","redaction_status":"none"}"#,
                    )?;
                } else if method == "POST" && path == "/v1/content/content-contract-1/expire" {
                    write_http_response(
                        &mut stream,
                        200,
                        r#"{"content_id":"content-contract-1","lifecycle_state":"expired","redaction_status":"none"}"#,
                    )?;
                } else if (method == "POST" && path == "/v1/content/content-contract-1/tombstone")
                    || (method == "GET" && path == "/v1/content/content-contract-1")
                {
                    write_http_response(
                        &mut stream,
                        200,
                        r#"{"content_id":"content-contract-1","lifecycle_state":"tombstoned","redaction_status":"redacted"}"#,
                    )?;
                } else if method == "POST" && path == "/v1/bridge/submit" {
                    write_http_response(
                        &mut stream,
                        202,
                        r#"{"bridge_id":"bridge-contract-1","source_message_id":"msg-bridge-source-1","bridge_status":"submitted"}"#,
                    )?;
                } else if method == "POST" && path == "/v1/bridge/bridge-contract-1/forward" {
                    write_http_response(
                        &mut stream,
                        200,
                        r#"{"bridge_id":"bridge-contract-1","bridge_status":"forwarded","target_message_id":"msg-bridge-target-1","forward_tx_hash":"sha256:bridge-forwarded-1"}"#,
                    )?;
                } else if method == "GET" && path == "/v1/bridge/bridge-contract-1" {
                    write_http_response(
                        &mut stream,
                        200,
                        r#"{"bridge_id":"bridge-contract-1","bridge_status":"forwarded","target_message_id":"msg-bridge-target-1","forward_tx_hash":"sha256:bridge-forwarded-1"}"#,
                    )?;
                } else {
                    write_http_response(
                        &mut stream,
                        404,
                        r#"{"error":"not-found","reason_code":"route_not_found"}"#,
                    )?;
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

fn wait_for_server_ready() {
    thread::sleep(Duration::from_millis(40));
}

fn real_backend(bind_addr: &str) -> KamnAgentHandle {
    let identity = AgentIdentity::from_agent_name("mcp-real-backend").expect("identity");
    KamnAgentHandle::with_identity(
        format!("http://{bind_addr}").as_str(),
        "http://127.0.0.1:3000",
        identity,
    )
    .expect("handle")
}

fn frame_request(body: &str) -> String {
    format!("Content-Length: {}\r\n\r\n{}", body.len(), body)
}

fn parse_framed_json(response: &str) -> String {
    let marker = "\r\n\r\n";
    let split = response
        .find(marker)
        .expect("framed response should include header/body split");
    let header = &response[..split];
    let body = &response[split + marker.len()..];
    let declared_length = header
        .strip_prefix("Content-Length: ")
        .expect("header should start with content length")
        .trim()
        .parse::<usize>()
        .expect("content length should be numeric");
    assert_eq!(
        declared_length,
        body.len(),
        "declared content length should match JSON body bytes",
    );
    body.to_owned()
}

#[test]
fn spec_c01_real_backend_dispatch_health_contract() {
    let bind_addr = reserve_loopback_addr();
    let server_addr = bind_addr.clone();
    let server = thread::spawn(move || run_real_backend_service_server(server_addr, 1));
    wait_for_server_ready();

    let backend = real_backend(bind_addr.as_str());
    let response = dispatch_tool_request_json(&backend, r#"{"id":"req-1","tool":"health"}"#)
        .expect("health dispatch should succeed");

    assert!(response.contains(r#""ok":true"#));
    assert!(response.contains(r#""tool":"health""#));
    assert!(response.contains(r#""status":"ok""#));
    assert!(response.contains(r#""runtime_mode":"api""#));

    let server_result = server.join().expect("server thread should join");
    assert!(
        server_result.is_ok(),
        "service fixture should satisfy request budget"
    );
}

#[test]
fn spec_c02_real_backend_dispatch_list_messages_contract() {
    let bind_addr = reserve_loopback_addr();
    let server_addr = bind_addr.clone();
    let server = thread::spawn(move || run_real_backend_service_server(server_addr, 1));
    wait_for_server_ready();

    let backend = real_backend(bind_addr.as_str());
    let response = dispatch_tool_request_json(
        &backend,
        r#"{"id":"req-2","tool":"list_messages","channel_id":"channel-contract-1"}"#,
    )
    .expect("list_messages dispatch should succeed");

    assert!(response.contains(r#""ok":true"#));
    assert!(response.contains(r#""tool":"list_messages""#));
    assert!(response.contains(r#""channel_id":"channel-contract-1""#));
    assert!(response.contains(r#""messages":["msg-a","msg-b"]"#));

    let server_result = server.join().expect("server thread should join");
    assert!(
        server_result.is_ok(),
        "service fixture should satisfy request budget"
    );
}

#[test]
fn spec_c03_real_backend_stdio_tools_call_contract() {
    let bind_addr = reserve_loopback_addr();
    let server_addr = bind_addr.clone();
    let server = thread::spawn(move || run_real_backend_service_server(server_addr, 1));
    wait_for_server_ready();

    let backend = real_backend(bind_addr.as_str());
    let request = frame_request(
        r#"{"jsonrpc":"2.0","id":"req-3","method":"tools/call","params":{"name":"list_messages","arguments":{"channel_id":"channel-contract-1"}}}"#,
    );

    let responses = process_stdio_input(&backend, request.as_str()).expect("stdio should parse");
    assert_eq!(responses.len(), 1, "tools/call should return one response");
    let body = parse_framed_json(responses[0].as_str());

    assert!(body.contains(r#""jsonrpc":"2.0""#));
    assert!(body.contains(r#""id":"req-3""#));
    assert!(body.contains(r#""result""#));
    assert!(body.contains(r#""tool":"list_messages""#));
    assert!(body.contains(r#""channel_id":"channel-contract-1""#));

    let server_result = server.join().expect("server thread should join");
    assert!(
        server_result.is_ok(),
        "service fixture should satisfy request budget"
    );
}

#[test]
fn spec_c04_real_backend_dispatch_invalid_request_contract() {
    let bind_addr = reserve_loopback_addr();
    let backend = real_backend(bind_addr.as_str());

    let response = dispatch_tool_request_json(&backend, r#"{"id":"req-4","tool":"list_messages"}"#)
        .expect("dispatcher should return structured invalid-request envelope");

    assert!(response.contains(r#""ok":false"#));
    assert!(response.contains(r#""kind":"invalid_request""#));
    assert!(response.contains("missing required field: channel_id"));
}

#[test]
fn spec_c05_real_backend_dispatch_query_task_contract() {
    let bind_addr = reserve_loopback_addr();
    let server_addr = bind_addr.clone();
    let server = thread::spawn(move || run_real_backend_service_server(server_addr, 1));
    wait_for_server_ready();

    let backend = real_backend(bind_addr.as_str());
    let response = dispatch_tool_request_json(
        &backend,
        r#"{"id":"req-5","tool":"query_task","task_id":"task-contract-1"}"#,
    )
    .expect("query_task dispatch should succeed");

    assert!(response.contains(r#""ok":true"#));
    assert!(response.contains(r#""tool":"query_task""#));
    assert!(response.contains(r#""task_id":"task-contract-1""#));
    assert!(response.contains(r#""state":"submitted""#));

    let server_result = server.join().expect("server thread should join");
    assert!(
        server_result.is_ok(),
        "service fixture should satisfy request budget"
    );
}

#[test]
fn spec_c06_real_backend_dispatch_query_agent_profile_contract() {
    let bind_addr = reserve_loopback_addr();
    let server_addr = bind_addr.clone();
    let server = thread::spawn(move || run_real_backend_service_server(server_addr, 1));
    wait_for_server_ready();

    let backend = real_backend(bind_addr.as_str());
    let response = dispatch_tool_request_json(
        &backend,
        r#"{"id":"req-6","tool":"query_agent_profile","did":"kamn:did:agent:alice"}"#,
    )
    .expect("query_agent_profile dispatch should succeed");

    assert!(response.contains(r#""ok":true"#));
    assert!(response.contains(r#""tool":"query_agent_profile""#));
    assert!(response.contains(r#""did":"kamn:did:agent:alice""#));
    assert!(response.contains(r#""reputation_score":42"#));

    let server_result = server.join().expect("server thread should join");
    assert!(
        server_result.is_ok(),
        "service fixture should satisfy request budget"
    );
}

#[test]
fn spec_c08_real_backend_dispatch_content_lifecycle_contract() {
    let bind_addr = reserve_loopback_addr();
    let server_addr = bind_addr.clone();
    let server = thread::spawn(move || run_real_backend_service_server(server_addr, 4));
    wait_for_server_ready();

    let backend = real_backend(bind_addr.as_str());

    let register_response = dispatch_tool_request_json(
        &backend,
        r#"{"id":"req-8a","tool":"register_content","payload":"{\"content\":\"real-backend\"}"}"#,
    )
    .expect("register_content dispatch should succeed");
    assert!(register_response.contains(r#""ok":true"#));
    assert!(register_response.contains(r#""tool":"register_content""#));
    assert!(register_response.contains(r#""content_id":"content-contract-1""#));
    assert!(register_response.contains(r#""retention_class":"standard""#));

    let expire_response = dispatch_tool_request_json(
        &backend,
        r#"{"id":"req-8b","tool":"expire_content","content_id":"content-contract-1"}"#,
    )
    .expect("expire_content dispatch should succeed");
    assert!(expire_response.contains(r#""ok":true"#));
    assert!(expire_response.contains(r#""tool":"expire_content""#));
    assert!(expire_response.contains(r#""content_id":"content-contract-1""#));
    assert!(expire_response.contains(r#""lifecycle_state":"expired""#));

    let tombstone_response = dispatch_tool_request_json(
        &backend,
        r#"{"id":"req-8c","tool":"tombstone_content","content_id":"content-contract-1"}"#,
    )
    .expect("tombstone_content dispatch should succeed");
    assert!(tombstone_response.contains(r#""ok":true"#));
    assert!(tombstone_response.contains(r#""tool":"tombstone_content""#));
    assert!(tombstone_response.contains(r#""content_id":"content-contract-1""#));
    assert!(tombstone_response.contains(r#""redaction_status":"redacted""#));

    let query_response = dispatch_tool_request_json(
        &backend,
        r#"{"id":"req-8d","tool":"query_content","content_id":"content-contract-1"}"#,
    )
    .expect("query_content dispatch should succeed");
    assert!(query_response.contains(r#""ok":true"#));
    assert!(query_response.contains(r#""tool":"query_content""#));
    assert!(query_response.contains(r#""content_id":"content-contract-1""#));
    assert!(query_response.contains(r#""lifecycle_state":"tombstoned""#));
    assert!(query_response.contains(r#""redaction_status":"redacted""#));

    let server_result = server.join().expect("server thread should join");
    assert!(
        server_result.is_ok(),
        "service fixture should satisfy request budget"
    );
}

#[test]
fn spec_c09_real_backend_dispatch_bridge_lifecycle_contract() {
    let bind_addr = reserve_loopback_addr();
    let server_addr = bind_addr.clone();
    let server = thread::spawn(move || run_real_backend_service_server(server_addr, 3));
    wait_for_server_ready();

    let backend = real_backend(bind_addr.as_str());

    let submit_response = dispatch_tool_request_json(
        &backend,
        r#"{"id":"req-9a","tool":"submit_bridge_message","payload":"{\"source_message_id\":\"msg-1\"}"}"#,
    )
    .expect("submit_bridge_message dispatch should succeed");
    assert!(submit_response.contains(r#""ok":true"#));
    assert!(submit_response.contains(r#""tool":"submit_bridge_message""#));
    assert!(submit_response.contains(r#""bridge_id":"bridge-contract-1""#));
    assert!(submit_response.contains(r#""bridge_status":"submitted""#));

    let forward_response = dispatch_tool_request_json(
        &backend,
        r#"{"id":"req-9b","tool":"forward_bridge_message","bridge_id":"bridge-contract-1"}"#,
    )
    .expect("forward_bridge_message dispatch should succeed");
    assert!(forward_response.contains(r#""ok":true"#));
    assert!(forward_response.contains(r#""tool":"forward_bridge_message""#));
    assert!(forward_response.contains(r#""bridge_status":"forwarded""#));

    let query_response = dispatch_tool_request_json(
        &backend,
        r#"{"id":"req-9c","tool":"query_bridge_message","bridge_id":"bridge-contract-1"}"#,
    )
    .expect("query_bridge_message dispatch should succeed");
    assert!(query_response.contains(r#""ok":true"#));
    assert!(query_response.contains(r#""tool":"query_bridge_message""#));
    assert!(query_response.contains(r#""forward_tx_hash":"sha256:bridge-forwarded-1""#));

    let server_result = server.join().expect("server thread should join");
    assert!(
        server_result.is_ok(),
        "service fixture should satisfy request budget"
    );
}
