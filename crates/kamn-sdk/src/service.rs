use crate::{AgentDid, SdkError};
use kamn_core::{
    service_auth_public_key_hex_from_private_key_hex, service_auth_sign_with_private_key_hex,
    service_auth_verify_with_public_key_hex, ServiceAuthSignatureError,
    SERVICE_AUTH_SIGNATURE_PRIVATE_KEY_ENV,
};
use rustls::pki_types::ServerName;
use rustls::{ClientConfig, ClientConnection, RootCertStore, StreamOwned};
use std::fs;
use std::io::{Cursor, ErrorKind, Read, Write};
use std::net::{IpAddr, TcpStream};
use std::sync::Arc;
use std::time::Duration;

#[path = "service_response.rs"]
mod service_response;
#[path = "service_websocket.rs"]
mod service_websocket;
use self::service_response::{
    expect_status, json_string_array_field, json_string_field, json_u64_field,
    map_non_success_response, parse_http_response, status_from_header,
};
use self::service_websocket::parse_unmasked_text_frame_payload;

const REQUEST_TIMEOUT_SECONDS_DEFAULT: u64 = 2;
const REQUEST_TIMEOUT_SECONDS_ENV: &str = "KAMN_SDK_SERVICE_TIMEOUT_SECONDS";
const REQUEST_TIMEOUT_SECONDS_FIELD: &str = "service.request_timeout_seconds";
const REQUEST_TIMEOUT_SECONDS_EMPTY_REASON: &str = "must not be empty when set";
const REQUEST_TIMEOUT_SECONDS_INVALID_REASON: &str = "must be valid integer seconds";
const REQUEST_TIMEOUT_SECONDS_NON_POSITIVE_REASON: &str = "must be greater than zero";
const SERVICE_TLS_CA_FILE_ENV: &str = "KAMN_SERVICE_API_TLS_CA_FILE";
const REQUEST_AUTH_SENDER_DID_HEADER: &str = "x-kamn-sender-did";
const REQUEST_AUTH_NONCE_HEADER: &str = "x-kamn-request-nonce";
const REQUEST_AUTH_SIGNATURE_HEADER: &str = "x-kamn-request-signature";
const REQUEST_AUTH_SIGNER_PUBLIC_KEY_HEADER: &str = "x-kamn-signer-public-key";
const REQUEST_AUTH_SCOPE_HEADER: &str = "x-kamn-authz-scope";
const SERVICE_TLS_CA_FILE_FIELD: &str = "service.tls.ca_file";
const SERVICE_TLS_CA_FILE_EMPTY_REASON: &str = "must not be empty when set";
const SERVICE_TLS_CA_FILE_UTF8_REASON: &str = "must be valid utf-8 when set";
const SERVICE_TLS_CA_FILE_READ_FAILED: &str = "service tls ca file read failed";
const SERVICE_TLS_CA_FILE_PARSE_FAILED: &str = "service tls ca file parse failed";
const SERVICE_TLS_CA_FILE_EMPTY_BUNDLE: &str =
    "service tls ca file did not contain valid certificates";
const SERVICE_TLS_CERTIFICATE_VERIFICATION_FAILED: &str =
    "service tls certificate verification failed";
const SERVICE_TLS_HANDSHAKE_FAILED: &str = "service tls handshake failed";
const SERVICE_TLS_SERVER_NAME_INVALID: &str = "service tls server name was invalid";
const SERVICE_WS_ROUTE: &str = "/v1/events/ws";
const REASON_CODE_WEBSOCKET_UPGRADE_REQUIRED: &str = "service_api_websocket_upgrade_required";
const REASON_CODE_METHOD_NOT_ALLOWED: &str = "service_api_method_not_allowed";
const REASON_CODE_ROUTE_NOT_FOUND: &str = "service_api_route_not_found";
const REASON_CODE_AUTH_SENDER_DID_HEADER_MISSING: &str =
    "service_api_auth_sender_did_header_missing";
const REASON_CODE_AUTH_NONCE_HEADER_MISSING: &str = "service_api_auth_nonce_header_missing";
const REASON_CODE_AUTH_NONCE_INVALID: &str = "service_api_auth_nonce_invalid";
const REASON_CODE_AUTH_NONCE_NON_POSITIVE: &str = "service_api_auth_nonce_non_positive";
const REASON_CODE_AUTH_SIGNATURE_HEADER_MISSING: &str = "service_api_auth_signature_header_missing";
const REASON_CODE_AUTH_SIGNATURE_VERIFICATION_FAILED: &str =
    "service_api_auth_signature_verification_failed";
const REASON_CODE_AUTH_REPLAY_NONCE_DETECTED: &str = "service_api_auth_replay_nonce_detected";
const REASON_CODE_LEGACY_UNAUTHORIZED: &str = "service_api_legacy_unauthorized";
const REASON_CODE_LEGACY_CONFLICT: &str = "service_api_legacy_conflict";
const REASON_CODE_LEGACY_BAD_REQUEST: &str = "service_api_legacy_bad_request";
const REASON_CODE_LEGACY_UNKNOWN: &str = "service_api_legacy_error_unknown";
const MAX_SERVICE_RESPONSE_BYTES: usize = 1_048_576;
const MAX_SERVICE_RESPONSE_READ_ITERATIONS: usize = 8_192;
const SERVICE_RESPONSE_SIZE_LIMIT_EXCEEDED: &str =
    "service response payload exceeded maximum supported size";
const SERVICE_RESPONSE_READ_ITERATION_LIMIT_EXCEEDED: &str =
    "service response payload read exceeded bounded iteration budget";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ServiceScheme {
    Http,
    Https,
}

impl ServiceScheme {
    fn default_port(self) -> u16 {
        match self {
            Self::Http => 80,
            Self::Https => 443,
        }
    }
}

enum ServiceStream {
    Tcp(TcpStream),
    Tls(Box<StreamOwned<ClientConnection, TcpStream>>),
}

impl Read for ServiceStream {
    fn read(&mut self, buffer: &mut [u8]) -> std::io::Result<usize> {
        match self {
            Self::Tcp(stream) => stream.read(buffer),
            Self::Tls(stream) => stream.read(buffer),
        }
    }
}

impl Write for ServiceStream {
    fn write(&mut self, buffer: &[u8]) -> std::io::Result<usize> {
        match self {
            Self::Tcp(stream) => stream.write(buffer),
            Self::Tls(stream) => stream.write(buffer),
        }
    }

