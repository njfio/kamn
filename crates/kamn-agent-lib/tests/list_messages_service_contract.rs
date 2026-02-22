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

fn run_list_messages_server(bind_addr: String) -> Result<(), String> {
    let listener = TcpListener::bind(bind_addr.as_str())
        .map_err(|error| format!("server bind failed: {error}"))?;
    listener
        .set_nonblocking(true)
        .map_err(|error| format!("server nonblocking mode failed: {error}"))?;

    let deadline = Instant::now() + Duration::from_secs(2);
    loop {
        if Instant::now() > deadline {
            return Err("server timed out before receiving request".to_owned());
        }
        match listener.accept() {
            Ok((mut stream, _)) => {
                let (method, path) = parse_http_request(&mut stream)?;
                if method == "GET" && path == "/v1/channels/channel-contract-1/messages" {
                    write_http_response(
                        &mut stream,
                        200,
                        r#"{"channel_id":"channel-contract-1","messages":["msg-one","msg-two"]}"#,
                    )?;
                    return Ok(());
                }
                write_http_response(
                    &mut stream,
                    200,
                    r#"{"channel_id":"unknown","messages":[]}"#,
                )?;
                return Ok(());
            }
            Err(error) if error.kind() == ErrorKind::WouldBlock => {
                thread::sleep(Duration::from_millis(5));
            }
            Err(error) => return Err(format!("server accept failed: {error}")),
        }
    }
}

fn wait_for_server_ready() {
    thread::sleep(Duration::from_millis(40));
}

#[test]
fn spec_c02_agent_handle_list_messages_uses_service_route_contract() {
    let bind_addr = reserve_loopback_addr();
    let server_addr = bind_addr.clone();
    let server = thread::spawn(move || run_list_messages_server(server_addr));
    wait_for_server_ready();

    let identity = AgentIdentity::from_agent_name("alice").expect("identity");
    let handle = KamnAgentHandle::with_identity(
        format!("http://{bind_addr}").as_str(),
        "http://localhost:3000",
        identity,
    )
    .expect("handle");

    let response = handle
        .list_messages("channel-contract-1")
        .expect("list messages should succeed");
    assert_eq!(response.channel_id, "channel-contract-1");
    assert_eq!(
        response.messages,
        vec!["msg-one".to_owned(), "msg-two".to_owned()]
    );

    let server_result = server.join().expect("server thread should join");
    assert!(
        server_result.is_ok(),
        "test service contract server should satisfy request budget"
    );
}
