use kamn_core::{
    KolmeApiBroadcastRequest, KolmeApiNextNonceRequest, KolmeCommitReceiptFinality,
    KolmeRuntimeCommitBlockFallbackTransport, KolmeRuntimeCommitFinalityChecker,
    KolmeRuntimeCommitHttpTransport, KolmeRuntimeCommitLiveProvider, KolmeRuntimeCommitProvider,
    KolmeRuntimeCommitProviderError, KolmeRuntimeCommitProviderOutcome, KolmeRuntimeCommitRequest,
};
use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpListener;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::{Mutex, OnceLock};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use std::{env, fs};

const TLS_CA_FILE_ENV: &str = "KAMN_KOLME_TLS_CA_FILE";

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
    cert_path: PathBuf,
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

fn generate_self_signed_certificate(temp_dir: &Path) -> (PathBuf, PathBuf) {
    let cert_path = temp_dir.join("cert.pem");
    let key_path = temp_dir.join("key.pem");

    let status = Command::new("openssl")
        .arg("req")
        .arg("-x509")
        .arg("-newkey")
        .arg("rsa:2048")
        .arg("-keyout")
        .arg(key_path.as_os_str())
        .arg("-out")
        .arg(cert_path.as_os_str())
        .arg("-days")
        .arg("1")
        .arg("-nodes")
        .arg("-subj")
        .arg("/CN=127.0.0.1")
        .arg("-addext")
        .arg("subjectAltName = DNS:localhost,IP:127.0.0.1")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .expect("openssl should run");
    assert!(
        status.success(),
        "openssl self-signed certificate generation should succeed"
    );

    (cert_path, key_path)
}

fn spawn_https_single_request_server(
    status_code: u16,
    response_body: &str,
) -> HttpsSingleRequestServer {
    let temp_dir = unique_temp_dir("kolme-https-server");
    let (cert_path, key_path) = generate_self_signed_certificate(temp_dir.as_path());
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
        .arg(cert_path.as_os_str())
        .arg(key_path.as_os_str())
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
        cert_path,
        child,
        temp_dir,
    }
}

fn read_http_request(stream: &mut std::net::TcpStream) -> String {
    let mut buffer = Vec::new();
    let mut chunk = [0_u8; 1024];
    let mut header_end = None;
    let mut expected_total = None;

    stream
        .set_read_timeout(Some(Duration::from_secs(2)))
        .expect("read timeout should be set");

    loop {
        let read_count = stream
            .read(&mut chunk)
            .expect("request bytes should be readable");
        if read_count == 0 {
            break;
        }
        buffer.extend_from_slice(&chunk[..read_count]);

        if header_end.is_none() {
            header_end = buffer
                .windows(4)
                .position(|window| window == b"\r\n\r\n")
                .map(|pos| pos + 4);
            if let Some(end) = header_end {
                let headers = String::from_utf8_lossy(&buffer[..end]);
                let content_length = headers
                    .lines()
                    .find_map(|line| {
                        let (name, value) = line.split_once(':')?;
                        if name.eq_ignore_ascii_case("Content-Length") {
                            return value.trim().parse::<usize>().ok();
                        }
                        None
                    })
                    .unwrap_or(0);
                expected_total = Some(end + content_length);
            }
        }

        if let Some(total) = expected_total {
            if buffer.len() >= total {
                break;
            }
        }
    }

    String::from_utf8(buffer).expect("request should be valid utf-8")
}

fn spawn_single_request_server(
    response_body: String,
    status_line: &str,
    handler: impl Fn(String) + Send + 'static,
) -> String {
    let listener = TcpListener::bind("127.0.0.1:0").expect("listener should bind");
    let addr = listener.local_addr().expect("local addr should resolve");
    let status_line = status_line.to_owned();
    thread::spawn(move || {
        let (mut stream, _) = listener.accept().expect("connection should be accepted");
        let request = read_http_request(&mut stream);
        handler(request);

        let response = format!(
            "{status_line}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
            response_body.len(),
            response_body
        );
        stream
            .write_all(response.as_bytes())
            .expect("response should write");
    });
    format!("http://{addr}")
}

fn spawn_server_with_raw_response(
    raw_response: String,
    handler: impl Fn(String) + Send + 'static,
) -> String {
    let listener = TcpListener::bind("127.0.0.1:0").expect("listener should bind");
    let addr = listener.local_addr().expect("local addr should resolve");
    thread::spawn(move || {
        let (mut stream, _) = listener.accept().expect("connection should be accepted");
        let request = read_http_request(&mut stream);
        handler(request);
        stream
            .write_all(raw_response.as_bytes())
            .expect("response should write");
    });
    format!("http://{addr}")
}

