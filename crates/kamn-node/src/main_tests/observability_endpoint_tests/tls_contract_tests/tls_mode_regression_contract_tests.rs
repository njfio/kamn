use super::super::support::*;
use super::super::*;

fn kolme_live_snapshot() -> RuntimeObservabilitySnapshot {
    let mut snapshot = sample_observability_snapshot();
    snapshot.source = "kolme-live".to_owned();
    snapshot.runtime_mode = "kolme-live".to_owned();
    snapshot
}

#[test]
fn regression_runtime_observability_endpoint_tls_mode_defaults_to_require_for_kolme_live() {
    let _env_lock = daemon_test_env_lock()
        .lock()
        .unwrap_or_else(|error| error.into_inner());
    let _tls_mode_guard = EnvVarGuard::set("KAMN_OBSERVABILITY_ENDPOINT_TLS_MODE", None);
    let _tls_cert_guard = EnvVarGuard::set("KAMN_OBSERVABILITY_ENDPOINT_TLS_CERT_FILE", None);
    let _tls_key_guard = EnvVarGuard::set("KAMN_OBSERVABILITY_ENDPOINT_TLS_KEY_FILE", None);

    let snapshot = kolme_live_snapshot();
    let endpoint_config = ObservabilityEndpointConfig {
        bind_addr: reserve_loopback_addr(),
        metrics_path: "/metrics".to_owned(),
        health_path: "/healthz".to_owned(),
        max_requests: 1,
        idle_timeout_ms: 50,
    };

    let error = serve_observability_endpoint(&endpoint_config, &snapshot)
        .expect_err("kolme-live default tls mode must fail closed without cert/key env");
    assert!(error.contains(
        "observability endpoint tls mode requires env: KAMN_OBSERVABILITY_ENDPOINT_TLS_CERT_FILE"
    ));
}

#[test]
fn integration_runtime_observability_endpoint_tls_mode_allows_explicit_disabled_override_for_kolme_live(
) {
    let _env_lock = daemon_test_env_lock()
        .lock()
        .unwrap_or_else(|error| error.into_inner());
    let _tls_mode_guard =
        EnvVarGuard::set("KAMN_OBSERVABILITY_ENDPOINT_TLS_MODE", Some("disabled"));
    let _tls_cert_guard = EnvVarGuard::set("KAMN_OBSERVABILITY_ENDPOINT_TLS_CERT_FILE", None);
    let _tls_key_guard = EnvVarGuard::set("KAMN_OBSERVABILITY_ENDPOINT_TLS_KEY_FILE", None);

    let snapshot = kolme_live_snapshot();
    let (bind_addr, server) = spawn_observability_server(&snapshot, 2, 2_000);
    assert!(send_http_get(bind_addr.as_str(), "/metrics").contains("HTTP/1.1 200 OK"));
    assert!(server
        .join()
        .expect("endpoint thread should complete")
        .is_ok());
}

#[test]
fn regression_runtime_observability_endpoint_tls_mode_rejects_missing_cert_file() {
    let endpoint_config = ObservabilityEndpointConfig {
        bind_addr: reserve_loopback_addr(),
        metrics_path: "/metrics".to_owned(),
        health_path: "/healthz".to_owned(),
        max_requests: 1,
        idle_timeout_ms: 2_000,
    };
    let _tls_override =
        set_tls_mode_override_for_current_thread(ObservabilityEndpointTlsModeOverride::Require {
            cert_file: observability_tls_temp_path("missing-cert"),
            key_file: observability_tls_temp_path("missing-key"),
        });
    let error = serve_observability_endpoint(&endpoint_config, &sample_observability_snapshot())
        .expect_err("missing observability tls cert should fail closed");
    assert!(error.contains("observability endpoint tls certificate file read failed"));
}

#[test]
fn regression_runtime_observability_endpoint_tls_mode_rejects_invalid_key_file() {
    let (cert_file, key_file) =
        super::super::super::service_api_endpoint_tests::write_test_service_api_tls_materials();
    std::fs::write(
        key_file.as_str(),
        b"invalid-observability-tls-private-key\n",
    )
    .expect("invalid observability tls key material should write");
    let endpoint_config = ObservabilityEndpointConfig {
        bind_addr: reserve_loopback_addr(),
        metrics_path: "/metrics".to_owned(),
        health_path: "/healthz".to_owned(),
        max_requests: 1,
        idle_timeout_ms: 2_000,
    };
    let _tls_override =
        set_tls_mode_override_for_current_thread(ObservabilityEndpointTlsModeOverride::Require {
            cert_file,
            key_file,
        });
    let error = serve_observability_endpoint(&endpoint_config, &sample_observability_snapshot())
        .expect_err("invalid observability tls key should fail closed");
    assert!(error.contains("observability endpoint tls key file parse failed"));
}

#[test]
fn regression_runtime_observability_endpoint_tls_mode_rejects_invalid_mode_value() {
    let endpoint_config = ObservabilityEndpointConfig {
        bind_addr: reserve_loopback_addr(),
        metrics_path: "/metrics".to_owned(),
        health_path: "/healthz".to_owned(),
        max_requests: 1,
        idle_timeout_ms: 2_000,
    };
    let _tls_override = set_tls_mode_override_for_current_thread(
        ObservabilityEndpointTlsModeOverride::InvalidMode {
            mode: "invalid-mode".to_owned(),
        },
    );
    let error = serve_observability_endpoint(&endpoint_config, &sample_observability_snapshot())
        .expect_err("invalid observability tls mode should fail closed");
    assert!(error.contains("observability endpoint tls mode is invalid: invalid-mode"));
}