    fn flush(&mut self) -> std::io::Result<()> {
        match self {
            Self::Tcp(stream) => stream.flush(),
            Self::Tls(stream) => stream.flush(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ServiceEndpoint {
    scheme: ServiceScheme,
    host: String,
    port: u16,
    base_path: String,
}

impl ServiceEndpoint {
    fn parse(endpoint: &str) -> Result<Self, SdkError> {
        let trimmed = endpoint.trim();
        if trimmed.is_empty() {
            return Err(SdkError::InvalidInput {
                field: "service.endpoint",
                reason: "must not be empty",
            });
        }

        let (scheme, suffix) = if let Some(suffix) = trimmed.strip_prefix("http://") {
            (ServiceScheme::Http, suffix)
        } else if let Some(suffix) = trimmed.strip_prefix("https://") {
            (ServiceScheme::Https, suffix)
        } else {
            return Err(SdkError::InvalidInput {
                field: "service.endpoint",
                reason: "must start with http:// or https://",
            });
        };

        let (authority, base_path) = match suffix.split_once('/') {
            Some((authority, path)) => (
                authority,
                format!("/{path}").trim_end_matches('/').to_owned(),
            ),
            None => (suffix, String::new()),
        };
        if authority.trim().is_empty() {
            return Err(SdkError::InvalidInput {
                field: "service.endpoint",
                reason: "host is required",
            });
        }

        let (host, port) = parse_host_port(authority, scheme.default_port())?;
        Ok(Self {
            scheme,
            host,
            port,
            base_path,
        })
    }

    fn route_path(&self, route: &str) -> String {
        if self.base_path.is_empty() {
            return route.to_owned();
        }
        format!("{}{}", self.base_path, route)
    }

    fn connect_tcp_stream(&self) -> Result<TcpStream, SdkError> {
        let timeout = Duration::from_secs(resolve_request_timeout_seconds()?);
        let stream = TcpStream::connect((self.host.as_str(), self.port))
            .map_err(|_| SdkError::TransportFailure("failed to connect to service endpoint"))?;
        stream
            .set_read_timeout(Some(timeout))
            .map_err(|_| SdkError::TransportFailure("failed to configure service read timeout"))?;
        stream
            .set_write_timeout(Some(timeout))
            .map_err(|_| SdkError::TransportFailure("failed to configure service write timeout"))?;
        Ok(stream)
    }

    fn connect_stream(&self) -> Result<ServiceStream, SdkError> {
        let tcp_stream = self.connect_tcp_stream()?;
        if self.scheme == ServiceScheme::Http {
            return Ok(ServiceStream::Tcp(tcp_stream));
        }

        let tls_client_config = resolve_tls_client_config()?;
        let server_name = resolve_tls_server_name(self.host.as_str())?;
        let connection = ClientConnection::new(tls_client_config, server_name)
            .map_err(|_| SdkError::TransportFailure(SERVICE_TLS_HANDSHAKE_FAILED))?;
        Ok(ServiceStream::Tls(Box::new(StreamOwned::new(
            connection, tcp_stream,
        ))))
    }
}

fn resolve_tls_client_config() -> Result<Arc<ClientConfig>, SdkError> {
    let mut root_store = RootCertStore::empty();
    match std::env::var(SERVICE_TLS_CA_FILE_ENV) {
        Ok(configured_ca_file) => {
            let normalized_ca_file = configured_ca_file.trim();
            if normalized_ca_file.is_empty() {
                return Err(SdkError::InvalidInput {
                    field: SERVICE_TLS_CA_FILE_FIELD,
                    reason: SERVICE_TLS_CA_FILE_EMPTY_REASON,
                });
            }
            let cert_bytes = fs::read(normalized_ca_file)
                .map_err(|_| SdkError::TransportFailure(SERVICE_TLS_CA_FILE_READ_FAILED))?;
            let mut cert_reader = Cursor::new(cert_bytes.as_slice());
            let certificates = rustls_pemfile::certs(&mut cert_reader)
                .collect::<Result<Vec<_>, _>>()
                .map_err(|_| SdkError::TransportFailure(SERVICE_TLS_CA_FILE_PARSE_FAILED))?;
            let (added, _) = root_store.add_parsable_certificates(certificates);
            if added == 0 {
                return Err(SdkError::TransportFailure(SERVICE_TLS_CA_FILE_EMPTY_BUNDLE));
            }
        }
        Err(std::env::VarError::NotPresent) => {
            root_store.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());
        }
        Err(std::env::VarError::NotUnicode(_)) => {
            return Err(SdkError::InvalidInput {
                field: SERVICE_TLS_CA_FILE_FIELD,
                reason: SERVICE_TLS_CA_FILE_UTF8_REASON,
            });
        }
    }

    let config = ClientConfig::builder()
        .with_root_certificates(root_store)
        .with_no_client_auth();
    Ok(Arc::new(config))
}

fn resolve_request_timeout_seconds() -> Result<u64, SdkError> {
    match std::env::var(REQUEST_TIMEOUT_SECONDS_ENV) {
        Ok(raw_timeout) => {
            let normalized_timeout = raw_timeout.trim();
            if normalized_timeout.is_empty() {
                return Err(SdkError::InvalidInput {
                    field: REQUEST_TIMEOUT_SECONDS_FIELD,
                    reason: REQUEST_TIMEOUT_SECONDS_EMPTY_REASON,
                });
            }
            let parsed_timeout =
                normalized_timeout
                    .parse::<u64>()
                    .map_err(|_| SdkError::InvalidInput {
                        field: REQUEST_TIMEOUT_SECONDS_FIELD,
                        reason: REQUEST_TIMEOUT_SECONDS_INVALID_REASON,
                    })?;
            if parsed_timeout == 0 {
                return Err(SdkError::InvalidInput {
                    field: REQUEST_TIMEOUT_SECONDS_FIELD,
                    reason: REQUEST_TIMEOUT_SECONDS_NON_POSITIVE_REASON,
                });
            }
            Ok(parsed_timeout)
        }
        Err(std::env::VarError::NotPresent) => Ok(REQUEST_TIMEOUT_SECONDS_DEFAULT),
        Err(std::env::VarError::NotUnicode(_)) => Err(SdkError::InvalidInput {
            field: REQUEST_TIMEOUT_SECONDS_FIELD,
            reason: REQUEST_TIMEOUT_SECONDS_INVALID_REASON,
        }),
    }
}

fn resolve_tls_server_name(host: &str) -> Result<ServerName<'static>, SdkError> {
    if let Ok(ip_addr) = host.parse::<IpAddr>() {
        return Ok(ServerName::IpAddress(ip_addr.into()));
    }
    ServerName::try_from(host.to_owned())
        .map_err(|_| SdkError::TransportFailure(SERVICE_TLS_SERVER_NAME_INVALID))
}

fn classify_tls_io_error(error: &std::io::Error) -> &'static str {
    if let Some(rustls_error) = error
        .get_ref()
        .and_then(|source| source.downcast_ref::<rustls::Error>())
    {
        return match rustls_error {
            rustls::Error::InvalidCertificate(_) => SERVICE_TLS_CERTIFICATE_VERIFICATION_FAILED,
            _ => SERVICE_TLS_HANDSHAKE_FAILED,
        };
    }
    if matches!(error.kind(), std::io::ErrorKind::TimedOut) {
        return "failed to read service response payload";
    }
    SERVICE_TLS_HANDSHAKE_FAILED
}

fn map_stream_write_error(error: &std::io::Error, default_reason: &'static str) -> SdkError {
    if error
        .get_ref()
        .and_then(|source| source.downcast_ref::<rustls::Error>())
        .is_some()
    {
        return SdkError::TransportFailure(classify_tls_io_error(error));
    }
    SdkError::TransportFailure(default_reason)
}

fn map_stream_read_error(error: &std::io::Error, default_reason: &'static str) -> SdkError {
    if error
        .get_ref()
        .and_then(|source| source.downcast_ref::<rustls::Error>())
        .is_some()
    {
        return SdkError::TransportFailure(classify_tls_io_error(error));
    }
    SdkError::TransportFailure(default_reason)
}

/// Request authentication envelope for service API routes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServiceRequestAuth {
    sender_did: AgentDid,
    nonce: u64,
    signature: String,
    signer_public_key_hex: Option<String>,
    scope: Option<String>,
}

impl ServiceRequestAuth {
    /// Builds a validated request auth envelope.
    pub fn new(sender_did: AgentDid, nonce: u64, signature: String) -> Result<Self, SdkError> {
        Self::new_with_signer_public_key_and_scope(sender_did, nonce, signature, None, None)
    }

    /// Builds a validated request auth envelope with optional auth scope marker.
    pub fn new_with_scope(
        sender_did: AgentDid,
        nonce: u64,
        signature: String,
        scope: Option<&str>,
    ) -> Result<Self, SdkError> {
        Self::new_with_signer_public_key_and_scope(sender_did, nonce, signature, None, scope)
    }

    /// Builds a validated request auth envelope with optional signer key and auth scope marker.
    pub fn new_with_signer_public_key_and_scope(
        sender_did: AgentDid,
        nonce: u64,
        signature: String,
        signer_public_key_hex: Option<&str>,
        scope: Option<&str>,
    ) -> Result<Self, SdkError> {
        if nonce == 0 {
            return Err(SdkError::InvalidInput {
                field: "request_auth.nonce",
                reason: "must be greater than zero",
            });
        }
        let normalized_signature = signature.trim();
        if normalized_signature.is_empty() {
            return Err(SdkError::InvalidInput {
                field: "request_auth.signature",
                reason: "must not be empty",
            });
        }
        validate_http_header_value("request_auth.sender_did", sender_did.as_str())?;
        validate_http_header_value("request_auth.signature", normalized_signature)?;
        let scope = match scope {
            Some(scope) => {
                let normalized = scope.trim();
                if normalized.is_empty() {
                    return Err(SdkError::InvalidInput {
                        field: "request_auth.scope",
                        reason: "must not be empty when set",
                    });
                }
                validate_http_header_value("request_auth.scope", normalized)?;
                Some(normalized.to_owned())
            }
            None => None,
        };
        let signer_public_key_hex = match signer_public_key_hex {
            Some(value) => {
                let normalized = value.trim();
                if normalized.is_empty() {
                    return Err(SdkError::InvalidInput {
                        field: "request_auth.signer_public_key",
                        reason: "must not be empty when set",
                    });
                }
                validate_http_header_value("request_auth.signer_public_key", normalized)?;
                Some(normalized.to_owned())
            }
            None => None,
        };
        Ok(Self {
            sender_did,
            nonce,
            signature: normalized_signature.to_owned(),
            signer_public_key_hex,
            scope,
        })
    }

