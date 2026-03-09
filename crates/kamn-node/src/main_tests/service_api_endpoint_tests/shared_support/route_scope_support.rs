use super::super::*;
use super::auth_fixture_support::{
    signed_header_present, test_service_api_auth_public_key_hex, test_service_api_sender_did,
};

pub(crate) const SERVICE_API_SCOPE_POLICY_FIXTURE: &str = include_str!(
    "../../../../../../fixtures/runtime/service_api_scope_policy_fixture_matrix.txt"
);

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ServiceApiRouteAuthzMatrixRow {
    pub(crate) method: &'static str,
    pub(crate) path: &'static str,
    pub(crate) body: &'static str,
    pub(crate) requires_auth: bool,
    pub(crate) expected_status_without_auth: &'static str,
}

const SERVICE_API_ROUTE_AUTHZ_MATRIX_ROWS: &[ServiceApiRouteAuthzMatrixRow] = &[
    ServiceApiRouteAuthzMatrixRow { method: "GET", path: "/healthz", body: "", requires_auth: false, expected_status_without_auth: "HTTP/1.1 200 OK" },
    ServiceApiRouteAuthzMatrixRow { method: "GET", path: "/metrics", body: "", requires_auth: false, expected_status_without_auth: "HTTP/1.1 200 OK" },
    ServiceApiRouteAuthzMatrixRow { method: "POST", path: "/v1/messages/send", body: "{\"message\":\"matrix-message\"}", requires_auth: true, expected_status_without_auth: "HTTP/1.1 401 Unauthorized" },
    ServiceApiRouteAuthzMatrixRow { method: "POST", path: "/v1/channels/create", body: "{\"name\":\"matrix-channel\"}", requires_auth: true, expected_status_without_auth: "HTTP/1.1 401 Unauthorized" },
    ServiceApiRouteAuthzMatrixRow { method: "POST", path: "/v1/tasks/create", body: "{\"task\":\"matrix-task\"}", requires_auth: true, expected_status_without_auth: "HTTP/1.1 401 Unauthorized" },
    ServiceApiRouteAuthzMatrixRow { method: "POST", path: "/v1/agents/search", body: "{\"capability\":\"code\"}", requires_auth: true, expected_status_without_auth: "HTTP/1.1 401 Unauthorized" },
    ServiceApiRouteAuthzMatrixRow { method: "POST", path: "/v1/tasks/task-matrix/accept", body: "{}", requires_auth: true, expected_status_without_auth: "HTTP/1.1 401 Unauthorized" },
    ServiceApiRouteAuthzMatrixRow { method: "POST", path: "/v1/tasks/task-matrix/complete", body: "{}", requires_auth: true, expected_status_without_auth: "HTTP/1.1 401 Unauthorized" },
    ServiceApiRouteAuthzMatrixRow { method: "POST", path: "/v1/escrow/fund", body: "{\"task_id\":\"task-matrix\",\"amount\":100}", requires_auth: true, expected_status_without_auth: "HTTP/1.1 401 Unauthorized" },
    ServiceApiRouteAuthzMatrixRow { method: "POST", path: "/v1/escrow/escrow-matrix/release", body: "{}", requires_auth: true, expected_status_without_auth: "HTTP/1.1 401 Unauthorized" },
    ServiceApiRouteAuthzMatrixRow { method: "POST", path: "/v1/content/register", body: "{\"content\":\"matrix-content\",\"retention_class\":\"standard\"}", requires_auth: true, expected_status_without_auth: "HTTP/1.1 401 Unauthorized" },
    ServiceApiRouteAuthzMatrixRow { method: "POST", path: "/v1/content/content-matrix/expire", body: "{}", requires_auth: true, expected_status_without_auth: "HTTP/1.1 401 Unauthorized" },
    ServiceApiRouteAuthzMatrixRow { method: "POST", path: "/v1/content/content-matrix/tombstone", body: "{}", requires_auth: true, expected_status_without_auth: "HTTP/1.1 401 Unauthorized" },
    ServiceApiRouteAuthzMatrixRow { method: "GET", path: "/v1/content/content-matrix", body: "", requires_auth: true, expected_status_without_auth: "HTTP/1.1 401 Unauthorized" },
    ServiceApiRouteAuthzMatrixRow { method: "POST", path: "/v1/bridge/submit", body: "{\"source_message_id\":\"msg-matrix\",\"target_network\":\"testnet\"}", requires_auth: true, expected_status_without_auth: "HTTP/1.1 401 Unauthorized" },
    ServiceApiRouteAuthzMatrixRow { method: "POST", path: "/v1/bridge/bridge-matrix/forward", body: "{}", requires_auth: true, expected_status_without_auth: "HTTP/1.1 401 Unauthorized" },
    ServiceApiRouteAuthzMatrixRow { method: "GET", path: "/v1/bridge/bridge-matrix", body: "", requires_auth: true, expected_status_without_auth: "HTTP/1.1 401 Unauthorized" },
    ServiceApiRouteAuthzMatrixRow { method: "GET", path: "/v1/messages/msg-matrix", body: "", requires_auth: true, expected_status_without_auth: "HTTP/1.1 401 Unauthorized" },
    ServiceApiRouteAuthzMatrixRow { method: "GET", path: "/v1/channels/channel-matrix/messages", body: "", requires_auth: true, expected_status_without_auth: "HTTP/1.1 401 Unauthorized" },
    ServiceApiRouteAuthzMatrixRow { method: "GET", path: "/v1/tasks/task-matrix", body: "", requires_auth: true, expected_status_without_auth: "HTTP/1.1 401 Unauthorized" },
    ServiceApiRouteAuthzMatrixRow { method: "POST", path: "/v1/agents/register", body: "{\"agent_type\":\"assistant\",\"model_family\":\"gpt-5\",\"capabilities\":[\"text\"]}", requires_auth: true, expected_status_without_auth: "HTTP/1.1 401 Unauthorized" },
    ServiceApiRouteAuthzMatrixRow { method: "GET", path: "/v1/agents/kamn:did:agent:matrix", body: "", requires_auth: true, expected_status_without_auth: "HTTP/1.1 401 Unauthorized" },
    ServiceApiRouteAuthzMatrixRow { method: "GET", path: "/v1/agents/kamn:did:agent:matrix/balance", body: "", requires_auth: true, expected_status_without_auth: "HTTP/1.1 401 Unauthorized" },
    ServiceApiRouteAuthzMatrixRow { method: "GET", path: "/v1/events/ws", body: "", requires_auth: true, expected_status_without_auth: "HTTP/1.1 401 Unauthorized" },
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ServiceApiScopePolicyFixtureRow {
    pub(crate) method: String,
    pub(crate) path: String,
    pub(crate) scope: String,
    pub(crate) expected: String,
}

pub(crate) fn service_api_route_authz_matrix_rows() -> Vec<ServiceApiRouteAuthzMatrixRow> {
    SERVICE_API_ROUTE_AUTHZ_MATRIX_ROWS.to_vec()
}

pub(crate) fn parse_service_api_scope_policy_fixture(
    fixture: &str,
) -> (BTreeMap<String, String>, Vec<ServiceApiScopePolicyFixtureRow>) {
    let mut metadata = BTreeMap::new();
    let mut rows = Vec::new();
    for line in fixture.lines().map(str::trim) {
        parse_scope_policy_fixture_line(line, &mut metadata, &mut rows);
    }
    (metadata, rows)
}

fn parse_scope_policy_fixture_line(
    line: &str,
    metadata: &mut BTreeMap<String, String>,
    rows: &mut Vec<ServiceApiScopePolicyFixtureRow>,
) {
    if line.is_empty() || line.starts_with('#') {
        return;
    }
    if let Some((key, value)) = line.split_once('=') {
        metadata.insert(key.trim().to_owned(), value.trim().to_owned());
        return;
    }
    if let Some(row) = line.strip_prefix("row|").and_then(parse_scope_policy_row) {
        rows.push(row);
    }
}

fn parse_scope_policy_row(payload: &str) -> Option<ServiceApiScopePolicyFixtureRow> {
    let mut parts = payload.split('|');
    let method = parts.next()?.trim().to_owned();
    let path = parts.next()?.trim().to_owned();
    let scope = parts.next()?.trim().to_owned();
    let expected = parts.next()?.trim().to_owned();
    (!method.is_empty() && !path.is_empty() && !scope.is_empty() && !expected.is_empty()).then_some(
        ServiceApiScopePolicyFixtureRow {
            method,
            path,
            scope,
            expected,
        },
    )
}

pub(crate) fn required_scope_for_test_route(method: &str, path: &str) -> Option<&'static str> {
    crate::service_api_endpoint::route_requires_auth(method, path)
        .then(|| protected_test_scope(method, path))
}

