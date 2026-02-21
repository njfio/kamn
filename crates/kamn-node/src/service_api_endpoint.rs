mod auth;
mod middleware_impl;
mod payload;
mod scope_fixture;
mod server;
mod websocket;

use crate::{
    logging::{log_info, log_warn},
    NodeBootstrapReport,
};
use axum::{
    body::{to_bytes, Body, Bytes},
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Request, State,
    },
    http::{HeaderMap, HeaderValue, StatusCode},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::{any, get},
    Extension, Router,
};
use kamn_core::{
    cross_store_replay_reason_codes_csv, cross_store_replay_reason_taxonomy_version,
    data_layer_m9_gateway_project_presence_event, signature_matches_supported_profile_for_fields,
    AgentDid, AntiSpamConfig, AntiSpamDecision, AntiSpamEngine, AntiSpamRejection,
    DataLayerM9GatewayBridgeError, DataLayerM9GatewayPresenceProjectionRequest,
    DataLayerM9PresenceConnectRequest, DataLayerM9PresenceQuery, DataLayerM9RealtimeDeliveryError,
    DataLayerM9RealtimeDeliveryRegistry, DATA_LAYER_M9_OWNER_SCOPE_DENIED_REASON_CODE,
    DATA_LAYER_M9_PRESENCE_VISIBILITY_DENIED_REASON_CODE,
};
#[cfg(test)]
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::fs;
use std::io::BufReader;
use std::net::SocketAddr;
use std::sync::{
    atomic::{AtomicBool, AtomicU64, Ordering},
    Arc,
};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tokio::runtime::Builder;
use tokio::sync::{Mutex, Notify, Semaphore};

pub(crate) const DEFAULT_SERVICE_API_MAX_REQUESTS: u64 = 1;
pub(crate) const DEFAULT_SERVICE_API_IDLE_TIMEOUT_MS: u64 = 5_000;
pub(crate) const DEFAULT_SERVICE_API_BODY_LIMIT_BYTES: u64 = 64 * 1024;
pub(crate) const DEFAULT_SERVICE_API_CONCURRENCY_LIMIT: u64 = 32;
pub(crate) const DEFAULT_SERVICE_API_RATE_LIMIT_PER_SECOND: u64 = 120;

const ROUTE_MESSAGES_SEND: &str = "/v1/messages/send";
const ROUTE_CHANNELS_CREATE: &str = "/v1/channels/create";
const ROUTE_TASKS_CREATE: &str = "/v1/tasks/create";
const ROUTE_MESSAGES_PREFIX: &str = "/v1/messages/";
const ROUTE_CHANNELS_PREFIX: &str = "/v1/channels/";
const ROUTE_CHANNELS_MESSAGES_SUFFIX: &str = "/messages";
const ROUTE_TASKS_PREFIX: &str = "/v1/tasks/";
const ROUTE_AGENTS_PREFIX: &str = "/v1/agents/";
const ROUTE_EVENTS_WS: &str = "/v1/events/ws";
const ROUTE_HEALTHZ: &str = "/healthz";
const ROUTE_METRICS: &str = "/metrics";
const REQUEST_AUTH_SENDER_DID_HEADER: &str = "x-kamn-sender-did";
const REQUEST_AUTH_NONCE_HEADER: &str = "x-kamn-request-nonce";
const REQUEST_AUTH_SIGNATURE_HEADER: &str = "x-kamn-request-signature";
const REQUEST_AUTH_SCOPE_HEADER: &str = "x-kamn-authz-scope";
const REASON_CODE_WEBSOCKET_UPGRADE_REQUIRED: &str = "service_api_websocket_upgrade_required";
const REASON_CODE_METHOD_NOT_ALLOWED: &str = "service_api_method_not_allowed";
const REASON_CODE_ROUTE_NOT_FOUND: &str = "service_api_route_not_found";
const REASON_CODE_REQUEST_READ_FAILED: &str = "service_api_request_read_failed";
const REASON_CODE_INGRESS_BODY_SIZE_LIMIT_EXCEEDED: &str =
    "service_api_ingress_body_size_limit_exceeded";
const REASON_CODE_INGRESS_CONCURRENCY_LIMIT_EXCEEDED: &str =
    "service_api_ingress_concurrency_limit_exceeded";
const REASON_CODE_INGRESS_RATE_LIMIT_EXCEEDED: &str = "service_api_ingress_rate_limit_exceeded";
const REASON_CODE_INGRESS_SENDER_RATE_LIMIT_EXCEEDED: &str =
    "service_api_ingress_sender_rate_limit_exceeded";
const REASON_CODE_INGRESS_SENDER_SUSPENDED: &str = "service_api_ingress_sender_suspended";
const REASON_CODE_INGRESS_SENDER_DUPLICATE_MESSAGE_ID: &str =
    "service_api_ingress_sender_duplicate_message_id";
