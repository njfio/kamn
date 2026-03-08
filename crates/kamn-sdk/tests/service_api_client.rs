use kamn_sdk::{
    AgentDid, SdkError, ServiceApiClient, ServiceRequestAuth, service_signature_for_fields,
};
use std::collections::{BTreeMap, BTreeSet};
use std::io::{BufRead, BufReader, ErrorKind, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::{Mutex, OnceLock};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use std::{env, fs};

const CHAIN_ID: &str = "kolme-localnet";
const CHAIN_VERSION: &str = "v0";
const REQUEST_AUTH_SCOPE_HEADER: &str = "x-kamn-authz-scope";
const SERVICE_AUTH_PRIVATE_KEY_ENV: &str = "KAMN_SERVICE_API_AUTH_PRIVATE_KEY_HEX";
const SERVICE_TLS_CA_FILE_ENV: &str = "KAMN_SERVICE_API_TLS_CA_FILE";
const TEST_SERVICE_AUTH_PRIVATE_KEY_HEX: &str =
    "658c3528422eb527b4c108b8f6d1e5f629543c304ea49cf608c67794424291c4";

fn ensure_test_service_auth_private_key() {
    static ONCE: OnceLock<()> = OnceLock::new();
    ONCE.get_or_init(|| {
        std::env::set_var(
            SERVICE_AUTH_PRIVATE_KEY_ENV,
            TEST_SERVICE_AUTH_PRIVATE_KEY_HEX,
        );
    });
}

fn reserve_loopback_addr() -> String {
    let listener = TcpListener::bind("127.0.0.1:0").expect("listener should bind");
    let addr = listener.local_addr().expect("local addr should resolve");
    drop(listener);
    addr.to_string()
}

fn tls_env_lock() -> &'static Mutex<()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
}

struct EnvVarGuard {
    key: &'static str,
    previous: Option<String>,
}

impl EnvVarGuard {
    fn set(key: &'static str, value: Option<&str>) -> Self {
        let previous = env::var(key).ok();
        match value {
            Some(value) => env::set_var(key, value),
            None => env::remove_var(key),
        }
        Self { key, previous }
    }
}

impl Drop for EnvVarGuard {
    fn drop(&mut self) {
        if let Some(previous) = self.previous.as_deref() {
            env::set_var(self.key, previous);
        } else {
            env::remove_var(self.key);
        }
    }
}

struct HttpsSingleRequestServer {
    base_url: String,
    ca_cert_path: PathBuf,
    child: Child,
    temp_dir: PathBuf,
}

impl HttpsSingleRequestServer {
    fn wait_for_exit(&mut self) {
        let deadline = Instant::now() + Duration::from_secs(3);
        loop {
            match self.child.try_wait() {
                Ok(Some(_)) => return,
                Ok(None) => {
                    if Instant::now() >= deadline {
                        let _ = self.child.kill();
                        let _ = self.child.wait();
                        panic!("https test server did not exit after handling request");
                    }
                    thread::sleep(Duration::from_millis(10));
                }
                Err(error) => panic!("failed to wait for https test server exit: {error}"),
            }
        }
    }
}

impl Drop for HttpsSingleRequestServer {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
        let _ = fs::remove_dir_all(&self.temp_dir);
    }
}

fn unique_temp_dir(prefix: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock should be after unix epoch")
        .as_nanos();
    let path = env::temp_dir().join(format!("{prefix}-{}-{nanos}", std::process::id()));
    fs::create_dir_all(&path).expect("temporary directory should be created");
    path
}

fn generate_test_ca_signed_certificate_chain(temp_dir: &Path) -> (PathBuf, PathBuf, PathBuf) {
    let ca_cert_path = temp_dir.join("ca-cert.pem");
    let ca_key_path = temp_dir.join("ca-key.pem");
    let server_key_path = temp_dir.join("server-key.pem");
    let server_csr_path = temp_dir.join("server.csr");
    let server_cert_path = temp_dir.join("server-cert.pem");
    let server_extensions_path = temp_dir.join("server-ext.cnf");

    let ca_status = Command::new("openssl")
        .arg("req")
        .arg("-x509")
        .arg("-newkey")
        .arg("rsa:2048")
        .arg("-keyout")
        .arg(ca_key_path.as_os_str())
        .arg("-out")
        .arg(ca_cert_path.as_os_str())
        .arg("-days")
        .arg("1")
        .arg("-nodes")
        .arg("-subj")
        .arg("/CN=kamn-test-ca")
        .arg("-addext")
        .arg("basicConstraints = critical,CA:TRUE")
        .arg("-addext")
        .arg("keyUsage = critical,keyCertSign,cRLSign")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .expect("openssl should run for CA certificate generation");
    assert!(
        ca_status.success(),
        "openssl CA certificate generation should succeed"
    );

    let csr_status = Command::new("openssl")
        .arg("req")
        .arg("-new")
        .arg("-newkey")
        .arg("rsa:2048")
        .arg("-keyout")
        .arg(server_key_path.as_os_str())
        .arg("-out")
        .arg(server_csr_path.as_os_str())
        .arg("-nodes")
        .arg("-subj")
        .arg("/CN=127.0.0.1")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .expect("openssl should run for server csr generation");
    assert!(
        csr_status.success(),
        "openssl server csr generation should succeed"
    );

    fs::write(
        server_extensions_path.as_path(),
        "subjectAltName = DNS:localhost,IP:127.0.0.1\nbasicConstraints = critical,CA:FALSE\nkeyUsage = critical,digitalSignature,keyEncipherment\nextendedKeyUsage = serverAuth\n",
    )
    .expect("server extension file should be written");

    let sign_status = Command::new("openssl")
        .arg("x509")
        .arg("-req")
        .arg("-in")
        .arg(server_csr_path.as_os_str())
        .arg("-CA")
        .arg(ca_cert_path.as_os_str())
        .arg("-CAkey")
        .arg(ca_key_path.as_os_str())
        .arg("-CAcreateserial")
        .arg("-out")
        .arg(server_cert_path.as_os_str())
        .arg("-days")
        .arg("1")
        .arg("-sha256")
        .arg("-extfile")
        .arg(server_extensions_path.as_os_str())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .expect("openssl should run for server certificate signing");
    assert!(
        sign_status.success(),
        "openssl server certificate signing should succeed"
    );

    (ca_cert_path, server_cert_path, server_key_path)
}