    fn sender_did(&self) -> &AgentDid {
        &self.sender_did
    }

    fn nonce(&self) -> u64 {
        self.nonce
    }

    fn signature(&self) -> &str {
        self.signature.as_str()
    }

    fn signer_public_key_hex(&self) -> Option<&str> {
        self.signer_public_key_hex.as_deref()
    }

    fn scope(&self) -> Option<&str> {
        self.scope.as_deref()
    }
}

/// Deterministic request signature builder for service API fields.
pub fn service_signature_for_fields(
    sender_did: &AgentDid,
    nonce: u64,
    chain_id: &str,
    chain_version: &str,
    body: &str,
) -> Result<String, SdkError> {
    let private_key_hex = std::env::var(SERVICE_AUTH_SIGNATURE_PRIVATE_KEY_ENV).map_err(|_| {
        SdkError::InvalidInput {
            field: "service.request_auth.private_key",
            reason: "missing KAMN_SERVICE_API_AUTH_PRIVATE_KEY_HEX",
        }
    })?;
    let state_hash = format!("service-api:{chain_id}:{chain_version}");
    service_signature_for_state_hash_with_private_key(
        sender_did,
        nonce,
        state_hash.as_str(),
        body,
        private_key_hex.as_str(),
    )
}

/// Derives signer public key hex from the configured service signing private key env.
pub fn service_signer_public_key_for_fields() -> Result<String, SdkError> {
    let private_key_hex = std::env::var(SERVICE_AUTH_SIGNATURE_PRIVATE_KEY_ENV).map_err(|_| {
        SdkError::InvalidInput {
            field: "service.request_auth.private_key",
            reason: "missing KAMN_SERVICE_API_AUTH_PRIVATE_KEY_HEX",
        }
    })?;
    service_public_key_for_private_key(private_key_hex.as_str())
}

/// Cryptographic request signature builder for canonical service state-hash fields.
pub fn service_signature_for_state_hash_with_private_key(
    sender_did: &AgentDid,
    nonce: u64,
    state_hash: &str,
    body: &str,
    private_key_hex: &str,
) -> Result<String, SdkError> {
    service_auth_sign_with_private_key_hex(
        sender_did.as_str(),
        nonce,
        state_hash,
        body,
        private_key_hex,
    )
    .map_err(map_service_auth_error_to_sdk)
}

/// Derives compressed secp256k1 public key hex from private key material.
pub fn service_public_key_for_private_key(private_key_hex: &str) -> Result<String, SdkError> {
    service_auth_public_key_hex_from_private_key_hex(private_key_hex).map_err(|error| match error {
        ServiceAuthSignatureError::EmptyField("private_key_hex")
        | ServiceAuthSignatureError::InvalidPrivateKeyHex => SdkError::InvalidInput {
            field: "service.request_auth.private_key",
            reason: "must be valid secp256k1 private key hex",
        },
        _ => SdkError::InvalidInput {
            field: "service.request_auth.private_key",
            reason: "failed to derive secp256k1 signer public key",
        },
    })
}

/// Verifies a service signature against canonical state-hash fields and signer public key.
pub fn service_verify_signature_with_public_key(
    sender_did: &AgentDid,
    nonce: u64,
    state_hash: &str,
    body: &str,
    signature: &str,
    signer_public_key_hex: &str,
) -> Result<(), SdkError> {
    service_auth_verify_with_public_key_hex(
        signature,
        sender_did.as_str(),
        nonce,
        state_hash,
        body,
        signer_public_key_hex,
    )
    .map_err(|error| match error {
        ServiceAuthSignatureError::EmptyField("expected_public_key_hex")
        | ServiceAuthSignatureError::InvalidPublicKeyHex => SdkError::InvalidInput {
            field: "service.request_auth.expected_public_key",
            reason: "must be valid compressed secp256k1 public key hex",
        },
        ServiceAuthSignatureError::EmptyField("state_hash") => SdkError::InvalidInput {
            field: "service.request_auth.state_hash",
            reason: "must not be empty",
        },
        ServiceAuthSignatureError::EmptyField("signature") => SdkError::InvalidInput {
            field: "service.request_auth.signature",
            reason: "must not be empty",
        },
        ServiceAuthSignatureError::InvalidNonce => SdkError::InvalidInput {
            field: "service.request_auth.nonce",
            reason: "must be greater than zero",
        },
        _ => SdkError::InvalidInput {
            field: "service.request_auth.signature",
            reason: "failed cryptographic signature verification",
        },
    })
}

fn map_service_auth_error_to_sdk(error: ServiceAuthSignatureError) -> SdkError {
    match error {
        ServiceAuthSignatureError::EmptyField("private_key_hex")
        | ServiceAuthSignatureError::InvalidPrivateKeyHex => SdkError::InvalidInput {
            field: "service.request_auth.private_key",
            reason: "must be valid secp256k1 private key hex",
        },
        ServiceAuthSignatureError::EmptyField("state_hash") => SdkError::InvalidInput {
            field: "service.request_auth.state_hash",
            reason: "must not be empty",
        },
        ServiceAuthSignatureError::InvalidNonce => SdkError::InvalidInput {
            field: "service.request_auth.nonce",
            reason: "must be greater than zero",
        },
        _ => SdkError::InvalidInput {
            field: "service.request_auth.signature",
            reason: "failed to produce cryptographic service signature",
        },
    }
}

/// Parsed response for `POST /v1/messages/send`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServiceMessageReceipt {
    /// Service-generated message identifier.
    pub message_id: String,
    /// Lifecycle status marker.
    pub status: String,
    /// Runtime mode reported by the service.
    pub runtime_mode: String,
}

/// Parsed response for `GET /v1/messages/{id}`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServiceMessageStatus {
    /// Message identifier.
    pub message_id: String,
    /// Lifecycle status marker.
    pub status: String,
}

/// Parsed response for `POST /v1/channels/create`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServiceChannelReceipt {
    /// Service-generated channel identifier.
    pub channel_id: String,
    /// Route status marker.
    pub status: String,
}

/// Parsed response for `GET /v1/channels/{id}/messages`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServiceChannelMessages {
    /// Channel identifier.
    pub channel_id: String,
    /// Message identifiers observed in the channel.
    pub messages: Vec<String>,
}

/// Parsed response for `POST /v1/tasks/create`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServiceTaskReceipt {
    /// Service-generated task identifier.
    pub task_id: String,
    /// Initial lifecycle state.
    pub state: String,
}

/// Parsed response for `GET /v1/tasks/{id}`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServiceTaskStatus {
    /// Task identifier.
    pub task_id: String,
    /// Current lifecycle state.
    pub state: String,
}

/// Parsed response for escrow lifecycle routes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServiceEscrowStatus {
    /// Escrow identifier.
    pub escrow_id: String,
    /// Current lifecycle state.
    pub state: String,
}

/// Parsed response for `POST /v1/content/register`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServiceContentRegistration {
    /// Service-generated content identifier.
    pub content_id: String,
    /// Retention class marker.
    pub retention_class: String,
    /// Lifecycle state marker.
    pub lifecycle_state: String,
    /// Redaction status marker.
    pub redaction_status: String,
}

/// Parsed response for content lifecycle routes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServiceContentStatus {
    /// Content identifier.
    pub content_id: String,
    /// Lifecycle state marker.
    pub lifecycle_state: String,
    /// Redaction status marker.
    pub redaction_status: String,
}

/// Parsed response for `POST /v1/bridge/submit`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServiceBridgeSubmission {
    /// Bridge identifier.
    pub bridge_id: String,
    /// Source message identifier.
    pub source_message_id: String,
    /// Bridge lifecycle status marker.
    pub bridge_status: String,
}

/// Parsed response for bridge forward/query routes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServiceBridgeStatus {
    /// Bridge identifier.
    pub bridge_id: String,
    /// Bridge lifecycle status marker.
    pub bridge_status: String,
    /// Target message identifier after forwarding.
    pub target_message_id: String,
    /// Forward transaction hash marker.
    pub forward_tx_hash: String,
}

/// Parsed response for `GET /v1/agents/{did}`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServiceAgentProfile {
    /// Agent DID.
    pub did: String,
    /// Current reputation score.
    pub reputation_score: u64,
}

/// Parsed response for `GET /healthz`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServiceHealthStatus {
    /// Service health marker.
    pub status: String,
    /// Runtime mode marker.
    pub runtime_mode: String,
    /// Node role marker.
    pub role: String,
    /// Observability source marker.
    pub observability_source: String,
    /// Observability health marker.
    pub observability_health: String,
}

