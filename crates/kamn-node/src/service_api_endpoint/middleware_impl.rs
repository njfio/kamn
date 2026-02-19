use super::*;

pub(super) async fn service_api_auth_middleware(
    State(state): State<Arc<ServiceApiRuntimeState>>,
    request: Request,
    next: Next,
) -> Response {
    let method_label = request.method().to_string();
    let path = request.uri().path().to_owned();
    let is_websocket_route = method_label == "GET" && path == ROUTE_EVENTS_WS;
    let concurrency_permit = match state.concurrency_limiter.clone().try_acquire_owned() {
        Ok(permit) => permit,
        Err(_) => {
            let projection = service_api_lifecycle_rejection_policy(
                REASON_CODE_INGRESS_CONCURRENCY_LIMIT_EXCEEDED,
            )
            .unwrap_or(ServiceApiLifecycleRejectionPolicy {
                rejection_class: LIFECYCLE_REJECTION_CLASS_ASYNC_LIMITER,
                reason_code: REASON_CODE_INGRESS_CONCURRENCY_LIMIT_EXCEEDED,
                status_code: StatusCode::TOO_MANY_REQUESTS,
                error_label: "too-many-requests",
                outcome: "concurrency-limit",
                default_message: "ingress concurrency limit exceeded",
            });
            let _projection_class = projection.rejection_class;
            let correlation_id = format!(
                "service-api:{}:{}:concurrency-limit",
                method_label.to_ascii_lowercase(),
                path
            );
            return service_api_middleware_error_response(
                &state,
                ServiceApiMiddlewareError {
                    correlation_id: correlation_id.as_str(),
                    method: method_label.as_str(),
                    path: path.as_str(),
                    status_code: projection.status_code,
                    error_label: projection.error_label,
                    reason_code: projection.reason_code,
                    message: projection.default_message,
                    outcome: projection.outcome,
                },
            );
        }
    };
    // Yield once so queued requests observe bounded in-flight concurrency on the
    // single-thread runtime and deterministically fail closed when over budget.
    tokio::task::yield_now().await;

    let (mut request, parsed_request) = match parse_service_api_request(
        request,
        is_websocket_route,
        state.body_limit_bytes,
    )
    .await
    {
        Ok(parsed_request) => parsed_request,
        Err(error) => {
            let correlation_id = format!(
                "service-api:parse-error:{:016x}",
                super::deterministic_body_tag(error.message.as_bytes())
            );
            return service_api_middleware_error_response(
                &state,
                ServiceApiMiddlewareError {
                    correlation_id: correlation_id.as_str(),
                    method: "unknown",
                    path: "unknown",
                    status_code: StatusCode::BAD_REQUEST,
                    error_label: "bad-request",
                    reason_code: error.reason_code,
                    message: error.message.as_str(),
                    outcome: "bad-request",
                },
            );
        }
    };

    let correlation_id = service_api_request_correlation_id(&parsed_request);
    if let Err(reason) = log_service_api_event_info(
        "service.api.request.received",
        &[
            ("correlation_id", correlation_id.as_str()),
            ("method", parsed_request.method.as_str()),
            ("path", parsed_request.path.as_str()),
        ],
    ) {
        return service_api_middleware_error_response(
            &state,
            ServiceApiMiddlewareError {
                correlation_id: correlation_id.as_str(),
                method: parsed_request.method.as_str(),
                path: parsed_request.path.as_str(),
                status_code: StatusCode::INTERNAL_SERVER_ERROR,
                error_label: "internal",
                reason_code: REASON_CODE_REQUEST_LOG_EMISSION_FAILED,
                message: reason.as_str(),
                outcome: "log-error",
            },
        );
    }

    {
        let mut replay_guard = state.replay_guard.lock().await;
        if let Err(error) = super::authorize_service_api_request(
            &state.snapshot,
            &parsed_request,
            &mut replay_guard,
        ) {
            let (status_code, error_label, auth_error, outcome) = match error {
                RequestAuthFailure::Unauthorized(reasoned_error) => (
                    StatusCode::UNAUTHORIZED,
                    "unauthorized",
                    reasoned_error,
                    "unauthorized",
                ),
                RequestAuthFailure::Replay(reasoned_error) => {
                    (StatusCode::CONFLICT, "replay", reasoned_error, "replay")
                }
            };
            return service_api_middleware_error_response(
                &state,
                ServiceApiMiddlewareError {
                    correlation_id: correlation_id.as_str(),
                    method: parsed_request.method.as_str(),
                    path: parsed_request.path.as_str(),
                    status_code,
                    error_label,
                    reason_code: auth_error.reason_code,
                    message: auth_error.message.as_str(),
                    outcome,
                },
            );
        }
    }

    if let Err(error) = super::enforce_sender_anti_spam(&state, &parsed_request).await {
        let projection = service_api_lifecycle_rejection_policy(error.reason_code).unwrap_or(
            ServiceApiLifecycleRejectionPolicy {
                rejection_class: LIFECYCLE_REJECTION_CLASS_SENDER_ADMISSION,
                reason_code: error.reason_code,
                status_code: StatusCode::TOO_MANY_REQUESTS,
                error_label: "too-many-requests",
                outcome: "anti-spam",
                default_message: "sender request rejected by anti-spam policy",
            },
        );
        let _projection_class = projection.rejection_class;
        return service_api_middleware_error_response(
            &state,
            ServiceApiMiddlewareError {
                correlation_id: correlation_id.as_str(),
                method: parsed_request.method.as_str(),
                path: parsed_request.path.as_str(),
                status_code: projection.status_code,
                error_label: projection.error_label,
                reason_code: projection.reason_code,
                message: error.message.as_str(),
                outcome: projection.outcome,
            },
        );
    }

    if route_requires_auth(parsed_request.method.as_str(), parsed_request.path.as_str()) {
        let mut ingress_rate_window = state.ingress_rate_window.lock().await;
        if !ingress_rate_window.try_record_request(Instant::now()) {
            let projection =
                service_api_lifecycle_rejection_policy(REASON_CODE_INGRESS_RATE_LIMIT_EXCEEDED)
                    .unwrap_or(ServiceApiLifecycleRejectionPolicy {
                        rejection_class: LIFECYCLE_REJECTION_CLASS_ASYNC_LIMITER,
                        reason_code: REASON_CODE_INGRESS_RATE_LIMIT_EXCEEDED,
                        status_code: StatusCode::TOO_MANY_REQUESTS,
                        error_label: "too-many-requests",
                        outcome: "rate-limit",
                        default_message: "ingress rate limit exceeded",
                    });
            let _projection_class = projection.rejection_class;
            return service_api_middleware_error_response(
                &state,
                ServiceApiMiddlewareError {
                    correlation_id: correlation_id.as_str(),
                    method: parsed_request.method.as_str(),
                    path: parsed_request.path.as_str(),
                    status_code: projection.status_code,
                    error_label: projection.error_label,
                    reason_code: projection.reason_code,
                    message: projection.default_message,
                    outcome: projection.outcome,
                },
            );
        }
    }

    if let Err(error) =
        super::validate_websocket_route_requirements(is_websocket_route, &parsed_request.headers)
    {
        return service_api_middleware_error_response(
            &state,
            ServiceApiMiddlewareError {
                correlation_id: correlation_id.as_str(),
                method: parsed_request.method.as_str(),
                path: parsed_request.path.as_str(),
                status_code: StatusCode::BAD_REQUEST,
                error_label: "bad-request",
                reason_code: error.reason_code,
                message: error.message.as_str(),
                outcome: "websocket-bad-request",
            },
        );
    }

    let method_for_outcome = parsed_request.method.clone();
    let path_for_outcome = parsed_request.path.clone();
    request.extensions_mut().insert(ServiceApiRequestContext {
        parsed_request,
        correlation_id: correlation_id.clone(),
    });

    let response = next.run(request).await;
    let outcome = response
        .extensions()
        .get::<ServiceApiRequestOutcome>()
        .map(|outcome| outcome.0)
        .unwrap_or_else(|| {
            if is_websocket_route && response.status().is_client_error() {
                "websocket-bad-request"
            } else {
                "handled"
            }
        });
    let _ = emit_service_api_request_outcome(
        correlation_id.as_str(),
        method_for_outcome.as_str(),
        path_for_outcome.as_str(),
        response.status().as_u16(),
        outcome,
    );
    state.request_budget.record_request();
    drop(concurrency_permit);
    response
}

