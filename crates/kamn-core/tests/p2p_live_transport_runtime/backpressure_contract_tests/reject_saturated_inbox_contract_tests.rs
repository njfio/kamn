use super::*;

#[cfg(not(feature = "libp2p-live-transport"))]
#[test]
fn functional_live_transport_dispatch_backpressure_rejects_saturated_inbox() {
    let bootstrap_seed = unique_tcp_listen_address();
    let sender_peer_id = "peer-dispatch-backpressure-sender";
    let recipient_peer_id = "peer-dispatch-backpressure-recipient";
    let sender_transport = Libp2pLivePeerLifecycleTransport::new(
        live_swarm_config_for_peer(
            sender_peer_id,
            unique_tcp_listen_address().as_str(),
            bootstrap_seed.as_str(),
        ),
        P2pSwarmHarnessMode::DryRun,
    )
    .expect("sender transport should initialize");
    let recipient_transport = Libp2pLivePeerLifecycleTransport::new(
        live_swarm_config_for_peer(
            recipient_peer_id,
            unique_tcp_listen_address().as_str(),
            bootstrap_seed.as_str(),
        ),
        P2pSwarmHarnessMode::DryRun,
    )
    .expect("recipient transport should initialize");

    sender_transport
        .advertise(
            PeerDiscoveryRecord::new(
                sender_peer_id,
                NodeRole::Processor,
                vec!["messages".to_owned()],
            )
            .expect("sender discovery record should build"),
        )
        .expect("sender advertise should pass");
    recipient_transport
        .advertise(
            PeerDiscoveryRecord::new(
                recipient_peer_id,
                NodeRole::Listener,
                vec!["messages".to_owned()],
            )
            .expect("recipient discovery record should build"),
        )
        .expect("recipient advertise should pass");

    let mut reject_error = None;
    for nonce in 0..512 {
        let frame = PeerGossipFrame::new(
            "messages",
            sender_peer_id,
            recipient_peer_id,
            format!("tx-backpressure-{nonce}").as_str(),
        )
        .expect("frame should build");
        match sender_transport.send(frame) {
            Ok(()) => {}
            Err(error) => {
                reject_error = Some(error);
                break;
            }
        }
    }

    let error = reject_error.expect("dispatch should fail closed once inbox saturates");
    match error {
        P2pTransportError::RuntimeBackpressureRejected { reason_code, .. } => {
            assert_eq!(reason_code, "runtime_backpressure_reject_new_enqueue");
        }
        other => panic!("expected runtime backpressure reject, found {other:?}"),
    }
}
