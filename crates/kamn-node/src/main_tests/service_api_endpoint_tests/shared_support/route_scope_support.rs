use super::super::*;
#[path = "route_scope_header_support.rs"]
mod route_scope_header_support;
#[path = "route_scope_matrix_data.rs"]
mod route_scope_matrix_data;
use route_scope_header_support::{
    add_missing_scope, add_missing_signer_public_key, clone_headers, normalize_sender_did,
    request_is_signed,
};
use route_scope_matrix_data::SERVICE_API_ROUTE_AUTHZ_MATRIX_ROWS;

pub(crate) const SERVICE_API_SCOPE_POLICY_FIXTURE: &str =
    include_str!("../../../../../../fixtures/runtime/service_api_scope_policy_fixture_matrix.txt");

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ServiceApiRouteAuthzMatrixRow {
    pub(crate) method: &'static str,
    pub(crate) path: &'static str,
    pub(crate) body: &'static str,
    pub(crate) requires_auth: bool,
    pub(crate) expected_status_without_auth: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ServiceApiScopePolicyFixtureRow {
    pub(crate) method: String,
    pub(crate) path: String,
    pub(crate) scope: String,
    pub(crate) expected: String,
}

pub(crate) fn service_api_route_authz_matrix_rows() -> Vec<ServiceApiRouteAuthzMatrixRow> {
    SERVICE_API_ROUTE_AUTHZ_MATRIX_ROWS
        .iter()
        .map(build_route_authz_row)
        .collect()
}

fn build_route_authz_row(
    row: &'static (&'static str, &'static str, &'static str, bool, &'static str),
) -> ServiceApiRouteAuthzMatrixRow {
    ServiceApiRouteAuthzMatrixRow {
        method: row.0,
        path: row.1,
        body: row.2,
        requires_auth: row.3,
        expected_status_without_auth: row.4,
    }
}

pub(crate) fn parse_service_api_scope_policy_fixture(
    fixture: &str,
) -> (
    BTreeMap<String, String>,
    Vec<ServiceApiScopePolicyFixtureRow>,
) {
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
    match method {
        "POST" => protected_post_scope(path),
        "GET" => protected_get_scope(path),
        _ => "protected:unknown",
    }
}

fn protected_post_scope(path: &str) -> &'static str {
    match path {
        "/v1/messages/send" | "/v1/messages/relay" => "messages:write",
        "/v1/channels/create" => "channels:write",
        "/v1/agents/search" => "agents:read",
        "/v1/agents/register" => "agents:write",
        "/v1/tasks/create" => "tasks:write",
        "/v1/escrow/fund" => "escrow:write",
        "/v1/content/register" => "content:write",
        "/v1/bridge/submit" => "bridge:write",
        _ => protected_post_dynamic_scope(path),
    }
}

fn protected_post_dynamic_scope(path: &str) -> &'static str {
    if path.starts_with("/v1/tasks/") && route_suffix_matches(path, &["/accept", "/complete"]) {
        return "tasks:write";
    }
    if path.starts_with("/v1/escrow/") && path.ends_with("/release") {
        return "escrow:write";
    }
    if path.starts_with("/v1/content/") && route_suffix_matches(path, &["/expire", "/tombstone"]) {
        return "content:write";
    }
    if path.starts_with("/v1/bridge/") && path.ends_with("/forward") {
        return "bridge:write";
    }
    "protected:unknown"
}

fn protected_get_scope(path: &str) -> &'static str {
    if path == "/v1/events/ws" {
        return "events:read";
    }
    if path.starts_with("/v1/content/") && path != "/v1/content/register" {
        return "content:read";
    }
    if path.starts_with("/v1/bridge/") && path != "/v1/bridge/submit" {
        return "bridge:read";
    }
    if path.starts_with("/v1/messages/") && path != "/v1/messages/send" {
        return "messages:read";
    }
    if path.starts_with("/v1/channels/") && path.ends_with("/messages") {
        return "channels:read";
    }
    if path.starts_with("/v1/tasks/") && path != "/v1/tasks/create" {
        return "tasks:read";
    }
    if path.starts_with("/v1/agents/") {
        return "agents:read";
    }
    "protected:unknown"
}

fn route_suffix_matches(path: &str, suffixes: &[&str]) -> bool {
    suffixes.iter().any(|suffix| path.ends_with(suffix))
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