fn spawn_https_single_request_server(
    status_code: u16,
    response_body: &str,
) -> HttpsSingleRequestServer {
    let temp_dir = unique_temp_dir("sdk-https-server");
    let (ca_cert_path, server_cert_path, server_key_path) =
        generate_test_ca_signed_certificate_chain(temp_dir.as_path());
    let server_script = r#"
import http.server
import ssl
import sys

port = int(sys.argv[1])
cert_file = sys.argv[2]
key_file = sys.argv[3]
status_code = int(sys.argv[4])
response_body = sys.argv[5].encode("utf-8")

class Handler(http.server.BaseHTTPRequestHandler):
    def _reply(self):
        if "Content-Length" in self.headers:
            _ = self.rfile.read(int(self.headers["Content-Length"]))
        self.send_response(status_code)
        self.send_header("Content-Type", "application/json")
        self.send_header("Content-Length", str(len(response_body)))
        self.send_header("Connection", "close")
        self.end_headers()
        self.wfile.write(response_body)

    def do_POST(self):
        self._reply()

    def do_GET(self):
        self._reply()

    def log_message(self, _format, *args):
        return

httpd = http.server.HTTPServer(("127.0.0.1", port), Handler)
context = ssl.SSLContext(ssl.PROTOCOL_TLS_SERVER)
context.load_cert_chain(certfile=cert_file, keyfile=key_file)
httpd.socket = context.wrap_socket(httpd.socket, server_side=True)
print(httpd.server_address[1], flush=True)
httpd.handle_request()
"#;

    let mut child = Command::new("python3")
        .arg("-u")
        .arg("-c")
        .arg(server_script)
        .arg("0")
        .arg(server_cert_path.as_os_str())
        .arg(server_key_path.as_os_str())
        .arg(status_code.to_string())
        .arg(response_body)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("python https test server should spawn");

    let stdout = child
        .stdout
        .take()
        .expect("python https test server stdout should be piped");
    let mut stdout_reader = BufReader::new(stdout);
    let mut port_line = String::new();
    stdout_reader
        .read_line(&mut port_line)
        .expect("python https test server should emit bound port");
    child.stdout = Some(stdout_reader.into_inner());

    let port = port_line
        .trim()
        .parse::<u16>()
        .expect("python https test server should emit a valid port");
    HttpsSingleRequestServer {
        base_url: format!("https://127.0.0.1:{port}"),
        ca_cert_path,
        child,
        temp_dir,
    }
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
        200 => "200 OK",
        201 => "201 Created",
        202 => "202 Accepted",
        400 => "400 Bad Request",
        401 => "401 Unauthorized",
        404 => "404 Not Found",
        409 => "409 Conflict",
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

const DEFAULT_WEBSOCKET_EVENT_PAYLOAD: &str =
    r#"{"event":"state-transition","runtime_mode":"api","role":"processor","sequence":1}"#;

fn write_websocket_upgrade_response(stream: &mut TcpStream, payload: &str) -> Result<(), String> {
    let handshake = "HTTP/1.1 101 Switching Protocols\r\nUpgrade: websocket\r\nConnection: Upgrade\r\nSec-WebSocket-Accept: kamn-test-accept\r\nX-KAMN-WebSocket-Contract: v1\r\n\r\n";
    stream
        .write_all(handshake.as_bytes())
        .map_err(|error| format!("websocket handshake write failed: {error}"))?;
    let payload = payload.as_bytes();
    let mut frame = Vec::with_capacity(10 + payload.len());
    frame.push(0x81);
    if payload.len() <= 125 {
        frame.push(payload.len() as u8);
    } else if payload.len() <= u16::MAX as usize {
        frame.push(126);
        frame.extend_from_slice((payload.len() as u16).to_be_bytes().as_slice());
    } else {
        frame.push(127);
        frame.extend_from_slice((payload.len() as u64).to_be_bytes().as_slice());
    }
    frame.extend_from_slice(payload);
    stream
        .write_all(frame.as_slice())
        .map_err(|error| format!("websocket frame write failed: {error}"))
}

fn validate_auth(
    method: &str,
    path: &str,
    body: &str,
    headers: &BTreeMap<String, String>,
    replay_guard: &mut BTreeSet<(String, u64)>,
) -> Result<(), (u16, &'static str, &'static str, &'static str)> {
    ensure_test_service_auth_private_key();
    let did = headers
        .get("x-kamn-sender-did")
        .ok_or((
            401,
            "unauthorized",
            "service_api_auth_sender_did_header_missing",
            "missing required header: x-kamn-sender-did",
        ))?
        .to_owned();
    AgentDid::parse(did.clone()).map_err(|_| {
        (
            401,
            "unauthorized",
            "service_api_auth_sender_did_invalid",
            "invalid sender did",
        )
    })?;
    let nonce = headers
        .get("x-kamn-request-nonce")
        .ok_or((
            401,
            "unauthorized",
            "service_api_auth_nonce_header_missing",
            "missing required header: x-kamn-request-nonce",
        ))?
        .parse::<u64>()
        .map_err(|_| {
            (
                401,
                "unauthorized",
                "service_api_auth_nonce_invalid",
                "invalid request nonce header: x-kamn-request-nonce",
            )
        })?;
    if nonce == 0 {
        return Err((
            401,
            "unauthorized",
            "service_api_auth_nonce_non_positive",
            "request nonce must be positive: x-kamn-request-nonce",
        ));
    }
    let signature = headers.get("x-kamn-request-signature").ok_or((
        401,
        "unauthorized",
        "service_api_auth_signature_header_missing",
        "missing required header: x-kamn-request-signature",
    ))?;
    let expected = service_signature_for_fields(
        &AgentDid::parse(did.clone()).map_err(|_| {
            (
                401,
                "unauthorized",
                "service_api_auth_sender_did_invalid",
                "invalid sender did",
            )
        })?,
        nonce,
        CHAIN_ID,
        CHAIN_VERSION,
        body,
    )
    .map_err(|_| {
        (
            401,
            "unauthorized",
            "service_api_auth_signature_verification_failed",
            "signature verification failed for request envelope",
        )
    })?;
    if &expected != signature {
        return Err((
            401,
            "unauthorized",
            "service_api_auth_signature_verification_failed",
            "signature verification failed for request envelope",
        ));
    }
    if !replay_guard.insert((did, nonce)) {
        return Err((
            409,
            "replay",
            "service_api_auth_replay_nonce_detected",
            "request nonce replay detected for sender",
        ));
    }

    if let Some(expected_scope) = required_scope_for_route(method, path) {
        let scope = headers.get(REQUEST_AUTH_SCOPE_HEADER).ok_or((
            401,
            "unauthorized",
            "service_api_auth_scope_header_missing",
            "missing required header: x-kamn-authz-scope",
        ))?;
        if scope != expected_scope {
            return Err((
                401,
                "unauthorized",
                "service_api_auth_scope_route_mismatch",
                "scope route mismatch",
            ));
        }
    }
    Ok(())
}

fn required_scope_for_route(method: &str, path: &str) -> Option<&'static str> {
    if !route_requires_auth(method, path) {
        return None;
    }

    Some(match (method, path) {
        ("POST", "/v1/messages/send") => "messages:write",
        ("POST", "/v1/channels/create") => "channels:write",
        ("POST", "/v1/agents/register") => "agents:write",
        ("POST", "/v1/agents/search") => "agents:read",
        ("POST", "/v1/tasks/create") => "tasks:write",
        ("POST", _) if path.starts_with("/v1/tasks/") && path.ends_with("/accept") => "tasks:write",
        ("POST", _) if path.starts_with("/v1/tasks/") && path.ends_with("/complete") => {
            "tasks:write"
        }
        ("POST", "/v1/escrow/fund") => "escrow:write",
        ("POST", _) if path.starts_with("/v1/escrow/") && path.ends_with("/release") => {
            "escrow:write"
        }
        ("POST", "/v1/bridge/submit") => "bridge:write",
        ("POST", _) if path.starts_with("/v1/bridge/") && path.ends_with("/forward") => {
            "bridge:write"
        }
        ("GET", _) if path.starts_with("/v1/bridge/") && path != "/v1/bridge/submit" => {
            "bridge:read"
        }
        ("GET", "/v1/events/ws") => "events:read",
        ("GET", _) if path.starts_with("/v1/messages/") => "messages:read",
        ("GET", _) if path.starts_with("/v1/channels/") && path.ends_with("/messages") => {
            "channels:read"
        }
        ("GET", _) if path.starts_with("/v1/tasks/") && path != "/v1/tasks/create" => "tasks:read",
        ("GET", _) if path.starts_with("/v1/agents/") => "agents:read",
        _ => "protected:unknown",
    })
}

