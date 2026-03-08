#[path = "live_transport_http.rs"]
mod http;

use self::http::parse_http_request;
use super::ExpectedRequest;
use kamn_sdk::{service_signature_for_fields, AgentDid};
use std::collections::BTreeMap;
use std::io::{ErrorKind, Write};
use std::net::{TcpListener, TcpStream};
use std::thread;
use std::time::{Duration, Instant};

const CHAIN_ID: &str = "kamn-sdk-live";
const CHAIN_VERSION: &str = "1";

pub(crate) fn run_contract_server(
    bind_addr: String,
    expected_requests: Vec<ExpectedRequest>,
) -> Result<(), String> {
    let listener =
        TcpListener::bind(bind_addr.as_str()).map_err(|error| format!("bind failed: {error}"))?;
    listener
        .set_nonblocking(true)
        .map_err(|error| format!("nonblocking setup failed: {error}"))?;
    let deadline = Instant::now() + Duration::from_secs(2);
    let mut served = 0_usize;
    while served < expected_requests.len() {
        if Instant::now() > deadline {
            return Err("server timed out before serving request budget".to_owned());
        }
        match listener.accept() {
            Ok((mut stream, _)) => {
                handle_expected_request(&mut stream, &expected_requests[served])?;
                served += 1;
            }
            Err(error) if matches!(error.kind(), ErrorKind::WouldBlock) => {
                thread::sleep(Duration::from_millis(5))
            }
            Err(error) => return Err(format!("accept failed: {error}")),
        }
    }
    Ok(())
}

fn handle_expected_request(
    stream: &mut TcpStream,
    expected: &ExpectedRequest,
) -> Result<(), String> {
    let (method, path, body, headers) = parse_http_request(stream)?;
    validate_request(expected, &method, &path, &body)?;
    validate_headers(expected, &method, &path, &body, &headers)?;
    write_http_response(
        stream,
        expected.response_status,
        expected.response_body.as_str(),
    )
}

fn validate_request(
    expected: &ExpectedRequest,
    method: &str,
    path: &str,
    body: &str,
) -> Result<(), String> {
    if method != expected.method || path != expected.path {
        return Err(format!(
            "unexpected request route: expected {} {}, got {} {}",
            expected.method, expected.path, method, path
        ));
    }
    if body == expected.body {
        return Ok(());
    }
    Err(format!(
        "unexpected request body for {method} {path}: expected `{}`, got `{body}`",
        expected.body
    ))
}

fn validate_headers(
    expected: &ExpectedRequest,
    method: &str,
    path: &str,
    body: &str,
    headers: &BTreeMap<String, String>,
) -> Result<(), String> {
    let sender_did = required_header(headers, "x-kamn-sender-did", "missing sender did header")?;
    if sender_did != expected.sender_did {
        return Err(format!(
            "unexpected sender did for {method} {path}: expected `{}`, got `{sender_did}`",
            expected.sender_did
        ));
    }
    let scope = required_header(headers, "x-kamn-authz-scope", "missing auth scope header")?;
    if scope != expected.scope {
        return Err(format!(
            "unexpected auth scope for {method} {path}: expected `{}`, got `{scope}`",
            expected.scope
        ));
    }
    validate_signature(sender_did, method, path, body, headers)
}

fn required_header<'a>(
    headers: &'a BTreeMap<String, String>,
    name: &str,
    missing_message: &str,
) -> Result<&'a str, String> {
    headers
        .get(name)
        .map(String::as_str)
        .ok_or_else(|| missing_message.to_owned())
}

fn validate_signature(
    sender_did: &str,
    method: &str,
    path: &str,
    body: &str,
    headers: &BTreeMap<String, String>,
) -> Result<(), String> {
    let nonce = required_header(
        headers,
        "x-kamn-request-nonce",
        "missing request nonce header",
    )?
    .parse::<u64>()
    .map_err(|_| "invalid request nonce header".to_owned())?;
    let signature = required_header(
        headers,
        "x-kamn-request-signature",
        "missing request signature header",
    )?;
    let parsed_sender =
        AgentDid::parse(sender_did).map_err(|error| format!("sender did should parse: {error}"))?;
    let expected_signature =
        service_signature_for_fields(&parsed_sender, nonce, CHAIN_ID, CHAIN_VERSION, body)
            .map_err(|error| format!("signature generation failed: {error}"))?;
    if signature == expected_signature {
        return Ok(());
    }
    Err(format!("signature mismatch for {method} {path}"))
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
        .map_err(|error| format!("response write failed: {error}"))
}
