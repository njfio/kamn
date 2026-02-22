use crate::{signature_for_fields, AgentDid, SdkError};
use std::io::{ErrorKind, Read, Write};
use std::net::TcpStream;
use std::time::Duration;

const REQUEST_TIMEOUT_SECONDS: u64 = 2;
const REQUEST_AUTH_SENDER_DID_HEADER: &str = "x-kamn-sender-did";
const REQUEST_AUTH_NONCE_HEADER: &str = "x-kamn-request-nonce";
const REQUEST_AUTH_SIGNATURE_HEADER: &str = "x-kamn-request-signature";
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

    fn connect_stream(&self) -> Result<TcpStream, SdkError> {
        if self.scheme == ServiceScheme::Https {
            return Err(SdkError::NotImplemented(
                "https transport is not yet supported by the std service client",
            ));
        }

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
}

/// Request authentication envelope for service API routes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServiceRequestAuth {
    sender_did: AgentDid,
    nonce: u64,
    signature: String,
}

impl ServiceRequestAuth {
    /// Builds a validated request auth envelope.
    pub fn new(sender_did: AgentDid, nonce: u64, signature: String) -> Result<Self, SdkError> {
        if nonce == 0 {
            return Err(SdkError::InvalidInput {
                field: "request_auth.nonce",
                reason: "must be greater than zero",
            });
        }
        if signature.trim().is_empty() {
            return Err(SdkError::InvalidInput {
                field: "request_auth.signature",
                reason: "must not be empty",
            });
        }
        Ok(Self {
            sender_did,
            nonce,
            signature,
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
}

/// Deterministic request signature builder for service API fields.
pub fn service_signature_for_fields(
    sender_did: &AgentDid,
    nonce: u64,
    chain_id: &str,
    chain_version: &str,
    body: &str,
) -> String {
    let state_hash = format!("service-api:{chain_id}:{chain_version}");
    signature_for_fields(sender_did.as_str(), nonce, state_hash.as_str(), body)
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
        if message_id.trim().is_empty() {
            return Err(SdkError::InvalidInput {
                field: "message_id",
                reason: "must not be empty",
            });
        }
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
        if channel_id.trim().is_empty() {
            return Err(SdkError::InvalidInput {
                field: "channel_id",
                reason: "must not be empty",
            });
        }
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
        if task_id.trim().is_empty() {
            return Err(SdkError::InvalidInput {
                field: "task_id",
                reason: "must not be empty",
            });
        }
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
        if task_id.trim().is_empty() {
            return Err(SdkError::InvalidInput {
                field: "task_id",
                reason: "must not be empty",
            });
        }
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
        if task_id.trim().is_empty() {
            return Err(SdkError::InvalidInput {
                field: "task_id",
                reason: "must not be empty",
            });
        }
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
        if escrow_id.trim().is_empty() {
            return Err(SdkError::InvalidInput {
                field: "escrow_id",
                reason: "must not be empty",
            });
        }
        let route = format!("/v1/escrow/{escrow_id}/release");
        let response = self.request("POST", route.as_str(), "{}", Some(auth))?;
        expect_status(response.status, 200)?;
        Ok(ServiceEscrowStatus {
            escrow_id: json_string_field(response.body.as_str(), "escrow_id")?,
            state: json_string_field(response.body.as_str(), "state")?,
        })
    }

    /// Queries an agent reputation/profile by DID.
    pub fn get_agent_profile(
        &self,
        did: &str,
        auth: &ServiceRequestAuth,
    ) -> Result<ServiceAgentProfile, SdkError> {
        if did.trim().is_empty() {
            return Err(SdkError::InvalidInput {
                field: "did",
                reason: "must not be empty",
            });
        }
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
        let authority = format!("{}:{}", self.endpoint.host, self.endpoint.port);
        let request = format!(
            "GET {route} HTTP/1.1\r\nHost: {authority}\r\nConnection: Upgrade\r\nUpgrade: websocket\r\nSec-WebSocket-Key: kamn-sdk-test-key\r\nSec-WebSocket-Version: 13\r\n{}: {}\r\n{}: {}\r\n{}: {}\r\nContent-Length: 0\r\n\r\n",
            REQUEST_AUTH_SENDER_DID_HEADER,
            auth.sender_did().as_str(),
            REQUEST_AUTH_NONCE_HEADER,
            auth.nonce(),
            REQUEST_AUTH_SIGNATURE_HEADER,
            auth.signature(),
        );
        stream
            .write_all(request.as_bytes())
            .map_err(|_| SdkError::TransportFailure("failed to write service websocket request"))?;

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
        let path = self.endpoint.route_path(route);
        let authority = format!("{}:{}", self.endpoint.host, self.endpoint.port);
        let mut auth_headers = String::new();
        if let Some(auth) = auth {
            auth_headers.push_str(
                format!(
                    "{REQUEST_AUTH_SENDER_DID_HEADER}: {}\r\n",
                    auth.sender_did().as_str()
                )
                .as_str(),
            );
            auth_headers
                .push_str(format!("{REQUEST_AUTH_NONCE_HEADER}: {}\r\n", auth.nonce()).as_str());
            auth_headers.push_str(
                format!("{REQUEST_AUTH_SIGNATURE_HEADER}: {}\r\n", auth.signature()).as_str(),
            );
        }
        let request = format!(
            "{method} {path} HTTP/1.1\r\nHost: {authority}\r\nContent-Type: application/json\r\n{auth_headers}Content-Length: {}\r\nConnection: close\r\n\r\n{}",
            body.len(),
            body
        );
        stream
            .write_all(request.as_bytes())
            .map_err(|_| SdkError::TransportFailure("failed to write service request"))?;

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
            let port = raw_port
                .parse::<u16>()
                .map_err(|_| SdkError::InvalidInput {
                    field: "service.endpoint",
                    reason: "port must be an unsigned integer in range",
                })?;
            Ok((host.to_owned(), port))
        }
        _ => Ok((authority.to_owned(), default_port)),
    }
}

fn read_response_bytes(stream: &mut TcpStream) -> Result<Vec<u8>, SdkError> {
    let mut response = Vec::new();
    let mut chunk = [0_u8; 1024];
    loop {
        match stream.read(&mut chunk) {
            Ok(0) => break,
            Ok(read_count) => response.extend_from_slice(&chunk[..read_count]),
            Err(error) if matches!(error.kind(), ErrorKind::WouldBlock | ErrorKind::TimedOut) => {
                break;
            }
            Err(_) => {
                return Err(SdkError::TransportFailure(
                    "failed to read service response payload",
                ));
            }
        }
    }
    Ok(response)
}

fn read_response_text(stream: &mut TcpStream) -> Result<String, SdkError> {
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

fn json_string_field(payload: &str, key: &str) -> Result<String, SdkError> {
    let marker = format!("\"{key}\":\"");
    let start = payload
        .find(marker.as_str())
        .map(|index| index + marker.len())
        .ok_or(SdkError::TransportFailure(
            "service response missing required field",
        ))?;
    let rest = &payload[start..];
    let end = rest.find('"').ok_or(SdkError::TransportFailure(
        "service response field was not terminated",
    ))?;
    Ok(rest[..end].to_owned())
}

fn json_u64_field(payload: &str, key: &str) -> Result<u64, SdkError> {
    let marker = format!("\"{key}\":");
    let start = payload
        .find(marker.as_str())
        .map(|index| index + marker.len())
        .ok_or(SdkError::TransportFailure(
            "service response missing required field",
        ))?;
    let rest = payload[start..].trim_start();
    let value_end = rest
        .find(|ch: char| !ch.is_ascii_digit())
        .unwrap_or(rest.len());
    if value_end == 0 {
        return Err(SdkError::TransportFailure(
            "service response numeric field was malformed",
        ));
    }
    rest[..value_end]
        .parse::<u64>()
        .map_err(|_| SdkError::TransportFailure("service response numeric field was malformed"))
}

fn json_optional_string_field(payload: &str, key: &str) -> Option<String> {
    let marker = format!("\"{key}\":\"");
    let start = payload.find(marker.as_str())? + marker.len();
    let rest = &payload[start..];
    let end = rest.find('"')?;
    Some(rest[..end].to_owned())
}

fn json_string_array_field(payload: &str, key: &str) -> Result<Vec<String>, SdkError> {
    let marker = format!("\"{key}\":[");
    let start = payload
        .find(marker.as_str())
        .map(|index| index + marker.len())
        .ok_or(SdkError::TransportFailure(
            "service response missing required field",
        ))?;
    let rest = &payload[start..];
    let end = rest.find(']').ok_or(SdkError::TransportFailure(
        "service response array field was not terminated",
    ))?;
    let raw_items = rest[..end].trim();
    if raw_items.is_empty() {
        return Ok(Vec::new());
    }

    raw_items
        .split(',')
        .map(|item| {
            let trimmed = item.trim();
            if !(trimmed.starts_with('"') && trimmed.ends_with('"') && trimmed.len() >= 2) {
                return Err(SdkError::TransportFailure(
                    "service response array item was malformed",
                ));
            }
            Ok(trimmed[1..trimmed.len() - 1].to_owned())
        })
        .collect()
}