#[test]
fn unit_http_transport_rejects_zero_timeout_seconds() {
    assert!(
        matches!(
            KolmeRuntimeCommitHttpTransport::new(0),
            Err(kamn_core::KolmeRuntimeCommitError::InvalidRequest {
                field: "transport_timeout_seconds",
                reason: "must be positive",
            })
        ),
        "http transport timeout must be positive"
    );
}

#[test]
fn unit_http_transport_block_fetch_rejects_zero_height() {
    let mut transport = KolmeRuntimeCommitHttpTransport::new(1).expect("transport should build");
    assert_eq!(
        transport.fetch_block_by_height("http://127.0.0.1:3030", "/block/{height}", 0),
        Err(KolmeRuntimeCommitProviderError::MalformedResponse {
            reason: "block height must be positive".to_owned(),
        })
    );
}

#[test]
fn integration_http_transport_submit_and_response_mapping() {
    let wire_payload = "operation_id=op-1\nstate_root=state-1\n";
    let idempotency_key = "kolme-runtime-commit:op-1:state-1:agent-1:1:payload-1";
    let response_body =
        "status=submitted\nprovider=kolme-local\ncommit_id=kolme-commit:1\nfinality=final\n";
    let base_url = spawn_single_request_server(
        response_body.to_owned(),
        "HTTP/1.1 200 OK",
        move |request| {
            assert!(request.contains("POST /broadcast/runtime-commit HTTP/1.1"));
            assert!(request.contains("X-Idempotency-Key: "));
            assert!(request.ends_with(wire_payload));
        },
    );

    let transport = KolmeRuntimeCommitHttpTransport::new(2).expect("transport should build");
    let mut provider = KolmeRuntimeCommitLiveProvider::new(
        base_url.as_str(),
        "/broadcast/runtime-commit",
        transport,
    )
    .expect("provider should build");

    let outcome = provider
        .submit_runtime_commit(wire_payload, idempotency_key)
        .expect("submit should succeed");
    match outcome {
        KolmeRuntimeCommitProviderOutcome::Submitted(receipt) => {
            assert_eq!(receipt.provider, "kolme-local");
            assert_eq!(receipt.commit_id, "kolme-commit:1");
            assert_eq!(receipt.finality, KolmeCommitReceiptFinality::Final);
        }
        other => panic!("unexpected provider outcome: {other:?}"),
    }
}

#[test]
fn integration_http_transport_fetch_next_nonce_query_and_parse() {
    let nonce_request =
        KolmeApiNextNonceRequest::new("pub:key/with space").expect("request should build");
    let base_url = spawn_single_request_server(
        "{\"next_nonce\":42,\"account_id\":\"acc-42\"}".to_owned(),
        "HTTP/1.1 200 OK",
        move |request| {
            assert!(
                request.contains("GET /get-next-nonce?pubkey=pub%3Akey%2Fwith%20space HTTP/1.1")
            );
        },
    );

    let mut transport = KolmeRuntimeCommitHttpTransport::new(2).expect("transport should build");
    let response = transport
        .fetch_next_nonce(base_url.as_str(), "/get-next-nonce", &nonce_request)
        .expect("nonce helper should succeed");
    assert_eq!(response.next_nonce, 42);
    assert_eq!(response.account_id.as_deref(), Some("acc-42"));
}

#[test]
fn integration_http_transport_submit_broadcast_request_put_and_parse_txhash() {
    let broadcast_request = KolmeApiBroadcastRequest::new("{\"nonce\":42}", "sig-42", 1)
        .expect("broadcast request should build");
    let idempotency_key = "kolme-runtime-commit:typed-broadcast-42";
    let base_url = spawn_single_request_server(
        "{\"txhash\":\"tx-typed-42\"}".to_owned(),
        "HTTP/1.1 200 OK",
        move |request| {
            assert!(request.contains("PUT /broadcast HTTP/1.1"));
            assert!(request.contains("Content-Type: application/json"));
            assert!(request.contains("X-Idempotency-Key: kolme-runtime-commit:typed-broadcast-42"));
            assert!(request.contains("\"message\":\"{\\\"nonce\\\":42}\""));
            assert!(request.contains("\"signature\":\"sig-42\""));
            assert!(request.contains("\"recovery_id\":1"));
        },
    );

    let mut transport = KolmeRuntimeCommitHttpTransport::new(2).expect("transport should build");
    let response = transport
        .submit_broadcast_request(
            base_url.as_str(),
            "/broadcast",
            &broadcast_request,
            idempotency_key,
        )
        .expect("broadcast helper should succeed");
    assert_eq!(response.txhash, "tx-typed-42");
}

