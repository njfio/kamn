use super::*;
use crate::service_api_endpoint::{
    parse_service_api_payload, project_service_api_lifecycle_rejection, ServiceApiAgentGetBody,
    ServiceApiChannelCreateBody, ServiceApiHealthBody, ServiceApiLifecycleRejectionProjection,
    ServiceApiMessageCreateBody, ServiceApiTaskCreateBody, DEFAULT_SERVICE_API_BODY_LIMIT_BYTES,
    DEFAULT_SERVICE_API_CONCURRENCY_LIMIT, DEFAULT_SERVICE_API_RATE_LIMIT_PER_SECOND,
};
use kamn_core::baseline_signature_for_fields;
use rustls::pki_types::{CertificateDer, ServerName, UnixTime};
use serde::Deserialize;
use serde_json::Value;
use std::fs;
use std::io::{ErrorKind, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Barrier, MutexGuard};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

const TEST_SERVICE_API_TLS_CERT_PEM: &str = "-----BEGIN CERTIFICATE-----
MIIDCTCCAfGgAwIBAgIUX9dYtx2K5dX0X33CQvg4re7nVwwwDQYJKoZIhvcNAQEL
BQAwFDESMBAGA1UEAwwJbG9jYWxob3N0MB4XDTI2MDIxNTEzMDkwNFoXDTI2MDIx
NjEzMDkwNFowFDESMBAGA1UEAwwJbG9jYWxob3N0MIIBIjANBgkqhkiG9w0BAQEF
AAOCAQ8AMIIBCgKCAQEApfGNzxPiL+e4z6Pok8RT5RkZE631O/Pg7VBgN4xnCjTz
xwjDihOJSCBl1wYM09xeFUHE6JjTO2ABHdmtXJxXWaAygWRUvdYOBbf1c8ObkanC
+0f0xzUn8rxYyDo8PknR9QR32dCVG5LM5XrIw08TQPAZxEdOEKPkgDqeCWRGsWO/
YbaziAHXNsNShvYucAlHxzfhXnhRhVKrdVyZ0G7wZZAZoMgSC15lWDWw1JxVbBqr
0ui8eajKEDg8NZz9mw0VEYGCJGacgn/Y7+YQviEKNL+2yj57LbGsFrXRfSczpNxV
JmgXChRy5849aLJsatm1NSAhYmFamX7d+7EErKPwhQIDAQABo1MwUTAdBgNVHQ4E
FgQU/EbABKdaVJZGhOBJ2/WodsjxNJcwHwYDVR0jBBgwFoAU/EbABKdaVJZGhOBJ
2/WodsjxNJcwDwYDVR0TAQH/BAUwAwEB/zANBgkqhkiG9w0BAQsFAAOCAQEAcYPD
j0Me1W3oQkgz9yGT75IYrM6bdSJRQt+vIKQI5AAVIqX5IoGfjP/zJFner96T0i/7
rPVinMFmyYTXYr/qqbQ9jdLt9FS+l0eIqN9oCmHC6Anhn9/FORZzBsIBQDPkZxXk
G5QUhQ/joTqTdUaQcrKh4UeRA1LJtlAnFnYc3CeQdKQQqB4W5JeZSdsU1E0FU5wl
fE7ucg85yIEn33V6aCexCfHhDh2TnLo25awqoyNCbFhu7DLnbnyOeKSB5lI3TdvK
ag0XPq+nohTyUBXw+XUR2PnYXOEGZxBQdhvyQO0ib/y2dcODuYbXkQDq+f0UuBbn
R/+8zPGgzivZEPa01Q==
-----END CERTIFICATE-----
";

const TEST_SERVICE_API_TLS_KEY_PEM: &str = "-----BEGIN PRIVATE KEY-----
MIIEvQIBADANBgkqhkiG9w0BAQEFAASCBKcwggSjAgEAAoIBAQCl8Y3PE+Iv57jP
o+iTxFPlGRkTrfU78+DtUGA3jGcKNPPHCMOKE4lIIGXXBgzT3F4VQcTomNM7YAEd
2a1cnFdZoDKBZFS91g4Ft/Vzw5uRqcL7R/THNSfyvFjIOjw+SdH1BHfZ0JUbkszl
esjDTxNA8BnER04Qo+SAOp4JZEaxY79htrOIAdc2w1KG9i5wCUfHN+FeeFGFUqt1
XJnQbvBlkBmgyBILXmVYNbDUnFVsGqvS6Lx5qMoQODw1nP2bDRURgYIkZpyCf9jv
5hC+IQo0v7bKPnstsawWtdF9JzOk3FUmaBcKFHLnzj1osmxq2bU1ICFiYVqZft37
sQSso/CFAgMBAAECggEACrFhAOn4FjwpRX/7WaI6AbY3TnRULBPP95rJSGsMrLSy
zK185CXUH8iup0dlhjVZ/qapSI+odNf/2muPZztPyZ+wAXR0nXLwnl+3Okltedpl
jQma9UcwlsyaL/TIsv7Qv6gVDP0KzqcL+vGJhERRKksObf5mQl49OCIO0u4aPA3l
0Y0h9WVtvydyhztQCFfVkkZNgiAY2WSI73xO72RFU0ZKnwc9ZVvona7yTKJIpV+i
3k0N/27kfc21UUtXJ7Nv5b07MIH8vkx+c1FX63vAPkyBdfXguNG/Yn/uVGqq2fbZ
xypp3JIRW3b2Heo/Ox02791gRuWJcpEmU369E4fq9QKBgQDT2T5m5HfHcA4Zpjk5
HtPvdINWntwkSZw8E/41LVY0PptOqM3yWDSb0TLQCoefhtPWm571RvSdevU+NpyB
jnzx+8gEXAeC1D6TKYmucO0wv7A5ZqC1WzLO7LKG4DuJbANs9PApuqPkzPAGegey
NkVOTRWO7ggLzmPxYFN8leW+fwKBgQDIhyOkchw1cl+GMDibnrB4Ynljvlxn0tDo
A4N3oSTv1Az8mO7DGJ+S/mY8aYmw4ogbPIGXZkxlhie0pS7kfttzswbY6TePgml6
pbLvfzv9OGUKLp0QhmNzfNygP6A8pIb2vbuEYJl6boE/jEIG7c8E0VmsUqe60Aoz
EcDLDtzW+wKBgQC5Hnj9CF/ykuR/XVVbqKih8jpikubjfr9bcE0OwtM1TBACqFdu
kc1G64NvcAQbToIGYm6A/sP6aNusxaP1QkHEYrPhu1mE5VrY1c9N87gQhTDEt/1u
/IZlc0h9u6vK5ewIZfEHReS5pquHvVLEU9A0H//aqf2182A6KGZL0+CymQKBgGsd
xSxSyD7EmcJUf+ihHCMydyWQykurkWxedBuzOMfjvgwwpVoSDSu4OWSL+8FBQPNL
nu4A905EG3GjyyjDmvZy63VzHvrJ7w5U9QB6NtFNDqwhukTZhMZsLG5tjmrWeEHV
mBVehJ2h6ejIQ3zwC2XHbt9eR7rC5q/hC9tsVQuBAoGAG8WncvZ15/VwncKQwz3G
bgoNsx0W5SO8NNfecDRVJLsCCuy5M9s5vn/u1Xz7l9pA0vCup9l6v96hQJTQBEQ6
urk/MQl1UlrSRdDK2gu40MToc8X5ig0dVDVG5QhPl7YmUu9G2EAL3WZTpJXsRh22
VpYUFFjotXCdBIUnUQ51PGg=
-----END PRIVATE KEY-----
";

#[derive(Debug, Deserialize)]
struct ServiceApiErrorEnvelope {
    error: String,
    reason_code: String,
    message: String,
}

#[derive(Debug)]
struct TestSkipServerVerification(Arc<rustls::crypto::CryptoProvider>);

impl TestSkipServerVerification {
    fn new() -> Arc<Self> {
        Arc::new(Self(Arc::new(rustls::crypto::ring::default_provider())))
    }
}

