use super::*;
use crate::service_api_endpoint::{
    parse_service_api_payload, project_service_api_lifecycle_rejection, ServiceApiAgentGetBody,
    ServiceApiChannelCreateBody, ServiceApiChannelMessagesBody, ServiceApiHealthBody,
    ServiceApiLifecycleRejectionProjection, ServiceApiMessageCreateBody, ServiceApiMessageGetBody,
    ServiceApiTaskCreateBody, DEFAULT_SERVICE_API_BODY_LIMIT_BYTES,
    DEFAULT_SERVICE_API_CONCURRENCY_LIMIT, DEFAULT_SERVICE_API_RATE_LIMIT_PER_SECOND,
    SERVICE_API_AUTH_REASON_CODES_CSV, SERVICE_API_AUTH_REASON_TAXONOMY_VERSION,
    SERVICE_API_LIFECYCLE_REJECTION_REASON_CODES_CSV,
    SERVICE_API_LIFECYCLE_REJECTION_REASON_TAXONOMY_VERSION,
    SERVICE_API_ROUTE_AUTHZ_MATRIX_PROTECTED_ROUTE_COUNT,
    SERVICE_API_ROUTE_AUTHZ_MATRIX_PUBLIC_ROUTE_COUNT,
    SERVICE_API_ROUTE_AUTHZ_MATRIX_SCHEMA_VERSION,
    SERVICE_API_ROUTE_AUTHZ_MATRIX_TOTAL_ROUTE_COUNT,
    SERVICE_API_SCOPE_POLICY_FIXTURE_SCHEMA_VERSION, SERVICE_API_SCOPE_POLICY_REASON_CODES_CSV,
    SERVICE_API_SCOPE_POLICY_REASON_TAXONOMY_VERSION, SERVICE_API_WEBSOCKET_REASON_CODES_CSV,
    SERVICE_API_WEBSOCKET_REASON_TAXONOMY_VERSION,
};
use kamn_core::{
    cross_store_replay_reason_codes_csv, cross_store_replay_reason_taxonomy_version,
    service_auth_public_key_hex_from_private_key_hex, service_auth_sign_with_private_key_hex,
};
use rustls::pki_types::{CertificateDer, ServerName, UnixTime};
use serde::Deserialize;
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io::{ErrorKind, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Barrier, MutexGuard};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

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

const SERVICE_API_AUTH_MISSING_HEADER_REASON_CODE: &str =
    "service_api_auth_sender_did_header_missing";
const SERVICE_API_AUTH_SCOPE_HEADER_MISSING_REASON_CODE: &str =
    "service_api_auth_scope_header_missing";
const SERVICE_API_AUTH_SCOPE_INVALID_REASON_CODE: &str = "service_api_auth_scope_invalid";
const SERVICE_API_AUTH_SCOPE_ROUTE_MISMATCH_REASON_CODE: &str =
    "service_api_auth_scope_route_mismatch";
const SERVICE_API_SCOPE_POLICY_FIXTURE: &str =
    include_str!("../../../../fixtures/runtime/service_api_scope_policy_fixture_matrix.txt");
const TEST_SERVICE_API_AUTH_PRIVATE_KEY_HEX: &str =
    "658c3528422eb527b4c108b8f6d1e5f629543c304ea49cf608c67794424291c4";

#[derive(Debug, Clone, PartialEq, Eq)]
struct ServiceApiRouteAuthzMatrixRow {
    method: &'static str,
    path: &'static str,
    body: &'static str,
    requires_auth: bool,
    expected_status_without_auth: &'static str,
}

fn service_api_route_authz_matrix_rows() -> Vec<ServiceApiRouteAuthzMatrixRow> {
    vec![
        ServiceApiRouteAuthzMatrixRow {
            method: "GET",
            path: "/healthz",
            body: "",
            requires_auth: false,
            expected_status_without_auth: "HTTP/1.1 200 OK",
        },
        ServiceApiRouteAuthzMatrixRow {
            method: "GET",
            path: "/metrics",
            body: "",
            requires_auth: false,
            expected_status_without_auth: "HTTP/1.1 200 OK",
        },
        ServiceApiRouteAuthzMatrixRow {
            method: "POST",
            path: "/v1/messages/send",
            body: "{\"message\":\"matrix-message\"}",
            requires_auth: true,
            expected_status_without_auth: "HTTP/1.1 401 Unauthorized",
        },
        ServiceApiRouteAuthzMatrixRow {
            method: "POST",
            path: "/v1/channels/create",
            body: "{\"name\":\"matrix-channel\"}",
            requires_auth: true,
            expected_status_without_auth: "HTTP/1.1 401 Unauthorized",
        },
        ServiceApiRouteAuthzMatrixRow {
            method: "POST",
            path: "/v1/tasks/create",
            body: "{\"task\":\"matrix-task\"}",
            requires_auth: true,
            expected_status_without_auth: "HTTP/1.1 401 Unauthorized",
        },
        ServiceApiRouteAuthzMatrixRow {
            method: "POST",
            path: "/v1/tasks/task-matrix/accept",
            body: "{}",
            requires_auth: true,
            expected_status_without_auth: "HTTP/1.1 401 Unauthorized",
        },
        ServiceApiRouteAuthzMatrixRow {
            method: "POST",
            path: "/v1/tasks/task-matrix/complete",
            body: "{}",
            requires_auth: true,
            expected_status_without_auth: "HTTP/1.1 401 Unauthorized",
        },
        ServiceApiRouteAuthzMatrixRow {
            method: "POST",
            path: "/v1/escrow/fund",
            body: "{\"task_id\":\"task-matrix\",\"amount\":100}",
            requires_auth: true,
            expected_status_without_auth: "HTTP/1.1 401 Unauthorized",
        },
        ServiceApiRouteAuthzMatrixRow {
            method: "POST",
            path: "/v1/escrow/escrow-matrix/release",
            body: "{}",
            requires_auth: true,
            expected_status_without_auth: "HTTP/1.1 401 Unauthorized",
        },
        ServiceApiRouteAuthzMatrixRow {
            method: "POST",
            path: "/v1/content/register",
            body: "{\"content\":\"matrix-content\",\"retention_class\":\"standard\"}",
            requires_auth: true,
            expected_status_without_auth: "HTTP/1.1 401 Unauthorized",
        },
        ServiceApiRouteAuthzMatrixRow {
            method: "POST",
            path: "/v1/content/content-matrix/expire",
            body: "{}",
            requires_auth: true,
            expected_status_without_auth: "HTTP/1.1 401 Unauthorized",
        },
        ServiceApiRouteAuthzMatrixRow {
            method: "POST",
            path: "/v1/content/content-matrix/tombstone",
            body: "{}",
            requires_auth: true,
            expected_status_without_auth: "HTTP/1.1 401 Unauthorized",
        },
        ServiceApiRouteAuthzMatrixRow {
            method: "GET",
            path: "/v1/content/content-matrix",
            body: "",
            requires_auth: true,
            expected_status_without_auth: "HTTP/1.1 401 Unauthorized",
        },
        ServiceApiRouteAuthzMatrixRow {
            method: "POST",
            path: "/v1/bridge/submit",
            body: "{\"source_message_id\":\"msg-matrix\",\"target_network\":\"testnet\"}",
            requires_auth: true,
            expected_status_without_auth: "HTTP/1.1 401 Unauthorized",
        },
        ServiceApiRouteAuthzMatrixRow {
            method: "POST",
            path: "/v1/bridge/bridge-matrix/forward",
            body: "{}",
            requires_auth: true,
            expected_status_without_auth: "HTTP/1.1 401 Unauthorized",
        },
        ServiceApiRouteAuthzMatrixRow {
            method: "GET",
            path: "/v1/bridge/bridge-matrix",
            body: "",
            requires_auth: true,
            expected_status_without_auth: "HTTP/1.1 401 Unauthorized",
        },
        ServiceApiRouteAuthzMatrixRow {
            method: "GET",
            path: "/v1/messages/msg-matrix",
            body: "",
            requires_auth: true,
            expected_status_without_auth: "HTTP/1.1 401 Unauthorized",
        },
        ServiceApiRouteAuthzMatrixRow {
            method: "GET",
            path: "/v1/channels/channel-matrix/messages",
            body: "",
            requires_auth: true,
            expected_status_without_auth: "HTTP/1.1 401 Unauthorized",
        },
        ServiceApiRouteAuthzMatrixRow {
            method: "GET",
            path: "/v1/tasks/task-matrix",
            body: "",
            requires_auth: true,
            expected_status_without_auth: "HTTP/1.1 401 Unauthorized",
        },
        ServiceApiRouteAuthzMatrixRow {
            method: "GET",
            path: "/v1/agents/kamn:did:agent:matrix",
            body: "",
            requires_auth: true,
            expected_status_without_auth: "HTTP/1.1 401 Unauthorized",
        },
        ServiceApiRouteAuthzMatrixRow {
            method: "GET",
            path: "/v1/events/ws",
            body: "",
            requires_auth: true,
            expected_status_without_auth: "HTTP/1.1 401 Unauthorized",
        },
    ]
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ServiceApiScopePolicyFixtureRow {
    method: String,
    path: String,
    scope: String,
    expected: String,
}

fn parse_service_api_scope_policy_fixture(
    fixture: &str,
) -> (
    BTreeMap<String, String>,
    Vec<ServiceApiScopePolicyFixtureRow>,
) {
    let mut metadata = BTreeMap::new();
    let mut rows = Vec::new();
    for line in fixture.lines().map(str::trim) {
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some((key, value)) = line.split_once('=') {
            metadata.insert(key.trim().to_owned(), value.trim().to_owned());
            continue;
        }
        if let Some(payload) = line.strip_prefix("row|") {
            let mut parts = payload.split('|');
            let method = parts.next().unwrap_or_default().trim().to_owned();
            let path = parts.next().unwrap_or_default().trim().to_owned();
            let scope = parts.next().unwrap_or_default().trim().to_owned();
            let expected = parts.next().unwrap_or_default().trim().to_owned();
            if !method.is_empty() && !path.is_empty() && !scope.is_empty() && !expected.is_empty() {
                rows.push(ServiceApiScopePolicyFixtureRow {
                    method,
                    path,
                    scope,
                    expected,
                });
            }
        }
    }
    (metadata, rows)
}

fn required_scope_for_test_route(method: &str, path: &str) -> Option<&'static str> {
    if !crate::service_api_endpoint::route_requires_auth(method, path) {
        return None;
    }
    let scope = match (method, path) {
        ("POST", "/v1/messages/send") => "messages:write",
        ("POST", "/v1/channels/create") => "channels:write",
        ("POST", "/v1/tasks/create") => "tasks:write",
        ("POST", "/v1/escrow/fund") => "escrow:write",
        ("POST", "/v1/content/register") => "content:write",
        ("POST", "/v1/bridge/submit") => "bridge:write",
        ("POST", _) if path.starts_with("/v1/tasks/") && path.ends_with("/accept") => "tasks:write",
        ("POST", _) if path.starts_with("/v1/tasks/") && path.ends_with("/complete") => {
            "tasks:write"
        }
        ("POST", _) if path.starts_with("/v1/escrow/") && path.ends_with("/release") => {
            "escrow:write"
        }
        ("POST", _) if path.starts_with("/v1/content/") && path.ends_with("/expire") => {
            "content:write"
        }
        ("POST", _) if path.starts_with("/v1/content/") && path.ends_with("/tombstone") => {
            "content:write"
        }
        ("POST", _) if path.starts_with("/v1/bridge/") && path.ends_with("/forward") => {
            "bridge:write"
        }
        ("GET", "/v1/events/ws") => "events:read",
        ("GET", _) if path.starts_with("/v1/content/") && path != "/v1/content/register" => {
            "content:read"
        }
        ("GET", _) if path.starts_with("/v1/bridge/") && path != "/v1/bridge/submit" => {
            "bridge:read"
        }
        ("GET", _) if path.starts_with("/v1/messages/") && path != "/v1/messages/send" => {
            "messages:read"
        }
        ("GET", _) if path.starts_with("/v1/channels/") && path.ends_with("/messages") => {
            "channels:read"
        }
        ("GET", _) if path.starts_with("/v1/tasks/") && path != "/v1/tasks/create" => "tasks:read",
        ("GET", _) if path.starts_with("/v1/agents/") => "agents:read",
        _ => "protected:unknown",
    };
    Some(scope)
}

fn signed_header_present(headers: &[(&str, &str)], name: &str) -> bool {
    headers
        .iter()
        .any(|(header_name, _)| header_name.eq_ignore_ascii_case(name))
}

fn enrich_signed_headers_with_scope<'a>(
    method: &str,
    path: &str,
    headers: &'a [(&'a str, &'a str)],
) -> Vec<(&'a str, &'a str)> {
    let mut enriched = headers.to_vec();
    let signed_request = signed_header_present(headers, "X-KAMN-Sender-DID")
        && signed_header_present(headers, "X-KAMN-Request-Nonce")
        && signed_header_present(headers, "X-KAMN-Request-Signature");
    let has_scope_header = signed_header_present(headers, "X-KAMN-Authz-Scope");
    if signed_request && !has_scope_header {
        if let Some(scope) = required_scope_for_test_route(method, path) {
            enriched.push(("X-KAMN-Authz-Scope", scope));
        }
    }
    enriched
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
    let enriched_headers = enrich_signed_headers_with_scope(method, path, headers);
    send_http_request_with_headers_raw(addr, method, path, body, &enriched_headers)
}

