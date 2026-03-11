use super::*;

pub(super) fn route_requires_auth(method: &str, path: &str) -> bool {
    !(method == "GET" && (path == ROUTE_HEALTHZ || path == ROUTE_METRICS))
}

pub(super) fn log_service_api_event_info(
    event: &str,
    fields: &[(&str, &str)],
) -> Result<(), String> {
    log_info(event, fields).map_err(|error| format!("service api log emission failed: {error}"))
}

pub(super) fn log_service_api_event_warn(
    event: &str,
    fields: &[(&str, &str)],
) -> Result<(), String> {
    log_warn(event, fields).map_err(|error| format!("service api log emission failed: {error}"))
}

pub(super) fn service_api_request_correlation_id(request: &ParsedRequest) -> String {
    let method = request.method.to_ascii_lowercase();
    if let (Some(sender_did), Some(nonce)) = (
        super::auth::header_value(&request.headers, REQUEST_AUTH_SENDER_DID_HEADER),
        super::auth::header_value(&request.headers, REQUEST_AUTH_NONCE_HEADER),
    ) {
        return format!("service-api:{method}:{}:{sender_did}:{nonce}", request.path);
    }
    let request_tag = super::deterministic_body_tag(request.body.as_bytes());
    format!("service-api:{method}:{}:{request_tag:016x}", request.path)
}

pub(super) fn service_api_lifecycle_rejection_policy(
    reason_code: &str,
) -> Option<ServiceApiLifecycleRejectionPolicy> {
    limiter_rejection_policy(reason_code)
        .or_else(|| sender_rejection_policy(reason_code))
        .or_else(|| engine_rejection_policy(reason_code))
}

fn limiter_rejection_policy(reason_code: &str) -> Option<ServiceApiLifecycleRejectionPolicy> {
    Some(match reason_code {
        REASON_CODE_INGRESS_CONCURRENCY_LIMIT_EXCEEDED => limiter_policy(
            REASON_CODE_INGRESS_CONCURRENCY_LIMIT_EXCEEDED,
            "concurrency-limit",
            "ingress concurrency limit exceeded",
        ),
        REASON_CODE_INGRESS_RATE_LIMIT_EXCEEDED => limiter_policy(
            REASON_CODE_INGRESS_RATE_LIMIT_EXCEEDED,
            "rate-limit",
            "ingress rate limit exceeded",
        ),
        _ => return None,
    })
}

fn sender_rejection_policy(reason_code: &str) -> Option<ServiceApiLifecycleRejectionPolicy> {
    Some(match reason_code {
        REASON_CODE_INGRESS_SENDER_RATE_LIMIT_EXCEEDED => sender_policy(
            REASON_CODE_INGRESS_SENDER_RATE_LIMIT_EXCEEDED,
            "sender anti-spam rate limit exceeded",
        ),
        REASON_CODE_INGRESS_SENDER_SUSPENDED => sender_policy(
            REASON_CODE_INGRESS_SENDER_SUSPENDED,
            "sender suspended by anti-spam policy",
        ),
        REASON_CODE_INGRESS_SENDER_DUPLICATE_MESSAGE_ID => sender_policy(
            REASON_CODE_INGRESS_SENDER_DUPLICATE_MESSAGE_ID,
            "sender anti-spam duplicate message id rejected",
        ),
        REASON_CODE_INGRESS_SENDER_INSUFFICIENT_DEPOSIT => sender_policy(
            REASON_CODE_INGRESS_SENDER_INSUFFICIENT_DEPOSIT,
            "sender deposit below anti-spam minimum",
        ),
        _ => return None,
    })
}

fn engine_rejection_policy(reason_code: &str) -> Option<ServiceApiLifecycleRejectionPolicy> {
    (reason_code == REASON_CODE_INGRESS_ANTI_SPAM_ENGINE_INVALID).then_some(
        ServiceApiLifecycleRejectionPolicy {
            rejection_class: LIFECYCLE_REJECTION_CLASS_ASYNC_ENGINE,
            reason_code: REASON_CODE_INGRESS_ANTI_SPAM_ENGINE_INVALID,
            status_code: StatusCode::INTERNAL_SERVER_ERROR,
            error_label: "internal",
            outcome: "anti-spam-error",
            default_message: "anti-spam decision evaluation failed",
        },
    )
}

fn limiter_policy(
    reason_code: &'static str,
    outcome: &'static str,
    default_message: &'static str,
) -> ServiceApiLifecycleRejectionPolicy {
    ServiceApiLifecycleRejectionPolicy {
        rejection_class: LIFECYCLE_REJECTION_CLASS_ASYNC_LIMITER,
        reason_code,
        status_code: StatusCode::TOO_MANY_REQUESTS,
        error_label: "too-many-requests",
        outcome,
        default_message,
    }
}

fn sender_policy(
    reason_code: &'static str,
    default_message: &'static str,
) -> ServiceApiLifecycleRejectionPolicy {
    ServiceApiLifecycleRejectionPolicy {
        rejection_class: LIFECYCLE_REJECTION_CLASS_SENDER_ADMISSION,
        reason_code,
        status_code: StatusCode::TOO_MANY_REQUESTS,
        error_label: "too-many-requests",
        outcome: "anti-spam",
        default_message,
    }
}

#[cfg(test)]
pub(crate) fn project_service_api_lifecycle_rejection(
    reason_code: &str,
) -> Option<ServiceApiLifecycleRejectionProjection> {
    service_api_lifecycle_rejection_policy(reason_code).map(|policy| {
        ServiceApiLifecycleRejectionProjection {
            rejection_class: policy.rejection_class,
            reason_code: policy.reason_code,
            status_code: policy.status_code.as_u16(),
            error_label: policy.error_label,
            outcome: policy.outcome,
        }
    })
}

pub(super) fn emit_service_api_request_outcome(
    correlation_id: &str,
    method: &str,
    path: &str,
    status_code: u16,
    outcome: &str,
) -> Result<(), String> {
    let status_code_label = status_code.to_string();
    let fields = [
        ("correlation_id", correlation_id),
        ("method", method),
        ("path", path),
        ("status_code", status_code_label.as_str()),
        ("outcome", outcome),
    ];
    if status_code >= 400 {
        return log_service_api_event_warn("service.api.request.outcome", &fields);
    }
    log_service_api_event_info("service.api.request.outcome", &fields)
}