impl rustls::client::danger::ServerCertVerifier for TestSkipServerVerification {
    fn verify_server_cert(
        &self,
        _end_entity: &CertificateDer<'_>,
        _intermediates: &[CertificateDer<'_>],
        _server_name: &ServerName<'_>,
        _ocsp_response: &[u8],
        _now: UnixTime,
    ) -> Result<rustls::client::danger::ServerCertVerified, rustls::Error> {
        Ok(rustls::client::danger::ServerCertVerified::assertion())
    }

    fn verify_tls12_signature(
        &self,
        message: &[u8],
        cert: &CertificateDer<'_>,
        dss: &rustls::DigitallySignedStruct,
    ) -> Result<rustls::client::danger::HandshakeSignatureValid, rustls::Error> {
        rustls::crypto::verify_tls12_signature(
            message,
            cert,
            dss,
            &self.0.signature_verification_algorithms,
        )
    }

    fn verify_tls13_signature(
        &self,
        message: &[u8],
        cert: &CertificateDer<'_>,
        dss: &rustls::DigitallySignedStruct,
    ) -> Result<rustls::client::danger::HandshakeSignatureValid, rustls::Error> {
        rustls::crypto::verify_tls13_signature(
            message,
            cert,
            dss,
            &self.0.signature_verification_algorithms,
        )
    }

    fn supported_verify_schemes(&self) -> Vec<rustls::SignatureScheme> {
        self.0.signature_verification_algorithms.supported_schemes()
    }
}

fn reserve_loopback_addr() -> String {
    let listener = TcpListener::bind("127.0.0.1:0").expect("listener should bind");
    let addr = listener.local_addr().expect("local addr should resolve");
    drop(listener);
    addr.to_string()
}

fn send_http_request(addr: &str, method: &str, path: &str, body: &str) -> String {
    send_http_request_with_headers(addr, method, path, body, &[])
}

fn send_http_request_with_headers(
    addr: &str,
    method: &str,
    path: &str,
    body: &str,
    headers: &[(&str, &str)],
) -> String {
    let mut stream = TcpStream::connect(addr).expect("endpoint should accept connections");
    stream
        .set_read_timeout(Some(Duration::from_secs(2)))
        .expect("read timeout should be configurable");
    let mut header_lines = String::new();
    for (name, value) in headers {
        header_lines.push_str(name);
        header_lines.push_str(": ");
        header_lines.push_str(value);
        header_lines.push_str("\r\n");
    }
    let request = format!(
        "{method} {path} HTTP/1.1\r\nHost: {addr}\r\nContent-Type: application/json\r\n{}Content-Length: {}\r\nConnection: close\r\n\r\n{}",
        header_lines,
        body.len(),
        body
    );
    stream
        .write_all(request.as_bytes())
        .expect("request should write");
    let mut response = String::new();
    let mut chunk = [0_u8; 1024];
    loop {
        match stream.read(&mut chunk) {
            Ok(0) => break,
            Ok(read_count) => {
                response.push_str(
                    std::str::from_utf8(&chunk[..read_count]).expect("response must be utf-8"),
                );
            }
            Err(error) if matches!(error.kind(), ErrorKind::WouldBlock | ErrorKind::TimedOut) => {
                break;
            }
            Err(error) => panic!("response should be readable: {error}"),
        }
    }
    response
}

fn send_https_request_with_headers(
    addr: &str,
    method: &str,
    path: &str,
    body: &str,
    headers: &[(&str, &str)],
    _root_cert_pem: &str,
) -> String {
    let client_config = rustls::ClientConfig::builder()
        .dangerous()
        .with_custom_certificate_verifier(TestSkipServerVerification::new())
        .with_no_client_auth();
    let server_name = rustls::pki_types::ServerName::try_from("localhost".to_owned())
        .expect("server name should parse");
    let connection = rustls::ClientConnection::new(Arc::new(client_config), server_name)
        .expect("tls client connection should initialize");
    let tcp_stream = TcpStream::connect(addr).expect("tls endpoint should accept connections");
    tcp_stream
        .set_read_timeout(Some(Duration::from_secs(2)))
        .expect("tls read timeout should be configurable");
    let mut stream = rustls::StreamOwned::new(connection, tcp_stream);

    let mut header_lines = String::new();
    for (name, value) in headers {
        header_lines.push_str(name);
        header_lines.push_str(": ");
        header_lines.push_str(value);
        header_lines.push_str("\r\n");
    }
    let request = format!(
        "{method} {path} HTTP/1.1\r\nHost: localhost\r\nContent-Type: application/json\r\n{}Content-Length: {}\r\nConnection: close\r\n\r\n{}",
        header_lines,
        body.len(),
        body
    );
    stream
        .write_all(request.as_bytes())
        .expect("tls request should write");
    stream.flush().expect("tls request should flush");

    let mut response = String::new();
    let mut chunk = [0_u8; 1024];
    loop {
        match stream.read(&mut chunk) {
            Ok(0) => break,
            Ok(read_count) => {
                response.push_str(
                    std::str::from_utf8(&chunk[..read_count]).expect("response must be utf-8"),
                );
            }
            Err(error) if matches!(error.kind(), ErrorKind::WouldBlock | ErrorKind::TimedOut) => {
                break;
            }
            Err(error) => panic!("tls response should be readable: {error}"),
        }
    }
    response
}

pub(super) fn write_test_service_api_tls_materials() -> (String, String) {
    let entropy = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock should be monotonic")
        .as_nanos();
    let base = std::env::temp_dir().join(format!(
        "kamn-node-service-api-tls-{}-{entropy}",
        std::process::id()
    ));
    fs::create_dir_all(&base).expect("temporary tls directory should be created");
    let cert_path = base.join("server-cert.pem");
    let key_path = base.join("server-key.pem");
    fs::write(&cert_path, TEST_SERVICE_API_TLS_CERT_PEM.as_bytes())
        .expect("test cert should write");
    fs::write(&key_path, TEST_SERVICE_API_TLS_KEY_PEM.as_bytes()).expect("test key should write");
    (
        cert_path.to_string_lossy().to_string(),
        key_path.to_string_lossy().to_string(),
    )
}

fn send_websocket_upgrade_request(addr: &str, path: &str, headers: &[(&str, &str)]) -> Vec<u8> {
    send_websocket_upgrade_request_with_version(addr, path, "13", headers)
}

fn send_websocket_upgrade_request_with_version(
    addr: &str,
    path: &str,
    websocket_version: &str,
    headers: &[(&str, &str)],
) -> Vec<u8> {
    let mut stream = TcpStream::connect(addr).expect("endpoint should accept websocket connection");
    stream
        .set_read_timeout(Some(Duration::from_secs(2)))
        .expect("websocket read timeout should be configurable");
    let mut header_lines = String::new();
    for (name, value) in headers {
        header_lines.push_str(name);
        header_lines.push_str(": ");
        header_lines.push_str(value);
        header_lines.push_str("\r\n");
    }
    let request = format!(
        "GET {path} HTTP/1.1\r\nHost: {addr}\r\nConnection: Upgrade\r\nUpgrade: websocket\r\nSec-WebSocket-Key: test-kamn-key\r\nSec-WebSocket-Version: {websocket_version}\r\n{}Content-Length: 0\r\n\r\n",
        header_lines
    );
    stream
        .write_all(request.as_bytes())
        .expect("websocket upgrade request should write");
    let mut response = Vec::new();
    let mut chunk = [0_u8; 1024];
    loop {
        match stream.read(&mut chunk) {
            Ok(0) => break,
            Ok(read_count) => response.extend_from_slice(&chunk[..read_count]),
            Err(error) if matches!(error.kind(), ErrorKind::WouldBlock | ErrorKind::TimedOut) => {
                break;
            }
            Err(error) => panic!("websocket response should be readable: {error}"),
        }
    }
    response
}

fn parse_http_content_length(response_head: &str) -> usize {
    for line in response_head.lines() {
        if let Some((name, value)) = line.split_once(':') {
            if name.eq_ignore_ascii_case("Content-Length") {
                return value.trim().parse::<usize>().unwrap_or(0);
            }
        }
    }
    0
}

fn extract_http_response_body(response: &str) -> &str {
    response
        .split_once("\r\n\r\n")
        .map(|(_, body)| body)
        .unwrap_or("")
}

fn parse_error_envelope(body: &str) -> ServiceApiErrorEnvelope {
    serde_json::from_str(body).expect("error payload should deserialize")
}

fn parse_error_envelope_from_http_response(response: &str) -> ServiceApiErrorEnvelope {
    parse_error_envelope(extract_http_response_body(response))
}

fn read_single_http_response(stream: &mut TcpStream) -> String {
    let mut response = Vec::new();
    let mut chunk = [0_u8; 1024];
    let mut header_end: Option<usize> = None;
    let mut expected_len: Option<usize> = None;

    loop {
        match stream.read(&mut chunk) {
            Ok(0) => break,
            Ok(read_count) => {
                response.extend_from_slice(&chunk[..read_count]);
                if header_end.is_none() {
                    header_end = response
                        .windows(4)
                        .position(|window| window == b"\r\n\r\n")
                        .map(|index| index + 4);
                    if let Some(header_end_index) = header_end {
                        let head = String::from_utf8_lossy(&response[..header_end_index]);
                        let content_len = parse_http_content_length(head.as_ref());
                        expected_len = Some(header_end_index + content_len);
                    }
                }
                if let Some(total) = expected_len {
                    if response.len() >= total {
                        break;
                    }
                }
            }
            Err(error) if matches!(error.kind(), ErrorKind::WouldBlock | ErrorKind::TimedOut) => {
                break;
            }
            Err(error) => panic!("response should be readable: {error}"),
        }
    }

    String::from_utf8(response).expect("http response should be utf-8")
}

fn parse_websocket_response(response: &[u8]) -> (String, String) {
    let header_end = response
        .windows(4)
        .position(|window| window == b"\r\n\r\n")
        .map(|index| index + 4)
        .expect("websocket response should include header terminator");
    let header = std::str::from_utf8(&response[..header_end])
        .expect("websocket header should be utf-8")
        .to_owned();
    let frame = &response[header_end..];
    assert!(
        frame.len() >= 2,
        "websocket response should include at least one frame"
    );
    assert_eq!(
        frame[0], 0x81,
        "expected single-frame text websocket opcode"
    );
    assert_eq!(
        frame[1] & 0x80,
        0,
        "server websocket frame must be unmasked"
    );
    let mut payload_index = 2;
    let payload_len = match frame[1] & 0x7f {
        value @ 0..=125 => value as usize,
        126 => {
            assert!(
                frame.len() >= 4,
                "websocket frame extended payload length must be available"
            );
            payload_index = 4;
            u16::from_be_bytes([frame[2], frame[3]]) as usize
        }
        127 => {
            assert!(
                frame.len() >= 10,
                "websocket frame 64-bit payload length must be available"
            );
            payload_index = 10;
            u64::from_be_bytes([
                frame[2], frame[3], frame[4], frame[5], frame[6], frame[7], frame[8], frame[9],
            ]) as usize
        }
        _ => unreachable!("websocket payload marker is constrained to 7 bits"),
    };
    assert!(
        frame.len() >= payload_len + payload_index,
        "websocket frame payload length must be available"
    );
    let payload = std::str::from_utf8(&frame[payload_index..payload_index + payload_len])
        .expect("websocket payload should be utf-8")
        .to_owned();
    (header, payload)
}

fn wait_for_endpoint_ready(addr: &str) {
    let deadline = Instant::now() + Duration::from_secs(2);
    while Instant::now() < deadline {
        if TcpStream::connect(addr).is_ok() {
            return;
        }
        thread::sleep(Duration::from_millis(10));
    }
    panic!("endpoint did not become ready within timeout");
}

struct ServiceApiTestEnvGuards {
    _env_lock: MutexGuard<'static, ()>,
    _tls_mode_guard: EnvVarGuard,
    _tls_cert_guard: EnvVarGuard,
    _tls_key_guard: EnvVarGuard,
    _log_level_guard: EnvVarGuard,
    _log_format_guard: EnvVarGuard,
    _chain_id_guard: EnvVarGuard,
    _sync_mode_guard: EnvVarGuard,
}

fn acquire_service_api_test_env() -> ServiceApiTestEnvGuards {
    let env_lock = lock_signer_env_guard();
    ServiceApiTestEnvGuards {
        _env_lock: env_lock,
        _tls_mode_guard: EnvVarGuard::set("KAMN_SERVICE_API_TLS_MODE", None),
        _tls_cert_guard: EnvVarGuard::set("KAMN_SERVICE_API_TLS_CERT_FILE", None),
        _tls_key_guard: EnvVarGuard::set("KAMN_SERVICE_API_TLS_KEY_FILE", None),
        _log_level_guard: EnvVarGuard::set("KAMN_NODE_LOG_LEVEL", None),
        _log_format_guard: EnvVarGuard::set("KAMN_NODE_LOG_FORMAT", None),
        _chain_id_guard: EnvVarGuard::set("KAMN_NODE_CHAIN_ID", None),
        _sync_mode_guard: EnvVarGuard::set("KAMN_NODE_SYNC_MODE", None),
    }
}

#[test]
fn regression_service_api_env_lock_recovers_from_signer_lock_poison() {
    // Regression: #5199
    let _ = std::panic::catch_unwind(|| {
        let _lock = lock_signer_env_guard();
        panic!("intentional signer env lock poison");
    });
    let _env = acquire_service_api_test_env();
}

#[test]
fn functional_service_api_endpoint_renders_required_route_contracts() {
    let _env = acquire_service_api_test_env();
    let parsed = parse_args(vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
        "--runtime-mode".to_owned(),
        "api".to_owned(),
        "--api-bind".to_owned(),
        "127.0.0.1:34051".to_owned(),
    ])
    .expect("api args should parse");
    let report = execute(parsed).expect("api execution should succeed");
    let snapshot = build_service_api_snapshot(&report);

    let send_response = render_service_api_endpoint_response(
        &snapshot,
        "POST",
        "/v1/messages/send",
        "{\"message\":\"hello\"}",
    );
    assert_eq!(send_response.status_code, 202);
    assert!(send_response.body.contains("\"message_id\":\"msg-local-"));

    let read_response =
        render_service_api_endpoint_response(&snapshot, "GET", "/v1/messages/msg-7", "");
    assert_eq!(read_response.status_code, 200);
    assert!(read_response.body.contains("\"status\":\"created\""));

    let channel_response = render_service_api_endpoint_response(
        &snapshot,
        "GET",
        "/v1/channels/channel-1/messages",
        "",
    );
    assert_eq!(channel_response.status_code, 200);

    let task_response =
        render_service_api_endpoint_response(&snapshot, "GET", "/v1/tasks/task-1", "");
    assert_eq!(task_response.status_code, 200);

    let agent_response = render_service_api_endpoint_response(
        &snapshot,
        "GET",
        "/v1/agents/kamn:did:agent:alpha",
        "",
    );
    assert_eq!(agent_response.status_code, 200);

    let health_response = render_service_api_endpoint_response(&snapshot, "GET", "/healthz", "");
    assert_eq!(health_response.status_code, 200);

    let metrics_response = render_service_api_endpoint_response(&snapshot, "GET", "/metrics", "");
    assert_eq!(metrics_response.status_code, 200);
    assert!(metrics_response
        .body
        .contains("kamn_service_api_observability_source{source=\"unknown\"} 1"));
    assert!(metrics_response
        .body
        .contains("kamn_service_api_observability_health{health=\"unknown\"} 0"));
    assert!(
        metrics_response
            .body
            .contains("kamn_service_api_observability_latency_p50_ms 0"),
        "metrics payload should publish runtime telemetry gauges even before daemon/kolme telemetry is available"
    );

    let ws_response = render_service_api_endpoint_response(&snapshot, "GET", "/v1/events/ws", "");
    assert_eq!(ws_response.status_code, 400);
    let ws_payload = parse_error_envelope(ws_response.body.as_str());
    assert_eq!(ws_payload.error, "bad-request");
    assert_eq!(
        ws_payload.reason_code,
        "service_api_websocket_upgrade_required"
    );
    assert!(ws_payload.message.contains("websocket upgrade required"));
}

#[test]
fn unit_service_api_endpoint_serde_payload_roundtrip_contracts() {
    let _env = acquire_service_api_test_env();
    let parsed = parse_args(vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
        "--runtime-mode".to_owned(),
        "api".to_owned(),
        "--api-bind".to_owned(),
        "127.0.0.1:34060".to_owned(),
    ])
    .expect("api args should parse");
    let report = execute(parsed).expect("api execution should succeed");
    let snapshot = build_service_api_snapshot(&report);

    let health = render_service_api_endpoint_response(&snapshot, "GET", "/healthz", "");
    let health_payload: ServiceApiHealthBody =
        parse_service_api_payload(health.body.as_str()).expect("health payload should deserialize");
    assert_eq!(health_payload.status, "ok");
    assert_eq!(health_payload.runtime_mode, "api");

    let send = render_service_api_endpoint_response(
        &snapshot,
        "POST",
        "/v1/messages/send",
        "{\"message\":\"serde\"}",
    );
    let send_payload: ServiceApiMessageCreateBody =
        parse_service_api_payload(send.body.as_str()).expect("send payload should deserialize");
    assert_eq!(send_payload.status, "created");
    assert!(send_payload.message_id.starts_with("msg-local-"));

    let channel = render_service_api_endpoint_response(
        &snapshot,
        "POST",
        "/v1/channels/create",
        "{\"name\":\"alpha\"}",
    );
    let channel_payload: ServiceApiChannelCreateBody =
        parse_service_api_payload(channel.body.as_str())
            .expect("channel payload should deserialize");
    assert_eq!(channel_payload.status, "created");

    let task = render_service_api_endpoint_response(
        &snapshot,
        "POST",
        "/v1/tasks/create",
        "{\"task\":\"x\"}",
    );
    let task_payload: ServiceApiTaskCreateBody =
        parse_service_api_payload(task.body.as_str()).expect("task payload should deserialize");
    assert_eq!(task_payload.state, "submitted");

    let agent = render_service_api_endpoint_response(
        &snapshot,
        "GET",
        "/v1/agents/kamn:did:agent:alpha",
        "",
    );
    let agent_payload: ServiceApiAgentGetBody =
        parse_service_api_payload(agent.body.as_str()).expect("agent payload should deserialize");
    assert_eq!(agent_payload.did, "kamn:did:agent:alpha");
    assert_eq!(agent_payload.reputation_score, 500);
}