fn send_http_request_with_headers_raw(
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

async fn send_http_request_with_headers_async(
    addr: &str,
    method: &str,
    path: &str,
    body: &str,
    headers: &[(&str, &str)],
) -> Result<String, String> {
    let enriched_headers = enrich_signed_headers_with_scope(method, path, headers);
    let mut stream = tokio::net::TcpStream::connect(addr)
        .await
        .map_err(|error| format!("async http connect should succeed: {error}"))?;
    let mut header_lines = String::new();
    for (name, value) in &enriched_headers {
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
        .await
        .map_err(|error| format!("async http request should write: {error}"))?;
    stream
        .flush()
        .await
        .map_err(|error| format!("async http request should flush: {error}"))?;

    let mut response = Vec::new();
    let mut chunk = [0_u8; 1024];
    loop {
        match tokio::time::timeout(Duration::from_millis(150), stream.read(&mut chunk)).await {
            Ok(Ok(0)) => break,
            Ok(Ok(read_count)) => response.extend_from_slice(&chunk[..read_count]),
            Ok(Err(error)) => return Err(format!("async http response read failed: {error}")),
            Err(_) => break,
        }
    }

    String::from_utf8(response)
        .map_err(|error| format!("async http response was not utf-8: {error}"))
}

fn send_https_request_with_headers(
    addr: &str,
    method: &str,
    path: &str,
    body: &str,
    headers: &[(&str, &str)],
    _root_cert_pem: &str,
) -> String {
    let enriched_headers = enrich_signed_headers_with_scope(method, path, headers);
    send_https_request_with_headers_raw(addr, method, path, body, &enriched_headers, _root_cert_pem)
}

fn send_https_request_with_headers_raw(
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
    send_websocket_upgrade_request_with_version_close_observation(
        addr,
        path,
        websocket_version,
        headers,
    )
    .0
}

fn send_websocket_upgrade_request_with_version_close_observation(
    addr: &str,
    path: &str,
    websocket_version: &str,
    headers: &[(&str, &str)],
) -> (Vec<u8>, bool) {
    let mut stream = TcpStream::connect(addr).expect("endpoint should accept websocket connection");
    stream
        .set_read_timeout(Some(Duration::from_secs(2)))
        .expect("websocket read timeout should be configurable");
    let enriched_headers = enrich_signed_headers_with_scope("GET", path, headers);
    let mut header_lines = String::new();
    for (name, value) in &enriched_headers {
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
    let mut peer_closed = false;
    let mut chunk = [0_u8; 1024];
    loop {
        match stream.read(&mut chunk) {
            Ok(0) => {
                peer_closed = true;
                break;
            }
            Ok(read_count) => response.extend_from_slice(&chunk[..read_count]),
            Err(error) if matches!(error.kind(), ErrorKind::WouldBlock | ErrorKind::TimedOut) => {
                break;
            }
            Err(error) => panic!("websocket response should be readable: {error}"),
        }
    }
    (response, peer_closed)
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

fn parse_scalar_metric_value(response: &str, metric_name: &str) -> Option<u64> {
    let body = extract_http_response_body(response);
    let expected_prefix = format!("{metric_name} ");
    body.lines().find_map(|line| {
        let value = line.trim().strip_prefix(expected_prefix.as_str())?;
        value.parse::<u64>().ok()
    })
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

fn parse_websocket_response_frames(response: &[u8]) -> (String, Vec<String>) {
    let header_end = response
        .windows(4)
        .position(|window| window == b"\r\n\r\n")
        .map(|index| index + 4)
        .expect("websocket response should include header terminator");
    let header = std::str::from_utf8(&response[..header_end])
        .expect("websocket header should be utf-8")
        .to_owned();
    let mut frames = Vec::new();
    let frame_bytes = &response[header_end..];
    let mut frame_index = 0_usize;

    while frame_index + 2 <= frame_bytes.len() {
        let first = frame_bytes[frame_index];
        let second = frame_bytes[frame_index + 1];
        let opcode = first & 0x0f;
        assert_eq!(
            first & 0x80,
            0x80,
            "fragmented websocket frames are not supported by test parser"
        );
        assert_eq!(second & 0x80, 0, "server websocket frame must be unmasked");

        let mut payload_index = frame_index + 2;
        let payload_len = match second & 0x7f {
            value @ 0..=125 => value as usize,
            126 => {
                assert!(
                    frame_bytes.len() >= frame_index + 4,
                    "websocket frame extended payload length must be available"
                );
                payload_index = frame_index + 4;
                u16::from_be_bytes([frame_bytes[frame_index + 2], frame_bytes[frame_index + 3]])
                    as usize
            }
            127 => {
                assert!(
                    frame_bytes.len() >= frame_index + 10,
                    "websocket frame 64-bit payload length must be available"
                );
                payload_index = frame_index + 10;
                u64::from_be_bytes([
                    frame_bytes[frame_index + 2],
                    frame_bytes[frame_index + 3],
                    frame_bytes[frame_index + 4],
                    frame_bytes[frame_index + 5],
                    frame_bytes[frame_index + 6],
                    frame_bytes[frame_index + 7],
                    frame_bytes[frame_index + 8],
                    frame_bytes[frame_index + 9],
                ]) as usize
            }
            _ => unreachable!("websocket payload marker is constrained to 7 bits"),
        };

        assert!(
            frame_bytes.len() >= payload_index + payload_len,
            "websocket frame payload length must be available"
        );
        let payload_slice = &frame_bytes[payload_index..payload_index + payload_len];
        frame_index = payload_index + payload_len;

        if opcode == 0x8 {
            break;
        }
        if opcode == 0x1 {
            frames.push(
                std::str::from_utf8(payload_slice)
                    .expect("websocket payload should be utf-8")
                    .to_owned(),
            );
        }
    }

    (header, frames)
}

fn parse_websocket_response(response: &[u8]) -> (String, String) {
    let (header, frames) = parse_websocket_response_frames(response);
    let payload = frames
        .into_iter()
        .next()
        .expect("websocket response should include at least one text frame");
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

fn service_api_request_signature_for_fields(
    sender: &str,
    nonce: u64,
    state_hash: &str,
    payload: &str,
) -> String {
    service_auth_sign_with_private_key_hex(
        sender,
        nonce,
        state_hash,
        payload,
        TEST_SERVICE_API_AUTH_PRIVATE_KEY_HEX,
    )
    .expect("service-auth signature should render for test fixture key")
}

struct ServiceApiTestEnvGuards {
    _env_lock: MutexGuard<'static, ()>,
    _tls_mode_guard: EnvVarGuard,
    _tls_cert_guard: EnvVarGuard,
    _tls_key_guard: EnvVarGuard,
    _auth_public_key_guard: EnvVarGuard,
    _state_file_guard: EnvVarGuard,
    _log_level_guard: EnvVarGuard,
    _log_format_guard: EnvVarGuard,
    _chain_id_guard: EnvVarGuard,
    _sync_mode_guard: EnvVarGuard,
}

fn acquire_service_api_test_env() -> ServiceApiTestEnvGuards {
    let env_lock = lock_signer_env_guard();
    let auth_public_key_hex =
        service_auth_public_key_hex_from_private_key_hex(TEST_SERVICE_API_AUTH_PRIVATE_KEY_HEX)
            .expect("service-auth public key should derive");
    ServiceApiTestEnvGuards {
        _env_lock: env_lock,
        _tls_mode_guard: EnvVarGuard::set("KAMN_SERVICE_API_TLS_MODE", None),
        _tls_cert_guard: EnvVarGuard::set("KAMN_SERVICE_API_TLS_CERT_FILE", None),
        _tls_key_guard: EnvVarGuard::set("KAMN_SERVICE_API_TLS_KEY_FILE", None),
        _auth_public_key_guard: EnvVarGuard::set(
            "KAMN_SERVICE_API_AUTH_PUBLIC_KEY_HEX",
            Some(auth_public_key_hex.as_str()),
        ),
        _state_file_guard: EnvVarGuard::set("KAMN_SERVICE_API_STATE_FILE", None),
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

    let task_accept_response =
        render_service_api_endpoint_response(&snapshot, "POST", "/v1/tasks/task-1/accept", "{}");
    assert_eq!(task_accept_response.status_code, 200);
    assert!(task_accept_response.body.contains("\"state\":\"accepted\""));

    let task_complete_response =
        render_service_api_endpoint_response(&snapshot, "POST", "/v1/tasks/task-1/complete", "{}");
    assert_eq!(task_complete_response.status_code, 200);
    assert!(task_complete_response
        .body
        .contains("\"state\":\"completed\""));

    let escrow_fund_response = render_service_api_endpoint_response(
        &snapshot,
        "POST",
        "/v1/escrow/fund",
        "{\"task_id\":\"task-1\",\"amount\":100}",
    );
    assert_eq!(escrow_fund_response.status_code, 200);
    assert!(escrow_fund_response
        .body
        .contains("\"escrow_id\":\"escrow-local-"));
    assert!(escrow_fund_response.body.contains("\"state\":\"funded\""));

    let escrow_release_response = render_service_api_endpoint_response(
        &snapshot,
        "POST",
        "/v1/escrow/escrow-1/release",
        "{}",
    );
    assert_eq!(escrow_release_response.status_code, 200);
    assert!(escrow_release_response
        .body
        .contains("\"state\":\"released\""));

    let content_register_response = render_service_api_endpoint_response(
        &snapshot,
        "POST",
        "/v1/content/register",
        "{\"content\":\"hello\"}",
    );
    assert_eq!(content_register_response.status_code, 201);
    assert!(content_register_response
        .body
        .contains("\"content_id\":\"content-local-"));
    assert!(content_register_response
        .body
        .contains("\"retention_class\":\"standard\""));

    let content_expire_response = render_service_api_endpoint_response(
        &snapshot,
        "POST",
        "/v1/content/content-1/expire",
        "{}",
    );
    assert_eq!(content_expire_response.status_code, 200);
    assert!(content_expire_response
        .body
        .contains("\"lifecycle_state\":\"expired\""));

    let content_tombstone_response = render_service_api_endpoint_response(
        &snapshot,
        "POST",
        "/v1/content/content-1/tombstone",
        "{}",
    );
    assert_eq!(content_tombstone_response.status_code, 200);
    assert!(content_tombstone_response
        .body
        .contains("\"redaction_status\":\"redacted\""));

    let content_query_response =
        render_service_api_endpoint_response(&snapshot, "GET", "/v1/content/content-1", "");
    assert_eq!(content_query_response.status_code, 200);
    assert!(content_query_response
        .body
        .contains("\"lifecycle_state\":\"tombstoned\""));

    let bridge_submit_response = render_service_api_endpoint_response(
        &snapshot,
        "POST",
        "/v1/bridge/submit",
        "{\"source_message_id\":\"msg-1\",\"target_network\":\"testnet\"}",
    );
    assert_eq!(bridge_submit_response.status_code, 202);
    assert!(bridge_submit_response
        .body
        .contains("\"bridge_id\":\"bridge-local-"));
    assert!(bridge_submit_response
        .body
        .contains("\"bridge_status\":\"submitted\""));

    let bridge_forward_response = render_service_api_endpoint_response(
        &snapshot,
        "POST",
        "/v1/bridge/bridge-1/forward",
        "{}",
    );
    assert_eq!(bridge_forward_response.status_code, 200);
    assert!(bridge_forward_response
        .body
        .contains("\"bridge_status\":\"forwarded\""));
    assert!(bridge_forward_response
        .body
        .contains("\"target_message_id\":\"msg-bridge-target-bridge-1\""));

    let bridge_query_response =
        render_service_api_endpoint_response(&snapshot, "GET", "/v1/bridge/bridge-1", "");
    assert_eq!(bridge_query_response.status_code, 200);
    assert!(bridge_query_response
        .body
        .contains("\"forward_tx_hash\":\"sha256:bridge-forwarded-bridge-1\""));

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
    let expected_reason_code_count = cross_store_replay_reason_codes_csv()
        .split(',')
        .filter(|value| !value.is_empty())
        .count();
    let expected_auth_reason_code_count = SERVICE_API_AUTH_REASON_CODES_CSV
        .split(',')
        .filter(|value| !value.is_empty())
        .count();
    let expected_scope_reason_code_count = SERVICE_API_SCOPE_POLICY_REASON_CODES_CSV
        .split(',')
        .filter(|value| !value.is_empty())
        .count();
    let (scope_policy_fixture_metadata, scope_policy_fixture_rows) =
        parse_service_api_scope_policy_fixture(SERVICE_API_SCOPE_POLICY_FIXTURE);
    let expected_scope_policy_fixture_reason_taxonomy_version = scope_policy_fixture_metadata
        .get("scope_policy_reason_taxonomy_version")
        .map(String::as_str)
        .unwrap_or_default();
    let expected_scope_policy_fixture_reason_code_count = scope_policy_fixture_metadata
        .get("scope_policy_reason_codes_csv")
        .map(|value| {
            value
                .split(',')
                .filter(|entry| !entry.trim().is_empty())
                .count()
        })
        .unwrap_or_default();
    let expected_scope_policy_fixture_row_count = scope_policy_fixture_rows.len();
    let expected_scope_policy_fixture_allow_row_count = scope_policy_fixture_rows
        .iter()
        .filter(|row| row.expected == "allow")
        .count();
    let expected_scope_policy_fixture_deny_row_count = scope_policy_fixture_rows
        .iter()
        .filter(|row| row.expected == "deny")
        .count();
    let expected_scope_policy_fixture_unique_route_count = scope_policy_fixture_rows
        .iter()
        .map(|row| (row.method.as_str(), row.path.as_str()))
        .collect::<BTreeSet<_>>()
        .len();
    let expected_scope_policy_fixture_unique_scope_count = scope_policy_fixture_rows
        .iter()
        .map(|row| row.scope.as_str())
        .collect::<BTreeSet<_>>()
        .len();
    let expected_scope_policy_fixture_unique_method_count = scope_policy_fixture_rows
        .iter()
        .map(|row| row.method.as_str())
        .collect::<BTreeSet<_>>()
        .len();
    let expected_scope_policy_fixture_unique_expected_outcome_count = scope_policy_fixture_rows
        .iter()
        .map(|row| row.expected.as_str())
        .collect::<BTreeSet<_>>()
        .len();
    let expected_scope_policy_fixture_unique_allow_scopes = scope_policy_fixture_rows
        .iter()
        .filter(|row| row.expected == "allow")
        .map(|row| row.scope.as_str())
        .collect::<BTreeSet<_>>();
    let expected_scope_policy_fixture_unique_deny_scopes = scope_policy_fixture_rows
        .iter()
        .filter(|row| row.expected == "deny")
        .map(|row| row.scope.as_str())
        .collect::<BTreeSet<_>>();
    let expected_scope_policy_fixture_unique_allow_scope_count =
        expected_scope_policy_fixture_unique_allow_scopes.len();
    let expected_scope_policy_fixture_unique_deny_scope_count =
        expected_scope_policy_fixture_unique_deny_scopes.len();
    let expected_scope_policy_fixture_unique_allow_deny_overlap_scope_count =
        expected_scope_policy_fixture_unique_allow_scopes
            .intersection(&expected_scope_policy_fixture_unique_deny_scopes)
            .count();
    let expected_scope_policy_fixture_unique_allow_only_scope_count =
        expected_scope_policy_fixture_unique_allow_scopes
            .difference(&expected_scope_policy_fixture_unique_deny_scopes)
            .count();
    let expected_scope_policy_fixture_unique_deny_only_scope_count =
        expected_scope_policy_fixture_unique_deny_scopes
            .difference(&expected_scope_policy_fixture_unique_allow_scopes)
            .count();
    let expected_scope_policy_fixture_unique_allow_methods = scope_policy_fixture_rows
        .iter()
        .filter(|row| row.expected == "allow")
        .map(|row| row.method.as_str())
        .collect::<BTreeSet<_>>();
    let expected_scope_policy_fixture_unique_deny_methods = scope_policy_fixture_rows
        .iter()
        .filter(|row| row.expected == "deny")
        .map(|row| row.method.as_str())
        .collect::<BTreeSet<_>>();
    let expected_scope_policy_fixture_unique_allow_deny_overlap_method_count =
        expected_scope_policy_fixture_unique_allow_methods
            .intersection(&expected_scope_policy_fixture_unique_deny_methods)
            .count();
    let expected_scope_policy_fixture_unique_allow_only_method_count =
        expected_scope_policy_fixture_unique_allow_methods
            .difference(&expected_scope_policy_fixture_unique_deny_methods)
            .count();
    let expected_scope_policy_fixture_unique_deny_only_method_count =
        expected_scope_policy_fixture_unique_deny_methods
            .difference(&expected_scope_policy_fixture_unique_allow_methods)
            .count();
    let expected_scope_policy_fixture_unique_allow_routes = scope_policy_fixture_rows
        .iter()
        .filter(|row| row.expected == "allow")
        .map(|row| (row.method.as_str(), row.path.as_str()))
        .collect::<BTreeSet<_>>();
    let expected_scope_policy_fixture_unique_deny_routes = scope_policy_fixture_rows
        .iter()
        .filter(|row| row.expected == "deny")
        .map(|row| (row.method.as_str(), row.path.as_str()))
        .collect::<BTreeSet<_>>();
    let expected_scope_policy_fixture_unique_allow_route_count =
        expected_scope_policy_fixture_unique_allow_routes.len();
    let expected_scope_policy_fixture_unique_deny_route_count =
        expected_scope_policy_fixture_unique_deny_routes.len();
    let expected_scope_policy_fixture_unique_allow_deny_overlap_route_count =
        expected_scope_policy_fixture_unique_allow_routes
            .intersection(&expected_scope_policy_fixture_unique_deny_routes)
            .count();
    let expected_scope_policy_fixture_unique_allow_only_route_count =
        expected_scope_policy_fixture_unique_allow_routes
            .difference(&expected_scope_policy_fixture_unique_deny_routes)
            .count();
    let expected_scope_policy_fixture_unique_deny_only_route_count =
        expected_scope_policy_fixture_unique_deny_routes
            .difference(&expected_scope_policy_fixture_unique_allow_routes)
            .count();
    let expected_websocket_reason_code_count = SERVICE_API_WEBSOCKET_REASON_CODES_CSV
        .split(',')
        .filter(|value| !value.is_empty())
        .count();
    let expected_lifecycle_rejection_reason_code_count =
        SERVICE_API_LIFECYCLE_REJECTION_REASON_CODES_CSV
            .split(',')
            .filter(|value| !value.is_empty())
            .count();
    assert!(metrics_response.body.contains(&format!(
        "kamn_service_api_cross_store_replay_reason_taxonomy_info{{version=\"{}\"}} 1",
        cross_store_replay_reason_taxonomy_version()
    )));
    assert!(metrics_response.body.contains(&format!(
        "kamn_service_api_cross_store_replay_reason_code_count {expected_reason_code_count}"
    )));
    assert!(metrics_response.body.contains(&format!(
        "kamn_service_api_auth_reason_taxonomy_info{{version=\"{}\"}} 1",
        SERVICE_API_AUTH_REASON_TAXONOMY_VERSION
    )));
    assert!(metrics_response.body.contains(&format!(
        "kamn_service_api_auth_reason_code_count {expected_auth_reason_code_count}"
    )));
    assert!(metrics_response.body.contains(&format!(
        "kamn_service_api_scope_policy_reason_taxonomy_info{{version=\"{}\"}} 1",
        SERVICE_API_SCOPE_POLICY_REASON_TAXONOMY_VERSION
    )));
    assert!(metrics_response.body.contains(&format!(
        "kamn_service_api_scope_policy_reason_code_count {expected_scope_reason_code_count}"
    )));
    assert!(metrics_response.body.contains(&format!(
        "kamn_service_api_scope_policy_fixture_schema_info{{version=\"{}\"}} 1",
        SERVICE_API_SCOPE_POLICY_FIXTURE_SCHEMA_VERSION
    )));
    assert!(metrics_response.body.contains(&format!(
        "kamn_service_api_scope_policy_fixture_reason_taxonomy_info{{version=\"{}\"}} 1",
        expected_scope_policy_fixture_reason_taxonomy_version
    )));
    assert!(metrics_response.body.contains(&format!(
        "kamn_service_api_scope_policy_fixture_reason_code_count {expected_scope_policy_fixture_reason_code_count}"
    )));
    assert!(metrics_response.body.contains(&format!(
        "kamn_service_api_scope_policy_fixture_row_count {expected_scope_policy_fixture_row_count}"
    )));
    assert!(metrics_response.body.contains(&format!(
        "kamn_service_api_scope_policy_fixture_allow_row_count {expected_scope_policy_fixture_allow_row_count}"
    )));
    assert!(metrics_response.body.contains(&format!(
        "kamn_service_api_scope_policy_fixture_deny_row_count {expected_scope_policy_fixture_deny_row_count}"
    )));
    assert!(metrics_response.body.contains(&format!(
        "kamn_service_api_scope_policy_fixture_unique_route_count {expected_scope_policy_fixture_unique_route_count}"
    )));
    assert!(metrics_response.body.contains(&format!(
        "kamn_service_api_scope_policy_fixture_unique_scope_count {expected_scope_policy_fixture_unique_scope_count}"
    )));
    assert!(metrics_response.body.contains(&format!(
        "kamn_service_api_scope_policy_fixture_unique_method_count {expected_scope_policy_fixture_unique_method_count}"
    )));
    assert!(metrics_response.body.contains(&format!(
        "kamn_service_api_scope_policy_fixture_unique_expected_outcome_count {expected_scope_policy_fixture_unique_expected_outcome_count}"
    )));
    assert!(metrics_response.body.contains(&format!(
        "kamn_service_api_scope_policy_fixture_unique_allow_scope_count {expected_scope_policy_fixture_unique_allow_scope_count}"
    )));
    assert!(metrics_response.body.contains(&format!(
        "kamn_service_api_scope_policy_fixture_unique_deny_scope_count {expected_scope_policy_fixture_unique_deny_scope_count}"
    )));
    assert!(metrics_response.body.contains(&format!(
        "kamn_service_api_scope_policy_fixture_unique_allow_deny_overlap_scope_count {expected_scope_policy_fixture_unique_allow_deny_overlap_scope_count}"
    )));
    assert!(metrics_response.body.contains(&format!(
        "kamn_service_api_scope_policy_fixture_unique_allow_only_scope_count {expected_scope_policy_fixture_unique_allow_only_scope_count}"
    )));
    assert!(metrics_response.body.contains(&format!(
        "kamn_service_api_scope_policy_fixture_unique_deny_only_scope_count {expected_scope_policy_fixture_unique_deny_only_scope_count}"
    )));
    assert!(metrics_response.body.contains(&format!(
        "kamn_service_api_scope_policy_fixture_unique_allow_deny_overlap_method_count {expected_scope_policy_fixture_unique_allow_deny_overlap_method_count}"
    )));
    assert!(metrics_response.body.contains(&format!(
        "kamn_service_api_scope_policy_fixture_unique_allow_only_method_count {expected_scope_policy_fixture_unique_allow_only_method_count}"
    )));
    assert!(metrics_response.body.contains(&format!(
        "kamn_service_api_scope_policy_fixture_unique_deny_only_method_count {expected_scope_policy_fixture_unique_deny_only_method_count}"
    )));
    assert!(metrics_response.body.contains(&format!(
        "kamn_service_api_scope_policy_fixture_unique_allow_route_count {expected_scope_policy_fixture_unique_allow_route_count}"
    )));
    assert!(metrics_response.body.contains(&format!(
        "kamn_service_api_scope_policy_fixture_unique_deny_route_count {expected_scope_policy_fixture_unique_deny_route_count}"
    )));
    assert!(metrics_response.body.contains(&format!(
        "kamn_service_api_scope_policy_fixture_unique_allow_deny_overlap_route_count {expected_scope_policy_fixture_unique_allow_deny_overlap_route_count}"
    )));
    assert!(metrics_response.body.contains(&format!(
        "kamn_service_api_scope_policy_fixture_unique_allow_only_route_count {expected_scope_policy_fixture_unique_allow_only_route_count}"
    )));
    assert!(metrics_response.body.contains(&format!(
        "kamn_service_api_scope_policy_fixture_unique_deny_only_route_count {expected_scope_policy_fixture_unique_deny_only_route_count}"
    )));
    assert!(metrics_response.body.contains(&format!(
        "kamn_service_api_route_authz_matrix_schema_info{{version=\"{}\"}} 1",
        SERVICE_API_ROUTE_AUTHZ_MATRIX_SCHEMA_VERSION
    )));
    assert!(metrics_response.body.contains(&format!(
        "kamn_service_api_route_authz_matrix_total_route_count {}",
        SERVICE_API_ROUTE_AUTHZ_MATRIX_TOTAL_ROUTE_COUNT
    )));
    assert!(metrics_response.body.contains(&format!(
        "kamn_service_api_route_authz_matrix_public_route_count {}",
        SERVICE_API_ROUTE_AUTHZ_MATRIX_PUBLIC_ROUTE_COUNT
    )));
    assert!(metrics_response.body.contains(&format!(
        "kamn_service_api_route_authz_matrix_protected_route_count {}",
        SERVICE_API_ROUTE_AUTHZ_MATRIX_PROTECTED_ROUTE_COUNT
    )));
    assert!(metrics_response.body.contains(&format!(
        "kamn_service_api_websocket_reason_taxonomy_info{{version=\"{}\"}} 1",
        SERVICE_API_WEBSOCKET_REASON_TAXONOMY_VERSION
    )));
    assert!(metrics_response.body.contains(&format!(
        "kamn_service_api_websocket_reason_code_count {expected_websocket_reason_code_count}"
    )));
    assert!(metrics_response.body.contains(&format!(
        "kamn_service_api_lifecycle_rejection_reason_taxonomy_info{{version=\"{}\"}} 1",
        SERVICE_API_LIFECYCLE_REJECTION_REASON_TAXONOMY_VERSION
    )));
    assert!(metrics_response.body.contains(&format!(
        "kamn_service_api_lifecycle_rejection_reason_code_count {expected_lifecycle_rejection_reason_code_count}"
    )));
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
    let signature = service_api_request_signature_for_fields(
        sender_did,
        sender_nonce,
        state_hash.as_str(),
        message_body,
    );
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
    let expected_reason_code_count = cross_store_replay_reason_codes_csv()
        .split(',')
        .filter(|value| !value.is_empty())
        .count();
    let expected_auth_reason_code_count = SERVICE_API_AUTH_REASON_CODES_CSV
        .split(',')
        .filter(|value| !value.is_empty())
        .count();
    let expected_scope_reason_code_count = SERVICE_API_SCOPE_POLICY_REASON_CODES_CSV
        .split(',')
        .filter(|value| !value.is_empty())
        .count();
    let (scope_policy_fixture_metadata, scope_policy_fixture_rows) =
        parse_service_api_scope_policy_fixture(SERVICE_API_SCOPE_POLICY_FIXTURE);
    let expected_scope_policy_fixture_reason_taxonomy_version = scope_policy_fixture_metadata
        .get("scope_policy_reason_taxonomy_version")
        .map(String::as_str)
        .unwrap_or_default();
    let expected_scope_policy_fixture_reason_code_count = scope_policy_fixture_metadata
        .get("scope_policy_reason_codes_csv")
        .map(|value| {
            value
                .split(',')
                .filter(|entry| !entry.trim().is_empty())
                .count()
        })
        .unwrap_or_default();
    let expected_scope_policy_fixture_row_count = scope_policy_fixture_rows.len();
    let expected_scope_policy_fixture_allow_row_count = scope_policy_fixture_rows
        .iter()
        .filter(|row| row.expected == "allow")
        .count();
    let expected_scope_policy_fixture_deny_row_count = scope_policy_fixture_rows
        .iter()
        .filter(|row| row.expected == "deny")
        .count();
    let expected_scope_policy_fixture_unique_route_count = scope_policy_fixture_rows
        .iter()
        .map(|row| (row.method.as_str(), row.path.as_str()))
        .collect::<BTreeSet<_>>()
        .len();
    let expected_scope_policy_fixture_unique_scope_count = scope_policy_fixture_rows
        .iter()
        .map(|row| row.scope.as_str())
        .collect::<BTreeSet<_>>()
        .len();
    let expected_scope_policy_fixture_unique_method_count = scope_policy_fixture_rows
        .iter()
        .map(|row| row.method.as_str())
        .collect::<BTreeSet<_>>()
        .len();
    let expected_scope_policy_fixture_unique_expected_outcome_count = scope_policy_fixture_rows
        .iter()
        .map(|row| row.expected.as_str())
        .collect::<BTreeSet<_>>()
        .len();
    let expected_scope_policy_fixture_unique_allow_scopes = scope_policy_fixture_rows
        .iter()
        .filter(|row| row.expected == "allow")
        .map(|row| row.scope.as_str())
        .collect::<BTreeSet<_>>();
    let expected_scope_policy_fixture_unique_deny_scopes = scope_policy_fixture_rows
        .iter()
        .filter(|row| row.expected == "deny")
        .map(|row| row.scope.as_str())
        .collect::<BTreeSet<_>>();
    let expected_scope_policy_fixture_unique_allow_scope_count =
        expected_scope_policy_fixture_unique_allow_scopes.len();
    let expected_scope_policy_fixture_unique_deny_scope_count =
        expected_scope_policy_fixture_unique_deny_scopes.len();
    let expected_scope_policy_fixture_unique_allow_deny_overlap_scope_count =
        expected_scope_policy_fixture_unique_allow_scopes
            .intersection(&expected_scope_policy_fixture_unique_deny_scopes)
            .count();
    let expected_scope_policy_fixture_unique_allow_only_scope_count =
        expected_scope_policy_fixture_unique_allow_scopes
            .difference(&expected_scope_policy_fixture_unique_deny_scopes)
            .count();
    let expected_scope_policy_fixture_unique_deny_only_scope_count =
        expected_scope_policy_fixture_unique_deny_scopes
            .difference(&expected_scope_policy_fixture_unique_allow_scopes)
            .count();
    let expected_scope_policy_fixture_unique_allow_methods = scope_policy_fixture_rows
        .iter()
        .filter(|row| row.expected == "allow")
        .map(|row| row.method.as_str())
        .collect::<BTreeSet<_>>();
    let expected_scope_policy_fixture_unique_deny_methods = scope_policy_fixture_rows
        .iter()
        .filter(|row| row.expected == "deny")
        .map(|row| row.method.as_str())
        .collect::<BTreeSet<_>>();
    let expected_scope_policy_fixture_unique_allow_deny_overlap_method_count =
        expected_scope_policy_fixture_unique_allow_methods
            .intersection(&expected_scope_policy_fixture_unique_deny_methods)
            .count();
    let expected_scope_policy_fixture_unique_allow_only_method_count =
        expected_scope_policy_fixture_unique_allow_methods
            .difference(&expected_scope_policy_fixture_unique_deny_methods)
            .count();
    let expected_scope_policy_fixture_unique_deny_only_method_count =
        expected_scope_policy_fixture_unique_deny_methods
            .difference(&expected_scope_policy_fixture_unique_allow_methods)
            .count();
    let expected_scope_policy_fixture_unique_allow_routes = scope_policy_fixture_rows
        .iter()
        .filter(|row| row.expected == "allow")
        .map(|row| (row.method.as_str(), row.path.as_str()))
        .collect::<BTreeSet<_>>();
    let expected_scope_policy_fixture_unique_deny_routes = scope_policy_fixture_rows
        .iter()
        .filter(|row| row.expected == "deny")
        .map(|row| (row.method.as_str(), row.path.as_str()))
        .collect::<BTreeSet<_>>();
    let expected_scope_policy_fixture_unique_allow_route_count =
        expected_scope_policy_fixture_unique_allow_routes.len();
    let expected_scope_policy_fixture_unique_deny_route_count =
        expected_scope_policy_fixture_unique_deny_routes.len();
    let expected_scope_policy_fixture_unique_allow_deny_overlap_route_count =
        expected_scope_policy_fixture_unique_allow_routes
            .intersection(&expected_scope_policy_fixture_unique_deny_routes)
            .count();
    let expected_scope_policy_fixture_unique_allow_only_route_count =
        expected_scope_policy_fixture_unique_allow_routes
            .difference(&expected_scope_policy_fixture_unique_deny_routes)
            .count();
    let expected_scope_policy_fixture_unique_deny_only_route_count =
        expected_scope_policy_fixture_unique_deny_routes
            .difference(&expected_scope_policy_fixture_unique_allow_routes)
            .count();
    let expected_websocket_reason_code_count = SERVICE_API_WEBSOCKET_REASON_CODES_CSV
        .split(',')
        .filter(|value| !value.is_empty())
        .count();
    let expected_lifecycle_rejection_reason_code_count =
        SERVICE_API_LIFECYCLE_REJECTION_REASON_CODES_CSV
            .split(',')
            .filter(|value| !value.is_empty())
            .count();
    assert!(metrics_response.contains(&format!(
        "kamn_service_api_cross_store_replay_reason_taxonomy_info{{version=\"{}\"}} 1",
        cross_store_replay_reason_taxonomy_version()
    )));
    assert!(metrics_response.contains(&format!(
        "kamn_service_api_cross_store_replay_reason_code_count {expected_reason_code_count}"
    )));
    assert!(metrics_response.contains(&format!(
        "kamn_service_api_auth_reason_taxonomy_info{{version=\"{}\"}} 1",
        SERVICE_API_AUTH_REASON_TAXONOMY_VERSION
    )));
    assert!(metrics_response.contains(&format!(
        "kamn_service_api_auth_reason_code_count {expected_auth_reason_code_count}"
    )));
    assert!(metrics_response.contains(&format!(
        "kamn_service_api_scope_policy_reason_taxonomy_info{{version=\"{}\"}} 1",
        SERVICE_API_SCOPE_POLICY_REASON_TAXONOMY_VERSION
    )));
    assert!(metrics_response.contains(&format!(
        "kamn_service_api_scope_policy_reason_code_count {expected_scope_reason_code_count}"
    )));
    assert!(metrics_response.contains(&format!(
        "kamn_service_api_scope_policy_fixture_schema_info{{version=\"{}\"}} 1",
        SERVICE_API_SCOPE_POLICY_FIXTURE_SCHEMA_VERSION
    )));
    assert!(metrics_response.contains(&format!(
        "kamn_service_api_scope_policy_fixture_reason_taxonomy_info{{version=\"{}\"}} 1",
        expected_scope_policy_fixture_reason_taxonomy_version
    )));
    assert!(metrics_response.contains(&format!(
        "kamn_service_api_scope_policy_fixture_reason_code_count {expected_scope_policy_fixture_reason_code_count}"
    )));
    assert!(metrics_response.contains(&format!(
        "kamn_service_api_scope_policy_fixture_row_count {expected_scope_policy_fixture_row_count}"
    )));
    assert!(metrics_response.contains(&format!(
        "kamn_service_api_scope_policy_fixture_allow_row_count {expected_scope_policy_fixture_allow_row_count}"
    )));
    assert!(metrics_response.contains(&format!(
        "kamn_service_api_scope_policy_fixture_deny_row_count {expected_scope_policy_fixture_deny_row_count}"
    )));
    assert!(metrics_response.contains(&format!(
        "kamn_service_api_scope_policy_fixture_unique_route_count {expected_scope_policy_fixture_unique_route_count}"
    )));
    assert!(metrics_response.contains(&format!(
        "kamn_service_api_scope_policy_fixture_unique_scope_count {expected_scope_policy_fixture_unique_scope_count}"
    )));
    assert!(metrics_response.contains(&format!(
        "kamn_service_api_scope_policy_fixture_unique_method_count {expected_scope_policy_fixture_unique_method_count}"
    )));
    assert!(metrics_response.contains(&format!(
        "kamn_service_api_scope_policy_fixture_unique_expected_outcome_count {expected_scope_policy_fixture_unique_expected_outcome_count}"
    )));
    assert!(metrics_response.contains(&format!(
        "kamn_service_api_scope_policy_fixture_unique_allow_scope_count {expected_scope_policy_fixture_unique_allow_scope_count}"
    )));
    assert!(metrics_response.contains(&format!(
        "kamn_service_api_scope_policy_fixture_unique_deny_scope_count {expected_scope_policy_fixture_unique_deny_scope_count}"
    )));
    assert!(metrics_response.contains(&format!(
        "kamn_service_api_scope_policy_fixture_unique_allow_deny_overlap_scope_count {expected_scope_policy_fixture_unique_allow_deny_overlap_scope_count}"
    )));
    assert!(metrics_response.contains(&format!(
        "kamn_service_api_scope_policy_fixture_unique_allow_only_scope_count {expected_scope_policy_fixture_unique_allow_only_scope_count}"
    )));
    assert!(metrics_response.contains(&format!(
        "kamn_service_api_scope_policy_fixture_unique_deny_only_scope_count {expected_scope_policy_fixture_unique_deny_only_scope_count}"
    )));
    assert!(metrics_response.contains(&format!(
        "kamn_service_api_scope_policy_fixture_unique_allow_deny_overlap_method_count {expected_scope_policy_fixture_unique_allow_deny_overlap_method_count}"
    )));
    assert!(metrics_response.contains(&format!(
        "kamn_service_api_scope_policy_fixture_unique_allow_only_method_count {expected_scope_policy_fixture_unique_allow_only_method_count}"
    )));
    assert!(metrics_response.contains(&format!(
        "kamn_service_api_scope_policy_fixture_unique_deny_only_method_count {expected_scope_policy_fixture_unique_deny_only_method_count}"
    )));
    assert!(metrics_response.contains(&format!(
        "kamn_service_api_scope_policy_fixture_unique_allow_route_count {expected_scope_policy_fixture_unique_allow_route_count}"
    )));
    assert!(metrics_response.contains(&format!(
        "kamn_service_api_scope_policy_fixture_unique_deny_route_count {expected_scope_policy_fixture_unique_deny_route_count}"
    )));
    assert!(metrics_response.contains(&format!(
        "kamn_service_api_scope_policy_fixture_unique_allow_deny_overlap_route_count {expected_scope_policy_fixture_unique_allow_deny_overlap_route_count}"
    )));
    assert!(metrics_response.contains(&format!(
        "kamn_service_api_scope_policy_fixture_unique_allow_only_route_count {expected_scope_policy_fixture_unique_allow_only_route_count}"
    )));
    assert!(metrics_response.contains(&format!(
        "kamn_service_api_scope_policy_fixture_unique_deny_only_route_count {expected_scope_policy_fixture_unique_deny_only_route_count}"
    )));
    assert!(metrics_response.contains(&format!(
        "kamn_service_api_route_authz_matrix_schema_info{{version=\"{}\"}} 1",
        SERVICE_API_ROUTE_AUTHZ_MATRIX_SCHEMA_VERSION
    )));
    assert!(metrics_response.contains(&format!(
        "kamn_service_api_route_authz_matrix_total_route_count {}",
        SERVICE_API_ROUTE_AUTHZ_MATRIX_TOTAL_ROUTE_COUNT
    )));
    assert!(metrics_response.contains(&format!(
        "kamn_service_api_route_authz_matrix_public_route_count {}",
        SERVICE_API_ROUTE_AUTHZ_MATRIX_PUBLIC_ROUTE_COUNT
    )));
    assert!(metrics_response.contains(&format!(
        "kamn_service_api_route_authz_matrix_protected_route_count {}",
        SERVICE_API_ROUTE_AUTHZ_MATRIX_PROTECTED_ROUTE_COUNT
    )));
    assert!(metrics_response.contains(&format!(
        "kamn_service_api_websocket_reason_taxonomy_info{{version=\"{}\"}} 1",
        SERVICE_API_WEBSOCKET_REASON_TAXONOMY_VERSION
    )));
    assert!(metrics_response.contains(&format!(
        "kamn_service_api_websocket_reason_code_count {expected_websocket_reason_code_count}"
    )));
    assert!(metrics_response.contains(&format!(
        "kamn_service_api_lifecycle_rejection_reason_taxonomy_info{{version=\"{}\"}} 1",
        SERVICE_API_LIFECYCLE_REJECTION_REASON_TAXONOMY_VERSION
    )));
    assert!(metrics_response.contains(&format!(
        "kamn_service_api_lifecycle_rejection_reason_code_count {expected_lifecycle_rejection_reason_code_count}"
    )));
    assert!(metrics_response
        .contains("kamn_service_api_observability_source{source=\"service-api-runtime\"} 1"));

    let server_result = server.join().expect("endpoint thread should complete");
    assert!(
        server_result.is_ok(),
        "service api endpoint should stop cleanly after configured request budget"
    );
}

#[test]
fn regression_service_api_runtime_observability_projects_live_metrics_under_traffic() {
    // Regression: #5903
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
        max_requests: 4,
        idle_timeout_ms: 2_000,
        body_limit_bytes: DEFAULT_SERVICE_API_BODY_LIMIT_BYTES,
        concurrency_limit: DEFAULT_SERVICE_API_CONCURRENCY_LIMIT,
        rate_limit_per_second: DEFAULT_SERVICE_API_RATE_LIMIT_PER_SECOND,
    };

    let server_snapshot = snapshot.clone();
    let server =
        thread::spawn(move || serve_service_api_endpoint(&endpoint_config, &server_snapshot));
    wait_for_endpoint_ready(bind_addr.as_str());

    let message_body = "{\"message\":\"runtime-observability\"}";
    let sender_did = "kamn:did:agent:runtime-observability";
    let sender_nonce = 1_u64;
    let state_hash = format!(
        "service-api:{}:{}",
        snapshot.chain_id.as_str(),
        snapshot.chain_version.as_str()
    );
    let signature = service_api_request_signature_for_fields(
        sender_did,
        sender_nonce,
        state_hash.as_str(),
        message_body,
    );
    let send_response = send_http_request_with_headers(
        bind_addr.as_str(),
        "POST",
        "/v1/messages/send",
        message_body,
        &[
            ("X-KAMN-Sender-DID", sender_did),
            ("X-KAMN-Request-Nonce", "1"),
            ("X-KAMN-Request-Signature", signature.as_str()),
            ("X-KAMN-Authz-Scope", "messages:write"),
        ],
    );
    assert!(send_response.contains("HTTP/1.1 202 Accepted"));

    let unauth_response = send_http_request(
        bind_addr.as_str(),
        "POST",
        "/v1/messages/send",
        "{\"message\":\"unauth\"}",
    );
    assert!(unauth_response.contains("HTTP/1.1 401 Unauthorized"));

    let health_response = send_http_request(bind_addr.as_str(), "GET", "/healthz", "");
    let metrics_response = send_http_request(bind_addr.as_str(), "GET", "/metrics", "");
    assert!(health_response.contains("HTTP/1.1 200 OK"));
    assert!(metrics_response.contains("HTTP/1.1 200 OK"));

    let health_payload: ServiceApiHealthBody =
        parse_service_api_payload(extract_http_response_body(health_response.as_str()))
            .expect("health payload should deserialize");
    assert_eq!(
        health_payload.observability_source, "service-api-runtime",
        "health should expose runtime observability source once traffic is recorded"
    );
    assert!(
        matches!(
            health_payload.observability_health.as_str(),
            "healthy" | "degraded" | "critical"
        ),
        "runtime health must map to known observability taxonomy"
    );

    let throughput_tps = parse_scalar_metric_value(
        metrics_response.as_str(),
        "kamn_service_api_observability_throughput_tps",
    )
    .expect("throughput metric should be present");
    assert!(throughput_tps > 0, "throughput should be traffic-derived");

    let error_rate_bps = parse_scalar_metric_value(
        metrics_response.as_str(),
        "kamn_service_api_observability_error_rate_bps",
    )
    .expect("error rate metric should be present");
    assert!(
        error_rate_bps > 0,
        "error rate should capture unauthorized request outcomes"
    );

    let latency_p50_ms = parse_scalar_metric_value(
        metrics_response.as_str(),
        "kamn_service_api_observability_latency_p50_ms",
    )
    .expect("latency p50 metric should be present");
    let latency_p99_ms = parse_scalar_metric_value(
        metrics_response.as_str(),
        "kamn_service_api_observability_latency_p99_ms",
    )
    .expect("latency p99 metric should be present");
    assert!(latency_p50_ms > 0, "latency p50 should be runtime-derived");
    assert!(latency_p99_ms > 0, "latency p99 should be runtime-derived");

    let availability_bps = parse_scalar_metric_value(
        metrics_response.as_str(),
        "kamn_service_api_observability_availability_bps",
    )
    .expect("availability metric should be present");
    assert!(
        availability_bps < 10_000,
        "availability should decrease when error outcomes occur"
    );

    let server_result = server.join().expect("endpoint thread should complete");
    assert!(
        server_result.is_ok(),
        "service api endpoint should stop cleanly after configured request budget"
    );
}

#[test]
fn integration_service_api_endpoint_async_runtime_handles_concurrent_http_routes() {
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
        max_requests: 5,
        idle_timeout_ms: 2_000,
        body_limit_bytes: DEFAULT_SERVICE_API_BODY_LIMIT_BYTES,
        concurrency_limit: DEFAULT_SERVICE_API_CONCURRENCY_LIMIT,
        rate_limit_per_second: DEFAULT_SERVICE_API_RATE_LIMIT_PER_SECOND,
    };
    let server_snapshot = snapshot.clone();
    let server =
        thread::spawn(move || serve_service_api_endpoint(&endpoint_config, &server_snapshot));
    wait_for_endpoint_ready(bind_addr.as_str());

    let sender_did = "kamn:did:agent:async-http-client";
    let body_one = "{\"message\":\"async-route-1\"}".to_owned();
    let body_two = "{\"message\":\"async-route-2\"}".to_owned();
    let state_hash = format!(
        "service-api:{}:{}",
        snapshot.chain_id.as_str(),
        snapshot.chain_version.as_str()
    );
    let signature_one = service_api_request_signature_for_fields(
        sender_did,
        900,
        state_hash.as_str(),
        body_one.as_str(),
    );
    let signature_two = service_api_request_signature_for_fields(
        sender_did,
        901,
        state_hash.as_str(),
        body_two.as_str(),
    );

    let runtime = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(2)
        .enable_all()
        .build()
        .expect("async runtime should initialize");
    let (health_response, metrics_response, send_one_response, send_two_response) = runtime
        .block_on(async {
            tokio::join!(
                send_http_request_with_headers_async(
                    bind_addr.as_str(),
                    "GET",
                    "/healthz",
                    "",
                    &[]
                ),
                send_http_request_with_headers_async(
                    bind_addr.as_str(),
                    "GET",
                    "/metrics",
                    "",
                    &[]
                ),
                async {
                    let headers = [
                        ("X-KAMN-Sender-DID", sender_did),
                        ("X-KAMN-Request-Nonce", "900"),
                        ("X-KAMN-Request-Signature", signature_one.as_str()),
                    ];
                    send_http_request_with_headers_async(
                        bind_addr.as_str(),
                        "POST",
                        "/v1/messages/send",
                        body_one.as_str(),
                        &headers,
                    )
                    .await
                },
                async {
                    let headers = [
                        ("X-KAMN-Sender-DID", sender_did),
                        ("X-KAMN-Request-Nonce", "901"),
                        ("X-KAMN-Request-Signature", signature_two.as_str()),
                    ];
                    send_http_request_with_headers_async(
                        bind_addr.as_str(),
                        "POST",
                        "/v1/messages/send",
                        body_two.as_str(),
                        &headers,
                    )
                    .await
                }
            )
        });

    let health_response = health_response.expect("async health request should succeed");
    let metrics_response = metrics_response.expect("async metrics request should succeed");
    let send_one_response = send_one_response.expect("async send request one should succeed");
    let send_two_response = send_two_response.expect("async send request two should succeed");

    assert!(health_response.contains("HTTP/1.1 200 OK"));
    assert!(metrics_response.contains("HTTP/1.1 200 OK"));
    assert!(send_one_response.contains("HTTP/1.1 202 Accepted"));
    assert!(send_two_response.contains("HTTP/1.1 202 Accepted"));

    let server_result = server.join().expect("endpoint thread should complete");
    let ended_cleanly_or_timeout = match &server_result {
        Ok(()) => true,
        Err(error) => error.contains("service api timed out after"),
    };
    assert!(
        ended_cleanly_or_timeout,
        "service api endpoint should end via request budget completion or idle-timeout fail-close after async concurrent request lane: {server_result:?}"
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
    let expected_reason_code_count = cross_store_replay_reason_codes_csv()
        .split(',')
        .filter(|value| !value.is_empty())
        .count();
    let expected_auth_reason_code_count = SERVICE_API_AUTH_REASON_CODES_CSV
        .split(',')
        .filter(|value| !value.is_empty())
        .count();
    let expected_scope_reason_code_count = SERVICE_API_SCOPE_POLICY_REASON_CODES_CSV
        .split(',')
        .filter(|value| !value.is_empty())
        .count();
    let (scope_policy_fixture_metadata, scope_policy_fixture_rows) =
        parse_service_api_scope_policy_fixture(SERVICE_API_SCOPE_POLICY_FIXTURE);
    let expected_scope_policy_fixture_reason_taxonomy_version = scope_policy_fixture_metadata
        .get("scope_policy_reason_taxonomy_version")
        .map(String::as_str)
        .unwrap_or_default();
    let expected_scope_policy_fixture_reason_code_count = scope_policy_fixture_metadata
        .get("scope_policy_reason_codes_csv")
        .map(|value| {
            value
                .split(',')
                .filter(|entry| !entry.trim().is_empty())
                .count()
        })
        .unwrap_or_default();
    let expected_scope_policy_fixture_row_count = scope_policy_fixture_rows.len();
    let expected_scope_policy_fixture_allow_row_count = scope_policy_fixture_rows
        .iter()
        .filter(|row| row.expected == "allow")
        .count();
    let expected_scope_policy_fixture_deny_row_count = scope_policy_fixture_rows
        .iter()
        .filter(|row| row.expected == "deny")
        .count();
    let expected_scope_policy_fixture_unique_route_count = scope_policy_fixture_rows
        .iter()
        .map(|row| (row.method.as_str(), row.path.as_str()))
        .collect::<BTreeSet<_>>()
        .len();
    let expected_scope_policy_fixture_unique_scope_count = scope_policy_fixture_rows
        .iter()
        .map(|row| row.scope.as_str())
        .collect::<BTreeSet<_>>()
        .len();
    let expected_scope_policy_fixture_unique_method_count = scope_policy_fixture_rows
        .iter()
        .map(|row| row.method.as_str())
        .collect::<BTreeSet<_>>()
        .len();
    let expected_scope_policy_fixture_unique_expected_outcome_count = scope_policy_fixture_rows
        .iter()
        .map(|row| row.expected.as_str())
        .collect::<BTreeSet<_>>()
        .len();
    let expected_scope_policy_fixture_unique_allow_scopes = scope_policy_fixture_rows
        .iter()
        .filter(|row| row.expected == "allow")
        .map(|row| row.scope.as_str())
        .collect::<BTreeSet<_>>();
    let expected_scope_policy_fixture_unique_deny_scopes = scope_policy_fixture_rows
        .iter()
        .filter(|row| row.expected == "deny")
        .map(|row| row.scope.as_str())
        .collect::<BTreeSet<_>>();
    let expected_scope_policy_fixture_unique_allow_scope_count =
        expected_scope_policy_fixture_unique_allow_scopes.len();
    let expected_scope_policy_fixture_unique_deny_scope_count =
        expected_scope_policy_fixture_unique_deny_scopes.len();
    let expected_scope_policy_fixture_unique_allow_deny_overlap_scope_count =
        expected_scope_policy_fixture_unique_allow_scopes
            .intersection(&expected_scope_policy_fixture_unique_deny_scopes)
            .count();
    let expected_scope_policy_fixture_unique_allow_only_scope_count =
        expected_scope_policy_fixture_unique_allow_scopes
            .difference(&expected_scope_policy_fixture_unique_deny_scopes)
            .count();
    let expected_scope_policy_fixture_unique_deny_only_scope_count =
        expected_scope_policy_fixture_unique_deny_scopes
            .difference(&expected_scope_policy_fixture_unique_allow_scopes)
            .count();
    let expected_scope_policy_fixture_unique_allow_methods = scope_policy_fixture_rows
        .iter()
        .filter(|row| row.expected == "allow")
        .map(|row| row.method.as_str())
        .collect::<BTreeSet<_>>();
    let expected_scope_policy_fixture_unique_deny_methods = scope_policy_fixture_rows
        .iter()
        .filter(|row| row.expected == "deny")
        .map(|row| row.method.as_str())
        .collect::<BTreeSet<_>>();
    let expected_scope_policy_fixture_unique_allow_deny_overlap_method_count =
        expected_scope_policy_fixture_unique_allow_methods
            .intersection(&expected_scope_policy_fixture_unique_deny_methods)
            .count();
    let expected_scope_policy_fixture_unique_allow_only_method_count =
        expected_scope_policy_fixture_unique_allow_methods
            .difference(&expected_scope_policy_fixture_unique_deny_methods)
            .count();
    let expected_scope_policy_fixture_unique_deny_only_method_count =
        expected_scope_policy_fixture_unique_deny_methods
            .difference(&expected_scope_policy_fixture_unique_allow_methods)
            .count();
    let expected_scope_policy_fixture_unique_allow_routes = scope_policy_fixture_rows
        .iter()
        .filter(|row| row.expected == "allow")
        .map(|row| (row.method.as_str(), row.path.as_str()))
        .collect::<BTreeSet<_>>();
    let expected_scope_policy_fixture_unique_deny_routes = scope_policy_fixture_rows
        .iter()
        .filter(|row| row.expected == "deny")
        .map(|row| (row.method.as_str(), row.path.as_str()))
        .collect::<BTreeSet<_>>();
    let expected_scope_policy_fixture_unique_allow_route_count =
        expected_scope_policy_fixture_unique_allow_routes.len();
    let expected_scope_policy_fixture_unique_deny_route_count =
        expected_scope_policy_fixture_unique_deny_routes.len();
    let expected_scope_policy_fixture_unique_allow_deny_overlap_route_count =
        expected_scope_policy_fixture_unique_allow_routes
            .intersection(&expected_scope_policy_fixture_unique_deny_routes)
            .count();
    let expected_scope_policy_fixture_unique_allow_only_route_count =
        expected_scope_policy_fixture_unique_allow_routes
            .difference(&expected_scope_policy_fixture_unique_deny_routes)
            .count();
    let expected_scope_policy_fixture_unique_deny_only_route_count =
        expected_scope_policy_fixture_unique_deny_routes
            .difference(&expected_scope_policy_fixture_unique_allow_routes)
            .count();
    let expected_websocket_reason_code_count = SERVICE_API_WEBSOCKET_REASON_CODES_CSV
        .split(',')
        .filter(|value| !value.is_empty())
        .count();
    let expected_lifecycle_rejection_reason_code_count =
        SERVICE_API_LIFECYCLE_REJECTION_REASON_CODES_CSV
            .split(',')
            .filter(|value| !value.is_empty())
            .count();
    assert!(metrics_response.contains(&format!(
        "kamn_service_api_cross_store_replay_reason_taxonomy_info{{version=\"{}\"}} 1",
        cross_store_replay_reason_taxonomy_version()
    )));
    assert!(metrics_response.contains(&format!(
        "kamn_service_api_cross_store_replay_reason_code_count {expected_reason_code_count}"
    )));
    assert!(metrics_response.contains(&format!(
        "kamn_service_api_auth_reason_taxonomy_info{{version=\"{}\"}} 1",
        SERVICE_API_AUTH_REASON_TAXONOMY_VERSION
    )));
    assert!(metrics_response.contains(&format!(
        "kamn_service_api_auth_reason_code_count {expected_auth_reason_code_count}"
    )));
    assert!(metrics_response.contains(&format!(
        "kamn_service_api_scope_policy_reason_taxonomy_info{{version=\"{}\"}} 1",
        SERVICE_API_SCOPE_POLICY_REASON_TAXONOMY_VERSION
    )));
    assert!(metrics_response.contains(&format!(
        "kamn_service_api_scope_policy_reason_code_count {expected_scope_reason_code_count}"
    )));
    assert!(metrics_response.contains(&format!(
        "kamn_service_api_scope_policy_fixture_schema_info{{version=\"{}\"}} 1",
        SERVICE_API_SCOPE_POLICY_FIXTURE_SCHEMA_VERSION
    )));
    assert!(metrics_response.contains(&format!(
        "kamn_service_api_scope_policy_fixture_reason_taxonomy_info{{version=\"{}\"}} 1",
        expected_scope_policy_fixture_reason_taxonomy_version
    )));
    assert!(metrics_response.contains(&format!(
        "kamn_service_api_scope_policy_fixture_reason_code_count {expected_scope_policy_fixture_reason_code_count}"
    )));
    assert!(metrics_response.contains(&format!(
        "kamn_service_api_scope_policy_fixture_row_count {expected_scope_policy_fixture_row_count}"
    )));
    assert!(metrics_response.contains(&format!(
        "kamn_service_api_scope_policy_fixture_allow_row_count {expected_scope_policy_fixture_allow_row_count}"
    )));
    assert!(metrics_response.contains(&format!(
        "kamn_service_api_scope_policy_fixture_deny_row_count {expected_scope_policy_fixture_deny_row_count}"
    )));
    assert!(metrics_response.contains(&format!(
        "kamn_service_api_scope_policy_fixture_unique_route_count {expected_scope_policy_fixture_unique_route_count}"
    )));
    assert!(metrics_response.contains(&format!(
        "kamn_service_api_scope_policy_fixture_unique_scope_count {expected_scope_policy_fixture_unique_scope_count}"
    )));
    assert!(metrics_response.contains(&format!(
        "kamn_service_api_scope_policy_fixture_unique_method_count {expected_scope_policy_fixture_unique_method_count}"
    )));
    assert!(metrics_response.contains(&format!(
        "kamn_service_api_scope_policy_fixture_unique_expected_outcome_count {expected_scope_policy_fixture_unique_expected_outcome_count}"
    )));
    assert!(metrics_response.contains(&format!(
        "kamn_service_api_scope_policy_fixture_unique_allow_scope_count {expected_scope_policy_fixture_unique_allow_scope_count}"
    )));
    assert!(metrics_response.contains(&format!(
        "kamn_service_api_scope_policy_fixture_unique_deny_scope_count {expected_scope_policy_fixture_unique_deny_scope_count}"
    )));
    assert!(metrics_response.contains(&format!(
        "kamn_service_api_scope_policy_fixture_unique_allow_deny_overlap_scope_count {expected_scope_policy_fixture_unique_allow_deny_overlap_scope_count}"
    )));
    assert!(metrics_response.contains(&format!(
        "kamn_service_api_scope_policy_fixture_unique_allow_only_scope_count {expected_scope_policy_fixture_unique_allow_only_scope_count}"
    )));
    assert!(metrics_response.contains(&format!(
        "kamn_service_api_scope_policy_fixture_unique_deny_only_scope_count {expected_scope_policy_fixture_unique_deny_only_scope_count}"
    )));
    assert!(metrics_response.contains(&format!(
        "kamn_service_api_scope_policy_fixture_unique_allow_deny_overlap_method_count {expected_scope_policy_fixture_unique_allow_deny_overlap_method_count}"
    )));
    assert!(metrics_response.contains(&format!(
        "kamn_service_api_scope_policy_fixture_unique_allow_only_method_count {expected_scope_policy_fixture_unique_allow_only_method_count}"
    )));
    assert!(metrics_response.contains(&format!(
        "kamn_service_api_scope_policy_fixture_unique_deny_only_method_count {expected_scope_policy_fixture_unique_deny_only_method_count}"
    )));
    assert!(metrics_response.contains(&format!(
        "kamn_service_api_scope_policy_fixture_unique_allow_route_count {expected_scope_policy_fixture_unique_allow_route_count}"
    )));
    assert!(metrics_response.contains(&format!(
        "kamn_service_api_scope_policy_fixture_unique_deny_route_count {expected_scope_policy_fixture_unique_deny_route_count}"
    )));
    assert!(metrics_response.contains(&format!(
        "kamn_service_api_scope_policy_fixture_unique_allow_deny_overlap_route_count {expected_scope_policy_fixture_unique_allow_deny_overlap_route_count}"
    )));
    assert!(metrics_response.contains(&format!(
        "kamn_service_api_scope_policy_fixture_unique_allow_only_route_count {expected_scope_policy_fixture_unique_allow_only_route_count}"
    )));
    assert!(metrics_response.contains(&format!(
        "kamn_service_api_scope_policy_fixture_unique_deny_only_route_count {expected_scope_policy_fixture_unique_deny_only_route_count}"
    )));
    assert!(metrics_response.contains(&format!(
        "kamn_service_api_route_authz_matrix_schema_info{{version=\"{}\"}} 1",
        SERVICE_API_ROUTE_AUTHZ_MATRIX_SCHEMA_VERSION
    )));
    assert!(metrics_response.contains(&format!(
        "kamn_service_api_route_authz_matrix_total_route_count {}",
        SERVICE_API_ROUTE_AUTHZ_MATRIX_TOTAL_ROUTE_COUNT
    )));
    assert!(metrics_response.contains(&format!(
        "kamn_service_api_route_authz_matrix_public_route_count {}",
        SERVICE_API_ROUTE_AUTHZ_MATRIX_PUBLIC_ROUTE_COUNT
    )));
    assert!(metrics_response.contains(&format!(
        "kamn_service_api_route_authz_matrix_protected_route_count {}",
        SERVICE_API_ROUTE_AUTHZ_MATRIX_PROTECTED_ROUTE_COUNT
    )));
    assert!(metrics_response.contains(&format!(
        "kamn_service_api_websocket_reason_taxonomy_info{{version=\"{}\"}} 1",
        SERVICE_API_WEBSOCKET_REASON_TAXONOMY_VERSION
    )));
    assert!(metrics_response.contains(&format!(
        "kamn_service_api_websocket_reason_code_count {expected_websocket_reason_code_count}"
    )));
    assert!(metrics_response.contains(&format!(
        "kamn_service_api_lifecycle_rejection_reason_taxonomy_info{{version=\"{}\"}} 1",
        SERVICE_API_LIFECYCLE_REJECTION_REASON_TAXONOMY_VERSION
    )));
    assert!(metrics_response.contains(&format!(
        "kamn_service_api_lifecycle_rejection_reason_code_count {expected_lifecycle_rejection_reason_code_count}"
    )));
    assert!(metrics_response
        .contains("kamn_service_api_observability_source{source=\"service-api-runtime\"} 1"));

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
    let signature = service_api_request_signature_for_fields(
        sender_did,
        sender_nonce,
        state_hash.as_str(),
        message_body,
    );
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
    assert!(second_response
        .contains("kamn_service_api_observability_source{source=\"service-api-runtime\"} 1"));

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
    let signature = service_api_request_signature_for_fields(
        sender_did,
        sender_nonce,
        state_hash.as_str(),
        message_body,
    );
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
    let expected_reason_code_count = cross_store_replay_reason_codes_csv()
        .split(',')
        .filter(|value| !value.is_empty())
        .count();
    let expected_auth_reason_code_count = SERVICE_API_AUTH_REASON_CODES_CSV
        .split(',')
        .filter(|value| !value.is_empty())
        .count();
    let expected_scope_reason_code_count = SERVICE_API_SCOPE_POLICY_REASON_CODES_CSV
        .split(',')
        .filter(|value| !value.is_empty())
        .count();
    let (scope_policy_fixture_metadata, scope_policy_fixture_rows) =
        parse_service_api_scope_policy_fixture(SERVICE_API_SCOPE_POLICY_FIXTURE);
    let expected_scope_policy_fixture_reason_taxonomy_version = scope_policy_fixture_metadata
        .get("scope_policy_reason_taxonomy_version")
        .map(String::as_str)
        .unwrap_or_default();
    let expected_scope_policy_fixture_reason_code_count = scope_policy_fixture_metadata
        .get("scope_policy_reason_codes_csv")
        .map(|value| {
            value
                .split(',')
                .filter(|entry| !entry.trim().is_empty())
                .count()
        })
        .unwrap_or_default();
    let expected_scope_policy_fixture_row_count = scope_policy_fixture_rows.len();
    let expected_scope_policy_fixture_allow_row_count = scope_policy_fixture_rows
        .iter()
        .filter(|row| row.expected == "allow")
        .count();
    let expected_scope_policy_fixture_deny_row_count = scope_policy_fixture_rows
        .iter()
        .filter(|row| row.expected == "deny")
        .count();
    let expected_scope_policy_fixture_unique_route_count = scope_policy_fixture_rows
        .iter()
        .map(|row| (row.method.as_str(), row.path.as_str()))
        .collect::<BTreeSet<_>>()
        .len();
    let expected_scope_policy_fixture_unique_scope_count = scope_policy_fixture_rows
        .iter()
        .map(|row| row.scope.as_str())
        .collect::<BTreeSet<_>>()
        .len();
    let expected_scope_policy_fixture_unique_method_count = scope_policy_fixture_rows
        .iter()
        .map(|row| row.method.as_str())
        .collect::<BTreeSet<_>>()
        .len();
    let expected_scope_policy_fixture_unique_expected_outcome_count = scope_policy_fixture_rows
        .iter()
        .map(|row| row.expected.as_str())
        .collect::<BTreeSet<_>>()
        .len();
    let expected_scope_policy_fixture_unique_allow_scopes = scope_policy_fixture_rows
        .iter()
        .filter(|row| row.expected == "allow")
        .map(|row| row.scope.as_str())
        .collect::<BTreeSet<_>>();
    let expected_scope_policy_fixture_unique_deny_scopes = scope_policy_fixture_rows
        .iter()
        .filter(|row| row.expected == "deny")
        .map(|row| row.scope.as_str())
        .collect::<BTreeSet<_>>();
    let expected_scope_policy_fixture_unique_allow_scope_count =
        expected_scope_policy_fixture_unique_allow_scopes.len();
    let expected_scope_policy_fixture_unique_deny_scope_count =
        expected_scope_policy_fixture_unique_deny_scopes.len();
    let expected_scope_policy_fixture_unique_allow_deny_overlap_scope_count =
        expected_scope_policy_fixture_unique_allow_scopes
            .intersection(&expected_scope_policy_fixture_unique_deny_scopes)
            .count();
    let expected_scope_policy_fixture_unique_allow_only_scope_count =
        expected_scope_policy_fixture_unique_allow_scopes
            .difference(&expected_scope_policy_fixture_unique_deny_scopes)
            .count();
    let expected_scope_policy_fixture_unique_deny_only_scope_count =
        expected_scope_policy_fixture_unique_deny_scopes
            .difference(&expected_scope_policy_fixture_unique_allow_scopes)
            .count();
    let expected_scope_policy_fixture_unique_allow_methods = scope_policy_fixture_rows
        .iter()
        .filter(|row| row.expected == "allow")
        .map(|row| row.method.as_str())
        .collect::<BTreeSet<_>>();
    let expected_scope_policy_fixture_unique_deny_methods = scope_policy_fixture_rows
        .iter()
        .filter(|row| row.expected == "deny")
        .map(|row| row.method.as_str())
        .collect::<BTreeSet<_>>();
    let expected_scope_policy_fixture_unique_allow_deny_overlap_method_count =
        expected_scope_policy_fixture_unique_allow_methods
            .intersection(&expected_scope_policy_fixture_unique_deny_methods)
            .count();
    let expected_scope_policy_fixture_unique_allow_only_method_count =
        expected_scope_policy_fixture_unique_allow_methods
            .difference(&expected_scope_policy_fixture_unique_deny_methods)
            .count();
    let expected_scope_policy_fixture_unique_deny_only_method_count =
        expected_scope_policy_fixture_unique_deny_methods
            .difference(&expected_scope_policy_fixture_unique_allow_methods)
            .count();
    let expected_scope_policy_fixture_unique_allow_routes = scope_policy_fixture_rows
        .iter()
        .filter(|row| row.expected == "allow")
        .map(|row| (row.method.as_str(), row.path.as_str()))
        .collect::<BTreeSet<_>>();
    let expected_scope_policy_fixture_unique_deny_routes = scope_policy_fixture_rows
        .iter()
        .filter(|row| row.expected == "deny")
        .map(|row| (row.method.as_str(), row.path.as_str()))
        .collect::<BTreeSet<_>>();
    let expected_scope_policy_fixture_unique_allow_route_count =
        expected_scope_policy_fixture_unique_allow_routes.len();
    let expected_scope_policy_fixture_unique_deny_route_count =
        expected_scope_policy_fixture_unique_deny_routes.len();
    let expected_scope_policy_fixture_unique_allow_deny_overlap_route_count =
        expected_scope_policy_fixture_unique_allow_routes
            .intersection(&expected_scope_policy_fixture_unique_deny_routes)
            .count();
    let expected_scope_policy_fixture_unique_allow_only_route_count =
        expected_scope_policy_fixture_unique_allow_routes
            .difference(&expected_scope_policy_fixture_unique_deny_routes)
            .count();
    let expected_scope_policy_fixture_unique_deny_only_route_count =
        expected_scope_policy_fixture_unique_deny_routes
            .difference(&expected_scope_policy_fixture_unique_allow_routes)
            .count();
    let expected_websocket_reason_code_count = SERVICE_API_WEBSOCKET_REASON_CODES_CSV
        .split(',')
        .filter(|value| !value.is_empty())
        .count();
    let expected_lifecycle_rejection_reason_code_count =
        SERVICE_API_LIFECYCLE_REJECTION_REASON_CODES_CSV
            .split(',')
            .filter(|value| !value.is_empty())
            .count();
    assert!(metrics_response.body.contains(&format!(
        "kamn_service_api_cross_store_replay_reason_taxonomy_info{{version=\"{}\"}} 1",
        cross_store_replay_reason_taxonomy_version()
    )));
    assert!(metrics_response.body.contains(&format!(
        "kamn_service_api_cross_store_replay_reason_code_count {expected_reason_code_count}"
    )));
    assert!(metrics_response.body.contains(&format!(
        "kamn_service_api_auth_reason_taxonomy_info{{version=\"{}\"}} 1",
        SERVICE_API_AUTH_REASON_TAXONOMY_VERSION
    )));
    assert!(metrics_response.body.contains(&format!(
        "kamn_service_api_auth_reason_code_count {expected_auth_reason_code_count}"
    )));
    assert!(metrics_response.body.contains(&format!(
        "kamn_service_api_scope_policy_reason_taxonomy_info{{version=\"{}\"}} 1",
        SERVICE_API_SCOPE_POLICY_REASON_TAXONOMY_VERSION
    )));
    assert!(metrics_response.body.contains(&format!(
        "kamn_service_api_scope_policy_reason_code_count {expected_scope_reason_code_count}"
    )));
    assert!(metrics_response.body.contains(&format!(
        "kamn_service_api_scope_policy_fixture_schema_info{{version=\"{}\"}} 1",
        SERVICE_API_SCOPE_POLICY_FIXTURE_SCHEMA_VERSION
    )));
    assert!(metrics_response.body.contains(&format!(
        "kamn_service_api_scope_policy_fixture_reason_taxonomy_info{{version=\"{}\"}} 1",
        expected_scope_policy_fixture_reason_taxonomy_version
    )));
    assert!(metrics_response.body.contains(&format!(
        "kamn_service_api_scope_policy_fixture_reason_code_count {expected_scope_policy_fixture_reason_code_count}"
    )));
    assert!(metrics_response.body.contains(&format!(
        "kamn_service_api_scope_policy_fixture_row_count {expected_scope_policy_fixture_row_count}"
    )));
    assert!(metrics_response.body.contains(&format!(
        "kamn_service_api_scope_policy_fixture_allow_row_count {expected_scope_policy_fixture_allow_row_count}"
    )));
    assert!(metrics_response.body.contains(&format!(
        "kamn_service_api_scope_policy_fixture_deny_row_count {expected_scope_policy_fixture_deny_row_count}"
    )));
    assert!(metrics_response.body.contains(&format!(
        "kamn_service_api_scope_policy_fixture_unique_route_count {expected_scope_policy_fixture_unique_route_count}"
    )));
    assert!(metrics_response.body.contains(&format!(
        "kamn_service_api_scope_policy_fixture_unique_scope_count {expected_scope_policy_fixture_unique_scope_count}"
    )));
    assert!(metrics_response.body.contains(&format!(
        "kamn_service_api_scope_policy_fixture_unique_method_count {expected_scope_policy_fixture_unique_method_count}"
    )));
    assert!(metrics_response.body.contains(&format!(
        "kamn_service_api_scope_policy_fixture_unique_expected_outcome_count {expected_scope_policy_fixture_unique_expected_outcome_count}"
    )));
    assert!(metrics_response.body.contains(&format!(
        "kamn_service_api_scope_policy_fixture_unique_allow_scope_count {expected_scope_policy_fixture_unique_allow_scope_count}"
    )));
    assert!(metrics_response.body.contains(&format!(
        "kamn_service_api_scope_policy_fixture_unique_deny_scope_count {expected_scope_policy_fixture_unique_deny_scope_count}"
    )));
    assert!(metrics_response.body.contains(&format!(
        "kamn_service_api_scope_policy_fixture_unique_allow_deny_overlap_scope_count {expected_scope_policy_fixture_unique_allow_deny_overlap_scope_count}"
    )));
    assert!(metrics_response.body.contains(&format!(
        "kamn_service_api_scope_policy_fixture_unique_allow_only_scope_count {expected_scope_policy_fixture_unique_allow_only_scope_count}"
    )));
    assert!(metrics_response.body.contains(&format!(
        "kamn_service_api_scope_policy_fixture_unique_deny_only_scope_count {expected_scope_policy_fixture_unique_deny_only_scope_count}"
    )));
    assert!(metrics_response.body.contains(&format!(
        "kamn_service_api_scope_policy_fixture_unique_allow_deny_overlap_method_count {expected_scope_policy_fixture_unique_allow_deny_overlap_method_count}"
    )));
    assert!(metrics_response.body.contains(&format!(
        "kamn_service_api_scope_policy_fixture_unique_allow_only_method_count {expected_scope_policy_fixture_unique_allow_only_method_count}"
    )));
    assert!(metrics_response.body.contains(&format!(
        "kamn_service_api_scope_policy_fixture_unique_deny_only_method_count {expected_scope_policy_fixture_unique_deny_only_method_count}"
    )));
    assert!(metrics_response.body.contains(&format!(
        "kamn_service_api_scope_policy_fixture_unique_allow_route_count {expected_scope_policy_fixture_unique_allow_route_count}"
    )));
    assert!(metrics_response.body.contains(&format!(
        "kamn_service_api_scope_policy_fixture_unique_deny_route_count {expected_scope_policy_fixture_unique_deny_route_count}"
    )));
    assert!(metrics_response.body.contains(&format!(
        "kamn_service_api_scope_policy_fixture_unique_allow_deny_overlap_route_count {expected_scope_policy_fixture_unique_allow_deny_overlap_route_count}"
    )));
    assert!(metrics_response.body.contains(&format!(
        "kamn_service_api_scope_policy_fixture_unique_allow_only_route_count {expected_scope_policy_fixture_unique_allow_only_route_count}"
    )));
    assert!(metrics_response.body.contains(&format!(
        "kamn_service_api_scope_policy_fixture_unique_deny_only_route_count {expected_scope_policy_fixture_unique_deny_only_route_count}"
    )));
    assert!(metrics_response.body.contains(&format!(
        "kamn_service_api_route_authz_matrix_schema_info{{version=\"{}\"}} 1",
        SERVICE_API_ROUTE_AUTHZ_MATRIX_SCHEMA_VERSION
    )));
    assert!(metrics_response.body.contains(&format!(
        "kamn_service_api_route_authz_matrix_total_route_count {}",
        SERVICE_API_ROUTE_AUTHZ_MATRIX_TOTAL_ROUTE_COUNT
    )));
    assert!(metrics_response.body.contains(&format!(
        "kamn_service_api_route_authz_matrix_public_route_count {}",
        SERVICE_API_ROUTE_AUTHZ_MATRIX_PUBLIC_ROUTE_COUNT
    )));
    assert!(metrics_response.body.contains(&format!(
        "kamn_service_api_route_authz_matrix_protected_route_count {}",
        SERVICE_API_ROUTE_AUTHZ_MATRIX_PROTECTED_ROUTE_COUNT
    )));
    assert!(metrics_response.body.contains(&format!(
        "kamn_service_api_websocket_reason_taxonomy_info{{version=\"{}\"}} 1",
        SERVICE_API_WEBSOCKET_REASON_TAXONOMY_VERSION
    )));
    assert!(metrics_response.body.contains(&format!(
        "kamn_service_api_websocket_reason_code_count {expected_websocket_reason_code_count}"
    )));
    assert!(metrics_response.body.contains(&format!(
        "kamn_service_api_lifecycle_rejection_reason_taxonomy_info{{version=\"{}\"}} 1",
        SERVICE_API_LIFECYCLE_REJECTION_REASON_TAXONOMY_VERSION
    )));
    assert!(metrics_response.body.contains(&format!(
        "kamn_service_api_lifecycle_rejection_reason_code_count {expected_lifecycle_rejection_reason_code_count}"
    )));
}

#[test]
fn unit_service_api_route_authz_matrix_matches_protected_and_public_paths() {
    assert_eq!(
        SERVICE_API_AUTH_REASON_TAXONOMY_VERSION,
        "kamn.runtime.service-api-auth-reason-taxonomy.v1"
    );
    assert!(SERVICE_API_AUTH_REASON_CODES_CSV.contains(SERVICE_API_AUTH_MISSING_HEADER_REASON_CODE));
    for row in service_api_route_authz_matrix_rows() {
        assert_eq!(
            crate::service_api_endpoint::route_requires_auth(row.method, row.path),
            row.requires_auth,
            "route authz matrix drift for {} {}",
            row.method,
            row.path
        );
    }
}

#[test]
fn integration_service_api_endpoint_route_authz_matrix_rejects_protected_paths_without_headers() {
    #[derive(Debug, Clone, PartialEq, Eq)]
    struct ObservedRouteOutcome {
        method: String,
        path: String,
        status_line: String,
        reason_code: Option<String>,
    }

    let _env = acquire_service_api_test_env();
    let parsed = parse_args(vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
        "--runtime-mode".to_owned(),
        "api".to_owned(),
        "--api-bind".to_owned(),
        "127.0.0.1:34074".to_owned(),
    ])
    .expect("api args should parse");
    let report = execute(parsed).expect("api execution should succeed");
    let snapshot = build_service_api_snapshot(&report);

    let matrix_rows = service_api_route_authz_matrix_rows();
    let rounds = 2_u64;
    let bind_addr = reserve_loopback_addr();
    let endpoint_config = ServiceApiEndpointConfig {
        bind_addr: bind_addr.clone(),
        max_requests: rounds * matrix_rows.len() as u64,
        idle_timeout_ms: 2_500,
        body_limit_bytes: DEFAULT_SERVICE_API_BODY_LIMIT_BYTES,
        concurrency_limit: DEFAULT_SERVICE_API_CONCURRENCY_LIMIT,
        rate_limit_per_second: DEFAULT_SERVICE_API_RATE_LIMIT_PER_SECOND,
    };
    let server_snapshot = snapshot.clone();
    let server =
        thread::spawn(move || serve_service_api_endpoint(&endpoint_config, &server_snapshot));
    wait_for_endpoint_ready(bind_addr.as_str());

    let mut baseline_outcomes: Option<Vec<ObservedRouteOutcome>> = None;
    for _round in 0..rounds {
        let mut outcomes = Vec::with_capacity(matrix_rows.len());
        for row in &matrix_rows {
            let response = send_http_request(bind_addr.as_str(), row.method, row.path, row.body);
            assert!(
                response.contains(row.expected_status_without_auth),
                "unexpected authz matrix status for {} {}: expected {}, response={response}",
                row.method,
                row.path,
                row.expected_status_without_auth
            );
            let reason_code = if row.requires_auth {
                let payload = parse_error_envelope_from_http_response(response.as_str());
                assert_eq!(payload.error, "unauthorized");
                assert_eq!(
                    payload.reason_code,
                    SERVICE_API_AUTH_MISSING_HEADER_REASON_CODE
                );
                Some(payload.reason_code)
            } else {
                None
            };
            outcomes.push(ObservedRouteOutcome {
                method: row.method.to_owned(),
                path: row.path.to_owned(),
                status_line: row.expected_status_without_auth.to_owned(),
                reason_code,
            });
        }
        if let Some(baseline) = baseline_outcomes.as_ref() {
            assert_eq!(
                outcomes, *baseline,
                "route authz outcomes must remain deterministic across rounds"
            );
        } else {
            baseline_outcomes = Some(outcomes);
        }
    }

    let server_result = server.join().expect("endpoint thread should complete");
    assert!(
        server_result.is_ok(),
        "service api endpoint should stop cleanly after route authz matrix validation"
    );
}

#[test]
fn unit_service_api_scope_policy_fixture_parser_contract() {
    let (metadata, rows) = parse_service_api_scope_policy_fixture(SERVICE_API_SCOPE_POLICY_FIXTURE);
    assert_eq!(
        metadata
            .get("scope_policy_fixture_matrix_schema_version")
            .map(String::as_str),
        Some(SERVICE_API_SCOPE_POLICY_FIXTURE_SCHEMA_VERSION)
    );
    assert_eq!(
        metadata
            .get("scope_policy_reason_taxonomy_version")
            .map(String::as_str),
        Some(SERVICE_API_SCOPE_POLICY_REASON_TAXONOMY_VERSION)
    );
    assert_eq!(
        metadata
            .get("scope_policy_reason_codes_csv")
            .map(String::as_str),
        Some(SERVICE_API_SCOPE_POLICY_REASON_CODES_CSV)
    );
    assert!(
        rows.len() >= 6,
        "scope policy fixture matrix should provide representative coverage"
    );
    assert!(
        rows.iter().any(|row| {
            row.method == "POST"
                && row.path == "/v1/messages/send"
                && row.scope == "messages:write"
                && row.expected == "allow"
        }),
        "fixture should include allow case for message send write scope"
    );
    assert!(
        rows.iter().any(|row| {
            row.method == "POST"
                && row.path == "/v1/messages/send"
                && row.scope == "messages:read"
                && row.expected == "deny"
        }),
        "fixture should include deny case for message send read scope"
    );
}

#[test]
fn functional_service_api_scope_policy_fixture_rows_match_route_scope_mapping() {
    let (_, rows) = parse_service_api_scope_policy_fixture(SERVICE_API_SCOPE_POLICY_FIXTURE);
    for row in rows {
        let expected_scope = required_scope_for_test_route(row.method.as_str(), row.path.as_str())
            .expect("fixture rows should target protected routes only");
        if row.expected == "allow" {
            assert_eq!(
                row.scope, expected_scope,
                "allow fixture row scope must match required route scope"
            );
        } else if row.expected == "deny" {
            assert_ne!(
                row.scope, expected_scope,
                "deny fixture row scope must not match required route scope"
            );
        } else {
            panic!("scope fixture expected field must be allow|deny");
        }
    }
}

#[test]
fn integration_service_api_endpoint_scope_policy_rejects_missing_invalid_and_mismatched_scopes() {
    let _env = acquire_service_api_test_env();
    let parsed = parse_args(vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
        "--runtime-mode".to_owned(),
        "api".to_owned(),
        "--api-bind".to_owned(),
        "127.0.0.1:34075".to_owned(),
    ])
    .expect("api args should parse");
    let report = execute(parsed).expect("api execution should succeed");
    let snapshot = build_service_api_snapshot(&report);

    let bind_addr = reserve_loopback_addr();
    let endpoint_config = ServiceApiEndpointConfig {
        bind_addr: bind_addr.clone(),
        max_requests: 4,
        idle_timeout_ms: 2_500,
        body_limit_bytes: DEFAULT_SERVICE_API_BODY_LIMIT_BYTES,
        concurrency_limit: DEFAULT_SERVICE_API_CONCURRENCY_LIMIT,
        rate_limit_per_second: DEFAULT_SERVICE_API_RATE_LIMIT_PER_SECOND,
    };
    let server_snapshot = snapshot.clone();
    let server =
        thread::spawn(move || serve_service_api_endpoint(&endpoint_config, &server_snapshot));
    wait_for_endpoint_ready(bind_addr.as_str());

    let message_body = "{\"message\":\"scope-policy-check\"}";
    let sender_did = "kamn:did:agent:test-client-scope-policy";
    let state_hash = format!(
        "service-api:{}:{}",
        snapshot.chain_id.as_str(),
        snapshot.chain_version.as_str()
    );
    let signature_missing_scope = service_api_request_signature_for_fields(
        sender_did,
        9101,
        state_hash.as_str(),
        message_body,
    );
    let signature_invalid_scope = service_api_request_signature_for_fields(
        sender_did,
        9102,
        state_hash.as_str(),
        message_body,
    );
    let signature_mismatch_scope = service_api_request_signature_for_fields(
        sender_did,
        9103,
        state_hash.as_str(),
        message_body,
    );
    let signature_allowed_scope = service_api_request_signature_for_fields(
        sender_did,
        9104,
        state_hash.as_str(),
        message_body,
    );

    let missing_scope_response = send_http_request_with_headers_raw(
        bind_addr.as_str(),
        "POST",
        "/v1/messages/send",
        message_body,
        &[
            ("X-KAMN-Sender-DID", sender_did),
            ("X-KAMN-Request-Nonce", "9101"),
            ("X-KAMN-Request-Signature", signature_missing_scope.as_str()),
        ],
    );
    assert!(missing_scope_response.contains("HTTP/1.1 401 Unauthorized"));
    let missing_scope_payload =
        parse_error_envelope_from_http_response(missing_scope_response.as_str());
    assert_eq!(missing_scope_payload.error, "unauthorized");
    assert_eq!(
        missing_scope_payload.reason_code,
        SERVICE_API_AUTH_SCOPE_HEADER_MISSING_REASON_CODE
    );

    let invalid_scope_response = send_http_request_with_headers_raw(
        bind_addr.as_str(),
        "POST",
        "/v1/messages/send",
        message_body,
        &[
            ("X-KAMN-Sender-DID", sender_did),
            ("X-KAMN-Request-Nonce", "9102"),
            ("X-KAMN-Request-Signature", signature_invalid_scope.as_str()),
            ("X-KAMN-Authz-Scope", ""),
        ],
    );
    assert!(invalid_scope_response.contains("HTTP/1.1 401 Unauthorized"));
    let invalid_scope_payload =
        parse_error_envelope_from_http_response(invalid_scope_response.as_str());
    assert_eq!(invalid_scope_payload.error, "unauthorized");
    assert_eq!(
        invalid_scope_payload.reason_code,
        SERVICE_API_AUTH_SCOPE_INVALID_REASON_CODE
    );

    let mismatch_scope_response = send_http_request_with_headers_raw(
        bind_addr.as_str(),
        "POST",
        "/v1/messages/send",
        message_body,
        &[
            ("X-KAMN-Sender-DID", sender_did),
            ("X-KAMN-Request-Nonce", "9103"),
            (
                "X-KAMN-Request-Signature",
                signature_mismatch_scope.as_str(),
            ),
            ("X-KAMN-Authz-Scope", "messages:read"),
        ],
    );
    assert!(mismatch_scope_response.contains("HTTP/1.1 401 Unauthorized"));
    let mismatch_scope_payload =
        parse_error_envelope_from_http_response(mismatch_scope_response.as_str());
    assert_eq!(mismatch_scope_payload.error, "unauthorized");
    assert_eq!(
        mismatch_scope_payload.reason_code,
        SERVICE_API_AUTH_SCOPE_ROUTE_MISMATCH_REASON_CODE
    );

    let allowed_scope_response = send_http_request_with_headers_raw(
        bind_addr.as_str(),
        "POST",
        "/v1/messages/send",
        message_body,
        &[
            ("X-KAMN-Sender-DID", sender_did),
            ("X-KAMN-Request-Nonce", "9104"),
            ("X-KAMN-Request-Signature", signature_allowed_scope.as_str()),
            ("X-KAMN-Authz-Scope", "messages:write"),
        ],
    );
    assert!(allowed_scope_response.contains("HTTP/1.1 202 Accepted"));

    let server_result = server.join().expect("endpoint thread should complete");
    assert!(
        server_result.is_ok(),
        "service api endpoint should stop cleanly after scope policy checks"
    );
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
fn integration_service_api_endpoint_rejects_legacy_deterministic_signature_profile() {
    let _env = acquire_service_api_test_env();
    let parsed = parse_args(vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
        "--runtime-mode".to_owned(),
        "api".to_owned(),
        "--api-bind".to_owned(),
        "127.0.0.1:34079".to_owned(),
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

    let sender_did = "kamn:did:agent:test-client-legacy-signature";
    let nonce = 1_u64;
    let payload = r#"{"message":"legacy-signature"}"#;
    let state_hash = format!(
        "service-api:{}:{}",
        snapshot.chain_id.as_str(),
        snapshot.chain_version.as_str()
    );
    let legacy_signature =
        kamn_core::legacy_signature_for_fields(sender_did, nonce, state_hash.as_str(), payload);

    let response = send_http_request_with_headers(
        bind_addr.as_str(),
        "POST",
        "/v1/messages/send",
        payload,
        &[
            ("X-KAMN-Sender-DID", sender_did),
            ("X-KAMN-Request-Nonce", "1"),
            ("X-KAMN-Request-Signature", legacy_signature.as_str()),
        ],
    );
    assert!(response.contains("HTTP/1.1 401 Unauthorized"));
    let error_payload = parse_error_envelope_from_http_response(response.as_str());
    assert_eq!(
        error_payload.reason_code,
        "service_api_auth_signature_verification_failed"
    );

    let server_result = server.join().expect("endpoint thread should complete");
    assert!(
        server_result.is_ok(),
        "service api endpoint should stop cleanly after configured request budget"
    );
}

#[test]
fn regression_service_api_endpoint_rejects_legacy_signature_when_toggle_env_is_true() {
    let _env = acquire_service_api_test_env();
    let _legacy_toggle_guard = EnvVarGuard::set(
        "KAMN_SERVICE_API_AUTH_ALLOW_LEGACY_SIGNATURES",
        Some("true"),
    );
    let parsed = parse_args(vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
        "--runtime-mode".to_owned(),
        "api".to_owned(),
        "--api-bind".to_owned(),
        "127.0.0.1:34095".to_owned(),
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

    let sender_did = "kamn:did:agent:test-client-legacy-toggle-true";
    let nonce = 1_u64;
    let payload = r#"{"message":"legacy-signature-toggle-true"}"#;
    let state_hash = format!(
        "service-api:{}:{}",
        snapshot.chain_id.as_str(),
        snapshot.chain_version.as_str()
    );
    let legacy_signature =
        kamn_core::legacy_signature_for_fields(sender_did, nonce, state_hash.as_str(), payload);

    let response = send_http_request_with_headers(
        bind_addr.as_str(),
        "POST",
        "/v1/messages/send",
        payload,
        &[
            ("X-KAMN-Sender-DID", sender_did),
            ("X-KAMN-Request-Nonce", "1"),
            ("X-KAMN-Request-Signature", legacy_signature.as_str()),
        ],
    );
    assert!(response.contains("HTTP/1.1 401 Unauthorized"));
    let error_payload = parse_error_envelope_from_http_response(response.as_str());
    assert_eq!(
        error_payload.reason_code,
        "service_api_auth_signature_verification_failed"
    );

    let server_result = server.join().expect("endpoint thread should complete");
    assert!(
        server_result.is_ok(),
        "service api endpoint should stop cleanly after configured request budget"
    );
}

#[test]
fn integration_service_api_endpoint_persists_message_state_across_restart_without_explicit_state_file_env(
) {
    let _env = acquire_service_api_test_env();

    let parsed = parse_args(vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
        "--runtime-mode".to_owned(),
        "api".to_owned(),
        "--api-bind".to_owned(),
        "127.0.0.1:34079".to_owned(),
    ])
    .expect("api args should parse");
    let report = execute(parsed).expect("api execution should succeed");
    let snapshot = build_service_api_snapshot(&report);
    let state_hash = format!(
        "service-api:{}:{}",
        snapshot.chain_id.as_str(),
        snapshot.chain_version.as_str()
    );

    let bind_addr = reserve_loopback_addr();
    let send_config = ServiceApiEndpointConfig {
        bind_addr: bind_addr.clone(),
        max_requests: 1,
        idle_timeout_ms: 2_000,
        body_limit_bytes: DEFAULT_SERVICE_API_BODY_LIMIT_BYTES,
        concurrency_limit: DEFAULT_SERVICE_API_CONCURRENCY_LIMIT,
        rate_limit_per_second: DEFAULT_SERVICE_API_RATE_LIMIT_PER_SECOND,
    };
    let send_snapshot = snapshot.clone();
    let send_server =
        thread::spawn(move || serve_service_api_endpoint(&send_config, &send_snapshot));
    wait_for_endpoint_ready(bind_addr.as_str());

    let payload = r#"{"message":"durable-store-default-check"}"#;
    let send_signature = service_api_request_signature_for_fields(
        "kamn:did:agent:test-client-persist-default",
        1,
        state_hash.as_str(),
        payload,
    );
    let send_response = send_http_request_with_headers(
        bind_addr.as_str(),
        "POST",
        "/v1/messages/send",
        payload,
        &[
            (
                "X-KAMN-Sender-DID",
                "kamn:did:agent:test-client-persist-default",
            ),
            ("X-KAMN-Request-Nonce", "1"),
            ("X-KAMN-Request-Signature", send_signature.as_str()),
        ],
    );
    assert!(send_response.contains("HTTP/1.1 202 Accepted"));
    let send_payload: ServiceApiMessageCreateBody =
        parse_service_api_payload(extract_http_response_body(send_response.as_str()))
            .expect("send payload should deserialize");

    let send_server_result = send_server
        .join()
        .expect("send server thread should complete");
    assert!(
        send_server_result.is_ok(),
        "send-phase service api endpoint should stop cleanly after request budget"
    );

    let query_config = ServiceApiEndpointConfig {
        bind_addr: bind_addr.clone(),
        max_requests: 1,
        idle_timeout_ms: 2_000,
        body_limit_bytes: DEFAULT_SERVICE_API_BODY_LIMIT_BYTES,
        concurrency_limit: DEFAULT_SERVICE_API_CONCURRENCY_LIMIT,
        rate_limit_per_second: DEFAULT_SERVICE_API_RATE_LIMIT_PER_SECOND,
    };
    let query_snapshot = snapshot.clone();
    let query_server =
        thread::spawn(move || serve_service_api_endpoint(&query_config, &query_snapshot));
    wait_for_endpoint_ready(bind_addr.as_str());

    let query_path = format!("/v1/messages/{}", send_payload.message_id);
    let query_signature = service_api_request_signature_for_fields(
        "kamn:did:agent:test-client-persist-default",
        2,
        state_hash.as_str(),
        "",
    );
    let query_response = send_http_request_with_headers(
        bind_addr.as_str(),
        "GET",
        query_path.as_str(),
        "",
        &[
            (
                "X-KAMN-Sender-DID",
                "kamn:did:agent:test-client-persist-default",
            ),
            ("X-KAMN-Request-Nonce", "2"),
            ("X-KAMN-Request-Signature", query_signature.as_str()),
        ],
    );
    assert!(query_response.contains("HTTP/1.1 200 OK"));
    let query_payload: ServiceApiMessageGetBody =
        parse_service_api_payload(extract_http_response_body(query_response.as_str()))
            .expect("query payload should deserialize");
    assert_eq!(query_payload.message_id, send_payload.message_id);
    assert_eq!(query_payload.status, "created");

    let query_server_result = query_server
        .join()
        .expect("query server thread should complete");
    assert!(
        query_server_result.is_ok(),
        "query-phase service api endpoint should stop cleanly after request budget"
    );
}

#[test]
fn integration_service_api_endpoint_persists_message_state_across_restart() {
    let _env = acquire_service_api_test_env();
    let state_file = std::env::temp_dir().join(format!(
        "kamn-node-service-api-state-{}-{}.json",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock should be monotonic")
            .as_nanos()
    ));
    let state_file_str = state_file.to_string_lossy().to_string();
    let _state_file_guard =
        EnvVarGuard::set("KAMN_SERVICE_API_STATE_FILE", Some(state_file_str.as_str()));

    let parsed = parse_args(vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
        "--runtime-mode".to_owned(),
        "api".to_owned(),
        "--api-bind".to_owned(),
        "127.0.0.1:34080".to_owned(),
    ])
    .expect("api args should parse");
    let report = execute(parsed).expect("api execution should succeed");
    let snapshot = build_service_api_snapshot(&report);
    let state_hash = format!(
        "service-api:{}:{}",
        snapshot.chain_id.as_str(),
        snapshot.chain_version.as_str()
    );

    let bind_addr = reserve_loopback_addr();
    let send_config = ServiceApiEndpointConfig {
        bind_addr: bind_addr.clone(),
        max_requests: 1,
        idle_timeout_ms: 2_000,
        body_limit_bytes: DEFAULT_SERVICE_API_BODY_LIMIT_BYTES,
        concurrency_limit: DEFAULT_SERVICE_API_CONCURRENCY_LIMIT,
        rate_limit_per_second: DEFAULT_SERVICE_API_RATE_LIMIT_PER_SECOND,
    };
    let send_snapshot = snapshot.clone();
    let send_server =
        thread::spawn(move || serve_service_api_endpoint(&send_config, &send_snapshot));
    wait_for_endpoint_ready(bind_addr.as_str());

    let payload = r#"{"message":"durable-store-check"}"#;
    let send_signature = service_api_request_signature_for_fields(
        "kamn:did:agent:test-client-persist",
        1,
        state_hash.as_str(),
        payload,
    );
    let send_response = send_http_request_with_headers(
        bind_addr.as_str(),
        "POST",
        "/v1/messages/send",
        payload,
        &[
            ("X-KAMN-Sender-DID", "kamn:did:agent:test-client-persist"),
            ("X-KAMN-Request-Nonce", "1"),
            ("X-KAMN-Request-Signature", send_signature.as_str()),
        ],
    );
    assert!(send_response.contains("HTTP/1.1 202 Accepted"));
    let send_payload: ServiceApiMessageCreateBody =
        parse_service_api_payload(extract_http_response_body(send_response.as_str()))
            .expect("send payload should deserialize");
    let send_server_result = send_server
        .join()
        .expect("send server thread should complete");
    assert!(
        send_server_result.is_ok(),
        "send-phase service api endpoint should stop cleanly after request budget"
    );

    let query_config = ServiceApiEndpointConfig {
        bind_addr: bind_addr.clone(),
        max_requests: 1,
        idle_timeout_ms: 2_000,
        body_limit_bytes: DEFAULT_SERVICE_API_BODY_LIMIT_BYTES,
        concurrency_limit: DEFAULT_SERVICE_API_CONCURRENCY_LIMIT,
        rate_limit_per_second: DEFAULT_SERVICE_API_RATE_LIMIT_PER_SECOND,
    };
    let query_snapshot = snapshot.clone();
    let query_server =
        thread::spawn(move || serve_service_api_endpoint(&query_config, &query_snapshot));
    wait_for_endpoint_ready(bind_addr.as_str());

    let query_path = format!("/v1/messages/{}", send_payload.message_id);
    let query_signature = service_api_request_signature_for_fields(
        "kamn:did:agent:test-client-persist",
        2,
        state_hash.as_str(),
        "",
    );
    let query_response = send_http_request_with_headers(
        bind_addr.as_str(),
        "GET",
        query_path.as_str(),
        "",
        &[
            ("X-KAMN-Sender-DID", "kamn:did:agent:test-client-persist"),
            ("X-KAMN-Request-Nonce", "2"),
            ("X-KAMN-Request-Signature", query_signature.as_str()),
        ],
    );
    assert!(query_response.contains("HTTP/1.1 200 OK"));
    let query_payload: ServiceApiMessageGetBody =
        parse_service_api_payload(extract_http_response_body(query_response.as_str()))
            .expect("query payload should deserialize");
    assert_eq!(query_payload.message_id, send_payload.message_id);
    assert_eq!(query_payload.status, "created");

    let query_server_result = query_server
        .join()
        .expect("query server thread should complete");
    assert!(
        query_server_result.is_ok(),
        "query-phase service api endpoint should stop cleanly after request budget"
    );

    let _ = fs::remove_file(state_file);
}

#[test]
fn integration_service_api_endpoint_send_path_persists_data_layer_runtime_evidence_for_m0_to_m11() {
    let _env = acquire_service_api_test_env();
    let state_file = std::env::temp_dir().join(format!(
        "kamn-node-service-api-data-layer-evidence-state-{}-{}.json",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock should be monotonic")
            .as_nanos()
    ));
    let state_file_str = state_file.to_string_lossy().to_string();
    let _state_file_guard =
        EnvVarGuard::set("KAMN_SERVICE_API_STATE_FILE", Some(state_file_str.as_str()));

    let parsed = parse_args(vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
        "--runtime-mode".to_owned(),
        "api".to_owned(),
        "--api-bind".to_owned(),
        "127.0.0.1:34082".to_owned(),
    ])
    .expect("api args should parse");
    let report = execute(parsed).expect("api execution should succeed");
    let snapshot = build_service_api_snapshot(&report);
    let state_hash = format!(
        "service-api:{}:{}",
        snapshot.chain_id.as_str(),
        snapshot.chain_version.as_str()
    );

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

    let send_payload_body =
        r#"{"recipient_did":"kamn:did:agent:e2e-recipient","message":"e2e-runtime-evidence"}"#;
    let send_signature = service_api_request_signature_for_fields(
        "kamn:did:agent:e2e-sender",
        81,
        state_hash.as_str(),
        send_payload_body,
    );
    let send_response = send_http_request_with_headers(
        bind_addr.as_str(),
        "POST",
        "/v1/messages/send",
        send_payload_body,
        &[
            ("X-KAMN-Sender-DID", "kamn:did:agent:e2e-sender"),
            ("X-KAMN-Request-Nonce", "81"),
            ("X-KAMN-Request-Signature", send_signature.as_str()),
        ],
    );
    assert!(send_response.contains("HTTP/1.1 202 Accepted"));
    let send_payload: ServiceApiMessageCreateBody =
        parse_service_api_payload(extract_http_response_body(send_response.as_str()))
            .expect("send payload should deserialize");

    let server_result = server.join().expect("endpoint thread should complete");
    assert!(
        server_result.is_ok(),
        "service api endpoint should stop cleanly after message send"
    );

    let state_payload =
        fs::read_to_string(state_file.as_path()).expect("state file should remain readable");
    let state_json: Value =
        serde_json::from_str(state_payload.as_str()).expect("state file should parse");
    let evidence =
        &state_json["messages"][send_payload.message_id.as_str()]["data_layer_runtime_evidence"];
    assert_eq!(
        evidence["schema_version"],
        "kamn.runtime.service-api-data-layer-runtime-evidence.v1"
    );
    assert!(evidence["m0_content_hash"]
        .as_str()
        .is_some_and(|value| value.starts_with("sha256:")));
    assert!(evidence["m1_merkle_root"]
        .as_str()
        .is_some_and(|value| value.starts_with("sha256:")));
    assert!(evidence["m2_authorization_reason_code"].as_str().is_some());
    assert!(evidence["m3_blind_index_token"].as_str().is_some());
    assert!(evidence["m4_transition_reason_code"].as_str().is_some());
    assert!(evidence["m5_record_hash"].as_str().is_some());
    assert!(evidence["m6_projection_edge_count"].as_u64().is_some());
    assert!(evidence["m7_observability_health"].as_str().is_some());
    assert!(evidence["m8_retention_due_count"].as_u64().is_some());
    assert!(evidence["m9_dispatch_reason_code"].as_str().is_some());
    assert!(evidence["m10_archived_partition_count"].as_u64().is_some());
    assert!(evidence["m11_decision"].as_str().is_some());

    let _ = fs::remove_file(state_file);
}

#[test]
fn integration_service_api_endpoint_lists_channel_messages_from_message_store() {
    let _env = acquire_service_api_test_env();
    let state_file = std::env::temp_dir().join(format!(
        "kamn-node-service-api-channel-state-{}-{}.json",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock should be monotonic")
            .as_nanos()
    ));
    let state_file_str = state_file.to_string_lossy().to_string();
    let _state_file_guard =
        EnvVarGuard::set("KAMN_SERVICE_API_STATE_FILE", Some(state_file_str.as_str()));

    let parsed = parse_args(vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
        "--runtime-mode".to_owned(),
        "api".to_owned(),
        "--api-bind".to_owned(),
        "127.0.0.1:34081".to_owned(),
    ])
    .expect("api args should parse");
    let report = execute(parsed).expect("api execution should succeed");
    let snapshot = build_service_api_snapshot(&report);
    let state_hash = format!(
        "service-api:{}:{}",
        snapshot.chain_id.as_str(),
        snapshot.chain_version.as_str()
    );

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

    let send_payload_body = r#"{"channel_id":"channel-contract-42","message":"hello"}"#;
    let send_signature = service_api_request_signature_for_fields(
        "kamn:did:agent:test-client-channel",
        11,
        state_hash.as_str(),
        send_payload_body,
    );
    let send_response = send_http_request_with_headers(
        bind_addr.as_str(),
        "POST",
        "/v1/messages/send",
        send_payload_body,
        &[
            ("X-KAMN-Sender-DID", "kamn:did:agent:test-client-channel"),
            ("X-KAMN-Request-Nonce", "11"),
            ("X-KAMN-Request-Signature", send_signature.as_str()),
        ],
    );
    assert!(send_response.contains("HTTP/1.1 202 Accepted"));
    let send_payload: ServiceApiMessageCreateBody =
        parse_service_api_payload(extract_http_response_body(send_response.as_str()))
            .expect("send payload should deserialize");

    let list_signature = service_api_request_signature_for_fields(
        "kamn:did:agent:test-client-channel",
        12,
        state_hash.as_str(),
        "",
    );
    let list_response = send_http_request_with_headers(
        bind_addr.as_str(),
        "GET",
        "/v1/channels/channel-contract-42/messages",
        "",
        &[
            ("X-KAMN-Sender-DID", "kamn:did:agent:test-client-channel"),
            ("X-KAMN-Request-Nonce", "12"),
            ("X-KAMN-Request-Signature", list_signature.as_str()),
        ],
    );
    assert!(list_response.contains("HTTP/1.1 200 OK"));
    let list_payload: ServiceApiChannelMessagesBody =
        parse_service_api_payload(extract_http_response_body(list_response.as_str()))
            .expect("channel list payload should deserialize");
    assert_eq!(list_payload.channel_id, "channel-contract-42");
    assert!(
        list_payload.messages.contains(&send_payload.message_id),
        "channel list should contain sent message id"
    );

    let server_result = server.join().expect("endpoint thread should complete");
    assert!(
        server_result.is_ok(),
        "service api endpoint should stop cleanly after channel list request budget"
    );

    let _ = fs::remove_file(state_file);
}

#[test]
fn integration_service_api_endpoint_persists_task_and_escrow_state_across_routes() {
    let _env = acquire_service_api_test_env();
    let state_file = std::env::temp_dir().join(format!(
        "kamn-node-service-api-task-escrow-state-{}-{}.json",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock should be monotonic")
            .as_nanos()
    ));
    let state_file_str = state_file.to_string_lossy().to_string();
    let _state_file_guard =
        EnvVarGuard::set("KAMN_SERVICE_API_STATE_FILE", Some(state_file_str.as_str()));

    let parsed = parse_args(vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
        "--runtime-mode".to_owned(),
        "api".to_owned(),
        "--api-bind".to_owned(),
        "127.0.0.1:34106".to_owned(),
    ])
    .expect("api args should parse");
    let report = execute(parsed).expect("api execution should succeed");
    let snapshot = build_service_api_snapshot(&report);
    let state_hash = format!(
        "service-api:{}:{}",
        snapshot.chain_id.as_str(),
        snapshot.chain_version.as_str()
    );
    let task_caller_did = "kamn:did:agent:test-client-task-state";
    let escrow_caller_did = "kamn:did:agent:test-client-escrow-state";

    let bind_addr = reserve_loopback_addr();
    let endpoint_config = ServiceApiEndpointConfig {
        bind_addr: bind_addr.clone(),
        max_requests: 5,
        idle_timeout_ms: 2_000,
        body_limit_bytes: DEFAULT_SERVICE_API_BODY_LIMIT_BYTES,
        concurrency_limit: DEFAULT_SERVICE_API_CONCURRENCY_LIMIT,
        rate_limit_per_second: DEFAULT_SERVICE_API_RATE_LIMIT_PER_SECOND,
    };
    let server_snapshot = snapshot.clone();
    let server =
        thread::spawn(move || serve_service_api_endpoint(&endpoint_config, &server_snapshot));
    wait_for_endpoint_ready(bind_addr.as_str());

    let create_task_payload =
        r#"{"title":"persisted-task","description":"task persistence contract"}"#;
    let create_task_signature = service_api_request_signature_for_fields(
        task_caller_did,
        21,
        state_hash.as_str(),
        create_task_payload,
    );
    let create_task_response = send_http_request_with_headers(
        bind_addr.as_str(),
        "POST",
        "/v1/tasks/create",
        create_task_payload,
        &[
            ("X-KAMN-Sender-DID", task_caller_did),
            ("X-KAMN-Request-Nonce", "21"),
            ("X-KAMN-Request-Signature", create_task_signature.as_str()),
        ],
    );
    assert!(create_task_response.contains("HTTP/1.1 201 Created"));
    let created_task: ServiceApiTaskCreateBody =
        parse_service_api_payload(extract_http_response_body(create_task_response.as_str()))
            .expect("task create payload should deserialize");
    assert_eq!(created_task.state, "submitted");

    let accept_path = format!("/v1/tasks/{}/accept", created_task.task_id);
    let accept_signature =
        service_api_request_signature_for_fields(task_caller_did, 22, state_hash.as_str(), "");
    let accept_response = send_http_request_with_headers(
        bind_addr.as_str(),
        "POST",
        accept_path.as_str(),
        "",
        &[
            ("X-KAMN-Sender-DID", task_caller_did),
            ("X-KAMN-Request-Nonce", "22"),
            ("X-KAMN-Request-Signature", accept_signature.as_str()),
        ],
    );
    assert!(accept_response.contains("HTTP/1.1 200 OK"));

    let query_task_path = format!("/v1/tasks/{}", created_task.task_id);
    let query_task_signature =
        service_api_request_signature_for_fields(task_caller_did, 23, state_hash.as_str(), "");
    let query_task_response = send_http_request_with_headers(
        bind_addr.as_str(),
        "GET",
        query_task_path.as_str(),
        "",
        &[
            ("X-KAMN-Sender-DID", task_caller_did),
            ("X-KAMN-Request-Nonce", "23"),
            ("X-KAMN-Request-Signature", query_task_signature.as_str()),
        ],
    );
    assert!(query_task_response.contains("HTTP/1.1 200 OK"));
    let queried_task: Value =
        parse_service_api_payload(extract_http_response_body(query_task_response.as_str()))
            .expect("task query payload should deserialize");
    assert_eq!(queried_task["task_id"], created_task.task_id);
    assert_eq!(queried_task["state"], "accepted");

    let fund_payload = r#"{"task_id":"persisted-task","amount":1}"#;
    let fund_signature = service_api_request_signature_for_fields(
        escrow_caller_did,
        24,
        state_hash.as_str(),
        fund_payload,
    );
    let fund_response = send_http_request_with_headers(
        bind_addr.as_str(),
        "POST",
        "/v1/escrow/fund",
        fund_payload,
        &[
            ("X-KAMN-Sender-DID", escrow_caller_did),
            ("X-KAMN-Request-Nonce", "24"),
            ("X-KAMN-Request-Signature", fund_signature.as_str()),
        ],
    );
    assert!(fund_response.contains("HTTP/1.1 200 OK"));
    let funded_escrow: Value =
        parse_service_api_payload(extract_http_response_body(fund_response.as_str()))
            .expect("escrow fund payload should deserialize");
    assert_eq!(funded_escrow["state"], "funded");
    let escrow_id = funded_escrow["escrow_id"]
        .as_str()
        .expect("escrow id should be string")
        .to_owned();

    let release_path = format!("/v1/escrow/{escrow_id}/release");
    let release_signature =
        service_api_request_signature_for_fields(escrow_caller_did, 25, state_hash.as_str(), "");
    let release_response = send_http_request_with_headers(
        bind_addr.as_str(),
        "POST",
        release_path.as_str(),
        "",
        &[
            ("X-KAMN-Sender-DID", escrow_caller_did),
            ("X-KAMN-Request-Nonce", "25"),
            ("X-KAMN-Request-Signature", release_signature.as_str()),
        ],
    );
    assert!(release_response.contains("HTTP/1.1 200 OK"));
    let released_escrow: Value =
        parse_service_api_payload(extract_http_response_body(release_response.as_str()))
            .expect("escrow release payload should deserialize");
    assert_eq!(released_escrow["escrow_id"], escrow_id);
    assert_eq!(released_escrow["state"], "released");

    let server_result = server.join().expect("endpoint thread should complete");
    assert!(
        server_result.is_ok(),
        "service api endpoint should stop cleanly after task+escrow persistence requests"
    );

    let _ = fs::remove_file(state_file);
}

#[test]
fn integration_service_api_endpoint_persists_task_and_escrow_state_across_restart() {
    let _env = acquire_service_api_test_env();
    let state_file = std::env::temp_dir().join(format!(
        "kamn-node-service-api-task-escrow-restart-state-{}-{}.json",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock should be monotonic")
            .as_nanos()
    ));
    let state_file_str = state_file.to_string_lossy().to_string();
    let _state_file_guard =
        EnvVarGuard::set("KAMN_SERVICE_API_STATE_FILE", Some(state_file_str.as_str()));

    let parsed = parse_args(vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
        "--runtime-mode".to_owned(),
        "api".to_owned(),
        "--api-bind".to_owned(),
        "127.0.0.1:34110".to_owned(),
    ])
    .expect("api args should parse");
    let report = execute(parsed).expect("api execution should succeed");
    let snapshot = build_service_api_snapshot(&report);
    let state_hash = format!(
        "service-api:{}:{}",
        snapshot.chain_id.as_str(),
        snapshot.chain_version.as_str()
    );
    let task_caller_did = "kamn:did:agent:test-client-task-restart";
    let escrow_caller_did = "kamn:did:agent:test-client-escrow-restart";

    let bind_addr = reserve_loopback_addr();
    let endpoint_config = ServiceApiEndpointConfig {
        bind_addr: bind_addr.clone(),
        max_requests: 4,
        idle_timeout_ms: 2_000,
        body_limit_bytes: DEFAULT_SERVICE_API_BODY_LIMIT_BYTES,
        concurrency_limit: DEFAULT_SERVICE_API_CONCURRENCY_LIMIT,
        rate_limit_per_second: DEFAULT_SERVICE_API_RATE_LIMIT_PER_SECOND,
    };
    let server_snapshot = snapshot.clone();
    let server =
        thread::spawn(move || serve_service_api_endpoint(&endpoint_config, &server_snapshot));
    wait_for_endpoint_ready(bind_addr.as_str());

    let create_task_payload = r#"{"title":"restart-task","description":"persist restart"}"#;
    let create_task_signature = service_api_request_signature_for_fields(
        task_caller_did,
        61,
        state_hash.as_str(),
        create_task_payload,
    );
    let create_task_response = send_http_request_with_headers(
        bind_addr.as_str(),
        "POST",
        "/v1/tasks/create",
        create_task_payload,
        &[
            ("X-KAMN-Sender-DID", task_caller_did),
            ("X-KAMN-Request-Nonce", "61"),
            ("X-KAMN-Request-Signature", create_task_signature.as_str()),
        ],
    );
    assert!(create_task_response.contains("HTTP/1.1 201 Created"));
    let created_task: ServiceApiTaskCreateBody =
        parse_service_api_payload(extract_http_response_body(create_task_response.as_str()))
            .expect("task create payload should deserialize");

    let accept_path = format!("/v1/tasks/{}/accept", created_task.task_id);
    let accept_signature =
        service_api_request_signature_for_fields(task_caller_did, 62, state_hash.as_str(), "");
    let accept_response = send_http_request_with_headers(
        bind_addr.as_str(),
        "POST",
        accept_path.as_str(),
        "",
        &[
            ("X-KAMN-Sender-DID", task_caller_did),
            ("X-KAMN-Request-Nonce", "62"),
            ("X-KAMN-Request-Signature", accept_signature.as_str()),
        ],
    );
    assert!(accept_response.contains("HTTP/1.1 200 OK"));

    let fund_payload = r#"{"task_id":"restart-task","amount":5}"#;
    let fund_signature = service_api_request_signature_for_fields(
        escrow_caller_did,
        63,
        state_hash.as_str(),
        fund_payload,
    );
    let fund_response = send_http_request_with_headers(
        bind_addr.as_str(),
        "POST",
        "/v1/escrow/fund",
        fund_payload,
        &[
            ("X-KAMN-Sender-DID", escrow_caller_did),
            ("X-KAMN-Request-Nonce", "63"),
            ("X-KAMN-Request-Signature", fund_signature.as_str()),
        ],
    );
    assert!(fund_response.contains("HTTP/1.1 200 OK"));
    let funded_escrow: Value =
        parse_service_api_payload(extract_http_response_body(fund_response.as_str()))
            .expect("escrow fund payload should deserialize");
    let escrow_id = funded_escrow["escrow_id"]
        .as_str()
        .expect("escrow id should be string")
        .to_owned();

    let release_path = format!("/v1/escrow/{escrow_id}/release");
    let release_signature =
        service_api_request_signature_for_fields(escrow_caller_did, 64, state_hash.as_str(), "");
    let release_response = send_http_request_with_headers(
        bind_addr.as_str(),
        "POST",
        release_path.as_str(),
        "",
        &[
            ("X-KAMN-Sender-DID", escrow_caller_did),
            ("X-KAMN-Request-Nonce", "64"),
            ("X-KAMN-Request-Signature", release_signature.as_str()),
        ],
    );
    assert!(release_response.contains("HTTP/1.1 200 OK"));

    let server_result = server.join().expect("endpoint thread should complete");
    assert!(
        server_result.is_ok(),
        "service api endpoint should stop cleanly after first persistence phase"
    );

    let restart_report = execute(
        parse_args(vec![
            "kamn-node".to_owned(),
            "--role".to_owned(),
            "processor".to_owned(),
            "--runtime-mode".to_owned(),
            "api".to_owned(),
            "--api-bind".to_owned(),
            "127.0.0.1:34111".to_owned(),
        ])
        .expect("restart api args should parse"),
    )
    .expect("restart api execution should succeed");
    let restart_snapshot = build_service_api_snapshot(&restart_report);
    let restart_state_hash = format!(
        "service-api:{}:{}",
        restart_snapshot.chain_id.as_str(),
        restart_snapshot.chain_version.as_str()
    );
    let restart_bind_addr = reserve_loopback_addr();
    let restart_endpoint_config = ServiceApiEndpointConfig {
        bind_addr: restart_bind_addr.clone(),
        max_requests: 1,
        idle_timeout_ms: 2_000,
        body_limit_bytes: DEFAULT_SERVICE_API_BODY_LIMIT_BYTES,
        concurrency_limit: DEFAULT_SERVICE_API_CONCURRENCY_LIMIT,
        rate_limit_per_second: DEFAULT_SERVICE_API_RATE_LIMIT_PER_SECOND,
    };
    let restart_server_snapshot = restart_snapshot.clone();
    let restart_server = thread::spawn(move || {
        serve_service_api_endpoint(&restart_endpoint_config, &restart_server_snapshot)
    });
    wait_for_endpoint_ready(restart_bind_addr.as_str());

    let query_task_path = format!("/v1/tasks/{}", created_task.task_id);
    let query_task_signature = service_api_request_signature_for_fields(
        task_caller_did,
        65,
        restart_state_hash.as_str(),
        "",
    );
    let query_task_response = send_http_request_with_headers(
        restart_bind_addr.as_str(),
        "GET",
        query_task_path.as_str(),
        "",
        &[
            ("X-KAMN-Sender-DID", task_caller_did),
            ("X-KAMN-Request-Nonce", "65"),
            ("X-KAMN-Request-Signature", query_task_signature.as_str()),
        ],
    );
    assert!(query_task_response.contains("HTTP/1.1 200 OK"));
    let queried_task: Value =
        parse_service_api_payload(extract_http_response_body(query_task_response.as_str()))
            .expect("task query payload should deserialize");
    assert_eq!(queried_task["task_id"], created_task.task_id);
    assert_eq!(queried_task["state"], "accepted");

    let restart_server_result = restart_server
        .join()
        .expect("restart endpoint thread should complete");
    assert!(
        restart_server_result.is_ok(),
        "service api endpoint should stop cleanly after restart query"
    );

    let state_payload = fs::read_to_string(state_file.as_path())
        .expect("state file should remain readable across restart");
    let state_json: Value =
        serde_json::from_str(state_payload.as_str()).expect("state payload should parse");
    assert_eq!(
        state_json["tasks"][created_task.task_id.as_str()]["state"],
        "accepted"
    );
    assert_eq!(
        state_json["escrows"][escrow_id.as_str()]["state"],
        "released"
    );

    let _ = fs::remove_file(state_file);
}

#[test]
fn integration_service_api_endpoint_persists_content_lifecycle_state_across_restart() {
    let _env = acquire_service_api_test_env();
    let state_file = std::env::temp_dir().join(format!(
        "kamn-node-service-api-content-restart-state-{}-{}.json",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock should be monotonic")
            .as_nanos()
    ));
    let state_file_str = state_file.to_string_lossy().to_string();
    let _state_file_guard =
        EnvVarGuard::set("KAMN_SERVICE_API_STATE_FILE", Some(state_file_str.as_str()));

    let parsed = parse_args(vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
        "--runtime-mode".to_owned(),
        "api".to_owned(),
        "--api-bind".to_owned(),
        "127.0.0.1:34113".to_owned(),
    ])
    .expect("api args should parse");
    let report = execute(parsed).expect("api execution should succeed");
    let snapshot = build_service_api_snapshot(&report);
    let state_hash = format!(
        "service-api:{}:{}",
        snapshot.chain_id.as_str(),
        snapshot.chain_version.as_str()
    );
    let caller_did = "kamn:did:agent:test-client-content-restart";

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

    let register_payload = r#"{"content":"restart-content-check"}"#;
    let register_signature = service_api_request_signature_for_fields(
        caller_did,
        91,
        state_hash.as_str(),
        register_payload,
    );
    let register_response = send_http_request_with_headers(
        bind_addr.as_str(),
        "POST",
        "/v1/content/register",
        register_payload,
        &[
            ("X-KAMN-Sender-DID", caller_did),
            ("X-KAMN-Request-Nonce", "91"),
            ("X-KAMN-Request-Signature", register_signature.as_str()),
        ],
    );
    assert!(register_response.contains("HTTP/1.1 201 Created"));
    let registered_content: Value =
        parse_service_api_payload(extract_http_response_body(register_response.as_str()))
            .expect("content register payload should deserialize");
    let content_id = registered_content["content_id"]
        .as_str()
        .expect("content id should be string")
        .to_owned();
    assert_eq!(registered_content["retention_class"], "standard");
    assert_eq!(registered_content["lifecycle_state"], "retained");
    assert_eq!(registered_content["redaction_status"], "none");

    let expire_path = format!("/v1/content/{content_id}/expire");
    let expire_signature =
        service_api_request_signature_for_fields(caller_did, 92, state_hash.as_str(), "");
    let expire_response = send_http_request_with_headers(
        bind_addr.as_str(),
        "POST",
        expire_path.as_str(),
        "",
        &[
            ("X-KAMN-Sender-DID", caller_did),
            ("X-KAMN-Request-Nonce", "92"),
            ("X-KAMN-Request-Signature", expire_signature.as_str()),
        ],
    );
    assert!(expire_response.contains("HTTP/1.1 200 OK"));
    let expired_payload: Value =
        parse_service_api_payload(extract_http_response_body(expire_response.as_str()))
            .expect("expire payload should deserialize");
    assert_eq!(expired_payload["content_id"], content_id);
    assert_eq!(expired_payload["lifecycle_state"], "expired");
    assert_eq!(expired_payload["redaction_status"], "none");

    let server_result = server.join().expect("endpoint thread should complete");
    assert!(
        server_result.is_ok(),
        "service api endpoint should stop cleanly after first content lifecycle phase"
    );

    let restart_report = execute(
        parse_args(vec![
            "kamn-node".to_owned(),
            "--role".to_owned(),
            "processor".to_owned(),
            "--runtime-mode".to_owned(),
            "api".to_owned(),
            "--api-bind".to_owned(),
            "127.0.0.1:34114".to_owned(),
        ])
        .expect("restart api args should parse"),
    )
    .expect("restart api execution should succeed");
    let restart_snapshot = build_service_api_snapshot(&restart_report);
    let restart_state_hash = format!(
        "service-api:{}:{}",
        restart_snapshot.chain_id.as_str(),
        restart_snapshot.chain_version.as_str()
    );
    let restart_bind_addr = reserve_loopback_addr();
    let restart_endpoint_config = ServiceApiEndpointConfig {
        bind_addr: restart_bind_addr.clone(),
        max_requests: 4,
        idle_timeout_ms: 2_000,
        body_limit_bytes: DEFAULT_SERVICE_API_BODY_LIMIT_BYTES,
        concurrency_limit: DEFAULT_SERVICE_API_CONCURRENCY_LIMIT,
        rate_limit_per_second: DEFAULT_SERVICE_API_RATE_LIMIT_PER_SECOND,
    };
    let restart_server_snapshot = restart_snapshot.clone();
    let restart_server = thread::spawn(move || {
        serve_service_api_endpoint(&restart_endpoint_config, &restart_server_snapshot)
    });
    wait_for_endpoint_ready(restart_bind_addr.as_str());

    let query_path = format!("/v1/content/{content_id}");
    let query_signature =
        service_api_request_signature_for_fields(caller_did, 93, restart_state_hash.as_str(), "");
    let query_response = send_http_request_with_headers(
        restart_bind_addr.as_str(),
        "GET",
        query_path.as_str(),
        "",
        &[
            ("X-KAMN-Sender-DID", caller_did),
            ("X-KAMN-Request-Nonce", "93"),
            ("X-KAMN-Request-Signature", query_signature.as_str()),
        ],
    );
    assert!(query_response.contains("HTTP/1.1 200 OK"));
    let queried_payload: Value =
        parse_service_api_payload(extract_http_response_body(query_response.as_str()))
            .expect("query payload should deserialize");
    assert_eq!(queried_payload["content_id"], content_id);
    assert_eq!(queried_payload["lifecycle_state"], "expired");
    assert_eq!(queried_payload["redaction_status"], "none");

    let tombstone_path = format!("/v1/content/{content_id}/tombstone");
    let tombstone_signature =
        service_api_request_signature_for_fields(caller_did, 94, restart_state_hash.as_str(), "");
    let tombstone_response = send_http_request_with_headers(
        restart_bind_addr.as_str(),
        "POST",
        tombstone_path.as_str(),
        "",
        &[
            ("X-KAMN-Sender-DID", caller_did),
            ("X-KAMN-Request-Nonce", "94"),
            ("X-KAMN-Request-Signature", tombstone_signature.as_str()),
        ],
    );
    assert!(tombstone_response.contains("HTTP/1.1 200 OK"));
    let tombstone_payload: Value =
        parse_service_api_payload(extract_http_response_body(tombstone_response.as_str()))
            .expect("tombstone payload should deserialize");
    assert_eq!(tombstone_payload["content_id"], content_id);
    assert_eq!(tombstone_payload["lifecycle_state"], "tombstoned");
    assert_eq!(tombstone_payload["redaction_status"], "redacted");

    let query_after_tombstone_signature =
        service_api_request_signature_for_fields(caller_did, 95, restart_state_hash.as_str(), "");
    let query_after_tombstone_response = send_http_request_with_headers(
        restart_bind_addr.as_str(),
        "GET",
        query_path.as_str(),
        "",
        &[
            ("X-KAMN-Sender-DID", caller_did),
            ("X-KAMN-Request-Nonce", "95"),
            (
                "X-KAMN-Request-Signature",
                query_after_tombstone_signature.as_str(),
            ),
        ],
    );
    assert!(query_after_tombstone_response.contains("HTTP/1.1 200 OK"));
    let queried_after_tombstone_payload: Value = parse_service_api_payload(
        extract_http_response_body(query_after_tombstone_response.as_str()),
    )
    .expect("post-tombstone query payload should deserialize");
    assert_eq!(queried_after_tombstone_payload["content_id"], content_id);
    assert_eq!(
        queried_after_tombstone_payload["lifecycle_state"],
        "tombstoned"
    );
    assert_eq!(
        queried_after_tombstone_payload["redaction_status"],
        "redacted"
    );

    let missing_caller_did = "kamn:did:agent:test-client-content-missing-restart";
    let missing_query_signature = service_api_request_signature_for_fields(
        missing_caller_did,
        96,
        restart_state_hash.as_str(),
        "",
    );
    let missing_query_response = send_http_request_with_headers(
        restart_bind_addr.as_str(),
        "GET",
        "/v1/content/content-missing-96",
        "",
        &[
            ("X-KAMN-Sender-DID", missing_caller_did),
            ("X-KAMN-Request-Nonce", "96"),
            ("X-KAMN-Request-Signature", missing_query_signature.as_str()),
        ],
    );
    assert!(missing_query_response.contains("HTTP/1.1 404 Not Found"));
    let missing_payload = parse_error_envelope_from_http_response(missing_query_response.as_str());
    assert_eq!(missing_payload.error, "not-found");
    assert_eq!(missing_payload.reason_code, "service_api_route_not_found");

    let restart_server_result = restart_server
        .join()
        .expect("restart endpoint thread should complete");
    assert!(
        restart_server_result.is_ok(),
        "service api endpoint should stop cleanly after content lifecycle restart phase"
    );

    let state_payload = fs::read_to_string(state_file.as_path())
        .expect("content lifecycle state file should remain readable");
    let state_json: Value =
        serde_json::from_str(state_payload.as_str()).expect("state payload should parse");
    assert_eq!(
        state_json["contents"][content_id.as_str()]["lifecycle_state"],
        "tombstoned"
    );
    assert_eq!(
        state_json["contents"][content_id.as_str()]["redaction_status"],
        "redacted"
    );

    let _ = fs::remove_file(state_file);
}

