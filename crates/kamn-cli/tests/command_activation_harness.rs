#![allow(dead_code)]

#[path = "command_activation_harness_routes.rs"]
mod command_activation_harness_routes;

use kamn_cli::{CommandKind, OutputFormat, ParsedCliArgs};
use std::io::{ErrorKind, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::thread;
use std::time::{Duration, Instant};

const SERVICE_AUTH_PRIVATE_KEY_ENV: &str = "KAMN_SERVICE_API_AUTH_PRIVATE_KEY_HEX";
const TEST_SERVICE_AUTH_PRIVATE_KEY_HEX: &str =
    "1111111111111111111111111111111111111111111111111111111111111111";

pub(crate) fn reserve_loopback_addr() -> String {
    let listener = TcpListener::bind("127.0.0.1:0").expect("listener should bind");
    let addr = listener.local_addr().expect("local addr should resolve");
    drop(listener);
    addr.to_string()
}

fn read_request(stream: &mut TcpStream) -> Result<String, String> {
    let mut request = Vec::new();
    let mut chunk = [0_u8; 1024];
    let deadline = Instant::now() + Duration::from_secs(5);
    stream
        .set_read_timeout(Some(Duration::from_millis(250)))
        .map_err(|error| format!("request read-timeout failed: {error}"))?;
    while Instant::now() <= deadline && !request.windows(4).any(|window| window == b"\r\n\r\n") {
        match stream.read(&mut chunk) {
            Ok(0) => thread::sleep(Duration::from_millis(5)),
            Ok(read_count) => request.extend_from_slice(&chunk[..read_count]),
            Err(error) if matches!(error.kind(), ErrorKind::WouldBlock | ErrorKind::TimedOut) => {}
            Err(error) => return Err(format!("request read failed: {error}")),
        }
    }
    String::from_utf8(request).map_err(|_| "request was not valid utf-8".to_owned())
}

fn parse_request_line(request: &str) -> Result<(String, String), String> {
    let (head, _) = request
        .split_once("\r\n\r\n")
        .ok_or_else(|| "request header terminator missing".to_owned())?;
    let line = head
        .lines()
        .next()
        .ok_or_else(|| "request line missing".to_owned())?;
    let mut parts = line.split_whitespace();
    let method = parts
        .next()
        .ok_or_else(|| "request method missing".to_owned())?;
    let path = parts
        .next()
        .ok_or_else(|| "request path missing".to_owned())?;
    Ok((method.to_owned(), path.to_owned()))
}

fn parse_http_request(stream: &mut TcpStream) -> Result<(String, String), String> {
    let request = read_request(stream)?;
    parse_request_line(request.as_str())
}

fn write_http_response(stream: &mut TcpStream, status: u16, body: &str) -> Result<(), String> {
    let status_text = match status {
        200 => "200 OK",
        201 => "201 Created",
        202 => "202 Accepted",
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

fn serve_connection(mut stream: TcpStream) -> Result<(), String> {
    let (method, path) = parse_http_request(&mut stream)?;
    let (status, body) =
        command_activation_harness_routes::response_for(method.as_str(), path.as_str());
    write_http_response(&mut stream, status, body)
}

pub(crate) fn run_cli_contract_server(
    bind_addr: String,
    max_requests: usize,
) -> Result<(), String> {
    let listener = TcpListener::bind(bind_addr.as_str())
        .map_err(|error| format!("server bind failed: {error}"))?;
    listener
        .set_nonblocking(true)
        .map_err(|error| format!("server nonblocking mode failed: {error}"))?;
    let deadline = Instant::now() + Duration::from_secs(15);
    let mut served = 0usize;
    while served < max_requests {
        if Instant::now() > deadline {
            return Err("server timed out before request budget".to_owned());
        }
        match listener.accept() {
            Ok((stream, _)) => {
                serve_connection(stream)?;
                served += 1;
            }
            Err(error) if error.kind() == ErrorKind::WouldBlock => {
                thread::sleep(Duration::from_millis(5))
            }
            Err(error) => return Err(format!("server accept failed: {error}")),
        }
    }
    Ok(())
}

pub(crate) fn wait_for_server_ready() {
    thread::sleep(Duration::from_millis(120));
}

fn parsed_with_format(
    command: CommandKind,
    endpoint: &str,
    output_format: OutputFormat,
    passthrough: &[&str],
) -> ParsedCliArgs {
    std::env::set_var(
        SERVICE_AUTH_PRIVATE_KEY_ENV,
        TEST_SERVICE_AUTH_PRIVATE_KEY_HEX,
    );
    ParsedCliArgs {
        command,
        output_format,
        endpoint: endpoint.to_owned(),
        passthrough: passthrough.iter().map(|value| value.to_string()).collect(),
    }
}

pub(crate) fn parsed(command: CommandKind, endpoint: &str, passthrough: &[&str]) -> ParsedCliArgs {
    parsed_with_format(command, endpoint, OutputFormat::Text, passthrough)
}

#[allow(dead_code)]
pub(crate) fn parsed_json(
    command: CommandKind,
    endpoint: &str,
    passthrough: &[&str],
) -> ParsedCliArgs {
    parsed_with_format(command, endpoint, OutputFormat::Json, passthrough)
}
