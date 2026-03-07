use crate::SdkError;

#[path = "service_auth_crypto.rs"]
mod service_auth_crypto;
#[path = "service_client.rs"]
mod service_client;
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
use self::service_client::HttpResponse;
pub use self::service_client::ServiceApiClient;
#[cfg(test)]
use self::service_endpoint::resolve_request_timeout_seconds;
use self::service_endpoint::ServiceEndpoint;
use self::service_http_io::{
    normalize_route_segment, read_response_bytes, read_response_text, render_auth_headers,
    validate_http_header_value, validate_request_method, validate_request_path,
    write_and_flush_request,
};
pub use self::service_models::{
    ServiceAgentBalance, ServiceAgentProfile, ServiceBridgeStatus, ServiceBridgeSubmission,
    ServiceChannelMessages, ServiceChannelReceipt, ServiceContentRegistration,
    ServiceContentStatus, ServiceEscrowStatus, ServiceHealthStatus, ServiceMessageReceipt,
    ServiceMessageStatus, ServiceRouteEvent, ServiceTaskReceipt, ServiceTaskStatus,
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

#[cfg(test)]
#[path = "service_tests.rs"]
mod tests;