#[test]
fn regression_service_api_endpoint_rejects_unknown_task_and_escrow_resource_transitions() {
    // Regression: #5866
    let _env = acquire_service_api_test_env();
    let parsed = parse_args(vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
        "--runtime-mode".to_owned(),
        "api".to_owned(),
        "--api-bind".to_owned(),
        "127.0.0.1:34112".to_owned(),
    ])
    .expect("api args should parse");
    let report = execute(parsed).expect("api execution should succeed");
    let snapshot = build_service_api_snapshot(&report);
    let state_hash = format!(
        "service-api:{}:{}",
        snapshot.chain_id.as_str(),
        snapshot.chain_version.as_str()
    );
    let caller_did = "kamn:did:agent:test-client-missing-resource";

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

    let accept_signature =
        service_api_request_signature_for_fields(caller_did, 71, state_hash.as_str(), "");
    let accept_response = send_http_request_with_headers(
        bind_addr.as_str(),
        "POST",
        "/v1/tasks/task-missing-71/accept",
        "",
        &[
            ("X-KAMN-Sender-DID", caller_did),
            ("X-KAMN-Request-Nonce", "71"),
            ("X-KAMN-Request-Signature", accept_signature.as_str()),
        ],
    );
    assert!(accept_response.contains("HTTP/1.1 404 Not Found"));
    let accept_payload = parse_error_envelope(extract_http_response_body(accept_response.as_str()));
    assert_eq!(accept_payload.error, "not-found");
    assert_eq!(accept_payload.reason_code, "service_api_route_not_found");

    let query_signature =
        service_api_request_signature_for_fields(caller_did, 72, state_hash.as_str(), "");
    let query_response = send_http_request_with_headers(
        bind_addr.as_str(),
        "GET",
        "/v1/tasks/task-missing-71",
        "",
        &[
            ("X-KAMN-Sender-DID", caller_did),
            ("X-KAMN-Request-Nonce", "72"),
            ("X-KAMN-Request-Signature", query_signature.as_str()),
        ],
    );
    assert!(query_response.contains("HTTP/1.1 404 Not Found"));
    let query_payload = parse_error_envelope(extract_http_response_body(query_response.as_str()));
    assert_eq!(query_payload.error, "not-found");
    assert_eq!(query_payload.reason_code, "service_api_route_not_found");

    let release_signature =
        service_api_request_signature_for_fields(caller_did, 73, state_hash.as_str(), "");
    let release_response = send_http_request_with_headers(
        bind_addr.as_str(),
        "POST",
        "/v1/escrow/escrow-missing-71/release",
        "",
        &[
            ("X-KAMN-Sender-DID", caller_did),
            ("X-KAMN-Request-Nonce", "73"),
            ("X-KAMN-Request-Signature", release_signature.as_str()),
        ],
    );
    assert!(release_response.contains("HTTP/1.1 404 Not Found"));
    let release_payload =
        parse_error_envelope(extract_http_response_body(release_response.as_str()));
    assert_eq!(release_payload.error, "not-found");
    assert_eq!(release_payload.reason_code, "service_api_route_not_found");

    let server_result = server.join().expect("endpoint thread should complete");
    assert!(
        server_result.is_ok(),
        "service api endpoint should stop cleanly after missing-resource regression flow"
    );
}