#[test]
fn unit_service_api_endpoint_error_envelopes_use_reason_code_and_message_contracts() {
    let _env = acquire_service_api_test_env();
    let parsed = parse_args(vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
        "--runtime-mode".to_owned(),
        "api".to_owned(),
        "--api-bind".to_owned(),
        "127.0.0.1:34061".to_owned(),
    ])
    .expect("api args should parse");
    let report = execute(parsed).expect("api execution should succeed");
    let snapshot = build_service_api_snapshot(&report);

    let websocket_required =
        render_service_api_endpoint_response(&snapshot, "GET", "/v1/events/ws", "");
    assert_eq!(websocket_required.status_code, 400);
    let websocket_required_payload = parse_error_envelope(websocket_required.body.as_str());
    assert_eq!(websocket_required_payload.error, "bad-request");
    assert_eq!(
        websocket_required_payload.reason_code,
        "service_api_websocket_upgrade_required"
    );
    assert!(websocket_required_payload
        .message
        .contains("websocket upgrade required"));

    let method_not_allowed =
        render_service_api_endpoint_response(&snapshot, "DELETE", "/v1/messages/send", "");
    assert_eq!(method_not_allowed.status_code, 405);
    let method_not_allowed_payload = parse_error_envelope(method_not_allowed.body.as_str());
    assert_eq!(method_not_allowed_payload.error, "method-not-allowed");
    assert_eq!(
        method_not_allowed_payload.reason_code,
        "service_api_method_not_allowed"
    );
    assert!(method_not_allowed_payload
        .message
        .contains("method not allowed"));

    let not_found = render_service_api_endpoint_response(&snapshot, "GET", "/v1/nope", "");
    assert_eq!(not_found.status_code, 404);
    let not_found_payload = parse_error_envelope(not_found.body.as_str());
    assert_eq!(not_found_payload.error, "not-found");
    assert_eq!(not_found_payload.reason_code, "service_api_route_not_found");
    assert!(not_found_payload.message.contains("not found"));
}

#[test]
fn regression_service_api_payload_parse_reason_codes_fail_closed() {
    let _env = acquire_service_api_test_env();
    let syntax_error = parse_service_api_payload::<ServiceApiHealthBody>("{\"status\":\"ok\"");
    let syntax_reason = syntax_error.expect_err("invalid json syntax should fail closed");
    assert!(
        syntax_reason.starts_with("service_api_payload_json_syntax_invalid:"),
        "unexpected syntax reason marker: {syntax_reason}"
    );

    let structure_error = parse_service_api_payload::<ServiceApiHealthBody>(
        "{\"status\":\"ok\",\"runtime_mode\":\"api\"}",
    );
    let structure_reason =
        structure_error.expect_err("invalid payload structure should fail closed");
    assert!(
        structure_reason.starts_with("service_api_payload_structure_invalid:"),
        "unexpected structure reason marker: {structure_reason}"
    );
}

#[test]
fn integration_service_api_endpoint_serves_required_http_routes() {
    let _env = acquire_service_api_test_env();
    let parsed = parse_args(vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
        "--runtime-mode".to_owned(),
        "api".to_owned(),
        "--api-bind".to_owned(),
        "127.0.0.1:34052".to_owned(),
    ])
    .expect("api args should parse");
    let report = execute(parsed).expect("api execution should succeed");
    let snapshot = build_service_api_snapshot(&report);

    let bind_addr = reserve_loopback_addr();
    let endpoint_config = ServiceApiEndpointConfig {
        bind_addr: bind_addr.clone(),
        max_requests: 3,
        idle_timeout_ms: 2_000,
        body_limit_bytes: DEFAULT_SERVICE_API_BODY_LIMIT_BYTES,
        concurrency_limit: DEFAULT_SERVICE_API_CONCURRENCY_LIMIT,
        rate_limit_per_second: DEFAULT_SERVICE_API_RATE_LIMIT_PER_SECOND,
    };

    let server_snapshot = snapshot.clone();
    let server =
        thread::spawn(move || serve_service_api_endpoint(&endpoint_config, &server_snapshot));
    wait_for_endpoint_ready(bind_addr.as_str());

    let message_body = "{\"message\":\"hello\"}";
    let sender_did = "kamn:did:agent:test-client-1";
    let sender_nonce = 1_u64;
    let state_hash = format!(
        "service-api:{}:{}",
        snapshot.chain_id.as_str(),
        snapshot.chain_version.as_str()
    );
    let signature =
        baseline_signature_for_fields(sender_did, sender_nonce, state_hash.as_str(), message_body);
    let send_response = send_http_request_with_headers(
        bind_addr.as_str(),
        "POST",
        "/v1/messages/send",
        message_body,
        &[
            ("X-KAMN-Sender-DID", sender_did),
            ("X-KAMN-Request-Nonce", "1"),
            ("X-KAMN-Request-Signature", signature.as_str()),
        ],
    );
    let health_response = send_http_request(bind_addr.as_str(), "GET", "/healthz", "");
    let metrics_response = send_http_request(bind_addr.as_str(), "GET", "/metrics", "");

    assert!(send_response.contains("HTTP/1.1 202 Accepted"));
    assert!(send_response.contains("\"message_id\":\"msg-local-"));
    assert!(health_response.contains("HTTP/1.1 200 OK"));
    assert!(metrics_response.contains("HTTP/1.1 200 OK"));
    assert!(
        metrics_response.contains("kamn_service_api_observability_source{source=\"unknown\"} 1")
    );

    let server_result = server.join().expect("endpoint thread should complete");
    assert!(
        server_result.is_ok(),
        "service api endpoint should stop cleanly after configured request budget"
    );
}

#[test]
fn integration_service_api_endpoint_tls_mode_serves_required_https_routes() {
    let _env = acquire_service_api_test_env();
    let (cert_file, key_file) = write_test_service_api_tls_materials();
    let _tls_mode = EnvVarGuard::set("KAMN_SERVICE_API_TLS_MODE", Some("require"));
    let _tls_cert = EnvVarGuard::set("KAMN_SERVICE_API_TLS_CERT_FILE", Some(cert_file.as_str()));
    let _tls_key = EnvVarGuard::set("KAMN_SERVICE_API_TLS_KEY_FILE", Some(key_file.as_str()));

    let parsed = parse_args(vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
        "--runtime-mode".to_owned(),
        "api".to_owned(),
        "--api-bind".to_owned(),
        "127.0.0.1:34091".to_owned(),
    ])
    .expect("api args should parse");
    let report = execute(parsed).expect("api execution should succeed");
    let snapshot = build_service_api_snapshot(&report);

    let bind_addr = reserve_loopback_addr();
    let endpoint_config = ServiceApiEndpointConfig {
        bind_addr: bind_addr.clone(),
        max_requests: 2,
        idle_timeout_ms: 2_000,
        body_limit_bytes: DEFAULT_SERVICE_API_BODY_LIMIT_BYTES,
        concurrency_limit: DEFAULT_SERVICE_API_CONCURRENCY_LIMIT,
        rate_limit_per_second: DEFAULT_SERVICE_API_RATE_LIMIT_PER_SECOND,
    };

    let server_snapshot = snapshot.clone();
    let server =
        thread::spawn(move || serve_service_api_endpoint(&endpoint_config, &server_snapshot));
    wait_for_endpoint_ready(bind_addr.as_str());

    let health_response = send_https_request_with_headers(
        bind_addr.as_str(),
        "GET",
        "/healthz",
        "",
        &[],
        TEST_SERVICE_API_TLS_CERT_PEM,
    );
    let metrics_response = send_https_request_with_headers(
        bind_addr.as_str(),
        "GET",
        "/metrics",
        "",
        &[],
        TEST_SERVICE_API_TLS_CERT_PEM,
    );
    assert!(health_response.contains("HTTP/1.1 200 OK"));
    assert!(health_response.contains("\"status\":\"ok\""));
    assert!(metrics_response.contains("HTTP/1.1 200 OK"));
    assert!(
        metrics_response.contains("kamn_service_api_observability_source{source=\"unknown\"} 1")
    );

    let server_result = server.join().expect("endpoint thread should complete");
    assert!(
        server_result.is_ok(),
        "service api endpoint tls mode should stop cleanly after configured request budget"
    );
}

#[test]
fn regression_service_api_endpoint_tls_mode_rejects_missing_cert_file() {
    let _env = acquire_service_api_test_env();
    let missing_cert_file = std::env::temp_dir().join("kamn-service-api-missing-cert.pem");
    let missing_key_file = std::env::temp_dir().join("kamn-service-api-missing-key.pem");
    let _tls_mode = EnvVarGuard::set("KAMN_SERVICE_API_TLS_MODE", Some("require"));
    let _tls_cert = EnvVarGuard::set(
        "KAMN_SERVICE_API_TLS_CERT_FILE",
        Some(missing_cert_file.to_string_lossy().as_ref()),
    );
    let _tls_key = EnvVarGuard::set(
        "KAMN_SERVICE_API_TLS_KEY_FILE",
        Some(missing_key_file.to_string_lossy().as_ref()),
    );

    let parsed = parse_args(vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
        "--runtime-mode".to_owned(),
        "api".to_owned(),
        "--api-bind".to_owned(),
        "127.0.0.1:34101".to_owned(),
    ])
    .expect("api args should parse");
    let report = execute(parsed).expect("api execution should succeed");
    let snapshot = build_service_api_snapshot(&report);

    let bind_addr = reserve_loopback_addr();
    let endpoint_config = ServiceApiEndpointConfig {
        bind_addr: bind_addr.clone(),
        max_requests: 1,
        idle_timeout_ms: 2_000,
        body_limit_bytes: DEFAULT_SERVICE_API_BODY_LIMIT_BYTES,
        concurrency_limit: DEFAULT_SERVICE_API_CONCURRENCY_LIMIT,
        rate_limit_per_second: DEFAULT_SERVICE_API_RATE_LIMIT_PER_SECOND,
    };

    let error = serve_service_api_endpoint(&endpoint_config, &snapshot)
        .expect_err("missing tls cert should fail closed");
    assert!(
        error.contains("service api tls certificate file read failed"),
        "unexpected tls missing-cert marker: {error}"
    );
}

#[test]
fn integration_service_api_endpoint_http_response_bodies_match_serde_contracts() {
    let _env = acquire_service_api_test_env();
    let parsed = parse_args(vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
        "--runtime-mode".to_owned(),
        "api".to_owned(),
        "--api-bind".to_owned(),
        "127.0.0.1:34061".to_owned(),
    ])
    .expect("api args should parse");
    let report = execute(parsed).expect("api execution should succeed");
    let snapshot = build_service_api_snapshot(&report);

    let bind_addr = reserve_loopback_addr();
    let endpoint_config = ServiceApiEndpointConfig {
        bind_addr: bind_addr.clone(),
        max_requests: 2,
        idle_timeout_ms: 2_000,
        body_limit_bytes: DEFAULT_SERVICE_API_BODY_LIMIT_BYTES,
        concurrency_limit: DEFAULT_SERVICE_API_CONCURRENCY_LIMIT,
        rate_limit_per_second: DEFAULT_SERVICE_API_RATE_LIMIT_PER_SECOND,
    };

    let server_snapshot = snapshot.clone();
    let server =
        thread::spawn(move || serve_service_api_endpoint(&endpoint_config, &server_snapshot));
    wait_for_endpoint_ready(bind_addr.as_str());

    let message_body = "{\"message\":\"serde-live\"}";
    let sender_did = "kamn:did:agent:test-client-serde";
    let sender_nonce = 31_u64;
    let state_hash = format!(
        "service-api:{}:{}",
        snapshot.chain_id.as_str(),
        snapshot.chain_version.as_str()
    );
    let signature =
        baseline_signature_for_fields(sender_did, sender_nonce, state_hash.as_str(), message_body);
    let send_response = send_http_request_with_headers(
        bind_addr.as_str(),
        "POST",
        "/v1/messages/send",
        message_body,
        &[
            ("X-KAMN-Sender-DID", sender_did),
            ("X-KAMN-Request-Nonce", "31"),
            ("X-KAMN-Request-Signature", signature.as_str()),
        ],
    );
    let health_response = send_http_request(bind_addr.as_str(), "GET", "/healthz", "");
    assert!(send_response.contains("HTTP/1.1 202 Accepted"));
    assert!(health_response.contains("HTTP/1.1 200 OK"));

    let send_payload: ServiceApiMessageCreateBody =
        parse_service_api_payload(extract_http_response_body(send_response.as_str()))
            .expect("send payload should deserialize");
    assert_eq!(send_payload.status, "created");
    assert_eq!(send_payload.runtime_mode, "api");

    let health_payload: ServiceApiHealthBody =
        parse_service_api_payload(extract_http_response_body(health_response.as_str()))
            .expect("health payload should deserialize");
    assert_eq!(health_payload.status, "ok");
    assert_eq!(health_payload.runtime_mode, "api");

    let server_result = server.join().expect("endpoint thread should complete");
    assert!(
        server_result.is_ok(),
        "service api endpoint should stop cleanly after configured request budget"
    );
}

#[test]
fn integration_service_api_endpoint_supports_keep_alive_requests_on_single_connection() {
    let _env = acquire_service_api_test_env();
    let parsed = parse_args(vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
        "--runtime-mode".to_owned(),
        "api".to_owned(),
        "--api-bind".to_owned(),
        "127.0.0.1:34059".to_owned(),
    ])
    .expect("api args should parse");
    let report = execute(parsed).expect("api execution should succeed");
    let snapshot = build_service_api_snapshot(&report);

    let bind_addr = reserve_loopback_addr();
    let endpoint_config = ServiceApiEndpointConfig {
        bind_addr: bind_addr.clone(),
        max_requests: 2,
        idle_timeout_ms: 2_000,
        body_limit_bytes: DEFAULT_SERVICE_API_BODY_LIMIT_BYTES,
        concurrency_limit: DEFAULT_SERVICE_API_CONCURRENCY_LIMIT,
        rate_limit_per_second: DEFAULT_SERVICE_API_RATE_LIMIT_PER_SECOND,
    };
    let server_snapshot = snapshot.clone();
    let server =
        thread::spawn(move || serve_service_api_endpoint(&endpoint_config, &server_snapshot));
    wait_for_endpoint_ready(bind_addr.as_str());

    let mut stream = TcpStream::connect(bind_addr.as_str()).expect("endpoint should accept");
    stream
        .set_read_timeout(Some(Duration::from_secs(2)))
        .expect("read timeout should be configurable");

    let request_one = format!(
        "GET /healthz HTTP/1.1\r\nHost: {bind_addr}\r\nConnection: keep-alive\r\nContent-Length: 0\r\n\r\n"
    );
    stream
        .write_all(request_one.as_bytes())
        .expect("first request should write");
    let first_response = read_single_http_response(&mut stream);
    assert!(first_response.contains("HTTP/1.1 200 OK"));
    assert!(first_response.contains("\"status\":\"ok\""));

    let request_two = format!(
        "GET /metrics HTTP/1.1\r\nHost: {bind_addr}\r\nConnection: close\r\nContent-Length: 0\r\n\r\n"
    );
    stream
        .write_all(request_two.as_bytes())
        .expect("second request should write over keep-alive connection");
    let second_response = read_single_http_response(&mut stream);
    assert!(second_response.contains("HTTP/1.1 200 OK"));
    assert!(second_response.contains("kamn_service_api_observability_source{source=\"unknown\"} 1"));

    let server_result = server.join().expect("endpoint thread should complete");
    assert!(
        server_result.is_ok(),
        "service api endpoint should stop cleanly after keep-alive request budget"
    );
}

