use super::super::support::*;
use super::super::*;

#[test]
fn functional_observability_endpoint_readiness_reports_degraded_timeout_reason_codes() {
    let parsed = parse_args_with_clean_daemon_env(vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
        "--runtime-mode".to_owned(),
        "daemon".to_owned(),
        "--daemon-max-ticks".to_owned(),
        "100".to_owned(),
        "--daemon-tick-interval-ms".to_owned(),
        "1".to_owned(),
        "--daemon-shutdown-signal-tick".to_owned(),
        "7".to_owned(),
        "--daemon-shutdown-drain-ticks".to_owned(),
        "4".to_owned(),
        "--daemon-shutdown-timeout-ticks".to_owned(),
        "2".to_owned(),
    ])
    .expect("daemon timeout args should parse");
    let report = execute(parsed).expect("daemon timeout execution should succeed");
    let snapshot =
        build_runtime_observability_snapshot(&report).expect("timeout report should map snapshot");
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
    let transport_readiness =
        render_observability_endpoint_response(&transport_degraded, "/readyz");
    assert!(transport_readiness
        .body
        .contains("\"readiness_reason_code\":\"readiness_transport_dependency_unhealthy\""));

    let mut signer_degraded = sample_observability_snapshot();
    signer_degraded.health = "critical".to_owned();
    signer_degraded.reason_code = "signer_rotation_stale".to_owned();
    signer_degraded.signer_checkpoint_failures = 1;
    let signer_readiness = render_observability_endpoint_response(&signer_degraded, "/readyz");
    assert!(signer_readiness
        .body
        .contains("\"readiness_reason_code\":\"readiness_signer_dependency_unhealthy\""));

    let mut commit_degraded = sample_observability_snapshot();
    commit_degraded.health = "critical".to_owned();
    commit_degraded.reason_code = "daemon_shutdown_timeout".to_owned();
    commit_degraded.commit_checkpoint_failures = 1;
    let commit_readiness = render_observability_endpoint_response(&commit_degraded, "/readyz");
    assert!(commit_readiness
        .body
        .contains("\"readiness_reason_code\":\"readiness_commit_dependency_unhealthy\""));

    let mut runtime_health_degraded = sample_observability_snapshot();
    runtime_health_degraded.health = "degraded".to_owned();
    runtime_health_degraded.reason_code = "daemon_slo_alert".to_owned();
    let runtime_health_readiness =
        render_observability_endpoint_response(&runtime_health_degraded, "/readyz");
    assert!(runtime_health_readiness
        .body
        .contains("\"readiness_reason_code\":\"readiness_runtime_health_degraded\""));
}

#[test]
fn functional_observability_endpoint_projects_readiness_reason_code_parity_across_endpoint_surfaces(
) {
    let assert_projection_parity = |snapshot: &RuntimeObservabilitySnapshot,
                                    expected_reason_code: &str| {
        let metrics = render_observability_endpoint_response(snapshot, "/metrics");
        let health = render_observability_endpoint_response(snapshot, "/healthz");
        let readiness = render_observability_endpoint_response(snapshot, "/readyz");
        let stream = render_observability_endpoint_response(snapshot, "/metrics.stream");
        let metrics_marker = format!(
            "kamn_observability_readiness_reason_code{{readiness_reason_code=\"{}\"}} 1",
            expected_reason_code
        );
        let json_marker = format!("\"readiness_reason_code\":\"{}\"", expected_reason_code);
        assert!(metrics.body.contains(metrics_marker.as_str()));
        assert!(health.body.contains(json_marker.as_str()));
        assert!(readiness.body.contains(json_marker.as_str()));
        assert!(stream.body.contains(json_marker.as_str()));
    };

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
