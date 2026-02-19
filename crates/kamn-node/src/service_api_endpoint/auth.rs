use super::*;

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