pub(super) async fn parse_service_api_request(
    request: Request,
    is_websocket_route: bool,
    body_limit_bytes: usize,
) -> Result<(Request, ParsedRequest), ServiceApiReasonedError> {
    let method_label = request.method().to_string();
    let path = request.uri().path().to_owned();
    let headers = request.headers().clone();

    if is_websocket_route {
        let parsed_request =
            build_parsed_request(method_label.as_str(), path.as_str(), &headers, Bytes::new())?;
        return Ok((request, parsed_request));
    }

    let (parts, body) = request.into_parts();
    let body_limit = body_limit_bytes;
    let body = to_bytes(body, body_limit).await.map_err(|error| {
        let message = error.to_string();
        if message.contains("length limit exceeded") {
            ServiceApiReasonedError::new(
                REASON_CODE_INGRESS_BODY_SIZE_LIMIT_EXCEEDED,
                format!("request body size limit exceeded: {body_limit} bytes"),
            )
        } else {
            ServiceApiReasonedError::new(
                REASON_CODE_REQUEST_READ_FAILED,
                format!("request read failed: {error}"),
            )
        }
    })?;
    let parsed_request =
        build_parsed_request(method_label.as_str(), path.as_str(), &headers, body.clone())?;
    let request = Request::from_parts(parts, Body::from(body));
    Ok((request, parsed_request))
}