#[test]
fn regression_issue_1912_http_transport_submit_broadcast_trims_idempotency_key() {
    // Regression: #1912
    let broadcast_request = KolmeApiBroadcastRequest::new("{\"nonce\":42}", "sig-42", 1)
        .expect("broadcast request should build");
    let base_url = spawn_single_request_server(
        "{\"txhash\":\"tx-typed-1912\"}".to_owned(),
        "HTTP/1.1 200 OK",
        move |request| {
            assert!(request.contains("PUT /broadcast HTTP/1.1"));
            assert!(
                request.contains("X-Idempotency-Key: kolme-runtime-commit:typed-broadcast-1912")
            );
        },
    );

    let mut transport = KolmeRuntimeCommitHttpTransport::new(2).expect("transport should build");
    let response = transport
        .submit_broadcast_request(
            base_url.as_str(),
            "/broadcast",
            &broadcast_request,
            "  kolme-runtime-commit:typed-broadcast-1912  ",
        )
        .expect("broadcast helper should normalize idempotency key");
    assert_eq!(response.txhash, "tx-typed-1912");
}

#[test]
fn regression_issue_1888_http_transport_submit_broadcast_defaults_empty_submit_path() {
    // Regression: #1888
    let broadcast_request = KolmeApiBroadcastRequest::new("{\"nonce\":8}", "sig-8", 1)
        .expect("broadcast request should build");
    let base_url = spawn_single_request_server(
        "{\"txhash\":\"tx-typed-8\"}".to_owned(),
        "HTTP/1.1 200 OK",
        move |request| {
            assert!(request.contains("PUT /broadcast HTTP/1.1"));
        },
    );

    let mut transport = KolmeRuntimeCommitHttpTransport::new(2).expect("transport should build");
    let response = transport
        .submit_broadcast_request(
            base_url.as_str(),
            "   ",
            &broadcast_request,
            "kolme-runtime-commit:typed-broadcast-8",
        )
        .expect("broadcast helper should default empty submit path");
    assert_eq!(response.txhash, "tx-typed-8");
}

#[test]
fn regression_http_transport_submit_broadcast_request_rejects_malformed_txhash_response() {
    let broadcast_request = KolmeApiBroadcastRequest::new("{\"nonce\":7}", "sig-7", 1)
        .expect("broadcast request should build");
    let base_url = spawn_single_request_server(
        "{\"status\":\"ok\"}".to_owned(),
        "HTTP/1.1 200 OK",
        move |request| {
            assert!(request.contains("PUT /broadcast HTTP/1.1"));
        },
    );

    let mut transport = KolmeRuntimeCommitHttpTransport::new(2).expect("transport should build");
    assert_eq!(
        transport.submit_broadcast_request(
            base_url.as_str(),
            "/broadcast",
            &broadcast_request,
            "kolme-runtime-commit:typed-broadcast-7",
        ),
        Err(KolmeRuntimeCommitProviderError::MalformedResponse {
            reason: "missing required field: txhash".to_owned(),
        })
    );
}

#[test]
fn integration_http_transport_finality_query_and_response_mapping() {
    let commit_id = "commit:id/with space";
    let response_body =
        "{\"provider\":\"kolme-local\",\"commit_id\":\"commit:id/with space\",\"finality\":\"final\"}";
    let base_url = spawn_single_request_server(
        response_body.to_owned(),
        "HTTP/1.1 200 OK",
        move |request| {
            assert!(request.contains(
                "GET /runtime-commit/status?commit_id=commit%3Aid%2Fwith%20space HTTP/1.1"
            ));
        },
    );

    let transport = KolmeRuntimeCommitHttpTransport::new(2).expect("transport should build");
    let mut checker = KolmeRuntimeCommitFinalityChecker::new(
        base_url.as_str(),
        "/runtime-commit/status",
        transport,
    )
    .expect("checker should build");

    let receipt = checker
        .check_commit_finality(commit_id)
        .expect("finality check should succeed");
    assert_eq!(receipt.provider, "kolme-local");
    assert_eq!(receipt.commit_id, commit_id);
    assert_eq!(receipt.finality, KolmeCommitReceiptFinality::Final);
}

