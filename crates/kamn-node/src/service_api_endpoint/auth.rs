use super::*;
use kamn_kolme::{ServiceApiScope, ServiceApiScopeError};

pub(super) fn header_value<'a>(
    headers: &'a BTreeMap<String, String>,
    name: &str,
) -> Option<&'a str> {
    headers.get(name).map(String::as_str)
}

pub(super) fn authorize_service_api_request(
    snapshot: &ServiceApiSnapshot,
    request: &ParsedRequest,
    replay_guard: &mut BTreeSet<(String, u64)>,
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
    AgentDid::parse(sender_did).map_err(|error| {
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
    let state_hash = service_api_signature_state_hash(snapshot);
    if !signature_matches_supported_profile_for_fields(
        signature,
        sender_did,
        nonce,
        state_hash.as_str(),
        request.body.as_str(),
    ) {
        return Err(RequestAuthFailure::Unauthorized(
            ServiceApiReasonedError::new(
                REASON_CODE_AUTH_SIGNATURE_VERIFICATION_FAILED,
                "signature verification failed for request envelope",
            ),
        ));
    }
    if !replay_guard.insert((sender_did.to_owned(), nonce)) {
        return Err(RequestAuthFailure::Replay(ServiceApiReasonedError::new(
            REASON_CODE_AUTH_REPLAY_NONCE_DETECTED,
            "request nonce replay detected for sender",
        )));
    }
    Ok(())
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

    #[test]
    fn unit_required_scope_for_route_maps_known_route_contracts() {
        assert_eq!(
            required_scope_for_route("POST", ROUTE_MESSAGES_SEND),
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
}
