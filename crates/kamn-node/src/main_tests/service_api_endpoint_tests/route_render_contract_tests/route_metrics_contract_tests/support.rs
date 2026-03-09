use super::super::super::*;
use crate::service_api_endpoint::{
    SERVICE_API_LIFECYCLE_REJECTION_REASON_TAXONOMY_VERSION,
    SERVICE_API_WEBSOCKET_REASON_TAXONOMY_VERSION,
};
use std::collections::BTreeSet;

pub(super) fn assert_common_route_metrics(metrics: &str) {
    assert_taxonomy_counts(metrics);
    assert_scope_fixture_metrics(metrics);
}

fn assert_taxonomy_counts(metrics: &str) {
    let reason_count = cross_store_replay_reason_codes_csv()
        .split(',')
        .filter(|v| !v.is_empty())
        .count();
    let auth_count = SERVICE_API_AUTH_REASON_CODES_CSV
        .split(',')
        .filter(|v| !v.is_empty())
        .count();
    let scope_count = SERVICE_API_SCOPE_POLICY_REASON_CODES_CSV
        .split(',')
        .filter(|v| !v.is_empty())
        .count();
    let websocket_count = SERVICE_API_WEBSOCKET_REASON_CODES_CSV
        .split(',')
        .filter(|v| !v.is_empty())
        .count();
    let lifecycle_count = SERVICE_API_LIFECYCLE_REJECTION_REASON_CODES_CSV
        .split(',')
        .filter(|v| !v.is_empty())
        .count();
    assert_markers(
        metrics,
        vec![
            format!(
                "kamn_service_api_cross_store_replay_reason_taxonomy_info{{version=\"{}\"}} 1",
                cross_store_replay_reason_taxonomy_version()
            ),
            format!("kamn_service_api_cross_store_replay_reason_code_count {reason_count}"),
            format!(
                "kamn_service_api_auth_reason_taxonomy_info{{version=\"{}\"}} 1",
                SERVICE_API_AUTH_REASON_TAXONOMY_VERSION
            ),
            format!("kamn_service_api_auth_reason_code_count {auth_count}"),
            format!(
                "kamn_service_api_scope_policy_reason_taxonomy_info{{version=\"{}\"}} 1",
                SERVICE_API_SCOPE_POLICY_REASON_TAXONOMY_VERSION
            ),
            format!("kamn_service_api_scope_policy_reason_code_count {scope_count}"),
            format!(
                "kamn_service_api_route_authz_matrix_schema_info{{version=\"{}\"}} 1",
                SERVICE_API_ROUTE_AUTHZ_MATRIX_SCHEMA_VERSION
            ),
            format!(
                "kamn_service_api_route_authz_matrix_total_route_count {}",
                SERVICE_API_ROUTE_AUTHZ_MATRIX_TOTAL_ROUTE_COUNT
            ),
            format!(
                "kamn_service_api_route_authz_matrix_public_route_count {}",
                SERVICE_API_ROUTE_AUTHZ_MATRIX_PUBLIC_ROUTE_COUNT
            ),
            format!(
                "kamn_service_api_route_authz_matrix_protected_route_count {}",
                SERVICE_API_ROUTE_AUTHZ_MATRIX_PROTECTED_ROUTE_COUNT
            ),
            format!(
                "kamn_service_api_websocket_reason_taxonomy_info{{version=\"{}\"}} 1",
                SERVICE_API_WEBSOCKET_REASON_TAXONOMY_VERSION
            ),
            format!("kamn_service_api_websocket_reason_code_count {websocket_count}"),
            format!(
                "kamn_service_api_lifecycle_rejection_reason_taxonomy_info{{version=\"{}\"}} 1",
                SERVICE_API_LIFECYCLE_REJECTION_REASON_TAXONOMY_VERSION
            ),
            format!("kamn_service_api_lifecycle_rejection_reason_code_count {lifecycle_count}"),
        ],
    );
}

fn assert_scope_fixture_metrics(metrics: &str) {
    let (metadata, rows) = parse_service_api_scope_policy_fixture(SERVICE_API_SCOPE_POLICY_FIXTURE);
    let reason_version = metadata
        .get("scope_policy_reason_taxonomy_version")
        .map(String::as_str)
        .unwrap_or_default();
    let reason_count = metadata
        .get("scope_policy_reason_codes_csv")
        .map(|v| v.split(',').filter(|e| !e.trim().is_empty()).count())
        .unwrap_or_default();
    let allow = rows.iter().filter(|row| row.expected == "allow").count();
    let deny = rows.iter().filter(|row| row.expected == "deny").count();
    assert_markers(
        metrics,
        vec![
            format!(
                "kamn_service_api_scope_policy_fixture_schema_info{{version=\"{}\"}} 1",
                SERVICE_API_SCOPE_POLICY_FIXTURE_SCHEMA_VERSION
            ),
            format!(
                "kamn_service_api_scope_policy_fixture_reason_taxonomy_info{{version=\"{}\"}} 1",
                reason_version
            ),
            format!("kamn_service_api_scope_policy_fixture_reason_code_count {reason_count}"),
            format!(
                "kamn_service_api_scope_policy_fixture_row_count {}",
                rows.len()
            ),
            format!("kamn_service_api_scope_policy_fixture_allow_row_count {allow}"),
            format!("kamn_service_api_scope_policy_fixture_deny_row_count {deny}"),
        ],
    );
    assert_scope_fixture_uniques(metrics, &rows);
}

