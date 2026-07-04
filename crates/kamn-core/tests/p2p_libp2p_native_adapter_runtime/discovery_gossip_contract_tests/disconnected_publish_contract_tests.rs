use super::super::support::*;

#[test]
fn integration_libp2p_native_adapter_disconnected_publish_fails_closed() {
    let (sender_transport, recipient_transport) = disconnected_pair();
    advertise_messages(
        &sender_transport,
        "peer-native-disconnected-sender",
        NodeRole::Processor,
    );
    advertise_messages(
        &recipient_transport,
        "peer-native-disconnected-recipient",
        NodeRole::Listener,
    );
    let error = sender_transport
        .send(message_frame(
            "peer-native-disconnected-sender",
            "peer-native-disconnected-recipient",
            "tx-native-disconnected-001",
        ))
        .expect_err("disconnected publish must fail closed");
    assert_eq!(error, P2pTransportError::LiveSocketSendFailed);
    assert_eq!(error.reason_code(), "p2p_transport_live_socket_send_failed");
}

fn disconnected_pair() -> (
    Libp2pLivePeerLifecycleTransport,
    Libp2pLivePeerLifecycleTransport,
) {
    disconnected_transport_pair(
        "peer-native-disconnected-sender",
        "peer-native-disconnected-recipient",
    )
}
