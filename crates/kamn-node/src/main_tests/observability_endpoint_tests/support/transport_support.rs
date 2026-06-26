use super::super::*;
use std::io::{ErrorKind, Read, Write};
use std::net::TcpStream;
use std::thread;

fn build_request(host: &str, path: &str) -> String {
    format!("GET {path} HTTP/1.1\r\nHost: {host}\r\nConnection: close\r\n\r\n")
}

fn read_response<R: Read>(reader: &mut R) -> Result<String, String> {
    let mut response = String::new();
    let mut chunk = [0_u8; 1024];
    loop {
        match reader.read(&mut chunk) {
            Ok(0) => return Ok(response),
            Ok(read_count) => response.push_str(
                std::str::from_utf8(&chunk[..read_count])
                    .map_err(|error| format!("response must be utf-8: {error}"))?,
            ),
            Err(error) if matches!(error.kind(), ErrorKind::WouldBlock | ErrorKind::TimedOut) => {
                return Ok(response);
            }
            Err(error) => return Err(format!("response should be readable: {error}")),
        }
    }
}

fn try_send_request(addr: &str, request: &str) -> Result<String, String> {
    let mut stream =
        TcpStream::connect(addr).map_err(|error| format!("connect should succeed: {error}"))?;
    stream
        .set_read_timeout(Some(Duration::from_secs(2)))
        .map_err(|error| format!("read timeout should be configurable: {error}"))?;
    stream
        .write_all(request.as_bytes())
        .map_err(|error| format!("request should write: {error}"))?;
    read_response(&mut stream)
}

pub(in super::super) fn send_http_get(addr: &str, path: &str) -> String {
    try_send_http_get(addr, path).expect("endpoint should accept connections")
}

pub(in super::super) fn try_send_http_get(addr: &str, path: &str) -> Result<String, String> {
    try_send_request(addr, build_request(addr, path).as_str())
}

pub(in super::super) fn send_raw_http_request(addr: &str, request: &str) -> String {
    try_send_request(addr, request).expect("endpoint should accept raw requests")
}

pub(in super::super) fn wait_for_endpoint_ready(addr: &str) {
    let deadline = Instant::now() + Duration::from_secs(2);
    while Instant::now() < deadline {
        if try_send_http_get(addr, "/readyz")
            .map(|response| response.contains("HTTP/1.1"))
            .unwrap_or(false)
        {
            return;
        }
        thread::sleep(Duration::from_millis(10));
    }
    panic!("endpoint did not become ready within timeout");
}