fn protected_test_scope(method: &str, path: &str) -> &'static str {
    match (method, path) {
        ("POST", "/v1/messages/send") | ("POST", "/v1/messages/relay") => "messages:write",
        ("POST", "/v1/channels/create") => "channels:write",
        ("POST", "/v1/agents/search") => "agents:read",
        ("POST", "/v1/agents/register") => "agents:write",
        ("POST", "/v1/tasks/create") | ("POST", _) if path.starts_with("/v1/tasks/") && ["/accept", "/complete"].iter().any(|suffix| path.ends_with(suffix)) => "tasks:write",
        ("POST", "/v1/escrow/fund") | ("POST", _) if path.starts_with("/v1/escrow/") && path.ends_with("/release") => "escrow:write",
        ("POST", "/v1/content/register") | ("POST", _) if path.starts_with("/v1/content/") && ["/expire", "/tombstone"].iter().any(|suffix| path.ends_with(suffix)) => "content:write",
        ("POST", "/v1/bridge/submit") | ("POST", _) if path.starts_with("/v1/bridge/") && path.ends_with("/forward") => "bridge:write",
        ("GET", "/v1/events/ws") => "events:read",
        ("GET", _) if path.starts_with("/v1/content/") && path != "/v1/content/register" => "content:read",
        ("GET", _) if path.starts_with("/v1/bridge/") && path != "/v1/bridge/submit" => "bridge:read",
        ("GET", _) if path.starts_with("/v1/messages/") && path != "/v1/messages/send" => "messages:read",
        ("GET", _) if path.starts_with("/v1/channels/") && path.ends_with("/messages") => "channels:read",
        ("GET", _) if path.starts_with("/v1/tasks/") && path != "/v1/tasks/create" => "tasks:read",
        ("GET", _) if path.starts_with("/v1/agents/") => "agents:read",
        _ => "protected:unknown",
    }
}