fn route_requires_auth(method: &str, path: &str) -> bool {
    !(method == "GET" && (path == "/healthz" || path == "/metrics"))
}

fn run_service_contract_server(bind_addr: String, max_requests: u64) -> Result<(), String> {
    run_service_contract_server_with_websocket_payload(
        bind_addr,
        max_requests,
        DEFAULT_WEBSOCKET_EVENT_PAYLOAD.to_owned(),
    )
}

fn run_service_contract_server_with_websocket_payload(
    bind_addr: String,
    max_requests: u64,
    websocket_payload: String,
) -> Result<(), String> {
    let listener = TcpListener::bind(bind_addr.as_str())
        .map_err(|error| format!("server bind failed: {error}"))?;
    listener
        .set_nonblocking(true)
        .map_err(|error| format!("server nonblocking mode failed: {error}"))?;

    let deadline = Instant::now() + Duration::from_secs(2);
    let mut served = 0_u64;
    let mut replay_guard: BTreeSet<(String, u64)> = BTreeSet::new();
    while served < max_requests {
        if Instant::now() > deadline {
            return Err("server timed out before serving request budget".to_owned());
        }
        match listener.accept() {
            Ok((mut stream, _)) => {
                let (method, path, body, headers) = parse_http_request(&mut stream)?;
                if method == "GET" && path == "/healthz" {
                    write_http_response(
                        &mut stream,
                        200,
                        r#"{"status":"ok","runtime_mode":"api","role":"processor","observability_source":"unknown","observability_health":"unknown"}"#,
                    )?;
                    served = served.saturating_add(1);
                    continue;
                }
                if method == "GET" && path == "/metrics" {
                    let body = "kamn_service_api_health{runtime_mode=\"api\"} 1\n";
                    let response = format!(
                        "HTTP/1.1 200 OK\r\nContent-Type: text/plain; version=0.0.4\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                        body.len(),
                        body
                    );
                    stream
                        .write_all(response.as_bytes())
                        .map_err(|error| format!("metrics write failed: {error}"))?;
                    served = served.saturating_add(1);
                    continue;
                }

                if let Err((status, error, reason_code, message)) = validate_auth(
                    method.as_str(),
                    path.as_str(),
                    &body,
                    &headers,
                    &mut replay_guard,
                ) {
                    let payload = format!(
                        "{{\"error\":\"{error}\",\"reason_code\":\"{reason_code}\",\"message\":\"{message}\"}}",
                    );
                    write_http_response(&mut stream, status, payload.as_str())?;
                    served = served.saturating_add(1);
                    continue;
                }

                if method == "GET" && path == "/v1/events/ws" {
                    let upgrade = headers.get("upgrade").cloned().unwrap_or_default();
                    let connection = headers.get("connection").cloned().unwrap_or_default();
                    let websocket_key = headers
                        .get("sec-websocket-key")
                        .cloned()
                        .unwrap_or_default();
                    let version = headers
                        .get("sec-websocket-version")
                        .cloned()
                        .unwrap_or_default();
                    if !upgrade.eq_ignore_ascii_case("websocket")
                        || !connection.to_ascii_lowercase().contains("upgrade")
                        || websocket_key.trim().is_empty()
                        || version.trim() != "13"
                    {
                        write_http_response(
                            &mut stream,
                            400,
                            r#"{"error":"bad-request","reason_code":"service_api_websocket_upgrade_required","message":"websocket upgrade required"}"#,
                        )?;
                    } else {
                        write_websocket_upgrade_response(&mut stream, websocket_payload.as_str())?;
                    }
                    served = served.saturating_add(1);
                    continue;
                }

                if method == "POST" && path == "/v1/messages/send" {
                    let message_id =
                        format!("msg-local-{:016x}", deterministic_tag(body.as_bytes()));
                    let payload = format!(
                        "{{\"message_id\":\"{}\",\"status\":\"created\",\"runtime_mode\":\"api\"}}",
                        message_id
                    );
                    write_http_response(&mut stream, 202, payload.as_str())?;
                } else if method == "GET" && path.starts_with("/v1/messages/") {
                    let message_id = path.trim_start_matches("/v1/messages/");
                    let payload = format!(
                        "{{\"message_id\":\"{}\",\"status\":\"created\"}}",
                        message_id
                    );
                    write_http_response(&mut stream, 200, payload.as_str())?;
                } else if method == "POST" && path == "/v1/channels/create" {
                    let channel_id =
                        format!("channel-local-{:016x}", deterministic_tag(body.as_bytes()));
                    let payload = format!(
                        "{{\"channel_id\":\"{}\",\"status\":\"created\"}}",
                        channel_id
                    );
                    write_http_response(&mut stream, 201, payload.as_str())?;
                } else if method == "GET"
                    && path.starts_with("/v1/channels/")
                    && path.ends_with("/messages")
                {
                    let channel_id = path
                        .trim_start_matches("/v1/channels/")
                        .trim_end_matches("/messages")
                        .trim_end_matches('/');
                    let payload = format!(
                        "{{\"channel_id\":\"{}\",\"messages\":[\"msg-local-a\",\"msg-local-b\"]}}",
                        channel_id
                    );
                    write_http_response(&mut stream, 200, payload.as_str())?;
                } else if method == "POST" && path == "/v1/tasks/create" {
                    let task_id = format!("task-local-{:016x}", deterministic_tag(body.as_bytes()));
                    let payload =
                        format!("{{\"task_id\":\"{}\",\"state\":\"submitted\"}}", task_id);
                    write_http_response(&mut stream, 201, payload.as_str())?;
                } else if method == "POST"
                    && path.starts_with("/v1/tasks/")
                    && path.ends_with("/accept")
                {
                    let task_id = path
                        .trim_start_matches("/v1/tasks/")
                        .trim_end_matches("/accept")
                        .trim_end_matches('/');
                    let payload = format!("{{\"task_id\":\"{}\",\"state\":\"accepted\"}}", task_id);
                    write_http_response(&mut stream, 200, payload.as_str())?;
                } else if method == "POST"
                    && path.starts_with("/v1/tasks/")
                    && path.ends_with("/complete")
                {
                    let task_id = path
                        .trim_start_matches("/v1/tasks/")
                        .trim_end_matches("/complete")
                        .trim_end_matches('/');
                    let payload =
                        format!("{{\"task_id\":\"{}\",\"state\":\"completed\"}}", task_id);
                    write_http_response(&mut stream, 200, payload.as_str())?;
                } else if method == "POST" && path == "/v1/escrow/fund" {
                    let escrow_id =
                        format!("escrow-local-{:016x}", deterministic_tag(body.as_bytes()));
                    let payload =
                        format!("{{\"escrow_id\":\"{}\",\"state\":\"funded\"}}", escrow_id);
                    write_http_response(&mut stream, 200, payload.as_str())?;
                } else if method == "POST" && path == "/v1/bridge/submit" {
                    let bridge_tag = deterministic_tag(body.as_bytes());
                    let payload = format!(
                        "{{\"bridge_id\":\"bridge-local-{bridge_tag:016x}\",\"source_message_id\":\"msg-bridge-source-{bridge_tag:016x}\",\"bridge_status\":\"submitted\"}}"
                    );
                    write_http_response(&mut stream, 202, payload.as_str())?;
                } else if method == "POST"
                    && path.starts_with("/v1/bridge/")
                    && path.ends_with("/forward")
                {
                    let bridge_id = path
                        .trim_start_matches("/v1/bridge/")
                        .trim_end_matches("/forward")
                        .trim_end_matches('/');
                    let payload = format!(
                        "{{\"bridge_id\":\"{}\",\"bridge_status\":\"forwarded\",\"target_message_id\":\"msg-bridge-target-{}\",\"forward_tx_hash\":\"sha256:bridge-forwarded-{}\"}}",
                        bridge_id, bridge_id, bridge_id
                    );
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
                        format!("{{\"escrow_id\":\"{}\",\"state\":\"released\"}}", escrow_id);
                    write_http_response(&mut stream, 200, payload.as_str())?;
                } else if method == "GET" && path.starts_with("/v1/tasks/") {
                    let task_id = path.trim_start_matches("/v1/tasks/");
                    let payload =
                        format!("{{\"task_id\":\"{}\",\"state\":\"submitted\"}}", task_id);
                    write_http_response(&mut stream, 200, payload.as_str())?;
                } else if method == "POST" && path == "/v1/agents/register" {
                    let payload = r#"{"did":"kamn:did:agent:sdk-register","reputation_score":500,"agent_type":"assistant","model_family":"gpt-5","capabilities":["text","code"]}"#;
                    write_http_response(&mut stream, 201, payload)?;
                } else if method == "POST" && path == "/v1/agents/search" {
                    let payload = r#"[{"did":"kamn:did:agent:sdk-register","reputation_score":500,"agent_type":"assistant","model_family":"gpt-5","capabilities":["text","code"]}]"#;
                    write_http_response(&mut stream, 200, payload)?;
                } else if method == "GET" && path.starts_with("/v1/agents/") {
                    let did = path.trim_start_matches("/v1/agents/");
                    let payload = format!(
                        "{{\"did\":\"{}\",\"reputation_score\":500,\"agent_type\":\"service-agent\",\"model_family\":\"service-api\",\"capabilities\":[\"profile:read\"]}}",
                        did
                    );
                    write_http_response(&mut stream, 200, payload.as_str())?;
                } else if method == "GET"
                    && path.starts_with("/v1/bridge/")
                    && path != "/v1/bridge/submit"
                {
                    let bridge_id = path.trim_start_matches("/v1/bridge/");
                    let payload = format!(
                        "{{\"bridge_id\":\"{}\",\"bridge_status\":\"forwarded\",\"target_message_id\":\"msg-bridge-target-{}\",\"forward_tx_hash\":\"sha256:bridge-forwarded-{}\"}}",
                        bridge_id, bridge_id, bridge_id
                    );
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

fn wait_for_server_ready(addr: &str) {
    assert!(!addr.trim().is_empty(), "server address must not be empty");
    thread::sleep(Duration::from_millis(40));
}

fn auth_with_scope(sender: &AgentDid, nonce: u64, body: &str, scope: &str) -> ServiceRequestAuth {
    ensure_test_service_auth_private_key();
    ServiceRequestAuth::new_with_scope(
        sender.clone(),
        nonce,
        service_signature_for_fields(sender, nonce, CHAIN_ID, CHAIN_VERSION, body)
            .expect("service signature should build"),
        Some(scope),
    )
    .expect("request auth with scope should build")
}

#[test]
fn unit_service_api_client_rejects_invalid_endpoint_scheme() {
    assert_eq!(
        ServiceApiClient::connect("tcp://127.0.0.1:35001"),
        Err(SdkError::InvalidInput {
            field: "service.endpoint",
            reason: "must start with http:// or https://",
        })
    );
}

#[test]
fn spec_c01_service_api_client_executes_https_health_route_with_trusted_ca() {
    let _tls_env_lock = tls_env_lock()
        .lock()
        .expect("tls env lock should not be poisoned");
    let mut server = spawn_https_single_request_server(
        200,
        r#"{"status":"ok","runtime_mode":"api","role":"processor","observability_source":"unknown","observability_health":"unknown"}"#,
    );
    let ca_file = server.ca_cert_path.to_string_lossy().to_string();
    let _ca_guard = EnvVarGuard::set(SERVICE_TLS_CA_FILE_ENV, Some(ca_file.as_str()));

    let client =
        ServiceApiClient::connect(server.base_url.as_str()).expect("https client should construct");
    let health = client
        .health()
        .expect("trusted CA should allow https service route request");
    assert_eq!(health.status, "ok");
    assert_eq!(health.runtime_mode, "api");

    server.wait_for_exit();
}

#[test]
fn spec_c02_service_api_client_rejects_untrusted_https_certificate_chain() {
    let _tls_env_lock = tls_env_lock()
        .lock()
        .expect("tls env lock should not be poisoned");
    let server = spawn_https_single_request_server(
        200,
        r#"{"status":"ok","runtime_mode":"api","role":"processor","observability_source":"unknown","observability_health":"unknown"}"#,
    );
    let _ca_guard = EnvVarGuard::set(SERVICE_TLS_CA_FILE_ENV, None);

    let client =
        ServiceApiClient::connect(server.base_url.as_str()).expect("https client should construct");
    let error = client
        .health()
        .expect_err("untrusted cert chain must fail closed");
    assert_eq!(
        error,
        SdkError::TransportFailure("service tls certificate verification failed")
    );
}

#[test]
fn spec_c02_service_api_client_rejects_missing_tls_ca_bundle_path() {
    let _tls_env_lock = tls_env_lock()
        .lock()
        .expect("tls env lock should not be poisoned");
    let server = spawn_https_single_request_server(
        200,
        r#"{"status":"ok","runtime_mode":"api","role":"processor","observability_source":"unknown","observability_health":"unknown"}"#,
    );
    let missing_ca_file = server
        .temp_dir
        .join("missing-ca.pem")
        .to_string_lossy()
        .to_string();
    let _ca_guard = EnvVarGuard::set(SERVICE_TLS_CA_FILE_ENV, Some(missing_ca_file.as_str()));

    let client =
        ServiceApiClient::connect(server.base_url.as_str()).expect("https client should construct");
    let error = client
        .health()
        .expect_err("missing TLS CA bundle path must fail closed");
    assert_eq!(
        error,
        SdkError::TransportFailure("service tls ca file read failed")
    );
}

#[test]
fn regression_service_api_client_rejects_crlf_route_identifier_payload() {
    // Regression: #5929
    ensure_test_service_auth_private_key();
    let client = ServiceApiClient::connect("http://127.0.0.1:1").expect("client should construct");
    let sender = AgentDid::parse("kamn:did:agent:sdk-route-injection").expect("did");
    let auth = auth_with_scope(&sender, 1, "", "messages:read");

    let error = client
        .get_message("msg-1\r\nx-injected-header: true", &auth)
        .expect_err("crlf payload must fail closed before request emission");
    assert_eq!(
        error,
        SdkError::InvalidInput {
            field: "message_id",
            reason: "contains characters not allowed in route segment",
        }
    );
}

#[test]
fn regression_service_request_auth_rejects_crlf_signature_payload() {
    // Regression: #5929
    let sender = AgentDid::parse("kamn:did:agent:sdk-header-injection").expect("did");
    let error = ServiceRequestAuth::new_with_scope(
        sender,
        1,
        "sig\r\nx-injected-header: true".to_owned(),
        Some("messages:write"),
    )
    .expect_err("signature header injection payload must fail closed");
    assert_eq!(
        error,
        SdkError::InvalidInput {
            field: "request_auth.signature",
            reason: "contains invalid http header characters",
        }
    );
}

#[test]
fn regression_service_api_client_rejects_legacy_agent_profile_did() {
    // Regression: #6514
    ensure_test_service_auth_private_key();
    let client = ServiceApiClient::connect("http://127.0.0.1:1").expect("client should construct");
    let sender = AgentDid::parse("kamn:did:agent:sdk-legacy-agent-profile").expect("did");
    let auth = auth_with_scope(&sender, 1, "", "agents:read");

    let error = client
        .get_agent_profile("did:kamn:agent:alice", &auth)
        .expect_err("legacy did must fail closed before request emission");
    assert_eq!(
        error,
        SdkError::InvalidInput {
            field: "did",
            reason: "must start with kamn:did:agent:",
        }
    );
}

#[test]
fn regression_service_api_client_rejects_crlf_agent_did_route_payload() {
    // Regression: #6228
    ensure_test_service_auth_private_key();
    let client = ServiceApiClient::connect("http://127.0.0.1:1").expect("client should construct");
    let sender = AgentDid::parse("kamn:did:agent:sdk-did-route-injection").expect("did");
    let auth = auth_with_scope(&sender, 1, "", "agents:read");

    let error = client
        .get_agent_profile("kamn:did:agent:alice\r\nx-injected-header: true", &auth)
        .expect_err("crlf did payload must fail closed before request emission");
    assert_eq!(
        error,
        SdkError::InvalidInput {
            field: "did",
            reason: "contains characters not allowed in route segment",
        }
    );
}

#[test]
fn regression_service_request_auth_rejects_crlf_scope_payload() {
    // Regression: #6228
    let sender = AgentDid::parse("kamn:did:agent:sdk-scope-injection").expect("did");
    let error = ServiceRequestAuth::new_with_scope(
        sender,
        1,
        "sig:ok".to_owned(),
        Some("messages:read\r\nx-injected-header: true"),
    )
    .expect_err("scope header injection payload must fail closed");
    assert_eq!(
        error,
        SdkError::InvalidInput {
            field: "request_auth.scope",
            reason: "contains invalid http header characters",
        }
    );
}

#[test]
fn functional_service_api_client_executes_signed_http_route_contracts() {
    let bind_addr = reserve_loopback_addr();
    let server_addr = bind_addr.clone();
    let server = thread::spawn(move || run_service_contract_server(server_addr, 10));
    wait_for_server_ready(bind_addr.as_str());

    let client = ServiceApiClient::connect(format!("http://{bind_addr}").as_str())
        .expect("client should connect");
    let sender = AgentDid::parse("kamn:did:agent:sdk-client").expect("sender did should parse");

    let send_payload = r#"{"message":"hello from sdk"}"#;
    let send_response = client
        .send_message(
            send_payload,
            &auth_with_scope(&sender, 1, send_payload, "messages:write"),
        )
        .expect("send message should succeed");
    assert!(send_response.message_id.starts_with("msg-local-"));
    assert_eq!(send_response.status, "created");

    let message_status = client
        .get_message(
            send_response.message_id.as_str(),
            &auth_with_scope(&sender, 2, "", "messages:read"),
        )
        .expect("get message should succeed");
    assert_eq!(message_status.message_id, send_response.message_id);
    assert_eq!(message_status.status, "created");

    let channel_payload = r#"{"name":"ops"}"#;
    let channel_response = client
        .create_channel(
            channel_payload,
            &auth_with_scope(&sender, 3, channel_payload, "channels:write"),
        )
        .expect("create channel should succeed");
    assert!(channel_response.channel_id.starts_with("channel-local-"));

    let task_payload = r#"{"task":"triage"}"#;
    let task_response = client
        .create_task(
            task_payload,
            &auth_with_scope(&sender, 4, task_payload, "tasks:write"),
        )
        .expect("create task should succeed");
    assert!(task_response.task_id.starts_with("task-local-"));

    let task_status = client
        .get_task(
            task_response.task_id.as_str(),
            &auth_with_scope(&sender, 5, "", "tasks:read"),
        )
        .expect("get task should succeed");
    assert_eq!(task_status.state, "submitted");

    let profile = client
        .get_agent_profile(
            sender.as_str(),
            &auth_with_scope(&sender, 6, "", "agents:read"),
        )
        .expect("agent profile should resolve");
    assert_eq!(profile.did, sender.as_str());
    assert_eq!(profile.reputation_score, 500);
    assert_eq!(profile.agent_type, "service-agent");
    assert_eq!(profile.model_family, "service-api");
    assert_eq!(profile.capabilities, vec!["profile:read".to_owned()]);

    let registration_metadata = kamn_sdk::AgentMetadata {
        agent_type: "assistant".to_owned(),
        model_family: "gpt-5".to_owned(),
        capabilities: vec!["text".to_owned(), "code".to_owned()],
    };
    let registration_payload = serde_json::json!({
        "agent_type": registration_metadata.agent_type,
        "model_family": registration_metadata.model_family,
        "capabilities": registration_metadata.capabilities,
    })
    .to_string();
    let registration = client
        .register_agent(
            &registration_metadata,
            &auth_with_scope(&sender, 7, registration_payload.as_str(), "agents:write"),
        )
        .expect("agent registration should succeed");
    assert_eq!(registration.did, "kamn:did:agent:sdk-register");
    assert_eq!(registration.agent_type, "assistant");
    assert_eq!(registration.model_family, "gpt-5");
    assert_eq!(
        registration.capabilities,
        vec!["text".to_owned(), "code".to_owned()]
    );

    let search_payload = r#"{"capability":"code","model_family":"gpt-5"}"#;
    let search_results = client
        .search_agents(
            &kamn_sdk::AgentQuery {
                capability: Some("code".to_owned()),
                model_family: Some("gpt-5".to_owned()),
            },
            &auth_with_scope(&sender, 8, search_payload, "agents:read"),
        )
        .expect("agent search should succeed");
    assert_eq!(search_results.len(), 1);
    assert_eq!(search_results[0].did, "kamn:did:agent:sdk-register");
    assert_eq!(search_results[0].agent_type, "assistant");
    assert_eq!(search_results[0].model_family, "gpt-5");
    assert_eq!(
        search_results[0].capabilities,
        vec!["text".to_owned(), "code".to_owned()]
    );

    let health = client.health().expect("health route should succeed");
    assert_eq!(health.status, "ok");
    assert_eq!(health.runtime_mode, "api");

    let metrics = client.metrics().expect("metrics route should succeed");
    assert!(
        metrics.contains("kamn_service_api_health{runtime_mode=\"api\"} 1"),
        "metrics contract should include service health gauge"
    );

    let server_result = server.join().expect("server thread should join");
    assert!(
        server_result.is_ok(),
        "test service contract server should satisfy request budget"
    );
}

#[test]
fn integration_service_api_client_reads_websocket_event_frame() {
    let bind_addr = reserve_loopback_addr();
    let server_addr = bind_addr.clone();
    let server = thread::spawn(move || run_service_contract_server(server_addr, 1));
    wait_for_server_ready(bind_addr.as_str());

    let client = ServiceApiClient::connect(format!("http://{bind_addr}").as_str())
        .expect("client should connect");
    let sender = AgentDid::parse("kamn:did:agent:sdk-events").expect("sender did should parse");

    let event = client
        .read_event_once(&auth_with_scope(&sender, 9, "", "events:read"))
        .expect("event read should succeed");
    assert_eq!(event.event, "state-transition");
    assert_eq!(event.runtime_mode, "api");
    assert_eq!(event.role, "processor");
    assert_eq!(event.sequence, 1);

    let server_result = server.join().expect("server thread should join");
    assert!(
        server_result.is_ok(),
        "test service contract server should satisfy websocket request budget"
    );
}

#[test]
fn integration_service_api_client_reads_websocket_event_frame_extended_length() {
    // Regression: #6111
    let bind_addr = reserve_loopback_addr();
    let server_addr = bind_addr.clone();
    let note = "x".repeat(200);
    let websocket_payload = format!(
        "{{\"event\":\"state-transition\",\"runtime_mode\":\"api\",\"role\":\"processor\",\"sequence\":1,\"note\":\"{note}\"}}"
    );
    let server = thread::spawn(move || {
        run_service_contract_server_with_websocket_payload(server_addr, 1, websocket_payload)
    });
    wait_for_server_ready(bind_addr.as_str());

    let client = ServiceApiClient::connect(format!("http://{bind_addr}").as_str())
        .expect("client should connect");
    let sender =
        AgentDid::parse("kamn:did:agent:sdk-events-extended").expect("sender did should parse");

    let event = client
        .read_event_once(&auth_with_scope(&sender, 10, "", "events:read"))
        .expect("event read should succeed");
    assert_eq!(event.event, "state-transition");
    assert_eq!(event.runtime_mode, "api");
    assert_eq!(event.role, "processor");
    assert_eq!(event.sequence, 1);

    let server_result = server.join().expect("server thread should join");
    assert!(
        server_result.is_ok(),
        "test service contract server should satisfy websocket request budget"
    );
}

#[test]
fn regression_service_api_client_rejects_replayed_nonce() {
    // Regression: #2946
    let bind_addr = reserve_loopback_addr();
    let server_addr = bind_addr.clone();
    let server = thread::spawn(move || run_service_contract_server(server_addr, 3));
    wait_for_server_ready(bind_addr.as_str());

    let client = ServiceApiClient::connect(format!("http://{bind_addr}").as_str())
        .expect("client should connect");
    let sender = AgentDid::parse("kamn:did:agent:sdk-replay").expect("sender did should parse");
    let payload = r#"{"message":"nonce replay"}"#;
    let replay_auth = auth_with_scope(&sender, 11, payload, "messages:write");

    client
        .send_message(payload, &replay_auth)
        .expect("first send should pass");
    let replay_error = client
        .send_message(payload, &replay_auth)
        .expect_err("replayed nonce should fail closed");
    assert!(
        replay_error
            .to_string()
            .contains("reason_code=service_api_auth_replay_nonce_detected"),
        "replay failure should expose deterministic reason code: {replay_error}"
    );

    let invalid_auth = auth_with_scope(
        &sender,
        12,
        r#"{"message":"mismatch-signature"}"#,
        "messages:write",
    );
    let unauthorized_error = client
        .send_message(payload, &invalid_auth)
        .expect_err("signature mismatch should fail closed");
    assert!(
        unauthorized_error
            .to_string()
            .contains("reason_code=service_api_auth_signature_verification_failed"),
        "unauthorized failure should expose deterministic reason code: {unauthorized_error}"
    );

    let server_result = server.join().expect("server thread should join");
    assert!(
        server_result.is_ok(),
        "test service contract server should satisfy replay request budget"
    );
}

#[test]
fn spec_c01_service_api_client_lists_channel_messages_through_route_contract() {
    let bind_addr = reserve_loopback_addr();
    let server_addr = bind_addr.clone();
    let server = thread::spawn(move || run_service_contract_server(server_addr, 1));
    wait_for_server_ready(bind_addr.as_str());

    let client = ServiceApiClient::connect(format!("http://{bind_addr}").as_str())
        .expect("client should connect");
    let sender = AgentDid::parse("kamn:did:agent:sdk-list").expect("sender did should parse");

    let messages = client
        .list_channel_messages(
            "channel-local-123",
            &auth_with_scope(&sender, 1, "", "channels:read"),
        )
        .expect("list channel messages should succeed");
    assert_eq!(messages.channel_id, "channel-local-123");
    assert_eq!(
        messages.messages,
        vec!["msg-local-a".to_owned(), "msg-local-b".to_owned()]
    );

    let server_result = server.join().expect("server thread should join");
    assert!(
        server_result.is_ok(),
        "test service contract server should satisfy request budget"
    );
}

#[test]
fn regression_service_api_client_registration_surface_contract_exists() {
    let route_source = std::fs::read_to_string("src/service_client_bridge_misc_routes.rs")
        .expect("route source should be readable");
    assert!(
        route_source.contains("pub fn register_agent("),
        "service client should expose a typed register_agent route"
    );

    let model_source = std::fs::read_to_string("src/service_models.rs")
        .expect("service models should be readable");
    assert!(
        model_source.contains("pub agent_type: String"),
        "service agent profile should expose agent_type"
    );
    assert!(
        model_source.contains("pub model_family: String"),
        "service agent profile should expose model_family"
    );
    assert!(
        model_source.contains("pub capabilities: Vec<String>"),
        "service agent profile should expose capabilities"
    );
    assert!(
        route_source.contains("pub fn search_agents("),
        "service client should expose a typed search_agents route"
    );
}

#[test]
fn spec_c02_service_api_client_executes_task_transition_and_escrow_route_contracts() {
    let bind_addr = reserve_loopback_addr();
    let server_addr = bind_addr.clone();
    let server = thread::spawn(move || run_service_contract_server(server_addr, 4));
    wait_for_server_ready(bind_addr.as_str());

    let client = ServiceApiClient::connect(format!("http://{bind_addr}").as_str())
        .expect("client should connect");
    let sender =
        AgentDid::parse("kamn:did:agent:sdk-task-escrow").expect("sender did should parse");

    let accepted = client
        .accept_task(
            "task-local-123",
            &auth_with_scope(&sender, 1, "{}", "tasks:write"),
        )
        .expect("accept task should succeed");
    assert_eq!(accepted.task_id, "task-local-123");
    assert_eq!(accepted.state, "accepted");

    let completed = client
        .complete_task(
            "task-local-123",
            &auth_with_scope(&sender, 2, "{}", "tasks:write"),
        )
        .expect("complete task should succeed");
    assert_eq!(completed.task_id, "task-local-123");
    assert_eq!(completed.state, "completed");

    let fund_payload = r#"{"task_id":"task-local-123","amount":100}"#;
    let funded = client
        .fund_escrow(
            fund_payload,
            &auth_with_scope(&sender, 3, fund_payload, "escrow:write"),
        )
        .expect("fund escrow should succeed");
    assert!(funded.escrow_id.starts_with("escrow-local-"));
    assert_eq!(funded.state, "funded");

    let released = client
        .release_escrow(
            funded.escrow_id.as_str(),
            &auth_with_scope(&sender, 4, "{}", "escrow:write"),
        )
        .expect("release escrow should succeed");
    assert_eq!(released.escrow_id, funded.escrow_id);
    assert_eq!(released.state, "released");

    let server_result = server.join().expect("server thread should join");
    assert!(
        server_result.is_ok(),
        "test service contract server should satisfy request budget"
    );
}

#[test]
fn spec_c03_service_api_client_executes_bridge_route_contracts() {
    let bind_addr = reserve_loopback_addr();
    let server_addr = bind_addr.clone();
    let server = thread::spawn(move || run_service_contract_server(server_addr, 3));
    wait_for_server_ready(bind_addr.as_str());

    let client = ServiceApiClient::connect(format!("http://{bind_addr}").as_str())
        .expect("client should connect");
    let sender = AgentDid::parse("kamn:did:agent:sdk-bridge").expect("sender did should parse");

    let submit_payload = r#"{"source_message_id":"msg-sdk","target_network":"testnet"}"#;
    let submitted = client
        .submit_bridge_message(
            submit_payload,
            &auth_with_scope(&sender, 1, submit_payload, "bridge:write"),
        )
        .expect("submit bridge should succeed");
    assert!(submitted.bridge_id.starts_with("bridge-local-"));
    assert_eq!(submitted.bridge_status, "submitted");

    let forwarded = client
        .forward_bridge_message(
            submitted.bridge_id.as_str(),
            &auth_with_scope(&sender, 2, "{}", "bridge:write"),
        )
        .expect("forward bridge should succeed");
    assert_eq!(forwarded.bridge_id, submitted.bridge_id);
    assert_eq!(forwarded.bridge_status, "forwarded");
    assert!(
        forwarded
            .target_message_id
            .starts_with("msg-bridge-target-"),
        "forward route should expose target message marker"
    );

    let queried = client
        .get_bridge_message(
            submitted.bridge_id.as_str(),
            &auth_with_scope(&sender, 3, "", "bridge:read"),
        )
        .expect("query bridge should succeed");
    assert_eq!(queried.bridge_id, submitted.bridge_id);
    assert_eq!(queried.bridge_status, forwarded.bridge_status);
    assert_eq!(queried.target_message_id, forwarded.target_message_id);
    assert_eq!(queried.forward_tx_hash, forwarded.forward_tx_hash);

    let server_result = server.join().expect("server thread should join");
    assert!(
        server_result.is_ok(),
        "test service contract server should satisfy request budget"
    );
}