#[test]
fn functional_service_api_endpoint_emits_structured_ingress_correlation_markers() {
    let _env = acquire_service_api_test_env();
    let _level_guard = EnvVarGuard::set("KAMN_NODE_LOG_LEVEL", Some("info"));
    let _format_guard = EnvVarGuard::set("KAMN_NODE_LOG_FORMAT", Some("json"));
    let parsed = parse_args(vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
        "--runtime-mode".to_owned(),
        "api".to_owned(),
        "--api-bind".to_owned(),
        "127.0.0.1:34058".to_owned(),
    ])
    .expect("api args should parse");
    let report = execute(parsed).expect("api execution should succeed");
    let snapshot = build_service_api_snapshot(&report);

    let bind_addr = reserve_loopback_addr();
    let endpoint_config = ServiceApiEndpointConfig {
        bind_addr: bind_addr.clone(),
        max_requests: 1,
        idle_timeout_ms: 2_000,
        body_limit_bytes: DEFAULT_SERVICE_API_BODY_LIMIT_BYTES,
        concurrency_limit: DEFAULT_SERVICE_API_CONCURRENCY_LIMIT,
        rate_limit_per_second: DEFAULT_SERVICE_API_RATE_LIMIT_PER_SECOND,
    };
    let sender_did = "kamn:did:agent:test-client-correlation";
    let sender_nonce = 41_u64;
    let message_body = "{\"message\":\"structured-correlation\"}";
    let state_hash = format!(
        "service-api:{}:{}",
        snapshot.chain_id.as_str(),
        snapshot.chain_version.as_str()
    );
    let signature =
        baseline_signature_for_fields(sender_did, sender_nonce, state_hash.as_str(), message_body);
    let client_bind_addr = bind_addr.clone();
    let client = thread::spawn(move || {
        thread::sleep(Duration::from_millis(50));
        send_http_request_with_headers(
            client_bind_addr.as_str(),
            "POST",
            "/v1/messages/send",
            message_body,
            &[
                ("X-KAMN-Sender-DID", sender_did),
                ("X-KAMN-Request-Nonce", "41"),
                ("X-KAMN-Request-Signature", signature.as_str()),
            ],
        )
    });

    let (serve_result, captured_logs) =
        capture_test_logs(|| serve_service_api_endpoint(&endpoint_config, &snapshot));
    let response = client.join().expect("client request should complete");
    assert!(
        serve_result.is_ok(),
        "service api endpoint should serve one request"
    );
    assert!(response.contains("HTTP/1.1 202 Accepted"));

    let ingress_line = captured_logs
        .iter()
        .find(|line| line.contains("\"event\":\"service.api.request.received\""))
        .expect("service api ingress should emit received marker");
    let outcome_line = captured_logs
        .iter()
        .find(|line| line.contains("\"event\":\"service.api.request.outcome\""))
        .expect("service api ingress should emit outcome marker");
    let ingress_correlation = extract_json_string_field(ingress_line, "correlation_id")
        .expect("ingress marker should include correlation id");
    let outcome_correlation = extract_json_string_field(outcome_line, "correlation_id")
        .expect("outcome marker should include correlation id");
    assert_eq!(ingress_correlation, outcome_correlation);
    assert_eq!(
        extract_json_string_field(ingress_line, "method").as_deref(),
        Some("POST")
    );
    assert_eq!(
        extract_json_string_field(ingress_line, "path").as_deref(),
        Some("/v1/messages/send")
    );
    assert_eq!(
        extract_json_string_field(outcome_line, "status_code").as_deref(),
        Some("202")
    );
}

#[test]
fn unit_service_api_endpoint_metrics_use_runtime_observability_when_present() {
    let _env = acquire_service_api_test_env();
    let parsed = parse_args(vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
        "--runtime-mode".to_owned(),
        "daemon".to_owned(),
        "--daemon-max-ticks".to_owned(),
        "3".to_owned(),
        "--daemon-tick-interval-ms".to_owned(),
        "25".to_owned(),
    ])
    .expect("daemon args should parse");
    let report = execute(parsed).expect("daemon execution should succeed");
    let snapshot = build_service_api_snapshot(&report);

    assert_eq!(snapshot.observability_source, "daemon");
    assert_eq!(snapshot.observability_health, "healthy");
    let metrics_response = render_service_api_endpoint_response(&snapshot, "GET", "/metrics", "");
    assert_eq!(metrics_response.status_code, 200);
    assert!(metrics_response
        .body
        .contains("kamn_service_api_observability_source{source=\"daemon\"} 1"));
    assert!(metrics_response
        .body
        .contains("kamn_service_api_observability_health{health=\"healthy\"} 1"));
}

#[test]
fn integration_service_api_endpoint_rejects_missing_request_auth_headers() {
    let _env = acquire_service_api_test_env();
    let parsed = parse_args(vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
        "--runtime-mode".to_owned(),
        "api".to_owned(),
        "--api-bind".to_owned(),
        "127.0.0.1:34053".to_owned(),
    ])
    .expect("api args should parse");
    let report = execute(parsed).expect("api execution should succeed");
    let snapshot = build_service_api_snapshot(&report);

    let bind_addr = reserve_loopback_addr();
    let endpoint_config = ServiceApiEndpointConfig {
        bind_addr: bind_addr.clone(),
        max_requests: 1,
        idle_timeout_ms: 2_000,
        body_limit_bytes: DEFAULT_SERVICE_API_BODY_LIMIT_BYTES,
        concurrency_limit: DEFAULT_SERVICE_API_CONCURRENCY_LIMIT,
        rate_limit_per_second: DEFAULT_SERVICE_API_RATE_LIMIT_PER_SECOND,
    };
    let server_snapshot = snapshot.clone();
    let server =
        thread::spawn(move || serve_service_api_endpoint(&endpoint_config, &server_snapshot));
    wait_for_endpoint_ready(bind_addr.as_str());

    let unauth_response = send_http_request(
        bind_addr.as_str(),
        "POST",
        "/v1/messages/send",
        "{\"message\":\"hello\"}",
    );
    assert!(unauth_response.contains("HTTP/1.1 401 Unauthorized"));
    let unauth_payload = parse_error_envelope_from_http_response(unauth_response.as_str());
    assert_eq!(unauth_payload.error, "unauthorized");
    assert_eq!(
        unauth_payload.reason_code,
        "service_api_auth_sender_did_header_missing"
    );
    assert!(unauth_payload.message.contains("x-kamn-sender-did"));

    let server_result = server.join().expect("endpoint thread should complete");
    assert!(
        server_result.is_ok(),
        "service api endpoint should stop cleanly after configured request budget"
    );
}

#[test]
fn functional_service_api_endpoint_rejects_when_rate_limit_is_exceeded() {
    let _env = acquire_service_api_test_env();
    let parsed = parse_args(vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
        "--runtime-mode".to_owned(),
        "api".to_owned(),
        "--api-bind".to_owned(),
        "127.0.0.1:34062".to_owned(),
    ])
    .expect("api args should parse");
    let report = execute(parsed).expect("api execution should succeed");
    let snapshot = build_service_api_snapshot(&report);

    let bind_addr = reserve_loopback_addr();
    let endpoint_config = ServiceApiEndpointConfig {
        bind_addr: bind_addr.clone(),
        max_requests: 2,
        idle_timeout_ms: 2_000,
        body_limit_bytes: DEFAULT_SERVICE_API_BODY_LIMIT_BYTES,
        concurrency_limit: DEFAULT_SERVICE_API_CONCURRENCY_LIMIT,
        rate_limit_per_second: 1,
    };
    let server_snapshot = snapshot.clone();
    let server =
        thread::spawn(move || serve_service_api_endpoint(&endpoint_config, &server_snapshot));
    wait_for_endpoint_ready(bind_addr.as_str());

    let message_body = "{\"message\":\"rate-limit-check\"}";
    let sender_did = "kamn:did:agent:test-client-rate-limit";
    let state_hash = format!(
        "service-api:{}:{}",
        snapshot.chain_id.as_str(),
        snapshot.chain_version.as_str()
    );
    let first_signature =
        baseline_signature_for_fields(sender_did, 101, state_hash.as_str(), message_body);
    let second_signature =
        baseline_signature_for_fields(sender_did, 102, state_hash.as_str(), message_body);

    let first_response = send_http_request_with_headers(
        bind_addr.as_str(),
        "POST",
        "/v1/messages/send",
        message_body,
        &[
            ("X-KAMN-Sender-DID", sender_did),
            ("X-KAMN-Request-Nonce", "101"),
            ("X-KAMN-Request-Signature", first_signature.as_str()),
        ],
    );
    let second_response = send_http_request_with_headers(
        bind_addr.as_str(),
        "POST",
        "/v1/messages/send",
        message_body,
        &[
            ("X-KAMN-Sender-DID", sender_did),
            ("X-KAMN-Request-Nonce", "102"),
            ("X-KAMN-Request-Signature", second_signature.as_str()),
        ],
    );

    assert!(first_response.contains("HTTP/1.1 202 Accepted"));
    assert!(second_response.contains("HTTP/1.1 429 Too Many Requests"));
    let second_payload = parse_error_envelope_from_http_response(second_response.as_str());
    assert_eq!(second_payload.error, "too-many-requests");
    assert_eq!(
        second_payload.reason_code,
        "service_api_ingress_rate_limit_exceeded"
    );
    assert!(second_payload
        .message
        .contains("ingress rate limit exceeded"));

    let server_result = server.join().expect("endpoint thread should complete");
    assert!(
        server_result.is_ok(),
        "service api endpoint should stop cleanly after configured request budget"
    );
}

#[test]
fn functional_service_api_endpoint_applies_sender_anti_spam_throttle_and_suspension() {
    let _env = acquire_service_api_test_env();
    let parsed = parse_args(vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
        "--runtime-mode".to_owned(),
        "api".to_owned(),
        "--api-bind".to_owned(),
        "127.0.0.1:34065".to_owned(),
    ])
    .expect("api args should parse");
    let report = execute(parsed).expect("api execution should succeed");
    let snapshot = build_service_api_snapshot(&report);

    let bind_addr = reserve_loopback_addr();
    let endpoint_config = ServiceApiEndpointConfig {
        bind_addr: bind_addr.clone(),
        max_requests: 6,
        idle_timeout_ms: 3_000,
        body_limit_bytes: DEFAULT_SERVICE_API_BODY_LIMIT_BYTES,
        concurrency_limit: DEFAULT_SERVICE_API_CONCURRENCY_LIMIT,
        rate_limit_per_second: 1_000,
    };
    let server_snapshot = snapshot.clone();
    let server =
        thread::spawn(move || serve_service_api_endpoint(&endpoint_config, &server_snapshot));
    wait_for_endpoint_ready(bind_addr.as_str());

    let message_body = "{\"message\":\"anti-spam-check\"}";
    let sender_did = "kamn:did:agent:test-client-anti-spam";
    let state_hash = format!(
        "service-api:{}:{}",
        snapshot.chain_id.as_str(),
        snapshot.chain_version.as_str()
    );

    let mut responses = Vec::new();
    for nonce in 610_u64..616_u64 {
        let signature =
            baseline_signature_for_fields(sender_did, nonce, state_hash.as_str(), message_body);
        let nonce_text = nonce.to_string();
        responses.push(send_http_request_with_headers(
            bind_addr.as_str(),
            "POST",
            "/v1/messages/send",
            message_body,
            &[
                ("X-KAMN-Sender-DID", sender_did),
                ("X-KAMN-Request-Nonce", nonce_text.as_str()),
                ("X-KAMN-Request-Signature", signature.as_str()),
            ],
        ));
    }

    assert!(responses[0].contains("HTTP/1.1 202 Accepted"));
    assert!(responses[1].contains("HTTP/1.1 202 Accepted"));
    assert!(responses[2].contains("HTTP/1.1 202 Accepted"));

    assert!(responses[3].contains("HTTP/1.1 429 Too Many Requests"));
    let fourth_payload = parse_error_envelope_from_http_response(responses[3].as_str());
    assert_eq!(fourth_payload.error, "too-many-requests");
    assert_eq!(
        fourth_payload.reason_code,
        "service_api_ingress_sender_rate_limit_exceeded"
    );

    assert!(responses[4].contains("HTTP/1.1 429 Too Many Requests"));
    let fifth_payload = parse_error_envelope_from_http_response(responses[4].as_str());
    assert_eq!(fifth_payload.error, "too-many-requests");
    assert_eq!(
        fifth_payload.reason_code,
        "service_api_ingress_sender_rate_limit_exceeded"
    );

    assert!(responses[5].contains("HTTP/1.1 429 Too Many Requests"));
    let sixth_payload = parse_error_envelope_from_http_response(responses[5].as_str());
    assert_eq!(sixth_payload.error, "too-many-requests");
    assert_eq!(
        sixth_payload.reason_code,
        "service_api_ingress_sender_suspended"
    );

    let server_result = server.join().expect("endpoint thread should complete");
    assert!(
        server_result.is_ok(),
        "service api endpoint should stop cleanly after configured request budget"
    );
}

