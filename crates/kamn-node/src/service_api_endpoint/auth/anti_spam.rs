use super::support::header_value;
use super::*;

pub(crate) async fn enforce_sender_anti_spam(
    state: &ServiceApiRuntimeState,
    request: &ParsedRequest,
) -> Result<(), ServiceApiReasonedError> {
    if !super::route_requires_auth(request.method.as_str(), request.path.as_str()) {
        return Ok(());
    }

    let sender_did = require_sender_did(request)?;
    let nonce = require_nonce(request)?;
    let decision =
        evaluate_anti_spam_decision(state, request.path.as_str(), sender_did, nonce).await?;
    match decision {
        AntiSpamDecision::Accepted => Ok(()),
        AntiSpamDecision::Rejected(rejection) => {
            Err(map_anti_spam_rejection_to_reasoned_error(rejection))
        }
    }
}

async fn evaluate_anti_spam_decision(
    state: &ServiceApiRuntimeState,
    path: &str,
    sender_did: &str,
    nonce: u64,
) -> Result<AntiSpamDecision, ServiceApiReasonedError> {
    let now_unix = current_unix_timestamp()?;
    let mut anti_spam = state.sender_anti_spam.lock().await;
    anti_spam
        .evaluate(
            sender_did,
            anti_spam_message_id(sender_did, nonce, path).as_str(),
            now_unix,
        )
        .map_err(anti_spam_engine_error)
}

pub(crate) fn map_anti_spam_rejection_to_reasoned_error(
    rejection: AntiSpamRejection,
) -> ServiceApiReasonedError {
    match rejection {
        AntiSpamRejection::InsufficientDeposit { required, provided } => {
            insufficient_deposit_error(required, provided)
        }
        AntiSpamRejection::RateLimitExceeded {
            limit,
            observed,
            window_seconds,
        } => rate_limit_error(limit, observed, window_seconds),
        AntiSpamRejection::SenderSuspended { until_unix } => sender_suspended_error(until_unix),
        AntiSpamRejection::DuplicateMessageId(message_id) => duplicate_message_id_error(message_id),
    }
}

fn require_sender_did(request: &ParsedRequest) -> Result<&str, ServiceApiReasonedError> {
    header_value(&request.headers, REQUEST_AUTH_SENDER_DID_HEADER).ok_or_else(|| {
        ServiceApiReasonedError::new(
            REASON_CODE_AUTH_SENDER_DID_HEADER_MISSING,
            format!("missing required header: {REQUEST_AUTH_SENDER_DID_HEADER}"),
        )
    })
}

fn require_nonce(request: &ParsedRequest) -> Result<u64, ServiceApiReasonedError> {
    let nonce_raw = header_value(&request.headers, REQUEST_AUTH_NONCE_HEADER)
        .ok_or_else(missing_nonce_error)?;
    nonce_raw.parse::<u64>().map_err(|_| invalid_nonce_error())
}

fn current_unix_timestamp() -> Result<u64, ServiceApiReasonedError> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .map_err(|error| {
            ServiceApiReasonedError::new(
                REASON_CODE_INGRESS_ANTI_SPAM_ENGINE_INVALID,
                format!("anti-spam clock evaluation failed: {error}"),
            )
        })
}

fn anti_spam_message_id(sender_did: &str, nonce: u64, path: &str) -> String {
    format!("{sender_did}:{nonce}:{path}")
}

fn anti_spam_engine_error(error: impl std::fmt::Display) -> ServiceApiReasonedError {
    ServiceApiReasonedError::new(
        REASON_CODE_INGRESS_ANTI_SPAM_ENGINE_INVALID,
        format!("anti-spam decision evaluation failed: {error}"),
    )
}

fn missing_nonce_error() -> ServiceApiReasonedError {
    ServiceApiReasonedError::new(
        REASON_CODE_AUTH_NONCE_HEADER_MISSING,
        format!("missing required header: {REQUEST_AUTH_NONCE_HEADER}"),
    )
}

fn invalid_nonce_error() -> ServiceApiReasonedError {
    ServiceApiReasonedError::new(
        REASON_CODE_AUTH_NONCE_INVALID,
        format!("invalid request nonce header: {REQUEST_AUTH_NONCE_HEADER}"),
    )
}

fn insufficient_deposit_error(required: u64, provided: u64) -> ServiceApiReasonedError {
    ServiceApiReasonedError::new(
        REASON_CODE_INGRESS_SENDER_INSUFFICIENT_DEPOSIT,
        format!("sender deposit below anti-spam minimum: required={required}, provided={provided}"),
    )
}

fn rate_limit_error(limit: usize, observed: usize, window_seconds: u64) -> ServiceApiReasonedError {
    ServiceApiReasonedError::new(
        REASON_CODE_INGRESS_SENDER_RATE_LIMIT_EXCEEDED,
        format!(
            "sender anti-spam rate limit exceeded: observed={observed}, limit={limit}, window_seconds={window_seconds}"
        ),
    )
}

fn sender_suspended_error(until_unix: u64) -> ServiceApiReasonedError {
    ServiceApiReasonedError::new(
        REASON_CODE_INGRESS_SENDER_SUSPENDED,
        format!("sender suspended by anti-spam policy until unix={until_unix}"),
    )
}

fn duplicate_message_id_error(message_id: String) -> ServiceApiReasonedError {
    ServiceApiReasonedError::new(
        REASON_CODE_INGRESS_SENDER_DUPLICATE_MESSAGE_ID,
        format!("sender anti-spam duplicate message id rejected: {message_id}"),
    )
}
