use super::support::*;
use super::*;

#[path = "endpoint_runtime_contract_tests/readiness_contract_tests.rs"]
mod readiness_contract_tests;

fn assert_metrics_payload(metrics_body: &str) {
    assert!(metrics_body.contains("kamn_observability_latency_p50_ms 1"));
    assert!(metrics_body.contains("kamn_observability_reason_code{reason_code=\"none\"} 1"));
    assert!(metrics_body
        .contains("kamn_observability_readiness_reason_code{readiness_reason_code=\"none\"} 1"));
    assert!(metrics_body.contains("kamn_observability_ready 1"));
    assert!(metrics_body.contains("kamn_observability_health{health=\"healthy\"} 1"));
}

fn assert_health_payload(health_body: &str) {
    assert!(health_body.contains("\"schema_version\":\"kamn.runtime.observability.health.v1\""));
    assert!(health_body.contains("\"health\":\"healthy\""));
    assert!(health_body.contains("\"runtime_mode\":\"daemon\""));
    assert!(health_body.contains("\"reason_code\":\"none\""));
    assert!(health_body.contains("\"ready\":true"));
    assert!(health_body.contains("\"readiness_reason_code\":\"none\""));
}

#[test]
fn functional_observability_endpoint_renders_metrics_and_health_payloads() {
    let snapshot = daemon_observability_snapshot();
    let metrics = render_observability_endpoint_response(&snapshot, "/metrics");
    let health = render_observability_endpoint_response(&snapshot, "/healthz");
    assert_eq!(metrics.status_code, 200);
    assert_eq!(metrics.content_type, "text/plain; version=0.0.4");
    assert_eq!(health.status_code, 200);
    assert_eq!(health.content_type, "application/json");
    assert_metrics_payload(metrics.body.as_str());
    assert_health_payload(health.body.as_str());
}

#[test]
fn integration_runtime_observability_endpoint_serves_metrics_and_health_paths() {
    let snapshot = daemon_observability_snapshot();
    let (bind_addr, server) = spawn_observability_server(&snapshot, 4, 2_000);
    assert!(send_http_get(bind_addr.as_str(), "/metrics").contains("HTTP/1.1 200 OK"));
    assert!(send_http_get(bind_addr.as_str(), "/healthz").contains("HTTP/1.1 200 OK"));
    assert!(send_http_get(bind_addr.as_str(), "/readyz").contains("HTTP/1.1 200 OK"));
    assert!(server
        .join()
        .expect("endpoint thread should complete")
        .is_ok());
}
