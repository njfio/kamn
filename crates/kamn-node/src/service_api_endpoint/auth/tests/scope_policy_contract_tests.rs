use super::super::scope_policy::{parse_scope, required_scope_for_route};
use super::super::*;
use kamn_kolme::ServiceApiScope;

#[test]
fn unit_required_scope_for_route_maps_post_write_contracts() {
    for (path, expected) in [
        (ROUTE_MESSAGES_SEND, ServiceApiScope::MessagesWrite),
        (ROUTE_MESSAGES_RELAY, ServiceApiScope::MessagesWrite),
        (ROUTE_CHANNELS_CREATE, ServiceApiScope::ChannelsWrite),
        (ROUTE_TASKS_CREATE, ServiceApiScope::TasksWrite),
        ("/v1/tasks/task-1/accept", ServiceApiScope::TasksWrite),
        ("/v1/tasks/task-1/complete", ServiceApiScope::TasksWrite),
        (ROUTE_ESCROW_FUND, ServiceApiScope::EscrowWrite),
        ("/v1/escrow/escrow-1/release", ServiceApiScope::EscrowWrite),
        (ROUTE_CONTENT_REGISTER, ServiceApiScope::ContentWrite),
        (
            "/v1/content/content-1/expire",
            ServiceApiScope::ContentWrite,
        ),
        (
            "/v1/content/content-1/tombstone",
            ServiceApiScope::ContentWrite,
        ),
        (ROUTE_BRIDGE_SUBMIT, ServiceApiScope::BridgeWrite),
        ("/v1/bridge/bridge-1/forward", ServiceApiScope::BridgeWrite),
    ] {
        assert_required_scope("POST", path, expected);
    }
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
fn unit_required_scope_for_route_maps_get_read_contracts() {
    for (path, expected) in [
        (ROUTE_EVENTS_WS, ServiceApiScope::EventsRead),
        ("/v1/content/content-1", ServiceApiScope::ContentRead),
        ("/v1/bridge/bridge-1", ServiceApiScope::BridgeRead),
        ("/v1/messages/message-1", ServiceApiScope::MessagesRead),
        (
            "/v1/channels/channel-1/messages",
            ServiceApiScope::ChannelsRead,
        ),
        ("/v1/tasks/task-1", ServiceApiScope::TasksRead),
        (
            "/v1/agents/kamn:did:agent:alice",
            ServiceApiScope::AgentsRead,
        ),
        (
            "/v1/agents/kamn:did:agent:alice/balance",
            ServiceApiScope::AgentsRead,
        ),
    ] {
        assert_required_scope("GET", path, expected);
    }
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
fn unit_parse_scope_accepts_trimmed_write_values() {
    assert_parsed_scope(" messages:write ", ServiceApiScope::MessagesWrite);
    assert_parsed_scope(" content:write ", ServiceApiScope::ContentWrite);
    assert_parsed_scope(" bridge:write ", ServiceApiScope::BridgeWrite);
    assert_eq!(
        parse_scope(" agents:write ").expect("scope").as_str(),
        "agents:write"
    );
}

#[test]
fn unit_parse_scope_accepts_trimmed_read_and_unknown_values() {
    assert_parsed_scope("tasks:read", ServiceApiScope::TasksRead);
    assert_parsed_scope("bridge:read", ServiceApiScope::BridgeRead);
    assert_parsed_scope("protected:unknown", ServiceApiScope::ProtectedUnknown);
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

fn assert_required_scope(method: &str, path: &str, expected: ServiceApiScope) {
    assert_eq!(required_scope_for_route(method, path), Some(expected));
}

fn assert_parsed_scope(scope: &str, expected: ServiceApiScope) {
    assert_eq!(parse_scope(scope).expect("scope"), expected);
}
