use super::*;
use kamn_kolme::{ServiceApiScope, ServiceApiScopeError};

const SELF_CERTIFYING_AGENT_DID_KEY_PREFIX: &str = "kamn:did:agent:pkh-";

pub(super) fn header_value<'a>(
    headers: &'a BTreeMap<String, String>,
    name: &str,
) -> Option<&'a str> {
    headers.get(name).map(String::as_str)
}

pub(super) fn authorize_service_api_request(
    state: &ServiceApiRuntimeState,
    request: &ParsedRequest,
    replay_guard: &mut ServiceApiReplayGuard,
) -> Result<(), RequestAuthFailure> {
    authorize_service_api_request_with_legacy_policy(
        state,
        request,
        replay_guard,
        cfg!(any(test, debug_assertions)),
    )
}

fn require_valid_sender_did_header(request: &ParsedRequest) -> Result<&str, RequestAuthFailure> {
    let sender_did =
        header_value(&request.headers, REQUEST_AUTH_SENDER_DID_HEADER).ok_or_else(|| {
            RequestAuthFailure::Unauthorized(ServiceApiReasonedError::new(
                REASON_CODE_AUTH_SENDER_DID_HEADER_MISSING,
                format!("missing required header: {REQUEST_AUTH_SENDER_DID_HEADER}"),
            ))
        })?;
    AgentDid::parse(sender_did).map_err(|error| {
        RequestAuthFailure::Unauthorized(ServiceApiReasonedError::new(
            REASON_CODE_AUTH_SENDER_DID_INVALID,
            format!("invalid sender did: {error}"),
        ))
    })?;
    Ok(sender_did)
}

fn authorize_service_api_request_with_legacy_policy(
    state: &ServiceApiRuntimeState,
    request: &ParsedRequest,
    replay_guard: &mut ServiceApiReplayGuard,
    allow_legacy_sender_binding: bool,
) -> Result<(), RequestAuthFailure> {
    if !super::route_requires_auth(request.method.as_str(), request.path.as_str()) {
        return Ok(());
    }
    let sender_did = require_valid_sender_did_header(request)?;
    let nonce_raw = header_value(&request.headers, REQUEST_AUTH_NONCE_HEADER).ok_or_else(|| {
        RequestAuthFailure::Unauthorized(ServiceApiReasonedError::new(
            REASON_CODE_AUTH_NONCE_HEADER_MISSING,
            format!("missing required header: {REQUEST_AUTH_NONCE_HEADER}"),
        ))
    })?;
    let nonce = nonce_raw.parse::<u64>().map_err(|_| {
        RequestAuthFailure::Unauthorized(ServiceApiReasonedError::new(
            REASON_CODE_AUTH_NONCE_INVALID,
            format!("invalid request nonce header: {REQUEST_AUTH_NONCE_HEADER}"),
        ))
    })?;
    if nonce == 0 {
        return Err(RequestAuthFailure::Unauthorized(
            ServiceApiReasonedError::new(
                REASON_CODE_AUTH_NONCE_NON_POSITIVE,
                format!("request nonce must be positive: {REQUEST_AUTH_NONCE_HEADER}"),
            ),
        ));
    }
    let signature =
        header_value(&request.headers, REQUEST_AUTH_SIGNATURE_HEADER).ok_or_else(|| {
            RequestAuthFailure::Unauthorized(ServiceApiReasonedError::new(
                REASON_CODE_AUTH_SIGNATURE_HEADER_MISSING,
                format!("missing required header: {REQUEST_AUTH_SIGNATURE_HEADER}"),
            ))
        })?;
    let signer_public_key_hex = resolve_signer_public_key_for_request(
        &request.headers,
        state.auth_public_key_hex.as_deref(),
        allow_legacy_sender_binding,
    )?;
    if !sender_did_matches_signer_public_key(
        sender_did,
        signer_public_key_hex,
        allow_legacy_sender_binding,
    ) {
        return Err(RequestAuthFailure::Unauthorized(
            ServiceApiReasonedError::new(
                REASON_CODE_AUTH_SIGNATURE_VERIFICATION_FAILED,
                "sender did does not match signer public key binding contract",
            ),
        ));
    }
    let state_hash = service_api_signature_state_hash(&state.snapshot);
    let crypto_verified = service_auth_verify_with_public_key_hex(
        signature,
        sender_did,
        nonce,
        state_hash.as_str(),
        request.body.as_str(),
        signer_public_key_hex,
    )
    .is_ok();
    if !crypto_verified {
        return Err(RequestAuthFailure::Unauthorized(
            ServiceApiReasonedError::new(
                REASON_CODE_AUTH_SIGNATURE_VERIFICATION_FAILED,
                "signature verification failed for request envelope",
            ),
        ));
    }
    if !replay_guard.record_nonce_if_fresh(sender_did, nonce, Instant::now()) {
        return Err(RequestAuthFailure::Replay(ServiceApiReasonedError::new(
            REASON_CODE_AUTH_REPLAY_NONCE_DETECTED,
            "request nonce replay detected for sender",
        )));
    }
    Ok(())
}

