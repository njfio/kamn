use kamn_agent_lib::{AgentIdentity, KamnAgentHandle};
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

fn parse_http_request(stream: &mut TcpStream) -> Result<(String, String, String), String> {
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
    let mut parts = request_line.split_whitespace();
    let method = parts
        .next()
        .ok_or_else(|| "request method missing".to_owned())?
        .to_owned();
    let path = parts
        .next()
        .ok_or_else(|| "request path missing".to_owned())?
        .to_owned();
    Ok((method, path, request_body.to_owned()))
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

fn deterministic_tag(payload: &[u8]) -> u64 {
    let mut acc: u64 = 0xcbf29ce484222325;
    for byte in payload {
        acc = acc.wrapping_mul(0x00000100000001B3);
        acc ^= u64::from(*byte);
    }
    acc
}

fn run_task_and_escrow_server(bind_addr: String, max_requests: u64) -> Result<(), String> {
    let listener = TcpListener::bind(bind_addr.as_str())
        .map_err(|error| format!("server bind failed: {error}"))?;
    listener
        .set_nonblocking(true)
        .map_err(|error| format!("server nonblocking mode failed: {error}"))?;

    let deadline = Instant::now() + Duration::from_secs(2);
    let mut served = 0_u64;
    while served < max_requests {
        if Instant::now() > deadline {
            return Err("server timed out before serving request budget".to_owned());
        }
        match listener.accept() {
            Ok((mut stream, _)) => {
                let (method, path, body) = parse_http_request(&mut stream)?;
                if method == "POST" && path == "/v1/tasks/task-contract-1/accept" {
                    write_http_response(
                        &mut stream,
                        200,
                        r#"{"task_id":"task-contract-1","state":"accepted"}"#,
                    )?;
                } else if method == "POST" && path == "/v1/tasks/task-contract-1/complete" {
                    write_http_response(
                        &mut stream,
                        200,
                        r#"{"task_id":"task-contract-1","state":"completed"}"#,
                    )?;
                } else if method == "POST" && path == "/v1/escrow/fund" {
                    let escrow_id =
                        format!("escrow-local-{:016x}", deterministic_tag(body.as_bytes()));
                    let payload = format!("{{\"escrow_id\":\"{escrow_id}\",\"state\":\"funded\"}}");
                    write_http_response(&mut stream, 200, payload.as_str())?;
                } else if method == "POST"
                    && path.starts_with("/v1/escrow/")
                    && path.ends_with("/release")
                {
                    let escrow_id = path
                        .trim_start_matches("/v1/escrow/")
                        .trim_end_matches("/release")
                        .trim_end_matches('/');
                    let payload =
                        format!("{{\"escrow_id\":\"{escrow_id}\",\"state\":\"released\"}}");
                    write_http_response(&mut stream, 200, payload.as_str())?;
                } else {
                    write_http_response(
                        &mut stream,
                        404,
                        r#"{"error":"not-found","reason_code":"service_api_route_not_found","message":"not found"}"#,
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

#[test]
fn spec_c04_agent_handle_executes_task_and_escrow_route_contracts() {
    let bind_addr = reserve_loopback_addr();
    let server_addr = bind_addr.clone();
    let server = thread::spawn(move || run_task_and_escrow_server(server_addr, 4));
    wait_for_server_ready();

    let identity = AgentIdentity::from_agent_name("alice").expect("identity");
    let handle = KamnAgentHandle::with_identity(
        format!("http://{bind_addr}").as_str(),
        "http://localhost:3000",
        identity,
    )
    .expect("handle");

    let accepted = handle
        .accept_task("task-contract-1")
        .expect("accept task should succeed");
    assert_eq!(accepted.task_id, "task-contract-1");
    assert_eq!(accepted.state, "accepted");

    let completed = handle
        .complete_task("task-contract-1")
        .expect("complete task should succeed");
    assert_eq!(completed.task_id, "task-contract-1");
    assert_eq!(completed.state, "completed");

    let payload = r#"{"task_id":"task-contract-1","amount":100}"#;
    let funded = handle
        .fund_escrow(payload)
        .expect("fund escrow should succeed");
    assert!(funded.escrow_id.starts_with("escrow-local-"));
    assert_eq!(funded.state, "funded");

    let released = handle
        .release_escrow(funded.escrow_id.as_str())
        .expect("release escrow should succeed");
    assert_eq!(released.escrow_id, funded.escrow_id);
    assert_eq!(released.state, "released");

    let server_result = server.join().expect("server thread should join");
    assert!(
        server_result.is_ok(),
        "test service contract server should satisfy request budget"
    );
}