#[test]
fn regression_http_transport_timeout_maps_to_provider_timeout() {
    let listener = TcpListener::bind("127.0.0.1:0").expect("listener should bind");
    let addr = listener.local_addr().expect("local addr should resolve");
    thread::spawn(move || {
        let (_stream, _) = listener.accept().expect("connection should be accepted");
        thread::sleep(Duration::from_secs(2));
    });

    let transport = KolmeRuntimeCommitHttpTransport::new(1).expect("transport should build");
    let mut provider = KolmeRuntimeCommitLiveProvider::new(
        format!("http://{addr}").as_str(),
        "/broadcast/runtime-commit",
        transport,
    )
    .expect("provider should build");

    assert_eq!(
        provider.submit_runtime_commit("operation_id=op-1\n", "idempotency-key-1"),
        Err(KolmeRuntimeCommitProviderError::Timeout)
    );
}

#[test]
fn regression_http_transport_rejects_invalid_port_before_network_io() {
    let transport = KolmeRuntimeCommitHttpTransport::new(1).expect("transport should build");
    let mut provider = KolmeRuntimeCommitLiveProvider::new(
        "http://127.0.0.1:abc",
        "/broadcast/runtime-commit",
        transport,
    )
    .expect("provider should build");

    assert_eq!(
        provider.submit_runtime_commit("operation_id=op-1\n", "idempotency-key-1"),
        Err(KolmeRuntimeCommitProviderError::Unavailable {
            reason: "base_url port is invalid".to_owned(),
        })
    );
}

#[test]
fn regression_issue_1884_http_transport_rejects_empty_idempotency_key() {
    // Regression: #1884
    let transport = KolmeRuntimeCommitHttpTransport::new(1).expect("transport should build");
    let mut provider = KolmeRuntimeCommitLiveProvider::new(
        "http://127.0.0.1:1",
        "/broadcast/runtime-commit",
        transport,
    )
    .expect("provider should build");

    assert_eq!(
        provider.submit_runtime_commit("operation_id=op-1\n", " "),
        Err(KolmeRuntimeCommitProviderError::MalformedResponse {
            reason: "idempotency_key must not be empty".to_owned(),
        })
    );
}

#[test]
fn regression_issue_1886_http_transport_rejects_empty_wire_payload() {
    // Regression: #1886
    let transport = KolmeRuntimeCommitHttpTransport::new(1).expect("transport should build");
    let mut provider = KolmeRuntimeCommitLiveProvider::new(
        "http://127.0.0.1:1",
        "/broadcast/runtime-commit",
        transport,
    )
    .expect("provider should build");

    assert_eq!(
        provider.submit_runtime_commit(" ", "idempotency-key-1"),
        Err(KolmeRuntimeCommitProviderError::MalformedResponse {
            reason: "wire_payload must not be empty".to_owned(),
        })
    );
}

#[test]
fn regression_http_transport_fails_closed_on_content_length_mismatch() {
    let body = "status=submitted\nprovider=kolme-local\ncommit_id=kolme-commit:1\nfinality=final\n";
    let declared_length = body.len() + 9;
    let raw_response = format!(
        "HTTP/1.1 200 OK\r\nContent-Length: {declared_length}\r\nConnection: close\r\n\r\n{body}"
    );

    let base_url = spawn_server_with_raw_response(raw_response, |request| {
        assert!(request.contains("POST /broadcast/runtime-commit HTTP/1.1"));
    });

    let transport = KolmeRuntimeCommitHttpTransport::new(1).expect("transport should build");
    let mut provider = KolmeRuntimeCommitLiveProvider::new(
        base_url.as_str(),
        "/broadcast/runtime-commit",
        transport,
    )
    .expect("provider should build");

    assert_eq!(
        provider.submit_runtime_commit("operation_id=op-1\n", "idempotency-key-1"),
        Err(KolmeRuntimeCommitProviderError::MalformedResponse {
            reason: format!(
                "http response content-length mismatch: declared {declared_length}, observed {}",
                body.len()
            ),
        })
    );
}

#[test]
fn functional_http_transport_includes_authorization_header_when_configured() {
    let wire_payload = "operation_id=op-auth\nstate_root=state-auth\n";
    let idempotency_key = "kolme-runtime-commit:op-auth:state-auth:agent-1:1:payload-auth";
    let response_body =
        "status=submitted\nprovider=kolme-local\ncommit_id=kolme-commit:auth\nfinality=final\n";
    let base_url = spawn_single_request_server(
        response_body.to_owned(),
        "HTTP/1.1 200 OK",
        move |request| {
            assert!(request.contains("Authorization: Bearer integration-token"));
        },
    );

    let transport =
        KolmeRuntimeCommitHttpTransport::new_with_authorization(2, "Bearer integration-token")
            .expect("transport should build");
    let mut provider = KolmeRuntimeCommitLiveProvider::new(
        base_url.as_str(),
        "/broadcast/runtime-commit",
        transport,
    )
    .expect("provider should build");

    let outcome = provider
        .submit_runtime_commit(wire_payload, idempotency_key)
        .expect("submit should succeed");
    match outcome {
        KolmeRuntimeCommitProviderOutcome::Submitted(receipt) => {
            assert_eq!(receipt.provider, "kolme-local");
            assert_eq!(receipt.commit_id, "kolme-commit:auth");
        }
        other => panic!("unexpected provider outcome: {other:?}"),
    }
}

