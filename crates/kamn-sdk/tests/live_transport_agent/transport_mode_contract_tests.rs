use super::support::*;

#[test]
fn unit_live_transport_config_rejects_non_http_endpoint() {
    with_env_lock(|| {
        assert_eq!(
            LiveTransportConfig::new("wss://live.kamn.testnet"),
            Err(SdkError::InvalidInput {
                field: "transport.endpoint",
                reason: "must start with http:// or https://",
            })
        );
    });
}

#[test]
fn spec_c01_live_transport_source_has_no_global_in_memory_registry() {
    with_env_lock(|| {
        let source = include_str!("../../src/live.rs");
        assert!(
            !source.contains("InMemoryKamnClient"),
            "live transport must not proxy through in-memory client simulation"
        );
        assert!(
            !source.contains("OnceLock"),
            "live transport must not use process-global endpoint registry"
        );
    });
}

#[test]
fn regression_live_transport_unreachable_endpoint_fails_closed() {
    with_env_lock(|| {
        ensure_live_test_env();
        let mut client = LiveTransportKamnClient::connect("http://127.0.0.1:1")
            .expect("endpoint format should be accepted");
        let error = client
            .send(Message {
                from: did("unreachable-sender"),
                to: did("unreachable-recipient"),
                body: "payload".to_owned(),
                channel: None,
            })
            .expect_err("send should fail when endpoint is unavailable");
        assert_eq!(
            error,
            SdkError::TransportFailure("failed to connect to service endpoint")
        );
    });
}

#[test]
fn spec_c05_live_transport_remaining_unsupported_methods_fail_closed() {
    with_env_lock(|| {
        ensure_live_test_env();
        let client = LiveTransportKamnClient::connect("http://127.0.0.1:65535")
            .expect("endpoint format should be accepted");
        assert_eq!(client.assert_transport_mode(TransportMode::Live), Ok(()));
    });
}

#[test]
fn regression_transport_mode_mismatch_is_rejected() {
    with_env_lock(|| {
        ensure_live_test_env();
        let live = LiveTransportKamnClient::connect("http://127.0.0.1:65535")
            .expect("connect live should succeed");
        assert_eq!(
            live.assert_transport_mode(TransportMode::InMemory),
            Err(SdkError::TransportModeMismatch {
                expected: "in-memory",
                found: "live",
            })
        );
    });
}
