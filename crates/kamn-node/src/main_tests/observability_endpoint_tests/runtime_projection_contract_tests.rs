use super::support::*;
use super::*;

fn assert_config_error(config: ObservabilityEndpointConfig, expected: &str) {
    let error = serve_observability_endpoint(&config, &sample_observability_snapshot())
        .expect_err("invalid observability config should fail");
    assert_eq!(error, expected);
}

#[test]
fn unit_observability_endpoint_rejects_metrics_path_without_leading_slash() {
    assert_config_error(
        ObservabilityEndpointConfig {
            bind_addr: "127.0.0.1:0".to_owned(),
            metrics_path: "metrics".to_owned(),
            health_path: "/healthz".to_owned(),
            max_requests: 1,
            idle_timeout_ms: 1_000,
        },
        "observability metrics path must start with '/'",
    );
}

#[test]
fn unit_observability_endpoint_rejects_health_path_without_leading_slash() {
    assert_config_error(
        ObservabilityEndpointConfig {
            bind_addr: "127.0.0.1:0".to_owned(),
            metrics_path: "/metrics".to_owned(),
            health_path: "healthz".to_owned(),
            max_requests: 1,
            idle_timeout_ms: 1_000,
        },
        "observability health path must start with '/'",
    );
}

#[test]
fn unit_observability_endpoint_rejects_zero_request_budget() {
    assert_config_error(
        ObservabilityEndpointConfig {
            bind_addr: "127.0.0.1:0".to_owned(),
            metrics_path: "/metrics".to_owned(),
            health_path: "/healthz".to_owned(),
            max_requests: 0,
            idle_timeout_ms: 1_000,
        },
        "observability endpoint max requests must be greater than zero",
    );
}

#[test]
fn unit_observability_endpoint_rejects_zero_idle_timeout_budget() {
    assert_config_error(
        ObservabilityEndpointConfig {
            bind_addr: "127.0.0.1:0".to_owned(),
            metrics_path: "/metrics".to_owned(),
            health_path: "/healthz".to_owned(),
            max_requests: 1,
            idle_timeout_ms: 0,
        },
        "observability endpoint idle timeout must be greater than zero",
    );
}

#[test]
fn unit_observability_endpoint_maps_daemon_telemetry_into_snapshot() {
    let snapshot = daemon_observability_snapshot();
    assert_eq!(snapshot.source, "daemon");
    assert_eq!(snapshot.runtime_mode, "daemon");
    assert_eq!(snapshot.latency_p50_ms, 1);
    assert_eq!(snapshot.latency_p99_ms, 1);
    assert_eq!(snapshot.throughput_tps, 1_000);
    assert_eq!(snapshot.error_rate_bps, 0);
    assert_eq!(snapshot.availability_bps, 10_000);
    assert_eq!(snapshot.health, "healthy");
    assert_eq!(snapshot.alert_count, 0);
}