#[test]
fn integration_service_api_endpoint_recipient_mailbox_and_delivery_status_contract() {
    let _env = acquire_service_api_test_env();
    let unique_suffix = format!(
        "{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock should be monotonic")
            .as_nanos()
    );
    let state_file = std::env::temp_dir().join(format!(
        "kamn-node-service-api-recipient-delivery-state-{unique_suffix}.json"
    ));
    let relay_spool_file = std::env::temp_dir().join(format!(
        "kamn-node-service-api-recipient-delivery-spool-{unique_suffix}.ndjson"
    ));
    let state_file_str = state_file.to_string_lossy().to_string();
    let relay_spool_file_str = relay_spool_file.to_string_lossy().to_string();
    let _state_file_guard =
        EnvVarGuard::set("KAMN_SERVICE_API_STATE_FILE", Some(state_file_str.as_str()));
    let _relay_spool_guard = EnvVarGuard::set(
        "KAMN_SERVICE_API_RELAY_SPOOL_FILE",
        Some(relay_spool_file_str.as_str()),
    );

    let parsed = parse_args(vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
        "--runtime-mode".to_owned(),
        "api".to_owned(),
        "--api-bind".to_owned(),
        "127.0.0.1:34107".to_owned(),
    ])
    .expect("api args should parse");
    let report = execute(parsed).expect("api execution should succeed");
    let snapshot = build_service_api_snapshot(&report);
    let state_hash = format!(
        "service-api:{}:{}",
        snapshot.chain_id.as_str(),
        snapshot.chain_version.as_str()
    );
    let sender_did = "kamn:did:agent:delivery-sender";
    let recipient_did = "kamn:did:agent:delivery-recipient";

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

    let send_body =
        r#"{"recipient_did":"kamn:did:agent:delivery-recipient","message":"deliver-me"}"#;
    let send_signature =
        service_api_request_signature_for_fields(sender_did, 31, state_hash.as_str(), send_body);
    let send_response = send_http_request_with_headers(
        bind_addr.as_str(),
        "POST",
        "/v1/messages/send",
        send_body,
        &[
            ("X-KAMN-Sender-DID", sender_did),
            ("X-KAMN-Request-Nonce", "31"),
            ("X-KAMN-Request-Signature", send_signature.as_str()),
        ],
    );
    assert!(send_response.contains("HTTP/1.1 202 Accepted"));
    let send_payload: ServiceApiMessageCreateBody =
        parse_service_api_payload(extract_http_response_body(send_response.as_str()))
            .expect("send payload should deserialize");

    let mailbox_path = format!("/v1/channels/recipient:{recipient_did}/messages");
    let mailbox_signature =
        service_api_request_signature_for_fields(recipient_did, 32, state_hash.as_str(), "");
    let mailbox_response = send_http_request_with_headers(
        bind_addr.as_str(),
        "GET",
        mailbox_path.as_str(),
        "",
        &[
            ("X-KAMN-Sender-DID", recipient_did),
            ("X-KAMN-Request-Nonce", "32"),
            ("X-KAMN-Request-Signature", mailbox_signature.as_str()),
        ],
    );
    assert!(mailbox_response.contains("HTTP/1.1 200 OK"));
    let mailbox_payload: ServiceApiChannelMessagesBody =
        parse_service_api_payload(extract_http_response_body(mailbox_response.as_str()))
            .expect("mailbox payload should deserialize");
    assert!(
        mailbox_payload.messages.contains(&send_payload.message_id),
        "recipient mailbox projection should include sent message id"
    );

    let server_result = server.join().expect("endpoint thread should complete");
    assert!(
        server_result.is_ok(),
        "service api endpoint should stop cleanly after recipient mailbox projection flow"
    );

    let relay_spool_payload = fs::read_to_string(relay_spool_file.as_path())
        .expect("relay spool file should remain readable after send phase");
    let relay_line = relay_spool_payload
        .lines()
        .find(|line| !line.trim().is_empty())
        .expect("relay spool should include at least one recipient relay entry");
    let relay_entry_json: Value =
        serde_json::from_str(relay_line).expect("relay spool entry should deserialize");
    assert_eq!(relay_entry_json["message_id"], send_payload.message_id);
    assert_eq!(relay_entry_json["recipient_did"], recipient_did);

    let daemon_report = execute(
        parse_args(vec![
            "kamn-node".to_owned(),
            "--role".to_owned(),
            "processor".to_owned(),
            "--runtime-mode".to_owned(),
            "daemon".to_owned(),
            "--daemon-max-ticks".to_owned(),
            "1".to_owned(),
            "--daemon-tick-interval-ms".to_owned(),
            "1".to_owned(),
            "--daemon-shutdown-signal-tick".to_owned(),
            "1".to_owned(),
            "--daemon-shutdown-drain-ticks".to_owned(),
            "1".to_owned(),
            "--daemon-shutdown-timeout-ticks".to_owned(),
            "1".to_owned(),
        ])
        .expect("daemon args should parse for relay projection"),
    )
    .expect("daemon runtime should project relay status");
    assert_eq!(daemon_report.runtime_mode, "daemon");
    assert!(
        daemon_report
            .daemon_observability_throughput_tps
            .unwrap_or(0)
            > 0,
        "daemon observability throughput should reflect relay projection work"
    );

    let post_daemon_relay_contents = fs::read_to_string(relay_spool_file.as_path())
        .expect("relay spool file should remain readable after daemon projection");
    assert!(
        post_daemon_relay_contents.trim().is_empty(),
        "daemon relay projection should drain the relay spool"
    );

    let post_daemon_state_payload =
        fs::read_to_string(state_file.as_path()).expect("state file should remain readable");
    let post_daemon_state_json: Value =
        serde_json::from_str(post_daemon_state_payload.as_str()).expect("state json should parse");
    assert_eq!(
        post_daemon_state_json["messages"][send_payload.message_id.as_str()]["status"],
        "relayed",
        "daemon projection should advance created message to relayed before recipient delivery"
    );

    let restart_report = execute(
        parse_args(vec![
            "kamn-node".to_owned(),
            "--role".to_owned(),
            "processor".to_owned(),
            "--runtime-mode".to_owned(),
            "api".to_owned(),
            "--api-bind".to_owned(),
            "127.0.0.1:34113".to_owned(),
        ])
        .expect("restart api args should parse"),
    )
    .expect("restart api execution should succeed");
    let restart_snapshot = build_service_api_snapshot(&restart_report);
    let restart_state_hash = format!(
        "service-api:{}:{}",
        restart_snapshot.chain_id.as_str(),
        restart_snapshot.chain_version.as_str()
    );
    let restart_bind_addr = reserve_loopback_addr();
    let restart_config = ServiceApiEndpointConfig {
        bind_addr: restart_bind_addr.clone(),
        max_requests: 1,
        idle_timeout_ms: 2_000,
        body_limit_bytes: DEFAULT_SERVICE_API_BODY_LIMIT_BYTES,
        concurrency_limit: DEFAULT_SERVICE_API_CONCURRENCY_LIMIT,
        rate_limit_per_second: DEFAULT_SERVICE_API_RATE_LIMIT_PER_SECOND,
    };
    let restart_server_snapshot = restart_snapshot.clone();
    let restart_server = thread::spawn(move || {
        serve_service_api_endpoint(&restart_config, &restart_server_snapshot)
    });
    wait_for_endpoint_ready(restart_bind_addr.as_str());

    let message_path = format!("/v1/messages/{}", send_payload.message_id);
    let message_signature = service_api_request_signature_for_fields(
        recipient_did,
        33,
        restart_state_hash.as_str(),
        "",
    );
    let message_response = send_http_request_with_headers(
        restart_bind_addr.as_str(),
        "GET",
        message_path.as_str(),
        "",
        &[
            ("X-KAMN-Sender-DID", recipient_did),
            ("X-KAMN-Request-Nonce", "33"),
            ("X-KAMN-Request-Signature", message_signature.as_str()),
        ],
    );
    assert!(message_response.contains("HTTP/1.1 200 OK"));
    let message_payload: Value =
        parse_service_api_payload(extract_http_response_body(message_response.as_str()))
            .expect("message query payload should deserialize");
    assert_eq!(message_payload["message_id"], send_payload.message_id);
    assert_eq!(message_payload["status"], "delivered");
    assert_eq!(message_payload["sender_did"], sender_did);
    assert_eq!(message_payload["recipient_did"], recipient_did);
    assert_eq!(message_payload["body"], send_body);

    let restart_server_result = restart_server
        .join()
        .expect("restart endpoint thread should complete");
    assert!(
        restart_server_result.is_ok(),
        "service api endpoint should stop cleanly after recipient delivery contract flow"
    );

    let delivered_state_payload =
        fs::read_to_string(state_file.as_path()).expect("state file should remain readable");
    let delivered_state_json: Value =
        serde_json::from_str(delivered_state_payload.as_str()).expect("state json should parse");
    assert_eq!(
        delivered_state_json["messages"][send_payload.message_id.as_str()]["status"],
        "delivered",
        "recipient retrieval should deterministically advance relayed message to delivered"
    );

    let _ = fs::remove_file(state_file);
    let _ = fs::remove_file(relay_spool_file);
}

