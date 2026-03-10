use super::support::*;
use super::*;

#[test]
fn unit_observability_endpoint_rejects_metrics_path_without_leading_slash() {
    let config = ObservabilityEndpointConfig {
        bind_addr: "127.0.0.1:0".to_owned(),
        metrics_path: "metrics".to_owned(),
        health_path: "/healthz".to_owned(),
        max_requests: 1,
        idle_timeout_ms: 1_000,
    };
    let snapshot = sample_observability_snapshot();

    let error = serve_observability_endpoint(&config, &snapshot)
        .expect_err("metrics path without leading slash must fail");
    assert_eq!(error, "observability metrics path must start with '/'");
}

#[test]
fn unit_observability_endpoint_rejects_health_path_without_leading_slash() {
    let config = ObservabilityEndpointConfig {
        bind_addr: "127.0.0.1:0".to_owned(),
        metrics_path: "/metrics".to_owned(),
        health_path: "healthz".to_owned(),
        max_requests: 1,
        idle_timeout_ms: 1_000,
    };
    let snapshot = sample_observability_snapshot();

    let error = serve_observability_endpoint(&config, &snapshot)
        .expect_err("health path without leading slash must fail");
    assert_eq!(error, "observability health path must start with '/'");
}

#[test]
fn unit_observability_endpoint_rejects_zero_request_budget() {
    let config = ObservabilityEndpointConfig {
        bind_addr: "127.0.0.1:0".to_owned(),
        metrics_path: "/metrics".to_owned(),
        health_path: "/healthz".to_owned(),
        max_requests: 0,
        idle_timeout_ms: 1_000,
    };
    let snapshot = sample_observability_snapshot();

    let error = serve_observability_endpoint(&config, &snapshot)
        .expect_err("zero request budget must fail");
    assert_eq!(
        error,
        "observability endpoint max requests must be greater than zero"
    );
}

#[test]
fn unit_observability_endpoint_rejects_zero_idle_timeout_budget() {
    let config = ObservabilityEndpointConfig {
        bind_addr: "127.0.0.1:0".to_owned(),
        metrics_path: "/metrics".to_owned(),
        health_path: "/healthz".to_owned(),
        max_requests: 1,
        idle_timeout_ms: 0,
    };
    let snapshot = sample_observability_snapshot();

    let error =
        serve_observability_endpoint(&config, &snapshot).expect_err("zero idle timeout must fail");
    assert_eq!(
        error,
        "observability endpoint idle timeout must be greater than zero"
    );
}

#[test]
fn unit_observability_endpoint_maps_daemon_telemetry_into_snapshot() {
    let parsed = parse_args_with_clean_daemon_env(vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
        "--runtime-mode".to_owned(),
        "daemon".to_owned(),
        "--daemon-max-ticks".to_owned(),
        "3".to_owned(),
        "--daemon-tick-interval-ms".to_owned(),
        "25".to_owned(),
    ])
    .expect("daemon args should parse");
    let report = execute(parsed).expect("daemon execution should succeed");

    let snapshot =
        build_runtime_observability_snapshot(&report).expect("daemon report should map snapshot");
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