pub(super) fn service_api_middleware_error_response(
    state: &ServiceApiRuntimeState,
    error: ServiceApiMiddlewareError<'_>,
) -> Response {
    let response = super::json_error_response(
        error.status_code,
        error.error_label,
        error.reason_code,
        error.message,
    );
    let _ = emit_service_api_request_outcome(
        error.correlation_id,
        error.method,
        error.path,
        error.status_code.as_u16(),
        error.outcome,
    );
    state.request_budget.record_request();
    response
}

pub(super) async fn handle_service_api_http_route(
    State(state): State<Arc<ServiceApiRuntimeState>>,
    Extension(context): Extension<ServiceApiRequestContext>,
) -> Response {
    let _ = context.correlation_id.as_str();
    let rendered = super::render_service_api_endpoint_response(
        &state.snapshot,
        context.parsed_request.method.as_str(),
        context.parsed_request.path.as_str(),
        context.parsed_request.body.as_str(),
    );
    super::contract_response(rendered)
}

pub(super) async fn handle_service_api_websocket_route(
    State(state): State<Arc<ServiceApiRuntimeState>>,
    Extension(context): Extension<ServiceApiRequestContext>,
    upgrade: WebSocketUpgrade,
) -> Response {
    let _ = context.correlation_id.as_str();
    let mut response = super::websocket_upgrade_response(upgrade, state.snapshot.clone());
    response
        .extensions_mut()
        .insert(ServiceApiRequestOutcome("websocket-upgrade"));
    response
}

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
        super::header_value(&request.headers, REQUEST_AUTH_SENDER_DID_HEADER),
        super::header_value(&request.headers, REQUEST_AUTH_NONCE_HEADER),
    ) {
        return format!("service-api:{method}:{}:{sender_did}:{nonce}", request.path);
    }
    let request_tag = super::deterministic_body_tag(request.body.as_bytes());
    format!("service-api:{method}:{}:{request_tag:016x}", request.path)
}