#[test]
fn integration_service_api_endpoint_sender_anti_spam_burst_rounds_remain_deterministic() {
    let _env = acquire_service_api_test_env();
    let parsed = parse_args(vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
        "--runtime-mode".to_owned(),
        "api".to_owned(),
        "--api-bind".to_owned(),
        "127.0.0.1:34069".to_owned(),
    ])
    .expect("api args should parse");
    let report = execute(parsed).expect("api execution should succeed");
    let snapshot = build_service_api_snapshot(&report);

    let rounds = 3_u64;
    let requests_per_round = 6_u64;
    let bind_addr = reserve_loopback_addr();
    let endpoint_config = ServiceApiEndpointConfig {
        bind_addr: bind_addr.clone(),
        max_requests: rounds * requests_per_round,
        idle_timeout_ms: 3_000,
        body_limit_bytes: DEFAULT_SERVICE_API_BODY_LIMIT_BYTES,
        concurrency_limit: DEFAULT_SERVICE_API_CONCURRENCY_LIMIT,
        rate_limit_per_second: 10_000,
    };
    let server_snapshot = snapshot.clone();
    let server =
        thread::spawn(move || serve_service_api_endpoint(&endpoint_config, &server_snapshot));
    wait_for_endpoint_ready(bind_addr.as_str());

    let state_hash = format!(
        "service-api:{}:{}",
        snapshot.chain_id.as_str(),
        snapshot.chain_version.as_str()
    );

    for round in 0..rounds {
        let sender_did = format!("kamn:did:agent:test-client-anti-spam-burst-{round}");
        let message_body = format!("{{\"message\":\"anti-spam-burst-round-{round}\"}}");
        let mut responses = Vec::new();

        for request_index in 0..requests_per_round {
            let nonce = 9_000 + round * requests_per_round + request_index;
            let signature = baseline_signature_for_fields(
                sender_did.as_str(),
                nonce,
                state_hash.as_str(),
                message_body.as_str(),
            );
            let nonce_text = nonce.to_string();
            responses.push(send_http_request_with_headers(
                bind_addr.as_str(),
                "POST",
                "/v1/messages/send",
                message_body.as_str(),
                &[
                    ("X-KAMN-Sender-DID", sender_did.as_str()),
                    ("X-KAMN-Request-Nonce", nonce_text.as_str()),
                    ("X-KAMN-Request-Signature", signature.as_str()),
                ],
            ));
        }

        assert!(
            responses[0].contains("HTTP/1.1 202 Accepted"),
            "round {round} first request should be accepted"
        );
        assert!(
            responses[1].contains("HTTP/1.1 202 Accepted"),
            "round {round} second request should be accepted"
        );
        assert!(
            responses[2].contains("HTTP/1.1 202 Accepted"),
            "round {round} third request should be accepted"
        );

        let fourth_payload = parse_error_envelope_from_http_response(responses[3].as_str());
        assert_eq!(fourth_payload.error, "too-many-requests");
        assert_eq!(
            fourth_payload.reason_code,
            "service_api_ingress_sender_rate_limit_exceeded"
        );

        let fifth_payload = parse_error_envelope_from_http_response(responses[4].as_str());
        assert_eq!(fifth_payload.error, "too-many-requests");
        assert_eq!(
            fifth_payload.reason_code,
            "service_api_ingress_sender_rate_limit_exceeded"
        );

        let sixth_payload = parse_error_envelope_from_http_response(responses[5].as_str());
        assert_eq!(sixth_payload.error, "too-many-requests");
        assert_eq!(
            sixth_payload.reason_code,
            "service_api_ingress_sender_suspended"
        );
    }

    let server_result = server.join().expect("endpoint thread should complete");
    assert!(
        server_result.is_ok(),
        "service api endpoint should stop cleanly after anti-spam burst rounds"
    );
}

#[test]
fn integration_service_api_endpoint_rejects_when_concurrency_limit_is_exceeded() {
    let _env = acquire_service_api_test_env();
    let parsed = parse_args(vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
        "--runtime-mode".to_owned(),
        "api".to_owned(),
        "--api-bind".to_owned(),
        "127.0.0.1:34063".to_owned(),
    ])
    .expect("api args should parse");
    let report = execute(parsed).expect("api execution should succeed");
    let snapshot = build_service_api_snapshot(&report);

    let bind_addr = reserve_loopback_addr();
    let worker_count = 6_usize;
    let endpoint_config = ServiceApiEndpointConfig {
        bind_addr: bind_addr.clone(),
        max_requests: worker_count as u64,
        idle_timeout_ms: 2_000,
        body_limit_bytes: DEFAULT_SERVICE_API_BODY_LIMIT_BYTES,
        concurrency_limit: 1,
        rate_limit_per_second: 1_000,
    };
    let server_snapshot = snapshot.clone();
    let server =
        thread::spawn(move || serve_service_api_endpoint(&endpoint_config, &server_snapshot));
    wait_for_endpoint_ready(bind_addr.as_str());

    let sender_did = "kamn:did:agent:test-client-concurrency-limit";
    let message_body = "{\"message\":\"concurrency-limit-check\"}";
    let state_hash = format!(
        "service-api:{}:{}",
        snapshot.chain_id.as_str(),
        snapshot.chain_version.as_str()
    );

    let barrier = Arc::new(Barrier::new(worker_count));
    let mut clients = Vec::with_capacity(worker_count);
    for request_index in 0..worker_count {
        let client_bind_addr = bind_addr.clone();
        let barrier = barrier.clone();
        let state_hash = state_hash.clone();
        clients.push(thread::spawn(move || {
            let nonce = 200 + request_index as u64;
            let signature =
                baseline_signature_for_fields(sender_did, nonce, state_hash.as_str(), message_body);
            let nonce_text = nonce.to_string();
            barrier.wait();
            send_http_request_with_headers(
                client_bind_addr.as_str(),
                "POST",
                "/v1/messages/send",
                message_body,
                &[
                    ("X-KAMN-Sender-DID", sender_did),
                    ("X-KAMN-Request-Nonce", nonce_text.as_str()),
                    ("X-KAMN-Request-Signature", signature.as_str()),
                ],
            )
        }));
    }

    let responses = clients
        .into_iter()
        .map(|client| client.join().expect("client request should complete"))
        .collect::<Vec<String>>();

    assert!(
        responses
            .iter()
            .any(|response| response.contains("HTTP/1.1 202 Accepted")),
        "expected at least one accepted request under constrained concurrency"
    );
    let concurrency_rejection = responses
        .iter()
        .find(|response| response.contains("HTTP/1.1 429 Too Many Requests"))
        .expect("expected at least one request to fail closed on concurrency limit");
    let rejection_payload = parse_error_envelope_from_http_response(concurrency_rejection);
    assert_eq!(rejection_payload.error, "too-many-requests");
    assert_eq!(
        rejection_payload.reason_code,
        "service_api_ingress_concurrency_limit_exceeded"
    );
    assert!(rejection_payload
        .message
        .contains("ingress concurrency limit exceeded"));

    let server_result = server.join().expect("endpoint thread should complete");
    assert!(
        server_result.is_ok(),
        "service api endpoint should stop cleanly after configured request budget"
    );
}

#[test]
fn integration_service_api_endpoint_concurrency_rejection_reason_stays_stable_under_bounded_bursts()
{
    let _env = acquire_service_api_test_env();
    let parsed = parse_args(vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
        "--runtime-mode".to_owned(),
        "api".to_owned(),
        "--api-bind".to_owned(),
        "127.0.0.1:34070".to_owned(),
    ])
    .expect("api args should parse");
    let report = execute(parsed).expect("api execution should succeed");
    let snapshot = build_service_api_snapshot(&report);

    let rounds = 2_u64;
    let worker_count = 8_usize;
    let bind_addr = reserve_loopback_addr();
    let endpoint_config = ServiceApiEndpointConfig {
        bind_addr: bind_addr.clone(),
        max_requests: rounds * worker_count as u64,
        idle_timeout_ms: 3_000,
        body_limit_bytes: DEFAULT_SERVICE_API_BODY_LIMIT_BYTES,
        concurrency_limit: 1,
        rate_limit_per_second: 10_000,
    };
    let server_snapshot = snapshot.clone();
    let server =
        thread::spawn(move || serve_service_api_endpoint(&endpoint_config, &server_snapshot));
    wait_for_endpoint_ready(bind_addr.as_str());

    let state_hash = format!(
        "service-api:{}:{}",
        snapshot.chain_id.as_str(),
        snapshot.chain_version.as_str()
    );

    for round in 0..rounds {
        let barrier = Arc::new(Barrier::new(worker_count));
        let mut clients = Vec::with_capacity(worker_count);
        for request_index in 0..worker_count {
            let client_bind_addr = bind_addr.clone();
            let barrier = barrier.clone();
            let state_hash = state_hash.clone();
            clients.push(thread::spawn(move || {
                let sender_did =
                    format!("kamn:did:agent:test-client-concurrency-burst-{round}-{request_index}");
                let body = format!(
                    "{{\"message\":\"concurrency-burst-round-{round}-request-{request_index}\"}}"
                );
                let nonce = 12_000 + round * worker_count as u64 + request_index as u64;
                let signature = baseline_signature_for_fields(
                    sender_did.as_str(),
                    nonce,
                    state_hash.as_str(),
                    body.as_str(),
                );
                let nonce_text = nonce.to_string();
                barrier.wait();
                send_http_request_with_headers(
                    client_bind_addr.as_str(),
                    "POST",
                    "/v1/messages/send",
                    body.as_str(),
                    &[
                        ("X-KAMN-Sender-DID", sender_did.as_str()),
                        ("X-KAMN-Request-Nonce", nonce_text.as_str()),
                        ("X-KAMN-Request-Signature", signature.as_str()),
                    ],
                )
            }));
        }

        let responses = clients
            .into_iter()
            .map(|client| client.join().expect("client request should complete"))
            .collect::<Vec<String>>();
        assert!(
            responses
                .iter()
                .any(|response| response.contains("HTTP/1.1 202 Accepted")),
            "round {round} expected at least one accepted request"
        );
        let rejection_payloads = responses
            .iter()
            .filter(|response| response.contains("HTTP/1.1 429 Too Many Requests"))
            .map(|response| parse_error_envelope_from_http_response(response))
            .collect::<Vec<ServiceApiErrorEnvelope>>();
        assert!(
            !rejection_payloads.is_empty(),
            "round {round} expected fail-closed concurrency rejections"
        );
        for payload in rejection_payloads {
            assert_eq!(payload.error, "too-many-requests");
            assert_eq!(
                payload.reason_code,
                "service_api_ingress_concurrency_limit_exceeded"
            );
            let projection = project_service_api_lifecycle_rejection(payload.reason_code.as_str())
                .expect("concurrency reason code should remain mappable");
            assert_eq!(projection.outcome, "concurrency-limit");
        }
    }

    let server_result = server.join().expect("endpoint thread should complete");
    assert!(
        server_result.is_ok(),
        "service api endpoint should stop cleanly after bounded concurrency bursts"
    );
}

#[test]
fn regression_service_api_endpoint_oversized_payload_maps_body_limit_reason_code() {
    let _env = acquire_service_api_test_env();
    let parsed = parse_args(vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
        "--runtime-mode".to_owned(),
        "api".to_owned(),
        "--api-bind".to_owned(),
        "127.0.0.1:34064".to_owned(),
    ])
    .expect("api args should parse");
    let report = execute(parsed).expect("api execution should succeed");
    let snapshot = build_service_api_snapshot(&report);

    let bind_addr = reserve_loopback_addr();
    let endpoint_config = ServiceApiEndpointConfig {
        bind_addr: bind_addr.clone(),
        max_requests: 1,
        idle_timeout_ms: 2_000,
        body_limit_bytes: 256,
        concurrency_limit: DEFAULT_SERVICE_API_CONCURRENCY_LIMIT,
        rate_limit_per_second: 1_000,
    };
    let server_snapshot = snapshot.clone();
    let server =
        thread::spawn(move || serve_service_api_endpoint(&endpoint_config, &server_snapshot));
    wait_for_endpoint_ready(bind_addr.as_str());

    let oversized_body = format!("{{\"message\":\"{}\"}}", "x".repeat(700));
    let sender_did = "kamn:did:agent:test-client-oversized";
    let nonce = 303_u64;
    let state_hash = format!(
        "service-api:{}:{}",
        snapshot.chain_id.as_str(),
        snapshot.chain_version.as_str()
    );
    let signature = baseline_signature_for_fields(
        sender_did,
        nonce,
        state_hash.as_str(),
        oversized_body.as_str(),
    );
    let response = send_http_request_with_headers(
        bind_addr.as_str(),
        "POST",
        "/v1/messages/send",
        oversized_body.as_str(),
        &[
            ("X-KAMN-Sender-DID", sender_did),
            ("X-KAMN-Request-Nonce", "303"),
            ("X-KAMN-Request-Signature", signature.as_str()),
        ],
    );
    assert!(response.contains("HTTP/1.1 400 Bad Request"));
    let payload = parse_error_envelope_from_http_response(response.as_str());
    assert_eq!(payload.error, "bad-request");
    assert_eq!(
        payload.reason_code,
        "service_api_ingress_body_size_limit_exceeded"
    );
    assert!(payload.message.contains("request body size limit exceeded"));

    let server_result = server.join().expect("endpoint thread should complete");
    assert!(
        server_result.is_ok(),
        "service api endpoint should stop cleanly after configured request budget"
    );
}

