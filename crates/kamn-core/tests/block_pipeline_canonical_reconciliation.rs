use kamn_core::{
    BlockPipelineError, CanonicalCandidateDecision, CanonicalCommitRecord,
    DeterministicCompetingBranchForkChoiceHook, GossipFrameTransportMempoolFeed,
    InMemoryCanonicalCommitStore, TransportFedBlockPipeline,
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

#[test]
fn functional_transport_candidate_reconciliation_persists_accepted_candidates() {
    let frames = vec![kamn_core::PeerGossipFrame::new(
        "kamn/blocks/v1",
        "peer-a",
        "peer-b",
        &block_frame_payload(41, "processor", "digest-41", "tx-41"),
    )
    .expect("frame should build")];
    let feed = GossipFrameTransportMempoolFeed::new(frames);
    let store = InMemoryCanonicalCommitStore::default();
    let hook = DeterministicCompetingBranchForkChoiceHook::new();
    let mut pipeline = TransportFedBlockPipeline::new(true, 1, 1, feed, store, hook)
        .expect("pipeline should build");

    let outcomes = pipeline
        .reconcile_transport_candidates()
        .expect("candidate reconciliation should pass");
    assert_eq!(outcomes.len(), 1);
    assert!(matches!(
        outcomes[0].decision,
        CanonicalCandidateDecision::Accepted
    ));

    let persisted = pipeline
        .list_canonical_commits()
        .expect("commit store should load");
    assert_eq!(
        persisted,
        vec![CanonicalCommitRecord {
            block_height: 41,
            producer_role: kamn_core::NodeRole::Processor,
            payload_digest: "digest-41".to_owned(),
            transaction_ids: vec!["tx-41".to_owned()],
        }]
    );
}

#[test]
fn integration_transport_candidate_reconciliation_emits_reject_reason_code_without_persisting() {
    let frames = vec![
        kamn_core::PeerGossipFrame::new(
            "kamn/blocks/v1",
            "peer-a",
            "peer-b",
            &block_frame_payload(50, "processor", "digest-z", "tx-z"),
        )
        .expect("seed frame should build"),
        kamn_core::PeerGossipFrame::new(
            "kamn/blocks/v1",
            "peer-a",
            "peer-b",
            &block_frame_payload(50, "processor", "digest-zz", "tx-zz"),
        )
        .expect("tie-break loser frame should build"),
    ];
    let feed = GossipFrameTransportMempoolFeed::new(frames);
    let store = InMemoryCanonicalCommitStore::default();
    let hook = DeterministicCompetingBranchForkChoiceHook::new();
    let mut pipeline = TransportFedBlockPipeline::new(true, 1, 1, feed, store, hook)
        .expect("pipeline should build");

    let outcomes = pipeline
        .reconcile_transport_candidates()
        .expect("candidate reconciliation should pass");
    assert_eq!(outcomes.len(), 2);
    assert!(matches!(
        outcomes[0].decision,
        CanonicalCandidateDecision::Accepted
    ));
    assert!(
        matches!(
            &outcomes[1].decision,
            CanonicalCandidateDecision::Rejected { reason_code }
            if reason_code == "fork_choice_tie_break_loser"
        ),
        "reject decision should carry deterministic fork-choice reason code"
    );

    let persisted = pipeline
        .list_canonical_commits()
        .expect("commit store should load");
    assert_eq!(persisted.len(), 1);
    assert_eq!(persisted[0].payload_digest, "digest-z");
}

#[test]
fn regression_transport_candidate_reconciliation_sorts_candidates_before_fork_choice() {
    // Regression: #3416
    let frames = vec![
        kamn_core::PeerGossipFrame::new(
            "kamn/blocks/v1",
            "peer-a",
            "peer-b",
            &block_frame_payload(60, "processor", "digest-b", "tx-b"),
        )
        .expect("higher lexical digest frame should build"),
        kamn_core::PeerGossipFrame::new(
            "kamn/blocks/v1",
            "peer-a",
            "peer-b",
            &block_frame_payload(60, "processor", "digest-a", "tx-a"),
        )
        .expect("lower lexical digest frame should build"),
    ];
    let feed = GossipFrameTransportMempoolFeed::new(frames);
    let store = InMemoryCanonicalCommitStore::default();
    let hook = DeterministicCompetingBranchForkChoiceHook::new();
    let mut pipeline = TransportFedBlockPipeline::new(true, 1, 1, feed, store, hook)
        .expect("pipeline should build");

    let outcomes = pipeline
        .reconcile_transport_candidates()
        .expect("candidate reconciliation should pass");
    assert_eq!(outcomes.len(), 2);
    assert!(matches!(
        &outcomes[0].decision,
        CanonicalCandidateDecision::Accepted
    ));
    assert!(matches!(
        &outcomes[1].decision,
        CanonicalCandidateDecision::Rejected { reason_code }
        if reason_code == "fork_choice_tie_break_loser"
    ));

    let persisted = pipeline
        .list_canonical_commits()
        .expect("commit store should load");
    assert_eq!(persisted.len(), 1);
    assert_eq!(persisted[0].payload_digest, "digest-a");
}

#[test]
fn integration_transport_candidate_reconciliation_preserves_empty_mempool_error_for_consensus_round(
) {
    let frames = vec![kamn_core::PeerGossipFrame::new(
        "kamn/blocks/v1",
        "peer-a",
        "peer-b",
        &block_frame_payload(72, "processor", "digest-72", "tx-72"),
    )
    .expect("frame should build")];
    let feed = GossipFrameTransportMempoolFeed::new(frames);
    let store = InMemoryCanonicalCommitStore::default();
    let hook = DeterministicCompetingBranchForkChoiceHook::new();
    let mut pipeline = TransportFedBlockPipeline::new(true, 1, 1, feed, store, hook)
        .expect("pipeline should build");

    let result = pipeline.run_transport_consensus_round(kamn_core::BlockConsensusRoundInput {
        listener_event_id: "event-1".to_owned(),
        listener_event_sequence: 1,
        outbound_action_id: "outbound-1".to_owned(),
        listener_votes: vec![("kamn:did:listener:alpha".to_owned(), "att-1".to_owned())],
        approver_votes: vec![(
            "kamn:did:agent:approver-alpha".to_owned(),
            "att-1".to_owned(),
            None,
        )],
    });
    assert_eq!(result, Err(BlockPipelineError::EmptyMempool));

    let persisted = pipeline
        .list_canonical_commits()
        .expect("commit store should load");
    assert_eq!(persisted.len(), 1);
    assert_eq!(persisted[0].payload_digest, "digest-72");
}
