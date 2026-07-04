use super::super::support::*;

#[test]
fn functional_transport_event_feed_decodes_inbox_frames_into_transactions() {
    ensure_default_signer_key_env();
    let transport = InMemoryPeerLifecycleTransport::default();
    let topic = "kamn/messages/v1";
    let sender = "peer-sender";
    let recipient = "peer-recipient";

    transport
        .advertise(
            PeerDiscoveryRecord::new(sender, NodeRole::Approver, vec![topic.to_owned()])
                .expect("sender discovery record should build"),
        )
        .expect("sender should advertise");
    transport
        .advertise(
            PeerDiscoveryRecord::new(recipient, NodeRole::Processor, vec![topic.to_owned()])
                .expect("recipient discovery record should build"),
        )
        .expect("recipient should advertise");

    let tx =
        BaselineTransaction::signed("tx-live-1", "agent-live", 1, "payload-live", "state:seed");
    let payload =
        encode_transport_candidate_payload(&tx).expect("transport payload should encode correctly");
    transport
        .send(
            PeerGossipFrame::new(topic, sender, recipient, &payload)
                .expect("gossip frame should build"),
        )
        .expect("gossip frame should send");

    let mut feed =
        TransportEventMempoolFeed::new(transport, recipient, Some(vec![topic.to_owned()]))
            .expect("feed should build");
    let drained = feed
        .drain_pending_transactions()
        .expect("feed should decode one transaction");
    assert_eq!(drained, vec![tx]);
}

#[test]
fn regression_transport_event_feed_rejects_malformed_payload() {
    let transport = InMemoryPeerLifecycleTransport::default();
    let topic = "kamn/messages/v1";
    let sender = "peer-alpha";
    let recipient = "peer-beta";

    transport
        .advertise(
            PeerDiscoveryRecord::new(sender, NodeRole::Listener, vec![topic.to_owned()])
                .expect("sender discovery record should build"),
        )
        .expect("sender should advertise");
    transport
        .advertise(
            PeerDiscoveryRecord::new(recipient, NodeRole::Processor, vec![topic.to_owned()])
                .expect("recipient discovery record should build"),
        )
        .expect("recipient should advertise");
    transport
        .send(
            PeerGossipFrame::new(topic, sender, recipient, "txwire:v1|missing|fields")
                .expect("gossip frame should build"),
        )
        .expect("gossip frame should send");

    let mut feed =
        TransportEventMempoolFeed::new(transport, recipient, Some(vec![topic.to_owned()]))
            .expect("feed should build");
    let result = feed.drain_pending_transactions();
    assert!(
        matches!(result, Err(BlockPipelineError::TransportFeed(detail)) if detail.contains("p2p_ingress_payload_line_malformed")),
        "malformed payload should fail with deterministic marker"
    );
}

#[test]
fn regression_transport_event_feed_rejects_topic_mismatch() {
    ensure_default_signer_key_env();
    let transport = InMemoryPeerLifecycleTransport::default();
    let sender = "peer-tx";
    let recipient = "peer-runtime";

    transport
        .advertise(
            PeerDiscoveryRecord::new(sender, NodeRole::Listener, vec!["kamn.tx.v1".to_owned()])
                .expect("sender discovery record should build"),
        )
        .expect("sender should advertise");
    transport
        .advertise(
            PeerDiscoveryRecord::new(
                recipient,
                NodeRole::Processor,
                vec!["kamn.tx.v1".to_owned()],
            )
            .expect("recipient discovery record should build"),
        )
        .expect("recipient should advertise");

    let tx = BaselineTransaction::signed(
        "tx-topic-1",
        "agent-topic",
        1,
        "payload-topic",
        "state:seed",
    );
    let payload =
        encode_transport_candidate_payload(&tx).expect("transport payload should encode correctly");
    transport
        .send(
            PeerGossipFrame::new("kamn.topic.unexpected", sender, recipient, &payload)
                .expect("gossip frame should build"),
        )
        .expect("gossip frame should send");

    let mut feed = TransportEventMempoolFeed::new(
        transport,
        recipient,
        Some(vec!["kamn/messages/v1".to_owned()]),
    )
    .expect("feed should build");
    let result = feed.drain_pending_transactions();
    assert!(
        matches!(result, Err(BlockPipelineError::TransportFeed(detail)) if detail.contains("transport_candidate_topic_mismatch")),
        "topic mismatch should fail with deterministic marker"
    );
}

#[test]
fn functional_transport_event_feed_drains_canonical_candidates_from_block_topic_frames() {
    let transport = InMemoryPeerLifecycleTransport::default();
    let topic = "kamn/blocks/v1";
    let sender = "peer-block-sender";
    let recipient = "peer-block-recipient";

    transport
        .advertise(
            PeerDiscoveryRecord::new(sender, NodeRole::Approver, vec![topic.to_owned()])
                .expect("sender discovery record should build"),
        )
        .expect("sender should advertise");
    transport
        .advertise(
            PeerDiscoveryRecord::new(recipient, NodeRole::Processor, vec![topic.to_owned()])
                .expect("recipient discovery record should build"),
        )
        .expect("recipient should advertise");

    let record = sample_canonical_record(17, "digest-17", "tx-17");
    let payload = encode_transport_canonical_candidate_payload(&record)
        .expect("canonical candidate payload should encode");
    transport
        .send(
            PeerGossipFrame::new(topic, sender, recipient, payload.as_str())
                .expect("gossip frame should build"),
        )
        .expect("gossip frame should send");

    let mut feed =
        TransportEventMempoolFeed::new(transport, recipient, Some(vec![topic.to_owned()]))
            .expect("feed should build");
    let candidates = feed
        .drain_canonical_candidates()
        .expect("candidate drain should decode block frame");
    assert_eq!(candidates, vec![record]);
}