/// Parsed event frame payload from `GET /v1/events/ws`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServiceRouteEvent {
    /// Event name.
    pub event: String,
    /// Runtime mode marker.
    pub runtime_mode: String,
    /// Node role marker.
    pub role: String,
    /// Event sequence identifier.
    pub sequence: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct HttpResponse {
    status: u16,
    body: String,
}

/// Synchronous SDK client for KAMN service HTTP and WebSocket routes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServiceApiClient {
    endpoint: ServiceEndpoint,
}

impl ServiceApiClient {
    /// Connects and validates the base service endpoint.
    pub fn connect(endpoint: &str) -> Result<Self, SdkError> {
        Ok(Self {
            endpoint: ServiceEndpoint::parse(endpoint)?,
        })
    }

    /// Sends a signed message payload through the service API.
    pub fn send_message(
        &self,
        payload: &str,
        auth: &ServiceRequestAuth,
    ) -> Result<ServiceMessageReceipt, SdkError> {
        let response = self.request("POST", "/v1/messages/send", payload, Some(auth))?;
        expect_status(response.status, 202)?;
        Ok(ServiceMessageReceipt {
            message_id: json_string_field(response.body.as_str(), "message_id")?,
            status: json_string_field(response.body.as_str(), "status")?,
            runtime_mode: json_string_field(response.body.as_str(), "runtime_mode")?,
        })
    }

    /// Queries a message status by identifier.
    pub fn get_message(
        &self,
        message_id: &str,
        auth: &ServiceRequestAuth,
    ) -> Result<ServiceMessageStatus, SdkError> {
        let message_id = normalize_route_segment("message_id", message_id)?;
        let route = format!("/v1/messages/{message_id}");
        let response = self.request("GET", route.as_str(), "", Some(auth))?;
        expect_status(response.status, 200)?;
        Ok(ServiceMessageStatus {
            message_id: json_string_field(response.body.as_str(), "message_id")?,
            status: json_string_field(response.body.as_str(), "status")?,
        })
    }

    /// Creates a channel payload through the service API.
    pub fn create_channel(
        &self,
        payload: &str,
        auth: &ServiceRequestAuth,
    ) -> Result<ServiceChannelReceipt, SdkError> {
        let response = self.request("POST", "/v1/channels/create", payload, Some(auth))?;
        expect_status(response.status, 201)?;
        Ok(ServiceChannelReceipt {
            channel_id: json_string_field(response.body.as_str(), "channel_id")?,
            status: json_string_field(response.body.as_str(), "status")?,
        })
    }

    /// Lists channel messages through `GET /v1/channels/{id}/messages`.
    pub fn list_channel_messages(
        &self,
        channel_id: &str,
        auth: &ServiceRequestAuth,
    ) -> Result<ServiceChannelMessages, SdkError> {
        let channel_id = normalize_route_segment("channel_id", channel_id)?;
        let route = format!("/v1/channels/{channel_id}/messages");
        let response = self.request("GET", route.as_str(), "", Some(auth))?;
        expect_status(response.status, 200)?;
        Ok(ServiceChannelMessages {
            channel_id: json_string_field(response.body.as_str(), "channel_id")?,
            messages: json_string_array_field(response.body.as_str(), "messages")?,
        })
    }

    /// Creates a task payload through the service API.
    pub fn create_task(
        &self,
        payload: &str,
        auth: &ServiceRequestAuth,
    ) -> Result<ServiceTaskReceipt, SdkError> {
        let response = self.request("POST", "/v1/tasks/create", payload, Some(auth))?;
        expect_status(response.status, 201)?;
        Ok(ServiceTaskReceipt {
            task_id: json_string_field(response.body.as_str(), "task_id")?,
            state: json_string_field(response.body.as_str(), "state")?,
        })
    }

    /// Queries task lifecycle state by identifier.
    pub fn get_task(
        &self,
        task_id: &str,
        auth: &ServiceRequestAuth,
    ) -> Result<ServiceTaskStatus, SdkError> {
        let task_id = normalize_route_segment("task_id", task_id)?;
        let route = format!("/v1/tasks/{task_id}");
        let response = self.request("GET", route.as_str(), "", Some(auth))?;
        expect_status(response.status, 200)?;
        Ok(ServiceTaskStatus {
            task_id: json_string_field(response.body.as_str(), "task_id")?,
            state: json_string_field(response.body.as_str(), "state")?,
        })
    }

    /// Accepts one task through `POST /v1/tasks/{id}/accept`.
    pub fn accept_task(
        &self,
        task_id: &str,
        auth: &ServiceRequestAuth,
    ) -> Result<ServiceTaskStatus, SdkError> {
        let task_id = normalize_route_segment("task_id", task_id)?;
        let route = format!("/v1/tasks/{task_id}/accept");
        let response = self.request("POST", route.as_str(), "{}", Some(auth))?;
        expect_status(response.status, 200)?;
        Ok(ServiceTaskStatus {
            task_id: json_string_field(response.body.as_str(), "task_id")?,
            state: json_string_field(response.body.as_str(), "state")?,
        })
    }

    /// Completes one task through `POST /v1/tasks/{id}/complete`.
    pub fn complete_task(
        &self,
        task_id: &str,
        auth: &ServiceRequestAuth,
    ) -> Result<ServiceTaskStatus, SdkError> {
        let task_id = normalize_route_segment("task_id", task_id)?;
        let route = format!("/v1/tasks/{task_id}/complete");
        let response = self.request("POST", route.as_str(), "{}", Some(auth))?;
        expect_status(response.status, 200)?;
        Ok(ServiceTaskStatus {
            task_id: json_string_field(response.body.as_str(), "task_id")?,
            state: json_string_field(response.body.as_str(), "state")?,
        })
    }

    /// Funds escrow through `POST /v1/escrow/fund`.
    pub fn fund_escrow(
        &self,
        payload: &str,
        auth: &ServiceRequestAuth,
    ) -> Result<ServiceEscrowStatus, SdkError> {
        let response = self.request("POST", "/v1/escrow/fund", payload, Some(auth))?;
        expect_status(response.status, 200)?;
        Ok(ServiceEscrowStatus {
            escrow_id: json_string_field(response.body.as_str(), "escrow_id")?,
            state: json_string_field(response.body.as_str(), "state")?,
        })
    }

    /// Releases escrow through `POST /v1/escrow/{id}/release`.
    pub fn release_escrow(
        &self,
        escrow_id: &str,
        auth: &ServiceRequestAuth,
    ) -> Result<ServiceEscrowStatus, SdkError> {
        let escrow_id = normalize_route_segment("escrow_id", escrow_id)?;
        let route = format!("/v1/escrow/{escrow_id}/release");
        let response = self.request("POST", route.as_str(), "{}", Some(auth))?;
        expect_status(response.status, 200)?;
        Ok(ServiceEscrowStatus {
            escrow_id: json_string_field(response.body.as_str(), "escrow_id")?,
            state: json_string_field(response.body.as_str(), "state")?,
        })
    }

    /// Registers content retention lifecycle via `POST /v1/content/register`.
    pub fn register_content(
        &self,
        payload: &str,
        auth: &ServiceRequestAuth,
    ) -> Result<ServiceContentRegistration, SdkError> {
        let response = self.request("POST", "/v1/content/register", payload, Some(auth))?;
        expect_status(response.status, 201)?;
        Ok(ServiceContentRegistration {
            content_id: json_string_field(response.body.as_str(), "content_id")?,
            retention_class: json_string_field(response.body.as_str(), "retention_class")?,
            lifecycle_state: json_string_field(response.body.as_str(), "lifecycle_state")?,
            redaction_status: json_string_field(response.body.as_str(), "redaction_status")?,
        })
    }

    /// Expires one content record via `POST /v1/content/{id}/expire`.
    pub fn expire_content(
        &self,
        content_id: &str,
        auth: &ServiceRequestAuth,
    ) -> Result<ServiceContentStatus, SdkError> {
        let content_id = normalize_route_segment("content_id", content_id)?;
        let route = format!("/v1/content/{content_id}/expire");
        let response = self.request("POST", route.as_str(), "{}", Some(auth))?;
        expect_status(response.status, 200)?;
        Ok(ServiceContentStatus {
            content_id: json_string_field(response.body.as_str(), "content_id")?,
            lifecycle_state: json_string_field(response.body.as_str(), "lifecycle_state")?,
            redaction_status: json_string_field(response.body.as_str(), "redaction_status")?,
        })
    }