const REASON_CODE_INGRESS_SENDER_INSUFFICIENT_DEPOSIT: &str =
    "service_api_ingress_sender_insufficient_deposit";
const REASON_CODE_INGRESS_ANTI_SPAM_ENGINE_INVALID: &str =
    "service_api_ingress_anti_spam_engine_invalid";
const REASON_CODE_REQUEST_HEADER_UTF8_INVALID: &str = "service_api_request_header_utf8_invalid";
const REASON_CODE_REQUEST_BODY_UTF8_INVALID: &str = "service_api_request_body_utf8_invalid";
const REASON_CODE_REQUEST_LOG_EMISSION_FAILED: &str = "service_api_request_log_emission_failed";
const REASON_CODE_AUTH_SENDER_DID_HEADER_MISSING: &str =
    "service_api_auth_sender_did_header_missing";
const REASON_CODE_AUTH_SENDER_DID_INVALID: &str = "service_api_auth_sender_did_invalid";
const REASON_CODE_AUTH_NONCE_HEADER_MISSING: &str = "service_api_auth_nonce_header_missing";
const REASON_CODE_AUTH_NONCE_INVALID: &str = "service_api_auth_nonce_invalid";
const REASON_CODE_AUTH_NONCE_NON_POSITIVE: &str = "service_api_auth_nonce_non_positive";
const REASON_CODE_AUTH_SIGNATURE_HEADER_MISSING: &str = "service_api_auth_signature_header_missing";
const REASON_CODE_AUTH_SIGNATURE_VERIFICATION_FAILED: &str =
    "service_api_auth_signature_verification_failed";
const REASON_CODE_AUTH_REPLAY_NONCE_DETECTED: &str = "service_api_auth_replay_nonce_detected";
const REASON_CODE_AUTH_SCOPE_HEADER_MISSING: &str = "service_api_auth_scope_header_missing";
const REASON_CODE_AUTH_SCOPE_INVALID: &str = "service_api_auth_scope_invalid";
const REASON_CODE_AUTH_SCOPE_ROUTE_MISMATCH: &str = "service_api_auth_scope_route_mismatch";
pub(crate) const SERVICE_API_AUTH_REASON_TAXONOMY_VERSION: &str =
    "kamn.runtime.service-api-auth-reason-taxonomy.v1";
pub(crate) const SERVICE_API_AUTH_REASON_CODES_CSV: &str = "service_api_auth_sender_did_header_missing,service_api_auth_sender_did_invalid,service_api_auth_nonce_header_missing,service_api_auth_nonce_invalid,service_api_auth_nonce_non_positive,service_api_auth_signature_header_missing,service_api_auth_signature_verification_failed,service_api_auth_replay_nonce_detected";
pub(crate) const SERVICE_API_SCOPE_POLICY_REASON_TAXONOMY_VERSION: &str =
    "kamn.runtime.service-api-scope-policy-reason-taxonomy.v1";
pub(crate) const SERVICE_API_SCOPE_POLICY_REASON_CODES_CSV: &str = "service_api_auth_scope_header_missing,service_api_auth_scope_invalid,service_api_auth_scope_route_mismatch";
pub(crate) const SERVICE_API_LIFECYCLE_REJECTION_REASON_TAXONOMY_VERSION: &str =
    "kamn.runtime.service-api-lifecycle-rejection-reason-taxonomy.v1";
pub(crate) const SERVICE_API_LIFECYCLE_REJECTION_REASON_CODES_CSV: &str = "service_api_ingress_concurrency_limit_exceeded,service_api_ingress_rate_limit_exceeded,service_api_ingress_sender_rate_limit_exceeded,service_api_ingress_sender_suspended,service_api_ingress_sender_duplicate_message_id,service_api_ingress_sender_insufficient_deposit,service_api_ingress_anti_spam_engine_invalid";
pub(crate) const SERVICE_API_SCOPE_POLICY_FIXTURE_SCHEMA_VERSION: &str =
    "kamn.runtime.service-api-scope-policy-fixture-matrix.v1";
const SERVICE_API_SCOPE_POLICY_FIXTURE: &str =
    include_str!("../../../fixtures/runtime/service_api_scope_policy_fixture_matrix.txt");
pub(crate) const SERVICE_API_ROUTE_AUTHZ_MATRIX_SCHEMA_VERSION: &str =
    "kamn.runtime.service-api-route-authz-matrix.v1";
