use super::super::super::*;

pub(crate) async fn enforce_transaction_authorization(
    state: &Arc<ServiceApiRuntimeState>,
    parsed_request: &ParsedRequest,
    correlation_id: &str,
    request_started_at: Instant,
) -> Result<(), Response> {
    let target = match super::auth::resolve_transaction_authorization_target(parsed_request) {
        Ok(Some(target)) => target,
        Ok(None) => return Ok(()),
        Err(error) => {
            return Err(authorization_response(
                state,
                parsed_request,
                correlation_id,
                request_started_at,
                error.reason_code,
                error.message.as_str(),
            )
            .await)
        }
    };
    let receipt_correlation_id = redacted_receipt_correlation_id(correlation_id);
    let request = message_store::ServiceApiAuthorizationRequest {
        correlation_id: receipt_correlation_id.as_str(),
        actor_did: target.actor_did.as_str(),
        resource: target.resource.as_str(),
        action: target.action,
        role: target.role,
    };
    let decision = state
        .message_store
        .lock()
        .await
        .authorize_transaction_action(request);
    resolve_authorization_decision(
        state,
        parsed_request,
        correlation_id,
        request_started_at,
        decision,
    )
    .await
}

async fn resolve_authorization_decision(
    state: &Arc<ServiceApiRuntimeState>,
    parsed_request: &ParsedRequest,
    correlation_id: &str,
    request_started_at: Instant,
    decision: Result<message_store::ServiceApiAuthorizationDecision, String>,
) -> Result<(), Response> {
    match decision {
        Ok(decision) if decision.allowed => Ok(()),
        Ok(decision) => Err(authorization_response(
            state,
            parsed_request,
            correlation_id,
            request_started_at,
            decision.reason_code,
            "transaction action is not authorized",
        )
        .await),
        Err(error) => Err(super::super::internal_response(
            state,
            request_started_at,
            parsed_request,
            super::super::InternalResponseProjection {
                correlation_id,
                reason_code: REASON_CODE_STATE_PERSISTENCE_FAILED,
                outcome: "persistence",
                error_label: "internal",
                status_code: StatusCode::INTERNAL_SERVER_ERROR,
                message: error.as_str(),
            },
        )
        .await),
    }
}

async fn authorization_response(
    state: &Arc<ServiceApiRuntimeState>,
    parsed_request: &ParsedRequest,
    correlation_id: &str,
    request_started_at: Instant,
    reason_code: &'static str,
    message: &str,
) -> Response {
    super::super::internal_response(
        state,
        request_started_at,
        parsed_request,
        super::super::InternalResponseProjection {
            correlation_id,
            reason_code,
            outcome: "forbidden",
            error_label: "forbidden",
            status_code: StatusCode::FORBIDDEN,
            message,
        },
    )
    .await
}

fn redacted_receipt_correlation_id(correlation_id: &str) -> String {
    format!(
        "service-api-authz:{:016x}",
        deterministic_body_tag(correlation_id.as_bytes())
    )
}
