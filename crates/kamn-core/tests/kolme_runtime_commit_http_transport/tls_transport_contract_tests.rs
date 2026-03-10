use super::*;
#[test]
fn functional_https_transport_submit_with_trusted_ca_succeeds() {
    let _guard = tls_env_lock()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let mut server = spawn_https_single_request_server(
        200,
        "status=submitted\nprovider=kolme-local\ncommit_id=kolme-commit:https\nfinality=final\n",
    );
    let ca_cert_path = server
        .ca_cert_path
        .to_str()
        .expect("temporary cert path should be valid utf-8")
        .to_owned();
    let _env_guard = EnvVarGuard::set(TLS_CA_FILE_ENV, Some(ca_cert_path.as_str()));

    let transport = KolmeRuntimeCommitHttpTransport::new(2).expect("transport should build");
    let mut provider = KolmeRuntimeCommitLiveProvider::new(
        server.base_url.as_str(),
        "/broadcast/runtime-commit",
        transport,
    )
    .expect("provider should build");

    let outcome = provider
        .submit_runtime_commit("operation_id=op-https\n", "idempotency-key-https")
        .expect("https submit should succeed");
    match outcome {
        KolmeRuntimeCommitProviderOutcome::Submitted(receipt) => {
            assert_eq!(receipt.provider, "kolme-local");
            assert_eq!(receipt.commit_id, "kolme-commit:https");
            assert_eq!(receipt.finality, KolmeCommitReceiptFinality::Final);
        }
        other => panic!("unexpected provider outcome: {other:?}"),
    }

    server.wait_for_exit();
}

#[test]
fn regression_https_transport_maps_certificate_errors_to_unavailable() {
    let _guard = tls_env_lock()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let mut server = spawn_https_single_request_server(
        200,
        "status=submitted\nprovider=kolme-local\ncommit_id=kolme-commit:https\nfinality=final\n",
    );
    let _env_guard = EnvVarGuard::set(TLS_CA_FILE_ENV, None);

    let transport = KolmeRuntimeCommitHttpTransport::new(2).expect("transport should build");
    let mut provider = KolmeRuntimeCommitLiveProvider::new(
        server.base_url.as_str(),
        "/broadcast/runtime-commit",
        transport,
    )
    .expect("provider should build");

    assert_eq!(
        provider.submit_runtime_commit("operation_id=op-https\n", "idempotency-key-https"),
        Err(KolmeRuntimeCommitProviderError::Unavailable {
            reason: "tls certificate verification failed".to_owned(),
        })
    );

    server.wait_for_exit();
}

#[test]
fn regression_https_transport_maps_tls_handshake_failures_to_unavailable() {
    let _guard = tls_env_lock()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let _env_guard = EnvVarGuard::set(TLS_CA_FILE_ENV, None);
    let listener = TcpListener::bind("127.0.0.1:0").expect("listener should bind");
    let addr = listener.local_addr().expect("local addr should resolve");
    thread::spawn(move || {
        let (mut stream, _) = listener.accept().expect("connection should be accepted");
        let _ = stream.read(&mut [0_u8; 64]);
        let _ =
            stream.write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 0\r\nConnection: close\r\n\r\n");
    });

    let transport = KolmeRuntimeCommitHttpTransport::new(2).expect("transport should build");
    let mut provider = KolmeRuntimeCommitLiveProvider::new(
        format!("https://{addr}").as_str(),
        "/broadcast/runtime-commit",
        transport,
    )
    .expect("provider should build");

    assert_eq!(
        provider.submit_runtime_commit("operation_id=op-https\n", "idempotency-key-https"),
        Err(KolmeRuntimeCommitProviderError::Unavailable {
            reason: "tls handshake failed".to_owned(),
        })
    );
}

#[test]
fn performance_https_transport_timeout_budget_is_bounded() {
    let _guard = tls_env_lock()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let _env_guard = EnvVarGuard::set(TLS_CA_FILE_ENV, None);
    let listener = TcpListener::bind("127.0.0.1:0").expect("listener should bind");
    let addr = listener.local_addr().expect("local addr should resolve");
    thread::spawn(move || {
        let (_stream, _) = listener.accept().expect("connection should be accepted");
        thread::sleep(Duration::from_secs(3));
    });

    let transport = KolmeRuntimeCommitHttpTransport::new(1).expect("transport should build");
    let mut provider = KolmeRuntimeCommitLiveProvider::new(
        format!("https://{addr}").as_str(),
        "/broadcast/runtime-commit",
        transport,
    )
    .expect("provider should build");

    let started = Instant::now();
    let result = provider.submit_runtime_commit("operation_id=op-https\n", "idempotency-key-https");
    let elapsed = started.elapsed();

    assert!(
        elapsed <= Duration::from_secs(2),
        "native HTTPS timeout handling exceeded 2s fast-gate budget window: {elapsed:?}"
    );
    assert!(
        matches!(
            result,
            Err(KolmeRuntimeCommitProviderError::Timeout)
                | Err(KolmeRuntimeCommitProviderError::Unavailable { .. })
        ),
        "slow TLS endpoint should fail closed within timeout budget"
    );
}

#[test]
fn regression_https_transport_does_not_use_openssl_subprocess() {
    // Regression: #2671
    const TRANSPORT_SOURCE: &str =
        include_str!("../../src/kolme_runtime_commit/http_transport.rs");
    const TLS_ADR_SOURCE: &str =
        include_str!("../../../../docs/architecture/adr-kamn-core-live-tls-transport.md");
    assert!(
        !TRANSPORT_SOURCE.contains("Command::new(\"openssl\")"),
        "HTTPS transport must not spawn openssl subprocesses"
    );
    assert!(
        !TRANSPORT_SOURCE.contains(".arg(\"s_client\")"),
        "HTTPS transport must not depend on openssl s_client subprocess path"
    );
    assert!(
        !TRANSPORT_SOURCE.contains("Command::new("),
        "HTTPS transport must not spawn subprocess commands in runtime request paths"
    );
    assert!(
        !TRANSPORT_SOURCE.contains("curl"),
        "HTTPS transport runtime request paths must not depend on curl subprocess fallback"
    );
    assert!(
        TLS_ADR_SOURCE
            .contains("Subprocess TLS paths (`curl`, `openssl s_client`) are not allowed"),
        "live TLS ADR must document subprocess fallback prohibition"
    );
    assert!(
        TLS_ADR_SOURCE.contains("Regression: #4105"),
        "live TLS ADR must include regression marker for subprocess fallback prohibition"
    );
}
