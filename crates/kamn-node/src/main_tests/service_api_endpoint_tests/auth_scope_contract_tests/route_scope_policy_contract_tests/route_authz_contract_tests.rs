use super::super::*;

#[derive(Debug, Clone, PartialEq, Eq)]
struct ObservedRouteOutcome {
    method: String,
    path: String,
    status_line: String,
    reason_code: Option<String>,
}

#[test]
fn unit_service_api_route_authz_matrix_matches_protected_and_public_paths() {
    assert_eq!(
        SERVICE_API_AUTH_REASON_TAXONOMY_VERSION,
        "kamn.runtime.service-api-auth-reason-taxonomy.v1"
    );
    assert!(SERVICE_API_AUTH_REASON_CODES_CSV.contains(SERVICE_API_AUTH_MISSING_HEADER_REASON_CODE));
    for row in service_api_route_authz_matrix_rows() {
        assert_eq!(
            crate::service_api_endpoint::route_requires_auth(row.method, row.path),
            row.requires_auth,
            "route authz matrix drift for {} {}",
            row.method,
            row.path
        );
    }
}

#[test]
fn integration_service_api_endpoint_route_authz_matrix_rejects_protected_paths_without_headers() {
    let (_snapshot, bind_addr, server) = start_service_api_server("127.0.0.1:34074", 48);
    let matrix_rows = service_api_route_authz_matrix_rows();
    let mut baseline_outcomes: Option<Vec<ObservedRouteOutcome>> = None;

    for _round in 0..2_u64 {
        let outcomes = observe_route_authz_outcomes(bind_addr.as_str(), &matrix_rows);
        if let Some(baseline) = baseline_outcomes.as_ref() {
            assert_eq!(
                outcomes, *baseline,
                "route authz outcomes must remain deterministic across rounds"
            );
        } else {
            baseline_outcomes = Some(outcomes);
        }
    }

    join_service_api_server(
        server,
        "service api endpoint should stop cleanly after route authz matrix validation",
    );
}

fn observe_route_authz_outcomes(
    bind_addr: &str,
    matrix_rows: &[ServiceApiRouteAuthzMatrixRow],
) -> Vec<ObservedRouteOutcome> {
    matrix_rows
        .iter()
        .map(|row| observe_route_authz_outcome(bind_addr, row))
        .collect()
}

fn observe_route_authz_outcome(
    bind_addr: &str,
    row: &ServiceApiRouteAuthzMatrixRow,
) -> ObservedRouteOutcome {
    let response = send_http_request(bind_addr, row.method, row.path, row.body);
    assert!(
        response.contains(row.expected_status_without_auth),
        "unexpected authz matrix status for {} {}: expected {}, response={response}",
        row.method,
        row.path,
        row.expected_status_without_auth
    );
    ObservedRouteOutcome {
        method: row.method.to_owned(),
        path: row.path.to_owned(),
        status_line: row.expected_status_without_auth.to_owned(),
        reason_code: route_authz_reason_code(row, response.as_str()),
    }
}

fn route_authz_reason_code(row: &ServiceApiRouteAuthzMatrixRow, response: &str) -> Option<String> {
    if !row.requires_auth {
        return None;
    }
    let payload = parse_error_envelope_from_http_response(response);
    assert_eq!(payload.error, "unauthorized");
    assert_eq!(
        payload.reason_code,
        SERVICE_API_AUTH_MISSING_HEADER_REASON_CODE
    );
    Some(payload.reason_code)
}
