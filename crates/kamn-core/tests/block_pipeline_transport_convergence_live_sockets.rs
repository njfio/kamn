use kamn_core::{
    build_transport_convergence_evidence_bundle, CanonicalCandidateDecision,
    DeterministicCompetingBranchForkChoiceHook, InMemoryCanonicalCommitStore, NodeRole,
    PeerDiscoveryRecord, PeerGossipFrame, PeerLifecycleTransport,
    TransportConvergenceEvidenceBundle, TransportEventMempoolFeed, TransportFedBlockPipeline,
    UdpPeerLifecycleTransport,
};
use std::time::{Instant, SystemTime, UNIX_EPOCH};

fn unique_network_id(tag: &str) -> String {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock should be monotonic")
        .as_nanos();
    format!("kamn-live-socket-{tag}-{nonce}")
}

fn block_frame_payload(
    block_height: u64,
    producer_role: &str,
    payload_digest: &str,
    transaction_ids: &str,
) -> String {
    format!(
        "block_height={block_height}\nproducer_role={producer_role}\npayload_digest={payload_digest}\ntransaction_ids={transaction_ids}"
    )
}

fn send_candidate_frame<TTransport: PeerLifecycleTransport>(
    transport: &TTransport,
    sender: &str,
    recipient: &str,
    payload: &str,
) {
    transport
        .send(
            PeerGossipFrame::new("kamn/blocks/v1", sender, recipient, payload)
                .expect("candidate frame should build"),
        )
        .expect("candidate frame should send");
}

fn build_pipeline<TTransport: PeerLifecycleTransport + Clone>(
    transport: TTransport,
    recipient: &str,
) -> TransportFedBlockPipeline<
    TransportEventMempoolFeed<TTransport>,
    InMemoryCanonicalCommitStore,
    DeterministicCompetingBranchForkChoiceHook,
> {
    let feed = TransportEventMempoolFeed::new(
        transport,
        recipient,
        Some(vec!["kamn/blocks/v1".to_owned()]),
    )
    .expect("transport event feed should build");
    let store = InMemoryCanonicalCommitStore::default();
    let hook = DeterministicCompetingBranchForkChoiceHook::new();
    TransportFedBlockPipeline::new(true, 1, 1, feed, store, hook).expect("pipeline should build")
}

#[test]
fn unit_live_socket_transport_rejects_empty_network_id() {
    assert!(UdpPeerLifecycleTransport::bind_ephemeral("", "peer-processor").is_err());
}

#[test]
fn functional_live_socket_partition_rejoin_evidence_marks_verified_status() {
    let network_id = unique_network_id("partition-rejoin");
    let sender = "peer-live-sender";
    let recipient = "peer-live-recipient";
    let sender_transport = UdpPeerLifecycleTransport::bind_ephemeral(&network_id, sender)
        .expect("sender transport should build");
    let recipient_transport = UdpPeerLifecycleTransport::bind_ephemeral(&network_id, recipient)
        .expect("recipient transport should build");

    sender_transport
        .advertise(
            PeerDiscoveryRecord::new(
                sender,
                NodeRole::Processor,
                vec!["kamn/blocks/v1".to_owned()],
            )
            .expect("sender record should build"),
        )
        .expect("sender should advertise");
    recipient_transport
        .advertise(
            PeerDiscoveryRecord::new(
                recipient,
                NodeRole::Processor,
                vec!["kamn/blocks/v1".to_owned()],
            )
            .expect("recipient record should build"),
        )
        .expect("recipient should advertise");

    let mut pipeline = build_pipeline(recipient_transport.clone(), recipient);
    let empty_round = pipeline
        .reconcile_transport_candidates()
        .expect("empty partitioned round should reconcile");
    let empty_commits = pipeline
        .list_canonical_commits()
        .expect("commit list should load");
    let evidence: TransportConvergenceEvidenceBundle = build_transport_convergence_evidence_bundle(
        "live-socket-partition-round-empty",
        &empty_round,
        &empty_commits,
    )
    .expect("empty evidence should build");
    assert_eq!(
        evidence.schema_version,
        "kamn.runtime.transport-convergence-evidence.v1"
    );
    assert_eq!(evidence.continuity_status, "verified");
}