fn resolve_signer_public_key_for_request<'a>(
    headers: &'a BTreeMap<String, String>,
    fallback_public_key_hex: Option<&'a str>,
    allow_legacy_sender_binding: bool,
) -> Result<&'a str, RequestAuthFailure> {
    if let Some(public_key_hex) = header_value(headers, REQUEST_AUTH_SIGNER_PUBLIC_KEY_HEADER) {
        let normalized = public_key_hex.trim();
        if normalized.is_empty() {
            return Err(RequestAuthFailure::Unauthorized(
                ServiceApiReasonedError::new(
                    REASON_CODE_AUTH_SIGNATURE_VERIFICATION_FAILED,
                    format!(
                        "invalid signer public key header: {REQUEST_AUTH_SIGNER_PUBLIC_KEY_HEADER}"
                    ),
                ),
            ));
        }
        return Ok(public_key_hex);
    }
    if allow_legacy_sender_binding {
        if let Some(public_key_hex) = fallback_public_key_hex {
            return Ok(public_key_hex);
        }
    }
    Err(RequestAuthFailure::Unauthorized(
        ServiceApiReasonedError::new(
            REASON_CODE_AUTH_SIGNATURE_VERIFICATION_FAILED,
            format!("missing required header: {REQUEST_AUTH_SIGNER_PUBLIC_KEY_HEADER}"),
        ),
    ))
}

fn sender_did_matches_signer_public_key(
    sender_did: &str,
    signer_public_key_hex: &str,
    allow_legacy_sender_binding: bool,
) -> bool {
    if let Ok(parsed_did) = AgentDid::parse(sender_did) {
        if parsed_did
            .ensure_public_key_hex_binding(signer_public_key_hex)
            .is_ok()
        {
            return true;
        }
    }
    if let Some(bound_public_key_hex) =
        sender_did.strip_prefix(SELF_CERTIFYING_AGENT_DID_KEY_PREFIX)
    {
        return normalized_public_key_hexes_match(
            bound_public_key_hex.trim(),
            signer_public_key_hex.trim(),
        );
    }
    allow_legacy_sender_binding
}

fn normalized_public_key_hexes_match(left: &str, right: &str) -> bool {
    let normalized_left = ascii_lowercase_bytes(left.as_bytes());
    let normalized_right = ascii_lowercase_bytes(right.as_bytes());
    constant_time_eq_bytes(normalized_left.as_slice(), normalized_right.as_slice())
}

fn ascii_lowercase_bytes(value: &[u8]) -> Vec<u8> {
    value.iter().map(u8::to_ascii_lowercase).collect()
}

fn constant_time_eq_bytes(left: &[u8], right: &[u8]) -> bool {
    if left.len() != right.len() {
        return false;
    }

    let mut diff = 0_u8;
    for (left_byte, right_byte) in left.iter().zip(right.iter()) {
        diff |= left_byte ^ right_byte;
    }
    diff == 0
}

pub(super) fn enforce_request_scope_policy(
    request: &ParsedRequest,
) -> Result<(), ServiceApiReasonedError> {
    let Some(expected_scope) =
        required_scope_for_route(request.method.as_str(), request.path.as_str())
    else {
        return Ok(());
    };
    let scope = header_value(&request.headers, REQUEST_AUTH_SCOPE_HEADER).ok_or_else(|| {
        ServiceApiReasonedError::new(
            REASON_CODE_AUTH_SCOPE_HEADER_MISSING,
            format!("missing required header: {REQUEST_AUTH_SCOPE_HEADER}"),
        )
    })?;
    let parsed_scope = parse_scope(scope)?;
    if parsed_scope != expected_scope {
        return Err(ServiceApiReasonedError::new(
            REASON_CODE_AUTH_SCOPE_ROUTE_MISMATCH,
            format!(
                "scope {} is not authorized for route {} {}",
                parsed_scope.as_str(),
                request.method,
                request.path
            ),
        ));
    }
    Ok(())
}