#[test]
fn integration_service_api_endpoint_cross_node_relay_delivery_contract() {
    let _env = acquire_service_api_test_env();
    let unique_suffix = format!(
        "{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock should be monotonic")
            .as_nanos()
    );
    let sender_state_file = std::env::temp_dir().join(format!(
        "kamn-node-service-api-cross-node-sender-state-{unique_suffix}.json"
    ));
    let sender_relay_spool_file = std::env::temp_dir().join(format!(
        "kamn-node-service-api-cross-node-sender-spool-{unique_suffix}.ndjson"
    ));
    let recipient_state_file = std::env::temp_dir().join(format!(
        "kamn-node-service-api-cross-node-recipient-state-{unique_suffix}.json"
    ));
    let recipient_relay_spool_file = std::env::temp_dir().join(format!(
        "kamn-node-service-api-cross-node-recipient-spool-{unique_suffix}.ndjson"
    ));
    let sender_state_file_str = sender_state_file.to_string_lossy().to_string();
    let sender_relay_spool_file_str = sender_relay_spool_file.to_string_lossy().to_string();
    let recipient_state_file_str = recipient_state_file.to_string_lossy().to_string();
    let recipient_relay_spool_file_str = recipient_relay_spool_file.to_string_lossy().to_string();

    let _sender_state_guard = EnvVarGuard::set(
        "KAMN_SERVICE_API_STATE_FILE",
        Some(sender_state_file_str.as_str()),
    );
    let _sender_relay_guard = EnvVarGuard::set(
        "KAMN_SERVICE_API_RELAY_SPOOL_FILE",
        Some(sender_relay_spool_file_str.as_str()),
    );
    let sender_bootstrap = execute(
        parse_args(vec![
            "kamn-node".to_owned(),
            "--role".to_owned(),
            "processor".to_owned(),
            "--runtime-mode".to_owned(),
            "api".to_owned(),
            "--api-bind".to_owned(),
            "127.0.0.1:34115".to_owned(),
        ])
        .expect("sender api args should parse"),
    )
    .expect("sender api execution should succeed");
    let sender_snapshot = build_service_api_snapshot(&sender_bootstrap);
    let sender_state_hash = format!(
        "service-api:{}:{}",
        sender_snapshot.chain_id.as_str(),
        sender_snapshot.chain_version.as_str()
    );

    let sender_bind_addr = reserve_loopback_addr();
    let sender_endpoint_config = ServiceApiEndpointConfig {
        bind_addr: sender_bind_addr.clone(),
        max_requests: 1,
        idle_timeout_ms: 2_000,
        body_limit_bytes: DEFAULT_SERVICE_API_BODY_LIMIT_BYTES,
        concurrency_limit: DEFAULT_SERVICE_API_CONCURRENCY_LIMIT,
        rate_limit_per_second: DEFAULT_SERVICE_API_RATE_LIMIT_PER_SECOND,
    };
    let sender_server_snapshot = sender_snapshot.clone();
    let sender_server = thread::spawn(move || {
        serve_service_api_endpoint(&sender_endpoint_config, &sender_server_snapshot)
    });
    wait_for_endpoint_ready(sender_bind_addr.as_str());

    let _recipient_state_guard = EnvVarGuard::set(
        "KAMN_SERVICE_API_STATE_FILE",
        Some(recipient_state_file_str.as_str()),
    );
    let _recipient_relay_guard = EnvVarGuard::set(
        "KAMN_SERVICE_API_RELAY_SPOOL_FILE",
        Some(recipient_relay_spool_file_str.as_str()),
    );
    let recipient_bootstrap = execute(
        parse_args(vec![
            "kamn-node".to_owned(),
            "--role".to_owned(),
            "processor".to_owned(),
            "--runtime-mode".to_owned(),
            "api".to_owned(),
            "--api-bind".to_owned(),
            "127.0.0.1:34116".to_owned(),
        ])
        .expect("recipient api args should parse"),
    )
    .expect("recipient api execution should succeed");
    let recipient_snapshot = build_service_api_snapshot(&recipient_bootstrap);
    let recipient_state_hash = format!(
        "service-api:{}:{}",
        recipient_snapshot.chain_id.as_str(),
        recipient_snapshot.chain_version.as_str()
    );

    let recipient_bind_addr = reserve_loopback_addr();
    let recipient_endpoint_config = ServiceApiEndpointConfig {
        bind_addr: recipient_bind_addr.clone(),
        max_requests: 4,
        idle_timeout_ms: 2_000,
        body_limit_bytes: DEFAULT_SERVICE_API_BODY_LIMIT_BYTES,
        concurrency_limit: DEFAULT_SERVICE_API_CONCURRENCY_LIMIT,
        rate_limit_per_second: DEFAULT_SERVICE_API_RATE_LIMIT_PER_SECOND,
    };
    let recipient_server_snapshot = recipient_snapshot.clone();
    let recipient_server = thread::spawn(move || {
        serve_service_api_endpoint(&recipient_endpoint_config, &recipient_server_snapshot)
    });
    wait_for_endpoint_ready(recipient_bind_addr.as_str());

    let sender_did = "kamn:did:agent:cross-node-sender";
    let recipient_did = "kamn:did:agent:cross-node-recipient";
    let send_body =
        r#"{"recipient_did":"kamn:did:agent:cross-node-recipient","message":"cross-node"}"#;
    let send_signature = service_api_request_signature_for_fields(
        sender_did,
        81,
        sender_state_hash.as_str(),
        send_body,
    );
    let send_response = send_http_request_with_headers(
        sender_bind_addr.as_str(),
        "POST",
        "/v1/messages/send",
        send_body,
        &[
            ("X-KAMN-Sender-DID", sender_did),
            ("X-KAMN-Request-Nonce", "81"),
            ("X-KAMN-Request-Signature", send_signature.as_str()),
        ],
    );
    assert!(send_response.contains("HTTP/1.1 202 Accepted"));
    let send_payload: ServiceApiMessageCreateBody =
        parse_service_api_payload(extract_http_response_body(send_response.as_str()))
            .expect("sender send payload should deserialize");
    let sender_server_result = sender_server
        .join()
        .expect("sender endpoint thread should complete");
    assert!(
        sender_server_result.is_ok(),
        "sender endpoint should stop cleanly after send request"
    );
    let sender_spool_seed = fs::read_to_string(sender_relay_spool_file.as_path())
        .expect("sender relay spool file should remain readable before daemon run");
    let sender_spool_seed_line = sender_spool_seed
        .lines()
        .find(|line| !line.trim().is_empty())
        .expect("sender relay spool should include at least one relay entry");
    fs::OpenOptions::new()
        .append(true)
        .open(sender_relay_spool_file.as_path())
        .expect("sender relay spool file should open for idempotency duplication")
        .write_all(format!("{sender_spool_seed_line}\n").as_bytes())
        .expect("duplicated relay spool line should append");

    {
        let relay_route_map = format!(r#"{{"{recipient_did}":"{recipient_bind_addr}"}}"#);
        let _daemon_sender_state_guard = EnvVarGuard::set(
            "KAMN_SERVICE_API_STATE_FILE",
            Some(sender_state_file_str.as_str()),
        );
        let _daemon_sender_spool_guard = EnvVarGuard::set(
            "KAMN_SERVICE_API_RELAY_SPOOL_FILE",
            Some(sender_relay_spool_file_str.as_str()),
        );
        let _relay_route_guard = EnvVarGuard::set(
            "KAMN_SERVICE_API_RELAY_RECIPIENT_ROUTE_MAP_JSON",
            Some(relay_route_map.as_str()),
        );
        let _daemon_private_key_guard = EnvVarGuard::set(
            "KAMN_SERVICE_API_AUTH_PRIVATE_KEY_HEX",
            Some(TEST_SERVICE_API_AUTH_PRIVATE_KEY_HEX),
        );

        let daemon_report = execute(
            parse_args(vec![
                "kamn-node".to_owned(),
                "--role".to_owned(),
                "processor".to_owned(),
                "--runtime-mode".to_owned(),
                "daemon".to_owned(),
                "--daemon-max-ticks".to_owned(),
                "1".to_owned(),
                "--daemon-tick-interval-ms".to_owned(),
                "1".to_owned(),
                "--daemon-shutdown-signal-tick".to_owned(),
                "1".to_owned(),
                "--daemon-shutdown-drain-ticks".to_owned(),
                "1".to_owned(),
                "--daemon-shutdown-timeout-ticks".to_owned(),
                "1".to_owned(),
            ])
            .expect("daemon args should parse"),
        )
        .expect("daemon relay projection should succeed");
        assert_eq!(daemon_report.runtime_mode, "daemon");
    }

    let recipient_mailbox_path = format!("/v1/channels/recipient:{recipient_did}/messages");
    let recipient_mailbox_signature = service_api_request_signature_for_fields(
        recipient_did,
        82,
        recipient_state_hash.as_str(),
        "",
    );
    let recipient_mailbox_response = send_http_request_with_headers(
        recipient_bind_addr.as_str(),
        "GET",
        recipient_mailbox_path.as_str(),
        "",
        &[
            ("X-KAMN-Sender-DID", recipient_did),
            ("X-KAMN-Request-Nonce", "82"),
            (
                "X-KAMN-Request-Signature",
                recipient_mailbox_signature.as_str(),
            ),
        ],
    );
    assert!(recipient_mailbox_response.contains("HTTP/1.1 200 OK"));
    let recipient_mailbox_payload: ServiceApiChannelMessagesBody = parse_service_api_payload(
        extract_http_response_body(recipient_mailbox_response.as_str()),
    )
    .expect("recipient mailbox payload should deserialize");
    let relayed_count = recipient_mailbox_payload
        .messages
        .iter()
        .filter(|message_id| *message_id == &send_payload.message_id)
        .count();
    assert_eq!(
        relayed_count, 1,
        "recipient mailbox should include the relayed sender message id exactly once"
    );

    let recipient_message_path = format!("/v1/messages/{}", send_payload.message_id);
    let recipient_message_signature = service_api_request_signature_for_fields(
        recipient_did,
        83,
        recipient_state_hash.as_str(),
        "",
    );
    let recipient_message_response = send_http_request_with_headers(
        recipient_bind_addr.as_str(),
        "GET",
        recipient_message_path.as_str(),
        "",
        &[
            ("X-KAMN-Sender-DID", recipient_did),
            ("X-KAMN-Request-Nonce", "83"),
            (
                "X-KAMN-Request-Signature",
                recipient_message_signature.as_str(),
            ),
        ],
    );
    assert!(recipient_message_response.contains("HTTP/1.1 200 OK"));
    let recipient_message_payload: Value = parse_service_api_payload(extract_http_response_body(
        recipient_message_response.as_str(),
    ))
    .expect("recipient message payload should deserialize");
    assert_eq!(
        recipient_message_payload["message_id"],
        send_payload.message_id
    );
    assert_eq!(recipient_message_payload["status"], "delivered");
    assert_eq!(recipient_message_payload["sender_did"], sender_did);
    assert_eq!(recipient_message_payload["recipient_did"], recipient_did);
    assert_eq!(recipient_message_payload["body"], send_body);

    let recipient_server_result = recipient_server
        .join()
        .expect("recipient endpoint thread should complete");
    assert!(
        recipient_server_result.is_ok(),
        "recipient endpoint should stop cleanly after relay(idempotent) + mailbox + message requests"
    );

    let restart_bootstrap = execute(
        parse_args(vec![
            "kamn-node".to_owned(),
            "--role".to_owned(),
            "processor".to_owned(),
            "--runtime-mode".to_owned(),
            "api".to_owned(),
            "--api-bind".to_owned(),
            "127.0.0.1:34117".to_owned(),
        ])
        .expect("recipient restart api args should parse"),
    )
    .expect("recipient restart api execution should succeed");
    let restart_snapshot = build_service_api_snapshot(&restart_bootstrap);
    let restart_state_hash = format!(
        "service-api:{}:{}",
        restart_snapshot.chain_id.as_str(),
        restart_snapshot.chain_version.as_str()
    );
    let restart_bind_addr = reserve_loopback_addr();
    let restart_endpoint_config = ServiceApiEndpointConfig {
        bind_addr: restart_bind_addr.clone(),
        max_requests: 1,
        idle_timeout_ms: 2_000,
        body_limit_bytes: DEFAULT_SERVICE_API_BODY_LIMIT_BYTES,
        concurrency_limit: DEFAULT_SERVICE_API_CONCURRENCY_LIMIT,
        rate_limit_per_second: DEFAULT_SERVICE_API_RATE_LIMIT_PER_SECOND,
    };
    let restart_server_snapshot = restart_snapshot.clone();
    let restart_server = thread::spawn(move || {
        serve_service_api_endpoint(&restart_endpoint_config, &restart_server_snapshot)
    });
    wait_for_endpoint_ready(restart_bind_addr.as_str());

    let restart_signature = service_api_request_signature_for_fields(
        recipient_did,
        84,
        restart_state_hash.as_str(),
        "",
    );
    let restart_response = send_http_request_with_headers(
        restart_bind_addr.as_str(),
        "GET",
        recipient_message_path.as_str(),
        "",
        &[
            ("X-KAMN-Sender-DID", recipient_did),
            ("X-KAMN-Request-Nonce", "84"),
            ("X-KAMN-Request-Signature", restart_signature.as_str()),
        ],
    );
    assert!(restart_response.contains("HTTP/1.1 200 OK"));
    let restart_payload: Value =
        parse_service_api_payload(extract_http_response_body(restart_response.as_str()))
            .expect("restart recipient payload should deserialize");
    assert_eq!(restart_payload["message_id"], send_payload.message_id);
    assert_eq!(restart_payload["status"], "delivered");

    let restart_server_result = restart_server
        .join()
        .expect("restart endpoint thread should complete");
    assert!(
        restart_server_result.is_ok(),
        "restart endpoint should stop cleanly after durable recipient query"
    );

    let sender_state_payload = fs::read_to_string(sender_state_file.as_path())
        .expect("sender state file should remain readable");
    let sender_state_json: Value = serde_json::from_str(sender_state_payload.as_str())
        .expect("sender state json should parse");
    assert_eq!(
        sender_state_json["messages"][send_payload.message_id.as_str()]["status"],
        "relayed",
        "sender state should project to relayed after successful cross-node forward"
    );

    let sender_relay_payload = fs::read_to_string(sender_relay_spool_file.as_path())
        .expect("sender relay spool file should remain readable");
    assert!(
        sender_relay_payload.trim().is_empty(),
        "sender relay spool should drain after successful cross-node forward"
    );

    let _ = fs::remove_file(sender_state_file);
    let _ = fs::remove_file(sender_relay_spool_file);
    let _ = fs::remove_file(recipient_state_file);
    let _ = fs::remove_file(recipient_relay_spool_file);
}

