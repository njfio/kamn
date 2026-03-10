use super::*;

#[path = "tls_transport_contract_tests/support.rs"]
mod support;

use support::*;

#[test]
fn functional_https_transport_submit_with_trusted_ca_succeeds() {
    with_trusted_https_server(|mut server| {
        let outcome = https_provider(server.base_url.as_str(), 2)
            .submit_runtime_commit("operation_id=op-https\n", "idempotency-key-https")
            .expect("https submit should succeed");
        assert_submitted_https_receipt(outcome, "kolme-commit:https");
        server.wait_for_exit();
    });
}

#[test]
fn regression_https_transport_maps_certificate_errors_to_unavailable() {
    with_untrusted_https_server(|mut server| {
        let error = https_provider(server.base_url.as_str(), 2)
            .submit_runtime_commit("operation_id=op-https\n", "idempotency-key-https")
            .expect_err("certificate mismatch must fail");
        assert_eq!(
            error,
            KolmeRuntimeCommitProviderError::Unavailable {
                reason: "tls certificate verification failed".to_owned(),
            }
        );
        server.wait_for_exit();
    });
}

#[test]
fn regression_https_transport_maps_tls_handshake_failures_to_unavailable() {
    with_tls_env_none(|| {
        let addr = spawn_plain_http_over_tls_socket(Duration::from_secs(0));
        let error = https_provider(format!("https://{addr}").as_str(), 2)
            .submit_runtime_commit("operation_id=op-https\n", "idempotency-key-https")
            .expect_err("handshake failure must map to unavailable");
        assert_eq!(
            error,
            KolmeRuntimeCommitProviderError::Unavailable {
                reason: "tls handshake failed".to_owned(),
            }
        );
    });
}

#[test]
fn performance_https_transport_timeout_budget_is_bounded() {
    with_tls_env_none(|| {
        let addr = spawn_plain_http_over_tls_socket(Duration::from_secs(3));
        let started = Instant::now();
        let result = https_provider(format!("https://{addr}").as_str(), 1)
            .submit_runtime_commit("operation_id=op-https\n", "idempotency-key-https");
        let elapsed = started.elapsed();

        assert!(elapsed <= Duration::from_secs(2), "native HTTPS timeout handling exceeded 2s fast-gate budget window: {elapsed:?}");
        assert!(
            matches!(
                result,
                Err(KolmeRuntimeCommitProviderError::Timeout)
                    | Err(KolmeRuntimeCommitProviderError::Unavailable { .. })
            ),
            "slow TLS endpoint should fail closed within timeout budget"
        );
    });
}

#[test]
fn regression_https_transport_does_not_use_openssl_subprocess() {
    assert!(!transport_source().contains("Command::new(\"openssl\")"));
    assert!(!transport_source().contains(".arg(\"s_client\")"));
    assert!(!transport_source().contains("Command::new("));
    assert!(!transport_source().contains("curl"));
    assert!(tls_adr_source().contains("Subprocess TLS paths (`curl`, `openssl s_client`) are not allowed"));
    assert!(tls_adr_source().contains("Regression: #4105"));
}
