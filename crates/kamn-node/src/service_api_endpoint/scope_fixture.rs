use super::{
    SERVICE_API_SCOPE_POLICY_REASON_CODES_CSV, SERVICE_API_SCOPE_POLICY_REASON_TAXONOMY_VERSION,
};
use std::collections::BTreeSet;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct ServiceApiScopePolicyFixtureProjection {
    pub(super) reason_taxonomy_version: String,
    pub(super) reason_code_count: usize,
    pub(super) row_count: usize,
    pub(super) allow_row_count: usize,
    pub(super) deny_row_count: usize,
    pub(super) unique_route_count: usize,
    pub(super) unique_scope_count: usize,
    pub(super) unique_method_count: usize,
    pub(super) unique_expected_outcome_count: usize,
    pub(super) unique_allow_route_count: usize,
    pub(super) unique_deny_route_count: usize,
    pub(super) unique_allow_deny_overlap_route_count: usize,
    pub(super) unique_allow_scope_count: usize,
    pub(super) unique_deny_scope_count: usize,
}

pub(super) fn parse_service_api_scope_policy_fixture_projection(
    fixture: &str,
) -> ServiceApiScopePolicyFixtureProjection {
    let mut projection = ServiceApiScopePolicyFixtureProjection {
        reason_taxonomy_version: String::new(),
        reason_code_count: 0,
        row_count: 0,
        allow_row_count: 0,
        deny_row_count: 0,
        unique_route_count: 0,
        unique_scope_count: 0,
        unique_method_count: 0,
        unique_expected_outcome_count: 0,
        unique_allow_route_count: 0,
        unique_deny_route_count: 0,
        unique_allow_deny_overlap_route_count: 0,
        unique_allow_scope_count: 0,
        unique_deny_scope_count: 0,
    };
    let mut reason_codes_csv = String::new();
    let mut unique_routes = BTreeSet::new();
    let mut unique_scopes = BTreeSet::new();
    let mut unique_methods = BTreeSet::new();
    let mut unique_expected_outcomes = BTreeSet::new();
    let mut unique_allow_routes = BTreeSet::new();
    let mut unique_deny_routes = BTreeSet::new();
    let mut unique_allow_scopes = BTreeSet::new();
    let mut unique_deny_scopes = BTreeSet::new();
    for line in fixture.lines().map(str::trim) {
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some((key, value)) = line.split_once('=') {
            let key = key.trim();
            let value = value.trim();
            if key == "scope_policy_reason_taxonomy_version" {
                projection.reason_taxonomy_version = value.to_owned();
            } else if key == "scope_policy_reason_codes_csv" {
                reason_codes_csv = value.to_owned();
            }
            continue;
        }
        let Some(payload) = line.strip_prefix("row|") else {
            continue;
        };
        let mut parts = payload.split('|');
        let method = parts.next().unwrap_or_default().trim();
        let path = parts.next().unwrap_or_default().trim();
        let scope = parts.next().unwrap_or_default().trim();
        let expected = parts.next().unwrap_or_default().trim();
        if method.is_empty() || path.is_empty() || scope.is_empty() || expected.is_empty() {
            continue;
        }
        projection.row_count += 1;
        if expected == "allow" {
            projection.allow_row_count += 1;
            unique_allow_routes.insert((method.to_owned(), path.to_owned()));
            unique_allow_scopes.insert(scope.to_owned());
        } else if expected == "deny" {
            projection.deny_row_count += 1;
            unique_deny_routes.insert((method.to_owned(), path.to_owned()));
            unique_deny_scopes.insert(scope.to_owned());
        }
        unique_routes.insert((method.to_owned(), path.to_owned()));
        unique_scopes.insert(scope.to_owned());
        unique_methods.insert(method.to_owned());
        unique_expected_outcomes.insert(expected.to_owned());
    }
    projection.reason_code_count = reason_codes_csv
        .split(',')
        .filter(|value| !value.trim().is_empty())
        .count();
    if projection.reason_taxonomy_version.is_empty() {
        projection.reason_taxonomy_version =
            SERVICE_API_SCOPE_POLICY_REASON_TAXONOMY_VERSION.to_owned();
    }
    if projection.reason_code_count == 0 {
        projection.reason_code_count = SERVICE_API_SCOPE_POLICY_REASON_CODES_CSV
            .split(',')
            .filter(|value| !value.is_empty())
            .count();
    }
    projection.unique_route_count = unique_routes.len();
    projection.unique_scope_count = unique_scopes.len();
    projection.unique_method_count = unique_methods.len();
    projection.unique_expected_outcome_count = unique_expected_outcomes.len();
    projection.unique_allow_route_count = unique_allow_routes.len();
    projection.unique_deny_route_count = unique_deny_routes.len();
    projection.unique_allow_deny_overlap_route_count = unique_allow_routes
        .intersection(&unique_deny_routes)
        .count();
    projection.unique_allow_scope_count = unique_allow_scopes.len();
    projection.unique_deny_scope_count = unique_deny_scopes.len();
    projection
}