pub(crate) const SERVICE_API_ROUTE_AUTHZ_MATRIX_TOTAL_ROUTE_COUNT: usize = 10;
pub(crate) const SERVICE_API_ROUTE_AUTHZ_MATRIX_PUBLIC_ROUTE_COUNT: usize = 2;
pub(crate) const SERVICE_API_ROUTE_AUTHZ_MATRIX_PROTECTED_ROUTE_COUNT: usize = 8;
const REASON_CODE_WS_UPGRADE_HEADER_MISSING: &str = "service_api_ws_upgrade_header_missing";
const REASON_CODE_WS_CONNECTION_HEADER_MISSING: &str = "service_api_ws_connection_header_missing";
const REASON_CODE_WS_KEY_HEADER_MISSING: &str = "service_api_ws_key_header_missing";
const REASON_CODE_WS_VERSION_HEADER_MISSING: &str = "service_api_ws_version_header_missing";
const REASON_CODE_WS_UPGRADE_HEADER_INVALID: &str = "service_api_ws_upgrade_header_invalid";
const REASON_CODE_WS_CONNECTION_HEADER_INVALID: &str = "service_api_ws_connection_header_invalid";
const REASON_CODE_WS_KEY_HEADER_EMPTY: &str = "service_api_ws_key_header_empty";
const REASON_CODE_WS_VERSION_HEADER_INVALID: &str = "service_api_ws_version_header_invalid";
const REASON_CODE_WS_EVENTS_MODE_INVALID: &str = "service_api_ws_events_mode_invalid";
const REASON_CODE_WS_PRESENCE_OWNER_DID_HEADER_MISSING: &str =
    "service_api_ws_presence_owner_did_header_missing";
const REASON_CODE_WS_PRESENCE_TARGET_AGENT_DID_HEADER_MISSING: &str =
    "service_api_ws_presence_target_agent_did_header_missing";
const REASON_CODE_WS_PRESENCE_REQUESTER_AGENT_DID_HEADER_MISSING: &str =
    "service_api_ws_presence_requester_agent_did_header_missing";
const REASON_CODE_WS_PRESENCE_CONNECTED_SINCE_INVALID: &str =
    "service_api_ws_presence_connected_since_invalid";
const REASON_CODE_WS_PRESENCE_LAST_HEARTBEAT_INVALID: &str =
    "service_api_ws_presence_last_heartbeat_invalid";
const REASON_CODE_WS_PRESENCE_CAPABILITIES_INVALID: &str =
    "service_api_ws_presence_capabilities_invalid";
const REASON_CODE_WS_PRESENCE_PROJECTION_INVALID: &str =
    "service_api_ws_presence_projection_invalid";
pub(crate) const SERVICE_API_WEBSOCKET_REASON_TAXONOMY_VERSION: &str =
    "kamn.runtime.service-api-websocket-reason-taxonomy.v1";