#[test]
fn regression_service_api_endpoint_rejects_replayed_request_nonce_for_sender() {
    let _env = acquire_service_api_test_env();
    let parsed = parse_args(vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
        "--runtime-mode".to_owned(),
        "api".to_owned(),
        "--api-bind".to_owned(),
        "127.0.0.1:34054".to_owned(),
    ])
    .expect("api args should parse");
    let report = execute(parsed).expect("api execution should succeed");
    let snapshot = build_service_api_snapshot(&report);

    let bind_addr = reserve_loopback_addr();
    let endpoint_config = ServiceApiEndpointConfig {
        bind_addr: bind_addr.clone(),
        max_requests: 2,
        idle_timeout_ms: 2_000,
        body_limit_bytes: DEFAULT_SERVICE_API_BODY_LIMIT_BYTES,
        concurrency_limit: DEFAULT_SERVICE_API_CONCURRENCY_LIMIT,
        rate_limit_per_second: DEFAULT_SERVICE_API_RATE_LIMIT_PER_SECOND,
    };
    let server_snapshot = snapshot.clone();
    let server =
        thread::spawn(move || serve_service_api_endpoint(&endpoint_config, &server_snapshot));
    wait_for_endpoint_ready(bind_addr.as_str());

    let message_body = "{\"message\":\"replay-check\"}";
    let sender_did = "kamn:did:agent:test-client-2";
    let sender_nonce = 7_u64;
    let state_hash = format!(
        "service-api:{}:{}",
        snapshot.chain_id.as_str(),
        snapshot.chain_version.as_str()
    );
    let signature =
        baseline_signature_for_fields(sender_did, sender_nonce, state_hash.as_str(), message_body);
    let auth_headers = [
        ("X-KAMN-Sender-DID", sender_did),
        ("X-KAMN-Request-Nonce", "7"),
        ("X-KAMN-Request-Signature", signature.as_str()),
    ];
    let first_response = send_http_request_with_headers(
        bind_addr.as_str(),
        "POST",
        "/v1/messages/send",
        message_body,
        &auth_headers,
    );
    let replay_response = send_http_request_with_headers(
        bind_addr.as_str(),
        "POST",
        "/v1/messages/send",
        message_body,
        &auth_headers,
    );

    assert!(first_response.contains("HTTP/1.1 202 Accepted"));
    assert!(replay_response.contains("HTTP/1.1 409 Conflict"));
    let replay_payload = parse_error_envelope_from_http_response(replay_response.as_str());
    assert_eq!(replay_payload.error, "replay");
    assert_eq!(
        replay_payload.reason_code,
        "service_api_auth_replay_nonce_detected"
    );
    assert!(replay_payload
        .message
        .contains("request nonce replay detected"));

    let server_result = server.join().expect("endpoint thread should complete");
    assert!(
        server_result.is_ok(),
        "service api endpoint should stop cleanly after configured request budget"
    );
}

#[test]
fn integration_service_api_endpoint_replay_rejection_remains_stable_with_anti_spam_enforcement() {
    let _env = acquire_service_api_test_env();
    let parsed = parse_args(vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
        "--runtime-mode".to_owned(),
        "api".to_owned(),
        "--api-bind".to_owned(),
        "127.0.0.1:34066".to_owned(),
    ])
    .expect("api args should parse");
    let report = execute(parsed).expect("api execution should succeed");
    let snapshot = build_service_api_snapshot(&report);

    let bind_addr = reserve_loopback_addr();
    let endpoint_config = ServiceApiEndpointConfig {
        bind_addr: bind_addr.clone(),
        max_requests: 3,
        idle_timeout_ms: 2_000,
        body_limit_bytes: DEFAULT_SERVICE_API_BODY_LIMIT_BYTES,
        concurrency_limit: DEFAULT_SERVICE_API_CONCURRENCY_LIMIT,
        rate_limit_per_second: 1_000,
    };
    let server_snapshot = snapshot.clone();
    let server =
        thread::spawn(move || serve_service_api_endpoint(&endpoint_config, &server_snapshot));
    wait_for_endpoint_ready(bind_addr.as_str());

    let message_body = "{\"message\":\"replay-anti-spam-matrix\"}";
    let sender_did = "kamn:did:agent:test-client-replay-anti-spam";
    let state_hash = format!(
        "service-api:{}:{}",
        snapshot.chain_id.as_str(),
        snapshot.chain_version.as_str()
    );
    let signature_nonce_one =
        baseline_signature_for_fields(sender_did, 701, state_hash.as_str(), message_body);
    let signature_nonce_two =
        baseline_signature_for_fields(sender_did, 702, state_hash.as_str(), message_body);

    let first_response = send_http_request_with_headers(
        bind_addr.as_str(),
        "POST",
        "/v1/messages/send",
        message_body,
        &[
            ("X-KAMN-Sender-DID", sender_did),
            ("X-KAMN-Request-Nonce", "701"),
            ("X-KAMN-Request-Signature", signature_nonce_one.as_str()),
        ],
    );
    let replay_response = send_http_request_with_headers(
        bind_addr.as_str(),
        "POST",
        "/v1/messages/send",
        message_body,
        &[
            ("X-KAMN-Sender-DID", sender_did),
            ("X-KAMN-Request-Nonce", "701"),
            ("X-KAMN-Request-Signature", signature_nonce_one.as_str()),
        ],
    );
    let fresh_nonce_response = send_http_request_with_headers(
        bind_addr.as_str(),
        "POST",
        "/v1/messages/send",
        message_body,
        &[
            ("X-KAMN-Sender-DID", sender_did),
            ("X-KAMN-Request-Nonce", "702"),
            ("X-KAMN-Request-Signature", signature_nonce_two.as_str()),
        ],
    );

    assert!(first_response.contains("HTTP/1.1 202 Accepted"));
    assert!(replay_response.contains("HTTP/1.1 409 Conflict"));
    let replay_payload = parse_error_envelope_from_http_response(replay_response.as_str());
    assert_eq!(replay_payload.error, "replay");
    assert_eq!(
        replay_payload.reason_code,
        "service_api_auth_replay_nonce_detected"
    );

    assert!(
        fresh_nonce_response.contains("HTTP/1.1 202 Accepted"),
        "replay rejection should not force anti-spam limiter rejection for the next valid nonce"
    );

    let server_result = server.join().expect("endpoint thread should complete");
    assert!(
        server_result.is_ok(),
        "service api endpoint should stop cleanly after configured request budget"
    );
}

#[test]
fn regression_service_api_endpoint_replay_duplicate_sequence_reason_ordering_stays_stable() {
    // Regression: #5283
    let _env = acquire_service_api_test_env();
    let parsed = parse_args(vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
        "--runtime-mode".to_owned(),
        "api".to_owned(),
        "--api-bind".to_owned(),
        "127.0.0.1:34071".to_owned(),
    ])
    .expect("api args should parse");
    let report = execute(parsed).expect("api execution should succeed");
    let snapshot = build_service_api_snapshot(&report);

    let rounds = 3_u64;
    let bind_addr = reserve_loopback_addr();
    let endpoint_config = ServiceApiEndpointConfig {
        bind_addr: bind_addr.clone(),
        max_requests: rounds * 2,
        idle_timeout_ms: 2_000,
        body_limit_bytes: DEFAULT_SERVICE_API_BODY_LIMIT_BYTES,
        concurrency_limit: DEFAULT_SERVICE_API_CONCURRENCY_LIMIT,
        rate_limit_per_second: DEFAULT_SERVICE_API_RATE_LIMIT_PER_SECOND,
    };
    let server_snapshot = snapshot.clone();
    let server =
        thread::spawn(move || serve_service_api_endpoint(&endpoint_config, &server_snapshot));
    wait_for_endpoint_ready(bind_addr.as_str());

    let sender_did = "kamn:did:agent:test-client-replay-duplicate-sequence";
    let state_hash = format!(
        "service-api:{}:{}",
        snapshot.chain_id.as_str(),
        snapshot.chain_version.as_str()
    );
    let mut observed = Vec::new();

    for round in 0..rounds {
        let body = format!("{{\"message\":\"replay-duplicate-round-{round}\"}}");
        let nonce = 13_000 + round;
        let signature =
            baseline_signature_for_fields(sender_did, nonce, state_hash.as_str(), body.as_str());
        let nonce_text = nonce.to_string();
        let headers = [
            ("X-KAMN-Sender-DID", sender_did),
            ("X-KAMN-Request-Nonce", nonce_text.as_str()),
            ("X-KAMN-Request-Signature", signature.as_str()),
        ];

        let first = send_http_request_with_headers(
            bind_addr.as_str(),
            "POST",
            "/v1/messages/send",
            body.as_str(),
            &headers,
        );
        assert!(
            first.contains("HTTP/1.1 202 Accepted"),
            "round {round} initial request should be accepted"
        );
        observed.push("accepted".to_owned());

        let replay = send_http_request_with_headers(
            bind_addr.as_str(),
            "POST",
            "/v1/messages/send",
            body.as_str(),
            &headers,
        );
        assert!(
            replay.contains("HTTP/1.1 409 Conflict"),
            "round {round} replay request should fail closed"
        );
        let replay_payload = parse_error_envelope_from_http_response(replay.as_str());
        observed.push(replay_payload.reason_code);
    }

    let expected = vec![
        "accepted".to_owned(),
        "service_api_auth_replay_nonce_detected".to_owned(),
        "accepted".to_owned(),
        "service_api_auth_replay_nonce_detected".to_owned(),
        "accepted".to_owned(),
        "service_api_auth_replay_nonce_detected".to_owned(),
    ];
    assert_eq!(observed, expected);

    let server_result = server.join().expect("endpoint thread should complete");
    assert!(
        server_result.is_ok(),
        "service api endpoint should stop cleanly after replay duplicate ordering regression"
    );
}

#[test]
fn integration_service_api_endpoint_websocket_upgrade_streams_state_transition_event() {
    let _env = acquire_service_api_test_env();
    let parsed = parse_args(vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
        "--runtime-mode".to_owned(),
        "api".to_owned(),
        "--api-bind".to_owned(),
        "127.0.0.1:34055".to_owned(),
    ])
    .expect("api args should parse");
    let report = execute(parsed).expect("api execution should succeed");
    let snapshot = build_service_api_snapshot(&report);

    let bind_addr = reserve_loopback_addr();
    let endpoint_config = ServiceApiEndpointConfig {
        bind_addr: bind_addr.clone(),
        max_requests: 1,
        idle_timeout_ms: 2_000,
        body_limit_bytes: DEFAULT_SERVICE_API_BODY_LIMIT_BYTES,
        concurrency_limit: DEFAULT_SERVICE_API_CONCURRENCY_LIMIT,
        rate_limit_per_second: DEFAULT_SERVICE_API_RATE_LIMIT_PER_SECOND,
    };
    let server_snapshot = snapshot.clone();
    let server =
        thread::spawn(move || serve_service_api_endpoint(&endpoint_config, &server_snapshot));
    wait_for_endpoint_ready(bind_addr.as_str());

    let sender_did = "kamn:did:agent:ws-client-1";
    let nonce = 19_u64;
    let state_hash = format!(
        "service-api:{}:{}",
        snapshot.chain_id.as_str(),
        snapshot.chain_version.as_str()
    );
    let signature = baseline_signature_for_fields(sender_did, nonce, state_hash.as_str(), "");
    let response = send_websocket_upgrade_request(
        bind_addr.as_str(),
        "/v1/events/ws",
        &[
            ("X-KAMN-Sender-DID", sender_did),
            ("X-KAMN-Request-Nonce", "19"),
            ("X-KAMN-Request-Signature", signature.as_str()),
        ],
    );
    let (header, payload) = parse_websocket_response(response.as_slice());
    assert!(header.contains("HTTP/1.1 101 Switching Protocols"));
    let normalized_header = header.to_ascii_lowercase();
    assert!(normalized_header.contains("upgrade: websocket"));
    assert!(normalized_header.contains("x-kamn-websocket-contract: v1"));
    assert!(payload.contains("\"event\":\"state-transition\""));
    assert!(payload.contains("\"runtime_mode\":\"api\""));
    assert!(payload.contains("\"role\":\"processor\""));

    let server_result = server.join().expect("endpoint thread should complete");
    assert!(
        server_result.is_ok(),
        "service api endpoint should stop cleanly after websocket request budget"
    );
}

