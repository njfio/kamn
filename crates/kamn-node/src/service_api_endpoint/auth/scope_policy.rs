use super::support::header_value;
use super::*;
use kamn_kolme::{ServiceApiScope, ServiceApiScopeError};

pub(crate) fn enforce_request_scope_policy(
    request: &ParsedRequest,
) -> Result<(), ServiceApiReasonedError> {
    let Some(expected_scope) =
        required_scope_for_route(request.method.as_str(), request.path.as_str())
    else {
        return Ok(());
    };
    let scope = require_scope_header(request)?;
    let parsed_scope = parse_scope(scope)?;
    if parsed_scope == expected_scope {
        return Ok(());
    }
    Err(scope_route_mismatch_error(request, parsed_scope))
}

pub(super) fn required_scope_for_route(method: &str, path: &str) -> Option<ServiceApiScope> {
    if !super::route_requires_auth(method, path) {
        return None;
    }
    Some(
        post_scope(method, path)
            .or_else(|| get_scope(method, path))
            .unwrap_or(ServiceApiScope::ProtectedUnknown),
    )
}

pub(super) fn parse_scope(scope: &str) -> Result<ServiceApiScope, ServiceApiReasonedError> {
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

fn require_scope_header(request: &ParsedRequest) -> Result<&str, ServiceApiReasonedError> {
    header_value(&request.headers, REQUEST_AUTH_SCOPE_HEADER).ok_or_else(|| {
        ServiceApiReasonedError::new(
            REASON_CODE_AUTH_SCOPE_HEADER_MISSING,
            format!("missing required header: {REQUEST_AUTH_SCOPE_HEADER}"),
        )
    })
}

fn scope_route_mismatch_error(
    request: &ParsedRequest,
    parsed_scope: ServiceApiScope,
) -> ServiceApiReasonedError {
    ServiceApiReasonedError::new(
        REASON_CODE_AUTH_SCOPE_ROUTE_MISMATCH,
        format!(
            "scope {} is not authorized for route {} {}",
            parsed_scope.as_str(),
            request.method,
            request.path
        ),
    )
}

fn post_scope(method: &str, path: &str) -> Option<ServiceApiScope> {
    if method != "POST" {
        return None;
    }
    post_exact_scope(path).or_else(|| post_dynamic_scope(path))
}

fn post_exact_scope(path: &str) -> Option<ServiceApiScope> {
    Some(match path {
        ROUTE_MESSAGES_SEND | ROUTE_MESSAGES_RELAY => ServiceApiScope::MessagesWrite,
        ROUTE_CHANNELS_CREATE => ServiceApiScope::ChannelsWrite,
        ROUTE_AGENTS_SEARCH => ServiceApiScope::AgentsRead,
        ROUTE_AGENTS_REGISTER => ServiceApiScope::AgentsWrite,
        ROUTE_TASKS_CREATE => ServiceApiScope::TasksWrite,
        ROUTE_ESCROW_FUND => ServiceApiScope::EscrowWrite,
        ROUTE_CONTENT_REGISTER => ServiceApiScope::ContentWrite,
        ROUTE_BRIDGE_SUBMIT => ServiceApiScope::BridgeWrite,
        _ => return None,
    })
}

fn post_dynamic_scope(path: &str) -> Option<ServiceApiScope> {
    Some(match path {
        _ if super::payload::task_accept_path_id(path).is_some() => ServiceApiScope::TasksWrite,
        _ if super::payload::task_complete_path_id(path).is_some() => ServiceApiScope::TasksWrite,
        _ if super::payload::escrow_release_path_id(path).is_some() => ServiceApiScope::EscrowWrite,
        _ if super::payload::content_expire_path_id(path).is_some() => {
            ServiceApiScope::ContentWrite
        }
        _ if super::payload::content_tombstone_path_id(path).is_some() => {
            ServiceApiScope::ContentWrite
        }
        _ if super::payload::bridge_forward_path_id(path).is_some() => ServiceApiScope::BridgeWrite,
        _ => return None,
    })
}

fn get_scope(method: &str, path: &str) -> Option<ServiceApiScope> {
    if method != "GET" {
        return None;
    }
    Some(match path {
        ROUTE_EVENTS_WS => ServiceApiScope::EventsRead,
        _ if super::payload::content_path_id(path).is_some() => ServiceApiScope::ContentRead,
        _ if super::payload::bridge_path_id(path).is_some() => ServiceApiScope::BridgeRead,
        _ if super::payload::message_path_id(path).is_some() => ServiceApiScope::MessagesRead,
        _ if super::payload::channel_messages_path_id(path).is_some() => {
            ServiceApiScope::ChannelsRead
        }
        _ if super::payload::task_path_id(path).is_some() => ServiceApiScope::TasksRead,
        _ if super::payload::task_participant_view_path_id(path).is_some() => {
            ServiceApiScope::TasksRead
        }
        _ if super::payload::task_verifier_view_path_id(path).is_some() => {
            ServiceApiScope::TasksRead
        }
        _ if super::payload::agent_balance_path_id(path).is_some() => ServiceApiScope::AgentsRead,
        _ if super::payload::agent_path_id(path).is_some() => ServiceApiScope::AgentsRead,
        _ => return None,
    })
}