#[test]
fn regression_service_api_endpoint_recipient_query_requires_relayed_state_before_delivery() {
    // Regression: #5867
    let _env = acquire_service_api_test_env();
    let unique_suffix = format!(
        "{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock should be monotonic")
            .as_nanos()
    );
    let state_file = std::env::temp_dir().join(format!(
        "kamn-node-service-api-delivery-gate-state-{unique_suffix}.json"
    ));
    let relay_spool_file = std::env::temp_dir().join(format!(
        "kamn-node-service-api-delivery-gate-spool-{unique_suffix}.ndjson"
    ));
    let state_file_str = state_file.to_string_lossy().to_string();
    let relay_spool_file_str = relay_spool_file.to_string_lossy().to_string();
    let _state_file_guard =
        EnvVarGuard::set("KAMN_SERVICE_API_STATE_FILE", Some(state_file_str.as_str()));
    let _relay_spool_guard = EnvVarGuard::set(
        "KAMN_SERVICE_API_RELAY_SPOOL_FILE",
        Some(relay_spool_file_str.as_str()),
    );

    let parsed = parse_args(vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
        "--runtime-mode".to_owned(),
        "api".to_owned(),
        "--api-bind".to_owned(),
        "127.0.0.1:34114".to_owned(),
    ])
    .expect("api args should parse");
    let report = execute(parsed).expect("api execution should succeed");
    let snapshot = build_service_api_snapshot(&report);
    let state_hash = format!(
        "service-api:{}:{}",
        snapshot.chain_id.as_str(),
        snapshot.chain_version.as_str()
    );
    let sender_did = "kamn:did:agent:delivery-gate-sender";
    let recipient_did = "kamn:did:agent:delivery-gate-recipient";

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

    let send_body =
        r#"{"recipient_did":"kamn:did:agent:delivery-gate-recipient","message":"deliver-gate"}"#;
    let send_signature =
        service_api_request_signature_for_fields(sender_did, 71, state_hash.as_str(), send_body);
    let send_response = send_http_request_with_headers(
        bind_addr.as_str(),
        "POST",
        "/v1/messages/send",
        send_body,
        &[
            ("X-KAMN-Sender-DID", sender_did),
            ("X-KAMN-Request-Nonce", "71"),
            ("X-KAMN-Request-Signature", send_signature.as_str()),
        ],
    );
    assert!(send_response.contains("HTTP/1.1 202 Accepted"));
    let send_payload: ServiceApiMessageCreateBody =
        parse_service_api_payload(extract_http_response_body(send_response.as_str()))
            .expect("send payload should deserialize");

    let message_path = format!("/v1/messages/{}", send_payload.message_id);
    let query_signature =
        service_api_request_signature_for_fields(recipient_did, 72, state_hash.as_str(), "");
    let query_response = send_http_request_with_headers(
        bind_addr.as_str(),
        "GET",
        message_path.as_str(),
        "",
        &[
            ("X-KAMN-Sender-DID", recipient_did),
            ("X-KAMN-Request-Nonce", "72"),
            ("X-KAMN-Request-Signature", query_signature.as_str()),
        ],
    );
    assert!(query_response.contains("HTTP/1.1 200 OK"));
    let query_payload: Value =
        parse_service_api_payload(extract_http_response_body(query_response.as_str()))
            .expect("recipient query payload should deserialize");
    assert_eq!(query_payload["message_id"], send_payload.message_id);
    assert_eq!(
        query_payload["status"], "created",
        "recipient query must not mark message delivered before daemon relay projection"
    );

    let server_result = server.join().expect("endpoint thread should complete");
    assert!(
        server_result.is_ok(),
        "service api endpoint should stop cleanly after delivery gate regression flow"
    );

    let state_payload =
        fs::read_to_string(state_file.as_path()).expect("state file should remain readable");
    let state_json: Value =
        serde_json::from_str(state_payload.as_str()).expect("state payload should parse");
    assert_eq!(
        state_json["messages"][send_payload.message_id.as_str()]["status"],
        "created",
        "state file must keep pre-relay status until daemon projection runs"
    );

    let _ = fs::remove_file(state_file);
    let _ = fs::remove_file(relay_spool_file);
}

