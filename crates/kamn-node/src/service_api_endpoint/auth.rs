use super::*;
use kamn_kolme::{ServiceApiScope, ServiceApiScopeError};

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
    if !super::route_requires_auth(request.method.as_str(), request.path.as_str()) {
        return Ok(());
    }
    let sender_did =
        header_value(&request.headers, REQUEST_AUTH_SENDER_DID_HEADER).ok_or_else(|| {
            RequestAuthFailure::Unauthorized(ServiceApiReasonedError::new(
                REASON_CODE_AUTH_SENDER_DID_HEADER_MISSING,
                format!("missing required header: {REQUEST_AUTH_SENDER_DID_HEADER}"),
            ))
        })?;
    let parsed_sender_did = AgentDid::parse(sender_did).map_err(|error| {
        RequestAuthFailure::Unauthorized(ServiceApiReasonedError::new(
            REASON_CODE_AUTH_SENDER_DID_INVALID,
            format!("invalid sender did: {error}"),
        ))
    })?;
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
    let state_hash = service_api_signature_state_hash(&state.snapshot);
    let selected_public_key_hex = select_service_api_auth_public_key_for_sender(
        sender_did,
        state.auth_public_keys_by_did.as_ref(),
        state.auth_public_key_hex.as_deref(),
    );
    enforce_sender_did_public_key_binding_if_required(
        &parsed_sender_did,
        selected_public_key_hex,
        state.auth_public_keys_by_did.as_ref(),
    )
    .map_err(RequestAuthFailure::Unauthorized)?;
    let crypto_verified = selected_public_key_hex
        .map(|public_key_hex| {
            service_auth_verify_with_public_key_hex(
                signature,
                sender_did,
                nonce,
                state_hash.as_str(),
                request.body.as_str(),
                public_key_hex,
            )
            .is_ok()
        })
        .unwrap_or(false);
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

fn select_service_api_auth_public_key_for_sender<'a>(
    sender_did: &str,
    auth_public_keys_by_did: Option<&'a BTreeMap<String, String>>,
    fallback_auth_public_key_hex: Option<&'a str>,
) -> Option<&'a str> {
    if let Some(public_keys_by_did) = auth_public_keys_by_did {
        return public_keys_by_did.get(sender_did).map(String::as_str);
    }
    fallback_auth_public_key_hex
}