#[test]
fn regression_http_transport_maps_401_to_authorization_unavailable_error() {
    let base_url = spawn_single_request_server(
        "{\"error\":\"unauthorized\"}".to_owned(),
        "HTTP/1.1 401 Unauthorized",
        |_| {},
    );

    let transport = KolmeRuntimeCommitHttpTransport::new(1).expect("transport should build");
    let mut provider = KolmeRuntimeCommitLiveProvider::new(
        base_url.as_str(),
        "/broadcast/runtime-commit",
        transport,
    )
    .expect("provider should build");

    assert_eq!(
        provider.submit_runtime_commit("operation_id=op-1\n", "idempotency-key-1"),
        Err(KolmeRuntimeCommitProviderError::Unavailable {
            reason: "http response status indicates authorization failure: 401".to_owned(),
        })
    );
}

#[test]
fn regression_http_transport_maps_422_to_invalid_request_malformed_error() {
    let base_url = spawn_single_request_server(
        "{\"error\":\"validation failed\"}".to_owned(),
        "HTTP/1.1 422 Unprocessable Entity",
        |_| {},
    );

    let transport = KolmeRuntimeCommitHttpTransport::new(1).expect("transport should build");
    let mut provider = KolmeRuntimeCommitLiveProvider::new(
        base_url.as_str(),
        "/broadcast/runtime-commit",
        transport,
    )
    .expect("provider should build");

    assert_eq!(
        provider.submit_runtime_commit("operation_id=op-1\n", "idempotency-key-1"),
        Err(KolmeRuntimeCommitProviderError::MalformedResponse {
            reason: "http response status indicates invalid request: 422".to_owned(),
        })
    );
}

#[test]
fn functional_https_transport_submit_with_trusted_ca_succeeds() {
    let _guard = tls_env_lock()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let mut server = spawn_https_single_request_server(
        200,
        "status=submitted\nprovider=kolme-local\ncommit_id=kolme-commit:https\nfinality=final\n",
    );
    let cert_path = server
        .cert_path
        .to_str()
        .expect("temporary cert path should be valid utf-8")
        .to_owned();
    let _env_guard = EnvVarGuard::set(TLS_CA_FILE_ENV, Some(cert_path.as_str()));

    let transport = KolmeRuntimeCommitHttpTransport::new(2).expect("transport should build");
    let mut provider = KolmeRuntimeCommitLiveProvider::new(
        server.base_url.as_str(),
        "/broadcast/runtime-commit",
        transport,
    )
    .expect("provider should build");

    let outcome = provider
        .submit_runtime_commit("operation_id=op-https\n", "idempotency-key-https")
        .expect("https submit should succeed");
    match outcome {
        KolmeRuntimeCommitProviderOutcome::Submitted(receipt) => {
            assert_eq!(receipt.provider, "kolme-local");
            assert_eq!(receipt.commit_id, "kolme-commit:https");
            assert_eq!(receipt.finality, KolmeCommitReceiptFinality::Final);
        }
        other => panic!("unexpected provider outcome: {other:?}"),
    }

    server.wait_for_exit();
}

#[test]
fn regression_https_transport_maps_certificate_errors_to_unavailable() {
    let _guard = tls_env_lock()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let mut server = spawn_https_single_request_server(
        200,
        "status=submitted\nprovider=kolme-local\ncommit_id=kolme-commit:https\nfinality=final\n",
    );
    let _env_guard = EnvVarGuard::set(TLS_CA_FILE_ENV, None);

    let transport = KolmeRuntimeCommitHttpTransport::new(2).expect("transport should build");
    let mut provider = KolmeRuntimeCommitLiveProvider::new(
        server.base_url.as_str(),
        "/broadcast/runtime-commit",
        transport,
    )
    .expect("provider should build");

    assert_eq!(
        provider.submit_runtime_commit("operation_id=op-https\n", "idempotency-key-https"),
        Err(KolmeRuntimeCommitProviderError::Unavailable {
            reason: "tls certificate verification failed".to_owned(),
        })
    );

    server.wait_for_exit();
}

