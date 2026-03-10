use super::*;

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
