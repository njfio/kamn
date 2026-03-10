use super::*;
use std::net::TcpListener;
use std::thread;

#[path = "support/env_support.rs"]
mod env_support;
#[path = "support/tls_support.rs"]
mod tls_support;
#[path = "support/transport_support.rs"]
mod transport_support;

pub(super) use env_support::parse_args_with_clean_daemon_env;
pub(super) use tls_support::{
    observability_tls_temp_path, send_https_get, set_tls_mode_override_for_current_thread,
    wait_for_https_endpoint_ready,
};
pub(super) use transport_support::{
    send_http_get, send_raw_http_request, try_send_http_get, wait_for_endpoint_ready,
};

fn daemon_args() -> Vec<String> {
    vec![
        "kamn-node".to_owned(),
        "--role".to_owned(),
        "processor".to_owned(),
        "--runtime-mode".to_owned(),
        "daemon".to_owned(),
        "--daemon-max-ticks".to_owned(),
        "3".to_owned(),
        "--daemon-tick-interval-ms".to_owned(),
        "25".to_owned(),
    ]
}

pub(in super::super) fn daemon_observability_snapshot() -> RuntimeObservabilitySnapshot {
    let parsed = parse_args_with_clean_daemon_env(daemon_args()).expect("daemon args should parse");
    let report = execute(parsed).expect("daemon execution should succeed");
    build_runtime_observability_snapshot(&report).expect("daemon report should map snapshot")
}

pub(in super::super) fn daemon_timeout_observability_snapshot() -> RuntimeObservabilitySnapshot {
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
    build_runtime_observability_snapshot(&report).expect("timeout report should map snapshot")
}

pub(in super::super) fn spawn_observability_server(
    snapshot: &RuntimeObservabilitySnapshot,
    max_requests: u64,
    idle_timeout_ms: u64,
) -> (String, thread::JoinHandle<Result<(), String>>) {
    let bind_addr = reserve_loopback_addr();
    let endpoint_config = ObservabilityEndpointConfig {
        bind_addr: bind_addr.clone(),
        metrics_path: "/metrics".to_owned(),
        health_path: "/healthz".to_owned(),
        max_requests,
        idle_timeout_ms,
    };
    let server_snapshot = snapshot.clone();
    let server =
        thread::spawn(move || serve_observability_endpoint(&endpoint_config, &server_snapshot));
    wait_for_endpoint_ready(bind_addr.as_str());
    (bind_addr, server)
}

fn serve_with_tls_override(
    endpoint_config: ObservabilityEndpointConfig,
    snapshot: RuntimeObservabilitySnapshot,
    cert_file: String,
    key_file: String,
) -> Result<(), String> {
    set_observability_endpoint_tls_mode_override_for_current_thread_for_tests(Some(
        ObservabilityEndpointTlsModeOverride::Require {
            cert_file,
            key_file,
        },
    ));
    let result = serve_observability_endpoint(&endpoint_config, &snapshot);
    set_observability_endpoint_tls_mode_override_for_current_thread_for_tests(None);
    result
}

pub(in super::super) fn spawn_tls_observability_server(
    snapshot: &RuntimeObservabilitySnapshot,
    max_requests: u64,
    idle_timeout_ms: u64,
    cert_file: String,
    key_file: String,
) -> (String, thread::JoinHandle<Result<(), String>>) {
    let bind_addr = reserve_loopback_addr();
    let endpoint_config = ObservabilityEndpointConfig {
        bind_addr: bind_addr.clone(),
        metrics_path: "/metrics".to_owned(),
        health_path: "/healthz".to_owned(),
        max_requests,
        idle_timeout_ms,
    };
    let server_snapshot = snapshot.clone();
    let server = thread::spawn(move || {
        serve_with_tls_override(endpoint_config, server_snapshot, cert_file, key_file)
    });
    wait_for_https_endpoint_ready(bind_addr.as_str());
    (bind_addr, server)
}

pub(super) fn reserve_loopback_addr() -> String {
    let listener = TcpListener::bind("127.0.0.1:0").expect("listener should bind");
    let addr = listener.local_addr().expect("local addr should resolve");
    drop(listener);
    addr.to_string()
}

pub(super) fn sample_observability_snapshot() -> RuntimeObservabilitySnapshot {
    RuntimeObservabilitySnapshot {
        source: "daemon".to_owned(),
        runtime_mode: "daemon".to_owned(),
        latency_p50_ms: 25,
        latency_p99_ms: 50,
        throughput_tps: 2_000,
        error_rate_bps: 50,
        availability_bps: 9_990,
        health: "healthy".to_owned(),
        alert_count: 0,
        reason_code: "none".to_owned(),
        transport_checkpoint_failures: 0,
        signer_checkpoint_failures: 0,
        commit_checkpoint_failures: 0,
    }
}