pub(crate) const SERVICE_API_WEBSOCKET_REASON_CODES_CSV: &str = "service_api_ws_upgrade_header_missing,service_api_ws_connection_header_missing,service_api_ws_key_header_missing,service_api_ws_version_header_missing,service_api_ws_upgrade_header_invalid,service_api_ws_connection_header_invalid,service_api_ws_key_header_empty,service_api_ws_version_header_invalid,service_api_ws_events_mode_invalid,service_api_ws_presence_owner_did_header_missing,service_api_ws_presence_target_agent_did_header_missing,service_api_ws_presence_requester_agent_did_header_missing,service_api_ws_presence_connected_since_invalid,service_api_ws_presence_last_heartbeat_invalid,service_api_ws_presence_capabilities_invalid,service_api_ws_presence_projection_invalid";
const REQUEST_WS_EVENTS_MODE_HEADER: &str = "x-kamn-events-mode";
const REQUEST_WS_PRESENCE_OWNER_DID_HEADER: &str = "x-kamn-presence-owner-did";
const REQUEST_WS_PRESENCE_TARGET_OWNER_DID_HEADER: &str = "x-kamn-presence-target-owner-did";
const REQUEST_WS_PRESENCE_TARGET_AGENT_DID_HEADER: &str = "x-kamn-presence-target-agent-did";
const REQUEST_WS_PRESENCE_GATEWAY_NODE_HEADER: &str = "x-kamn-presence-gateway-node";
const REQUEST_WS_PRESENCE_CONNECTED_SINCE_HEADER: &str = "x-kamn-presence-connected-since";
const REQUEST_WS_PRESENCE_LAST_HEARTBEAT_HEADER: &str = "x-kamn-presence-last-heartbeat";
const REQUEST_WS_PRESENCE_CAPABILITIES_HEADER: &str = "x-kamn-presence-capabilities";
const LIFECYCLE_REJECTION_CLASS_ASYNC_LIMITER: &str = "async-lifecycle-limiter";
const LIFECYCLE_REJECTION_CLASS_SENDER_ADMISSION: &str = "sender-admission-limiter";
const LIFECYCLE_REJECTION_CLASS_ASYNC_ENGINE: &str = "async-lifecycle-engine";
const SERVICE_API_TLS_MODE_ENV: &str = "KAMN_SERVICE_API_TLS_MODE";
const SERVICE_API_TLS_CERT_FILE_ENV: &str = "KAMN_SERVICE_API_TLS_CERT_FILE";
const SERVICE_API_TLS_KEY_FILE_ENV: &str = "KAMN_SERVICE_API_TLS_KEY_FILE";
const SERVICE_API_TLS_MODE_DISABLED: &str = "disabled";
const SERVICE_API_TLS_MODE_REQUIRE: &str = "require";

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ServiceApiEndpointConfig {
    pub(crate) bind_addr: String,
    pub(crate) max_requests: u64,
    pub(crate) idle_timeout_ms: u64,
    pub(crate) body_limit_bytes: u64,
    pub(crate) concurrency_limit: u64,
    pub(crate) rate_limit_per_second: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ServiceApiSnapshot {
    pub(crate) runtime_mode: String,
    pub(crate) role: String,
    pub(crate) chain_id: String,
    pub(crate) chain_version: String,
    pub(crate) cross_store_replay_reason_taxonomy_version: String,
    pub(crate) cross_store_replay_reason_code_count: usize,
    pub(crate) auth_reason_taxonomy_version: String,
    pub(crate) auth_reason_code_count: usize,
    pub(crate) scope_policy_reason_taxonomy_version: String,
    pub(crate) scope_policy_reason_code_count: usize,
    pub(crate) scope_policy_fixture_reason_taxonomy_version: String,
    pub(crate) scope_policy_fixture_reason_code_count: usize,
    pub(crate) scope_policy_fixture_row_count: usize,
    pub(crate) scope_policy_fixture_allow_row_count: usize,
    pub(crate) scope_policy_fixture_deny_row_count: usize,
    pub(crate) scope_policy_fixture_unique_route_count: usize,
    pub(crate) scope_policy_fixture_unique_scope_count: usize,
    pub(crate) scope_policy_fixture_unique_method_count: usize,
    pub(crate) scope_policy_fixture_unique_expected_outcome_count: usize,
    pub(crate) scope_policy_fixture_unique_allow_scope_count: usize,
    pub(crate) scope_policy_fixture_unique_deny_scope_count: usize,
    pub(crate) lifecycle_rejection_reason_taxonomy_version: String,
    pub(crate) lifecycle_rejection_reason_code_count: usize,
    pub(crate) route_authz_matrix_schema_version: String,
    pub(crate) route_authz_matrix_total_route_count: usize,
    pub(crate) route_authz_matrix_public_route_count: usize,
    pub(crate) route_authz_matrix_protected_route_count: usize,
    pub(crate) websocket_reason_taxonomy_version: String,
    pub(crate) websocket_reason_code_count: usize,
    pub(crate) observability_source: String,
    pub(crate) observability_latency_p50_ms: u64,
    pub(crate) observability_latency_p99_ms: u64,
    pub(crate) observability_throughput_tps: u64,
    pub(crate) observability_error_rate_bps: u64,
    pub(crate) observability_availability_bps: u64,
    pub(crate) observability_health: String,
    pub(crate) observability_alert_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ServiceApiEndpointResponse {
    pub(crate) status_code: u16,
    pub(crate) content_type: &'static str,
    pub(crate) body: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct ServiceApiErrorBody {
    pub(crate) error: String,
    pub(crate) reason_code: String,
    pub(crate) message: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct ServiceApiHealthBody {
    pub(crate) status: String,
    pub(crate) runtime_mode: String,
    pub(crate) role: String,
    pub(crate) observability_source: String,
    pub(crate) observability_health: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct ServiceApiMessageCreateBody {
    pub(crate) message_id: String,
    pub(crate) status: String,
    pub(crate) runtime_mode: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct ServiceApiMessageGetBody {
    pub(crate) message_id: String,
    pub(crate) status: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct ServiceApiChannelCreateBody {
    pub(crate) channel_id: String,
    pub(crate) status: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct ServiceApiChannelMessagesBody {
    pub(crate) channel_id: String,
    pub(crate) messages: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct ServiceApiTaskCreateBody {
    pub(crate) task_id: String,
    pub(crate) state: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct ServiceApiTaskGetBody {
    pub(crate) task_id: String,
    pub(crate) state: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct ServiceApiAgentGetBody {
    pub(crate) did: String,
    pub(crate) reputation_score: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct ServiceApiWebsocketStateTransitionBody {
    pub(crate) event: String,
    pub(crate) runtime_mode: String,
    pub(crate) role: String,
    pub(crate) sequence: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ParsedRequest {
    method: String,
    path: String,
    body: String,
    headers: BTreeMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum RequestAuthFailure {
    Unauthorized(ServiceApiReasonedError),
    Replay(ServiceApiReasonedError),
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ServiceApiReasonedError {
    reason_code: &'static str,
    message: String,
}

impl ServiceApiReasonedError {
    fn new(reason_code: &'static str, message: impl Into<String>) -> Self {
        Self {
            reason_code,
            message: message.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum ServiceApiTlsMode {
    Disabled,
    Require { cert_file: String, key_file: String },
}

#[derive(Debug)]
struct ServiceApiRequestBudget {
    max_requests: u64,
    served_requests: AtomicU64,
    completion: Notify,
}

impl ServiceApiRequestBudget {
    fn new(max_requests: u64) -> Self {
        Self {
            max_requests,
            served_requests: AtomicU64::new(0),
            completion: Notify::new(),
        }
    }

    fn record_request(&self) {
        let served = self.served_requests.fetch_add(1, Ordering::SeqCst) + 1;
        if served >= self.max_requests {
            self.completion.notify_waiters();
        }
    }

    async fn wait_until_complete(&self) {
        loop {
            if self.served_requests.load(Ordering::SeqCst) >= self.max_requests {
                return;
            }
            self.completion.notified().await;
        }
    }
}

impl ServiceApiIngressRateWindow {
    fn new(max_requests_per_second: u64) -> Self {
        Self {
            window_start: Instant::now(),
            accepted_requests: 0,
            max_requests_per_second,
        }
    }

    fn try_record_request(&mut self, now: Instant) -> bool {
        if now.duration_since(self.window_start) >= Duration::from_secs(1) {
            self.window_start = now;
            self.accepted_requests = 0;
        }
        if self.accepted_requests >= self.max_requests_per_second {
            return false;
        }
        self.accepted_requests += 1;
        true
    }
}

#[derive(Debug)]
struct ServiceApiRuntimeState {
    snapshot: ServiceApiSnapshot,
    replay_guard: Arc<Mutex<BTreeSet<(String, u64)>>>,
    request_budget: Arc<ServiceApiRequestBudget>,
    body_limit_bytes: usize,
    concurrency_limiter: Arc<Semaphore>,
    ingress_rate_window: Arc<Mutex<ServiceApiIngressRateWindow>>,
    sender_anti_spam: Arc<Mutex<AntiSpamEngine>>,
}

#[derive(Debug)]
struct ServiceApiIngressRateWindow {
    window_start: Instant,
    accepted_requests: u64,
    max_requests_per_second: u64,
}

#[derive(Debug, Clone)]
struct ServiceApiRequestContext {
    parsed_request: ParsedRequest,
    correlation_id: String,
}

#[derive(Debug, Clone, Copy)]
struct ServiceApiRequestOutcome(&'static str);

#[derive(Debug, Clone)]
struct ServiceApiMiddlewareError<'a> {
    correlation_id: &'a str,
    method: &'a str,
    path: &'a str,
    status_code: StatusCode,
    error_label: &'a str,
    reason_code: &'a str,
    message: &'a str,
    outcome: &'a str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ServiceApiLifecycleRejectionPolicy {
    rejection_class: &'static str,
    reason_code: &'static str,
    status_code: StatusCode,
    error_label: &'static str,
    outcome: &'static str,
    default_message: &'static str,
}

#[cfg(test)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ServiceApiLifecycleRejectionProjection {
    pub(crate) rejection_class: &'static str,
    pub(crate) reason_code: &'static str,
    pub(crate) status_code: u16,
    pub(crate) error_label: &'static str,
    pub(crate) outcome: &'static str,
}

pub(crate) fn build_service_api_snapshot(report: &NodeBootstrapReport) -> ServiceApiSnapshot {
    let observability = resolve_service_api_observability(report);
    let cross_store_replay_reason_taxonomy_version =
        cross_store_replay_reason_taxonomy_version().to_owned();
    let cross_store_replay_reason_code_count = cross_store_replay_reason_codes_csv()
        .split(',')
        .filter(|value| !value.is_empty())
        .count();
    let auth_reason_taxonomy_version = SERVICE_API_AUTH_REASON_TAXONOMY_VERSION.to_owned();
    let auth_reason_code_count = SERVICE_API_AUTH_REASON_CODES_CSV
        .split(',')
        .filter(|value| !value.is_empty())
        .count();
    let scope_policy_reason_taxonomy_version =
        SERVICE_API_SCOPE_POLICY_REASON_TAXONOMY_VERSION.to_owned();
    let scope_policy_reason_code_count = SERVICE_API_SCOPE_POLICY_REASON_CODES_CSV
        .split(',')
        .filter(|value| !value.is_empty())
        .count();
    let scope_policy_fixture_projection =
        scope_fixture::parse_service_api_scope_policy_fixture_projection(
            SERVICE_API_SCOPE_POLICY_FIXTURE,
        );
    let lifecycle_rejection_reason_taxonomy_version =
        SERVICE_API_LIFECYCLE_REJECTION_REASON_TAXONOMY_VERSION.to_owned();
    let lifecycle_rejection_reason_code_count = SERVICE_API_LIFECYCLE_REJECTION_REASON_CODES_CSV
        .split(',')
        .filter(|value| !value.is_empty())
        .count();
    let route_authz_matrix_schema_version =
        SERVICE_API_ROUTE_AUTHZ_MATRIX_SCHEMA_VERSION.to_owned();
    let websocket_reason_taxonomy_version =
        SERVICE_API_WEBSOCKET_REASON_TAXONOMY_VERSION.to_owned();
    let websocket_reason_code_count = SERVICE_API_WEBSOCKET_REASON_CODES_CSV
        .split(',')
        .filter(|value| !value.is_empty())
        .count();
    ServiceApiSnapshot {
        runtime_mode: report.runtime_mode.clone(),
        role: report.role.clone(),
        chain_id: report.chain_id.clone(),
        chain_version: report.chain_version.clone(),
        cross_store_replay_reason_taxonomy_version,
        cross_store_replay_reason_code_count,
        auth_reason_taxonomy_version,
        auth_reason_code_count,
        scope_policy_reason_taxonomy_version,
        scope_policy_reason_code_count,
        scope_policy_fixture_reason_taxonomy_version: scope_policy_fixture_projection
            .reason_taxonomy_version,
        scope_policy_fixture_reason_code_count: scope_policy_fixture_projection.reason_code_count,
        scope_policy_fixture_row_count: scope_policy_fixture_projection.row_count,
        scope_policy_fixture_allow_row_count: scope_policy_fixture_projection.allow_row_count,
        scope_policy_fixture_deny_row_count: scope_policy_fixture_projection.deny_row_count,
        scope_policy_fixture_unique_route_count: scope_policy_fixture_projection.unique_route_count,
        scope_policy_fixture_unique_scope_count: scope_policy_fixture_projection.unique_scope_count,
        scope_policy_fixture_unique_method_count: scope_policy_fixture_projection
            .unique_method_count,
        scope_policy_fixture_unique_expected_outcome_count: scope_policy_fixture_projection
            .unique_expected_outcome_count,
        scope_policy_fixture_unique_allow_scope_count: scope_policy_fixture_projection
            .unique_allow_scope_count,
        scope_policy_fixture_unique_deny_scope_count: scope_policy_fixture_projection
            .unique_deny_scope_count,
        lifecycle_rejection_reason_taxonomy_version,
        lifecycle_rejection_reason_code_count,
        route_authz_matrix_schema_version,
        route_authz_matrix_total_route_count: SERVICE_API_ROUTE_AUTHZ_MATRIX_TOTAL_ROUTE_COUNT,
        route_authz_matrix_public_route_count: SERVICE_API_ROUTE_AUTHZ_MATRIX_PUBLIC_ROUTE_COUNT,
        route_authz_matrix_protected_route_count:
            SERVICE_API_ROUTE_AUTHZ_MATRIX_PROTECTED_ROUTE_COUNT,
        websocket_reason_taxonomy_version,
        websocket_reason_code_count,
        observability_source: observability.source,
        observability_latency_p50_ms: observability.latency_p50_ms,
        observability_latency_p99_ms: observability.latency_p99_ms,
        observability_throughput_tps: observability.throughput_tps,
        observability_error_rate_bps: observability.error_rate_bps,
        observability_availability_bps: observability.availability_bps,
        observability_health: observability.health,
        observability_alert_count: observability.alert_count,
    }
}

pub(crate) fn render_service_api_endpoint_response(
    snapshot: &ServiceApiSnapshot,
    method: &str,
    path: &str,
    body: &str,
) -> ServiceApiEndpointResponse {
    payload::render_service_api_endpoint_response(snapshot, method, path, body)
}

pub(crate) fn serve_service_api_endpoint(
    config: &ServiceApiEndpointConfig,
    snapshot: &ServiceApiSnapshot,
) -> Result<(), String> {
    if config.max_requests == 0 {
        return Err("service api max requests must be greater than zero".to_owned());
    }
    if config.idle_timeout_ms == 0 {
        return Err("service api idle timeout must be greater than zero".to_owned());
    }
    if config.body_limit_bytes == 0 {
        return Err("service api body limit bytes must be greater than zero".to_owned());
    }
    if config.concurrency_limit == 0 {
        return Err("service api concurrency limit must be greater than zero".to_owned());
    }
    if config.rate_limit_per_second == 0 {
        return Err("service api rate limit per second must be greater than zero".to_owned());
    }
    if config.body_limit_bytes > usize::MAX as u64 {
        return Err("service api body limit bytes exceed platform usize range".to_owned());
    }
    if config.concurrency_limit > usize::MAX as u64 {
        return Err("service api concurrency limit exceeds platform usize range".to_owned());
    }

    let runtime = Builder::new_current_thread()
        .enable_io()
        .enable_time()
        .build()
        .map_err(|error| format!("service api runtime init failed: {error}"))?;
    runtime.block_on(serve_service_api_endpoint_async(
        config.clone(),
        snapshot.clone(),
    ))
}

async fn serve_service_api_endpoint_async(
    config: ServiceApiEndpointConfig,
    snapshot: ServiceApiSnapshot,
) -> Result<(), String> {
    server::serve_service_api_endpoint_async(config, snapshot).await
}

async fn service_api_auth_middleware(
    State(state): State<Arc<ServiceApiRuntimeState>>,
    request: Request,
    next: Next,
) -> Response {
    middleware_impl::service_api_auth_middleware(State(state), request, next).await
}

async fn handle_service_api_http_route(
    State(state): State<Arc<ServiceApiRuntimeState>>,
    Extension(context): Extension<ServiceApiRequestContext>,
) -> Response {
    middleware_impl::handle_service_api_http_route(State(state), Extension(context)).await
}

async fn handle_service_api_websocket_route(
    State(state): State<Arc<ServiceApiRuntimeState>>,
    Extension(context): Extension<ServiceApiRequestContext>,
    upgrade: WebSocketUpgrade,
) -> Response {
    middleware_impl::handle_service_api_websocket_route(State(state), Extension(context), upgrade)
        .await
}
pub(crate) fn route_requires_auth(method: &str, path: &str) -> bool {
    middleware_impl::route_requires_auth(method, path)
}

#[cfg(test)]
pub(crate) fn project_service_api_lifecycle_rejection(
    reason_code: &str,
) -> Option<ServiceApiLifecycleRejectionProjection> {
    middleware_impl::project_service_api_lifecycle_rejection(reason_code)
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ServiceApiObservabilitySnapshot {
    source: String,
    latency_p50_ms: u64,
    latency_p99_ms: u64,
    throughput_tps: u64,
    error_rate_bps: u64,
    availability_bps: u64,
    health: String,
    alert_count: usize,
}

fn resolve_service_api_observability(
    report: &NodeBootstrapReport,
) -> ServiceApiObservabilitySnapshot {
    if let (
        Some(latency_p50_ms),
        Some(latency_p99_ms),
        Some(throughput_tps),
        Some(error_rate_bps),
        Some(availability_bps),
        Some(health),
        Some(alert_count),
    ) = (
        report.daemon_observability_latency_p50_ms,
        report.daemon_observability_latency_p99_ms,
        report.daemon_observability_throughput_tps,
        report.daemon_observability_error_rate_bps,
        report.daemon_observability_availability_bps,
        report.daemon_observability_health.as_deref(),
        report.daemon_observability_alert_count,
    ) {
        return ServiceApiObservabilitySnapshot {
            source: "daemon".to_owned(),
            latency_p50_ms,
            latency_p99_ms,
            throughput_tps,
            error_rate_bps,
            availability_bps,
            health: health.to_owned(),
            alert_count,
        };
    }

    if let (
        Some(latency_p50_ms),
        Some(latency_p99_ms),
        Some(throughput_tps),
        Some(error_rate_bps),
        Some(availability_bps),
        Some(health),
        Some(alert_count),
    ) = (
        report.kolme_live_observability_latency_p50_ms,
        report.kolme_live_observability_latency_p99_ms,
        report.kolme_live_observability_throughput_tps,
        report.kolme_live_observability_error_rate_bps,
        report.kolme_live_observability_availability_bps,
        report.kolme_live_observability_health.as_deref(),
        report.kolme_live_observability_alert_count,
    ) {
        return ServiceApiObservabilitySnapshot {
            source: "kolme-live".to_owned(),
            latency_p50_ms,
            latency_p99_ms,
            throughput_tps,
            error_rate_bps,
            availability_bps,
            health: health.to_owned(),
            alert_count,
        };
    }

    ServiceApiObservabilitySnapshot {
        source: "unknown".to_owned(),
        latency_p50_ms: 0,
        latency_p99_ms: 0,
        throughput_tps: 0,
        error_rate_bps: 0,
        availability_bps: 0,
        health: "unknown".to_owned(),
        alert_count: 0,
    }
}

fn header_value<'a>(headers: &'a BTreeMap<String, String>, name: &str) -> Option<&'a str> {
    auth::header_value(headers, name)
}

fn authorize_service_api_request(
    snapshot: &ServiceApiSnapshot,
    request: &ParsedRequest,
    replay_guard: &mut BTreeSet<(String, u64)>,
) -> Result<(), RequestAuthFailure> {
    auth::authorize_service_api_request(snapshot, request, replay_guard)
}

async fn enforce_sender_anti_spam(
    state: &ServiceApiRuntimeState,
    request: &ParsedRequest,
) -> Result<(), ServiceApiReasonedError> {
    auth::enforce_sender_anti_spam(state, request).await
}

fn validate_websocket_route_requirements(
    is_websocket_route: bool,
    headers: &BTreeMap<String, String>,
) -> Result<(), ServiceApiReasonedError> {
    websocket::validate_websocket_route_requirements(is_websocket_route, headers)
}

fn websocket_upgrade_response(upgrade: WebSocketUpgrade, event_payload: String) -> Response {
    websocket::websocket_upgrade_response(upgrade, event_payload)
}

fn project_websocket_event_payload(
    snapshot: &ServiceApiSnapshot,
    headers: &BTreeMap<String, String>,
) -> Result<String, ServiceApiReasonedError> {
    websocket::project_websocket_event_payload(snapshot, headers)
}

fn project_websocket_error_response(
    error: &ServiceApiReasonedError,
) -> (StatusCode, &'static str, &'static str) {
    websocket::project_websocket_error_response(error)
}

fn contract_response(response: ServiceApiEndpointResponse) -> Response {
    payload::contract_response(response)
}

fn json_error_response(
    status_code: StatusCode,
    error: &str,
    reason_code: &str,
    message: &str,
) -> Response {
    payload::json_error_response(status_code, error, reason_code, message)
}

#[cfg(test)]
pub(crate) fn parse_service_api_payload<T: DeserializeOwned>(payload: &str) -> Result<T, String> {
    payload::parse_service_api_payload(payload)
}

fn serialize_service_api_json<T: Serialize>(payload: &T) -> String {
    payload::serialize_service_api_json(payload)
}

fn deterministic_body_tag(payload: &[u8]) -> u64 {
    payload::deterministic_body_tag(payload)
}

#[cfg(test)]
mod tests {
    use super::{
        auth::map_anti_spam_rejection_to_reasoned_error, AntiSpamRejection,
        REASON_CODE_INGRESS_SENDER_DUPLICATE_MESSAGE_ID,
        REASON_CODE_INGRESS_SENDER_INSUFFICIENT_DEPOSIT,
        REASON_CODE_INGRESS_SENDER_RATE_LIMIT_EXCEEDED, REASON_CODE_INGRESS_SENDER_SUSPENDED,
    };

    #[test]
    fn anti_spam_rate_limit_rejection_maps_to_sender_rate_limit_reason_code() {
        let error =
            map_anti_spam_rejection_to_reasoned_error(AntiSpamRejection::RateLimitExceeded {
                limit: 3,
                observed: 3,
                window_seconds: 5,
            });
        assert_eq!(
            error.reason_code,
            REASON_CODE_INGRESS_SENDER_RATE_LIMIT_EXCEEDED
        );
        assert!(error.message.contains("observed=3"));
    }

    #[test]
    fn anti_spam_sender_suspension_maps_to_sender_suspended_reason_code() {
        let error = map_anti_spam_rejection_to_reasoned_error(AntiSpamRejection::SenderSuspended {
            until_unix: 123_456,
        });
        assert_eq!(error.reason_code, REASON_CODE_INGRESS_SENDER_SUSPENDED);
        assert!(error.message.contains("123456"));
    }

    #[test]
    fn anti_spam_insufficient_deposit_maps_to_sender_deposit_reason_code() {
        let error =
            map_anti_spam_rejection_to_reasoned_error(AntiSpamRejection::InsufficientDeposit {
                required: 9,
                provided: 4,
            });
        assert_eq!(
            error.reason_code,
            REASON_CODE_INGRESS_SENDER_INSUFFICIENT_DEPOSIT
        );
        assert!(error.message.contains("required=9"));
        assert!(error.message.contains("provided=4"));
    }

    #[test]
    fn anti_spam_duplicate_message_maps_to_sender_duplicate_reason_code() {
        let error = map_anti_spam_rejection_to_reasoned_error(
            AntiSpamRejection::DuplicateMessageId("message-1".to_owned()),
        );
        assert_eq!(
            error.reason_code,
            REASON_CODE_INGRESS_SENDER_DUPLICATE_MESSAGE_ID
        );
        assert!(error.message.contains("message-1"));
    }
}
