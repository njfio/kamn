use super::{
    build_bootstrap_report, build_kolme_live_direct_signed_wire_payload,
    build_kolme_live_managed_signing_key, build_kolme_live_request,
    build_kolme_live_signer_adapter, build_runtime_observability_snapshot,
    build_service_api_snapshot, capture_test_logs,
    classify_full_bootstrap_component_contract_violation,
    classify_full_supervisor_stop_contract_violation,
    classify_kolme_live_signer_key_source_policy_violation,
    classify_production_transport_profile_violation, encode_kolme_hex_lower,
    enforce_kolme_live_signer_contract_policy, enforce_kolme_live_signer_key_source_policy,
    enforce_kolme_live_signer_preflight, execute, parse_args, render_bootstrap_report,
    render_kolme_live_native_direct_message, render_log_event_line,
    render_observability_endpoint_response, render_service_api_endpoint_response,
    resolve_kolme_live_allow_local_signer_testing_override,
    resolve_kolme_live_managed_signer_required_marker, resolve_kolme_live_nonce,
    resolve_kolme_live_signer_private_key_env_name, resolve_log_config_from_inputs,
    select_runtime_transport_profile_for_runtime_mode, serve_observability_endpoint,
    serve_service_api_endpoint, sign_kolme_live_managed_external_message,
    validate_full_supervisor_stop_contract, DiagnosticsMode, KolmeForkSecp256k1SignerAdapter,
    LocalProfile, NodeBootstrapReport, NodeLogConfig, NodeLogFormat, NodeLogLevel,
    ObservabilityEndpointConfig, OutputMode, RuntimeExecutionBundle, RuntimeMode,
    ServiceApiEndpointConfig,
};
use kamn_core::{
    bootstrap, ConfigError, KolmeRuntimeCommitHttpTransport, KolmeRuntimeCommitRequest, NodeConfig,
    NodeRole, SignerProviderHandshakeMatrix, SyncMode,
};
use std::io::{ErrorKind, Read, Write};
use std::net::TcpListener;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};
use std::{env, sync::OnceLock};

const TEST_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX: &str =
    "658c3528422eb527b4c108b8f6d1e5f629543c304ea49cf608c67794424291c4";
const TEST_KOLME_LIVE_SIGNER_PRIVATE_KEY_HEX_SECONDARY: &str =
    "838c3528422eb527b4c108b8f6d1e5f629543c304ea49cf608c67794424291c4";
const TEST_KOLME_LIVE_MANAGED_KEY_REFERENCE: &str =
    "secure:aws-kms:role-operator/key-live-ops-primary";
const TEST_KOLME_LIVE_MANAGED_KEY_REFERENCE_SECONDARY: &str =
    "secure:aws-kms:role-operator/key-live-ops-secondary";

fn signer_env_lock() -> &'static Mutex<()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
}

fn log_env_lock() -> &'static Mutex<()> {
    // Log-config and signer tests both mutate process-wide env; share one lock to avoid races.
    signer_env_lock()
}

fn managed_signer_public_key_hex(key_reference: &str) -> String {
    let signing_key = build_kolme_live_managed_signing_key(key_reference)
        .expect("managed signing key should derive");
    encode_kolme_hex_lower(
        signing_key
            .verifying_key()
            .to_encoded_point(true)
            .as_bytes(),
    )
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

#[derive(Clone)]
struct MockHttpReply {
    status_line: &'static str,
    body: String,
}

impl MockHttpReply {
    fn ok(body: &str) -> Self {
        Self {
            status_line: "HTTP/1.1 200 OK",
            body: body.to_owned(),
        }
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
        let read_count = match stream.read(&mut chunk) {
            Ok(read_count) => read_count,
            Err(error) if matches!(error.kind(), ErrorKind::WouldBlock | ErrorKind::TimedOut) => {
                break;
            }
            Err(error) => panic!("request bytes should be readable: {error}"),
        };
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

fn request_body(raw_request: &str) -> &str {
    raw_request
        .split_once("\r\n\r\n")
        .map(|(_, body)| body)
        .unwrap_or("")
}

fn extract_json_string_field(body: &str, field: &str) -> Option<String> {
    let marker = format!("\"{field}\":\"");
    let start = body.find(marker.as_str())?;
    let remainder = &body[start + marker.len()..];
    let end = remainder.find('"')?;
    Some(remainder[..end].to_owned())
}

fn spawn_kolme_live_mock_server(replies: Vec<MockHttpReply>) -> (String, Arc<Mutex<Vec<String>>>) {
    let listener = TcpListener::bind("127.0.0.1:0").expect("listener should bind");
    listener
        .set_nonblocking(true)
        .expect("listener should allow nonblocking accepts");
    let addr = listener.local_addr().expect("local addr should resolve");
    let recorded_requests = Arc::new(Mutex::new(Vec::new()));
    let recorded_requests_ref = Arc::clone(&recorded_requests);
    thread::spawn(move || {
        for reply in replies {
            let accept_deadline = Instant::now() + Duration::from_secs(2);
            loop {
                match listener.accept() {
                    Ok((mut stream, _)) => {
                        let request = read_http_request(&mut stream);
                        recorded_requests_ref
                            .lock()
                            .expect("request mutex should lock")
                            .push(request);
                        let response = format!(
                            "{}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                            reply.status_line,
                            reply.body.len(),
                            reply.body
                        );
                        stream
                            .write_all(response.as_bytes())
                            .expect("response should write");
                        break;
                    }
                    Err(error) if error.kind() == ErrorKind::WouldBlock => {
                        if Instant::now() >= accept_deadline {
                            return;
                        }
                        thread::sleep(Duration::from_millis(5));
                    }
                    Err(error) => panic!("accept should succeed: {error}"),
                }
            }
        }
    });
    (format!("http://{addr}"), recorded_requests)
}

// main_tests structural budget shell only; keep domain tests in src/main_tests/*.rs
mod async_runtime_contract_tests;
mod cli_contract_tests;
mod core_behavior_tests;
mod daemon_tests;
mod observability_endpoint_tests;
mod report_tests;
mod runtime_tests;
mod service_api_endpoint_tests;
mod signer_tests;