#[test]
fn regression_https_transport_maps_tls_handshake_failures_to_unavailable() {
    let _guard = tls_env_lock()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let _env_guard = EnvVarGuard::set(TLS_CA_FILE_ENV, None);
    let listener = TcpListener::bind("127.0.0.1:0").expect("listener should bind");
    let addr = listener.local_addr().expect("local addr should resolve");
    thread::spawn(move || {
        let (mut stream, _) = listener.accept().expect("connection should be accepted");
        let _ = stream.read(&mut [0_u8; 64]);
        let _ =
            stream.write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 0\r\nConnection: close\r\n\r\n");
    });

    let transport = KolmeRuntimeCommitHttpTransport::new(2).expect("transport should build");
    let mut provider = KolmeRuntimeCommitLiveProvider::new(
        format!("https://{addr}").as_str(),
        "/broadcast/runtime-commit",
        transport,
    )
    .expect("provider should build");

    assert_eq!(
        provider.submit_runtime_commit("operation_id=op-https\n", "idempotency-key-https"),
        Err(KolmeRuntimeCommitProviderError::Unavailable {
            reason: "tls handshake failed".to_owned(),
        })
    );
}

#[test]
fn functional_kolme_fork_submit_profile_uses_put_broadcast_and_maps_txhash_response() {
    let wire_payload = "operation_id=op-1\nstate_root=state-1\n";
    let idempotency_key = "kolme-runtime-commit:op-1:state-1:agent-1:1:payload-1";
    let base_url = spawn_single_request_server(
        "{\"txhash\":\"ab12cd34\"}".to_owned(),
        "HTTP/1.1 200 OK",
        move |request| {
            assert!(request.contains("PUT /broadcast HTTP/1.1"));
            assert!(request.contains("Content-Type: application/json"));
            assert!(request.contains("X-Idempotency-Key: "));
        },
    );

    let transport = KolmeRuntimeCommitHttpTransport::new(2).expect("transport should build");
    let mut provider = KolmeRuntimeCommitLiveProvider::new_kolme_fork_broadcast_profile(
        base_url.as_str(),
        "kolme-fork-local",
        transport,
    )
    .expect("provider should build");

    let outcome = provider
        .submit_runtime_commit(wire_payload, idempotency_key)
        .expect("submit should succeed");
    match outcome {
        KolmeRuntimeCommitProviderOutcome::Submitted(receipt) => {
            assert_eq!(receipt.provider, "kolme-fork-local");
            assert_eq!(receipt.commit_id, "kolme-commit:ab12cd34");
            assert_eq!(receipt.finality, KolmeCommitReceiptFinality::Pending);
        }
        other => panic!("unexpected provider outcome: {other:?}"),
    }
}

#[test]
fn regression_kolme_fork_submit_profile_requires_non_empty_provider_hint() {
    let transport = KolmeRuntimeCommitHttpTransport::new(2).expect("transport should build");
    let error = KolmeRuntimeCommitLiveProvider::new_kolme_fork_broadcast_profile(
        "http://127.0.0.1:3030",
        "",
        transport,
    )
    .expect_err("empty provider hint must fail validation");

    assert_eq!(
        error.to_string(),
        "invalid runtime commit request provider_hint: must not be empty"
    );
}

#[test]
fn integration_kolme_fork_signed_envelope_submit_maps_txhash_response() {
    let request = KolmeRuntimeCommitRequest::deterministic(
        "op-1506-http-a",
        "state:1506",
        "kamn:did:agent:http-1506-a",
        21,
        "payload:1506-http-a",
    )
    .expect("request should build");
    let signed_envelope = request
        .translate_to_signed_broadcast_envelope(
            "kamn:key:signer:http-1",
            request.to_wire_payload().as_str(),
            "sig-1506-http-a",
            1,
        )
        .expect("signed envelope should build");
    let wire_payload = signed_envelope.to_wire_payload();

    let base_url = spawn_single_request_server(
        "{\"txhash\":\"ab12cd34\"}".to_owned(),
        "HTTP/1.1 200 OK",
        move |raw_request| {
            assert!(raw_request.contains("PUT /broadcast HTTP/1.1"));
            assert!(raw_request.contains("Content-Type: application/json"));
            assert!(raw_request.contains("\"message\":\"operation_id=op-1506-http-a"));
            assert!(raw_request.contains("\"signature\":\"sig-1506-http-a\""));
            assert!(raw_request.contains("\"recovery_id\":1"));
        },
    );

    let transport = KolmeRuntimeCommitHttpTransport::new(2).expect("transport should build");
    let mut provider = KolmeRuntimeCommitLiveProvider::new_kolme_fork_broadcast_profile(
        base_url.as_str(),
        "kolme-fork-local",
        transport,
    )
    .expect("provider should build");

    let outcome = provider
        .submit_runtime_commit(wire_payload.as_str(), request.idempotency_key())
        .expect("signed submit should succeed");
    match outcome {
        KolmeRuntimeCommitProviderOutcome::Submitted(receipt) => {
            assert_eq!(receipt.provider, "kolme-fork-local");
            assert_eq!(receipt.commit_id, "kolme-commit:ab12cd34");
            assert_eq!(receipt.finality, KolmeCommitReceiptFinality::Pending);
        }
        other => panic!("unexpected provider outcome: {other:?}"),
    }
}