pub(super) async fn enforce_sender_anti_spam(
    state: &ServiceApiRuntimeState,
    request: &ParsedRequest,
) -> Result<(), ServiceApiReasonedError> {
    if !super::route_requires_auth(request.method.as_str(), request.path.as_str()) {
        return Ok(());
    }

    let sender_did =
        header_value(&request.headers, REQUEST_AUTH_SENDER_DID_HEADER).ok_or_else(|| {
            ServiceApiReasonedError::new(
                REASON_CODE_AUTH_SENDER_DID_HEADER_MISSING,
                format!("missing required header: {REQUEST_AUTH_SENDER_DID_HEADER}"),
            )
        })?;
    let nonce_raw = header_value(&request.headers, REQUEST_AUTH_NONCE_HEADER).ok_or_else(|| {
        ServiceApiReasonedError::new(
            REASON_CODE_AUTH_NONCE_HEADER_MISSING,
            format!("missing required header: {REQUEST_AUTH_NONCE_HEADER}"),
        )
    })?;
    let nonce = nonce_raw.parse::<u64>().map_err(|_| {
        ServiceApiReasonedError::new(
            REASON_CODE_AUTH_NONCE_INVALID,
            format!("invalid request nonce header: {REQUEST_AUTH_NONCE_HEADER}"),
        )
    })?;
    let now_unix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| {
            ServiceApiReasonedError::new(
                REASON_CODE_INGRESS_ANTI_SPAM_ENGINE_INVALID,
                format!("anti-spam clock evaluation failed: {error}"),
            )
        })?
        .as_secs();
    let message_id = format!("{sender_did}:{nonce}:{}", request.path);
    let decision = {
        let mut anti_spam = state.sender_anti_spam.lock().await;
        anti_spam
            .evaluate(sender_did, message_id.as_str(), now_unix)
            .map_err(|error| {
                ServiceApiReasonedError::new(
                    REASON_CODE_INGRESS_ANTI_SPAM_ENGINE_INVALID,
                    format!("anti-spam decision evaluation failed: {error}"),
                )
            })?
    };

    match decision {
        AntiSpamDecision::Accepted => Ok(()),
        AntiSpamDecision::Rejected(rejection) => {
            Err(map_anti_spam_rejection_to_reasoned_error(rejection))
        }
    }
}

pub(super) fn map_anti_spam_rejection_to_reasoned_error(
    rejection: AntiSpamRejection,
) -> ServiceApiReasonedError {
    match rejection {
        AntiSpamRejection::InsufficientDeposit { required, provided } => {
            ServiceApiReasonedError::new(
                REASON_CODE_INGRESS_SENDER_INSUFFICIENT_DEPOSIT,
                format!(
                    "sender deposit below anti-spam minimum: required={required}, provided={provided}"
                ),
            )
        }
        AntiSpamRejection::RateLimitExceeded {
            limit,
            observed,
            window_seconds,
        } => ServiceApiReasonedError::new(
            REASON_CODE_INGRESS_SENDER_RATE_LIMIT_EXCEEDED,
            format!(
                "sender anti-spam rate limit exceeded: observed={observed}, limit={limit}, window_seconds={window_seconds}"
            ),
        ),
        AntiSpamRejection::SenderSuspended { until_unix } => ServiceApiReasonedError::new(
            REASON_CODE_INGRESS_SENDER_SUSPENDED,
            format!("sender suspended by anti-spam policy until unix={until_unix}"),
        ),
        AntiSpamRejection::DuplicateMessageId(message_id) => ServiceApiReasonedError::new(
            REASON_CODE_INGRESS_SENDER_DUPLICATE_MESSAGE_ID,
            format!("sender anti-spam duplicate message id rejected: {message_id}"),
        ),
    }
}

pub(super) fn service_api_signature_state_hash(snapshot: &ServiceApiSnapshot) -> String {
    format!(
        "service-api:{}:{}",
        snapshot.chain_id.as_str(),
        snapshot.chain_version.as_str()
    )
}

fn required_scope_for_route(method: &str, path: &str) -> Option<ServiceApiScope> {
    if !super::route_requires_auth(method, path) {
        return None;
    }
    let scope = match (method, path) {
        ("POST", ROUTE_MESSAGES_SEND) => ServiceApiScope::MessagesWrite,
        ("POST", ROUTE_MESSAGES_RELAY) => ServiceApiScope::MessagesWrite,
        ("POST", ROUTE_CHANNELS_CREATE) => ServiceApiScope::ChannelsWrite,
        ("POST", ROUTE_AGENTS_SEARCH) => ServiceApiScope::AgentsRead,
        ("POST", ROUTE_AGENTS_REGISTER) => ServiceApiScope::AgentsWrite,
        ("POST", ROUTE_TASKS_CREATE) => ServiceApiScope::TasksWrite,
        ("POST", _) if super::payload::task_accept_path_id(path).is_some() => {
            ServiceApiScope::TasksWrite
        }
        ("POST", _) if super::payload::task_complete_path_id(path).is_some() => {
            ServiceApiScope::TasksWrite
        }
        ("POST", ROUTE_ESCROW_FUND) => ServiceApiScope::EscrowWrite,
        ("POST", _) if super::payload::escrow_release_path_id(path).is_some() => {
            ServiceApiScope::EscrowWrite
        }
        ("POST", ROUTE_CONTENT_REGISTER) => ServiceApiScope::ContentWrite,
        ("POST", _) if super::payload::content_expire_path_id(path).is_some() => {
            ServiceApiScope::ContentWrite
        }
        ("POST", _) if super::payload::content_tombstone_path_id(path).is_some() => {
            ServiceApiScope::ContentWrite
        }
        ("POST", ROUTE_BRIDGE_SUBMIT) => ServiceApiScope::BridgeWrite,
        ("POST", _) if super::payload::bridge_forward_path_id(path).is_some() => {
            ServiceApiScope::BridgeWrite
        }
        ("GET", ROUTE_EVENTS_WS) => ServiceApiScope::EventsRead,
        ("GET", _) if super::payload::content_path_id(path).is_some() => {
            ServiceApiScope::ContentRead
        }
        ("GET", _) if super::payload::bridge_path_id(path).is_some() => ServiceApiScope::BridgeRead,
        ("GET", _) if super::payload::message_path_id(path).is_some() => {
            ServiceApiScope::MessagesRead
        }
        ("GET", _) if super::payload::channel_messages_path_id(path).is_some() => {
            ServiceApiScope::ChannelsRead
        }
        ("GET", _) if super::payload::task_path_id(path).is_some() => ServiceApiScope::TasksRead,
        ("GET", _) if super::payload::agent_balance_path_id(path).is_some() => {
            ServiceApiScope::AgentsRead
        }
        ("GET", _) if super::payload::agent_path_id(path).is_some() => ServiceApiScope::AgentsRead,
        _ => ServiceApiScope::ProtectedUnknown,
    };
    Some(scope)
}