#[test]
fn integration_service_api_endpoint_recipient_query_promotes_relayed_to_delivered() {
    let _env = acquire_service_api_test_env();
    let unique_suffix = format!(
        "{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock should be monotonic")
            .as_nanos()
    );
    let state_file = std::env::temp_dir().join(format!(
        "kamn-node-service-api-relayed-to-delivered-state-{unique_suffix}.json"
    ));
    let state_file_str = state_file.to_string_lossy().to_string();
    std::fs::write(
        state_file.as_path(),
        r#"{
  "schema_version":"kamn.runtime.service-api-message-store.v2",
  "messages":{
    "msg-relayed-to-delivered-1":{
      "message_id":"msg-relayed-to-delivered-1",
      "status":"relayed",
      "channel_id":null,
      "sender_did":"kamn:did:agent:sender-relayed",
      "recipient_did":"kamn:did:agent:recipient-relayed",
      "body":"{\"recipient_did\":\"kamn:did:agent:recipient-relayed\",\"message\":\"relay-complete\"}"
    }
  },
  "channel_messages":{
    "recipient:kamn:did:agent:recipient-relayed":["msg-relayed-to-delivered-1"]
  },
  "tasks":{},
  "escrows":{}
}"#,
    )
    .expect("state fixture should write");
    let _state_file_guard =
        EnvVarGuard::set("KAMN_SERVICE_API_STATE_FILE", Some(state_file_str.as_str()));

    let parsed = parse_args(vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
        "--runtime-mode".to_owned(),
        "api".to_owned(),
        "--api-bind".to_owned(),
        "127.0.0.1:34109".to_owned(),
    ])
    .expect("api args should parse");
    let report = execute(parsed).expect("api execution should succeed");
    let snapshot = build_service_api_snapshot(&report);
    let state_hash = format!(
        "service-api:{}:{}",
        snapshot.chain_id.as_str(),
        snapshot.chain_version.as_str()
    );
    let non_recipient_did = "kamn:did:agent:recipient-relayed-observer";
    let recipient_did = "kamn:did:agent:recipient-relayed";

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

    let non_recipient_signature =
        service_api_request_signature_for_fields(non_recipient_did, 50, state_hash.as_str(), "");
    let non_recipient_response = send_http_request_with_headers(
        bind_addr.as_str(),
        "GET",
        "/v1/messages/msg-relayed-to-delivered-1",
        "",
        &[
            ("X-KAMN-Sender-DID", non_recipient_did),
            ("X-KAMN-Request-Nonce", "50"),
            ("X-KAMN-Request-Signature", non_recipient_signature.as_str()),
        ],
    );
    assert!(non_recipient_response.contains("HTTP/1.1 200 OK"));
    let non_recipient_payload: Value =
        parse_service_api_payload(extract_http_response_body(non_recipient_response.as_str()))
            .expect("non-recipient payload should deserialize");
    assert_eq!(
        non_recipient_payload["message_id"],
        "msg-relayed-to-delivered-1"
    );
    assert_eq!(
        non_recipient_payload["status"], "relayed",
        "non-recipient retrieval must not mark relayed message as delivered"
    );

    let message_signature =
        service_api_request_signature_for_fields(recipient_did, 51, state_hash.as_str(), "");
    let message_response = send_http_request_with_headers(
        bind_addr.as_str(),
        "GET",
        "/v1/messages/msg-relayed-to-delivered-1",
        "",
        &[
            ("X-KAMN-Sender-DID", recipient_did),
            ("X-KAMN-Request-Nonce", "51"),
            ("X-KAMN-Request-Signature", message_signature.as_str()),
        ],
    );
    assert!(message_response.contains("HTTP/1.1 200 OK"));
    let message_payload: Value =
        parse_service_api_payload(extract_http_response_body(message_response.as_str()))
            .expect("message query payload should deserialize");
    assert_eq!(message_payload["message_id"], "msg-relayed-to-delivered-1");
    assert_eq!(
        message_payload["status"], "delivered",
        "recipient query should promote relayed status to delivered"
    );

    let server_result = server.join().expect("endpoint thread should complete");
    assert!(
        server_result.is_ok(),
        "service api endpoint should stop cleanly after relayed recipient query flow"
    );
    let _ = fs::remove_file(state_file);
}

