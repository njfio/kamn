use super::support::header_value;
use super::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct TransactionAuthorizationTarget {
    pub(crate) actor_did: String,
    pub(crate) resource: String,
    pub(crate) action: &'static str,
    pub(crate) role: &'static str,
}

pub(crate) fn resolve_transaction_authorization_target(
    request: &ParsedRequest,
) -> Result<Option<TransactionAuthorizationTarget>, ServiceApiReasonedError> {
    if request.method == "POST" && request.path == ROUTE_AGENTS_REGISTER {
        return Ok(None);
    }
    let Some((resource, action, role)) = route_target(request)? else {
        return Ok(None);
    };
    let actor_did = header_value(&request.headers, REQUEST_AUTH_SENDER_DID_HEADER)
        .ok_or_else(missing_actor_error)?
        .to_owned();
    Ok(Some(TransactionAuthorizationTarget {
        actor_did,
        resource,
        action,
        role,
    }))
}

fn route_target(
    request: &ParsedRequest,
) -> Result<Option<(String, &'static str, &'static str)>, ServiceApiReasonedError> {
    let method = request.method.as_str();
    let path = request.path.as_str();
    if method == "POST" && path == ROUTE_TASKS_CREATE {
        return Ok(Some((
            "transaction:new".to_owned(),
            "task:create",
            "initiator",
        )));
    }
    if method == "POST" && path == ROUTE_ESCROW_FUND {
        return Ok(Some((
            escrow_fund_resource(request)?,
            "escrow:fund",
            "initiator",
        )));
    }
    Ok(dynamic_route_target(method, path))
}

fn dynamic_route_target(method: &str, path: &str) -> Option<(String, &'static str, &'static str)> {
    if method == "GET" {
        if super::payload::task_participant_view_path_id(path).is_some()
            || super::payload::task_verifier_view_path_id(path).is_some()
        {
            return None;
        }
        return super::payload::task_path_id(path)
            .map(|id| (task_resource(id), "task:read", "participant"));
    }
    if method != "POST" {
        return None;
    }
    if let Some(id) = super::payload::task_accept_path_id(path) {
        return Some((task_resource(id), "task:accept", "provider"));
    }
    if let Some(id) = super::payload::task_complete_path_id(path) {
        return Some((task_resource(id), "task:complete", "provider"));
    }
    super::payload::escrow_release_path_id(path)
        .map(|id| (format!("escrow:{id}"), "escrow:release", "initiator"))
}

fn escrow_fund_resource(request: &ParsedRequest) -> Result<String, ServiceApiReasonedError> {
    super::message_store::escrow_fund_task_id(request.body.as_str())
        .map(|task_id| task_resource(task_id.as_str()))
        .map_err(|error| unresolved_resource_error(error.as_str()))
}

fn task_resource(task_id: &str) -> String {
    format!("task:{task_id}")
}

fn missing_actor_error() -> ServiceApiReasonedError {
    ServiceApiReasonedError::new(
        REASON_CODE_AUTH_SENDER_DID_HEADER_MISSING,
        "verified authorization actor did is missing",
    )
}

fn unresolved_resource_error(message: &str) -> ServiceApiReasonedError {
    ServiceApiReasonedError::new(REASON_CODE_RESOURCE_ROLE_MISMATCH, message)
}