#[test]
fn integration_kolme_fork_direct_signed_payload_submit_maps_txhash_response() {
    let wire_payload = "{\"message\":\"{\\\"pubkey\\\":\\\"pk-direct\\\",\\\"nonce\\\":1,\\\"created\\\":\\\"2026-02-11T00:00:00Z\\\",\\\"messages\\\":[],\\\"max_height\\\":null}\",\"signature\":\"sig-direct\",\"recovery_id\":1}";
    let idempotency_key = "kolme-runtime-commit:direct-signed:1";

    let base_url = spawn_single_request_server(
        "{\"txhash\":\"ab12cd34\"}".to_owned(),
        "HTTP/1.1 200 OK",
        move |raw_request| {
            assert!(raw_request.contains("PUT /broadcast HTTP/1.1"));
            assert!(raw_request.contains("Content-Type: application/json"));
            assert!(raw_request.contains("\"signature\":\"sig-direct\""));
            assert!(raw_request.contains("\"recovery_id\":1"));
            assert!(raw_request.contains("\\\"pubkey\\\":\\\"pk-direct\\\""));
        },
    );

    let transport = KolmeRuntimeCommitHttpTransport::new(2).expect("transport should build");
    let mut provider = KolmeRuntimeCommitLiveProvider::new_kolme_fork_broadcast_profile(
        base_url.as_str(),
        "kolme-fork-local",
        transport,
    )
    .expect("provider should build");

    let outcome = provider
        .submit_runtime_commit(wire_payload, idempotency_key)
        .expect("direct signed submit should succeed");
    match outcome {
        KolmeRuntimeCommitProviderOutcome::Submitted(receipt) => {
            assert_eq!(receipt.provider, "kolme-fork-local");
            assert_eq!(receipt.commit_id, "kolme-commit:ab12cd34");
            assert_eq!(receipt.finality, KolmeCommitReceiptFinality::Pending);
        }
        other => panic!("unexpected provider outcome: {other:?}"),
    }
}

#[test]
fn regression_kolme_fork_signed_envelope_requires_signer_key_id() {
    // Regression: #1506
    let transport = KolmeRuntimeCommitHttpTransport::new(2).expect("transport should build");
    let mut provider = KolmeRuntimeCommitLiveProvider::new_kolme_fork_broadcast_profile(
        "http://127.0.0.1:3030",
        "kolme-fork-local",
        transport,
    )
    .expect("provider should build");

    let malformed_envelope = "{\"signer_key_id\":\"\",\"message\":\"operation_id=op\\nidempotency_key=abc\\n\",\"signature\":\"sig\",\"recovery_id\":1}";
    assert_eq!(
        provider.submit_runtime_commit(malformed_envelope, "abc"),
        Err(KolmeRuntimeCommitProviderError::MalformedResponse {
            reason: "field must not be empty: signer_key_id".to_owned(),
        })
    );
}

#[test]
fn regression_kolme_fork_direct_signed_payload_requires_json_message_shape() {
    // Regression: #1516
    let transport = KolmeRuntimeCommitHttpTransport::new(2).expect("transport should build");
    let mut provider = KolmeRuntimeCommitLiveProvider::new_kolme_fork_broadcast_profile(
        "http://127.0.0.1:3030",
        "kolme-fork-local",
        transport,
    )
    .expect("provider should build");

    let malformed_direct_payload =
        "{\"message\":\"operation_id=op\\nidempotency_key=abc\\n\",\"signature\":\"sig\",\"recovery_id\":1}";
    assert_eq!(
        provider.submit_runtime_commit(malformed_direct_payload, "abc"),
        Err(KolmeRuntimeCommitProviderError::MalformedResponse {
            reason: "direct signed payload message must be a JSON object string".to_owned(),
        })
    );
}