fn enforce_sender_did_public_key_binding_if_required(
    sender_did: &AgentDid,
    selected_public_key_hex: Option<&str>,
    auth_public_keys_by_did: Option<&BTreeMap<String, String>>,
) -> Result<(), ServiceApiReasonedError> {
    if auth_public_keys_by_did.is_none() {
        return Ok(());
    }
    let Some(public_key_hex) = selected_public_key_hex else {
        return Ok(());
    };
    sender_did
        .ensure_public_key_hex_binding(public_key_hex)
        .map_err(|error| {
            ServiceApiReasonedError::new(
                REASON_CODE_AUTH_DID_KEY_BINDING_INVALID,
                format!("sender did key binding invalid: {error}"),
            )
        })
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
        AntiSpamRejection::InsufficientDeposit { required, provided } => ServiceApiReasonedError::new(
            REASON_CODE_INGRESS_SENDER_INSUFFICIENT_DEPOSIT,
            format!(
                "sender deposit below anti-spam minimum: required={required}, provided={provided}"
            ),
        ),
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
    use std::fs;
    use std::process;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn replay_guard_temp_state_file(label: &str) -> String {
        let mut path = std::env::temp_dir();
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock")
            .as_nanos();
        path.push(format!(
            "kamn-replay-guard-{label}-{}-{nanos}.json",
            process::id()
        ));
        path.to_string_lossy().to_string()
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
            parse_scope("protected:unknown").expect("scope"),
            ServiceApiScope::ProtectedUnknown
        );
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

        assert!(guard.record_nonce_if_fresh(
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
        assert!(guard.record_nonce_if_fresh(
            "kamn:did:agent:bob",
            9,
            start + Duration::from_secs(3)
        ));
    }

    #[test]
    fn regression_replay_guard_rejects_nonce_after_restart_when_state_file_is_shared() {
        // Regression: #6060
        let state_file = replay_guard_temp_state_file("restart-replay");
        let _ = fs::remove_file(state_file.as_str());
        let start = Instant::now();

        let mut first = ServiceApiReplayGuard::from_state_file(
            8,
            Duration::from_secs(300),
            Some(state_file.as_str()),
        )
        .expect("first guard init");
        assert!(first.record_nonce_if_fresh("kamn:did:agent:alice", 42, start));
        drop(first);

        let mut restarted = ServiceApiReplayGuard::from_state_file(
            8,
            Duration::from_secs(300),
            Some(state_file.as_str()),
        )
        .expect("restarted guard init");
        assert!(!restarted.record_nonce_if_fresh(
            "kamn:did:agent:alice",
            42,
            start + Duration::from_secs(1)
        ));
        assert!(restarted.record_nonce_if_fresh(
            "kamn:did:agent:alice",
            43,
            start + Duration::from_secs(2)
        ));

        let _ = fs::remove_file(state_file.as_str());
    }

    #[test]
    fn regression_replay_guard_rejects_non_monotonic_nonce_progression() {
        // Regression: #6060
        let state_file = replay_guard_temp_state_file("monotonic-floor");
        let _ = fs::remove_file(state_file.as_str());
        let start = Instant::now();
        let mut guard = ServiceApiReplayGuard::from_state_file(
            8,
            Duration::from_secs(300),
            Some(state_file.as_str()),
        )
        .expect("guard init");

        assert!(guard.record_nonce_if_fresh("kamn:did:agent:bob", 5, start));
        assert!(!guard.record_nonce_if_fresh(
            "kamn:did:agent:bob",
            5,
            start + Duration::from_secs(1)
        ));
        assert!(!guard.record_nonce_if_fresh(
            "kamn:did:agent:bob",
            4,
            start + Duration::from_secs(2)
        ));
        assert!(guard.record_nonce_if_fresh(
            "kamn:did:agent:bob",
            6,
            start + Duration::from_secs(3)
        ));

        let _ = fs::remove_file(state_file.as_str());
    }

    #[test]
    fn unit_select_service_api_auth_public_key_falls_back_to_single_key_when_map_absent() {
        let selected = select_service_api_auth_public_key_for_sender(
            "kamn:did:agent:alice",
            None,
            Some("single-shared-key"),
        );
        assert_eq!(selected, Some("single-shared-key"));
    }

    #[test]
    fn unit_select_service_api_auth_public_key_returns_sender_specific_key_when_mapped() {
        let mut keys_by_did = BTreeMap::new();
        keys_by_did.insert(
            "kamn:did:agent:alice".to_owned(),
            "alice-key-hex".to_owned(),
        );
        keys_by_did.insert("kamn:did:agent:bob".to_owned(), "bob-key-hex".to_owned());

        let selected = select_service_api_auth_public_key_for_sender(
            "kamn:did:agent:bob",
            Some(&keys_by_did),
            Some("fallback-shared-key"),
        );
        assert_eq!(selected, Some("bob-key-hex"));
    }

    #[test]
    fn unit_select_service_api_auth_public_key_rejects_unknown_sender_when_map_configured() {
        let mut keys_by_did = BTreeMap::new();
        keys_by_did.insert(
            "kamn:did:agent:alice".to_owned(),
            "alice-key-hex".to_owned(),
        );

        let selected = select_service_api_auth_public_key_for_sender(
            "kamn:did:agent:charlie",
            Some(&keys_by_did),
            Some("fallback-shared-key"),
        );
        assert_eq!(selected, None);
    }

    #[test]
    fn regression_enforce_sender_did_binding_passes_when_mapped_did_matches_public_key() {
        // Regression: #6109
        let public_key_hex = "025f6ceceac37540cf6ef5f09d4f62c05f0b8f57fe6d8ae32a8f13f4a2eb6e940d";
        let sender_did = AgentDid::with_public_key_hex_binding("alice", public_key_hex)
            .expect("bound did should render");
        let mut keys_by_did = BTreeMap::new();
        keys_by_did.insert(sender_did.as_str().to_owned(), public_key_hex.to_owned());
        enforce_sender_did_public_key_binding_if_required(
            &sender_did,
            Some(public_key_hex),
            Some(&keys_by_did),
        )
        .expect("matching did/public-key binding should pass");
    }

    #[test]
    fn regression_enforce_sender_did_binding_fails_closed_when_binding_missing_in_map_mode() {
        // Regression: #6109
        let sender_did = AgentDid::parse("kamn:did:agent:alice").expect("did should parse");
        let public_key_hex = "025f6ceceac37540cf6ef5f09d4f62c05f0b8f57fe6d8ae32a8f13f4a2eb6e940d";
        let mut keys_by_did = BTreeMap::new();
        keys_by_did.insert(sender_did.as_str().to_owned(), public_key_hex.to_owned());
        let error = enforce_sender_did_public_key_binding_if_required(
            &sender_did,
            Some(public_key_hex),
            Some(&keys_by_did),
        )
        .expect_err("map mode should require did key-binding suffix");
        assert_eq!(error.reason_code, REASON_CODE_AUTH_DID_KEY_BINDING_INVALID);
        assert!(
            error.message.contains("key binding"),
            "binding failure should preserve deterministic context"
        );
    }
}
