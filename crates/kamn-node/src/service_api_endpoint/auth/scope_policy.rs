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
    let scope = header_value(&request.headers, REQUEST_AUTH_SCOPE_HEADER).ok_or_else(|| {
        ServiceApiReasonedError::new(
            REASON_CODE_AUTH_SCOPE_HEADER_MISSING,
            format!("missing required header: {REQUEST_AUTH_SCOPE_HEADER}"),
        )
    })?;
    let parsed_scope = parse_scope(scope)?;
    if parsed_scope == expected_scope {
        return Ok(());
    }
    Err(ServiceApiReasonedError::new(
        REASON_CODE_AUTH_SCOPE_ROUTE_MISMATCH,
        format!(
            "scope {} is not authorized for route {} {}",
            parsed_scope.as_str(),
            request.method,
            request.path
        ),
    ))
}

pub(super) fn required_scope_for_route(method: &str, path: &str) -> Option<ServiceApiScope> {
    if !super::route_requires_auth(method, path) {
        return None;
    }
    Some(match (method, path) {
        ("POST", ROUTE_MESSAGES_SEND) | ("POST", ROUTE_MESSAGES_RELAY) => {
            ServiceApiScope::MessagesWrite
        }
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
    })
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