pub(super) fn service_api_lifecycle_rejection_policy(
    reason_code: &str,
) -> Option<ServiceApiLifecycleRejectionPolicy> {
    let policy = match reason_code {
        REASON_CODE_INGRESS_CONCURRENCY_LIMIT_EXCEEDED => ServiceApiLifecycleRejectionPolicy {
            rejection_class: LIFECYCLE_REJECTION_CLASS_ASYNC_LIMITER,
            reason_code: REASON_CODE_INGRESS_CONCURRENCY_LIMIT_EXCEEDED,
            status_code: StatusCode::TOO_MANY_REQUESTS,
            error_label: "too-many-requests",
            outcome: "concurrency-limit",
            default_message: "ingress concurrency limit exceeded",
        },
        REASON_CODE_INGRESS_RATE_LIMIT_EXCEEDED => ServiceApiLifecycleRejectionPolicy {
            rejection_class: LIFECYCLE_REJECTION_CLASS_ASYNC_LIMITER,
            reason_code: REASON_CODE_INGRESS_RATE_LIMIT_EXCEEDED,
            status_code: StatusCode::TOO_MANY_REQUESTS,
            error_label: "too-many-requests",
            outcome: "rate-limit",
            default_message: "ingress rate limit exceeded",
        },
        REASON_CODE_INGRESS_SENDER_RATE_LIMIT_EXCEEDED => ServiceApiLifecycleRejectionPolicy {
            rejection_class: LIFECYCLE_REJECTION_CLASS_SENDER_ADMISSION,
            reason_code: REASON_CODE_INGRESS_SENDER_RATE_LIMIT_EXCEEDED,
            status_code: StatusCode::TOO_MANY_REQUESTS,
            error_label: "too-many-requests",
            outcome: "anti-spam",
            default_message: "sender anti-spam rate limit exceeded",
        },
        REASON_CODE_INGRESS_SENDER_SUSPENDED => ServiceApiLifecycleRejectionPolicy {
            rejection_class: LIFECYCLE_REJECTION_CLASS_SENDER_ADMISSION,
            reason_code: REASON_CODE_INGRESS_SENDER_SUSPENDED,
            status_code: StatusCode::TOO_MANY_REQUESTS,
            error_label: "too-many-requests",
            outcome: "anti-spam",
            default_message: "sender suspended by anti-spam policy",
        },
        REASON_CODE_INGRESS_SENDER_DUPLICATE_MESSAGE_ID => ServiceApiLifecycleRejectionPolicy {
            rejection_class: LIFECYCLE_REJECTION_CLASS_SENDER_ADMISSION,
            reason_code: REASON_CODE_INGRESS_SENDER_DUPLICATE_MESSAGE_ID,
            status_code: StatusCode::TOO_MANY_REQUESTS,
            error_label: "too-many-requests",
            outcome: "anti-spam",
            default_message: "sender anti-spam duplicate message id rejected",
        },
        REASON_CODE_INGRESS_SENDER_INSUFFICIENT_DEPOSIT => ServiceApiLifecycleRejectionPolicy {
            rejection_class: LIFECYCLE_REJECTION_CLASS_SENDER_ADMISSION,
            reason_code: REASON_CODE_INGRESS_SENDER_INSUFFICIENT_DEPOSIT,
            status_code: StatusCode::TOO_MANY_REQUESTS,
            error_label: "too-many-requests",
            outcome: "anti-spam",
            default_message: "sender deposit below anti-spam minimum",
        },
        REASON_CODE_INGRESS_ANTI_SPAM_ENGINE_INVALID => ServiceApiLifecycleRejectionPolicy {
            rejection_class: LIFECYCLE_REJECTION_CLASS_ASYNC_ENGINE,
            reason_code: REASON_CODE_INGRESS_ANTI_SPAM_ENGINE_INVALID,
            status_code: StatusCode::INTERNAL_SERVER_ERROR,
            error_label: "internal",
            outcome: "anti-spam-error",
            default_message: "anti-spam decision evaluation failed",
        },
        _ => return None,
    };
    Some(policy)
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
        log_service_api_event_warn("service.api.request.outcome", &fields)
    } else {
        log_service_api_event_info("service.api.request.outcome", &fields)
    }
}

pub(super) fn build_parsed_request(
    method: &str,
    path: &str,
    headers: &HeaderMap,
    body: Bytes,
) -> Result<ParsedRequest, ServiceApiReasonedError> {
    let mut normalized_headers = BTreeMap::new();
    for (header_name, header_value) in headers {
        let value = header_value.to_str().map_err(|_| {
            ServiceApiReasonedError::new(
                REASON_CODE_REQUEST_HEADER_UTF8_INVALID,
                format!("request header value was not valid utf-8: {header_name}"),
            )
        })?;
        normalized_headers.insert(
            header_name.as_str().to_ascii_lowercase(),
            value.trim().to_owned(),
        );
    }
    let body = String::from_utf8(body.to_vec()).map_err(|_| {
        ServiceApiReasonedError::new(
            REASON_CODE_REQUEST_BODY_UTF8_INVALID,
            "request was not valid utf-8",
        )
    })?;
    Ok(ParsedRequest {
        method: method.to_owned(),
        path: path.to_owned(),
        body,
        headers: normalized_headers,
    })
}