pub(crate) fn enrich_signed_headers_with_scope(
    method: &str,
    path: &str,
    headers: &[(&str, &str)],
) -> Vec<(String, String)> {
    let mut enriched = clone_headers(headers);
    if !request_is_signed(headers) {
        return enriched;
    }
    add_missing_scope(method, path, &mut enriched, headers);
    normalize_sender_did(&mut enriched);
    add_missing_signer_public_key(&mut enriched, headers);
    enriched
}

fn clone_headers(headers: &[(&str, &str)]) -> Vec<(String, String)> {
    headers
        .iter()
        .map(|(name, value)| ((*name).to_owned(), (*value).to_owned()))
        .collect()
}

fn request_is_signed(headers: &[(&str, &str)]) -> bool {
    [
        "X-KAMN-Sender-DID",
        "X-KAMN-Request-Nonce",
        "X-KAMN-Request-Signature",
    ]
    .iter()
    .all(|name| signed_header_present(headers, name))
}

fn add_missing_scope(
    method: &str,
    path: &str,
    enriched: &mut Vec<(String, String)>,
    headers: &[(&str, &str)],
) {
    if signed_header_present(headers, "X-KAMN-Authz-Scope") {
        return;
    }
    if let Some(scope) = required_scope_for_test_route(method, path) {
        enriched.push(("X-KAMN-Authz-Scope".to_owned(), scope.to_owned()));
    }
}

fn normalize_sender_did(enriched: &mut [(String, String)]) {
    for (name, value) in enriched {
        if name.eq_ignore_ascii_case("X-KAMN-Sender-DID") {
            *value = test_service_api_sender_did(value.as_str());
        }
    }
}

fn add_missing_signer_public_key(
    enriched: &mut Vec<(String, String)>,
    headers: &[(&str, &str)],
) {
    if signed_header_present(headers, "X-KAMN-Signer-Public-Key") {
        return;
    }
    enriched.push((
        "X-KAMN-Signer-Public-Key".to_owned(),
        test_service_api_auth_public_key_hex(),
    ));
}
