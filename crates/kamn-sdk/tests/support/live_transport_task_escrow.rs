use kamn_sdk::{service_signature_for_fields, AgentDid};
use std::collections::BTreeMap;
use std::io::{ErrorKind, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::thread;
use std::time::{Duration, Instant};

const CHAIN_ID: &str = "kamn-sdk-live";
const CHAIN_VERSION: &str = "1";
const LIVE_CHAIN_ID_ENV: &str = "KAMN_SDK_LIVE_CHAIN_ID";
const LIVE_CHAIN_VERSION_ENV: &str = "KAMN_SDK_LIVE_CHAIN_VERSION";
const LIVE_REQUESTER_DID_ENV: &str = "KAMN_SDK_LIVE_REQUESTER_DID";
const SERVICE_AUTH_PRIVATE_KEY_ENV: &str = "KAMN_SERVICE_API_AUTH_PRIVATE_KEY_HEX";
const TEST_SERVICE_AUTH_PRIVATE_KEY_HEX: &str =
    "658c3528422eb527b4c108b8f6d1e5f629543c304ea49cf608c67794424291c4";

pub(crate) struct ExpectedRequest {
    pub(crate) method: &'static str,
    pub(crate) path: String,
    pub(crate) body: String,
    pub(crate) sender_did: String,
    pub(crate) scope: &'static str,
    pub(crate) response_status: u16,
    pub(crate) response_body: String,
}

pub(crate) fn did(identifier: &str) -> AgentDid {
    AgentDid::parse(format!("kamn:did:agent:{identifier}")).expect("did should parse")
}

pub(crate) fn ensure_live_test_env() {
    std::env::set_var(
        SERVICE_AUTH_PRIVATE_KEY_ENV,
        TEST_SERVICE_AUTH_PRIVATE_KEY_HEX,
    );
    std::env::set_var(LIVE_CHAIN_ID_ENV, CHAIN_ID);
    std::env::set_var(LIVE_CHAIN_VERSION_ENV, CHAIN_VERSION);
    std::env::set_var(LIVE_REQUESTER_DID_ENV, "kamn:did:agent:live-requester");
}

pub(crate) fn reserve_loopback_addr() -> String {
    let listener = TcpListener::bind("127.0.0.1:0").expect("listener should bind");
    let addr = listener.local_addr().expect("local addr should resolve");
    drop(listener);
    addr.to_string()
}

pub(crate) fn wait_for_server_ready() {
    thread::sleep(Duration::from_millis(40));
}

fn parse_http_request(
    stream: &mut TcpStream,
) -> Result<(String, String, String, BTreeMap<String, String>), String> {
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
    let Some((request_head, request_body)) = request_text.split_once("\r\n\r\n") else {
        return Err("request header terminator missing".to_owned());
    };
    let request_line = request_head
        .lines()
        .next()
        .ok_or_else(|| "request line missing".to_owned())?;
    let mut headers = BTreeMap::new();
    for line in request_head.lines().skip(1) {
        let Some((name, value)) = line.split_once(':') else {
            continue;
        };
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
                let expected = &expected_requests[served];
                let (method, path, body, headers) = parse_http_request(&mut stream)?;
                if method != expected.method || path != expected.path {
                    return Err(format!(
                        "unexpected request route: expected {} {}, got {} {}",
                        expected.method, expected.path, method, path
                    ));
                }
                if body != expected.body {
                    return Err(format!(
                        "unexpected request body for {method} {path}: expected `{}`, got `{body}`",
                        expected.body
                    ));
                }
                let sender_did = headers
                    .get("x-kamn-sender-did")
                    .ok_or_else(|| "missing sender did header".to_owned())?;
                if sender_did != &expected.sender_did {
                    return Err(format!(
                        "unexpected sender did for {method} {path}: expected `{}`, got `{sender_did}`",
                        expected.sender_did
                    ));
                }
                let scope = headers
                    .get("x-kamn-authz-scope")
                    .ok_or_else(|| "missing auth scope header".to_owned())?;
                if scope != expected.scope {
                    return Err(format!(
                        "unexpected auth scope for {method} {path}: expected `{}`, got `{scope}`",
                        expected.scope
                    ));
                }
                let nonce = headers
                    .get("x-kamn-request-nonce")
                    .ok_or_else(|| "missing request nonce header".to_owned())?
                    .parse::<u64>()
                    .map_err(|_| "invalid request nonce header".to_owned())?;
                let signature = headers
                    .get("x-kamn-request-signature")
                    .ok_or_else(|| "missing request signature header".to_owned())?;
                let parsed_sender = AgentDid::parse(sender_did.clone())
                    .map_err(|error| format!("sender did should parse: {error}"))?;
                let expected_signature = service_signature_for_fields(
                    &parsed_sender,
                    nonce,
                    CHAIN_ID,
                    CHAIN_VERSION,
                    body.as_str(),
                )
                .map_err(|error| format!("signature generation failed: {error}"))?;
                if signature != &expected_signature {
                    return Err(format!("signature mismatch for {method} {path}"));
                }
                write_http_response(&mut stream, expected.response_status, expected.response_body.as_str())?;
                served += 1;
            }
            Err(error) if matches!(error.kind(), ErrorKind::WouldBlock) => {
                thread::sleep(Duration::from_millis(5));
            }
            Err(error) => return Err(format!("accept failed: {error}")),
        }
    }

    Ok(())
}
