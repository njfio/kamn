use kamn_agent_lib::{AgentIdentity, KamnAgentHandle};
use std::ffi::OsString;
use std::io::{ErrorKind, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::thread;
use std::time::{Duration, Instant};

const SERVICE_AUTH_PRIVATE_KEY_ENV: &str = "KAMN_SERVICE_API_AUTH_PRIVATE_KEY_HEX";
const TEST_SERVICE_AUTH_PRIVATE_KEY_HEX: &str =
    "658c3528422eb527b4c108b8f6d1e5f629543c304ea49cf608c67794424291c4";

struct EnvVarGuard {
    key: &'static str,
    previous: Option<OsString>,
}

impl EnvVarGuard {
    fn set(key: &'static str, value: &str) -> Self {
        let previous = std::env::var_os(key);
        // SAFETY: test mutates process env in a single-threaded setup for this key.
        unsafe { std::env::set_var(key, value) };
        Self { key, previous }
    }
}

impl Drop for EnvVarGuard {
    fn drop(&mut self) {
        match self.previous.take() {
            Some(previous) => {
                // SAFETY: restoring process env key after test completion.
                unsafe { std::env::set_var(self.key, previous) }
            }
            None => {
                // SAFETY: restoring process env key after test completion.
                unsafe { std::env::remove_var(self.key) }
            }
        }
    }
}

fn bind_loopback_listener() -> TcpListener {
    let listener = TcpListener::bind("127.0.0.1:0").expect("listener should bind");
    listener
        .set_nonblocking(true)
        .expect("listener nonblocking mode should configure");
    listener
}

fn parse_http_request(stream: &mut TcpStream) -> Result<(String, String, String), String> {
    let mut request = Vec::new();
    let mut chunk = [0_u8; 1024];
    stream
        .set_read_timeout(Some(Duration::from_millis(100)))
        .map_err(|error| format!("request read-timeout failed: {error}"))?;

    let deadline = Instant::now() + Duration::from_secs(10);
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
                if Instant::now() > deadline {
                    return Err("request read timed out before complete http payload".to_owned());
                }
                thread::sleep(Duration::from_millis(5));
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
        .map_err(|error| format!("service api write failed: {error}"))?;
    stream
        .flush()
        .map_err(|error| format!("service api flush failed: {error}"))
}

fn deterministic_tag(payload: &[u8]) -> u64 {
    let mut acc: u64 = 0xcbf29ce484222325;
    for byte in payload {
        acc = acc.wrapping_mul(0x00000100000001B3);
        acc ^= u64::from(*byte);
    }
    acc
}

fn run_task_and_escrow_server(listener: TcpListener, max_requests: u64) -> Result<(), String> {
    let deadline = Instant::now() + Duration::from_secs(10);
    let mut served = 0_u64;
    while served < max_requests {
        if Instant::now() > deadline {
            return Err("server timed out before serving request budget".to_owned());
        }
        served = served.saturating_add(accept_task_escrow_request(&listener)?);
    }
    Ok(())
}

fn accept_task_escrow_request(listener: &TcpListener) -> Result<u64, String> {
    match listener.accept() {
        Ok((mut stream, _)) => {
            serve_task_escrow_connection(&mut stream)?;
            Ok(1)
        }
        Err(error) if error.kind() == ErrorKind::WouldBlock => {
            thread::sleep(Duration::from_millis(5));
            Ok(0)
        }
        Err(error) => Err(format!("server accept failed: {error}")),
    }
}

fn serve_task_escrow_connection(stream: &mut TcpStream) -> Result<(), String> {
    let (method, path, body) = parse_http_request(stream)?;
    if write_task_response(stream, &method, &path)? {
        return Ok(());
    }
    if write_escrow_response(stream, &method, &path, &body)? {
        return Ok(());
    }
    write_http_response(
        stream,
        404,
        r#"{"error":"not-found","reason_code":"service_api_route_not_found","message":"not found"}"#,
    )
}

fn write_task_response(stream: &mut TcpStream, method: &str, path: &str) -> Result<bool, String> {
    let body = match (method, path) {
        ("POST", "/v1/tasks/task-contract-1/accept") => {
            r#"{"task_id":"task-contract-1","state":"accepted"}"#
        }
        ("POST", "/v1/tasks/task-contract-1/complete") => {
            r#"{"task_id":"task-contract-1","state":"completed"}"#
        }
        _ => return Ok(false),
    };
    write_http_response(stream, 200, body)?;
    Ok(true)
}

fn write_escrow_response(
    stream: &mut TcpStream,
    method: &str,
    path: &str,
    body: &str,
) -> Result<bool, String> {
    if method == "POST" && path == "/v1/escrow/fund" {
        let escrow_id = format!("escrow-local-{:016x}", deterministic_tag(body.as_bytes()));
        let payload = format!("{{\"escrow_id\":\"{escrow_id}\",\"state\":\"funded\"}}");
        write_http_response(stream, 200, payload.as_str())?;
        return Ok(true);
    }
    if method == "POST" && path.starts_with("/v1/escrow/") && path.ends_with("/release") {
        let escrow_id = path
            .trim_start_matches("/v1/escrow/")
            .trim_end_matches("/release")
            .trim_end_matches('/');
        let payload = format!("{{\"escrow_id\":\"{escrow_id}\",\"state\":\"released\"}}");
        write_http_response(stream, 200, payload.as_str())?;
        return Ok(true);
    }
    Ok(false)
}

#[test]
fn spec_c04_agent_handle_executes_task_and_escrow_route_contracts() {
    let _auth_key_guard = EnvVarGuard::set(
        SERVICE_AUTH_PRIVATE_KEY_ENV,
        TEST_SERVICE_AUTH_PRIVATE_KEY_HEX,
    );
    let listener = bind_loopback_listener();
    let bind_addr = listener
        .local_addr()
        .expect("listener address should resolve")
        .to_string();
    let server = thread::spawn(move || run_task_and_escrow_server(listener, 4));

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