    /// Tombstones one content record via `POST /v1/content/{id}/tombstone`.
    pub fn tombstone_content(
        &self,
        content_id: &str,
        auth: &ServiceRequestAuth,
    ) -> Result<ServiceContentStatus, SdkError> {
        let content_id = normalize_route_segment("content_id", content_id)?;
        let route = format!("/v1/content/{content_id}/tombstone");
        let response = self.request("POST", route.as_str(), "{}", Some(auth))?;
        expect_status(response.status, 200)?;
        Ok(ServiceContentStatus {
            content_id: json_string_field(response.body.as_str(), "content_id")?,
            lifecycle_state: json_string_field(response.body.as_str(), "lifecycle_state")?,
            redaction_status: json_string_field(response.body.as_str(), "redaction_status")?,
        })
    }

    /// Queries one content lifecycle status via `GET /v1/content/{id}`.
    pub fn get_content(
        &self,
        content_id: &str,
        auth: &ServiceRequestAuth,
    ) -> Result<ServiceContentStatus, SdkError> {
        let content_id = normalize_route_segment("content_id", content_id)?;
        let route = format!("/v1/content/{content_id}");
        let response = self.request("GET", route.as_str(), "", Some(auth))?;
        expect_status(response.status, 200)?;
        Ok(ServiceContentStatus {
            content_id: json_string_field(response.body.as_str(), "content_id")?,
            lifecycle_state: json_string_field(response.body.as_str(), "lifecycle_state")?,
            redaction_status: json_string_field(response.body.as_str(), "redaction_status")?,
        })
    }

    /// Submits one bridge message via `POST /v1/bridge/submit`.
    pub fn submit_bridge_message(
        &self,
        payload: &str,
        auth: &ServiceRequestAuth,
    ) -> Result<ServiceBridgeSubmission, SdkError> {
        let response = self.request("POST", "/v1/bridge/submit", payload, Some(auth))?;
        expect_status(response.status, 202)?;
        Ok(ServiceBridgeSubmission {
            bridge_id: json_string_field(response.body.as_str(), "bridge_id")?,
            source_message_id: json_string_field(response.body.as_str(), "source_message_id")?,
            bridge_status: json_string_field(response.body.as_str(), "bridge_status")?,
        })
    }

    /// Forwards one submitted bridge message via `POST /v1/bridge/{id}/forward`.
    pub fn forward_bridge_message(
        &self,
        bridge_id: &str,
        auth: &ServiceRequestAuth,
    ) -> Result<ServiceBridgeStatus, SdkError> {
        let bridge_id = normalize_route_segment("bridge_id", bridge_id)?;
        let route = format!("/v1/bridge/{bridge_id}/forward");
        let response = self.request("POST", route.as_str(), "{}", Some(auth))?;
        expect_status(response.status, 200)?;
        Ok(ServiceBridgeStatus {
            bridge_id: json_string_field(response.body.as_str(), "bridge_id")?,
            bridge_status: json_string_field(response.body.as_str(), "bridge_status")?,
            target_message_id: json_string_field(response.body.as_str(), "target_message_id")?,
            forward_tx_hash: json_string_field(response.body.as_str(), "forward_tx_hash")?,
        })
    }

    /// Queries one bridge forwarding status via `GET /v1/bridge/{id}`.
    pub fn get_bridge_message(
        &self,
        bridge_id: &str,
        auth: &ServiceRequestAuth,
    ) -> Result<ServiceBridgeStatus, SdkError> {
        let bridge_id = normalize_route_segment("bridge_id", bridge_id)?;
        let route = format!("/v1/bridge/{bridge_id}");
        let response = self.request("GET", route.as_str(), "", Some(auth))?;
        expect_status(response.status, 200)?;
        Ok(ServiceBridgeStatus {
            bridge_id: json_string_field(response.body.as_str(), "bridge_id")?,
            bridge_status: json_string_field(response.body.as_str(), "bridge_status")?,
            target_message_id: json_string_field(response.body.as_str(), "target_message_id")?,
            forward_tx_hash: json_string_field(response.body.as_str(), "forward_tx_hash")?,
        })
    }

    /// Queries an agent reputation/profile by DID.
    pub fn get_agent_profile(
        &self,
        did: &str,
        auth: &ServiceRequestAuth,
    ) -> Result<ServiceAgentProfile, SdkError> {
        let did = normalize_route_segment("did", did)?;
        let route = format!("/v1/agents/{did}");
        let response = self.request("GET", route.as_str(), "", Some(auth))?;
        expect_status(response.status, 200)?;
        Ok(ServiceAgentProfile {
            did: json_string_field(response.body.as_str(), "did")?,
            reputation_score: json_u64_field(response.body.as_str(), "reputation_score")?,
        })
    }

    /// Queries service health route without request auth.
    pub fn health(&self) -> Result<ServiceHealthStatus, SdkError> {
        let response = self.request("GET", "/healthz", "", None)?;
        expect_status(response.status, 200)?;
        Ok(ServiceHealthStatus {
            status: json_string_field(response.body.as_str(), "status")?,
            runtime_mode: json_string_field(response.body.as_str(), "runtime_mode")?,
            role: json_string_field(response.body.as_str(), "role")?,
            observability_source: json_string_field(
                response.body.as_str(),
                "observability_source",
            )?,
            observability_health: json_string_field(
                response.body.as_str(),
                "observability_health",
            )?,
        })
    }

    /// Reads raw prometheus metrics exposition text.
    pub fn metrics(&self) -> Result<String, SdkError> {
        let response = self.request("GET", "/metrics", "", None)?;
        expect_status(response.status, 200)?;
        Ok(response.body)
    }

    /// Performs WebSocket upgrade on `/v1/events/ws` and reads one event frame.
    pub fn read_event_once(
        &self,
        auth: &ServiceRequestAuth,
    ) -> Result<ServiceRouteEvent, SdkError> {
        let mut stream = self.endpoint.connect_stream()?;
        let route = self.endpoint.route_path(SERVICE_WS_ROUTE);
        validate_request_path(route.as_str())?;
        let authority = format!("{}:{}", self.endpoint.host, self.endpoint.port);
        validate_http_header_value("service.endpoint.authority", authority.as_str())?;
        let auth_headers = render_auth_headers(Some(auth))?;
        let request = format!(
            "GET {route} HTTP/1.1\r\nHost: {authority}\r\nConnection: Upgrade\r\nUpgrade: websocket\r\nSec-WebSocket-Key: kamn-sdk-test-key\r\nSec-WebSocket-Version: 13\r\n{auth_headers}Content-Length: 0\r\n\r\n",
        );
        write_and_flush_request(
            &mut stream,
            request.as_bytes(),
            "failed to write service websocket request",
        )?;

        let response_bytes = read_response_bytes(&mut stream)?;
        let header_end = response_bytes
            .windows(4)
            .position(|window| window == b"\r\n\r\n")
            .map(|index| index + 4)
            .ok_or(SdkError::TransportFailure(
                "service websocket response missing header terminator",
            ))?;
        let header_text =
            String::from_utf8(response_bytes[..header_end].to_vec()).map_err(|_| {
                SdkError::TransportFailure("service websocket response header not utf-8")
            })?;
        if !header_text.starts_with("HTTP/1.1 101") {
            let body = String::from_utf8(response_bytes[header_end..].to_vec()).unwrap_or_default();
            return map_non_success_response(
                status_from_header(header_text.as_str()),
                body.as_str(),
            );
        }

        let frame = &response_bytes[header_end..];
        let payload_bytes = parse_unmasked_text_frame_payload(frame)?;
        let payload = String::from_utf8(payload_bytes.to_vec())
            .map_err(|_| SdkError::TransportFailure("service websocket event payload not utf-8"))?;
        Ok(ServiceRouteEvent {
            event: json_string_field(payload.as_str(), "event")?,
            runtime_mode: json_string_field(payload.as_str(), "runtime_mode")?,
            role: json_string_field(payload.as_str(), "role")?,
            sequence: json_u64_field(payload.as_str(), "sequence")?,
        })
    }

