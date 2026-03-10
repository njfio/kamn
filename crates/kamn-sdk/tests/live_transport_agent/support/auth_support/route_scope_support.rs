pub(crate) fn required_scope_for_route(method: &str, path: &str) -> Option<&'static str> {
    Some(match (method, path) {
        ("POST", "/v1/agents/register") => "agents:write",
        ("POST", "/v1/agents/search") => "agents:read",
        ("POST", "/v1/channels/create") => "channels:write",
        ("POST", "/v1/messages/send") => "messages:write",
        ("GET", _) if path.starts_with("/v1/messages/") => "messages:read",
        ("GET", _) if path.starts_with("/v1/agents/") => "agents:read",
        _ => return None,
    })
}
