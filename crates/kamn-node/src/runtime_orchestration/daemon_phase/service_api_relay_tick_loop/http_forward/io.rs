use std::io::{Read, Write};
use std::net::{SocketAddr, TcpStream};
use std::time::Duration;

const SERVICE_API_RELAY_FORWARD_CONNECT_TIMEOUT_MS: u64 = 500;
const SERVICE_API_RELAY_FORWARD_IO_TIMEOUT_MS: u64 = 500;

pub(super) fn send_relay_request(relay_addr: &str, request: &str) -> Result<(), String> {
    let relay_socket_addr = parse_relay_socket_addr(relay_addr)?;
    let mut stream = connect_relay_stream(relay_addr, relay_socket_addr)?;
    configure_relay_stream(&stream)?;
    write_relay_request(&mut stream, request)?;
    let response = read_http_response(stream)?;
    validate_http_status(relay_addr, response.as_str())
}

fn parse_relay_socket_addr(relay_addr: &str) -> Result<SocketAddr, String> {
    relay_addr.parse::<SocketAddr>().map_err(|error| {
        format!("relay recipient address parse failed: addr={relay_addr}: {error}")
    })
}

fn connect_relay_stream(
    relay_addr: &str,
    relay_socket_addr: SocketAddr,
) -> Result<TcpStream, String> {
    TcpStream::connect_timeout(
        &relay_socket_addr,
        Duration::from_millis(SERVICE_API_RELAY_FORWARD_CONNECT_TIMEOUT_MS),
    )
    .map_err(|error| format!("relay recipient connect failed: addr={relay_addr}: {error}"))
}

fn configure_relay_stream(stream: &TcpStream) -> Result<(), String> {
    let timeout = Duration::from_millis(SERVICE_API_RELAY_FORWARD_IO_TIMEOUT_MS);
    stream
        .set_write_timeout(Some(timeout))
        .map_err(|error| format!("relay recipient write-timeout set failed: {error}"))?;
    stream
        .set_read_timeout(Some(timeout))
        .map_err(|error| format!("relay recipient read-timeout set failed: {error}"))
}

fn write_relay_request(stream: &mut TcpStream, request: &str) -> Result<(), String> {
    stream
        .write_all(request.as_bytes())
        .map_err(|error| format!("relay request write failed: {error}"))?;
    stream
        .flush()
        .map_err(|error| format!("relay request flush failed: {error}"))
}

fn read_http_response(mut stream: TcpStream) -> Result<String, String> {
    let mut response_bytes = Vec::new();
    let mut buffer = [0_u8; 512];
    loop {
        let read_count = stream
            .read(&mut buffer)
            .map_err(|error| format!("relay response read failed: {error}"))?;
        if read_count == 0 {
            break;
        }
        response_bytes.extend_from_slice(&buffer[..read_count]);
        if response_bytes
            .windows(4)
            .any(|window| window == b"\r\n\r\n")
        {
            break;
        }
    }
    String::from_utf8(response_bytes)
        .map_err(|error| format!("relay response utf-8 parse failed: {error}"))
}

fn validate_http_status(relay_addr: &str, response: &str) -> Result<(), String> {
    let status_line = response.lines().next().unwrap_or("");
    if status_line.starts_with("HTTP/1.1 200")
        || status_line.starts_with("HTTP/1.1 201")
        || status_line.starts_with("HTTP/1.1 202")
    {
        return Ok(());
    }
    Err(format!(
        "relay request returned non-success status: addr={relay_addr};status={status_line}"
    ))
}