#[test]
fn integration_service_api_endpoint_websocket_presence_mode_streams_bridge_projection_event() {
    let _env = acquire_service_api_test_env();
    let parsed = parse_args(vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
        "--runtime-mode".to_owned(),
        "api".to_owned(),
        "--api-bind".to_owned(),
        "127.0.0.1:34058".to_owned(),
    ])
    .expect("api args should parse");
    let report = execute(parsed).expect("api execution should succeed");
    let snapshot = build_service_api_snapshot(&report);

    let bind_addr = reserve_loopback_addr();
    let endpoint_config = ServiceApiEndpointConfig {
        bind_addr: bind_addr.clone(),
        max_requests: 1,
        idle_timeout_ms: 2_000,
        body_limit_bytes: DEFAULT_SERVICE_API_BODY_LIMIT_BYTES,
        concurrency_limit: DEFAULT_SERVICE_API_CONCURRENCY_LIMIT,
        rate_limit_per_second: DEFAULT_SERVICE_API_RATE_LIMIT_PER_SECOND,
    };
    let server_snapshot = snapshot.clone();
    let server =
        thread::spawn(move || serve_service_api_endpoint(&endpoint_config, &server_snapshot));
    wait_for_endpoint_ready(bind_addr.as_str());

    let sender_did = "kamn:did:agent:ws-presence-client-1";
    let nonce = 31_u64;
    let state_hash = format!(
        "service-api:{}:{}",
        snapshot.chain_id.as_str(),
        snapshot.chain_version.as_str()
    );
    let signature = baseline_signature_for_fields(sender_did, nonce, state_hash.as_str(), "");
    let response = send_websocket_upgrade_request(
        bind_addr.as_str(),
        "/v1/events/ws",
        &[
            ("X-KAMN-Sender-DID", sender_did),
            ("X-KAMN-Request-Nonce", "31"),
            ("X-KAMN-Request-Signature", signature.as_str()),
            ("X-KAMN-Events-Mode", "presence"),
            ("X-KAMN-Presence-Owner-DID", "kamn:did:owner:alpha"),
            ("X-KAMN-Presence-Target-Owner-DID", "kamn:did:owner:alpha"),
            ("X-KAMN-Presence-Target-Agent-DID", sender_did),
            ("X-KAMN-Presence-Gateway-Node", "gateway-alpha"),
            ("X-KAMN-Presence-Connected-Since", "1709000000"),
            ("X-KAMN-Presence-Last-Heartbeat", "1709000005"),
            ("X-KAMN-Presence-Capabilities", "ws,notify"),
        ],
    );
    let (header, payload) = parse_websocket_response(response.as_slice());
    assert!(header.contains("HTTP/1.1 101 Switching Protocols"));
    let payload_json: Value =
        serde_json::from_str(payload.as_str()).expect("presence websocket payload should be json");
    assert_eq!(
        payload_json.get("event").and_then(Value::as_str),
        Some("m9.presence.snapshot")
    );
    assert_eq!(
        payload_json
            .get("transport_profile")
            .and_then(Value::as_str),
        Some("websocket")
    );
    assert_eq!(
        payload_json
            .get("requester_owner_did")
            .and_then(Value::as_str),
        Some("kamn:did:owner:alpha")
    );
    assert_eq!(
        payload_json
            .get("requester_agent_did")
            .and_then(Value::as_str),
        Some(sender_did)
    );
    assert_eq!(
        payload_json.get("target_owner_did").and_then(Value::as_str),
        Some("kamn:did:owner:alpha")
    );
    assert_eq!(
        payload_json.get("target_agent_did").and_then(Value::as_str),
        Some(sender_did)
    );
    assert_eq!(
        payload_json.get("visible").and_then(Value::as_bool),
        Some(true)
    );
    assert_eq!(
        payload_json
            .get("target_gateway_node")
            .and_then(Value::as_str),
        Some("gateway-alpha")
    );
    assert_eq!(
        payload_json
            .get("target_last_heartbeat_epoch_seconds")
            .and_then(Value::as_u64),
        Some(1_709_000_005)
    );
    assert_eq!(
        payload_json.get("reason_code").and_then(Value::as_str),
        Some("m9_gateway_presence_visible")
    );

    let server_result = server.join().expect("endpoint thread should complete");
    assert!(
        server_result.is_ok(),
        "service api endpoint should stop cleanly after websocket presence request budget"
    );
}

#[test]
fn regression_service_api_endpoint_websocket_presence_mode_rejects_unsupported_mode() {
    let _env = acquire_service_api_test_env();
    let parsed = parse_args(vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
        "--runtime-mode".to_owned(),
        "api".to_owned(),
        "--api-bind".to_owned(),
        "127.0.0.1:34059".to_owned(),
    ])
    .expect("api args should parse");
    let report = execute(parsed).expect("api execution should succeed");
    let snapshot = build_service_api_snapshot(&report);

    let bind_addr = reserve_loopback_addr();
    let endpoint_config = ServiceApiEndpointConfig {
        bind_addr: bind_addr.clone(),
        max_requests: 1,
        idle_timeout_ms: 2_000,
        body_limit_bytes: DEFAULT_SERVICE_API_BODY_LIMIT_BYTES,
        concurrency_limit: DEFAULT_SERVICE_API_CONCURRENCY_LIMIT,
        rate_limit_per_second: DEFAULT_SERVICE_API_RATE_LIMIT_PER_SECOND,
    };
    let server_snapshot = snapshot.clone();
    let server =
        thread::spawn(move || serve_service_api_endpoint(&endpoint_config, &server_snapshot));
    wait_for_endpoint_ready(bind_addr.as_str());

    let sender_did = "kamn:did:agent:ws-presence-client-unsupported";
    let nonce = 37_u64;
    let state_hash = format!(
        "service-api:{}:{}",
        snapshot.chain_id.as_str(),
        snapshot.chain_version.as_str()
    );
    let signature = baseline_signature_for_fields(sender_did, nonce, state_hash.as_str(), "");
    let response = send_websocket_upgrade_request(
        bind_addr.as_str(),
        "/v1/events/ws",
        &[
            ("X-KAMN-Sender-DID", sender_did),
            ("X-KAMN-Request-Nonce", "37"),
            ("X-KAMN-Request-Signature", signature.as_str()),
            ("X-KAMN-Events-Mode", "presence-v2"),
        ],
    );
    let response_text =
        String::from_utf8(response).expect("unsupported mode response should be utf-8");
    assert!(response_text.contains("HTTP/1.1 400 Bad Request"));
    let payload = parse_error_envelope_from_http_response(response_text.as_str());
    assert_eq!(payload.error, "bad-request");
    assert_eq!(payload.reason_code, "service_api_ws_events_mode_invalid");

    let server_result = server.join().expect("endpoint thread should complete");
    assert!(
        server_result.is_ok(),
        "service api endpoint should stop cleanly after unsupported websocket mode rejection"
    );
}

#[test]
fn regression_service_api_endpoint_websocket_presence_mode_rejects_missing_owner_header() {
    let _env = acquire_service_api_test_env();
    let parsed = parse_args(vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
        "--runtime-mode".to_owned(),
        "api".to_owned(),
        "--api-bind".to_owned(),
        "127.0.0.1:34060".to_owned(),
    ])
    .expect("api args should parse");
    let report = execute(parsed).expect("api execution should succeed");
    let snapshot = build_service_api_snapshot(&report);

    let bind_addr = reserve_loopback_addr();
    let endpoint_config = ServiceApiEndpointConfig {
        bind_addr: bind_addr.clone(),
        max_requests: 1,
        idle_timeout_ms: 2_000,
        body_limit_bytes: DEFAULT_SERVICE_API_BODY_LIMIT_BYTES,
        concurrency_limit: DEFAULT_SERVICE_API_CONCURRENCY_LIMIT,
        rate_limit_per_second: DEFAULT_SERVICE_API_RATE_LIMIT_PER_SECOND,
    };
    let server_snapshot = snapshot.clone();
    let server =
        thread::spawn(move || serve_service_api_endpoint(&endpoint_config, &server_snapshot));
    wait_for_endpoint_ready(bind_addr.as_str());

    let sender_did = "kamn:did:agent:ws-presence-client-missing-owner";
    let nonce = 43_u64;
    let state_hash = format!(
        "service-api:{}:{}",
        snapshot.chain_id.as_str(),
        snapshot.chain_version.as_str()
    );
    let signature = baseline_signature_for_fields(sender_did, nonce, state_hash.as_str(), "");
    let response = send_websocket_upgrade_request(
        bind_addr.as_str(),
        "/v1/events/ws",
        &[
            ("X-KAMN-Sender-DID", sender_did),
            ("X-KAMN-Request-Nonce", "43"),
            ("X-KAMN-Request-Signature", signature.as_str()),
            ("X-KAMN-Events-Mode", "presence"),
            ("X-KAMN-Presence-Target-Agent-DID", sender_did),
        ],
    );
    let response_text =
        String::from_utf8(response).expect("missing-owner response should be utf-8");
    assert!(response_text.contains("HTTP/1.1 400 Bad Request"));
    let payload = parse_error_envelope_from_http_response(response_text.as_str());
    assert_eq!(payload.error, "bad-request");
    assert_eq!(
        payload.reason_code,
        "service_api_ws_presence_owner_did_header_missing"
    );

    let server_result = server.join().expect("endpoint thread should complete");
    assert!(
        server_result.is_ok(),
        "service api endpoint should stop cleanly after missing-owner websocket rejection"
    );
}

#[test]
fn regression_service_api_endpoint_websocket_presence_mode_rejects_cross_owner_scope() {
    let _env = acquire_service_api_test_env();
    let parsed = parse_args(vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
        "--runtime-mode".to_owned(),
        "api".to_owned(),
        "--api-bind".to_owned(),
        "127.0.0.1:34061".to_owned(),
    ])
    .expect("api args should parse");
    let report = execute(parsed).expect("api execution should succeed");
    let snapshot = build_service_api_snapshot(&report);

    let bind_addr = reserve_loopback_addr();
    let endpoint_config = ServiceApiEndpointConfig {
        bind_addr: bind_addr.clone(),
        max_requests: 1,
        idle_timeout_ms: 2_000,
        body_limit_bytes: DEFAULT_SERVICE_API_BODY_LIMIT_BYTES,
        concurrency_limit: DEFAULT_SERVICE_API_CONCURRENCY_LIMIT,
        rate_limit_per_second: DEFAULT_SERVICE_API_RATE_LIMIT_PER_SECOND,
    };
    let server_snapshot = snapshot.clone();
    let server =
        thread::spawn(move || serve_service_api_endpoint(&endpoint_config, &server_snapshot));
    wait_for_endpoint_ready(bind_addr.as_str());

    let sender_did = "kamn:did:agent:ws-presence-client-scope";
    let nonce = 41_u64;
    let state_hash = format!(
        "service-api:{}:{}",
        snapshot.chain_id.as_str(),
        snapshot.chain_version.as_str()
    );
    let signature = baseline_signature_for_fields(sender_did, nonce, state_hash.as_str(), "");
    let response = send_websocket_upgrade_request(
        bind_addr.as_str(),
        "/v1/events/ws",
        &[
            ("X-KAMN-Sender-DID", sender_did),
            ("X-KAMN-Request-Nonce", "41"),
            ("X-KAMN-Request-Signature", signature.as_str()),
            ("X-KAMN-Events-Mode", "presence"),
            ("X-KAMN-Presence-Owner-DID", "kamn:did:owner:alpha"),
            ("X-KAMN-Presence-Target-Owner-DID", "kamn:did:owner:beta"),
            (
                "X-KAMN-Presence-Target-Agent-DID",
                "kamn:did:agent:beta-target",
            ),
            ("X-KAMN-Presence-Gateway-Node", "gateway-beta"),
            ("X-KAMN-Presence-Connected-Since", "1709000100"),
            ("X-KAMN-Presence-Last-Heartbeat", "1709000105"),
        ],
    );
    let response_text =
        String::from_utf8(response).expect("cross-owner scope denial response should be utf-8");
    assert!(response_text.contains("HTTP/1.1 403 Forbidden"));
    let payload = parse_error_envelope_from_http_response(response_text.as_str());
    assert_eq!(payload.error, "forbidden");
    assert_eq!(payload.reason_code, "m9_realtime_owner_scope_denied");

    let server_result = server.join().expect("endpoint thread should complete");
    assert!(
        server_result.is_ok(),
        "service api endpoint should stop cleanly after cross-owner websocket rejection"
    );
}

#[test]
fn regression_service_api_endpoint_websocket_route_rejects_missing_upgrade_headers() {
    let _env = acquire_service_api_test_env();
    let parsed = parse_args(vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
        "--runtime-mode".to_owned(),
        "api".to_owned(),
        "--api-bind".to_owned(),
        "127.0.0.1:34056".to_owned(),
    ])
    .expect("api args should parse");
    let report = execute(parsed).expect("api execution should succeed");
    let snapshot = build_service_api_snapshot(&report);

    let bind_addr = reserve_loopback_addr();
    let endpoint_config = ServiceApiEndpointConfig {
        bind_addr: bind_addr.clone(),
        max_requests: 1,
        idle_timeout_ms: 2_000,
        body_limit_bytes: DEFAULT_SERVICE_API_BODY_LIMIT_BYTES,
        concurrency_limit: DEFAULT_SERVICE_API_CONCURRENCY_LIMIT,
        rate_limit_per_second: DEFAULT_SERVICE_API_RATE_LIMIT_PER_SECOND,
    };
    let server_snapshot = snapshot.clone();
    let server =
        thread::spawn(move || serve_service_api_endpoint(&endpoint_config, &server_snapshot));
    wait_for_endpoint_ready(bind_addr.as_str());

    let sender_did = "kamn:did:agent:ws-client-2";
    let nonce = 23_u64;
    let state_hash = format!(
        "service-api:{}:{}",
        snapshot.chain_id.as_str(),
        snapshot.chain_version.as_str()
    );
    let signature = baseline_signature_for_fields(sender_did, nonce, state_hash.as_str(), "");
    let response = send_http_request_with_headers(
        bind_addr.as_str(),
        "GET",
        "/v1/events/ws",
        "",
        &[
            ("X-KAMN-Sender-DID", sender_did),
            ("X-KAMN-Request-Nonce", "23"),
            ("X-KAMN-Request-Signature", signature.as_str()),
        ],
    );
    assert!(response.contains("HTTP/1.1 400 Bad Request"));
    let payload = parse_error_envelope_from_http_response(response.as_str());
    assert_eq!(payload.error, "bad-request");
    assert_eq!(payload.reason_code, "service_api_ws_upgrade_header_missing");
    assert!(payload
        .message
        .contains("missing required websocket upgrade header"));

    let server_result = server.join().expect("endpoint thread should complete");
    assert!(
        server_result.is_ok(),
        "service api endpoint should stop cleanly after websocket rejection budget"
    );
}

