use super::super::*;

#[test]
fn unit_service_api_scope_policy_fixture_parser_contract() {
    let (metadata, rows) = parse_service_api_scope_policy_fixture(SERVICE_API_SCOPE_POLICY_FIXTURE);
    assert_eq!(
        metadata
            .get("scope_policy_fixture_matrix_schema_version")
            .map(String::as_str),
        Some(SERVICE_API_SCOPE_POLICY_FIXTURE_SCHEMA_VERSION)
    );
    assert_eq!(
        metadata
            .get("scope_policy_reason_taxonomy_version")
            .map(String::as_str),
        Some(SERVICE_API_SCOPE_POLICY_REASON_TAXONOMY_VERSION)
    );
    assert_eq!(
        metadata
            .get("scope_policy_reason_codes_csv")
            .map(String::as_str),
        Some(SERVICE_API_SCOPE_POLICY_REASON_CODES_CSV)
    );
    assert!(
        rows.len() >= 6,
        "scope policy fixture matrix should provide representative coverage"
    );
    assert!(rows.iter().any(|row| {
        row.method == "POST"
            && row.path == "/v1/messages/send"
            && row.scope == "messages:write"
            && row.expected == "allow"
    }));
    assert!(rows.iter().any(|row| {
        row.method == "POST"
            && row.path == "/v1/messages/send"
            && row.scope == "messages:read"
            && row.expected == "deny"
    }));
}

#[test]
fn functional_service_api_scope_policy_fixture_rows_match_route_scope_mapping() {
    let (_, rows) = parse_service_api_scope_policy_fixture(SERVICE_API_SCOPE_POLICY_FIXTURE);
    for row in rows {
        let expected_scope = required_scope_for_test_route(row.method.as_str(), row.path.as_str())
            .expect("fixture rows should target protected routes only");
        if row.expected == "allow" {
            assert_eq!(
                row.scope, expected_scope,
                "allow fixture row scope must match required route scope"
            );
        } else if row.expected == "deny" {
            assert_ne!(
                row.scope, expected_scope,
                "deny fixture row scope must not match required route scope"
            );
        } else {
            panic!("scope fixture expected field must be allow|deny");
        }
    }
}
