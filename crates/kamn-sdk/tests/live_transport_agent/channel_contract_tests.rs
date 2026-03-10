use super::support::*;

#[test]
fn spec_c08_live_transport_create_channel_uses_service_route() {
    with_env_lock(|| {
        ensure_live_test_env();
        let bind_addr = reserve_loopback_addr();
        let server_addr = bind_addr.clone();
        let server = thread::spawn(move || {
            run_live_transport_contract_server(server_addr, 1, "kamn:did:agent:live-tester", None)
        });
        wait_for_server_ready(bind_addr.as_str());

        let mut client = LiveTransportKamnClient::connect(format!("http://{bind_addr}").as_str())
            .expect("live client should connect");
        let channel_id = client
            .create_channel("ops-lane")
            .expect("live create_channel should succeed");

        assert_eq!(channel_id.0, "channel-live-ops-lane");

        let server_result = server.join().expect("server thread should join");
        assert!(
            server_result.is_ok(),
            "test service contract server should satisfy request budget"
        );
    });
}

#[test]
fn regression_live_transport_create_channel_rejects_empty_service_channel_id() {
    with_env_lock(|| {
        ensure_live_test_env();
        let bind_addr = reserve_loopback_addr();
        let server_addr = bind_addr.clone();
        let server = thread::spawn(move || {
            run_live_transport_contract_server(server_addr, 1, "kamn:did:agent:live-tester", None)
        });
        wait_for_server_ready(bind_addr.as_str());

        let mut client = LiveTransportKamnClient::connect(format!("http://{bind_addr}").as_str())
            .expect("live client should connect");
        let error = client
            .create_channel("empty-channel")
            .expect_err("empty service channel id must fail closed");

        assert_eq!(
            error,
            SdkError::TransportFailure(
                "service returned empty channel_id in create_channel response",
            )
        );

        let server_result = server.join().expect("server thread should join");
        assert!(
            server_result.is_ok(),
            "test service contract server should satisfy request budget"
        );
    });
}
