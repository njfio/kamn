use super::super::support::*;

#[test]
fn integration_libp2p_native_adapter_three_node_partition_rejoin_and_publish_drop_convergence_over_sockets(
) {
    let (sender_a_transport, sender_b_transport, recipient_transport) = three_node_transports();
    advertise_messages(&sender_a_transport, "peer-native-three-node-sender-a", NodeRole::Processor);
    advertise_messages(&sender_b_transport, "peer-native-three-node-sender-b", NodeRole::Processor);
    assert_partitioned_publish_fails(&sender_b_transport);
    advertise_messages(&recipient_transport, "peer-native-three-node-recipient", NodeRole::Listener);
    assert_recipient_discovered(&sender_a_transport, "peer-native-three-node-sender-a");
    assert_recipient_discovered(&sender_b_transport, "peer-native-three-node-sender-b");
    assert_rejoin_publish_succeeds(&sender_a_transport, "peer-native-three-node-sender-a", "tx-native-three-node-rejoin-a");
    assert_rejoin_publish_succeeds(&sender_b_transport, "peer-native-three-node-sender-b", "tx-native-three-node-rejoin-b");
}

fn three_node_transports(
) -> (
    Libp2pLivePeerLifecycleTransport,
    Libp2pLivePeerLifecycleTransport,
    Libp2pLivePeerLifecycleTransport,
) {
    let (sender_a_listen, sender_b_listen, recipient_listen, bootstrap_peers) =
        three_node_bootstrap();
    let sender_a = build_three_node_sender(
        "peer-native-three-node-sender-a",
        sender_a_listen.as_str(),
        bootstrap_peers.clone(),
    );
    let sender_b = build_three_node_sender(
        "peer-native-three-node-sender-b",
        sender_b_listen.as_str(),
        bootstrap_peers.clone(),
    );
    let recipient = new_transport_with_bootstrap(
        "peer-native-three-node-recipient",
        recipient_listen.as_str(),
        bootstrap_peers,
    );
    (sender_a, sender_b, recipient)
}

fn three_node_bootstrap() -> (String, String, String, Vec<String>) {
    let sender_a_listen = unique_listen_address();
    let sender_b_listen = unique_listen_address();
    let recipient_listen = unique_listen_address();
    let bootstrap_peers = vec![
        sender_a_listen.clone(),
        sender_b_listen.clone(),
        recipient_listen.clone(),
    ];
    (
        sender_a_listen,
        sender_b_listen,
        recipient_listen,
        bootstrap_peers,
    )
}

fn build_three_node_sender(
    peer_id: &str,
    listen_address: &str,
    bootstrap_peers: Vec<String>,
) -> Libp2pLivePeerLifecycleTransport {
    settle_mesh();
    new_transport_with_bootstrap(peer_id, listen_address, bootstrap_peers)
}

fn assert_partitioned_publish_fails(transport: &Libp2pLivePeerLifecycleTransport) {
    let error = transport
        .send(message_frame(
            "peer-native-three-node-sender-b",
            "peer-native-three-node-recipient",
            "tx-native-three-node-partition-publish",
        ))
        .expect_err("partitioned sender must fail closed");
    assert_eq!(
        error,
        P2pTransportError::UnknownRecipientPeer("peer-native-three-node-recipient".to_owned())
    );
    assert_eq!(error.reason_code(), "p2p_transport_unknown_recipient_peer");
}

fn assert_recipient_discovered(
    transport: &Libp2pLivePeerLifecycleTransport,
    sender_id: &str,
) {
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        let discovered = transport.discover(sender_id, "messages").expect("discovery should succeed");
        if discovered.iter().any(|record| record.peer_id == "peer-native-three-node-recipient") {
            return;
        }
        assert!(Instant::now() < deadline, "sender failed to discover recipient within timeout");
        std::thread::sleep(Duration::from_millis(25));
    }
}

fn assert_rejoin_publish_succeeds(
    transport: &Libp2pLivePeerLifecycleTransport,
    sender_id: &str,
    payload: &str,
) {
    let frame = message_frame(sender_id, "peer-native-three-node-recipient", payload);
    send_with_retry(transport, &frame, Duration::from_secs(5))
        .expect("rejoin publish should succeed");
}
