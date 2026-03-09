use super::super::*;

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
    #[derive(Debug, Clone, PartialEq, Eq)]
    struct ObservedRouteOutcome {
        method: String,
        path: String,
        status_line: String,
        reason_code: Option<String>,
    }

    let (snapshot, bind_addr, server) = start_service_api_server("127.0.0.1:34074", 48);
    let matrix_rows = service_api_route_authz_matrix_rows();
    let mut baseline_outcomes: Option<Vec<ObservedRouteOutcome>> = None;

    for _round in 0..2_u64 {
        let mut outcomes = Vec::with_capacity(matrix_rows.len());
        for row in &matrix_rows {
            let response = send_http_request(bind_addr.as_str(), row.method, row.path, row.body);
            assert!(
                response.contains(row.expected_status_without_auth),
                "unexpected authz matrix status for {} {}: expected {}, response={response}",
                row.method,
                row.path,
                row.expected_status_without_auth
            );
            let reason_code = if row.requires_auth {
                let payload = parse_error_envelope_from_http_response(response.as_str());
                assert_eq!(payload.error, "unauthorized");
                assert_eq!(payload.reason_code, SERVICE_API_AUTH_MISSING_HEADER_REASON_CODE);
                Some(payload.reason_code)
            } else {
                None
            };
            outcomes.push(ObservedRouteOutcome {
                method: row.method.to_owned(),
                path: row.path.to_owned(),
                status_line: row.expected_status_without_auth.to_owned(),
                reason_code,
            });
        }
        if let Some(baseline) = baseline_outcomes.as_ref() {
            assert_eq!(outcomes, *baseline, "route authz outcomes must remain deterministic across rounds");
        } else {
            baseline_outcomes = Some(outcomes);
        }
    }

    let _ = snapshot;
    join_service_api_server(
        server,
        "service api endpoint should stop cleanly after route authz matrix validation",
    );
}