#[test]
fn regression_service_api_endpoint_non_recipient_query_keeps_relayed_status_across_restart() {
    // Regression: #5979
    let _env = acquire_service_api_test_env();
    let unique_suffix = format!(
        "{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock should be monotonic")
            .as_nanos()
    );
    let state_file = std::env::temp_dir().join(format!(
        "kamn-node-service-api-relayed-non-recipient-restart-state-{unique_suffix}.json"
    ));
    let state_file_str = state_file.to_string_lossy().to_string();
    std::fs::write(
        state_file.as_path(),
        r#"{
  "schema_version":"kamn.runtime.service-api-message-store.v2",
  "messages":{
    "msg-relayed-non-recipient-restart-1":{
      "message_id":"msg-relayed-non-recipient-restart-1",
      "status":"relayed",
      "channel_id":null,
      "sender_did":"kamn:did:agent:sender-relayed-restart",
      "recipient_did":"kamn:did:agent:recipient-relayed-restart",
      "body":"{\"recipient_did\":\"kamn:did:agent:recipient-relayed-restart\",\"message\":\"relay-restart\"}"
    }
  },
  "channel_messages":{
    "recipient:kamn:did:agent:recipient-relayed-restart":["msg-relayed-non-recipient-restart-1"]
  },
  "tasks":{},
  "escrows":{}
}"#,
    )
    .expect("state fixture should write");
    let _state_file_guard =
        EnvVarGuard::set("KAMN_SERVICE_API_STATE_FILE", Some(state_file_str.as_str()));

    let non_recipient_did = "kamn:did:agent:recipient-relayed-restart-observer";
    let message_path = "/v1/messages/msg-relayed-non-recipient-restart-1";

    let first_report = execute(
        parse_args(vec![
            "kamn-node".to_owned(),
            "--role".to_owned(),
            "processor".to_owned(),
            "--runtime-mode".to_owned(),
            "api".to_owned(),
            "--api-bind".to_owned(),
            "127.0.0.1:34115".to_owned(),
        ])
        .expect("first api args should parse"),
    )
    .expect("first api execution should succeed");
    let first_snapshot = build_service_api_snapshot(&first_report);
    let first_state_hash = format!(
        "service-api:{}:{}",
        first_snapshot.chain_id.as_str(),
        first_snapshot.chain_version.as_str()
    );

    let first_bind_addr = reserve_loopback_addr();
    let first_config = ServiceApiEndpointConfig {
        bind_addr: first_bind_addr.clone(),
        max_requests: 1,
        idle_timeout_ms: 2_000,
        body_limit_bytes: DEFAULT_SERVICE_API_BODY_LIMIT_BYTES,
        concurrency_limit: DEFAULT_SERVICE_API_CONCURRENCY_LIMIT,
        rate_limit_per_second: DEFAULT_SERVICE_API_RATE_LIMIT_PER_SECOND,
    };
    let first_server_snapshot = first_snapshot.clone();
    let first_server =
        thread::spawn(move || serve_service_api_endpoint(&first_config, &first_server_snapshot));
    wait_for_endpoint_ready(first_bind_addr.as_str());

    let first_signature = service_api_request_signature_for_fields(
        non_recipient_did,
        81,
        first_state_hash.as_str(),
        "",
    );
    let first_response = send_http_request_with_headers(
        first_bind_addr.as_str(),
        "GET",
        message_path,
        "",
        &[
            ("X-KAMN-Sender-DID", non_recipient_did),
            ("X-KAMN-Request-Nonce", "81"),
            ("X-KAMN-Request-Signature", first_signature.as_str()),
        ],
    );
    assert!(first_response.contains("HTTP/1.1 200 OK"));
    let first_payload: Value =
        parse_service_api_payload(extract_http_response_body(first_response.as_str()))
            .expect("first non-recipient payload should deserialize");
    assert_eq!(
        first_payload["message_id"],
        "msg-relayed-non-recipient-restart-1"
    );
    assert_eq!(
        first_payload["status"], "relayed",
        "non-recipient retrieval must keep relayed status before restart"
    );

    let first_server_result = first_server
        .join()
        .expect("first endpoint thread should complete");
    assert!(
        first_server_result.is_ok(),
        "service api endpoint should stop cleanly after first non-recipient relay query"
    );

    let restart_report = execute(
        parse_args(vec![
            "kamn-node".to_owned(),
            "--role".to_owned(),
            "processor".to_owned(),
            "--runtime-mode".to_owned(),
            "api".to_owned(),
            "--api-bind".to_owned(),
            "127.0.0.1:34116".to_owned(),
        ])
        .expect("restart api args should parse"),
    )
    .expect("restart api execution should succeed");
    let restart_snapshot = build_service_api_snapshot(&restart_report);
    let restart_state_hash = format!(
        "service-api:{}:{}",
        restart_snapshot.chain_id.as_str(),
        restart_snapshot.chain_version.as_str()
    );
    let restart_bind_addr = reserve_loopback_addr();
    let restart_config = ServiceApiEndpointConfig {
        bind_addr: restart_bind_addr.clone(),
        max_requests: 1,
        idle_timeout_ms: 2_000,
        body_limit_bytes: DEFAULT_SERVICE_API_BODY_LIMIT_BYTES,
        concurrency_limit: DEFAULT_SERVICE_API_CONCURRENCY_LIMIT,
        rate_limit_per_second: DEFAULT_SERVICE_API_RATE_LIMIT_PER_SECOND,
    };
    let restart_server_snapshot = restart_snapshot.clone();
    let restart_server = thread::spawn(move || {
        serve_service_api_endpoint(&restart_config, &restart_server_snapshot)
    });
    wait_for_endpoint_ready(restart_bind_addr.as_str());

    let restart_signature = service_api_request_signature_for_fields(
        non_recipient_did,
        82,
        restart_state_hash.as_str(),
        "",
    );
    let restart_response = send_http_request_with_headers(
        restart_bind_addr.as_str(),
        "GET",
        message_path,
        "",
        &[
            ("X-KAMN-Sender-DID", non_recipient_did),
            ("X-KAMN-Request-Nonce", "82"),
            ("X-KAMN-Request-Signature", restart_signature.as_str()),
        ],
    );
    assert!(restart_response.contains("HTTP/1.1 200 OK"));
    let restart_payload: Value =
        parse_service_api_payload(extract_http_response_body(restart_response.as_str()))
            .expect("restart non-recipient payload should deserialize");
    assert_eq!(
        restart_payload["message_id"],
        "msg-relayed-non-recipient-restart-1"
    );
    assert_eq!(
        restart_payload["status"], "relayed",
        "non-recipient retrieval must remain relayed after restart"
    );

    let restart_server_result = restart_server
        .join()
        .expect("restart endpoint thread should complete");
    assert!(
        restart_server_result.is_ok(),
        "service api endpoint should stop cleanly after restart non-recipient relay query"
    );

    let state_payload =
        fs::read_to_string(state_file.as_path()).expect("state file should remain readable");
    let state_json: Value =
        serde_json::from_str(state_payload.as_str()).expect("state payload should parse");
    assert_eq!(
        state_json["messages"]["msg-relayed-non-recipient-restart-1"]["status"], "relayed",
        "state file should retain relayed status for non-recipient queries across restart"
    );

    let _ = fs::remove_file(state_file);
}

#[test]
fn integration_service_api_endpoint_enqueues_recipient_relays_to_durable_spool() {
    let _env = acquire_service_api_test_env();
    let unique_suffix = format!(
        "{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock should be monotonic")
            .as_nanos()
    );
    let state_file = std::env::temp_dir().join(format!(
        "kamn-node-service-api-relay-spool-state-{unique_suffix}.json"
    ));
    let relay_spool_file = std::env::temp_dir().join(format!(
        "kamn-node-service-api-relay-spool-{unique_suffix}.ndjson"
    ));
    let state_file_str = state_file.to_string_lossy().to_string();
    let relay_spool_file_str = relay_spool_file.to_string_lossy().to_string();
    let _state_file_guard =
        EnvVarGuard::set("KAMN_SERVICE_API_STATE_FILE", Some(state_file_str.as_str()));
    let _relay_spool_guard = EnvVarGuard::set(
        "KAMN_SERVICE_API_RELAY_SPOOL_FILE",
        Some(relay_spool_file_str.as_str()),
    );

    let parsed = parse_args(vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
        "--runtime-mode".to_owned(),
        "api".to_owned(),
        "--api-bind".to_owned(),
        "127.0.0.1:34108".to_owned(),
    ])
    .expect("api args should parse");
    let report = execute(parsed).expect("api execution should succeed");
    let snapshot = build_service_api_snapshot(&report);
    let state_hash = format!(
        "service-api:{}:{}",
        snapshot.chain_id.as_str(),
        snapshot.chain_version.as_str()
    );
    let sender_did = "kamn:did:agent:relay-spool-sender";
    let recipient_did = "kamn:did:agent:relay-spool-recipient";

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

    let send_body =
        r#"{"recipient_did":"kamn:did:agent:relay-spool-recipient","message":"relay-me"}"#;
    let send_signature =
        service_api_request_signature_for_fields(sender_did, 41, state_hash.as_str(), send_body);
    let send_response = send_http_request_with_headers(
        bind_addr.as_str(),
        "POST",
        "/v1/messages/send",
        send_body,
        &[
            ("X-KAMN-Sender-DID", sender_did),
            ("X-KAMN-Request-Nonce", "41"),
            ("X-KAMN-Request-Signature", send_signature.as_str()),
        ],
    );
    assert!(send_response.contains("HTTP/1.1 202 Accepted"));
    let send_payload: ServiceApiMessageCreateBody =
        parse_service_api_payload(extract_http_response_body(send_response.as_str()))
            .expect("send payload should deserialize");

    let server_result = server.join().expect("endpoint thread should complete");
    assert!(
        server_result.is_ok(),
        "service api endpoint should stop cleanly after relay spool enqueue request"
    );

    let relay_contents = fs::read_to_string(relay_spool_file.as_path())
        .expect("relay spool file should exist after recipient send");
    let relay_line = relay_contents
        .lines()
        .find(|line| !line.trim().is_empty())
        .expect("relay spool should contain at least one entry");
    let relay_payload: Value =
        serde_json::from_str(relay_line).expect("relay spool line should be valid json");
    assert_eq!(relay_payload["message_id"], send_payload.message_id);
    assert_eq!(relay_payload["sender_did"], sender_did);
    assert_eq!(relay_payload["recipient_did"], recipient_did);
    assert_eq!(relay_payload["body"], send_body);

    let _ = fs::remove_file(state_file);
    let _ = fs::remove_file(relay_spool_file);
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
    let first_signature = service_api_request_signature_for_fields(
        sender_did,
        101,
        state_hash.as_str(),
        message_body,
    );
    let second_signature = service_api_request_signature_for_fields(
        sender_did,
        102,
        state_hash.as_str(),
        message_body,
    );

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
        let signature = service_api_request_signature_for_fields(
            sender_did,
            nonce,
            state_hash.as_str(),
            message_body,
        );
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
            let signature = service_api_request_signature_for_fields(
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
            let signature = service_api_request_signature_for_fields(
                sender_did,
                nonce,
                state_hash.as_str(),
                message_body,
            );
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
                let signature = service_api_request_signature_for_fields(
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
    let signature = service_api_request_signature_for_fields(
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
    let signature = service_api_request_signature_for_fields(
        sender_did,
        sender_nonce,
        state_hash.as_str(),
        message_body,
    );
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
    let signature_nonce_one = service_api_request_signature_for_fields(
        sender_did,
        701,
        state_hash.as_str(),
        message_body,
    );
    let signature_nonce_two = service_api_request_signature_for_fields(
        sender_did,
        702,
        state_hash.as_str(),
        message_body,
    );

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
        let signature = service_api_request_signature_for_fields(
            sender_did,
            nonce,
            state_hash.as_str(),
            body.as_str(),
        );
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
    let signature =
        service_api_request_signature_for_fields(sender_did, nonce, state_hash.as_str(), "");
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
fn integration_service_api_endpoint_websocket_upgrade_keeps_connection_open_after_initial_event() {
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

    let sender_did = "kamn:did:agent:ws-client-multi";
    let nonce = 57_u64;
    let state_hash = format!(
        "service-api:{}:{}",
        snapshot.chain_id.as_str(),
        snapshot.chain_version.as_str()
    );
    let signature =
        service_api_request_signature_for_fields(sender_did, nonce, state_hash.as_str(), "");
    let read_start = Instant::now();
    let (response, peer_closed) = send_websocket_upgrade_request_with_version_close_observation(
        bind_addr.as_str(),
        "/v1/events/ws",
        "13",
        &[
            ("X-KAMN-Sender-DID", sender_did),
            ("X-KAMN-Request-Nonce", "57"),
            ("X-KAMN-Request-Signature", signature.as_str()),
        ],
    );
    let (_header, frames) = parse_websocket_response_frames(response.as_slice());
    assert!(
        !frames.is_empty(),
        "websocket stream should emit an initial state-transition event frame"
    );
    let first: Value = serde_json::from_str(frames[0].as_str())
        .expect("initial websocket state-transition frame should be json");
    assert_eq!(
        first.get("event").and_then(Value::as_str),
        Some("state-transition")
    );
    assert_eq!(first.get("sequence").and_then(Value::as_u64), Some(1));
    let read_elapsed = read_start.elapsed();
    let remained_open_or_timed_out = !peer_closed || read_elapsed >= Duration::from_millis(1_500);
    assert!(
        remained_open_or_timed_out,
        "websocket stream should not close immediately after initial frame; peer_closed={peer_closed} elapsed={read_elapsed:?}"
    );
    let server_result = server.join().expect("endpoint thread should complete");
    let ended_cleanly_or_timeout = match &server_result {
        Ok(()) => true,
        Err(error) => error.contains("service api timed out after"),
    };
    assert!(
        ended_cleanly_or_timeout,
        "websocket keep-open test should end via request budget completion or idle-timeout fail-close: {server_result:?}"
    );
}

#[test]
fn regression_service_api_endpoint_websocket_stream_delivers_live_message_event_after_upgrade() {
    // Regression: #5905
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

    let bind_addr = reserve_loopback_addr();
    let endpoint_config = ServiceApiEndpointConfig {
        bind_addr: bind_addr.clone(),
        max_requests: 6,
        idle_timeout_ms: 2_000,
        body_limit_bytes: DEFAULT_SERVICE_API_BODY_LIMIT_BYTES,
        concurrency_limit: DEFAULT_SERVICE_API_CONCURRENCY_LIMIT,
        rate_limit_per_second: DEFAULT_SERVICE_API_RATE_LIMIT_PER_SECOND,
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
    let websocket_sender_did = "kamn:did:agent:ws-live-stream-client";
    let websocket_signature = service_api_request_signature_for_fields(
        websocket_sender_did,
        601,
        state_hash.as_str(),
        "",
    );

    let post_bind_addr = bind_addr.clone();
    let post_state_hash = state_hash.clone();
    let post_thread = thread::spawn(move || {
        thread::sleep(Duration::from_millis(75));
        let sender_did = "kamn:did:agent:ws-live-stream-publisher";
        let first_body = "{\"message\":\"websocket-live-event-1\"}";
        let first_signature =
            service_api_request_signature_for_fields(sender_did, 602, &post_state_hash, first_body);
        let first_response = send_http_request_with_headers(
            post_bind_addr.as_str(),
            "POST",
            "/v1/messages/send",
            first_body,
            &[
                ("X-KAMN-Sender-DID", sender_did),
                ("X-KAMN-Request-Nonce", "602"),
                ("X-KAMN-Request-Signature", first_signature.as_str()),
                ("X-KAMN-Authz-Scope", "messages:write"),
            ],
        );
        thread::sleep(Duration::from_millis(25));
        let second_body = "{\"message\":\"websocket-live-event-2\"}";
        let second_signature = service_api_request_signature_for_fields(
            sender_did,
            603,
            &post_state_hash,
            second_body,
        );
        let second_response = send_http_request_with_headers(
            post_bind_addr.as_str(),
            "POST",
            "/v1/messages/send",
            second_body,
            &[
                ("X-KAMN-Sender-DID", sender_did),
                ("X-KAMN-Request-Nonce", "603"),
                ("X-KAMN-Request-Signature", second_signature.as_str()),
                ("X-KAMN-Authz-Scope", "messages:write"),
            ],
        );
        (first_response, second_response)
    });

    let websocket_response = send_websocket_upgrade_request(
        bind_addr.as_str(),
        "/v1/events/ws",
        &[
            ("X-KAMN-Sender-DID", websocket_sender_did),
            ("X-KAMN-Request-Nonce", "601"),
            ("X-KAMN-Request-Signature", websocket_signature.as_str()),
        ],
    );
    let (first_post_response, second_post_response) = post_thread
        .join()
        .expect("post request thread should complete");
    assert!(
        first_post_response.contains("HTTP/1.1 202 Accepted"),
        "first publisher request should be accepted: {first_post_response}"
    );
    assert!(
        second_post_response.contains("HTTP/1.1 202 Accepted"),
        "second publisher request should be accepted: {second_post_response}"
    );

    let (_header, frames) = parse_websocket_response_frames(websocket_response.as_slice());
    let created_sequences = frames
        .iter()
        .filter_map(|frame| {
            let payload: Value = serde_json::from_str(frame).ok()?;
            if payload.get("event").and_then(Value::as_str) != Some("service-api.message.created") {
                return None;
            }
            payload.get("sequence").and_then(Value::as_u64)
        })
        .collect::<Vec<u64>>();
    assert!(
        created_sequences.len() >= 2,
        "websocket stream should include multiple live message-created event frames after upgrade: {frames:?}"
    );
    let mut unique_sequences = created_sequences;
    unique_sequences.sort_unstable();
    unique_sequences.dedup();
    assert!(
        unique_sequences.len() >= 2
            && unique_sequences[1] > unique_sequences[0],
        "message-created websocket event sequence should advance across events: {unique_sequences:?}"
    );

    let server_result = server.join().expect("endpoint thread should complete");
    let ended_cleanly_or_timeout = match &server_result {
        Ok(()) => true,
        Err(error) => error.contains("service api timed out after"),
    };
    assert!(
        ended_cleanly_or_timeout,
        "service api endpoint should end via request budget completion or idle-timeout fail-close after websocket live stream regression test: {server_result:?}"
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
    let signature =
        service_api_request_signature_for_fields(sender_did, nonce, state_hash.as_str(), "");
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
    let signature =
        service_api_request_signature_for_fields(sender_did, nonce, state_hash.as_str(), "");
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
    let signature =
        service_api_request_signature_for_fields(sender_did, nonce, state_hash.as_str(), "");
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
    let signature =
        service_api_request_signature_for_fields(sender_did, nonce, state_hash.as_str(), "");
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
    let signature =
        service_api_request_signature_for_fields(sender_did, nonce, state_hash.as_str(), "");
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
    let signature =
        service_api_request_signature_for_fields(sender_did, nonce, state_hash.as_str(), "");
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
            let signature = service_api_request_signature_for_fields(
                sender_did,
                nonce,
                state_hash.as_str(),
                message_body,
            );
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

    let state_hash = format!(
        "service-api:{}:{}",
        snapshot.chain_id.as_str(),
        snapshot.chain_version.as_str()
    );

    for round in 0..rounds {
        let sender_did = format!("kamn:did:agent:test-client-concurrency-regression-{round}");
        let barrier = Arc::new(Barrier::new(worker_count));
        let mut clients = Vec::with_capacity(worker_count);
        for request_index in 0..worker_count {
            let client_bind_addr = bind_addr.clone();
            let barrier = barrier.clone();
            let sender_did = sender_did.clone();
            let state_hash = state_hash.clone();
            clients.push(thread::spawn(move || {
                let body = format!(
                    "{{\"message\":\"concurrency-stability-round-{round}-request-{request_index}\"}}"
                );
                let nonce = 4_000 + round * worker_count as u64 + request_index as u64;
                let signature =
                    service_api_request_signature_for_fields(
                        sender_did.as_str(),
                        nonce,
                        state_hash.as_str(),
                        &body,
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

#[test]
fn regression_service_api_endpoint_unauthorized_ingress_consumes_request_budget() {
    // Regression: #5903
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

    let bind_addr = reserve_loopback_addr();
    let endpoint_config = ServiceApiEndpointConfig {
        bind_addr: bind_addr.clone(),
        max_requests: 1,
        idle_timeout_ms: 80,
        body_limit_bytes: DEFAULT_SERVICE_API_BODY_LIMIT_BYTES,
        concurrency_limit: DEFAULT_SERVICE_API_CONCURRENCY_LIMIT,
        rate_limit_per_second: DEFAULT_SERVICE_API_RATE_LIMIT_PER_SECOND,
    };
    let server_snapshot = snapshot.clone();
    let server =
        thread::spawn(move || serve_service_api_endpoint(&endpoint_config, &server_snapshot));
    wait_for_endpoint_ready(bind_addr.as_str());

    let response = send_http_request(
        bind_addr.as_str(),
        "POST",
        "/v1/messages/send",
        "{\"message\":\"unsigned\"}",
    );
    let server_result = server.join().expect("endpoint thread should complete");

    assert!(
        response.contains("HTTP/1.1 401 Unauthorized"),
        "unsigned ingress must fail closed with unauthorized status: {response}"
    );
    let payload = parse_error_envelope_from_http_response(response.as_str());
    assert_eq!(
        payload.reason_code,
        SERVICE_API_AUTH_MISSING_HEADER_REASON_CODE
    );
    assert!(
        server_result.is_ok(),
        "unauthorized ingress must still consume request budget for graceful shutdown"
    );
}

#[test]
fn regression_service_api_endpoint_returns_timeout_error_when_no_requests_arrive() {
    // Regression: #5903
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

    let bind_addr = reserve_loopback_addr();
    let endpoint_config = ServiceApiEndpointConfig {
        bind_addr,
        max_requests: 1,
        idle_timeout_ms: 40,
        body_limit_bytes: DEFAULT_SERVICE_API_BODY_LIMIT_BYTES,
        concurrency_limit: DEFAULT_SERVICE_API_CONCURRENCY_LIMIT,
        rate_limit_per_second: DEFAULT_SERVICE_API_RATE_LIMIT_PER_SECOND,
    };

    let started = Instant::now();
    let result = serve_service_api_endpoint(&endpoint_config, &snapshot);
    assert!(
        result.is_err(),
        "endpoint must fail closed with timeout when request budget is never consumed"
    );
    let error = result.expect_err("timeout error should be returned");
    assert!(
        error.contains("service api timed out after 40 ms waiting for requests"),
        "timeout error should include configured budget: {error}"
    );
    assert!(
        started.elapsed() <= Duration::from_secs(1),
        "timeout regression should complete quickly for local tests"
    );
}