    fn request(
        &self,
        method: &str,
        route: &str,
        body: &str,
        auth: Option<&ServiceRequestAuth>,
    ) -> Result<HttpResponse, SdkError> {
        let mut stream = self.endpoint.connect_stream()?;
        validate_request_method(method)?;
        let path = self.endpoint.route_path(route);
        validate_request_path(path.as_str())?;
        let authority = format!("{}:{}", self.endpoint.host, self.endpoint.port);
        validate_http_header_value("service.endpoint.authority", authority.as_str())?;
        let auth_headers = render_auth_headers(auth)?;
        let request = format!(
            "{method} {path} HTTP/1.1\r\nHost: {authority}\r\nContent-Type: application/json\r\n{auth_headers}Content-Length: {}\r\nConnection: close\r\n\r\n{}",
            body.len(),
            body
        );
        write_and_flush_request(
            &mut stream,
            request.as_bytes(),
            "failed to write service request",
        )?;

        let response_text = read_response_text(&mut stream)?;
        parse_http_response(response_text.as_str())
    }
}

fn write_and_flush_request<W: Write>(
    stream: &mut W,
    payload: &[u8],
    failure_reason: &'static str,
) -> Result<(), SdkError> {
    stream
        .write_all(payload)
        .map_err(|error| map_stream_write_error(&error, failure_reason))?;
    stream
        .flush()
        .map_err(|error| map_stream_write_error(&error, failure_reason))?;
    Ok(())
}

fn parse_host_port(authority: &str, default_port: u16) -> Result<(String, u16), SdkError> {
    if authority.starts_with('[') {
        let closing = authority.find(']').ok_or(SdkError::InvalidInput {
            field: "service.endpoint",
            reason: "unterminated ipv6 host",
        })?;
        let host = authority[..=closing].to_owned();
        validate_endpoint_host(host.as_str())?;
        let suffix = &authority[closing + 1..];
        if suffix.is_empty() {
            return Ok((host, default_port));
        }
        let port = suffix
            .strip_prefix(':')
            .ok_or(SdkError::InvalidInput {
                field: "service.endpoint",
                reason: "invalid ipv6 authority suffix",
            })?
            .parse::<u16>()
            .map_err(|_| SdkError::InvalidInput {
                field: "service.endpoint",
                reason: "port must be an unsigned integer in range",
            })?;
        return Ok((host, port));
    }

    match authority.rsplit_once(':') {
        Some((host, raw_port)) if !host.is_empty() => {
            validate_endpoint_host(host)?;
            let port = raw_port
                .parse::<u16>()
                .map_err(|_| SdkError::InvalidInput {
                    field: "service.endpoint",
                    reason: "port must be an unsigned integer in range",
                })?;
            Ok((host.to_owned(), port))
        }
        _ => {
            validate_endpoint_host(authority)?;
            Ok((authority.to_owned(), default_port))
        }
    }
}

