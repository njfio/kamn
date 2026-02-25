use crate::{AgentDid, SdkError};
use kamn_core::{
    service_auth_sign_with_private_key_hex, ServiceAuthSignatureError,
    SERVICE_AUTH_SIGNATURE_PRIVATE_KEY_ENV,
};
use rustls::pki_types::ServerName;
use rustls::{ClientConfig, ClientConnection, RootCertStore, StreamOwned};
use serde_json::Value;
use std::fs;
use std::io::{Cursor, ErrorKind, Read, Write};
use std::net::{IpAddr, TcpStream};
use std::sync::Arc;
use std::time::Duration;

const REQUEST_TIMEOUT_SECONDS: u64 = 2;
const SERVICE_TLS_CA_FILE_ENV: &str = "KAMN_SERVICE_API_TLS_CA_FILE";
const REQUEST_AUTH_SENDER_DID_HEADER: &str = "x-kamn-sender-did";
const REQUEST_AUTH_NONCE_HEADER: &str = "x-kamn-request-nonce";
const REQUEST_AUTH_SIGNATURE_HEADER: &str = "x-kamn-request-signature";
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
        let stream = TcpStream::connect((self.host.as_str(), self.port))
            .map_err(|_| SdkError::TransportFailure("failed to connect to service endpoint"))?;
        stream
            .set_read_timeout(Some(Duration::from_secs(REQUEST_TIMEOUT_SECONDS)))
            .map_err(|_| SdkError::TransportFailure("failed to configure service read timeout"))?;
        stream
            .set_write_timeout(Some(Duration::from_secs(REQUEST_TIMEOUT_SECONDS)))
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
    scope: Option<String>,
}

impl ServiceRequestAuth {
    /// Builds a validated request auth envelope.
    pub fn new(sender_did: AgentDid, nonce: u64, signature: String) -> Result<Self, SdkError> {
        Self::new_with_scope(sender_did, nonce, signature, None)
    }

