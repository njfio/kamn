use super::support::*;
use super::*;

fn assert_surface_contract(surface: ObservabilityEndpointPayloadSurface, body: &str) {
    assert!(validate_observability_endpoint_payload_contract(surface, body).is_ok());
}

#[test]
fn spec_c01_observability_endpoint_contract_checker_accepts_valid_surface_payloads() {
    let snapshot = sample_observability_snapshot();
    assert_surface_contract(
        ObservabilityEndpointPayloadSurface::Metrics,
        render_observability_endpoint_response(&snapshot, "/metrics")
            .body
            .as_str(),
    );
    assert_surface_contract(
        ObservabilityEndpointPayloadSurface::Health,
        render_observability_endpoint_response(&snapshot, "/healthz")
            .body
            .as_str(),
    );
    assert_surface_contract(
        ObservabilityEndpointPayloadSurface::Readiness,
        render_observability_endpoint_response(&snapshot, "/readyz")
            .body
            .as_str(),
    );
    assert_surface_contract(
        ObservabilityEndpointPayloadSurface::Stream,
        render_observability_endpoint_response(&snapshot, "/metrics.stream")
            .body
            .as_str(),
    );
}

#[test]
fn spec_c02_observability_endpoint_contract_checker_rejects_missing_health_reason_code_field() {
    let snapshot = sample_observability_snapshot();
    let health = render_observability_endpoint_response(&snapshot, "/healthz");
    let tampered = health.body.replace("\"reason_code\":\"none\",", "");

    let error = validate_observability_endpoint_payload_contract(
        ObservabilityEndpointPayloadSurface::Health,
        tampered.as_str(),
    )
    .expect_err("missing health reason_code must fail contract check");
    assert_eq!(
        error,
        "runtime_observability_policy_required_field_missing:health.reason_code"
    );
}

#[test]
fn spec_c03_observability_endpoint_contract_checker_rejects_metrics_readiness_metric_drift() {
    let snapshot = sample_observability_snapshot();
    let metrics = render_observability_endpoint_response(&snapshot, "/metrics");
    let tampered = metrics.body.replace(
        "kamn_observability_readiness_reason_code",
        "kamn_observability_readiness_reason_label",
    );

    let error = validate_observability_endpoint_payload_contract(
        ObservabilityEndpointPayloadSurface::Metrics,
        tampered.as_str(),
    )
    .expect_err("missing readiness reason metric line must fail contract check");
    assert_eq!(error, "runtime_observability_policy_required_field_missing:metrics.kamn_observability_readiness_reason_code");
}

#[test]
fn spec_c04_observability_endpoint_contract_checker_rejects_stream_schema_version_drift() {
    let snapshot = sample_observability_snapshot();
    let stream = render_observability_endpoint_response(&snapshot, "/metrics.stream");
    let tampered = stream.body.replace(
        "\"schema_version\":\"kamn.runtime.observability.stream.v1\"",
        "\"schema_version\":\"kamn.runtime.observability.stream.v2\"",
    );

    let error = validate_observability_endpoint_payload_contract(
        ObservabilityEndpointPayloadSurface::Stream,
        tampered.as_str(),
    )
    .expect_err("stream schema drift must fail contract check");
    assert_eq!(
        error,
        "runtime_observability_policy_schema_drift:stream.schema_version"
    );
}

#[test]
fn spec_c05_observability_endpoint_contract_checker_fails_closed_with_stable_reason_markers() {
    let fail_closed = enforce_observability_endpoint_payload_contract(
        ObservabilityEndpointPayloadSurface::Health,
        "application/json",
        "{\"schema_version\":\"kamn.runtime.observability.health.v1\"}".to_owned(),
    );

    assert_eq!(fail_closed.status_code, 503);
    assert_eq!(fail_closed.content_type, "application/json");
    assert!(fail_closed
        .body
        .contains("\"schema_version\":\"kamn.runtime.observability.endpoint-fail-closed.v1\""));
    assert!(fail_closed.body.contains("\"status\":\"fail_closed\""));
    assert!(fail_closed.body.contains("\"final_decision\":\"NO-GO\""));
    assert!(fail_closed.body.contains(
        "\"reason_taxonomy_version\":\"kamn.runtime.observability-endpoint-reason-taxonomy.v1\""
    ));
    assert!(fail_closed.body.contains(
        "\"reason_code\":\"runtime_observability_policy_required_field_missing:health.source\""
    ));
}
