use super::super::support::*;
use super::super::*;

fn assert_readiness_reason(snapshot: &RuntimeObservabilitySnapshot, expected_reason: &str) {
    let readiness = render_observability_endpoint_response(snapshot, "/readyz");
    let marker = format!("\"readiness_reason_code\":\"{expected_reason}\"");
    assert!(readiness.body.contains(marker.as_str()));
}

fn assert_projection_parity(snapshot: &RuntimeObservabilitySnapshot, expected_reason: &str) {
    let metrics = render_observability_endpoint_response(snapshot, "/metrics");
    let health = render_observability_endpoint_response(snapshot, "/healthz");
    let readiness = render_observability_endpoint_response(snapshot, "/readyz");
    let stream = render_observability_endpoint_response(snapshot, "/metrics.stream");
    let metrics_marker = format!(
        "kamn_observability_readiness_reason_code{{readiness_reason_code=\"{}\"}} 1",
        expected_reason
    );
    let json_marker = format!("\"readiness_reason_code\":\"{expected_reason}\"");
    assert!(metrics.body.contains(metrics_marker.as_str()));
    assert!(health.body.contains(json_marker.as_str()));
    assert!(readiness.body.contains(json_marker.as_str()));
    assert!(stream.body.contains(json_marker.as_str()));
}

#[test]
fn functional_observability_endpoint_readiness_reports_degraded_timeout_reason_codes() {
    let snapshot = daemon_timeout_observability_snapshot();
    let readiness = render_observability_endpoint_response(&snapshot, "/readyz");

    assert_eq!(readiness.status_code, 200);
    assert!(readiness.body.contains("\"ready\":false"));
    assert!(readiness.body.contains("\"health\":\"critical\""));
    assert!(readiness
        .body
        .contains("\"reason_code\":\"daemon_shutdown_timeout\""));
    assert!(readiness
        .body
        .contains("\"readiness_reason_code\":\"readiness_commit_dependency_unhealthy\""));
}

#[test]
fn functional_observability_endpoint_readiness_reason_taxonomy_covers_dependency_probe_matrix() {
    let mut transport_degraded = sample_observability_snapshot();
    transport_degraded.health = "critical".to_owned();
    transport_degraded.reason_code = "transport_finality_retry_unavailable".to_owned();
    transport_degraded.transport_checkpoint_failures = 2;
    assert_readiness_reason(
        &transport_degraded,
        "readiness_transport_dependency_unhealthy",
    );

    let mut signer_degraded = sample_observability_snapshot();
    signer_degraded.health = "critical".to_owned();
    signer_degraded.reason_code = "signer_rotation_stale".to_owned();
    signer_degraded.signer_checkpoint_failures = 1;
    assert_readiness_reason(&signer_degraded, "readiness_signer_dependency_unhealthy");

    let mut commit_degraded = sample_observability_snapshot();
    commit_degraded.health = "critical".to_owned();
    commit_degraded.reason_code = "daemon_shutdown_timeout".to_owned();
    commit_degraded.commit_checkpoint_failures = 1;
    assert_readiness_reason(&commit_degraded, "readiness_commit_dependency_unhealthy");

    let mut runtime_health_degraded = sample_observability_snapshot();
    runtime_health_degraded.health = "degraded".to_owned();
    runtime_health_degraded.reason_code = "daemon_slo_alert".to_owned();
    assert_readiness_reason(
        &runtime_health_degraded,
        "readiness_runtime_health_degraded",
    );
}

#[test]
fn functional_observability_endpoint_projects_readiness_reason_code_parity_across_endpoint_surfaces(
) {
    assert_projection_parity(&sample_observability_snapshot(), "none");
    let mut transport_degraded = sample_observability_snapshot();
    transport_degraded.health = "critical".to_owned();
    transport_degraded.reason_code = "transport_finality_retry_unavailable".to_owned();
    transport_degraded.transport_checkpoint_failures = 1;
    assert_projection_parity(
        &transport_degraded,
        "readiness_transport_dependency_unhealthy",
    );
}