#[test]
fn integration_live_socket_partition_rejoin_and_publish_drop_drill_executes_over_udp() {
    let network_id = unique_network_id("integration");
    let sender = "peer-live-sender-int";
    let recipient = "peer-live-recipient-int";
    let sender_transport = UdpPeerLifecycleTransport::bind_ephemeral(&network_id, sender)
        .expect("sender transport should build");
    let recipient_transport = UdpPeerLifecycleTransport::bind_ephemeral(&network_id, recipient)
        .expect("recipient transport should build");

    sender_transport
        .advertise(
            PeerDiscoveryRecord::new(
                sender,
                NodeRole::Processor,
                vec!["kamn/blocks/v1".to_owned()],
            )
            .expect("sender record should build"),
        )
        .expect("sender should advertise");
    recipient_transport
        .advertise(
            PeerDiscoveryRecord::new(
                recipient,
                NodeRole::Processor,
                vec!["kamn/blocks/v1".to_owned()],
            )
            .expect("recipient record should build"),
        )
        .expect("recipient should advertise");

    let mut pipeline = build_pipeline(recipient_transport.clone(), recipient);
    let round_one = pipeline
        .reconcile_transport_candidates()
        .expect("partitioned round should reconcile");
    assert_eq!(round_one.len(), 0);

    send_candidate_frame(
        &sender_transport,
        sender,
        recipient,
        &block_frame_payload(700, "processor", "digest-700", "tx-700"),
    );
    let round_two = pipeline
        .reconcile_transport_candidates()
        .expect("rejoin round should reconcile");
    assert!(matches!(
        round_two[0].decision,
        CanonicalCandidateDecision::Accepted
    ));

    send_candidate_frame(
        &sender_transport,
        sender,
        recipient,
        &block_frame_payload(702, "processor", "digest-702", "tx-702"),
    );
    let round_three = pipeline
        .reconcile_transport_candidates()
        .expect("post-drop round should reconcile");
    assert!(matches!(
        round_three[0].decision,
        CanonicalCandidateDecision::Accepted
    ));
}

#[test]
fn integration_process_isolated_three_node_partition_rejoin_and_publish_drop_convergence_over_udp()
{
    let network_id = unique_network_id("process-isolated-three-node");
    let sender_a = "peer-live-sender-a";
    let sender_b = "peer-live-sender-b";
    let recipient = "peer-live-recipient-three-node";
    let sender_a_transport = UdpPeerLifecycleTransport::bind_ephemeral(&network_id, sender_a)
        .expect("sender A transport should build");
    let sender_b_transport = UdpPeerLifecycleTransport::bind_ephemeral(&network_id, sender_b)
        .expect("sender B transport should build");
    let recipient_transport = UdpPeerLifecycleTransport::bind_ephemeral(&network_id, recipient)
        .expect("recipient transport should build");

    for peer_id in [sender_a, sender_b, recipient] {
        let transport = match peer_id {
            id if id == sender_a => &sender_a_transport,
            id if id == sender_b => &sender_b_transport,
            _ => &recipient_transport,
        };
        transport
            .advertise(
                PeerDiscoveryRecord::new(
                    peer_id,
                    NodeRole::Processor,
                    vec!["kamn/blocks/v1".to_owned()],
                )
                .expect("discovery record should build"),
            )
            .expect("peer should advertise");
    }

    let mut pipeline = build_pipeline(recipient_transport.clone(), recipient);

    let partition_round = pipeline
        .reconcile_transport_candidates()
        .expect("partition round should reconcile");
    assert!(partition_round.is_empty());
    let partition_evidence = build_transport_convergence_evidence_bundle(
        "process-isolated-three-node-partition",
        &partition_round,
        &pipeline
            .list_canonical_commits()
            .expect("partition commit list should load"),
    )
    .expect("partition evidence should build");
    assert_eq!(partition_evidence.continuity_status, "verified");

    send_candidate_frame(
        &sender_a_transport,
        sender_a,
        recipient,
        &block_frame_payload(900, "processor", "digest-900", "tx-900"),
    );
    send_candidate_frame(
        &sender_b_transport,
        sender_b,
        recipient,
        &block_frame_payload(901, "processor", "digest-901", "tx-901"),
    );
    let rejoin_round = pipeline
        .reconcile_transport_candidates()
        .expect("rejoin round should reconcile");
    assert!(
        rejoin_round
            .iter()
            .any(|outcome| matches!(outcome.decision, CanonicalCandidateDecision::Accepted)),
        "rejoin round must include at least one accepted candidate"
    );
    let rejoin_evidence = build_transport_convergence_evidence_bundle(
        "process-isolated-three-node-rejoin",
        &rejoin_round,
        &pipeline
            .list_canonical_commits()
            .expect("rejoin commit list should load"),
    )
    .expect("rejoin evidence should build");
    assert_eq!(
        rejoin_evidence.schema_version,
        "kamn.runtime.transport-convergence-evidence.v1"
    );
    assert_eq!(rejoin_evidence.continuity_status, "verified");

    send_candidate_frame(
        &sender_a_transport,
        sender_a,
        recipient,
        &block_frame_payload(903, "processor", "digest-903", "tx-903"),
    );
    let post_drop_round = pipeline
        .reconcile_transport_candidates()
        .expect("post-drop round should reconcile");
    assert!(
        post_drop_round
            .iter()
            .any(|outcome| matches!(outcome.decision, CanonicalCandidateDecision::Accepted)),
        "post-drop round must include an accepted recovery candidate"
    );

    send_candidate_frame(
        &sender_b_transport,
        sender_b,
        recipient,
        &block_frame_payload(902, "processor", "digest-902", "tx-902"),
    );
    let delayed_round = pipeline
        .reconcile_transport_candidates()
        .expect("delayed publish round should reconcile");
    assert!(
        delayed_round.iter().any(|outcome| matches!(
            outcome.decision,
            CanonicalCandidateDecision::Rejected { ref reason_code }
                if reason_code == "fork_choice_stale_block_height"
        )),
        "delayed publish must fail closed with stale block-height reason code"
    );
    let delayed_evidence = build_transport_convergence_evidence_bundle(
        "process-isolated-three-node-publish-drop",
        &delayed_round,
        &pipeline
            .list_canonical_commits()
            .expect("publish-drop commit list should load"),
    )
    .expect("publish-drop evidence should build");
    assert_eq!(delayed_evidence.continuity_status, "verified");
}

