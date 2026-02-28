use kamn_agent_lib::{AgentIdentity, KamnAgentHandle};
use kamn_sdk::{service_verify_signature_with_public_key, AgentDid};
use std::collections::BTreeMap;
use std::ffi::OsString;
use std::io::{ErrorKind, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Mutex, OnceLock, PoisonError};
use std::thread;
use std::time::{Duration, Instant};

const TEST_SERVICE_AUTH_PRIVATE_KEY_HEX: &str =
    "658c3528422eb527b4c108b8f6d1e5f629543c304ea49cf608c67794424291c4";

fn reserve_loopback_addr() -> String {
    let listener = TcpListener::bind("127.0.0.1:0").expect("listener should bind");
    let addr = listener.local_addr().expect("local addr should resolve");
    drop(listener);
    addr.to_string()
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
        202 => "202 Accepted",
        401 => "401 Unauthorized",
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

fn run_chain_context_contract_server(
    bind_addr: String,
    expected_chain_id: &'static str,
    expected_chain_version: &'static str,
) -> Result<(), String> {
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
                let (method, path, body, headers) = parse_http_request(&mut stream)?;
                if method != "POST" || path != "/v1/messages/send" {
                    write_http_response(
                        &mut stream,
                        404,
                        r#"{"error":"not-found","reason_code":"service_api_route_not_found","message":"not found"}"#,
                    )?;
                    return Ok(());
                }

                let did = headers
                    .get("x-kamn-sender-did")
                    .ok_or_else(|| "missing sender did header".to_owned())?;
                let nonce = headers
                    .get("x-kamn-request-nonce")
                    .ok_or_else(|| "missing nonce header".to_owned())?
                    .parse::<u64>()
                    .map_err(|_| "invalid nonce header".to_owned())?;
                let signature = headers
                    .get("x-kamn-request-signature")
                    .ok_or_else(|| "missing signature header".to_owned())?;
                let signer_public_key = headers
                    .get("x-kamn-signer-public-key")
                    .ok_or_else(|| "missing signer public key header".to_owned())?;

                let parsed_did = AgentDid::parse(did.as_str())
                    .map_err(|error| format!("did parse failed: {error}"))?;
                let expected_state_hash =
                    format!("service-api:{expected_chain_id}:{expected_chain_version}");
                let verify_result = service_verify_signature_with_public_key(
                    &parsed_did,
                    nonce,
                    expected_state_hash.as_str(),
                    body.as_str(),
                    signature.as_str(),
                    signer_public_key.as_str(),
                );
                if let Err(error) = verify_result {
                    write_http_response(
                        &mut stream,
                        401,
                        r#"{"error":"unauthorized","reason_code":"service_api_auth_signature_verification_failed","message":"signature verification failed for request envelope"}"#,
                    )?;
                    let _ = error;
                    return Ok(());
                }

                write_http_response(
                    &mut stream,
                    202,
                    r#"{"message_id":"msg-chain-context","status":"created","runtime_mode":"api"}"#,
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

fn env_lock() -> &'static Mutex<()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
}

fn with_env_vars<F>(updates: &[(&str, Option<&str>)], test: F)
where
    F: FnOnce(),
{
    let _guard = env_lock().lock().unwrap_or_else(PoisonError::into_inner);
    let previous = updates
        .iter()
        .map(|(key, _)| ((*key).to_owned(), std::env::var_os(key)))
        .collect::<Vec<(String, Option<OsString>)>>();

    for (key, value) in updates {
        match value {
            Some(value) => {
                // SAFETY: env mutation is serialized with a process-wide mutex.
                unsafe { std::env::set_var(key, value) }
            }
            None => {
                // SAFETY: env mutation is serialized with a process-wide mutex.
                unsafe { std::env::remove_var(key) }
            }
        }
    }

    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(test));
    for (key, value) in previous {
        match value {
            Some(value) => {
                // SAFETY: env mutation is serialized with a process-wide mutex.
                unsafe { std::env::set_var(key, value) }
            }
            None => {
                // SAFETY: env mutation is serialized with a process-wide mutex.
                unsafe { std::env::remove_var(key) }
            }
        }
    }
    if let Err(payload) = result {
        std::panic::resume_unwind(payload);
    }
}

#[test]
fn spec_c01_agent_handle_chain_context_env_override_aligns_service_signature_contract() {
    let bind_addr = reserve_loopback_addr();
    let server_addr = bind_addr.clone();
    let server = thread::spawn(move || {
        run_chain_context_contract_server(server_addr, "kamn-devnet", "v0.1.0")
    });
    thread::sleep(Duration::from_millis(40));

    with_env_vars(
        &[
            ("KAMN_AGENT_CHAIN_ID", Some("kamn-devnet")),
            ("KAMN_AGENT_CHAIN_VERSION", Some("v0.1.0")),
            (
                "KAMN_SERVICE_API_AUTH_PRIVATE_KEY_HEX",
                Some(TEST_SERVICE_AUTH_PRIVATE_KEY_HEX),
            ),
        ],
        || {
            let identity = AgentIdentity::from_agent_name("chain-context-test").expect("identity");
            let handle = KamnAgentHandle::with_identity(
                format!("http://{bind_addr}").as_str(),
                "http://localhost:3000",
                identity,
            )
            .expect("handle should connect");

            let receipt = handle
                .send_message(r#"{"message":"chain-context"}"#)
                .expect("request should satisfy server signature contract");
            assert_eq!(receipt.status, "created");
            assert_eq!(receipt.message_id, "msg-chain-context");
        },
    );

    let server_result = server.join().expect("server thread should join");
    assert!(
        server_result.is_ok(),
        "chain-context contract server should complete without internal errors"
    );
}