fn normalize_route_segment(field: &'static str, value: &str) -> Result<String, SdkError> {
    let normalized = value.trim();
    if normalized.is_empty() {
        return Err(SdkError::InvalidInput {
            field,
            reason: "must not be empty",
        });
    }
    if normalized
        .bytes()
        .any(|byte| !matches!(byte, b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' | b':'))
    {
        return Err(SdkError::InvalidInput {
            field,
            reason: "contains characters not allowed in route segment",
        });
    }
    Ok(normalized.to_owned())
}

fn validate_http_header_value(field: &'static str, value: &str) -> Result<(), SdkError> {
    if value.bytes().any(|byte| !matches!(byte, 0x20..=0x7e)) {
        return Err(SdkError::InvalidInput {
            field,
            reason: "contains invalid http header characters",
        });
    }
    Ok(())
}

fn validate_endpoint_host(host: &str) -> Result<(), SdkError> {
    if host.trim().is_empty() {
        return Err(SdkError::InvalidInput {
            field: "service.endpoint",
            reason: "host is required",
        });
    }
    if host
        .bytes()
        .any(|byte| byte <= 0x20 || byte == 0x7f || matches!(byte, b'/' | b'\\' | b'?' | b'#'))
    {
        return Err(SdkError::InvalidInput {
            field: "service.endpoint",
            reason: "host contains invalid characters",
        });
    }
    Ok(())
}

fn validate_request_method(method: &str) -> Result<(), SdkError> {
    if method.bytes().all(|byte| byte.is_ascii_uppercase()) {
        return Ok(());
    }
    Err(SdkError::InvalidInput {
        field: "service.request_method",
        reason: "contains invalid http method characters",
    })
}

fn validate_request_path(path: &str) -> Result<(), SdkError> {
    if !path.starts_with('/') {
        return Err(SdkError::InvalidInput {
            field: "service.request_path",
            reason: "must start with '/'",
        });
    }
    if path
        .bytes()
        .any(|byte| byte <= 0x20 || byte == 0x7f || matches!(byte, b'?' | b'#'))
    {
        return Err(SdkError::InvalidInput {
            field: "service.request_path",
            reason: "contains invalid path characters",
        });
    }
    Ok(())
}

fn render_auth_headers(auth: Option<&ServiceRequestAuth>) -> Result<String, SdkError> {
    let Some(auth) = auth else {
        return Ok(String::new());
    };
    validate_http_header_value("request_auth.sender_did", auth.sender_did().as_str())?;
    validate_http_header_value("request_auth.signature", auth.signature())?;
    let mut headers = String::new();
    headers.push_str(
        format!(
            "{REQUEST_AUTH_SENDER_DID_HEADER}: {}\r\n",
            auth.sender_did().as_str()
        )
        .as_str(),
    );
    headers.push_str(format!("{REQUEST_AUTH_NONCE_HEADER}: {}\r\n", auth.nonce()).as_str());
    headers.push_str(format!("{REQUEST_AUTH_SIGNATURE_HEADER}: {}\r\n", auth.signature()).as_str());
    if let Some(signer_public_key_hex) = auth.signer_public_key_hex() {
        validate_http_header_value("request_auth.signer_public_key", signer_public_key_hex)?;
        headers.push_str(
            format!("{REQUEST_AUTH_SIGNER_PUBLIC_KEY_HEADER}: {signer_public_key_hex}\r\n",)
                .as_str(),
        );
    }
    if let Some(scope) = auth.scope() {
        validate_http_header_value("request_auth.scope", scope)?;
        headers.push_str(format!("{REQUEST_AUTH_SCOPE_HEADER}: {scope}\r\n").as_str());
    }
    Ok(headers)
}

fn read_response_bytes<R: Read>(stream: &mut R) -> Result<Vec<u8>, SdkError> {
    let mut response = Vec::new();
    let mut chunk = [0_u8; 1024];
    let mut read_iterations = 0_usize;
    loop {
        if read_iterations >= MAX_SERVICE_RESPONSE_READ_ITERATIONS {
            return Err(SdkError::TransportFailure(
                SERVICE_RESPONSE_READ_ITERATION_LIMIT_EXCEEDED,
            ));
        }
        match stream.read(&mut chunk) {
            Ok(0) => break,
            Ok(read_count) => {
                read_iterations = read_iterations.saturating_add(1);
                response.extend_from_slice(&chunk[..read_count]);
                if response.len() > MAX_SERVICE_RESPONSE_BYTES {
                    return Err(SdkError::TransportFailure(
                        SERVICE_RESPONSE_SIZE_LIMIT_EXCEEDED,
                    ));
                }
            }
            Err(error) if matches!(error.kind(), ErrorKind::WouldBlock | ErrorKind::TimedOut) => {
                break;
            }
            Err(error)
                if matches!(error.kind(), ErrorKind::UnexpectedEof) && !response.is_empty() =>
            {
                break;
            }
            Err(error) => {
                return Err(map_stream_read_error(
                    &error,
                    "failed to read service response payload",
                ));
            }
        }
    }
    Ok(response)
}

fn read_response_text<R: Read>(stream: &mut R) -> Result<String, SdkError> {
    String::from_utf8(read_response_bytes(stream)?)
        .map_err(|_| SdkError::TransportFailure("service response payload was not utf-8"))
}

#[cfg(test)]
mod tests {
    use super::{
        parse_unmasked_text_frame_payload, read_response_bytes, resolve_request_timeout_seconds,
        service_public_key_for_private_key, service_signature_for_state_hash_with_private_key,
        service_verify_signature_with_public_key, write_and_flush_request, SdkError,
        MAX_SERVICE_RESPONSE_BYTES, REQUEST_TIMEOUT_SECONDS_ENV,
        SERVICE_RESPONSE_READ_ITERATION_LIMIT_EXCEEDED, SERVICE_RESPONSE_SIZE_LIMIT_EXCEEDED,
    };
    use crate::AgentDid;
    use std::collections::VecDeque;
    use std::ffi::OsString;
    use std::io::{ErrorKind, Read, Write};
    use std::sync::{Mutex, OnceLock};

    enum ReadStep {
        Bytes(Vec<u8>),
        Error(ErrorKind),
        RepeatByte(u8),
        Eof,
    }

    struct ScriptedReader {
        steps: VecDeque<ReadStep>,
    }

    impl ScriptedReader {
        fn new(steps: impl IntoIterator<Item = ReadStep>) -> Self {
            Self {
                steps: steps.into_iter().collect(),
            }
        }
    }

    impl Read for ScriptedReader {
        fn read(&mut self, buffer: &mut [u8]) -> std::io::Result<usize> {
            if buffer.is_empty() {
                return Ok(0);
            }
            let Some(step) = self.steps.front_mut() else {
                return Ok(0);
            };
            match step {
                ReadStep::Bytes(bytes) => {
                    let read_count = buffer.len().min(bytes.len());
                    buffer[..read_count].copy_from_slice(&bytes[..read_count]);
                    bytes.drain(..read_count);
                    if bytes.is_empty() {
                        let _ = self.steps.pop_front();
                    }
                    Ok(read_count)
                }
                ReadStep::Error(kind) => {
                    let kind = *kind;
                    let _ = self.steps.pop_front();
                    Err(std::io::Error::from(kind))
                }
                ReadStep::RepeatByte(byte) => {
                    buffer[0] = *byte;
                    Ok(1)
                }
                ReadStep::Eof => {
                    let _ = self.steps.pop_front();
                    Ok(0)
                }
            }
        }
    }

    struct RecordingWriter {
        bytes: Vec<u8>,
        flush_calls: u64,
        fail_flush: bool,
    }

    struct EnvVarGuard {
        key: &'static str,
        previous: Option<OsString>,
    }

    impl EnvVarGuard {
        fn set_utf8(key: &'static str, value: &str) -> Self {
            let previous = std::env::var_os(key);
            // SAFETY: tests serialize env writes with test-local guard lifetime.
            unsafe { std::env::set_var(key, value) };
            Self { key, previous }
        }

        fn unset(key: &'static str) -> Self {
            let previous = std::env::var_os(key);
            // SAFETY: tests serialize env writes with test-local guard lifetime.
            unsafe { std::env::remove_var(key) };
            Self { key, previous }
        }
    }

    impl Drop for EnvVarGuard {
        fn drop(&mut self) {
            match self.previous.as_ref() {
                Some(previous) => {
                    // SAFETY: restoring process env value captured at guard construction.
                    unsafe { std::env::set_var(self.key, previous) };
                }
                None => {
                    // SAFETY: restoring process env to prior absent state.
                    unsafe { std::env::remove_var(self.key) };
                }
            }
        }
    }

    fn timeout_env_lock() -> &'static Mutex<()> {
        static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOCK.get_or_init(|| Mutex::new(()))
    }

    impl RecordingWriter {
        fn with_flush_failure(fail_flush: bool) -> Self {
            Self {
                bytes: Vec::new(),
                flush_calls: 0,
                fail_flush,
            }
        }
    }

    impl Write for RecordingWriter {
        fn write(&mut self, buffer: &[u8]) -> std::io::Result<usize> {
            self.bytes.extend_from_slice(buffer);
            Ok(buffer.len())
        }

        fn flush(&mut self) -> std::io::Result<()> {
            self.flush_calls = self.flush_calls.saturating_add(1);
            if self.fail_flush {
                return Err(std::io::Error::new(
                    ErrorKind::BrokenPipe,
                    "synthetic flush failure",
                ));
            }
            Ok(())
        }
    }

    #[test]
    fn unit_write_and_flush_request_invokes_flush_on_success_path() {
        let mut writer = RecordingWriter::with_flush_failure(false);
        let payload = b"GET /healthz HTTP/1.1\r\n\r\n";
        write_and_flush_request(&mut writer, payload, "failed to write service request")
            .expect("write and flush should succeed");
        assert_eq!(writer.bytes, payload);
        assert_eq!(
            writer.flush_calls, 1,
            "flush should be invoked exactly once"
        );
    }

    #[test]
    fn regression_issue_6208_request_timeout_defaults_when_env_missing() {
        let _lock = timeout_env_lock().lock().expect("lock poisoned");
        let _guard = EnvVarGuard::unset(REQUEST_TIMEOUT_SECONDS_ENV);
        assert_eq!(resolve_request_timeout_seconds(), Ok(2));
    }

    #[test]
    fn regression_issue_6208_request_timeout_accepts_configured_positive_value() {
        let _lock = timeout_env_lock().lock().expect("lock poisoned");
        let _guard = EnvVarGuard::set_utf8(REQUEST_TIMEOUT_SECONDS_ENV, "7");
        assert_eq!(resolve_request_timeout_seconds(), Ok(7));
    }

    #[test]
    fn regression_issue_6208_request_timeout_rejects_zero_or_non_numeric_values() {
        let _lock = timeout_env_lock().lock().expect("lock poisoned");
        let _guard = EnvVarGuard::set_utf8(REQUEST_TIMEOUT_SECONDS_ENV, "0");
        assert_eq!(
            resolve_request_timeout_seconds(),
            Err(SdkError::InvalidInput {
                field: "service.request_timeout_seconds",
                reason: "must be greater than zero",
            })
        );

        let _guard = EnvVarGuard::set_utf8(REQUEST_TIMEOUT_SECONDS_ENV, "fast");
        assert_eq!(
            resolve_request_timeout_seconds(),
            Err(SdkError::InvalidInput {
                field: "service.request_timeout_seconds",
                reason: "must be valid integer seconds",
            })
        );
    }

    #[test]
    fn regression_write_and_flush_request_propagates_flush_failure() {
        // Regression: #5953
        let mut writer = RecordingWriter::with_flush_failure(true);
        let error = write_and_flush_request(&mut writer, b"{}", "failed to write service request")
            .expect_err("flush failure should fail closed");
        assert_eq!(
            writer.flush_calls, 1,
            "flush failure path must invoke flush"
        );
        assert_eq!(
            error,
            SdkError::TransportFailure("failed to write service request")
        );
    }

    #[test]
    fn regression_read_response_bytes_allows_partial_payload_before_unexpected_eof() {
        // Regression: #5953
        let payload = b"HTTP/1.1 200 OK\r\nContent-Length: 2\r\n\r\n{}";
        let mut reader = ScriptedReader::new([
            ReadStep::Bytes(payload.to_vec()),
            ReadStep::Error(ErrorKind::UnexpectedEof),
            ReadStep::Eof,
        ]);
        let response = read_response_bytes(&mut reader)
            .expect("partial payload should be preserved across unexpected eof");
        assert_eq!(response, payload);
    }

    #[test]
    fn regression_read_response_bytes_rejects_unexpected_eof_without_payload() {
        // Regression: #5953
        let mut reader = ScriptedReader::new([ReadStep::Error(ErrorKind::UnexpectedEof)]);
        let error = read_response_bytes(&mut reader)
            .expect_err("unexpected eof without payload should fail closed");
        assert_eq!(
            error,
            SdkError::TransportFailure("failed to read service response payload")
        );
    }

    #[test]
    fn regression_read_response_bytes_fails_closed_when_iteration_budget_exceeded() {
        // Regression: #5953
        let mut reader = ScriptedReader::new([ReadStep::RepeatByte(b'x')]);
        let error = read_response_bytes(&mut reader)
            .expect_err("pathological single-byte stream should exceed iteration budget");
        assert_eq!(
            error,
            SdkError::TransportFailure(SERVICE_RESPONSE_READ_ITERATION_LIMIT_EXCEEDED)
        );
    }

    #[test]
    fn unit_read_response_bytes_accepts_payload_at_exact_size_limit() {
        let payload = vec![b'a'; MAX_SERVICE_RESPONSE_BYTES];
        let mut reader = ScriptedReader::new([ReadStep::Bytes(payload.clone()), ReadStep::Eof]);
        let response =
            read_response_bytes(&mut reader).expect("exact limit payload should be accepted");
        assert_eq!(response.len(), MAX_SERVICE_RESPONSE_BYTES);
        assert_eq!(response, payload);
    }

    #[test]
    fn regression_read_response_bytes_rejects_payload_exceeding_size_limit() {
        // Regression: #5953
        let payload = vec![b'b'; MAX_SERVICE_RESPONSE_BYTES.saturating_add(1)];
        let mut reader = ScriptedReader::new([ReadStep::Bytes(payload), ReadStep::Eof]);
        let error = read_response_bytes(&mut reader)
            .expect_err("payloads exceeding size limit should fail closed");
        assert_eq!(
            error,
            SdkError::TransportFailure(SERVICE_RESPONSE_SIZE_LIMIT_EXCEEDED)
        );
    }

    fn websocket_text_frame(payload: &[u8], length_mode: u8) -> Vec<u8> {
        let mut frame = Vec::new();
        frame.push(0x81);
        match length_mode {
            125 => {
                frame.push(payload.len() as u8);
            }
            126 => {
                frame.push(126);
                frame.extend_from_slice(&(payload.len() as u16).to_be_bytes());
            }
            127 => {
                frame.push(127);
                frame.extend_from_slice(&(payload.len() as u64).to_be_bytes());
            }
            _ => panic!("unsupported test length mode"),
        }
        frame.extend_from_slice(payload);
        frame
    }

    #[test]
    fn unit_parse_websocket_frame_payload_accepts_16bit_extended_length() {
        let payload = vec![b'a'; 130];
        let frame = websocket_text_frame(payload.as_slice(), 126);
        let parsed = parse_unmasked_text_frame_payload(frame.as_slice())
            .expect("16-bit extended websocket payload should parse");
        assert_eq!(parsed, payload.as_slice());
    }

    #[test]
    fn unit_parse_websocket_frame_payload_accepts_64bit_extended_length() {
        let payload = vec![b'b'; 130];
        let frame = websocket_text_frame(payload.as_slice(), 127);
        let parsed = parse_unmasked_text_frame_payload(frame.as_slice())
            .expect("64-bit extended websocket payload should parse");
        assert_eq!(parsed, payload.as_slice());
    }

    #[test]
    fn regression_parse_websocket_frame_payload_rejects_truncated_extended_header() {
        // Regression: #6190
        let frame = vec![0x81, 126, 0x00];
        let error = parse_unmasked_text_frame_payload(frame.as_slice())
            .expect_err("truncated extended-length header must fail closed");
        assert_eq!(
            error,
            SdkError::TransportFailure("service websocket response frame payload truncated")
        );
    }

    const TEST_PRIVATE_KEY_HEX: &str =
        "658c3528422eb527b4c108b8f6d1e5f629543c304ea49cf608c67794424291c4";

    #[test]
    fn unit_service_public_key_for_private_key_derives_compressed_hex_key() {
        let public_key = service_public_key_for_private_key(TEST_PRIVATE_KEY_HEX)
            .expect("valid private key should derive signer public key");
        assert_eq!(public_key.len(), 66);
        assert!(
            public_key.starts_with("02") || public_key.starts_with("03"),
            "compressed secp256k1 key should start with 02 or 03"
        );
    }

    #[test]
    fn regression_service_public_key_for_private_key_rejects_invalid_private_key_hex() {
        // Regression: #5977
        let error = service_public_key_for_private_key("not-a-private-key")
            .expect_err("invalid key material must fail closed");
        assert_eq!(
            error,
            SdkError::InvalidInput {
                field: "service.request_auth.private_key",
                reason: "must be valid secp256k1 private key hex",
            }
        );
    }

    #[test]
    fn unit_service_verify_signature_with_public_key_accepts_valid_signature() {
        let sender_did = AgentDid::parse("kamn:did:agent:alice").expect("sender did should parse");
        let state_hash = "service-api:kamn-sdk:1";
        let body = r#"{"message":"hello"}"#;
        let signature = service_signature_for_state_hash_with_private_key(
            &sender_did,
            7,
            state_hash,
            body,
            TEST_PRIVATE_KEY_HEX,
        )
        .expect("signature should be produced");
        let signer_public_key = service_public_key_for_private_key(TEST_PRIVATE_KEY_HEX)
            .expect("public key should be derived");
        service_verify_signature_with_public_key(
            &sender_did,
            7,
            state_hash,
            body,
            signature.as_str(),
            signer_public_key.as_str(),
        )
        .expect("valid signature should verify");
    }

    #[test]
    fn regression_service_verify_signature_with_public_key_rejects_invalid_public_key_hex() {
        // Regression: #5977
        let sender_did = AgentDid::parse("kamn:did:agent:alice").expect("sender did should parse");
        let signature = service_signature_for_state_hash_with_private_key(
            &sender_did,
            8,
            "service-api:kamn-sdk:1",
            "{}",
            TEST_PRIVATE_KEY_HEX,
        )
        .expect("signature should be produced");
        let error = service_verify_signature_with_public_key(
            &sender_did,
            8,
            "service-api:kamn-sdk:1",
            "{}",
            signature.as_str(),
            "invalid-public-key",
        )
        .expect_err("invalid public key should fail closed");
        assert_eq!(
            error,
            SdkError::InvalidInput {
                field: "service.request_auth.expected_public_key",
                reason: "must be valid compressed secp256k1 public key hex",
            }
        );
    }

    #[test]
    fn regression_service_verify_signature_with_public_key_rejects_empty_state_hash() {
        // Regression: #5977
        let sender_did = AgentDid::parse("kamn:did:agent:alice").expect("sender did should parse");
        let signer_public_key = service_public_key_for_private_key(TEST_PRIVATE_KEY_HEX)
            .expect("public key should be derived");
        let signature = service_signature_for_state_hash_with_private_key(
            &sender_did,
            9,
            "service-api:kamn-sdk:1",
            "{}",
            TEST_PRIVATE_KEY_HEX,
        )
        .expect("signature should be produced");
        let error = service_verify_signature_with_public_key(
            &sender_did,
            9,
            "",
            "{}",
            signature.as_str(),
            signer_public_key.as_str(),
        )
        .expect_err("empty state hash should fail closed");
        assert_eq!(
            error,
            SdkError::InvalidInput {
                field: "service.request_auth.state_hash",
                reason: "must not be empty",
            }
        );
    }

    #[test]
    fn regression_service_verify_signature_with_public_key_rejects_empty_signature() {
        // Regression: #5977
        let sender_did = AgentDid::parse("kamn:did:agent:alice").expect("sender did should parse");
        let signer_public_key = service_public_key_for_private_key(TEST_PRIVATE_KEY_HEX)
            .expect("public key should be derived");
        let error = service_verify_signature_with_public_key(
            &sender_did,
            10,
            "service-api:kamn-sdk:1",
            "{}",
            "",
            signer_public_key.as_str(),
        )
        .expect_err("empty signature should fail closed");
        assert_eq!(
            error,
            SdkError::InvalidInput {
                field: "service.request_auth.signature",
                reason: "must not be empty",
            }
        );
    }

    #[test]
    fn regression_service_verify_signature_with_public_key_rejects_non_positive_nonce() {
        // Regression: #5977
        let sender_did = AgentDid::parse("kamn:did:agent:alice").expect("sender did should parse");
        let signer_public_key = service_public_key_for_private_key(TEST_PRIVATE_KEY_HEX)
            .expect("public key should be derived");
        let signature = service_signature_for_state_hash_with_private_key(
            &sender_did,
            1,
            "service-api:kamn-sdk:1",
            "{}",
            TEST_PRIVATE_KEY_HEX,
        )
        .expect("signature should be produced");
        let error = service_verify_signature_with_public_key(
            &sender_did,
            0,
            "service-api:kamn-sdk:1",
            "{}",
            signature.as_str(),
            signer_public_key.as_str(),
        )
        .expect_err("non-positive nonce should fail closed");
        assert_eq!(
            error,
            SdkError::InvalidInput {
                field: "service.request_auth.nonce",
                reason: "must be greater than zero",
            }
        );
    }

    #[test]
    fn regression_service_verify_signature_with_public_key_rejects_tampered_signature() {
        // Regression: #5977
        let sender_did = AgentDid::parse("kamn:did:agent:alice").expect("sender did should parse");
        let signer_public_key = service_public_key_for_private_key(TEST_PRIVATE_KEY_HEX)
            .expect("public key should be derived");
        let mut signature = service_signature_for_state_hash_with_private_key(
            &sender_did,
            11,
            "service-api:kamn-sdk:1",
            "{}",
            TEST_PRIVATE_KEY_HEX,
        )
        .expect("signature should be produced");
        signature.push('f');
        let error = service_verify_signature_with_public_key(
            &sender_did,
            11,
            "service-api:kamn-sdk:1",
            "{}",
            signature.as_str(),
            signer_public_key.as_str(),
        )
        .expect_err("tampered signature should fail closed");
        assert_eq!(
            error,
            SdkError::InvalidInput {
                field: "service.request_auth.signature",
                reason: "failed cryptographic signature verification",
            }
        );
    }
}