#[test]
fn regression_kolme_fork_direct_signed_payload_requires_core_transaction_keys() {
    // Regression: #1519
    let missing_key_cases = [
        (
            "pubkey",
            "{\"nonce\":1,\"created\":\"2026-02-11T00:00:00Z\",\"messages\":[],\"max_height\":null}",
        ),
        (
            "nonce",
            "{\"pubkey\":\"pk-direct\",\"created\":\"2026-02-11T00:00:00Z\",\"messages\":[],\"max_height\":null}",
        ),
        (
            "created",
            "{\"pubkey\":\"pk-direct\",\"nonce\":1,\"messages\":[],\"max_height\":null}",
        ),
        (
            "messages",
            "{\"pubkey\":\"pk-direct\",\"nonce\":1,\"created\":\"2026-02-11T00:00:00Z\",\"max_height\":null}",
        ),
    ];

    for (missing_field, message_json) in missing_key_cases {
        let wire_payload = format!(
            "{{\"message\":\"{}\",\"signature\":\"sig-direct\",\"recovery_id\":1}}",
            message_json.replace('\\', "\\\\").replace('\"', "\\\"")
        );
        let base_url = spawn_single_request_server(
            "{\"txhash\":\"ab12cd34\"}".to_owned(),
            "HTTP/1.1 200 OK",
            |_raw_request| {},
        );

        let transport = KolmeRuntimeCommitHttpTransport::new(2).expect("transport should build");
        let mut provider = KolmeRuntimeCommitLiveProvider::new_kolme_fork_broadcast_profile(
            base_url.as_str(),
            "kolme-fork-local",
            transport,
        )
        .expect("provider should build");

        assert_eq!(
            provider.submit_runtime_commit(
                wire_payload.as_str(),
                "kolme-runtime-commit:direct-required-fields:1"
            ),
            Err(KolmeRuntimeCommitProviderError::MalformedResponse {
                reason: format!(
                    "direct signed payload message missing required field: {missing_field}"
                ),
            })
        );
    }
}

#[test]
#[ignore = "requires reachable local Kolme node and explicit local opt-in lane"]
fn integration_kolme_fork_live_node_submit_reaches_endpoint() {
    let base_url = env::var("KAMN_KOLME_LIVE_BASE_URL")
        .expect("KAMN_KOLME_LIVE_BASE_URL must be set for live node smoke");
    let provider_hint =
        env::var("KAMN_KOLME_LIVE_PROVIDER_HINT").unwrap_or_else(|_| "kolme-fork-local".to_owned());
    let authorization_header = env::var("KAMN_KOLME_LIVE_AUTHORIZATION").ok();

    let transport = if let Some(value) = authorization_header {
        KolmeRuntimeCommitHttpTransport::new_with_authorization(10, value.as_str())
            .expect("transport with authorization should build")
    } else {
        KolmeRuntimeCommitHttpTransport::new(10).expect("transport should build")
    };

    let mut provider = KolmeRuntimeCommitLiveProvider::new_kolme_fork_broadcast_profile(
        base_url.as_str(),
        provider_hint.as_str(),
        transport,
    )
    .expect("provider should build");

    let unique_suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock should be after unix epoch")
        .as_nanos();
    let message_json = format!(
        "{{\"pubkey\":\"pk-live-smoke-{unique_suffix}\",\"nonce\":1,\"created\":\"2026-02-11T00:00:00Z\",\"messages\":[],\"max_height\":null}}"
    );
    let wire_payload = format!(
        "{{\"message\":\"{}\",\"signature\":\"sig-live-smoke-{unique_suffix}\",\"recovery_id\":1}}",
        message_json.replace('\\', "\\\\").replace('\"', "\\\"")
    );
    let idempotency_key = format!("kolme-runtime-commit:live-smoke:{unique_suffix}");

    let outcome = provider.submit_runtime_commit(wire_payload.as_str(), idempotency_key.as_str());
    match outcome {
        Ok(KolmeRuntimeCommitProviderOutcome::Submitted(receipt))
        | Ok(KolmeRuntimeCommitProviderOutcome::Duplicate(receipt)) => {
            assert!(!receipt.provider.trim().is_empty());
            assert!(!receipt.commit_id.trim().is_empty());
        }
        Ok(KolmeRuntimeCommitProviderOutcome::Rejected { reason }) => {
            assert!(!reason.trim().is_empty());
        }
        Err(KolmeRuntimeCommitProviderError::MalformedResponse { reason }) => {
            assert!(
                reason.contains("invalid request")
                    || reason.contains("missing required field")
                    || reason.contains("txhash"),
                "unexpected malformed response reason from live node: {reason}"
            );
        }
        Err(other) => {
            panic!("live node smoke expected endpoint reachability outcome, got error: {other:?}");
        }
    }
}
