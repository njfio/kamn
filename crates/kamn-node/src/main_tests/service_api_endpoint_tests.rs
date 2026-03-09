use super::*;
use crate::service_api_endpoint::{
    parse_service_api_payload, project_service_api_lifecycle_rejection,
    upsert_service_api_relayed_message_from_daemon, ServiceApiAgentGetBody,
    ServiceApiChannelCreateBody, ServiceApiErrorBody,
    ServiceApiHealthBody, ServiceApiLifecycleRejectionProjection, ServiceApiMessageCreateBody,
    ServiceApiRelaySpoolEntry, ServiceApiTaskCreateBody,
    DEFAULT_SERVICE_API_BODY_LIMIT_BYTES, DEFAULT_SERVICE_API_CONCURRENCY_LIMIT,
    DEFAULT_SERVICE_API_RATE_LIMIT_PER_SECOND, SERVICE_API_AUTH_REASON_CODES_CSV,
    SERVICE_API_AUTH_REASON_TAXONOMY_VERSION, SERVICE_API_LIFECYCLE_REJECTION_REASON_CODES_CSV,
    SERVICE_API_ROUTE_AUTHZ_MATRIX_PROTECTED_ROUTE_COUNT,
    SERVICE_API_ROUTE_AUTHZ_MATRIX_PUBLIC_ROUTE_COUNT,
    SERVICE_API_ROUTE_AUTHZ_MATRIX_SCHEMA_VERSION,
    SERVICE_API_ROUTE_AUTHZ_MATRIX_TOTAL_ROUTE_COUNT,
    SERVICE_API_SCOPE_POLICY_FIXTURE_SCHEMA_VERSION, SERVICE_API_SCOPE_POLICY_REASON_CODES_CSV,
    SERVICE_API_SCOPE_POLICY_REASON_TAXONOMY_VERSION, SERVICE_API_WEBSOCKET_REASON_CODES_CSV,
};
use kamn_core::AgentDid;
use kamn_core::{
    cross_store_replay_reason_codes_csv, cross_store_replay_reason_taxonomy_version,
    service_auth_public_key_hex_from_private_key_hex, service_auth_sign_with_private_key_hex,
};
use rustls::pki_types::{CertificateDer, ServerName, UnixTime};
use serde::Deserialize;
use serde_json::Value;
use std::collections::BTreeMap;
use std::fs;
use std::io::{ErrorKind, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Barrier, MutexGuard};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

#[path = "service_api_endpoint_tests/auth_scope_contract_tests.rs"]
mod auth_scope_contract_tests;
#[path = "service_api_endpoint_tests/balance_contract_tests.rs"]
mod balance_contract_tests;
#[path = "service_api_endpoint_tests/bridge_persistence_restart_contract_tests.rs"]
mod bridge_persistence_restart_contract_tests;
#[path = "service_api_endpoint_tests/mailbox_relay_delivery_contract_tests.rs"]
mod mailbox_relay_delivery_contract_tests;
#[path = "service_api_endpoint_tests/channel_agent_directory_contract_tests.rs"]
mod channel_agent_directory_contract_tests;
#[path = "service_api_endpoint_tests/content_lifecycle_restart_contract_tests.rs"]
mod content_lifecycle_restart_contract_tests;
#[path = "service_api_endpoint_tests/message_persistence_contract_tests.rs"]
mod message_persistence_contract_tests;
#[path = "service_api_endpoint_tests/route_render_contract_tests.rs"]
mod route_render_contract_tests;
#[path = "service_api_endpoint_tests/task_escrow_persistence_contract_tests.rs"]
mod task_escrow_persistence_contract_tests;
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

fn test_service_api_auth_public_key_hex() -> String {
    service_auth_public_key_hex_from_private_key_hex(TEST_SERVICE_API_AUTH_PRIVATE_KEY_HEX)
        .expect("service-auth public key should derive")
}

fn test_service_api_sender_did(sender: &str) -> String {
    let public_key_hex = test_service_api_auth_public_key_hex();
    let Ok(parsed_sender_did) = AgentDid::parse(sender) else {
        return sender.to_owned();
    };
    if parsed_sender_did.method_specific_id().starts_with("pkh-") {
        return sender.to_owned();
    }
    if parsed_sender_did
        .ensure_public_key_hex_binding(public_key_hex.as_str())
        .is_ok()
    {
        return sender.to_owned();
    }
    AgentDid::with_public_key_hex_binding(
        parsed_sender_did.method_specific_id(),
        public_key_hex.as_str(),
    )
    .expect("test sender did should bind to fixture signer key")
    .as_str()
    .to_owned()
}

fn enrich_signed_headers_with_scope(
    method: &str,
    path: &str,
    headers: &[(&str, &str)],
) -> Vec<(String, String)> {
    let mut enriched: Vec<(String, String)> = headers
        .iter()
        .map(|(name, value)| ((*name).to_owned(), (*value).to_owned()))
        .collect();
    let signed_request = signed_header_present(headers, "X-KAMN-Sender-DID")
        && signed_header_present(headers, "X-KAMN-Request-Nonce")
        && signed_header_present(headers, "X-KAMN-Request-Signature");
    let has_scope_header = signed_header_present(headers, "X-KAMN-Authz-Scope");
    let has_signer_public_key_header = signed_header_present(headers, "X-KAMN-Signer-Public-Key");
    if signed_request && !has_scope_header {
        if let Some(scope) = required_scope_for_test_route(method, path) {
            enriched.push(("X-KAMN-Authz-Scope".to_owned(), scope.to_owned()));
        }
    }
    if signed_request {
        for (name, value) in &mut enriched {
            if name.eq_ignore_ascii_case("X-KAMN-Sender-DID") {
                *value = test_service_api_sender_did(value.as_str());
            }
        }
    }
    if signed_request && !has_signer_public_key_header {
        enriched.push((
            "X-KAMN-Signer-Public-Key".to_owned(),
            test_service_api_auth_public_key_hex(),
        ));
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
    let enriched_header_refs: Vec<(&str, &str)> = enriched_headers
        .iter()
        .map(|(name, value)| (name.as_str(), value.as_str()))
        .collect();
    send_http_request_with_headers_raw(addr, method, path, body, enriched_header_refs.as_slice())
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
        header_lines.push_str(name.as_str());
        header_lines.push_str(": ");
        header_lines.push_str(value.as_str());
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
    let enriched_header_refs: Vec<(&str, &str)> = enriched_headers
        .iter()
        .map(|(name, value)| (name.as_str(), value.as_str()))
        .collect();
    send_https_request_with_headers_raw(
        addr,
        method,
        path,
        body,
        enriched_header_refs.as_slice(),
        _root_cert_pem,
    )
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
        test_service_api_sender_did(sender).as_str(),
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
    let auth_public_key_hex = test_service_api_auth_public_key_hex();
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
        route_render_contract_tests::websocket_upgrade_required_reason_code()
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
    route_render_contract_tests::assert_common_route_metrics(metrics_response.as_str());
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
    route_render_contract_tests::assert_common_route_metrics(metrics_response.as_str());
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
    let metrics_response = route_render_contract_tests::render_metrics_response(&snapshot);
    assert_eq!(metrics_response.status_code, 200);
    assert!(metrics_response
        .body
        .contains("kamn_service_api_observability_source{source=\"daemon\"} 1"));
    assert!(metrics_response
        .body
        .contains("kamn_service_api_observability_health{health=\"healthy\"} 1"));
    route_render_contract_tests::assert_common_route_metrics(metrics_response.body.as_str());
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
