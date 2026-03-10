pub(crate) fn required_scope_for_route(method: &str, path: &str) -> Option<&'static str> {
    if !route_requires_auth(method, path) {
        return None;
    }
    Some(
        post_scope(method, path)
            .or_else(|| get_scope(method, path))
            .unwrap_or("protected:unknown"),
    )
}

pub(crate) fn route_requires_auth(method: &str, path: &str) -> bool {
    !(method == "GET" && (path == "/healthz" || path == "/metrics"))
}

fn post_scope(method: &str, path: &str) -> Option<&'static str> {
    match (method, path) {
        ("POST", "/v1/messages/send") => Some("messages:write"),
        ("POST", "/v1/channels/create") => Some("channels:write"),
        ("POST", "/v1/agents/register") => Some("agents:write"),
        ("POST", "/v1/agents/search") => Some("agents:read"),
        ("POST", "/v1/content/register") => Some("content:write"),
        ("POST", "/v1/tasks/create") => Some("tasks:write"),
        ("POST", "/v1/escrow/fund") => Some("escrow:write"),
        ("POST", "/v1/bridge/submit") => Some("bridge:write"),
        _ => post_dynamic_scope(method, path),
    }
}

fn post_dynamic_scope(method: &str, path: &str) -> Option<&'static str> {
    if method != "POST" {
        return None;
    }
    if path.starts_with("/v1/content/") && path.ends_with("/expire") {
        return Some("content:write");
    }
    if path.starts_with("/v1/content/") && path.ends_with("/tombstone") {
        return Some("content:write");
    }
    if path.starts_with("/v1/tasks/") && path.ends_with("/accept") {
        return Some("tasks:write");
    }
    if path.starts_with("/v1/tasks/") && path.ends_with("/complete") {
        return Some("tasks:write");
    }
    if path.starts_with("/v1/escrow/") && path.ends_with("/release") {
        return Some("escrow:write");
    }
    (path.starts_with("/v1/bridge/") && path.ends_with("/forward")).then_some("bridge:write")
}

fn get_scope(method: &str, path: &str) -> Option<&'static str> {
    if method != "GET" {
        return None;
    }
    if path == "/v1/events/ws" {
        return Some("events:read");
    }
    if path.starts_with("/v1/messages/") {
        return Some("messages:read");
    }
    if path.starts_with("/v1/channels/") && path.ends_with("/messages") {
        return Some("channels:read");
    }
    if path.starts_with("/v1/content/") {
        return Some("content:read");
    }
    if path.starts_with("/v1/tasks/") && path != "/v1/tasks/create" {
        return Some("tasks:read");
    }
    if path.starts_with("/v1/agents/") {
        return Some("agents:read");
    }
    (path.starts_with("/v1/bridge/") && path != "/v1/bridge/submit").then_some("bridge:read")
}
