use super::*;
use crate::service_api_endpoint::{
    DEFAULT_SERVICE_API_BODY_LIMIT_BYTES, DEFAULT_SERVICE_API_CONCURRENCY_LIMIT,
    DEFAULT_SERVICE_API_RATE_LIMIT_PER_SECOND, SERVICE_API_AUTH_REASON_CODES_CSV,
    SERVICE_API_AUTH_REASON_TAXONOMY_VERSION, SERVICE_API_LIFECYCLE_REJECTION_REASON_CODES_CSV,
    SERVICE_API_LIFECYCLE_REJECTION_REASON_TAXONOMY_VERSION,
    SERVICE_API_ROUTE_AUTHZ_MATRIX_PROTECTED_ROUTE_COUNT,
    SERVICE_API_ROUTE_AUTHZ_MATRIX_PUBLIC_ROUTE_COUNT,
    SERVICE_API_ROUTE_AUTHZ_MATRIX_SCHEMA_VERSION,
    SERVICE_API_ROUTE_AUTHZ_MATRIX_TOTAL_ROUTE_COUNT,
    SERVICE_API_SCOPE_POLICY_FIXTURE_SCHEMA_VERSION, SERVICE_API_SCOPE_POLICY_REASON_CODES_CSV,
    SERVICE_API_SCOPE_POLICY_REASON_TAXONOMY_VERSION, SERVICE_API_WEBSOCKET_REASON_CODES_CSV,
    SERVICE_API_WEBSOCKET_REASON_TAXONOMY_VERSION, ServiceApiAgentGetBody,
    ServiceApiChannelCreateBody, ServiceApiChannelMessagesBody, ServiceApiErrorBody,
    ServiceApiHealthBody, ServiceApiLifecycleRejectionProjection, ServiceApiMessageCreateBody,
    ServiceApiMessageGetBody, ServiceApiRelaySpoolEntry, ServiceApiTaskCreateBody,
    parse_service_api_payload, project_service_api_lifecycle_rejection,
    upsert_service_api_relayed_message_from_daemon,
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

#[path = "service_api_endpoint_tests/balance_contract_tests.rs"]
mod balance_contract_tests;
#[path = "service_api_endpoint_tests/websocket_contract_tests.rs"]
mod websocket_contract_tests;

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
const SERVICE_API_AGENT_DID_PATH_INVALID_REASON_CODE: &str = "service_api_agent_did_path_invalid";
const SERVICE_API_MESSAGE_RECIPIENT_DID_INVALID_REASON_CODE: &str =
    "service_api_message_recipient_did_invalid";
const SERVICE_API_RELAY_DID_INVALID_REASON_CODE: &str = "service_api_relay_did_invalid";
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
            path: "/v1/agents/search",
            body: "{\"capability\":\"code\"}",
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
            method: "POST",
            path: "/v1/agents/register",
            body: "{\"agent_type\":\"assistant\",\"model_family\":\"gpt-5\",\"capabilities\":[\"text\"]}",
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
            path: "/v1/agents/kamn:did:agent:matrix/balance",
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
        ("POST", "/v1/messages/relay") => "messages:write",
        ("POST", "/v1/channels/create") => "channels:write",
        ("POST", "/v1/agents/search") => "agents:read",
        ("POST", "/v1/agents/register") => "agents:write",
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

fn unique_service_api_test_state_file_path() -> String {
    let entropy = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock should be monotonic")
        .as_nanos();
    let mut path = std::env::temp_dir();
    path.push(format!(
        "kamn-node-service-api-state-test-{}-{entropy}.json",
        std::process::id()
    ));
    path.to_string_lossy().to_string()
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
    // Use an isolated state file per test run to prevent replay nonce watermark bleed
    // between retry attempts in CI.
    let state_file = unique_service_api_test_state_file_path();
    ServiceApiTestEnvGuards {
        _env_lock: env_lock,
        _tls_mode_guard: EnvVarGuard::set("KAMN_SERVICE_API_TLS_MODE", None),
        _tls_cert_guard: EnvVarGuard::set("KAMN_SERVICE_API_TLS_CERT_FILE", None),
        _tls_key_guard: EnvVarGuard::set("KAMN_SERVICE_API_TLS_KEY_FILE", None),
        _auth_public_key_guard: EnvVarGuard::set(
            "KAMN_SERVICE_API_AUTH_PUBLIC_KEY_HEX",
            Some(auth_public_key_hex.as_str()),
        ),
        _state_file_guard: EnvVarGuard::set(
            "KAMN_SERVICE_API_STATE_FILE",
            Some(state_file.as_str()),
        ),
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
    assert!(
        task_complete_response
            .body
            .contains("\"state\":\"completed\"")
    );

    let escrow_fund_response = render_service_api_endpoint_response(
        &snapshot,
        "POST",
        "/v1/escrow/fund",
        "{\"task_id\":\"task-1\",\"amount\":100}",
    );
    assert_eq!(escrow_fund_response.status_code, 200);
    assert!(
        escrow_fund_response
            .body
            .contains("\"escrow_id\":\"escrow-local-")
    );
    assert!(escrow_fund_response.body.contains("\"state\":\"funded\""));

    let escrow_release_response = render_service_api_endpoint_response(
        &snapshot,
        "POST",
        "/v1/escrow/escrow-1/release",
        "{}",
    );
    assert_eq!(escrow_release_response.status_code, 200);
    assert!(
        escrow_release_response
            .body
            .contains("\"state\":\"released\"")
    );

    let content_register_response = render_service_api_endpoint_response(
        &snapshot,
        "POST",
        "/v1/content/register",
        "{\"content\":\"hello\"}",
    );
    assert_eq!(content_register_response.status_code, 201);
    assert!(
        content_register_response
            .body
            .contains("\"content_id\":\"content-local-")
    );
    assert!(
        content_register_response
            .body
            .contains("\"retention_class\":\"standard\"")
    );

    let content_expire_response = render_service_api_endpoint_response(
        &snapshot,
        "POST",
        "/v1/content/content-1/expire",
        "{}",
    );
    assert_eq!(content_expire_response.status_code, 200);
    assert!(
        content_expire_response
            .body
            .contains("\"lifecycle_state\":\"expired\"")
    );

    let content_tombstone_response = render_service_api_endpoint_response(
        &snapshot,
        "POST",
        "/v1/content/content-1/tombstone",
        "{}",
    );
    assert_eq!(content_tombstone_response.status_code, 200);
    assert!(
        content_tombstone_response
            .body
            .contains("\"redaction_status\":\"redacted\"")
    );

    let content_query_response =
        render_service_api_endpoint_response(&snapshot, "GET", "/v1/content/content-1", "");
    assert_eq!(content_query_response.status_code, 200);
    assert!(
        content_query_response
            .body
            .contains("\"lifecycle_state\":\"tombstoned\"")
    );

    let bridge_submit_response = render_service_api_endpoint_response(
        &snapshot,
        "POST",
        "/v1/bridge/submit",
        "{\"source_message_id\":\"msg-1\",\"target_network\":\"testnet\"}",
    );
    assert_eq!(bridge_submit_response.status_code, 202);
    assert!(
        bridge_submit_response
            .body
            .contains("\"bridge_id\":\"bridge-local-")
    );
    assert!(
        bridge_submit_response
            .body
            .contains("\"bridge_status\":\"submitted\"")
    );

    let bridge_forward_response = render_service_api_endpoint_response(
        &snapshot,
        "POST",
        "/v1/bridge/bridge-1/forward",
        "{}",
    );
    assert_eq!(bridge_forward_response.status_code, 200);
    assert!(
        bridge_forward_response
            .body
            .contains("\"bridge_status\":\"forwarded\"")
    );
    assert!(
        bridge_forward_response
            .body
            .contains("\"target_message_id\":\"msg-bridge-target-bridge-1\"")
    );

    let bridge_query_response =
        render_service_api_endpoint_response(&snapshot, "GET", "/v1/bridge/bridge-1", "");
    assert_eq!(bridge_query_response.status_code, 200);
    assert!(
        bridge_query_response
            .body
            .contains("\"forward_tx_hash\":\"sha256:bridge-forwarded-bridge-1\"")
    );

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
    assert!(
        metrics_response
            .body
            .contains("kamn_service_api_observability_source{source=\"unknown\"} 1")
    );
    assert!(
        metrics_response
            .body
            .contains("kamn_service_api_observability_health{health=\"unknown\"} 0")
    );
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
    let agent_json: Value =
        serde_json::from_str(agent.body.as_str()).expect("agent payload should parse as json");
    assert_eq!(agent_json["agent_type"], "service-agent");
    assert_eq!(agent_json["model_family"], "service-api");
    assert_eq!(
        agent_json["capabilities"],
        serde_json::json!(["profile:read"])
    );
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
    assert!(
        websocket_required_payload
            .message
            .contains("websocket upgrade required")
    );

    let method_not_allowed =
        render_service_api_endpoint_response(&snapshot, "DELETE", "/v1/messages/send", "");
    assert_eq!(method_not_allowed.status_code, 405);
    let method_not_allowed_payload = parse_error_envelope(method_not_allowed.body.as_str());
    assert_eq!(method_not_allowed_payload.error, "method-not-allowed");
    assert_eq!(
        method_not_allowed_payload.reason_code,
        "service_api_method_not_allowed"
    );
    assert!(
        method_not_allowed_payload
            .message
            .contains("method not allowed")
    );

    let not_found = render_service_api_endpoint_response(&snapshot, "GET", "/v1/nope", "");
    assert_eq!(not_found.status_code, 404);
    let not_found_payload = parse_error_envelope(not_found.body.as_str());
    assert_eq!(not_found_payload.error, "not-found");
    assert_eq!(not_found_payload.reason_code, "service_api_route_not_found");
    assert!(not_found_payload.message.contains("not found"));

    let baseline_config = ServiceApiEndpointConfig {
        bind_addr: "127.0.0.1:0".to_owned(),
        max_requests: 1,
        idle_timeout_ms: 1,
        body_limit_bytes: 1,
        concurrency_limit: 1,
        rate_limit_per_second: 1,
    };

    let mut max_requests_zero = baseline_config.clone();
    max_requests_zero.max_requests = 0;
    let max_requests_error = serve_service_api_endpoint(&max_requests_zero, &snapshot)
        .expect_err("max_requests=0 must fail closed");
    assert_eq!(
        max_requests_error,
        "service api max requests must be greater than zero"
    );

    let mut idle_timeout_zero = baseline_config.clone();
    idle_timeout_zero.idle_timeout_ms = 0;
    let idle_timeout_error = serve_service_api_endpoint(&idle_timeout_zero, &snapshot)
        .expect_err("idle_timeout_ms=0 must fail closed");
    assert_eq!(
        idle_timeout_error,
        "service api idle timeout must be greater than zero"
    );

    let mut body_limit_zero = baseline_config.clone();
    body_limit_zero.body_limit_bytes = 0;
    let body_limit_error = serve_service_api_endpoint(&body_limit_zero, &snapshot)
        .expect_err("body_limit_bytes=0 must fail closed");
    assert_eq!(
        body_limit_error,
        "service api body limit bytes must be greater than zero"
    );

    let mut concurrency_limit_zero = baseline_config.clone();
    concurrency_limit_zero.concurrency_limit = 0;
    let concurrency_limit_error = serve_service_api_endpoint(&concurrency_limit_zero, &snapshot)
        .expect_err("concurrency_limit=0 must fail closed");
    assert_eq!(
        concurrency_limit_error,
        "service api concurrency limit must be greater than zero"
    );

    let mut rate_limit_zero = baseline_config;
    rate_limit_zero.rate_limit_per_second = 0;
    let rate_limit_error = serve_service_api_endpoint(&rate_limit_zero, &snapshot)
        .expect_err("rate_limit_per_second=0 must fail closed");
    assert_eq!(
        rate_limit_error,
        "service api rate limit per second must be greater than zero"
    );

    let relay_entry = ServiceApiRelaySpoolEntry {
        message_id: "msg-test-relay".to_owned(),
        sender_did: Some("kamn:did:agent:sender".to_owned()),
        recipient_did: "kamn:did:agent:recipient".to_owned(),
        body: "{\"message\":\"relay\"}".to_owned(),
        queued_at_unix: 1,
    };
    let relayed = upsert_service_api_relayed_message_from_daemon(None, &relay_entry)
        .expect("daemon relay upsert should succeed without a state file");
    assert_eq!(relayed.message_id, "msg-test-relay");
    assert_eq!(relayed.status, "relayed");
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
    assert!(
        metrics_response
            .contains("kamn_service_api_observability_source{source=\"service-api-runtime\"} 1")
    );

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
        // Keep budget above readiness + concurrent lane probes to avoid racey shutdowns.
        max_requests: 8,
        idle_timeout_ms: 2_000,
        body_limit_bytes: DEFAULT_SERVICE_API_BODY_LIMIT_BYTES,
        concurrency_limit: DEFAULT_SERVICE_API_CONCURRENCY_LIMIT,
        rate_limit_per_second: DEFAULT_SERVICE_API_RATE_LIMIT_PER_SECOND,
    };
    let server_snapshot = snapshot.clone();
    let server =
        thread::spawn(move || serve_service_api_endpoint(&endpoint_config, &server_snapshot));
    wait_for_endpoint_ready(bind_addr.as_str());

    let sender_did_one = "kamn:did:agent:async-http-client-1";
    let sender_did_two = "kamn:did:agent:async-http-client-2";
    let body_one = "{\"message\":\"async-route-1\"}".to_owned();
    let body_two = "{\"message\":\"async-route-2\"}".to_owned();
    let state_hash = format!(
        "service-api:{}:{}",
        snapshot.chain_id.as_str(),
        snapshot.chain_version.as_str()
    );
    let signature_one = service_api_request_signature_for_fields(
        sender_did_one,
        900,
        state_hash.as_str(),
        body_one.as_str(),
    );
    let signature_two = service_api_request_signature_for_fields(
        sender_did_two,
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
                        ("X-KAMN-Sender-DID", sender_did_one),
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
                        ("X-KAMN-Sender-DID", sender_did_two),
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
    assert!(
        metrics_response
            .contains("kamn_service_api_observability_source{source=\"service-api-runtime\"} 1")
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
fn regression_service_api_endpoint_rejects_disabled_tls_for_non_loopback_api_runtime_path() {
    let _env = acquire_service_api_test_env();
    let _tls_mode = EnvVarGuard::set("KAMN_SERVICE_API_TLS_MODE", Some("disabled"));

    let parsed = parse_args(vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
        "--runtime-mode".to_owned(),
        "api".to_owned(),
        "--api-bind".to_owned(),
        "127.0.0.1:34102".to_owned(),
    ])
    .expect("api args should parse");
    let report = execute(parsed).expect("api execution should succeed");
    let snapshot = build_service_api_snapshot(&report);

    let endpoint_config = ServiceApiEndpointConfig {
        bind_addr: "0.0.0.0:34103".to_owned(),
        max_requests: 1,
        idle_timeout_ms: 2_000,
        body_limit_bytes: DEFAULT_SERVICE_API_BODY_LIMIT_BYTES,
        concurrency_limit: DEFAULT_SERVICE_API_CONCURRENCY_LIMIT,
        rate_limit_per_second: DEFAULT_SERVICE_API_RATE_LIMIT_PER_SECOND,
    };

    let error = serve_service_api_endpoint(&endpoint_config, &snapshot)
        .expect_err("disabled tls must fail closed for non-loopback api runtime path");
    assert!(
        error.contains("service api tls disabled is forbidden"),
        "unexpected disabled tls policy marker: {error}"
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
    assert!(
        second_response
            .contains("kamn_service_api_observability_source{source=\"service-api-runtime\"} 1")
    );

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
    assert!(
        metrics_response
            .body
            .contains("kamn_service_api_observability_source{source=\"daemon\"} 1")
    );
    assert!(
        metrics_response
            .body
            .contains("kamn_service_api_observability_health{health=\"healthy\"} 1")
    );
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
fn integration_service_api_endpoint_accepts_case_variant_self_certifying_sender_did_binding() {
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

    let signer_public_key_hex =
        service_auth_public_key_hex_from_private_key_hex(TEST_SERVICE_API_AUTH_PRIVATE_KEY_HEX)
            .expect("service-auth public key should derive");
    let sender_did = format!("kamn:did:agent:pkh-{signer_public_key_hex}");
    let message_body = "{\"message\":\"hello\"}";
    let state_hash = format!(
        "service-api:{}:{}",
        snapshot.chain_id.as_str(),
        snapshot.chain_version.as_str()
    );
    let signature = service_api_request_signature_for_fields(
        sender_did.as_str(),
        1,
        state_hash.as_str(),
        message_body,
    );
    let response = send_http_request_with_headers(
        bind_addr.as_str(),
        "POST",
        "/v1/messages/send",
        message_body,
        &[
            ("X-KAMN-Sender-DID", sender_did.as_str()),
            ("X-KAMN-Request-Nonce", "1"),
            ("X-KAMN-Request-Signature", signature.as_str()),
            (
                "x-kamn-signer-public-key",
                signer_public_key_hex.to_uppercase().as_str(),
            ),
        ],
    );

    assert!(response.contains("HTTP/1.1 202 Accepted"));

    let server_result = server.join().expect("endpoint thread should complete");
    assert!(
        server_result.is_ok(),
        "service api endpoint should stop cleanly after case-variant sender DID auth flow"
    );
}

#[test]
fn regression_service_api_endpoint_rejects_legacy_sender_binding_without_signer_public_key_header()
{
    let _env = acquire_service_api_test_env();
    let parsed = parse_args(vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
        "--runtime-mode".to_owned(),
        "api".to_owned(),
        "--api-bind".to_owned(),
        "127.0.0.1:34076".to_owned(),
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

    let sender_did = "kamn:did:agent:legacy-auth-binding";
    let message_body = r#"{"recipient_did":"kamn:did:agent:legacy-auth-target","message":"hello"}"#;
    let signature =
        service_api_request_signature_for_fields(sender_did, 1, state_hash.as_str(), message_body);
    let response = send_http_request_with_headers(
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

    assert!(response.contains("HTTP/1.1 401 Unauthorized"));
    let error_payload = parse_error_envelope_from_http_response(response.as_str());
    assert_eq!(
        error_payload.reason_code,
        "service_api_auth_signature_verification_failed"
    );
    assert!(
        error_payload.message.contains("x-kamn-signer-public-key"),
        "missing signer header rejection should explain the explicit signer binding contract"
    );

    let server_result = server.join().expect("endpoint thread should complete");
    assert!(
        server_result.is_ok(),
        "service api endpoint should stop cleanly after rejecting legacy auth fallback"
    );
}

#[test]
fn unit_service_api_route_authz_matrix_matches_protected_and_public_paths() {
    assert_eq!(
        SERVICE_API_AUTH_REASON_TAXONOMY_VERSION,
        "kamn.runtime.service-api-auth-reason-taxonomy.v1"
    );
    assert!(
        SERVICE_API_AUTH_REASON_CODES_CSV.contains(SERVICE_API_AUTH_MISSING_HEADER_REASON_CODE)
    );
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
fn integration_service_api_endpoint_persists_message_state_across_restart_without_explicit_state_file_env()
 {
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
    assert!(
        evidence["m0_content_hash"]
            .as_str()
            .is_some_and(|value| value.starts_with("sha256:"))
    );
    assert!(
        evidence["m1_merkle_root"]
            .as_str()
            .is_some_and(|value| value.starts_with("sha256:"))
    );
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
fn integration_service_api_endpoint_persists_channel_creation_state_across_restart() {
    let _env = acquire_service_api_test_env();
    let state_file = std::env::temp_dir().join(format!(
        "kamn-node-service-api-channel-create-restart-state-{}-{}.json",
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
        "127.0.0.1:34117".to_owned(),
    ])
    .expect("api args should parse");
    let report = execute(parsed).expect("api execution should succeed");
    let snapshot = build_service_api_snapshot(&report);
    let state_hash = format!(
        "service-api:{}:{}",
        snapshot.chain_id.as_str(),
        snapshot.chain_version.as_str()
    );
    let caller_did = "kamn:did:agent:test-client-channel-create-restart";

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

    let create_payload = r#"{"name":"channel-restart-contract"}"#;
    let create_signature = service_api_request_signature_for_fields(
        caller_did,
        111,
        state_hash.as_str(),
        create_payload,
    );
    let create_response = send_http_request_with_headers(
        bind_addr.as_str(),
        "POST",
        "/v1/channels/create",
        create_payload,
        &[
            ("X-KAMN-Sender-DID", caller_did),
            ("X-KAMN-Request-Nonce", "111"),
            ("X-KAMN-Request-Signature", create_signature.as_str()),
        ],
    );
    assert!(create_response.contains("HTTP/1.1 201 Created"));
    let created_channel: ServiceApiChannelCreateBody =
        parse_service_api_payload(extract_http_response_body(create_response.as_str()))
            .expect("channel create payload should deserialize");
    assert_eq!(created_channel.status, "created");

    let server_result = server.join().expect("endpoint thread should complete");
    assert!(
        server_result.is_ok(),
        "service api endpoint should stop cleanly after channel create phase"
    );

    let phase_one_state_payload = fs::read_to_string(state_file.as_path())
        .expect("channel state file should remain readable after create phase");
    let phase_one_state_json: Value =
        serde_json::from_str(phase_one_state_payload.as_str()).expect("state payload should parse");
    let persisted_channel = phase_one_state_json["channel_messages"]
        .get(created_channel.channel_id.as_str())
        .and_then(serde_json::Value::as_array);
    assert!(
        persisted_channel.is_some(),
        "channel create should persist channel id into durable channel_messages map"
    );
    assert_eq!(
        persisted_channel.map(std::vec::Vec::len),
        Some(0),
        "newly created channel should start with empty message list"
    );

    let restart_report = execute(
        parse_args(vec![
            "kamn-node".to_owned(),
            "--role".to_owned(),
            "processor".to_owned(),
            "--runtime-mode".to_owned(),
            "api".to_owned(),
            "--api-bind".to_owned(),
            "127.0.0.1:34118".to_owned(),
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

    let list_path = format!("/v1/channels/{}/messages", created_channel.channel_id);
    let list_signature =
        service_api_request_signature_for_fields(caller_did, 112, restart_state_hash.as_str(), "");
    let list_response = send_http_request_with_headers(
        restart_bind_addr.as_str(),
        "GET",
        list_path.as_str(),
        "",
        &[
            ("X-KAMN-Sender-DID", caller_did),
            ("X-KAMN-Request-Nonce", "112"),
            ("X-KAMN-Request-Signature", list_signature.as_str()),
        ],
    );
    assert!(list_response.contains("HTTP/1.1 200 OK"));
    let listed_channel: ServiceApiChannelMessagesBody =
        parse_service_api_payload(extract_http_response_body(list_response.as_str()))
            .expect("channel list payload should deserialize");
    assert_eq!(listed_channel.channel_id, created_channel.channel_id);
    assert!(
        listed_channel.messages.is_empty(),
        "created channel should still have no messages after restart"
    );

    let restart_server_result = restart_server
        .join()
        .expect("restart endpoint thread should complete");
    assert!(
        restart_server_result.is_ok(),
        "service api endpoint should stop cleanly after channel restart query"
    );

    let _ = fs::remove_file(state_file);
}

#[test]
fn integration_service_api_endpoint_persists_agent_profile_query_state_across_restart() {
    let _env = acquire_service_api_test_env();
    let state_file = std::env::temp_dir().join(format!(
        "kamn-node-service-api-agent-profile-restart-state-{}-{}.json",
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
        "127.0.0.1:34119".to_owned(),
    ])
    .expect("api args should parse");
    let report = execute(parsed).expect("api execution should succeed");
    let snapshot = build_service_api_snapshot(&report);
    let state_hash = format!(
        "service-api:{}:{}",
        snapshot.chain_id.as_str(),
        snapshot.chain_version.as_str()
    );
    let caller_did = "kamn:did:agent:test-client-agent-profile-restart";
    let target_agent_did = "kamn:did:agent:profile-restart-target";

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

    let query_path = format!("/v1/agents/{target_agent_did}");
    let query_signature =
        service_api_request_signature_for_fields(caller_did, 121, state_hash.as_str(), "");
    let query_response = send_http_request_with_headers(
        bind_addr.as_str(),
        "GET",
        query_path.as_str(),
        "",
        &[
            ("X-KAMN-Sender-DID", caller_did),
            ("X-KAMN-Request-Nonce", "121"),
            ("X-KAMN-Request-Signature", query_signature.as_str()),
        ],
    );
    assert!(query_response.contains("HTTP/1.1 200 OK"));
    let first_profile: ServiceApiAgentGetBody =
        parse_service_api_payload(extract_http_response_body(query_response.as_str()))
            .expect("agent query payload should deserialize");
    assert_eq!(first_profile.did, target_agent_did);
    assert_eq!(first_profile.reputation_score, 500);

    let server_result = server.join().expect("endpoint thread should complete");
    assert!(
        server_result.is_ok(),
        "service api endpoint should stop cleanly after first agent query phase"
    );

    let phase_one_state_payload = fs::read_to_string(state_file.as_path())
        .expect("agent profile state file should remain readable after first phase");
    let phase_one_state_json: Value =
        serde_json::from_str(phase_one_state_payload.as_str()).expect("state payload should parse");
    assert_eq!(
        phase_one_state_json["agents"][target_agent_did]["did"],
        target_agent_did
    );
    assert_eq!(
        phase_one_state_json["agents"][target_agent_did]["reputation_score"],
        500
    );

    let restart_report = execute(
        parse_args(vec![
            "kamn-node".to_owned(),
            "--role".to_owned(),
            "processor".to_owned(),
            "--runtime-mode".to_owned(),
            "api".to_owned(),
            "--api-bind".to_owned(),
            "127.0.0.1:34120".to_owned(),
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

    let restart_query_signature =
        service_api_request_signature_for_fields(caller_did, 122, restart_state_hash.as_str(), "");
    let restart_query_response = send_http_request_with_headers(
        restart_bind_addr.as_str(),
        "GET",
        query_path.as_str(),
        "",
        &[
            ("X-KAMN-Sender-DID", caller_did),
            ("X-KAMN-Request-Nonce", "122"),
            ("X-KAMN-Request-Signature", restart_query_signature.as_str()),
        ],
    );
    assert!(restart_query_response.contains("HTTP/1.1 200 OK"));
    let restart_profile: ServiceApiAgentGetBody =
        parse_service_api_payload(extract_http_response_body(restart_query_response.as_str()))
            .expect("restart agent query payload should deserialize");
    assert_eq!(restart_profile.did, target_agent_did);
    assert_eq!(restart_profile.reputation_score, 500);

    let restart_server_result = restart_server
        .join()
        .expect("restart endpoint thread should complete");
    assert!(
        restart_server_result.is_ok(),
        "service api endpoint should stop cleanly after restart agent query"
    );

    let _ = fs::remove_file(state_file);
}

#[test]
fn integration_service_api_endpoint_registers_agent_metadata_idempotently_and_conflicts_on_mismatch()
 {
    let _env = acquire_service_api_test_env();
    let parsed = parse_args(vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
        "--runtime-mode".to_owned(),
        "api".to_owned(),
        "--api-bind".to_owned(),
        "127.0.0.1:34121".to_owned(),
    ])
    .expect("api args should parse");
    let report = execute(parsed).expect("api execution should succeed");
    let snapshot = build_service_api_snapshot(&report);
    let state_hash = format!(
        "service-api:{}:{}",
        snapshot.chain_id.as_str(),
        snapshot.chain_version.as_str()
    );
    let caller_did = "kamn:did:agent:register-agent-profile";
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

    let registration_body =
        r#"{"agent_type":"assistant","model_family":"gpt-5","capabilities":["text","code"]}"#;
    let registration_signature = service_api_request_signature_for_fields(
        caller_did,
        201,
        state_hash.as_str(),
        registration_body,
    );
    let registration_response = send_http_request_with_headers(
        bind_addr.as_str(),
        "POST",
        "/v1/agents/register",
        registration_body,
        &[
            ("X-KAMN-Sender-DID", caller_did),
            ("X-KAMN-Request-Nonce", "201"),
            ("X-KAMN-Request-Signature", registration_signature.as_str()),
        ],
    );
    assert!(registration_response.contains("HTTP/1.1 201 Created"));
    let registration_payload: ServiceApiAgentGetBody =
        parse_service_api_payload(extract_http_response_body(registration_response.as_str()))
            .expect("registration payload should deserialize");
    assert_eq!(registration_payload.did, caller_did);
    assert_eq!(registration_payload.agent_type, "assistant");
    assert_eq!(registration_payload.model_family, "gpt-5");
    assert_eq!(
        registration_payload.capabilities,
        vec!["text".to_owned(), "code".to_owned()]
    );

    let duplicate_signature = service_api_request_signature_for_fields(
        caller_did,
        202,
        state_hash.as_str(),
        registration_body,
    );
    let duplicate_response = send_http_request_with_headers(
        bind_addr.as_str(),
        "POST",
        "/v1/agents/register",
        registration_body,
        &[
            ("X-KAMN-Sender-DID", caller_did),
            ("X-KAMN-Request-Nonce", "202"),
            ("X-KAMN-Request-Signature", duplicate_signature.as_str()),
        ],
    );
    assert!(duplicate_response.contains("HTTP/1.1 201 Created"));

    let mismatched_body =
        r#"{"agent_type":"assistant","model_family":"gpt-5o","capabilities":["text"]}"#;
    let mismatch_signature = service_api_request_signature_for_fields(
        caller_did,
        203,
        state_hash.as_str(),
        mismatched_body,
    );
    let mismatch_response = send_http_request_with_headers(
        bind_addr.as_str(),
        "POST",
        "/v1/agents/register",
        mismatched_body,
        &[
            ("X-KAMN-Sender-DID", caller_did),
            ("X-KAMN-Request-Nonce", "203"),
            ("X-KAMN-Request-Signature", mismatch_signature.as_str()),
        ],
    );
    assert!(mismatch_response.contains("HTTP/1.1 409 Conflict"));
    let mismatch_error =
        parse_error_envelope(extract_http_response_body(mismatch_response.as_str()));
    assert_eq!(
        mismatch_error.reason_code,
        "service_api_agent_registration_conflict"
    );

    let reader_did = "kamn:did:agent:register-agent-profile-reader";
    let query_signature =
        service_api_request_signature_for_fields(reader_did, 301, state_hash.as_str(), "");
    let query_response = send_http_request_with_headers(
        bind_addr.as_str(),
        "GET",
        format!("/v1/agents/{caller_did}").as_str(),
        "",
        &[
            ("X-KAMN-Sender-DID", reader_did),
            ("X-KAMN-Request-Nonce", "301"),
            ("X-KAMN-Request-Signature", query_signature.as_str()),
        ],
    );
    assert!(query_response.contains("HTTP/1.1 200 OK"));
    let query_payload: ServiceApiAgentGetBody =
        parse_service_api_payload(extract_http_response_body(query_response.as_str()))
            .expect("agent query payload should deserialize");
    assert_eq!(query_payload.did, caller_did);
    assert_eq!(query_payload.agent_type, "assistant");
    assert_eq!(query_payload.model_family, "gpt-5");
    assert_eq!(
        query_payload.capabilities,
        vec!["text".to_owned(), "code".to_owned()]
    );

    let server_result = server.join().expect("endpoint thread should complete");
    assert!(
        server_result.is_ok(),
        "service api endpoint should stop cleanly after agent registration contract"
    );
}

#[test]
fn integration_service_api_endpoint_searches_registered_agent_metadata() {
    let _env = acquire_service_api_test_env();
    let state_file = std::env::temp_dir().join(format!(
        "kamn-node-service-api-agent-search-{}-{}.json",
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
        "127.0.0.1:34121".to_owned(),
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

    for (nonce, caller_did, registration_body) in [
        (
            401_u64,
            "kamn:did:agent:search-alpha",
            r#"{"agent_type":"assistant","model_family":"gpt-5","capabilities":["text","code"]}"#,
        ),
        (
            402_u64,
            "kamn:did:agent:search-beta",
            r#"{"agent_type":"assistant","model_family":"gpt-4.1","capabilities":["text"]}"#,
        ),
    ] {
        let signature = service_api_request_signature_for_fields(
            caller_did,
            nonce,
            state_hash.as_str(),
            registration_body,
        );
        let response = send_http_request_with_headers(
            bind_addr.as_str(),
            "POST",
            "/v1/agents/register",
            registration_body,
            &[
                ("X-KAMN-Sender-DID", caller_did),
                ("X-KAMN-Request-Nonce", nonce.to_string().as_str()),
                ("X-KAMN-Request-Signature", signature.as_str()),
            ],
        );
        assert!(response.contains("HTTP/1.1 201 Created"));
    }

    let search_body = r#"{"capability":"code","model_family":"gpt-5"}"#;
    let reader_did = "kamn:did:agent:search-reader";
    let search_signature =
        service_api_request_signature_for_fields(reader_did, 403, state_hash.as_str(), search_body);
    let search_response = send_http_request_with_headers(
        bind_addr.as_str(),
        "POST",
        "/v1/agents/search",
        search_body,
        &[
            ("X-KAMN-Sender-DID", reader_did),
            ("X-KAMN-Request-Nonce", "403"),
            ("X-KAMN-Request-Signature", search_signature.as_str()),
            ("X-KAMN-Authz-Scope", "agents:read"),
        ],
    );
    assert!(search_response.contains("HTTP/1.1 200 OK"));
    let search_payload: Vec<ServiceApiAgentGetBody> =
        parse_service_api_payload(extract_http_response_body(search_response.as_str()))
            .expect("search payload should deserialize");
    assert_eq!(search_payload.len(), 1);
    assert_eq!(search_payload[0].did, "kamn:did:agent:search-alpha");
    assert_eq!(search_payload[0].agent_type, "assistant");
    assert_eq!(search_payload[0].model_family, "gpt-5");
    assert_eq!(
        search_payload[0].capabilities,
        vec!["text".to_owned(), "code".to_owned()]
    );

    let server_result = server.join().expect("endpoint thread should complete");
    assert!(
        server_result.is_ok(),
        "service api endpoint should stop cleanly after agent search contract"
    );
}

#[test]
fn integration_service_api_endpoint_rejects_invalid_agent_search_payload() {
    let _env = acquire_service_api_test_env();
    let parsed = parse_args(vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
        "--runtime-mode".to_owned(),
        "api".to_owned(),
        "--api-bind".to_owned(),
        "127.0.0.1:34121".to_owned(),
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

    let caller_did = "kamn:did:agent:search-invalid";
    let search_body = r#"{"capability":"   "}"#;
    let search_signature =
        service_api_request_signature_for_fields(caller_did, 404, state_hash.as_str(), search_body);
    let response = send_http_request_with_headers(
        bind_addr.as_str(),
        "POST",
        "/v1/agents/search",
        search_body,
        &[
            ("X-KAMN-Sender-DID", caller_did),
            ("X-KAMN-Request-Nonce", "404"),
            ("X-KAMN-Request-Signature", search_signature.as_str()),
            ("X-KAMN-Authz-Scope", "agents:read"),
        ],
    );

    assert!(response.contains("HTTP/1.1 400 Bad Request"));
    let payload: ServiceApiErrorBody =
        parse_service_api_payload(extract_http_response_body(response.as_str()))
            .expect("error payload should deserialize");
    assert_eq!(payload.error, "bad-request");
    assert_eq!(
        payload.reason_code,
        "service_api_agent_search_payload_invalid"
    );
    assert!(
        payload
            .message
            .contains("agent search payload capability must not be empty when provided")
    );

    let server_result = server.join().expect("endpoint thread should complete");
    assert!(
        server_result.is_ok(),
        "service api endpoint should stop cleanly after invalid agent search payload contract"
    );
}

#[test]
fn integration_service_api_endpoint_rejects_legacy_agent_profile_path_dids() {
    let _env = acquire_service_api_test_env();
    let state_file = std::env::temp_dir().join(format!(
        "kamn-node-service-api-agent-profile-legacy-path-{}-{}.json",
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
        "127.0.0.1:34121".to_owned(),
    ])
    .expect("api args should parse");
    let report = execute(parsed).expect("api execution should succeed");
    let snapshot = build_service_api_snapshot(&report);
    let state_hash = format!(
        "service-api:{}:{}",
        snapshot.chain_id.as_str(),
        snapshot.chain_version.as_str()
    );
    let caller_did = "kamn:did:agent:test-client-agent-profile-legacy-path";
    let legacy_target_did = "did:kamn:agent:legacy-alpha";

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

    let query_path = format!("/v1/agents/{legacy_target_did}");
    let query_signature =
        service_api_request_signature_for_fields(caller_did, 121, state_hash.as_str(), "");
    let query_response = send_http_request_with_headers(
        bind_addr.as_str(),
        "GET",
        query_path.as_str(),
        "",
        &[
            ("X-KAMN-Sender-DID", caller_did),
            ("X-KAMN-Request-Nonce", "121"),
            ("X-KAMN-Request-Signature", query_signature.as_str()),
        ],
    );
    assert!(query_response.contains("HTTP/1.1 400 Bad Request"));
    let error_payload = parse_error_envelope_from_http_response(query_response.as_str());
    assert_eq!(
        error_payload.reason_code,
        SERVICE_API_AGENT_DID_PATH_INVALID_REASON_CODE
    );
    assert!(
        error_payload.message.contains("invalid agent did path"),
        "legacy path rejection should explain the invalid did boundary"
    );

    let server_result = server.join().expect("endpoint thread should complete");
    assert!(
        server_result.is_ok(),
        "service api endpoint should stop cleanly after legacy path rejection"
    );

    if state_file.exists() {
        let state_payload = fs::read_to_string(state_file.as_path())
            .expect("state file should remain readable when present");
        let state_json: Value =
            serde_json::from_str(state_payload.as_str()).expect("state payload should parse");
        assert!(
            state_json["agents"].get(legacy_target_did).is_none(),
            "legacy agent did should not be persisted"
        );
    }

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
fn integration_service_api_endpoint_persists_bridge_state_across_restart() {
    let _env = acquire_service_api_test_env();
    let state_file = std::env::temp_dir().join(format!(
        "kamn-node-service-api-bridge-restart-state-{}-{}.json",
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
        "127.0.0.1:34115".to_owned(),
    ])
    .expect("api args should parse");
    let report = execute(parsed).expect("api execution should succeed");
    let snapshot = build_service_api_snapshot(&report);
    let state_hash = format!(
        "service-api:{}:{}",
        snapshot.chain_id.as_str(),
        snapshot.chain_version.as_str()
    );
    let caller_did = "kamn:did:agent:test-client-bridge-restart";

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

    let submit_payload = r#"{"source_message_id":"msg-bridge-restart-source"}"#;
    let submit_signature = service_api_request_signature_for_fields(
        caller_did,
        101,
        state_hash.as_str(),
        submit_payload,
    );
    let submit_response = send_http_request_with_headers(
        bind_addr.as_str(),
        "POST",
        "/v1/bridge/submit",
        submit_payload,
        &[
            ("X-KAMN-Sender-DID", caller_did),
            ("X-KAMN-Request-Nonce", "101"),
            ("X-KAMN-Request-Signature", submit_signature.as_str()),
        ],
    );
    assert!(submit_response.contains("HTTP/1.1 202 Accepted"));
    let submit_json: Value =
        parse_service_api_payload(extract_http_response_body(submit_response.as_str()))
            .expect("bridge submit payload should deserialize");
    let bridge_id = submit_json["bridge_id"]
        .as_str()
        .expect("bridge id should be string")
        .to_owned();
    assert_eq!(submit_json["bridge_status"], "submitted");

    let server_result = server.join().expect("endpoint thread should complete");
    assert!(
        server_result.is_ok(),
        "service api endpoint should stop cleanly after bridge submit phase"
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
    let restart_endpoint_config = ServiceApiEndpointConfig {
        bind_addr: restart_bind_addr.clone(),
        max_requests: 3,
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

    let forward_path = format!("/v1/bridge/{bridge_id}/forward");
    let forward_signature =
        service_api_request_signature_for_fields(caller_did, 102, restart_state_hash.as_str(), "");
    let forward_response = send_http_request_with_headers(
        restart_bind_addr.as_str(),
        "POST",
        forward_path.as_str(),
        "",
        &[
            ("X-KAMN-Sender-DID", caller_did),
            ("X-KAMN-Request-Nonce", "102"),
            ("X-KAMN-Request-Signature", forward_signature.as_str()),
        ],
    );
    assert!(forward_response.contains("HTTP/1.1 200 OK"));
    let forward_json: Value =
        parse_service_api_payload(extract_http_response_body(forward_response.as_str()))
            .expect("bridge forward payload should deserialize");
    assert_eq!(forward_json["bridge_id"], bridge_id);
    assert_eq!(forward_json["bridge_status"], "forwarded");
    assert_eq!(
        forward_json["target_message_id"],
        format!("msg-bridge-target-{bridge_id}")
    );

    let query_path = format!("/v1/bridge/{bridge_id}");
    let query_signature =
        service_api_request_signature_for_fields(caller_did, 103, restart_state_hash.as_str(), "");
    let query_response = send_http_request_with_headers(
        restart_bind_addr.as_str(),
        "GET",
        query_path.as_str(),
        "",
        &[
            ("X-KAMN-Sender-DID", caller_did),
            ("X-KAMN-Request-Nonce", "103"),
            ("X-KAMN-Request-Signature", query_signature.as_str()),
        ],
    );
    assert!(query_response.contains("HTTP/1.1 200 OK"));
    let query_json: Value =
        parse_service_api_payload(extract_http_response_body(query_response.as_str()))
            .expect("bridge query payload should deserialize");
    assert_eq!(query_json["bridge_id"], bridge_id);
    assert_eq!(query_json["bridge_status"], "forwarded");
    assert_eq!(
        query_json["target_message_id"],
        format!("msg-bridge-target-{bridge_id}")
    );

    let missing_caller_did = "kamn:did:agent:test-client-bridge-missing";
    let missing_signature = service_api_request_signature_for_fields(
        missing_caller_did,
        104,
        restart_state_hash.as_str(),
        "",
    );
    let missing_response = send_http_request_with_headers(
        restart_bind_addr.as_str(),
        "GET",
        "/v1/bridge/bridge-missing-104",
        "",
        &[
            ("X-KAMN-Sender-DID", missing_caller_did),
            ("X-KAMN-Request-Nonce", "104"),
            ("X-KAMN-Request-Signature", missing_signature.as_str()),
        ],
    );
    assert!(missing_response.contains("HTTP/1.1 404 Not Found"));
    let missing_payload = parse_error_envelope_from_http_response(missing_response.as_str());
    assert_eq!(missing_payload.error, "not-found");
    assert_eq!(missing_payload.reason_code, "service_api_route_not_found");

    let restart_server_result = restart_server
        .join()
        .expect("restart endpoint thread should complete");
    assert!(
        restart_server_result.is_ok(),
        "service api endpoint should stop cleanly after bridge restart phase"
    );

    let state_payload =
        fs::read_to_string(state_file.as_path()).expect("bridge state file should remain readable");
    let state_json: Value =
        serde_json::from_str(state_payload.as_str()).expect("state payload should parse");
    assert_eq!(
        state_json["bridges"][bridge_id.as_str()]["bridge_status"],
        "forwarded"
    );
    assert_eq!(
        state_json["bridges"][bridge_id.as_str()]["target_message_id"],
        format!("msg-bridge-target-{bridge_id}")
    );
    assert_eq!(
        state_json["bridges"][bridge_id.as_str()]["forward_tx_hash"],
        format!("sha256:bridge-forwarded-{bridge_id}")
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

    let relay_listener = TcpListener::bind("127.0.0.1:0")
        .expect("relay receiver listener should bind for daemon forwarding");
    let relay_receiver_addr = relay_listener
        .local_addr()
        .expect("relay receiver listener addr should resolve")
        .to_string();
    let relay_route_map = serde_json::json!({
        recipient_did: relay_receiver_addr.clone(),
    })
    .to_string();
    let _relay_route_guard = EnvVarGuard::set(
        "KAMN_SERVICE_API_RELAY_RECIPIENT_ROUTE_MAP_JSON",
        Some(relay_route_map.as_str()),
    );
    let _daemon_private_key_guard = EnvVarGuard::set(
        "KAMN_SERVICE_API_AUTH_PRIVATE_KEY_HEX",
        Some(TEST_SERVICE_API_AUTH_PRIVATE_KEY_HEX),
    );
    let relay_receiver = thread::spawn(move || {
        let (mut relay_stream, _) = relay_listener
            .accept()
            .expect("relay receiver should accept daemon forwarding connection");
        relay_stream
            .set_read_timeout(Some(Duration::from_secs(2)))
            .expect("relay receiver read timeout should configure");
        let mut request = String::new();
        let mut chunk = [0_u8; 1024];
        loop {
            match relay_stream.read(&mut chunk) {
                Ok(0) => break,
                Ok(read_count) => {
                    request.push_str(
                        std::str::from_utf8(&chunk[..read_count])
                            .expect("relay request should be utf-8"),
                    );
                    if request.contains("\r\n\r\n") {
                        break;
                    }
                }
                Err(error)
                    if matches!(error.kind(), ErrorKind::WouldBlock | ErrorKind::TimedOut) =>
                {
                    break;
                }
                Err(error) => panic!("relay receiver request read should succeed: {error}"),
            }
        }
        relay_stream
            .write_all(b"HTTP/1.1 202 Accepted\r\nContent-Length: 2\r\nConnection: close\r\n\r\n{}")
            .expect("relay receiver response should write");
        request
    });

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
    let relay_forward_request = relay_receiver
        .join()
        .expect("relay receiver thread should join after daemon projection");
    assert!(
        relay_forward_request.starts_with("POST /v1/messages/relay HTTP/1.1"),
        "daemon relay forward should target /v1/messages/relay"
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
fn integration_service_api_endpoint_rejects_legacy_message_send_recipient_dids() {
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
        "kamn-node-service-api-legacy-recipient-state-{unique_suffix}.json"
    ));
    let relay_spool_file = std::env::temp_dir().join(format!(
        "kamn-node-service-api-legacy-recipient-spool-{unique_suffix}.ndjson"
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
        "127.0.0.1:34122".to_owned(),
    ])
    .expect("api args should parse");
    let report = execute(parsed).expect("api execution should succeed");
    let snapshot = build_service_api_snapshot(&report);
    let state_hash = format!(
        "service-api:{}:{}",
        snapshot.chain_id.as_str(),
        snapshot.chain_version.as_str()
    );
    let sender_did = "kamn:did:agent:legacy-recipient-sender";
    let canonical_recipient_did = "kamn:did:agent:legacy-recipient-target";
    let legacy_send_body =
        r#"{"recipient_did":"did:kamn:agent:legacy-alpha","message":"reject-me"}"#;
    let canonical_send_body =
        format!(r#"{{"recipient_did":"{canonical_recipient_did}","message":"accept-me"}}"#);

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

    let legacy_signature = service_api_request_signature_for_fields(
        sender_did,
        41,
        state_hash.as_str(),
        legacy_send_body,
    );
    let legacy_response = send_http_request_with_headers(
        bind_addr.as_str(),
        "POST",
        "/v1/messages/send",
        legacy_send_body,
        &[
            ("X-KAMN-Sender-DID", sender_did),
            ("X-KAMN-Request-Nonce", "41"),
            ("X-KAMN-Request-Signature", legacy_signature.as_str()),
        ],
    );
    assert!(legacy_response.contains("HTTP/1.1 400 Bad Request"));
    let legacy_payload = parse_error_envelope_from_http_response(legacy_response.as_str());
    assert_eq!(
        legacy_payload.reason_code,
        SERVICE_API_MESSAGE_RECIPIENT_DID_INVALID_REASON_CODE
    );
    assert!(
        legacy_payload.message.contains("invalid recipient did"),
        "legacy recipient did rejection should explain the invalid recipient boundary"
    );

    let canonical_signature = service_api_request_signature_for_fields(
        sender_did,
        42,
        state_hash.as_str(),
        canonical_send_body.as_str(),
    );
    let canonical_response = send_http_request_with_headers(
        bind_addr.as_str(),
        "POST",
        "/v1/messages/send",
        canonical_send_body.as_str(),
        &[
            ("X-KAMN-Sender-DID", sender_did),
            ("X-KAMN-Request-Nonce", "42"),
            ("X-KAMN-Request-Signature", canonical_signature.as_str()),
        ],
    );
    assert!(canonical_response.contains("HTTP/1.1 202 Accepted"));
    let canonical_payload: ServiceApiMessageCreateBody =
        parse_service_api_payload(extract_http_response_body(canonical_response.as_str()))
            .expect("canonical send payload should deserialize");

    let server_result = server.join().expect("endpoint thread should complete");
    assert!(
        server_result.is_ok(),
        "service api endpoint should stop cleanly after legacy recipient rejection flow"
    );

    let state_payload =
        fs::read_to_string(state_file.as_path()).expect("state file should remain readable");
    let state_json: Value =
        serde_json::from_str(state_payload.as_str()).expect("state payload should parse");
    let messages = state_json["messages"]
        .as_object()
        .expect("messages snapshot should be an object");
    assert_eq!(messages.len(), 1, "only canonical sends should persist");
    let persisted_message = messages
        .values()
        .next()
        .expect("canonical send should persist exactly one message");
    assert_eq!(
        persisted_message["message_id"],
        canonical_payload.message_id
    );
    assert_eq!(persisted_message["recipient_did"], canonical_recipient_did);
    assert_eq!(persisted_message["sender_did"], sender_did);

    let relay_spool_payload = fs::read_to_string(relay_spool_file.as_path())
        .expect("relay spool should remain readable after canonical send");
    let relay_lines: Vec<&str> = relay_spool_payload
        .lines()
        .filter(|line| !line.trim().is_empty())
        .collect();
    assert_eq!(
        relay_lines.len(),
        1,
        "only canonical sends should enqueue relay spool entries"
    );
    let relay_entry_json: Value =
        serde_json::from_str(relay_lines[0]).expect("relay spool entry should deserialize");
    assert_eq!(relay_entry_json["message_id"], canonical_payload.message_id);
    assert_eq!(relay_entry_json["recipient_did"], canonical_recipient_did);

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
fn integration_service_api_endpoint_rejects_legacy_relay_ingest_dids() {
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
        "kamn-node-service-api-legacy-relay-state-{unique_suffix}.json"
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
        "127.0.0.1:34123".to_owned(),
    ])
    .expect("api args should parse");
    let report = execute(parsed).expect("api execution should succeed");
    let snapshot = build_service_api_snapshot(&report);
    let state_hash = format!(
        "service-api:{}:{}",
        snapshot.chain_id.as_str(),
        snapshot.chain_version.as_str()
    );
    let caller_did = "kamn:did:agent:relay-ingest-caller";
    let legacy_recipient_body = r#"{
      "message_id":"msg-relay-legacy-recipient",
      "sender_did":"kamn:did:agent:relay-sender",
      "recipient_did":"did:kamn:agent:legacy-recipient",
      "body":"{\"message\":\"relay-recipient\"}"
    }"#;
    let legacy_sender_body = r#"{
      "message_id":"msg-relay-legacy-sender",
      "sender_did":"did:kamn:agent:legacy-sender",
      "recipient_did":"kamn:did:agent:relay-recipient",
      "body":"{\"message\":\"relay-sender\"}"
    }"#;
    let canonical_body = r#"{
      "message_id":"msg-relay-canonical",
      "sender_did":"kamn:did:agent:relay-sender",
      "recipient_did":"kamn:did:agent:relay-recipient",
      "body":"{\"message\":\"relay-canonical\"}"
    }"#;

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

    let legacy_recipient_signature = service_api_request_signature_for_fields(
        caller_did,
        51,
        state_hash.as_str(),
        legacy_recipient_body,
    );
    let legacy_recipient_response = send_http_request_with_headers(
        bind_addr.as_str(),
        "POST",
        "/v1/messages/relay",
        legacy_recipient_body,
        &[
            ("X-KAMN-Sender-DID", caller_did),
            ("X-KAMN-Request-Nonce", "51"),
            (
                "X-KAMN-Request-Signature",
                legacy_recipient_signature.as_str(),
            ),
        ],
    );
    assert!(legacy_recipient_response.contains("HTTP/1.1 400 Bad Request"));
    let legacy_recipient_payload =
        parse_error_envelope_from_http_response(legacy_recipient_response.as_str());
    assert_eq!(
        legacy_recipient_payload.reason_code,
        SERVICE_API_RELAY_DID_INVALID_REASON_CODE
    );
    assert!(
        legacy_recipient_payload
            .message
            .contains("invalid relay recipient did"),
        "legacy relay recipient rejection should explain the invalid recipient boundary"
    );

    let legacy_sender_signature = service_api_request_signature_for_fields(
        caller_did,
        52,
        state_hash.as_str(),
        legacy_sender_body,
    );
    let legacy_sender_response = send_http_request_with_headers(
        bind_addr.as_str(),
        "POST",
        "/v1/messages/relay",
        legacy_sender_body,
        &[
            ("X-KAMN-Sender-DID", caller_did),
            ("X-KAMN-Request-Nonce", "52"),
            ("X-KAMN-Request-Signature", legacy_sender_signature.as_str()),
        ],
    );
    assert!(legacy_sender_response.contains("HTTP/1.1 400 Bad Request"));
    let legacy_sender_payload =
        parse_error_envelope_from_http_response(legacy_sender_response.as_str());
    assert_eq!(
        legacy_sender_payload.reason_code,
        SERVICE_API_RELAY_DID_INVALID_REASON_CODE
    );
    assert!(
        legacy_sender_payload
            .message
            .contains("invalid relay sender did"),
        "legacy relay sender rejection should explain the invalid sender boundary"
    );

    let canonical_signature = service_api_request_signature_for_fields(
        caller_did,
        53,
        state_hash.as_str(),
        canonical_body,
    );
    let canonical_response = send_http_request_with_headers(
        bind_addr.as_str(),
        "POST",
        "/v1/messages/relay",
        canonical_body,
        &[
            ("X-KAMN-Sender-DID", caller_did),
            ("X-KAMN-Request-Nonce", "53"),
            ("X-KAMN-Request-Signature", canonical_signature.as_str()),
        ],
    );
    assert!(canonical_response.contains("HTTP/1.1 202 Accepted"));

    let server_result = server.join().expect("endpoint thread should complete");
    assert!(
        server_result.is_ok(),
        "service api endpoint should stop cleanly after relay did rejection flow"
    );

    let state_payload =
        fs::read_to_string(state_file.as_path()).expect("state file should remain readable");
    let state_json: Value =
        serde_json::from_str(state_payload.as_str()).expect("state payload should parse");
    let messages = state_json["messages"]
        .as_object()
        .expect("messages snapshot should be an object");
    assert_eq!(
        messages.len(),
        1,
        "only canonical relay payloads should persist"
    );
    assert!(
        messages.contains_key("msg-relay-canonical"),
        "canonical relay payload should persist under its message id"
    );
    assert!(
        !messages.contains_key("msg-relay-legacy-recipient"),
        "legacy relay recipient payload must not persist"
    );
    assert!(
        !messages.contains_key("msg-relay-legacy-sender"),
        "legacy relay sender payload must not persist"
    );

    let _ = fs::remove_file(state_file);
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
    assert!(
        second_payload
            .message
            .contains("ingress rate limit exceeded")
    );

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
    assert!(
        rejection_payload
            .message
            .contains("ingress concurrency limit exceeded")
    );

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
    assert!(
        replay_payload
            .message
            .contains("request nonce replay detected")
    );

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
            assert!(
                payload
                    .message
                    .contains("ingress concurrency limit exceeded")
            );
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