    /// Builds a validated request auth envelope with optional auth scope marker.
    pub fn new_with_scope(
        sender_did: AgentDid,
        nonce: u64,
        signature: String,
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
        Ok(Self {
            sender_did,
            nonce,
            signature: normalized_signature.to_owned(),
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
        stream.write_all(request.as_bytes()).map_err(|error| {
            map_stream_write_error(&error, "failed to write service websocket request")
        })?;
        stream.flush().map_err(|error| {
            map_stream_write_error(&error, "failed to write service websocket request")
        })?;

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
        if frame.len() < 2 {
            return Err(SdkError::TransportFailure(
                "service websocket response missing event frame",
            ));
        }
        if frame[0] != 0x81 {
            return Err(SdkError::TransportFailure(
                "service websocket response frame opcode unsupported",
            ));
        }
        if frame[1] & 0x80 != 0 {
            return Err(SdkError::TransportFailure(
                "service websocket response frame unexpectedly masked",
            ));
        }
        let payload_len = (frame[1] & 0x7f) as usize;
        if frame.len() < payload_len + 2 {
            return Err(SdkError::TransportFailure(
                "service websocket response frame payload truncated",
            ));
        }
        let payload = String::from_utf8(frame[2..2 + payload_len].to_vec())
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
        stream
            .write_all(request.as_bytes())
            .map_err(|error| map_stream_write_error(&error, "failed to write service request"))?;
        stream
            .flush()
            .map_err(|error| map_stream_write_error(&error, "failed to write service request"))?;

        let response_text = read_response_text(&mut stream)?;
        parse_http_response(response_text.as_str())
    }
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
    if let Some(scope) = auth.scope() {
        validate_http_header_value("request_auth.scope", scope)?;
        headers.push_str(format!("{REQUEST_AUTH_SCOPE_HEADER}: {scope}\r\n").as_str());
    }
    Ok(headers)
}

fn read_response_bytes<R: Read>(stream: &mut R) -> Result<Vec<u8>, SdkError> {
    let mut response = Vec::new();
    let mut chunk = [0_u8; 1024];
    loop {
        match stream.read(&mut chunk) {
            Ok(0) => break,
            Ok(read_count) => response.extend_from_slice(&chunk[..read_count]),
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

fn parse_http_response(response: &str) -> Result<HttpResponse, SdkError> {
    let (header, body) = response
        .split_once("\r\n\r\n")
        .ok_or(SdkError::TransportFailure(
            "service response missing header terminator",
        ))?;
    let status = status_from_header(header).ok_or(SdkError::TransportFailure(
        "service response status line invalid",
    ))?;
    if status >= 400 {
        return map_non_success_response(Some(status), body);
    }

    Ok(HttpResponse {
        status,
        body: body.to_owned(),
    })
}

fn status_from_header(header: &str) -> Option<u16> {
    let line = header.lines().next()?;
    let raw_code = line.split_whitespace().nth(1)?;
    raw_code.parse::<u16>().ok()
}

fn map_non_success_response<T>(status: Option<u16>, body: &str) -> Result<T, SdkError> {
    if let Some(status_code) = status {
        if let Some((error, reason_code, message)) = parse_service_api_error_envelope(body)
            .or_else(|| parse_service_api_legacy_error_envelope(status_code, body))
        {
            return Err(SdkError::ServiceApiError {
                status: status_code,
                error,
                reason_code,
                message,
            });
        }
    }
    match status {
        Some(409) => Err(SdkError::Conflict("request rejected by service api")),
        Some(401) => Err(SdkError::TransportFailure(
            "request rejected by service api",
        )),
        Some(400) => Err(SdkError::TransportFailure(
            "request rejected by service api",
        )),
        Some(404) => Err(SdkError::NotFound {
            entity: "service-route",
            id: "requested-route".to_owned(),
        }),
        _ => Err(SdkError::TransportFailure(
            "request rejected by service api",
        )),
    }
}

fn parse_service_api_error_envelope(body: &str) -> Option<(String, String, String)> {
    let error = json_optional_string_field(body, "error")?;
    let reason_code = json_optional_string_field(body, "reason_code")?;
    let message = json_optional_string_field(body, "message")?;
    Some((error, reason_code, message))
}

fn parse_service_api_legacy_error_envelope(
    status: u16,
    body: &str,
) -> Option<(String, String, String)> {
    let error = json_optional_string_field(body, "error")?;
    let reason = json_optional_string_field(body, "reason")?;
    let reason_code =
        classify_legacy_service_api_reason_code(status, error.as_str(), reason.as_str()).to_owned();
    Some((error, reason_code, reason))
}

fn classify_legacy_service_api_reason_code(status: u16, error: &str, reason: &str) -> &'static str {
    if reason.contains(REQUEST_AUTH_SENDER_DID_HEADER) {
        return REASON_CODE_AUTH_SENDER_DID_HEADER_MISSING;
    }
    if reason.contains(REQUEST_AUTH_NONCE_HEADER) && reason.contains("missing required header") {
        return REASON_CODE_AUTH_NONCE_HEADER_MISSING;
    }
    if reason.contains("invalid request nonce header") {
        return REASON_CODE_AUTH_NONCE_INVALID;
    }
    if reason.contains("request nonce must be positive") {
        return REASON_CODE_AUTH_NONCE_NON_POSITIVE;
    }
    if reason.contains(REQUEST_AUTH_SIGNATURE_HEADER) && reason.contains("missing required header")
    {
        return REASON_CODE_AUTH_SIGNATURE_HEADER_MISSING;
    }
    if reason.contains("signature verification failed") {
        return REASON_CODE_AUTH_SIGNATURE_VERIFICATION_FAILED;
    }
    if reason.contains("replay") {
        return REASON_CODE_AUTH_REPLAY_NONCE_DETECTED;
    }
    if reason.contains("websocket upgrade required") {
        return REASON_CODE_WEBSOCKET_UPGRADE_REQUIRED;
    }
    match status {
        404 => REASON_CODE_ROUTE_NOT_FOUND,
        405 => REASON_CODE_METHOD_NOT_ALLOWED,
        401 if error == "unauthorized" => REASON_CODE_LEGACY_UNAUTHORIZED,
        409 => REASON_CODE_LEGACY_CONFLICT,
        400 => REASON_CODE_LEGACY_BAD_REQUEST,
        _ => REASON_CODE_LEGACY_UNKNOWN,
    }
}

fn expect_status(actual: u16, expected: u16) -> Result<(), SdkError> {
    if actual == expected {
        return Ok(());
    }
    if actual == 409 {
        return Err(SdkError::Conflict("request rejected by service api"));
    }
    Err(SdkError::TransportFailure(
        "request rejected by service api",
    ))
}

fn parse_json_root(payload: &str) -> Result<Value, SdkError> {
    serde_json::from_str(payload)
        .map_err(|_| SdkError::TransportFailure("service response payload was not valid json"))
}

fn json_string_field(payload: &str, key: &str) -> Result<String, SdkError> {
    let root = parse_json_root(payload)?;
    let value = root.get(key).ok_or(SdkError::TransportFailure(
        "service response missing required field",
    ))?;
    if let Some(parsed) = value.as_str() {
        return Ok(parsed.to_owned());
    }
    if let Some(parsed) = value.as_u64() {
        return Ok(parsed.to_string());
    }
    Err(SdkError::TransportFailure(
        "service response field was not a string",
    ))
}

fn json_u64_field(payload: &str, key: &str) -> Result<u64, SdkError> {
    let root = parse_json_root(payload)?;
    let value = root.get(key).ok_or(SdkError::TransportFailure(
        "service response missing required field",
    ))?;
    if let Some(parsed) = value.as_u64() {
        return Ok(parsed);
    }
    if let Some(parsed) = value.as_str().and_then(|raw| raw.parse::<u64>().ok()) {
        return Ok(parsed);
    }
    Err(SdkError::TransportFailure(
        "service response numeric field was malformed",
    ))
}

fn json_optional_string_field(payload: &str, key: &str) -> Option<String> {
    let root = serde_json::from_str::<Value>(payload).ok()?;
    let value = root.get(key)?;
    value.as_str().map(str::to_owned)
}

fn json_string_array_field(payload: &str, key: &str) -> Result<Vec<String>, SdkError> {
    let root = parse_json_root(payload)?;
    let value = root.get(key).ok_or(SdkError::TransportFailure(
        "service response missing required field",
    ))?;
    let items = value.as_array().ok_or(SdkError::TransportFailure(
        "service response array field was malformed",
    ))?;
    let mut parsed = Vec::with_capacity(items.len());
    for item in items {
        let value = item.as_str().ok_or(SdkError::TransportFailure(
            "service response array item was malformed",
        ))?;
        parsed.push(value.to_owned());
    }
    Ok(parsed)
}
