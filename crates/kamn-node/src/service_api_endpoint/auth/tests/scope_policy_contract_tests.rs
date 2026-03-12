use super::super::scope_policy::{parse_scope, required_scope_for_route};
use super::super::*;
use kamn_kolme::ServiceApiScope;

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
    assert_eq!(
        required_scope_for_route("GET", "/v1/agents/kamn:did:agent:alice/balance"),
        Some(ServiceApiScope::AgentsRead)
    );
}

#[test]
fn regression_required_scope_for_route_preserves_public_and_unknown_contracts() {
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
        parse_scope(" agents:write ").expect("scope").as_str(),
        "agents:write"
    );
    assert_eq!(
        parse_scope("protected:unknown").expect("scope"),
        ServiceApiScope::ProtectedUnknown
    );
}

#[test]
fn regression_required_scope_for_route_maps_agent_registration_to_agents_write() {
    assert_eq!(
        required_scope_for_route("POST", "/v1/agents/register").map(ServiceApiScope::as_str),
        Some("agents:write")
    );
}

#[test]
fn regression_required_scope_for_route_maps_agent_search_to_agents_read() {
    assert_eq!(
        required_scope_for_route("POST", "/v1/agents/search").map(ServiceApiScope::as_str),
        Some("agents:read")
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