fn assert_scope_fixture_uniques(metrics: &str, rows: &[ServiceApiScopePolicyFixtureRow]) {
    let unique_route_count = rows
        .iter()
        .map(|row| (row.method.as_str(), row.path.as_str()))
        .collect::<BTreeSet<_>>()
        .len();
    let unique_scope_count = rows
        .iter()
        .map(|row| row.scope.as_str())
        .collect::<BTreeSet<_>>()
        .len();
    let unique_method_count = rows
        .iter()
        .map(|row| row.method.as_str())
        .collect::<BTreeSet<_>>()
        .len();
    let unique_expected_outcome_count = rows
        .iter()
        .map(|row| row.expected.as_str())
        .collect::<BTreeSet<_>>()
        .len();
    let allow_scopes = rows
        .iter()
        .filter(|row| row.expected == "allow")
        .map(|row| row.scope.as_str())
        .collect::<BTreeSet<_>>();
    let deny_scopes = rows
        .iter()
        .filter(|row| row.expected == "deny")
        .map(|row| row.scope.as_str())
        .collect::<BTreeSet<_>>();
    let allow_methods = rows
        .iter()
        .filter(|row| row.expected == "allow")
        .map(|row| row.method.as_str())
        .collect::<BTreeSet<_>>();
    let deny_methods = rows
        .iter()
        .filter(|row| row.expected == "deny")
        .map(|row| row.method.as_str())
        .collect::<BTreeSet<_>>();
    let allow_routes = rows
        .iter()
        .filter(|row| row.expected == "allow")
        .map(|row| (row.method.as_str(), row.path.as_str()))
        .collect::<BTreeSet<_>>();
    let deny_routes = rows
        .iter()
        .filter(|row| row.expected == "deny")
        .map(|row| (row.method.as_str(), row.path.as_str()))
        .collect::<BTreeSet<_>>();
    assert_markers(metrics, vec![
        format!("kamn_service_api_scope_policy_fixture_unique_route_count {unique_route_count}"),
        format!("kamn_service_api_scope_policy_fixture_unique_scope_count {unique_scope_count}"),
        format!("kamn_service_api_scope_policy_fixture_unique_method_count {unique_method_count}"),
        format!("kamn_service_api_scope_policy_fixture_unique_expected_outcome_count {unique_expected_outcome_count}"),
        format!("kamn_service_api_scope_policy_fixture_unique_allow_scope_count {}", allow_scopes.len()),
        format!("kamn_service_api_scope_policy_fixture_unique_deny_scope_count {}", deny_scopes.len()),
        format!("kamn_service_api_scope_policy_fixture_unique_allow_deny_overlap_scope_count {}", allow_scopes.intersection(&deny_scopes).count()),
        format!("kamn_service_api_scope_policy_fixture_unique_allow_only_scope_count {}", allow_scopes.difference(&deny_scopes).count()),
        format!("kamn_service_api_scope_policy_fixture_unique_deny_only_scope_count {}", deny_scopes.difference(&allow_scopes).count()),
        format!("kamn_service_api_scope_policy_fixture_unique_allow_deny_overlap_method_count {}", allow_methods.intersection(&deny_methods).count()),
        format!("kamn_service_api_scope_policy_fixture_unique_allow_only_method_count {}", allow_methods.difference(&deny_methods).count()),
        format!("kamn_service_api_scope_policy_fixture_unique_deny_only_method_count {}", deny_methods.difference(&allow_methods).count()),
        format!("kamn_service_api_scope_policy_fixture_unique_allow_route_count {}", allow_routes.len()),
        format!("kamn_service_api_scope_policy_fixture_unique_deny_route_count {}", deny_routes.len()),
        format!("kamn_service_api_scope_policy_fixture_unique_allow_deny_overlap_route_count {}", allow_routes.intersection(&deny_routes).count()),
        format!("kamn_service_api_scope_policy_fixture_unique_allow_only_route_count {}", allow_routes.difference(&deny_routes).count()),
        format!("kamn_service_api_scope_policy_fixture_unique_deny_only_route_count {}", deny_routes.difference(&allow_routes).count()),
    ]);
}

fn assert_markers(metrics: &str, markers: Vec<String>) {
    for marker in markers {
        assert!(
            metrics.contains(&marker),
            "metrics payload missing marker: {marker}"
        );
    }
}