fn parse_scope(scope: &str) -> Result<ServiceApiScope, ServiceApiReasonedError> {
    ServiceApiScope::parse(scope).map_err(|error| match error {
        ServiceApiScopeError::Empty => ServiceApiReasonedError::new(
            REASON_CODE_AUTH_SCOPE_INVALID,
            format!("scope header must not be empty: {REQUEST_AUTH_SCOPE_HEADER}"),
        ),
        ServiceApiScopeError::Unknown => ServiceApiReasonedError::new(
            REASON_CODE_AUTH_SCOPE_INVALID,
            format!("scope header value is invalid: {REQUEST_AUTH_SCOPE_HEADER}"),
        ),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use kamn_core::{
        service_auth_public_key_hex_from_private_key_hex, service_auth_sign_with_private_key_hex,
    };

    const TEST_SERVICE_API_AUTH_PRIVATE_KEY_HEX: &str =
        "658c3528422eb527b4c108b8f6d1e5f629543c304ea49cf608c67794424291c4";

    fn test_service_api_runtime_state() -> ServiceApiRuntimeState {
        let auth_public_key_hex =
            service_auth_public_key_hex_from_private_key_hex(TEST_SERVICE_API_AUTH_PRIVATE_KEY_HEX)
                .expect("service-auth public key should derive");
        let sender_anti_spam =
            AntiSpamEngine::new(AntiSpamConfig::default()).expect("anti-spam config should build");
        ServiceApiRuntimeState {
            snapshot: build_service_api_snapshot(&NodeBootstrapReport {
                runtime_mode: "test".to_owned(),
                diagnostics_mode: "basic".to_owned(),
                component_count: 1,
                planning_expected_state_hash: None,
                planning_candidate_count: None,
                planning_scheduled_candidate_ids: None,
                recovery_expected_state_version: None,
                recovery_expected_state_hash: None,
                recovery_attempt_count: None,
                recovery_decisions: None,
                daemon_max_ticks: None,
                daemon_tick_interval_ms: None,
                daemon_executed_ticks: None,
                daemon_completion_reason: None,
                daemon_service_api_relay_drained_count: None,
                daemon_service_api_relay_projected_state_count: None,
                daemon_observability_latency_p50_ms: None,
                daemon_observability_latency_p99_ms: None,
                daemon_observability_throughput_tps: None,
                daemon_observability_error_rate_bps: None,
                daemon_observability_availability_bps: None,
                daemon_observability_health: None,
                daemon_observability_alert_count: None,
                daemon_observability_reason_code: None,
                daemon_observability_transport_checkpoint_failures: None,
                daemon_observability_signer_checkpoint_failures: None,
                daemon_observability_commit_checkpoint_failures: None,
                daemon_peer_id: None,
                daemon_peer_lifecycle_final_state: None,
                daemon_peer_lifecycle_applied_events: None,
                daemon_phase6_runtime_reason_taxonomy_version: None,
                daemon_phase6_runtime_reason_codes_csv: None,
                daemon_phase6_runtime_reason_code: None,
                daemon_phase6_runtime_total_cycles: None,
                daemon_phase6_runtime_executed_cycles: None,
                daemon_phase6_runtime_deferred_cycles: None,
                daemon_phase6_runtime_fail_closed_cycles: None,
                daemon_convergence_reason_taxonomy_version: None,
                daemon_convergence_reason_codes_csv: None,
                daemon_convergence_decision: None,
                daemon_convergence_reason_code: None,
                daemon_convergence_schema_gate_passed: None,
                daemon_convergence_error_path_gate_passed: None,
                daemon_convergence_concurrency_gate_passed: None,
                daemon_convergence_performance_budget_gate_passed: None,
                daemon_convergence_cost_budget_gate_passed: None,
                daemon_live_postgres_multi_host_execution_bundle_schema_version: None,
                daemon_live_postgres_multi_host_execution_bundle_selector_prefix: None,
                daemon_live_postgres_multi_host_execution_bundle_row_count: None,
                daemon_live_postgres_multi_host_execution_bundle_selector_rows_fingerprint: None,
                kolme_live_provider_client_contract: None,
                kolme_live_base_url: None,
                kolme_live_provider_hint: None,
                kolme_live_signing_profile: None,
                kolme_live_signer_profile_selector_env: None,
                kolme_live_signer_profile: None,
                kolme_live_signer_key_source: None,
                kolme_live_signer_private_key_env: None,
                kolme_live_execution_status: None,
                kolme_live_observability_latency_p50_ms: None,
                kolme_live_observability_latency_p99_ms: None,
                kolme_live_observability_throughput_tps: None,
                kolme_live_observability_error_rate_bps: None,
                kolme_live_observability_availability_bps: None,
                kolme_live_observability_health: None,
                kolme_live_observability_alert_count: None,
                kolme_live_observability_reason_code: None,
                kolme_live_observability_transport_checkpoint_failures: None,
                kolme_live_observability_signer_checkpoint_failures: None,
                kolme_live_observability_commit_checkpoint_failures: None,
                profile: None,
                role: "processor".to_owned(),
                chain_id: "kamn-devnet".to_owned(),
                chain_version: "v0.1.0".to_owned(),
                storage_dir: "./data".to_owned(),
                gossip_enabled: false,
                sync_mode: "local".to_owned(),
                sync_startup: "cold".to_owned(),
                sync_recovery: "disabled".to_owned(),
                state_version: 1,
                pending_migrations: 0,
                components: vec!["service-api".to_owned()],
            }),
            replay_guard: Arc::new(Mutex::new(ServiceApiReplayGuard::new(
                8,
                Duration::from_secs(60),
            ))),
            request_budget: Arc::new(ServiceApiRequestBudget::new(1)),
            websocket_events: ServiceApiWebsocketEventFanout::new(),
            runtime_observability: Arc::new(Mutex::new(ServiceApiRuntimeObservability::new(
                Instant::now(),
            ))),
            body_limit_bytes: 1024,
            concurrency_limiter: Arc::new(Semaphore::new(1)),
            ingress_rate_window: Arc::new(Mutex::new(ServiceApiIngressRateWindow::new(120))),
            sender_anti_spam: Arc::new(Mutex::new(sender_anti_spam)),
            auth_public_key_hex: Some(auth_public_key_hex),
            message_store: Arc::new(Mutex::new(
                ServiceApiMessageStore::from_optional_state_file(None)
                    .expect("test message store should build"),
            )),
            relay_spool_file: None,
        }
    }

    fn legacy_sender_request(snapshot: &ServiceApiSnapshot) -> ParsedRequest {
        let sender_did = "kamn:did:agent:alice";
        let nonce = 41_u64;
        let body = "{}";
        let signature = service_auth_sign_with_private_key_hex(
            sender_did,
            nonce,
            service_api_signature_state_hash(snapshot).as_str(),
            body,
            TEST_SERVICE_API_AUTH_PRIVATE_KEY_HEX,
        )
        .expect("service-auth signature should render for test fixture key");
        ParsedRequest {
            method: "POST".to_owned(),
            path: ROUTE_MESSAGES_SEND.to_owned(),
            body: body.to_owned(),
            headers: BTreeMap::from([
                (
                    REQUEST_AUTH_SENDER_DID_HEADER.to_owned(),
                    sender_did.to_owned(),
                ),
                (REQUEST_AUTH_NONCE_HEADER.to_owned(), nonce.to_string()),
                (REQUEST_AUTH_SIGNATURE_HEADER.to_owned(), signature),
            ]),
        }
    }

    #[test]
    fn unit_required_scope_for_route_maps_known_route_contracts() {
        assert_eq!(
            required_scope_for_route("POST", ROUTE_MESSAGES_SEND),
            Some(ServiceApiScope::MessagesWrite)
        );
        assert_eq!(
            required_scope_for_route("POST", ROUTE_MESSAGES_RELAY),
            Some(ServiceApiScope::MessagesWrite)
        );
        assert_eq!(
            required_scope_for_route("POST", ROUTE_CHANNELS_CREATE),
            Some(ServiceApiScope::ChannelsWrite)
        );
        assert_eq!(
            required_scope_for_route("POST", ROUTE_TASKS_CREATE),
            Some(ServiceApiScope::TasksWrite)
        );
        assert_eq!(
            required_scope_for_route("POST", "/v1/tasks/task-1/accept"),
            Some(ServiceApiScope::TasksWrite)
        );
        assert_eq!(
            required_scope_for_route("POST", "/v1/tasks/task-1/complete"),
            Some(ServiceApiScope::TasksWrite)
        );
        assert_eq!(
            required_scope_for_route("POST", ROUTE_ESCROW_FUND),
            Some(ServiceApiScope::EscrowWrite)
        );
        assert_eq!(
            required_scope_for_route("POST", "/v1/escrow/escrow-1/release"),
            Some(ServiceApiScope::EscrowWrite)
        );
        assert_eq!(
            required_scope_for_route("POST", ROUTE_CONTENT_REGISTER),
            Some(ServiceApiScope::ContentWrite)
        );
        assert_eq!(
            required_scope_for_route("POST", "/v1/content/content-1/expire"),
            Some(ServiceApiScope::ContentWrite)
        );
        assert_eq!(
            required_scope_for_route("POST", "/v1/content/content-1/tombstone"),
            Some(ServiceApiScope::ContentWrite)
        );
        assert_eq!(
            required_scope_for_route("POST", ROUTE_BRIDGE_SUBMIT),
            Some(ServiceApiScope::BridgeWrite)
        );
        assert_eq!(
            required_scope_for_route("POST", "/v1/bridge/bridge-1/forward"),
            Some(ServiceApiScope::BridgeWrite)
        );
        assert_eq!(
            required_scope_for_route("GET", ROUTE_EVENTS_WS),
            Some(ServiceApiScope::EventsRead)
        );
        assert_eq!(
            required_scope_for_route("GET", "/v1/content/content-1"),
            Some(ServiceApiScope::ContentRead)
        );
        assert_eq!(
            required_scope_for_route("GET", "/v1/bridge/bridge-1"),
            Some(ServiceApiScope::BridgeRead)
        );
        assert_eq!(
            required_scope_for_route("GET", "/v1/messages/message-1"),
            Some(ServiceApiScope::MessagesRead)
        );
        assert_eq!(
            required_scope_for_route("GET", "/v1/channels/channel-1/messages"),
            Some(ServiceApiScope::ChannelsRead)
        );
        assert_eq!(
            required_scope_for_route("GET", "/v1/tasks/task-1"),
            Some(ServiceApiScope::TasksRead)
        );
        assert_eq!(
            required_scope_for_route("GET", "/v1/agents/kamn:did:agent:alice"),
            Some(ServiceApiScope::AgentsRead)
        );
        assert_eq!(
            required_scope_for_route("GET", "/v1/agents/kamn:did:agent:alice/balance"),
            Some(ServiceApiScope::AgentsRead)
        );
    }

    #[test]
    fn regression_required_scope_for_route_preserves_public_and_unknown_contracts() {
        // Regression: #5831
        assert_eq!(required_scope_for_route("GET", ROUTE_HEALTHZ), None);
        assert_eq!(required_scope_for_route("GET", ROUTE_METRICS), None);
        assert_eq!(
            required_scope_for_route("DELETE", "/v1/unknown/path"),
            Some(ServiceApiScope::ProtectedUnknown)
        );
        assert_eq!(
            required_scope_for_route("POST", "/v1/unknown/path"),
            Some(ServiceApiScope::ProtectedUnknown)
        );
        assert_eq!(
            required_scope_for_route("GET", "/v1/unknown/path"),
            Some(ServiceApiScope::ProtectedUnknown)
        );
    }

    #[test]
    fn unit_parse_scope_accepts_trimmed_canonical_values() {
        assert_eq!(
            parse_scope(" messages:write ").expect("scope"),
            ServiceApiScope::MessagesWrite
        );
        assert_eq!(
            parse_scope("tasks:read").expect("scope"),
            ServiceApiScope::TasksRead
        );
        assert_eq!(
            parse_scope(" content:write ").expect("scope"),
            ServiceApiScope::ContentWrite
        );
        assert_eq!(
            parse_scope(" bridge:write ").expect("scope"),
            ServiceApiScope::BridgeWrite
        );
        assert_eq!(
            parse_scope("bridge:read").expect("scope"),
            ServiceApiScope::BridgeRead
        );
        assert_eq!(
            parse_scope(" agents:write ").expect("scope").as_str(),
            "agents:write"
        );
        assert_eq!(
            parse_scope("protected:unknown").expect("scope"),
            ServiceApiScope::ProtectedUnknown
        );
    }

    #[test]
    fn regression_required_scope_for_route_maps_agent_registration_to_agents_write() {
        assert_eq!(
            required_scope_for_route("POST", "/v1/agents/register").map(ServiceApiScope::as_str),
            Some("agents:write")
        );
    }

    #[test]
    fn regression_required_scope_for_route_maps_agent_search_to_agents_read() {
        assert_eq!(
            required_scope_for_route("POST", "/v1/agents/search").map(ServiceApiScope::as_str),
            Some("agents:read")
        );
    }

    #[test]
    fn regression_public_auth_entrypoint_rejects_legacy_sender_binding_without_signer_header() {
        let state = test_service_api_runtime_state();
        let request = legacy_sender_request(&state.snapshot);
        let mut replay_guard = ServiceApiReplayGuard::new(8, Duration::from_secs(60));

        let error = authorize_service_api_request(&state, &request, &mut replay_guard)
            .expect_err("public auth entrypoint should fail closed without signer header");
        let RequestAuthFailure::Unauthorized(error) = error else {
            panic!("expected unauthorized auth failure");
        };
        assert_eq!(
            error.reason_code,
            REASON_CODE_AUTH_SIGNATURE_VERIFICATION_FAILED
        );
        assert!(
            error
                .message
                .contains(REQUEST_AUTH_SIGNER_PUBLIC_KEY_HEADER)
        );
    }

    #[test]
    fn regression_explicit_legacy_auth_policy_still_allows_test_only_fallback_binding() {
        let state = test_service_api_runtime_state();
        let request = legacy_sender_request(&state.snapshot);
        let mut replay_guard = ServiceApiReplayGuard::new(8, Duration::from_secs(60));

        authorize_service_api_request_with_legacy_policy(&state, &request, &mut replay_guard, true)
            .expect("explicit legacy policy should remain available for targeted tests");
    }

    #[test]
    fn unit_parse_scope_rejects_empty_and_unknown_values() {
        let empty_error = parse_scope("  ").expect_err("empty scope should fail");
        assert_eq!(empty_error.reason_code, REASON_CODE_AUTH_SCOPE_INVALID);
        assert!(empty_error.message.contains("must not be empty"));

        let unknown_error = parse_scope("content:admin").expect_err("unknown scope should fail");
        assert_eq!(unknown_error.reason_code, REASON_CODE_AUTH_SCOPE_INVALID);
        assert!(unknown_error.message.contains("value is invalid"));
    }

    #[test]
    fn regression_replay_guard_capacity_eviction_bounds_memory_and_releases_oldest_nonce() {
        // Regression: #5928
        let start = Instant::now();
        let mut guard = ServiceApiReplayGuard::new(3, Duration::from_secs(300));

        assert!(guard.record_nonce_if_fresh("kamn:did:agent:alice", 1, start));
        assert!(guard.record_nonce_if_fresh(
            "kamn:did:agent:alice",
            2,
            start + Duration::from_secs(1)
        ));
        assert!(guard.record_nonce_if_fresh(
            "kamn:did:agent:alice",
            3,
            start + Duration::from_secs(2)
        ));
        assert_eq!(guard.tracked_entry_count(), 3);

        assert!(guard.record_nonce_if_fresh(
            "kamn:did:agent:alice",
            4,
            start + Duration::from_secs(3)
        ));
        assert_eq!(guard.tracked_entry_count(), 3);

        assert!(!guard.record_nonce_if_fresh(
            "kamn:did:agent:alice",
            1,
            start + Duration::from_secs(4)
        ));
    }

    #[test]
    fn regression_replay_guard_ttl_eviction_rejects_only_within_active_window() {
        // Regression: #5928
        let start = Instant::now();
        let mut guard = ServiceApiReplayGuard::new(8, Duration::from_secs(2));

        assert!(guard.record_nonce_if_fresh("kamn:did:agent:bob", 9, start));
        assert!(!guard.record_nonce_if_fresh(
            "kamn:did:agent:bob",
            9,
            start + Duration::from_secs(1)
        ));
        assert!(!guard.record_nonce_if_fresh(
            "kamn:did:agent:bob",
            9,
            start + Duration::from_secs(3)
        ));
    }

    #[test]
    fn regression_issue_6196_nonce_contract_rejects_post_ttl_replay_nonce_values() {
        // Regression: #6196
        let start = Instant::now();
        let mut guard = ServiceApiReplayGuard::new(8, Duration::from_secs(2));

        assert!(guard.record_nonce_if_fresh("kamn:did:agent:ivy", 50, start));
        assert!(!guard.record_nonce_if_fresh(
            "kamn:did:agent:ivy",
            50,
            start + Duration::from_secs(3)
        ));
        assert!(!guard.record_nonce_if_fresh(
            "kamn:did:agent:ivy",
            49,
            start + Duration::from_secs(4)
        ));
        assert!(guard.record_nonce_if_fresh(
            "kamn:did:agent:ivy",
            51,
            start + Duration::from_secs(5)
        ));
    }

    #[test]
    fn regression_replay_guard_seeded_nonce_rejects_stale_values_after_restart() {
        // Regression: #6186
        let start = Instant::now();
        let mut guard = ServiceApiReplayGuard::new(8, Duration::from_secs(60));
        guard.seed_sender_nonce_high_watermark("kamn:did:agent:carol", 42);

        assert!(!guard.record_nonce_if_fresh("kamn:did:agent:carol", 42, start));
        assert!(!guard.record_nonce_if_fresh(
            "kamn:did:agent:carol",
            41,
            start + Duration::from_secs(1)
        ));
        assert!(guard.record_nonce_if_fresh(
            "kamn:did:agent:carol",
            43,
            start + Duration::from_secs(2)
        ));
    }

    #[test]
    fn unit_sender_did_binding_accepts_self_certifying_public_key_suffix() {
        let signer_public_key_hex =
            "02f89df7f03f4db9ef84f54cf1f4df4df8fd5bca90b7c2f4c0333b3c0f4bc0fe11";
        let sender_did = format!("kamn:did:agent:pkh-{signer_public_key_hex}");
        assert!(sender_did_matches_signer_public_key(
            sender_did.as_str(),
            signer_public_key_hex,
            false,
        ));
    }

    #[test]
    fn regression_sender_did_binding_accepts_case_variant_signer_public_key_header() {
        let signer_public_key_hex =
            "02f89df7f03f4db9ef84f54cf1f4df4df8fd5bca90b7c2f4c0333b3c0f4bc0fe11";
        let sender_did = format!("kamn:did:agent:pkh-{signer_public_key_hex}");
        assert!(sender_did_matches_signer_public_key(
            sender_did.as_str(),
            signer_public_key_hex.to_uppercase().as_str(),
            false,
        ));
    }

    #[test]
    fn regression_sender_did_binding_accepts_keyh_bound_pkh_did() {
        // Regression: #6303 e2e S-02 must accept did:key-binding sender identities.
        let signer_public_key_hex =
            "02f89df7f03f4db9ef84f54cf1f4df4df8fd5bca90b7c2f4c0333b3c0f4bc0fe11";
        let sender_did = AgentDid::with_public_key_hex_binding(
            format!("pkh-{signer_public_key_hex}").as_str(),
            signer_public_key_hex,
        )
        .expect("key-bound sender did should build");
        assert!(sender_did_matches_signer_public_key(
            sender_did.as_str(),
            signer_public_key_hex,
            false,
        ));
    }

    #[test]
    fn unit_sender_did_binding_rejects_self_certifying_key_mismatch() {
        let sender_did =
            "kamn:did:agent:pkh-02f89df7f03f4db9ef84f54cf1f4df4df8fd5bca90b7c2f4c0333b3c0f4bc0fe11";
        assert!(!sender_did_matches_signer_public_key(
            sender_did,
            "03f89df7f03f4db9ef84f54cf1f4df4df8fd5bca90b7c2f4c0333b3c0f4bc0fe11",
            false,
        ));
    }

    #[test]
    fn regression_sender_did_binding_rejects_keyh_bound_pkh_did_mismatch() {
        // Regression: #6303 sender did keyh mismatch must fail closed.
        let sender_did = AgentDid::with_public_key_hex_binding(
            "pkh-02f89df7f03f4db9ef84f54cf1f4df4df8fd5bca90b7c2f4c0333b3c0f4bc0fe11",
            "02f89df7f03f4db9ef84f54cf1f4df4df8fd5bca90b7c2f4c0333b3c0f4bc0fe11",
        )
        .expect("key-bound sender did should build");
        assert!(!sender_did_matches_signer_public_key(
            sender_did.as_str(),
            "03f89df7f03f4db9ef84f54cf1f4df4df8fd5bca90b7c2f4c0333b3c0f4bc0fe11",
            false,
        ));
    }

    #[test]
    fn unit_sender_did_binding_rejects_legacy_did_without_legacy_policy() {
        assert!(!sender_did_matches_signer_public_key(
            "kamn:did:agent:alice",
            "02f89df7f03f4db9ef84f54cf1f4df4df8fd5bca90b7c2f4c0333b3c0f4bc0fe11",
            false,
        ));
        assert!(sender_did_matches_signer_public_key(
            "kamn:did:agent:alice",
            "02f89df7f03f4db9ef84f54cf1f4df4df8fd5bca90b7c2f4c0333b3c0f4bc0fe11",
            true,
        ));
    }

    #[test]
    fn regression_signer_public_key_resolution_requires_header_without_legacy_policy() {
        // Regression: #6184
        let headers = BTreeMap::new();
        let error = resolve_signer_public_key_for_request(
            &headers,
            Some("02f89df7f03f4db9ef84f54cf1f4df4df8fd5bca90b7c2f4c0333b3c0f4bc0fe11"),
            false,
        )
        .expect_err("production policy should require explicit signer public key header");
        let RequestAuthFailure::Unauthorized(error) = error else {
            panic!("expected unauthorized auth failure");
        };
        assert_eq!(
            error.reason_code,
            REASON_CODE_AUTH_SIGNATURE_VERIFICATION_FAILED
        );
        assert!(
            error
                .message
                .contains(REQUEST_AUTH_SIGNER_PUBLIC_KEY_HEADER)
        );
    }

    #[test]
    fn unit_signer_public_key_resolution_allows_legacy_fallback_when_enabled() {
        let headers = BTreeMap::new();
        let resolved = resolve_signer_public_key_for_request(
            &headers,
            Some("02f89df7f03f4db9ef84f54cf1f4df4df8fd5bca90b7c2f4c0333b3c0f4bc0fe11"),
            true,
        )
        .expect("legacy policy should allow shared fallback key");
        assert_eq!(
            resolved,
            "02f89df7f03f4db9ef84f54cf1f4df4df8fd5bca90b7c2f4c0333b3c0f4bc0fe11"
        );
    }

    #[test]
    fn regression_sender_did_header_rejects_legacy_did_shape() {
        // Regression: #6502
        let request = ParsedRequest {
            method: "POST".to_owned(),
            path: ROUTE_MESSAGES_SEND.to_owned(),
            body: "{}".to_owned(),
            headers: BTreeMap::from([(
                REQUEST_AUTH_SENDER_DID_HEADER.to_owned(),
                "did:kamn:agent:legacy-alpha".to_owned(),
            )]),
        };
        let error = require_valid_sender_did_header(&request)
            .expect_err("legacy did shape should fail closed at auth ingress");
        let RequestAuthFailure::Unauthorized(error) = error else {
            panic!("expected unauthorized auth failure");
        };
        assert_eq!(error.reason_code, REASON_CODE_AUTH_SENDER_DID_INVALID);
        assert!(error.message.contains("invalid sender did"));
    }
}