#[test]
fn regression_service_api_endpoint_websocket_rejects_invalid_version_header() {
    let _env = acquire_service_api_test_env();
    let parsed = parse_args(vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
        "--runtime-mode".to_owned(),
        "api".to_owned(),
        "--api-bind".to_owned(),
        "127.0.0.1:34057".to_owned(),
    ])
    .expect("api args should parse");
    let report = execute(parsed).expect("api execution should succeed");
    let snapshot = build_service_api_snapshot(&report);

    let bind_addr = reserve_loopback_addr();
    let endpoint_config = ServiceApiEndpointConfig {
        bind_addr: bind_addr.clone(),
        max_requests: 1,
        idle_timeout_ms: 2_000,
        body_limit_bytes: DEFAULT_SERVICE_API_BODY_LIMIT_BYTES,
        concurrency_limit: DEFAULT_SERVICE_API_CONCURRENCY_LIMIT,
        rate_limit_per_second: DEFAULT_SERVICE_API_RATE_LIMIT_PER_SECOND,
    };
    let server_snapshot = snapshot.clone();
    let server =
        thread::spawn(move || serve_service_api_endpoint(&endpoint_config, &server_snapshot));
    wait_for_endpoint_ready(bind_addr.as_str());

    let sender_did = "kamn:did:agent:ws-client-3";
    let nonce = 29_u64;
    let state_hash = format!(
        "service-api:{}:{}",
        snapshot.chain_id.as_str(),
        snapshot.chain_version.as_str()
    );
    let signature = baseline_signature_for_fields(sender_did, nonce, state_hash.as_str(), "");
    let response = send_websocket_upgrade_request_with_version(
        bind_addr.as_str(),
        "/v1/events/ws",
        "12",
        &[
            ("X-KAMN-Sender-DID", sender_did),
            ("X-KAMN-Request-Nonce", "29"),
            ("X-KAMN-Request-Signature", signature.as_str()),
        ],
    );
    let response_text =
        String::from_utf8(response).expect("invalid websocket version response should be utf-8");
    assert!(response_text.contains("HTTP/1.1 400 Bad Request"));
    let payload = parse_error_envelope_from_http_response(response_text.as_str());
    assert_eq!(payload.error, "bad-request");
    assert_eq!(payload.reason_code, "service_api_ws_version_header_invalid");
    assert!(payload.message.contains("invalid websocket version header"));

    let server_result = server.join().expect("endpoint thread should complete");
    assert!(
        server_result.is_ok(),
        "service api endpoint should stop cleanly after websocket version rejection"
    );
}

#[test]
fn unit_service_api_endpoint_lifecycle_rejection_projection_is_deterministic() {
    let first = project_service_api_lifecycle_rejection("service_api_ingress_rate_limit_exceeded")
        .expect("known lifecycle reason code should project");
    let second = project_service_api_lifecycle_rejection("service_api_ingress_rate_limit_exceeded")
        .expect("known lifecycle reason code should project");
    assert_eq!(first, second);
}

#[test]
fn functional_service_api_endpoint_lifecycle_rejection_projection_maps_limiter_classes() {
    let concurrency =
        project_service_api_lifecycle_rejection("service_api_ingress_concurrency_limit_exceeded")
            .expect("concurrency limiter reason must project");
    assert_eq!(
        concurrency,
        ServiceApiLifecycleRejectionProjection {
            rejection_class: "async-lifecycle-limiter",
            reason_code: "service_api_ingress_concurrency_limit_exceeded",
            status_code: 429,
            error_label: "too-many-requests",
            outcome: "concurrency-limit",
        }
    );

    let sender_suspended =
        project_service_api_lifecycle_rejection("service_api_ingress_sender_suspended")
            .expect("sender suspension reason must project");
    assert_eq!(sender_suspended.rejection_class, "sender-admission-limiter");
    assert_eq!(sender_suspended.status_code, 429);
    assert_eq!(sender_suspended.error_label, "too-many-requests");
    assert_eq!(sender_suspended.outcome, "anti-spam");
}

#[test]
fn functional_service_api_endpoint_backpressure_projection_covers_reason_codes() {
    let expected = [
        (
            "service_api_ingress_concurrency_limit_exceeded",
            "async-lifecycle-limiter",
            "concurrency-limit",
        ),
        (
            "service_api_ingress_rate_limit_exceeded",
            "async-lifecycle-limiter",
            "rate-limit",
        ),
        (
            "service_api_ingress_sender_rate_limit_exceeded",
            "sender-admission-limiter",
            "anti-spam",
        ),
    ];

    for (reason_code, rejection_class, outcome) in expected {
        let projection = project_service_api_lifecycle_rejection(reason_code)
            .expect("known backpressure reason code should project");
        assert_eq!(projection.reason_code, reason_code);
        assert_eq!(projection.rejection_class, rejection_class);
        assert_eq!(projection.status_code, 429);
        assert_eq!(projection.error_label, "too-many-requests");
        assert_eq!(projection.outcome, outcome);
    }
}

#[test]
fn integration_service_api_endpoint_lifecycle_projection_matches_live_concurrency_rejection() {
    let _env = acquire_service_api_test_env();
    let parsed = parse_args(vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
        "--runtime-mode".to_owned(),
        "api".to_owned(),
        "--api-bind".to_owned(),
        "127.0.0.1:34067".to_owned(),
    ])
    .expect("api args should parse");
    let report = execute(parsed).expect("api execution should succeed");
    let snapshot = build_service_api_snapshot(&report);

    let bind_addr = reserve_loopback_addr();
    let worker_count = 4_usize;
    let endpoint_config = ServiceApiEndpointConfig {
        bind_addr: bind_addr.clone(),
        max_requests: worker_count as u64,
        idle_timeout_ms: 2_000,
        body_limit_bytes: DEFAULT_SERVICE_API_BODY_LIMIT_BYTES,
        concurrency_limit: 1,
        rate_limit_per_second: 1_000,
    };
    let server_snapshot = snapshot.clone();
    let server =
        thread::spawn(move || serve_service_api_endpoint(&endpoint_config, &server_snapshot));
    wait_for_endpoint_ready(bind_addr.as_str());

    let sender_did = "kamn:did:agent:test-client-lifecycle-projection";
    let message_body = "{\"message\":\"lifecycle-projection-check\"}";
    let state_hash = format!(
        "service-api:{}:{}",
        snapshot.chain_id.as_str(),
        snapshot.chain_version.as_str()
    );

    let barrier = Arc::new(Barrier::new(worker_count));
    let mut clients = Vec::with_capacity(worker_count);
    for request_index in 0..worker_count {
        let client_bind_addr = bind_addr.clone();
        let barrier = barrier.clone();
        let state_hash = state_hash.clone();
        clients.push(thread::spawn(move || {
            let nonce = 810 + request_index as u64;
            let signature =
                baseline_signature_for_fields(sender_did, nonce, state_hash.as_str(), message_body);
            let nonce_text = nonce.to_string();
            barrier.wait();
            send_http_request_with_headers(
                client_bind_addr.as_str(),
                "POST",
                "/v1/messages/send",
                message_body,
                &[
                    ("X-KAMN-Sender-DID", sender_did),
                    ("X-KAMN-Request-Nonce", nonce_text.as_str()),
                    ("X-KAMN-Request-Signature", signature.as_str()),
                ],
            )
        }));
    }

    let responses = clients
        .into_iter()
        .map(|client| client.join().expect("client request should complete"))
        .collect::<Vec<String>>();
    let rejection = responses
        .iter()
        .find(|response| response.contains("HTTP/1.1 429 Too Many Requests"))
        .expect("expected at least one lifecycle limiter rejection");
    let rejection_payload = parse_error_envelope_from_http_response(rejection);
    let projection =
        project_service_api_lifecycle_rejection(rejection_payload.reason_code.as_str())
            .expect("live rejection reason should have projection");
    assert_eq!(projection.rejection_class, "async-lifecycle-limiter");
    assert_eq!(
        projection.reason_code,
        "service_api_ingress_concurrency_limit_exceeded"
    );
    assert_eq!(projection.status_code, 429);
    assert_eq!(projection.error_label, "too-many-requests");
    assert_eq!(projection.outcome, "concurrency-limit");

    let server_result = server.join().expect("endpoint thread should complete");
    assert!(
        server_result.is_ok(),
        "service api endpoint should stop cleanly after lifecycle projection integration budget"
    );
}

#[test]
fn regression_service_api_endpoint_concurrency_limit_reason_code_stays_stable_across_rounds() {
    // Regression: #4315
    let _env = acquire_service_api_test_env();
    let parsed = parse_args(vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
        "--runtime-mode".to_owned(),
        "api".to_owned(),
        "--api-bind".to_owned(),
        "127.0.0.1:34068".to_owned(),
    ])
    .expect("api args should parse");
    let report = execute(parsed).expect("api execution should succeed");
    let snapshot = build_service_api_snapshot(&report);

    let bind_addr = reserve_loopback_addr();
    let rounds = 3_u64;
    let worker_count = 4_usize;
    let endpoint_config = ServiceApiEndpointConfig {
        bind_addr: bind_addr.clone(),
        max_requests: rounds * worker_count as u64,
        idle_timeout_ms: 2_000,
        body_limit_bytes: DEFAULT_SERVICE_API_BODY_LIMIT_BYTES,
        concurrency_limit: 1,
        rate_limit_per_second: 1_000,
    };
    let server_snapshot = snapshot.clone();
    let server =
        thread::spawn(move || serve_service_api_endpoint(&endpoint_config, &server_snapshot));
    wait_for_endpoint_ready(bind_addr.as_str());

    let sender_did = "kamn:did:agent:test-client-concurrency-regression";
    let state_hash = format!(
        "service-api:{}:{}",
        snapshot.chain_id.as_str(),
        snapshot.chain_version.as_str()
    );

    for round in 0..rounds {
        let barrier = Arc::new(Barrier::new(worker_count));
        let mut clients = Vec::with_capacity(worker_count);
        for request_index in 0..worker_count {
            let client_bind_addr = bind_addr.clone();
            let barrier = barrier.clone();
            let state_hash = state_hash.clone();
            clients.push(thread::spawn(move || {
                let body = format!(
                    "{{\"message\":\"concurrency-stability-round-{round}-request-{request_index}\"}}"
                );
                let nonce = 4_000 + round * worker_count as u64 + request_index as u64;
                let signature =
                    baseline_signature_for_fields(sender_did, nonce, state_hash.as_str(), &body);
                let nonce_text = nonce.to_string();
                barrier.wait();
                send_http_request_with_headers(
                    client_bind_addr.as_str(),
                    "POST",
                    "/v1/messages/send",
                    body.as_str(),
                    &[
                        ("X-KAMN-Sender-DID", sender_did),
                        ("X-KAMN-Request-Nonce", nonce_text.as_str()),
                        ("X-KAMN-Request-Signature", signature.as_str()),
                    ],
                )
            }));
        }

        let responses = clients
            .into_iter()
            .map(|client| client.join().expect("client request should complete"))
            .collect::<Vec<String>>();
        assert!(
            responses
                .iter()
                .any(|response| response.contains("HTTP/1.1 202 Accepted")),
            "round {round} expected at least one accepted request"
        );
        let rejection_payloads = responses
            .iter()
            .filter(|response| response.contains("HTTP/1.1 429 Too Many Requests"))
            .map(|response| parse_error_envelope_from_http_response(response))
            .collect::<Vec<ServiceApiErrorEnvelope>>();
        assert!(
            !rejection_payloads.is_empty(),
            "round {round} expected at least one fail-closed concurrency rejection"
        );
        for payload in rejection_payloads {
            assert_eq!(payload.error, "too-many-requests");
            assert_eq!(
                payload.reason_code,
                "service_api_ingress_concurrency_limit_exceeded"
            );
            assert!(payload
                .message
                .contains("ingress concurrency limit exceeded"));
        }
    }

    let server_result = server.join().expect("endpoint thread should complete");
    assert!(
        server_result.is_ok(),
        "service api endpoint should stop cleanly after concurrency regression rounds"
    );
}

#[test]
fn regression_service_api_endpoint_lifecycle_projection_sender_suspension_class_stays_stable() {
    // Regression: #4316
    let projection =
        project_service_api_lifecycle_rejection("service_api_ingress_sender_suspended")
            .expect("sender suspension projection should exist");
    assert_eq!(projection.rejection_class, "sender-admission-limiter");
    assert_eq!(projection.status_code, 429);
    assert_eq!(projection.outcome, "anti-spam");
}

#[test]
fn performance_service_api_endpoint_lifecycle_projection_loop_stays_within_local_budget() {
    let started = Instant::now();
    let reason_codes = [
        "service_api_ingress_concurrency_limit_exceeded",
        "service_api_ingress_rate_limit_exceeded",
        "service_api_ingress_sender_rate_limit_exceeded",
        "service_api_ingress_sender_suspended",
    ];

    for idx in 0..60_000_u32 {
        let reason_code = reason_codes[idx as usize % reason_codes.len()];
        let projection = project_service_api_lifecycle_rejection(reason_code)
            .expect("known lifecycle reason should project");
        assert_eq!(projection.reason_code, reason_code);
    }

    assert!(
        started.elapsed() <= Duration::from_secs(2),
        "lifecycle projection loop exceeded local budget"
    );
}
