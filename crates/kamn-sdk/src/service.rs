use crate::SdkError;

#[path = "service_auth_crypto.rs"]
mod service_auth_crypto;
#[path = "service_endpoint.rs"]
mod service_endpoint;
#[path = "service_http_io.rs"]
mod service_http_io;
#[path = "service_models.rs"]
mod service_models;
#[path = "service_request_auth.rs"]
mod service_request_auth;
#[path = "service_response.rs"]
mod service_response;
#[path = "service_websocket.rs"]
mod service_websocket;
pub use self::service_auth_crypto::{
    service_public_key_for_private_key, service_signature_for_fields,
    service_signature_for_state_hash_with_private_key, service_signer_public_key_for_fields,
    service_verify_signature_with_public_key,
};
#[cfg(test)]
use self::service_endpoint::resolve_request_timeout_seconds;
use self::service_endpoint::ServiceEndpoint;
use self::service_http_io::{
    normalize_route_segment, read_response_bytes, read_response_text, render_auth_headers,
    validate_http_header_value, validate_request_method, validate_request_path,
    write_and_flush_request,
};
pub use self::service_models::{
    ServiceAgentProfile, ServiceBridgeStatus, ServiceBridgeSubmission, ServiceChannelMessages,
    ServiceChannelReceipt, ServiceContentRegistration, ServiceContentStatus, ServiceEscrowStatus,
    ServiceHealthStatus, ServiceMessageReceipt, ServiceMessageStatus, ServiceRouteEvent,
    ServiceTaskReceipt, ServiceTaskStatus,
};
pub use self::service_request_auth::ServiceRequestAuth;
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
        parse_escrow_status(response.body.as_str())
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
        parse_escrow_status(response.body.as_str())
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

fn parse_escrow_status(body: &str) -> Result<ServiceEscrowStatus, SdkError> {
    Ok(ServiceEscrowStatus {
        escrow_id: json_string_field(body, "escrow_id")?,
        state: json_string_field(body, "state")?,
    })
}

#[cfg(test)]
#[path = "service_tests.rs"]
mod tests;
