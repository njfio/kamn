use kamn_core::{
    build_transport_convergence_evidence_bundle, CanonicalCandidateDecision, CanonicalCommitRecord,
    DeterministicCompetingBranchForkChoiceHook, InMemoryCanonicalCommitStore,
    InMemoryPeerLifecycleTransport, NodeRole, PeerDiscoveryRecord, PeerGossipFrame,
    PeerLifecycleTransport, TransportConvergenceEvidenceBundle, TransportEventMempoolFeed,
    TransportFedBlockPipeline,
};

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

fn send_candidate_frame(
    transport: &InMemoryPeerLifecycleTransport,
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

fn build_pipeline(
    transport: InMemoryPeerLifecycleTransport,
    recipient: &str,
) -> TransportFedBlockPipeline<
    TransportEventMempoolFeed<InMemoryPeerLifecycleTransport>,
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
fn integration_partition_rejoin_convergence_evidence_tracks_empty_then_recovered_round() {
    let transport = InMemoryPeerLifecycleTransport::default();
    let sender = "peer-partition-sender";
    let recipient = "peer-partition-recipient";
    transport
        .advertise(
            PeerDiscoveryRecord::new(
                sender,
                NodeRole::Processor,
                vec!["kamn/blocks/v1".to_owned()],
            )
            .expect("sender record should build"),
        )
        .expect("sender should advertise");
    transport
        .advertise(
            PeerDiscoveryRecord::new(
                recipient,
                NodeRole::Processor,
                vec!["kamn/blocks/v1".to_owned()],
            )
            .expect("recipient record should build"),
        )
        .expect("recipient should advertise");

    let mut pipeline = build_pipeline(transport.clone(), recipient);
    let round_one = pipeline
        .reconcile_transport_candidates()
        .expect("empty partitioned round should reconcile");
    let round_one_commits = pipeline
        .list_canonical_commits()
        .expect("commit list should load");
    let round_one_evidence: TransportConvergenceEvidenceBundle =
        build_transport_convergence_evidence_bundle(
            "partition-round-empty",
            &round_one,
            &round_one_commits,
        )
        .expect("empty round evidence should build");
    assert_eq!(round_one_evidence.accepted_candidate_count, 0);
    assert_eq!(round_one_evidence.persisted_commit_count, 0);

    send_candidate_frame(
        &transport,
        sender,
        recipient,
        &block_frame_payload(401, "processor", "digest-401", "tx-401"),
    );
    let round_two = pipeline
        .reconcile_transport_candidates()
        .expect("rejoin round should reconcile");
    let round_two_commits = pipeline
        .list_canonical_commits()
        .expect("commit list should load");
    let round_two_evidence = build_transport_convergence_evidence_bundle(
        "partition-round-rejoin",
        &round_two,
        &round_two_commits,
    )
    .expect("rejoin evidence should build");
    assert_eq!(round_two_evidence.accepted_candidate_count, 1);
    assert_eq!(round_two_evidence.persisted_highest_block_height, Some(401));
}

#[test]
fn regression_publish_drop_and_delayed_delivery_converges_with_stale_reason_code() {
    // Regression: #3579
    let transport = InMemoryPeerLifecycleTransport::default();
    let sender = "peer-drop-sender";
    let recipient = "peer-drop-recipient";
    transport
        .advertise(
            PeerDiscoveryRecord::new(
                sender,
                NodeRole::Processor,
                vec!["kamn/blocks/v1".to_owned()],
            )
            .expect("sender record should build"),
        )
        .expect("sender should advertise");
    transport
        .advertise(
            PeerDiscoveryRecord::new(
                recipient,
                NodeRole::Processor,
                vec!["kamn/blocks/v1".to_owned()],
            )
            .expect("recipient record should build"),
        )
        .expect("recipient should advertise");

    let mut pipeline = build_pipeline(transport.clone(), recipient);

    send_candidate_frame(
        &transport,
        sender,
        recipient,
        &block_frame_payload(500, "processor", "digest-500", "tx-500"),
    );
    let outcomes_a = pipeline
        .reconcile_transport_candidates()
        .expect("first publish should reconcile");
    assert!(matches!(
        outcomes_a[0].decision,
        CanonicalCandidateDecision::Accepted
    ));

    // Simulate publish-drop of height 501 by not sending it, then deliver height 502.
    send_candidate_frame(
        &transport,
        sender,
        recipient,
        &block_frame_payload(502, "processor", "digest-502", "tx-502"),
    );
    let outcomes_b = pipeline
        .reconcile_transport_candidates()
        .expect("post-drop publish should reconcile");
    assert!(matches!(
        outcomes_b[0].decision,
        CanonicalCandidateDecision::Accepted
    ));

    // Delayed delivery of dropped height 501 must fail closed as stale.
    send_candidate_frame(
        &transport,
        sender,
        recipient,
        &block_frame_payload(501, "processor", "digest-501", "tx-501"),
    );
    let delayed = pipeline
        .reconcile_transport_candidates()
        .expect("delayed delivery should reconcile");
    assert!(matches!(
        delayed[0].decision,
        CanonicalCandidateDecision::Rejected { ref reason_code }
        if reason_code == "fork_choice_stale_block_height"
    ));

    let commits = pipeline
        .list_canonical_commits()
        .expect("commit list should load");
    let heights = commits
        .iter()
        .map(|record| record.block_height)
        .collect::<Vec<_>>();
    assert_eq!(heights, vec![500, 502]);

    let mut all_outcomes = Vec::new();
    all_outcomes.extend(outcomes_a);
    all_outcomes.extend(outcomes_b);
    all_outcomes.extend(delayed.clone());
    let evidence = build_transport_convergence_evidence_bundle(
        "publish-drop-delayed",
        &all_outcomes,
        &commits,
    )
    .expect("evidence should build");
    assert_eq!(evidence.accepted_candidate_count, 2);
    assert_eq!(evidence.rejected_candidate_count, 1);
    assert!(evidence
        .rejected_reason_codes
        .contains(&"fork_choice_stale_block_height".to_owned()));
    assert_eq!(evidence.continuity_status, "verified");
}

#[test]
fn unit_transport_convergence_evidence_rejects_empty_case_identifier() {
    let result = build_transport_convergence_evidence_bundle("", &[], &[]);
    assert!(result.is_err(), "empty case id must fail closed");
}

#[test]
fn performance_transport_convergence_evidence_bundle_stays_within_local_budget() {
    let outcomes = (1..=128)
        .map(|height| kamn_core::CanonicalCandidateOutcome {
            block_height: height,
            payload_digest: format!("digest-{height}"),
            decision: CanonicalCandidateDecision::Accepted,
        })
        .collect::<Vec<_>>();
    let commits = (1..=128)
        .map(|height| CanonicalCommitRecord {
            block_height: height,
            producer_role: NodeRole::Processor,
            payload_digest: format!("digest-{height}"),
            transaction_ids: vec![format!("tx-{height}")],
        })
        .collect::<Vec<_>>();

    let started = std::time::Instant::now();
    let evidence = build_transport_convergence_evidence_bundle("performance", &outcomes, &commits)
        .expect("evidence should build");
    assert_eq!(evidence.persisted_commit_count, 128);
    assert!(
        started.elapsed() <= std::time::Duration::from_secs(1),
        "convergence evidence exceeded local performance budget"
    );
}
