use super::super::support::*;

#[test]
fn regression_libp2p_native_runtime_config_error_reason_code_stays_stable() {
    let config_error = build_p2p_swarm_deterministic_config(
        &config_for(NodeRole::Processor, true),
        "peer-native-invalid-regression",
        "/ip4/127.0.0.1/tcp/invalid-port",
        vec!["/ip4/127.0.0.1/tcp/9201".to_owned()],
        vec!["messages".to_owned()],
        3,
    )
    .expect("base config should build");
    let error = Libp2pLivePeerLifecycleTransport::new(config_error, P2pSwarmHarnessMode::DryRun)
        .expect_err("invalid config should fail");
    assert_eq!(error.reason_code(), "p2p_transport_libp2p_runtime_config_invalid");
}

#[test]
fn regression_libp2p_native_adapter_partition_publish_drop_reason_code_stays_stable() {
    let (sender_transport, recipient_transport) = reason_regression_pair();
    advertise_messages(&sender_transport, "peer-native-reason-regression-sender", NodeRole::Processor);
    advertise_messages(&recipient_transport, "peer-native-reason-regression-recipient", NodeRole::Listener);
    let error = sender_transport
        .send(message_frame(
            "peer-native-reason-regression-sender",
            "peer-native-reason-regression-recipient",
            "tx-native-reason-regression",
        ))
        .expect_err("partitioned publish must fail closed");
    assert_eq!(error, P2pTransportError::LiveSocketSendFailed);
    assert_eq!(error.reason_code(), "p2p_transport_live_socket_send_failed");
}

fn reason_regression_pair() -> (
    Libp2pLivePeerLifecycleTransport,
    Libp2pLivePeerLifecycleTransport,
) {
    disconnected_transport_pair(
        "peer-native-reason-regression-sender",
        "peer-native-reason-regression-recipient",
    )
}