#[test]
fn regression_live_socket_delayed_publish_emits_stale_reason_code() {
    // Regression: #3652
    let network_id = unique_network_id("regression");
    let sender = "peer-live-sender-reg";
    let recipient = "peer-live-recipient-reg";
    let sender_transport = UdpPeerLifecycleTransport::bind_ephemeral(&network_id, sender)
        .expect("sender transport should build");
    let recipient_transport = UdpPeerLifecycleTransport::bind_ephemeral(&network_id, recipient)
        .expect("recipient transport should build");

    sender_transport
        .advertise(
            PeerDiscoveryRecord::new(
                sender,
                NodeRole::Processor,
                vec!["kamn/blocks/v1".to_owned()],
            )
            .expect("sender record should build"),
        )
        .expect("sender should advertise");
    recipient_transport
        .advertise(
            PeerDiscoveryRecord::new(
                recipient,
                NodeRole::Processor,
                vec!["kamn/blocks/v1".to_owned()],
            )
            .expect("recipient record should build"),
        )
        .expect("recipient should advertise");

    let mut pipeline = build_pipeline(recipient_transport, recipient);

    send_candidate_frame(
        &sender_transport,
        sender,
        recipient,
        &block_frame_payload(800, "processor", "digest-800", "tx-800"),
    );
    pipeline
        .reconcile_transport_candidates()
        .expect("baseline publish should reconcile");

    send_candidate_frame(
        &sender_transport,
        sender,
        recipient,
        &block_frame_payload(802, "processor", "digest-802", "tx-802"),
    );
    pipeline
        .reconcile_transport_candidates()
        .expect("post-drop publish should reconcile");

    send_candidate_frame(
        &sender_transport,
        sender,
        recipient,
        &block_frame_payload(801, "processor", "digest-801", "tx-801"),
    );
    let delayed = pipeline
        .reconcile_transport_candidates()
        .expect("delayed publish should reconcile");

    assert!(matches!(
        delayed[0].decision,
        CanonicalCandidateDecision::Rejected { ref reason_code }
            if reason_code == "fork_choice_stale_block_height"
    ));
}

#[test]
fn performance_live_socket_convergence_drill_stays_within_local_budget() {
    let started = Instant::now();
    let network_id = unique_network_id("performance");
    let sender = "peer-live-sender-perf";
    let recipient = "peer-live-recipient-perf";
    let sender_transport = UdpPeerLifecycleTransport::bind_ephemeral(&network_id, sender)
        .expect("sender transport should build");
    let recipient_transport = UdpPeerLifecycleTransport::bind_ephemeral(&network_id, recipient)
        .expect("recipient transport should build");

    sender_transport
        .advertise(
            PeerDiscoveryRecord::new(
                sender,
                NodeRole::Processor,
                vec!["kamn/blocks/v1".to_owned()],
            )
            .expect("sender record should build"),
        )
        .expect("sender should advertise");
    recipient_transport
        .advertise(
            PeerDiscoveryRecord::new(
                recipient,
                NodeRole::Processor,
                vec!["kamn/blocks/v1".to_owned()],
            )
            .expect("recipient record should build"),
        )
        .expect("recipient should advertise");

    let mut pipeline = build_pipeline(recipient_transport, recipient);
    for height in 1..=48 {
        send_candidate_frame(
            &sender_transport,
            sender,
            recipient,
            &block_frame_payload(
                height,
                "processor",
                &format!("digest-{height}"),
                &format!("tx-{height}"),
            ),
        );
        let outcomes = pipeline
            .reconcile_transport_candidates()
            .expect("performance publish should reconcile");
        assert!(matches!(
            outcomes[0].decision,
            CanonicalCandidateDecision::Accepted
        ));
    }

    let commits = pipeline
        .list_canonical_commits()
        .expect("commits should load");
    assert_eq!(commits.len(), 48);
    assert!(
        started.elapsed() <= std::time::Duration::from_secs(2),
        "live socket convergence drill exceeded local runtime budget"
    );
}
